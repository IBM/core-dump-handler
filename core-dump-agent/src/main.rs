extern crate dotenv;
extern crate s3;

use advisory_lock::{AdvisoryFileLock, FileLockMode};
use env_logger::Env;
use inotify::{EventMask, Inotify, WatchMask};
use log::{error, info, warn};
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::process::Command;
use std::time::Duration;
use thiserror::Error;
use tokio::runtime::Handle;
use tokio_cron_scheduler::{Job, JobScheduler};

#[allow(dead_code)]
struct Storage {
    name: String,
    region: Region,
    credentials: Credentials,
    bucket: String,
    location_supported: bool,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to set (name {name:?}, value {value:?})")]
    InvalidOverWrite { name: String, value: String },
}

const BIN_PATH: &str = "/bin:/sbin:/usr/bin:/usr/sbin:/usr/local/bin";
const CDC_NAME: &str = "cdc";
static DEFAULT_BASE_DIR: &str = "/var/mnt/core-dump-handler";
static DEFAULT_CORE_DIR: &str = "/var/mnt/core-dump-handler/cores";

static DEFAULT_SUID_DUMPABLE: &str = "2";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut env_path = env::current_exe()?;
    env_path.pop();
    env_path.push(".env");

    let mut envloadmsg = String::from("Loading .env");
    match dotenv::from_path(env_path) {
        Ok(v) => v,
        Err(_) => {
            envloadmsg = "no .env file found \n That's ok if running in kubernetes".to_string()
        }
    }

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let host_dir = env::var("HOST_DIR").unwrap_or_else(|_| DEFAULT_BASE_DIR.to_string());
    let core_dir = env::var("CORE_DIR").unwrap_or_else(|_| DEFAULT_CORE_DIR.to_string());
    let suid = env::var("SUID_DUMPABLE").unwrap_or_else(|_| DEFAULT_SUID_DUMPABLE.to_string());
    let deploy_crio_config = env::var("DEPLOY_CRIO_CONFIG")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase();

    let deploy_crio_exe = env::var("DEPLOY_CRIO_EXE")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase();

    let host_location = host_dir.as_str();
    let pattern: String = std::env::args().nth(1).unwrap_or_default();

    info!("{}", envloadmsg);
    // Catch remove command for uninstall options
    if pattern == "remove" {
        info!("Removing {}", host_location);
        remove()?;
        process::exit(0);
    }

    if pattern == "sweep" {
        let file = std::env::args().nth(2).unwrap_or_default();
        if !file.is_empty() {
            let bucket = match get_bucket() {
                Ok(v) => v,
                Err(e) => {
                    error!("Bucket creation failed in sweep: {}", e);
                    process::exit(1);
                }
            };
            let p = Path::new(&file);
            info!("Uploading {}", file);
            process_file(p, &bucket).await;
        } else {
            let core_store = core_dir.clone();
            info!("Uploading all content in {}", core_store);
            run_polling_agent(core_store.as_str()).await;
        }
        process::exit(0);
    }
    info!("Setting host location to: {}", host_location);
    info!(
        "Current Directory for setup is {}",
        env::current_dir().unwrap().display()
    );

    if deploy_crio_config == "true" {
        generate_crio_config(host_location)?;
    }

    if deploy_crio_exe == "true" {
        copy_crictl_to_hostdir(host_location)?;
    }
    copy_core_dump_composer_to_hostdir(host_location)?;
    apply_sysctl(
        "kernel.core_pattern",
        format!("{}/core_pattern.bak", host_location).as_str(),
        format!(
            "|{}/{} -c=%c -e=%e -p=%p -s=%s -t=%t -d={} -h=%h -E=%E",
            host_location, CDC_NAME, core_dir
        )
        .as_str(),
    )?;
    apply_sysctl(
        "kernel.core_pipe_limit",
        format!("{}/core_pipe_limit.bak", host_location).as_str(),
        "128",
    )?;

    apply_sysctl(
        "fs.suid_dumpable",
        format!("{}/suid_dumpable.bak", host_location).as_str(),
        &suid,
    )?;

    let core_location = core_dir.clone();

    create_env_file(host_location)?;
    // Run polling agent on startup to clean up files.

    let interval = env::var("INTERVAL").unwrap_or_else(|_| String::from(""));
    let mut schedule = env::var("SCHEDULE").unwrap_or_else(|_| String::from(""));

    let use_inotify = env::var("USE_INOTIFY")
        .unwrap_or_else(|_| String::from("false"))
        .to_lowercase();

    // Allow no uploads when none of the upload configs have been supplied
    // then we are leaving the cores on the box
    if interval.is_empty() && schedule.is_empty() && use_inotify == "false" {
        // let it spin
        loop {
            std::thread::sleep(Duration::from_millis(1000));
        }
    } else {
        run_polling_agent(core_location.as_str()).await;
    }

    if !interval.is_empty() && !schedule.is_empty() {
        warn!(
            "Interval set to: {}, Schedule set to:{}\n Using interval:{}",
            interval, schedule, interval
        );
    }

    if !interval.is_empty() {
        let mut i_interval = match interval.parse::<u64>() {
            Ok(v) => v,
            Err(e) => {
                error!("Error parsing interval : {} Error: {}", interval, e);
                panic!("Error parsing interval {}", e);
            }
        };
        i_interval /= 1000;
        schedule = format!("1/{} * * * * *", i_interval);
        if use_inotify == "true" {
            warn!("Both schedule and INotify set. Running schedule")
        }
    }

    let notify_location = core_location.clone();
    if !schedule.is_empty() {
        info!("Schedule Initialising with: {}", schedule);
        let sched = match JobScheduler::new() {
            Ok(v) => v,
            Err(e) => {
                error!("Schedule Creation Failed with {}", e);
                panic!("Schedule Creation Failed with {}", e)
            }
        };
        let s_job = match Job::new(schedule.as_str(), move |_uuid, _l| {
            let handle = Handle::current();
            let core_str = core_location.clone();
            handle.spawn(async move {
                run_polling_agent(&core_str).await;
            });
        }) {
            Ok(v) => v,
            Err(e) => {
                error!("Schedule Job Creation with {} failed, {}", schedule, e);
                panic!("Schedule Job Creation with {} failed, {}", schedule, e)
            }
        };
        info!("Created Schedule job: {:?}", s_job.guid());
        match sched.add(s_job) {
            Ok(v) => v,
            Err(e) => {
                error!("Job Add failed {:#?}", e);
                panic!("Job Scheduing failed, {:#?}", e);
            }
        };
        match sched.start() {
            Ok(v) => v,
            Err(e) => {
                error!("Schedule Start failed {:#?}", e);
                panic!("Schedule Start failed, {:#?}", e);
            }
        };
    }

    if use_inotify == "true" {
        info!("INotify Starting...");
        let inotify_task = tokio::spawn(async move {
            let mut inotify = match Inotify::init() {
                Ok(v) => v,
                Err(e) => {
                    error!("Inotify init failed: {}", e);
                    panic!("Inotify init failed: {}", e)
                }
            };
            info!("INotify Initialised...");
            match inotify.add_watch(&notify_location, WatchMask::CLOSE) {
                Ok(_) => {}
                Err(e) => {
                    error!("Add watch failed: {}", e);
                    panic!("Add watch failed: {}", e)
                }
            };
            info!("INotify watching : {}", notify_location);
            let mut buffer = [0; 4096];
            loop {
                let events = match inotify.read_events_blocking(&mut buffer) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("read events failed: {}", e);
                        continue;
                    }
                };
                for event in events {
                    if event.mask.contains(EventMask::CLOSE_WRITE) {
                        if event.mask.contains(EventMask::ISDIR) {
                            warn!("Unknown Directory created: {:?}", event.name);
                        } else {
                            let bucket = match get_bucket() {
                                Ok(v) => v,
                                Err(e) => {
                                    error!("Bucket creation failed in event: {}", e);
                                    continue;
                                }
                            };
                            match event.name {
                                Some(s) => {
                                    let file = format!(
                                        "{}/{}",
                                        notify_location,
                                        s.to_str().unwrap_or_default()
                                    );
                                    let p = Path::new(&file);
                                    process_file(p, &bucket).await
                                }
                                None => {
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
        });
        inotify_task.await?;
    }

    Ok(())
}

async fn process_file(zip_path: &Path, bucket: &Bucket) {
    info!("Uploading: {}", zip_path.display());

    let f = File::open(&zip_path).expect("no file found");

    match f.try_lock(FileLockMode::Shared) {
        Ok(_) => { /* If we can lock then we are ok */ }
        Err(e) => {
            let l_inotify = env::var("USE_INOTIFY")
                .unwrap_or_else(|_| String::from("false"))
                .to_lowercase();
            if l_inotify == "false" {
                info!("File locked so we are ignoring it for this iteration {}", e);
            } else {
                error!("File locked on INotify shouldn't happen as we are waiting for file close events.\nPlease recycling pod to perform sweep\n{}", e);
            }
            return;
        }
    }

    let metadata = fs::metadata(&zip_path).expect("unable to read metadata");
    info!("zip size is {}", metadata.len());
    let path_str = match zip_path.to_str() {
        Some(v) => v,
        None => {
            error!("Failed to extract path");
            return;
        }
    };
    let upload_file_name: &str = match zip_path.file_name().unwrap().to_str() {
        Some(v) => v,
        None => {
            error!("Failed to get file name for upload");
            return;
        }
    };

    let mut fasync = tokio::fs::File::open(zip_path)
        .await
        .expect("file was removed");

    let code = match bucket
        .put_object_stream(&mut fasync, upload_file_name)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            error!("Upload Failed {}", e);
            return;
        }
    };
    match fs::remove_file(path_str) {
        Ok(v) => v,
        Err(e) => {
            error!("File delete failed: {}", e);
            return;
        }
    };
    info!("S3 Returned: {}", code);
}

fn get_bucket() -> Result<Bucket, anyhow::Error> {
    let s3_access_key = env::var("S3_ACCESS_KEY").unwrap_or_default();
    let s3_secret = env::var("S3_SECRET").unwrap_or_default();
    let s3_bucket_name = env::var("S3_BUCKET_NAME").unwrap_or_default();
    let s3_region = env::var("S3_REGION").unwrap_or_default();

    let custom_endpoint = env::var("S3_ENDPOINT").unwrap_or_default();

    let region = if custom_endpoint.is_empty() {
        s3_region.parse().unwrap()
    } else {
        info!("Setting s3 endpoint location to: {}", custom_endpoint);

        Region::Custom {
            region: s3_region,
            endpoint: custom_endpoint,
        }
    };

    let credentials = if env::var("AWS_WEB_IDENTITY_TOKEN_FILE").is_ok() {
        Credentials::from_sts_env(std::env!("CARGO_PKG_NAME"))
    } else if s3_access_key.is_empty() || s3_secret.is_empty() {
        Credentials::new(None, None, None, None, None)
    } else {
        Credentials::new(
            Some(s3_access_key.as_str()),
            Some(s3_secret.as_str()),
            None,
            None,
            None,
        )
    }?;

    let s3 = Storage {
        name: "aws".into(),
        region,
        credentials,
        bucket: s3_bucket_name,
        location_supported: false,
    };
    Ok(Bucket::new(&s3.bucket, s3.region, s3.credentials)?.with_path_style())
}

async fn run_polling_agent(core_location: &str) {
    info!("Executing Agent with location : {}", core_location);

    let bucket = match get_bucket() {
        Ok(v) => v,
        Err(e) => {
            error!("Bucket Creation Failed: {}", e);
            return;
        }
    };

    let dir = fs::read_dir(core_location).unwrap();

    let paths: Vec<PathBuf> = dir
        .into_iter()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap().path())
        .filter(|r| !r.is_dir())
        .collect();

    info!("Dir Content {:?}", paths);
    for zip_path in paths {
        process_file(&zip_path, &bucket).await;
    }
}

fn generate_crio_config(host_location: &str) -> Result<(), std::io::Error> {
    info!("Generating crio file");
    let endpoint = env::var("CRIO_ENDPOINT")
        .unwrap_or_else(|_| "unix:///run/containerd/containerd.sock".to_string());
    let destination = format!("{}/{}", host_location, "crictl.yaml");
    let mut crictl_file = File::create(destination)?;
    let text = format!("runtime-endpoint: {}\nimage-endpoint: {}\ntimeout: 2\ndebug: false\npull-image-on-create: false", endpoint, endpoint);
    crictl_file.write_all(text.as_bytes())?;
    crictl_file.flush()?;
    Ok(())
}

fn copy_crictl_to_hostdir(host_location: &str) -> Result<(), std::io::Error> {
    let location = "./crictl".to_string();
    let destination = format!("{}/{}", host_location, "crictl");
    info!("Copying the crictl from {} to {}", location, destination);
    fs::copy(location, destination)?;
    Ok(())
}

fn copy_core_dump_composer_to_hostdir(host_location: &str) -> Result<(), std::io::Error> {
    let version = env::var("VENDOR").unwrap_or_else(|_| "default".to_string());
    match version.to_lowercase().as_str() {
        "default" => {
            let location = format!("./vendor/default/{}", CDC_NAME);
            let destination = format!("{}/{}", host_location, CDC_NAME);
            info!("Copying the composer from {} to {}", location, destination);
            fs::copy(location, destination)?;
        }
        "rhel7" => {
            let location = format!("./vendor/rhel7/{}", CDC_NAME);
            let destination = format!("{}/{}", host_location, CDC_NAME);
            info!("Copying the composer from {} to {}", location, destination);
            fs::copy(location, destination)?;
        }
        _ => {
            error!("Unknown vendor: {}", version);
            process::exit(1);
        }
    }
    Ok(())
}

fn create_env_file(host_location: &str) -> Result<(), std::io::Error> {
    let loglevel = env::var("COMP_LOG_LEVEL").unwrap_or_else(|_| "debug".to_string());
    let ignore_crio = env::var("COMP_IGNORE_CRIO")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase();
    let crio_image = env::var("COMP_CRIO_IMAGE_CMD").unwrap_or_else(|_| "img".to_string());
    let destination = format!("{}/{}", host_location, ".env");
    let use_crio_config = env::var("DEPLOY_CRIO_CONFIG")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase();
    let filename_template = env::var("COMP_FILENAME_TEMPLATE").unwrap_or_else(|_| {
        "{uuid}-dump-{timestamp}-{hostname}-{exe_name}-{pid}-{signal}".to_string()
    });
    let log_length = env::var("LOG_LENGTH").unwrap_or_else(|_| "500".to_string());
    let pod_selector_label = env::var("COMP_POD_SELECTOR_LABEL").unwrap_or_default();
    info!("Creating {} file with LOG_LEVEL={}", destination, loglevel);
    let mut env_file = File::create(destination)?;
    let text = format!(
        "LOG_LEVEL={}\nIGNORE_CRIO={}\nCRIO_IMAGE_CMD={}\nUSE_CRIO_CONF={}\nFILENAME_TEMPLATE={}\nLOG_LENGTH={}\nPOD_SELECTOR_LABEL={}\n",
        loglevel, ignore_crio, crio_image, use_crio_config, filename_template, log_length, pod_selector_label
    );
    info!("Writing composer .env \n{}", text);
    env_file.write_all(text.as_bytes())?;
    env_file.flush()?;
    Ok(())
}

fn get_path() -> String {
    let mut local_bin = env::var("LOCAL_BIN").unwrap_or_else(|_| "".to_string());
    local_bin.push(':');
    local_bin.push_str(BIN_PATH);
    local_bin
}

fn get_sysctl(name: &str) -> Result<String, anyhow::Error> {
    info!("Getting sysctl for {}", name);
    let output = Command::new("sysctl")
        .env("PATH", get_path())
        .args(&["-n", name])
        .output()?;
    let lines = String::from_utf8(output.stdout)?;
    let line = lines.lines().take(1).next().unwrap_or("");
    Ok(line.to_string())
}
fn apply_sysctl(name: &str, location: &str, value: &str) -> Result<(), anyhow::Error> {
    info!("Starting sysctl for {} {} with {}", name, location, value);
    let ctl = get_sysctl(name)?;
    // The values are different so let's back up and apply
    if ctl.as_str() != value {
        let mut file = File::create(location)?;
        file.write_all(ctl.as_bytes())?;
        file.flush()?;
        info!("Created Backup of {}", location);
        overwrite_sysctl(name, value)?
    } else {
        info!("{} with value {} is already applied", name, ctl);
    }
    Ok(())
}

fn overwrite_sysctl(name: &str, value: &str) -> Result<(), anyhow::Error> {
    let s = format!("{}={}", name, value);
    let output = Command::new("sysctl")
        .env("PATH", get_path())
        .args(&["-w", s.as_str()])
        .status()?;
    if !output.success() {
        let e = Error::InvalidOverWrite {
            name: name.to_string(),
            value: value.to_string(),
        };
        return Err(anyhow::Error::new(e));
    }
    Ok(())
}

fn remove() -> Result<(), anyhow::Error> {
    restore_sysctl("kernel", "core_pattern")?;
    restore_sysctl("kernel", "core_pipe_limit")?;
    restore_sysctl("fs", "suid_dumpable")?;
    let host_dir = env::var("HOST_DIR").unwrap_or_else(|_| DEFAULT_BASE_DIR.to_string());
    let exe = format!("{}/{}", host_dir, CDC_NAME);
    let env_file = format!("{}/{}", host_dir, ".env");
    let crictl_file = format!("{}/{}", host_dir, "crictl.yaml");
    let composer_file = format!("{}/{}", host_dir, "composer.log");
    let crictl_exe = format!("{}/{}", host_dir, "crictl");

    fs::remove_file(exe)?;
    fs::remove_file(env_file)?;

    if !Path::new(&crictl_exe).exists() {
        fs::remove_file(crictl_exe)?;
    }
    if !Path::new(&crictl_file).exists() {
        fs::remove_file(crictl_file)?;
    }
    if !Path::new(&composer_file).exists() {
        fs::remove_file(composer_file)?;
    }

    Ok(())
}
fn restore_sysctl(prefix: &str, name: &str) -> Result<(), anyhow::Error> {
    info!("Restoring Backup of {}", name);
    let host_dir = env::var("HOST_DIR").unwrap_or_else(|_| DEFAULT_BASE_DIR.to_string());
    let file_name = format!("{}/{}.bak", host_dir, name);
    let sysctl_name = format!("{}.{}", prefix, name);
    let line = fs::read_to_string(&file_name)?;
    overwrite_sysctl(sysctl_name.as_str(), line.as_str())?;
    fs::remove_file(file_name)?;

    Ok(())
}
