extern crate dotenv;

use advisory_lock::{AdvisoryFileLock, FileLockMode};
use clap::{App, Arg};
use log::{debug, error, info, LevelFilter};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use serde_json::{json, Value};
use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::Write;
use std::process;
use std::process::Command;
use std::str::FromStr;
use uuid::Uuid;
use zip::write::FileOptions;
use zip::ZipWriter;

fn main() -> Result<(), anyhow::Error> {
    let mut env_path = env::current_exe()?;
    env_path.pop();
    env_path.push(".env");

    let mut config_path = env::current_exe()?;
    config_path.pop();
    config_path.push("crictl.yaml");
    let config_path_str = config_path.into_os_string().into_string().unwrap();

    let mut base_path = env::current_exe()?;
    base_path.pop();
    let base_path_str = base_path
        .into_os_string()
        .into_string()
        .unwrap_or_else(|_| "/var/mnt/core-dump-handler".to_string());
    let mut envloadmsg = String::from("Loading .env");
    match dotenv::from_path(env_path) {
        Ok(v) => v,
        Err(e) => envloadmsg = format!("no .env file found so using Debug level logging {}", e),
    }

    debug!("Arguments: {:?}", env::args());

    let loglevel = env::var("LOG_LEVEL").unwrap_or_default();
    let ignore_crio = env::var("IGNORE_CRIO")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase();
    let img = env::var("CRIO_IMAGE_CMD").unwrap_or_else(|_| "img".to_string());
    let use_crio_config =
        env::var("USE_CRIO_CONF").unwrap_or_else(|_| "false".to_string().to_lowercase());

    info!(
        "Environment config:\n IGNORE_CRIO={}\nCRIO_IMAGE_CMD={}\nUSE_CRIO_CONF={}",
        ignore_crio, img, use_crio_config
    );
    let logfilter = match LevelFilter::from_str(loglevel.as_str()) {
        Ok(v) => v,
        Err(_) => LevelFilter::Debug,
    };

    let mut log_path = env::current_exe()?;
    log_path.pop();
    log_path.push("composer.log");

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {d} - {m}\n")))
        .build(&log_path)?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(logfilter))?;

    log4rs::init_config(config)?;

    info!("{}", envloadmsg);
    info!("Set logfile to: {:?}", &log_path);

    let matches = match App::new("Core Dump Composer")
        .version("0.1.0")
        .author("Anton Whalley <anton@venshare.com>")
        .about("Processes Core Dumps in a K8s System")
        .arg(
            Arg::new("limit-size")
                .short('c')
                .long("limit-size")
                .required(true)
                .takes_value(true)
                .about("Core file size soft resource limit of crashing process"),
        )
        .arg(
            Arg::new("exe-name")
                .short('e')
                .long("exe-name")
                .required(true)
                .takes_value(true)
                .about(
                    "The process or thread's comm value, which typically is the
                same as the executable filename (without path prefix, and
                truncated to a maximum of 15 characters)",
                ),
        )
        .arg(
            Arg::new("pid")
                .short('p')
                .long("pid")
                .required(true)
                .takes_value(true)
                .about(
                    "PID of dumped process, as seen in the PID namespace in which
                the process resides.",
                ),
        )
        .arg(
            Arg::new("signal")
                .short('s')
                .long("signal")
                .required(true)
                .takes_value(true)
                .about("Number of signal causing dump."),
        )
        .arg(
            Arg::new("timestamp")
                .short('t')
                .long("timestamp")
                .required(true)
                .takes_value(true)
                .about("Time of dump, expressed as seconds since the Epoch."),
        )
        .arg(
            Arg::new("directory")
                .short('d')
                .long("dir")
                .required(true)
                .takes_value(true)
                .about("Directory to save the core dump to."),
        )
        .arg(
            Arg::new("hostname")
                .short('h')
                .long("hostname")
                .required(true)
                .takes_value(true)
                .about("Hostname (same as nodename returned by uname(2))"),
        )
        .arg(
            Arg::new("pathname")
                .short('E')
                .long("pathname")
                .required(true)
                .takes_value(true)
                .about("Hostname (same as nodename returned by uname(2))"),
        )
        .try_get_matches()
    {
        Ok(v) => v,
        Err(e) => {
            error!("Incorrect Parameters: {}", e);
            e.exit()
        }
    };
    let bin_path_string = format!(
        "/bin:/sbin:/usr/bin:/usr/sbin:/usr/local/bin:/home/kubernetes/bin:{}",
        base_path_str
    );
    let bin_path = bin_path_string.as_str();
    let _core_limit_size = matches.value_of("limit-size").unwrap_or("");
    let core_exe_name = matches.value_of("exe-name").unwrap_or("");
    let core_pid = matches.value_of("pid").unwrap_or("");
    let core_signal = matches.value_of("signal").unwrap_or("");
    let core_timestamp = matches.value_of("timestamp").unwrap_or("");
    let core_directory = matches.value_of("directory").unwrap_or("");
    let core_hostname = matches.value_of("hostname").unwrap_or("");
    let _core_pathname = matches.value_of("pathname").unwrap_or("");
    let core_uuid = Uuid::new_v4();
    // Using deflated as bzip2 throws an error in unzip
    // https://github.com/madler/unzip/commit/5e2efcd633a4a1fb95a129a75508e7d769e767be
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o444);

    let os_hostname =
        hostname::get().unwrap_or_else(|_| OsString::from_str("unknown").unwrap_or_default());
    let node_hostname = os_hostname
        .into_string()
        .unwrap_or_else(|_| "unknown".to_string());

    let dump_name = format!(
        "{}-dump-{}-{}-{}-{}-{}",
        core_uuid, core_timestamp, core_hostname, core_exe_name, core_pid, core_signal
    );
    debug!("Creating dump for {}", dump_name);

    // Create the base zip file that we are going to put everything into
    let zip_path = format!("{}/{}.zip", core_directory, dump_name);
    let path = std::path::Path::new(&zip_path);

    let file = match File::create(&path) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to create file: {}", e);
            process::exit(1);
        }
    };
    file.lock(FileLockMode::Exclusive)?;
    let mut zip = ZipWriter::new(&file);

    // Create a JSON file to store the dump meta data
    let dump_info_name = format!("{}-dump-info.json", dump_name);
    match zip.start_file(dump_info_name, options) {
        Ok(v) => v,
        Err(e) => error!("Errer starting zip file{}", e),
    };
    let dump_info_content = format!(
        "{{\"uuid\":\"{}\", \"dump_file\":\"{}.core\", \"timestamp\": \"{}\", 
    \"hostname\": \"{}\", \"exe\": \"{}\", \"real_pid\": \"{}\", \"signal\": \"{}\", \"node_hostname\": \"{}\" }}",
        core_uuid, dump_name, core_timestamp, core_hostname, core_exe_name, core_pid, core_signal, node_hostname
    );
    match zip.write_all(dump_info_content.as_bytes()) {
        Ok(v) => v,
        Err(e) => error!("Errer writing zip file{}", e),
    };

    // Pipe the core file to zip
    let dump_file_name = format!("{}.core", dump_name);
    match zip.start_file(dump_file_name, options) {
        Ok(v) => v,
        Err(e) => error!("Errer starting zip file{}", e),
    };

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut core_file: Vec<u8> = Vec::new();

    let mut data = [0u8; 8192];

    while let Ok(n) = stdin.read(&mut data) {
        if n == 0 {
            break;
        }
        core_file.extend_from_slice(&data);
    }
    match zip.write_all(&core_file) {
        Ok(v) => v,
        Err(e) => error!("Errer writing zip file{}", e),
    };

    if FromStr::from_str(&ignore_crio) == Ok(true) {
        zip.finish()?;
        file.unlock()?;
        process::exit(0)
    }
    let pod_output_args;
    if use_crio_config == "true" {
        pod_output_args = vec![
            "-c",
            config_path_str.as_str(),
            "pods",
            "--name",
            core_hostname,
            "-o",
            "json",
        ];
    } else {
        pod_output_args = vec!["pods", "--name", core_hostname, "-o", "json"];
    }

    info!("Running crictl {:?}", pod_output_args);

    let pod_output = match Command::new("crictl")
        .env("PATH", bin_path)
        .args(&pod_output_args)
        .output()
    {
        Ok(v) => v,
        Err(e) => {
            error!("failed to execute crictl pods {}", e);
            zip.finish()?;
            file.unlock()?;
            process::exit(1)
        }
    };

    let pod_object: Value = match serde_json::from_slice(pod_output.stdout.as_slice()) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to get container info {}", e);
            json!({})
        }
    };

    let pod_file_name = format!("{}-pod-info.json", dump_name);
    debug!("Using runtime_file_name:{}", pod_file_name);
    match zip.start_file(pod_file_name, options) {
        Ok(v) => v,
        Err(e) => error!("Errer starting zip file{}", e),
    };

    debug!("pod object {}", pod_object);
    match zip.write_all(pod_object.to_string().as_bytes()) {
        Ok(v) => v,
        Err(e) => error!("Errer writing zip file{}", e),
    };

    let pod_id = match pod_object["items"][0]["id"].as_str() {
        Some(v) => v,
        None => {
            error!("Failed to get pod id");
            ""
        }
    };

    // With the pod_id get the runtime information from crictl

    debug!("Using pod_id:{}", pod_id);
    let inspect_output_args;
    if use_crio_config == "true" {
        inspect_output_args = vec!["-c", config_path_str.as_str(), "inspectp", pod_id];
    } else {
        inspect_output_args = vec!["inspectp", pod_id];
    }
    info!("Running crictl {:?}", inspect_output_args);
    match Command::new("crictl")
        .env("PATH", bin_path)
        .args(&inspect_output_args)
        .output()
    {
        Ok(inspectp_output) => {
            debug!("inspectp_output status: {}", inspectp_output.status);
            debug!(
                "inspectp_output stderr, {}",
                String::from_utf8(inspectp_output.stderr.clone()).unwrap_or_default()
            );
            let runtime_file_name = format!("{}-runtime-info.json", dump_name);
            debug!("Using runtime_file_name:{}", runtime_file_name);
            match zip.start_file(runtime_file_name, options) {
                Ok(v) => v,
                Err(e) => error!("Errer starting zip file{}", e),
            };

            debug!(
                "inspectp_output: {}",
                String::from_utf8(inspectp_output.stdout.clone()).unwrap_or_default()
            );
            match zip.write_all(&inspectp_output.stdout) {
                Ok(v) => v,
                Err(e) => error!("Errer writing zip file{}", e),
            };
        }
        Err(e) => {
            error!("Failed to get inspectp details {}", e);
        }
    };

    // Get the container_image_name based on the pod_id
    let mut ps_object: Value = json!({});
    let ps_output_args;
    if use_crio_config == "true" {
        ps_output_args = vec![
            "-c",
            config_path_str.as_str(),
            "ps",
            "-o",
            "json",
            "-p",
            pod_id,
        ];
    } else {
        ps_output_args = vec!["ps", "-o", "json", "-p", pod_id];
    }
    info!("Running crictl {:?}", ps_output_args);
    match Command::new("crictl")
        .env("PATH", bin_path)
        .args(&ps_output_args)
        .output()
    {
        Ok(ps_output) => {
            debug!("ps_output status: {}", ps_output.status);
            debug!(
                "ps_output stderr, {}",
                String::from_utf8(ps_output.stderr.clone()).unwrap_or_default()
            );
            debug!(
                "ps_output: {}",
                String::from_utf8(ps_output.stdout.clone()).unwrap_or_default()
            );

            ps_object = match serde_json::from_str(
                String::from_utf8(ps_output.stdout)
                    .unwrap_or_default()
                    .as_str(),
            ) {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to get ps info {}", e);
                    json!({})
                }
            };
            let ps_info_name = format!("{}-ps-info.json", dump_name);
            match zip.start_file(ps_info_name, options) {
                Ok(v) => v,
                Err(e) => error!("Errer starting zip file{}", e),
            };
            let ps_info_content = serde_json::to_string(&ps_object).unwrap_or_default();
            match zip.write_all(ps_info_content.as_bytes()) {
                Ok(v) => v,
                Err(e) => error!("Errer writing zip file{}", e),
            };
        }
        Err(e) => {
            error!("Failed to get ps details {}", e);
        }
    };

    debug!("Successfully got the process details");
    // TOD0: We should iterate over the containers to make sure we have the right crashing pod
    // This will need a different pod defined to test
    let img_id = match serde_json::to_string(&ps_object["containers"][0]["imageRef"]) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to get pod id {}", e);
            String::from_str("").unwrap()
        }
    };
    debug!("found img_id {}", img_id);

    let mut image_list: Value = json!({});
    let image_args;

    if use_crio_config == "true" {
        image_args = vec!["-c", config_path_str.as_str(), img.as_str(), "-o", "json"];
    } else {
        image_args = vec![img.as_str(), "-o", "json"];
    }
    info!("Running crictl {:?}", image_args);

    match Command::new("crictl")
        .env("PATH", bin_path)
        .args(&image_args)
        .output()
    {
        Ok(img_output) => {
            image_list = match serde_json::from_slice(img_output.stdout.as_slice()) {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to get imagelist info {}", e);
                    json!({})
                }
            };
        }
        Err(e) => error!("Failed to run crictl img info {}", e),
    }
    debug!("Found image list:\n {}", image_list);
    let img_id_iter = img_id.split("@sha256:").collect::<Vec<&str>>();
    let clean_image_id;
    if img_id_iter.len() == 2 {
        clean_image_id = String::from(img_id_iter[1]);
    } else {
        clean_image_id = img_id;
    }

    match image_list["images"].as_array() {
        Some(img_lines) => {
            debug!("Found {} images", img_lines.len());
            for line in img_lines {
                let line_obj: Value = serde_json::to_value(line).unwrap();

                debug!("Matching {} to {}", &line_obj["id"], clean_image_id);

                if serde_json::to_string(&line_obj["id"]).unwrap_or_default() == clean_image_id {
                    let img_info_name = format!("{}-image-info.json", dump_name);
                    debug!("found image {} starting to zip {}", clean_image_id, img_info_name);
                    match zip.start_file(img_info_name, options) {
                        Ok(v) => v,
                        Err(e) => error!("Errer starting zip file{}", e),
                    };
                    let img_info_content = serde_json::to_string(&line_obj).unwrap_or_default();
                    match zip.write_all(img_info_content.as_bytes()) {
                        Ok(v) => v,
                        Err(e) => error!("Errer writing zip file{}", e),
                    };
                    break;
                }
            }
        }
        None => {
            error!("no images found in crictl img")
        }
    }
    zip.finish()?;
    file.unlock()?;
    Ok(())
}
