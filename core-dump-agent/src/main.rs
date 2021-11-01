extern crate dotenv;
extern crate s3;

use advisory_lock::{AdvisoryFileLock, FileLockMode};
use anyhow::Result;
use env_logger::Env;
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
// use std::{thread, time};
use tokio_cron_scheduler::{Job, JobScheduler};

#[allow(dead_code)]
struct Storage {
    name: String,
    region: Region,
    credentials: Credentials,
    bucket: String,
    location_supported: bool,
}

const BIN_PATH: &str = "/bin:/sbin:/usr/bin:/usr/sbin:/usr/local/bin";
const CDC_NAME: &str = "cdc";
static DEFAULT_BASE_DIR: &str = "/var/mnt/core-dump-handler";
static DEFAULT_SUID_DUMPABLE: &str = "2";
// Keep it single threaded so resource requirements from both a network
// and computation perspective are kept to a minimum.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let mut env_path = env::current_exe()?;
    env_path.pop();
    env_path.push(".env");

    let mut envloadmsg = String::from("Loading .env");
    match dotenv::from_path(env_path) {
        Ok(v) => v,
        Err(e) => {
            envloadmsg = format!(
                "no .env file found \n That's ok if running in kubernetes\n{}",
                e
            )
        }
    }

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let host_dir = env::var("HOST_DIR").unwrap_or_else(|_| DEFAULT_BASE_DIR.to_string());
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

    if pattern == "remove" {
        info!("Removing {}", host_location);
        remove()?;
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
    copy_sysctl_to_file(
        "kernel.core_pattern",
        format!("{}/core_pattern.bak", host_location).as_str(),
    )?;
    copy_sysctl_to_file(
        "kernel.core_pipe_limit",
        format!("{}/core_pipe_limit.bak", host_location).as_str(),
    )?;

    copy_sysctl_to_file(
        "fs.suid_dumpable",
        format!("{}/suid_dumpable.bak", host_location).as_str(),
    )?;

    overwrite_sysctl(
        "kernel.core_pattern",
        format!(
            "|{}/{} -c=%c -e=%e -p=%p -s=%s -t=%t -d={}/core -h=%h -E=%E",
            host_location, CDC_NAME, host_dir
        )
        .as_str(),
    )?;
    overwrite_sysctl("kernel.core_pipe_limit", "128")?;

    overwrite_sysctl("fs.suid_dumpable", &suid)?;

    let core_location = format!("{}/core", host_location);

    fs::create_dir_all(&core_location)?;

    create_env_file(host_location)?;
    // Run polling agent on startup to clean up files.
    run_polling_agent(core_location.as_str());

    let interval = env::var("INTERVAL").unwrap_or_else(|_| String::from(""));
    let mut schedule = env::var("SCHEDULE").unwrap_or_else(|_| String::from(""));

    let _use_inotify = match env::var("USE_INOTIFY")
        .unwrap_or_else(|_| String::from("false"))
        .parse::<u64>()
    {
        Ok(v) => v,
        Err(e) => {
            panic!("Error parsing interval {}", e);
        }
    };

    if !interval.is_empty() && !schedule.is_empty() {
        warn!(
            "Interval set to: {}, Schedule set to:{}\n Using interval:{}",
            interval, schedule, interval
        );
    }
    // Overwriting the schedule string if interval present
    if !interval.is_empty() {
        let mut i_interval = match interval.parse::<u64>() {
            Ok(v) => v,
            Err(e) => {
                panic!("Error parsing interval {}", e);
            }
        };
        i_interval = i_interval / 1000;
        schedule = format!("1/{} * * * * *", i_interval);
    }
    let mut sched = JobScheduler::new();
    let s_job = match Job::new(schedule.as_str(), move |_uuid, _l| {
        run_polling_agent(core_location.as_str());
    }) {
        Ok(v) => v,
        Err(e) => panic!("Job Creation failed, {}", e),
    };
    match sched.add(s_job) {
        Ok(v) => v,
        Err(e) => panic!("Job Scheduing failed, {}", e),
    }

    tokio::spawn(sched.start());

    // sched.add(Job::new(schedule.parse().unwrap(), || {
    //         //run_polling_agent(core_location.as_str());
    //         println!("I get executed every 10 seconds!");
    //     }));
    Ok(())
    // let mut sched = JobScheduler::new();
    // sched.add(Job::new("1/10 * * * * *".parse().unwrap(), || {
    //     run_polling_agent(core_location.as_str());
    //     println!("I get executed every 10 seconds!");
    // }));

    // loop {
    //     run_polling_agent(core_location.as_str());

    //     let millis = time::Duration::from_millis(interval);
    //     thread::sleep(millis);
    // }
}

fn process_file(zip_path: &Path, bucket: &Bucket) {
    info!("Uploading: {}", zip_path.display());

    let mut f = File::open(&zip_path).expect("no file found");

    match f.try_lock(FileLockMode::Shared) {
        Ok(_) => { /* If we can lock then we are ok */ }
        Err(e) => {
            info!("file locked so we are ignoring it for this iteration {}", e);
            return;
        }
    }

    let metadata = fs::metadata(&zip_path).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read_exact(&mut buffer)
        .expect("Failed to read_exact for file");
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

    let (_, code) = match bucket.put_object_blocking(upload_file_name, buffer.as_slice()) {
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

fn get_bucket() -> Result<s3::Bucket, s3::S3Error> {
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

    let s3 = Storage {
        name: "aws".into(),
        region,
        credentials: Credentials::new(
            Some(s3_access_key.as_str()),
            Some(s3_secret.as_str()),
            None,
            None,
            None,
        )
        .unwrap(),
        bucket: s3_bucket_name,
        location_supported: false,
    };
    Bucket::new_with_path_style(&s3.bucket, s3.region, s3.credentials)
}

fn run_polling_agent(core_location: &str) {
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
        process_file(&zip_path, &bucket);
    }
}

fn generate_crio_config(host_location: &str) -> Result<(), std::io::Error> {
    info!("Generating crio file");
    let destination = format!("{}/{}", host_location, "crictl.yaml");
    let mut crictl_file = File::create(destination)?;
    let text = "runtime-endpoint: unix:///run/containerd/containerd.sock\nimage-endpoint: unix:///run/containerd/containerd.sock\ntimeout: 2\ndebug: false\npull-image-on-create: false";
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
    let loglevel = env::var("COMP_LOG_LEVEL").unwrap_or_else(|_| "error".to_string());
    let ignore_crio = env::var("COMP_IGNORE_CRIO")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase();
    let crio_image = env::var("COMP_CRIO_IMAGE_CMD").unwrap_or_else(|_| "error".to_string());
    let destination = format!("{}/{}", host_location, ".env");
    let use_crio_config = env::var("DEPLOY_CRIO_CONFIG")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase();

    info!("Creating {} file with LOG_LEVEL={}", destination, loglevel);
    let mut env_file = File::create(destination)?;
    let text = format!(
        "LOG_LEVEL={}\nIGNORE_CRIO={}\nCRIO_IMAGE_CMD={}\nUSE_CRIO_CONF={}\n",
        loglevel, ignore_crio, crio_image, use_crio_config
    );
    info!("Writing composer .env \n{}", text);
    env_file.write_all(text.as_bytes())?;
    env_file.flush()?;
    Ok(())
}

fn copy_sysctl_to_file(name: &str, location: &str) -> Result<(), std::io::Error> {
    info!("Starting sysctl for {} {}", name, location);
    let output = match Command::new("sysctl")
        .env("PATH", BIN_PATH)
        .args(&["-n", name])
        .output()
    {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to run command {}", e);
            panic!("Exiting copy sysctl")
        }
    };

    let line = match String::from_utf8(output.stdout) {
        Ok(v) => v,
        Err(e) => panic!("failed to copy {} {}", location, e),
    };

    let mut file = File::create(location)?;
    file.write_all(
        line.lines()
            .take(1)
            .next()
            .expect("Failed to get line for sysctl file")
            .as_bytes(),
    )?;
    file.flush()?;
    info!("Created Backup of {}", location);
    Ok(())
}

fn overwrite_sysctl(name: &str, value: &str) -> Result<(), std::io::Error> {
    let s = format!("{}={}", name, value);
    let output = Command::new("sysctl")
        .env("PATH", BIN_PATH)
        .args(&["-w", s.as_str()])
        .status()?;
    if !output.success() {
        panic!("Failed to set {} to {}", name, value);
    }
    info!("Created sysctl of {}", s);
    Ok(())
}

fn remove() -> Result<(), std::io::Error> {
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
fn restore_sysctl(prefix: &str, name: &str) -> Result<(), std::io::Error> {
    info!("Restoring Backup of {}", name);
    let host_dir = env::var("HOST_DIR").unwrap_or_else(|_| DEFAULT_BASE_DIR.to_string());
    let file_name = format!("{}/{}.bak", host_dir, name);
    let sysctl_name = format!("{}.{}", prefix, name);
    let line = match fs::read_to_string(&file_name) {
        Ok(l) => l,
        Err(e) => panic!("{} does not contain a line\n {}", name, e),
    };
    overwrite_sysctl(sysctl_name.as_str(), line.as_str())?;
    fs::remove_file(file_name)?;

    Ok(())
}
