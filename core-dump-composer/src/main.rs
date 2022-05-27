extern crate dotenv;

use advisory_lock::{AdvisoryFileLock, FileLockMode};
use libcrio::Cli;
use log::{debug, error, info};
use serde_json::json;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::process;
use zip::write::FileOptions;
use zip::ZipWriter;

mod config;
mod logging;

fn main() -> Result<(), anyhow::Error> {
    let mut cc = config::CoreConfig::new()?;
    cc.set_namespace("default".to_string());
    let mut envloadmsg = String::from("Loading .env");
    let l_dot_env_path = cc.dot_env_path.clone();
    match dotenv::from_path(l_dot_env_path) {
        Ok(v) => v,
        Err(e) => envloadmsg = format!("no .env file found so using Debug level logging {}", e),
    }
    let l_log_level = cc.log_level.clone();
    let log_path = logging::init_logger(l_log_level)?;
    debug!("Arguments: {:?}", env::args());

    info!(
        "Environment config:\n IGNORE_CRIO={}\nCRIO_IMAGE_CMD={}\nUSE_CRIO_CONF={}",
        cc.ignore_crio, cc.image_command, cc.use_crio_config
    );
    info!("{}", envloadmsg);
    info!("Set logfile to: {:?}", &log_path);
    debug!("Creating dump for {}", cc.get_templated_name());

    let l_crictl_config_path = cc.crictl_config_path.clone();

    let config_path = if cc.use_crio_config {
        Some(
            l_crictl_config_path
                .into_os_string()
                .to_string_lossy()
                .to_string(),
        )
    } else {
        None
    };
    let l_bin_path = cc.bin_path.clone();
    let l_image_command = cc.image_command.clone();
    let cli = Cli {
        bin_path: l_bin_path,
        config_path,
        image_command: l_image_command,
    };
    let pod_object = match cli.pod(&cc.params.hostname) {
        Ok(v) => v,
        Err(e) => {
            error!("{}", e);
            // We fall through here as the coredump and info can still be captured.
            json!({})
        }
    };

    let namespace = pod_object["metadata"]["namespace"]
        .as_str()
        .unwrap_or("unknown");

    cc.set_namespace(namespace.to_string());

    let podname = pod_object["metadata"]["name"]
        .as_str()
        .unwrap_or("unknown");

    cc.set_podname(podname.to_string());

    // Create the base zip file that we are going to put everything into
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o444)
        .large_file(true);

    let file = match File::create(cc.get_zip_full_path()) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to create file: {}", e);
            process::exit(1);
        }
    };
    file.lock(FileLockMode::Exclusive)?;
    let mut zip = ZipWriter::new(&file);

    debug!(
        "Create a JSON file to store the dump meta data\n{}",
        cc.get_dump_info_filename()
    );

    match zip.start_file(cc.get_dump_info_filename(), options) {
        Ok(v) => v,
        Err(e) => {
            error!("Error starting dump file in zip \n{}", e);
            zip.finish()?;
            file.unlock()?;
            process::exit(1);
        }
    };

    match zip.write_all(cc.get_dump_info().as_bytes()) {
        Ok(v) => v,
        Err(e) => {
            error!("Error writing pod file in zip \n{}", e);
            zip.finish()?;
            file.unlock()?;
            process::exit(1);
        }
    };

    // Pipe the core file to zip
    match zip.start_file(cc.get_core_filename(), options) {
        Ok(v) => v,
        Err(e) => error!("Error starting core file \n{}", e),
    };

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut data = [0u8; 8192];

    while let Ok(n) = stdin.read(&mut data) {
        if n == 0 {
            break;
        }
        match zip.write_all(&data) {
            Ok(v) => v,
            Err(e) => {
                error!("Error writing core file \n{}", e);
                process::exit(1);
            }
        };
    }
    zip.flush()?;

    if cc.ignore_crio {
        zip.finish()?;
        file.unlock()?;
        process::exit(0);
    }

    // let l_crictl_config_path = cc.crictl_config_path.clone();

    // let config_path = if cc.use_crio_config {
    //     Some(
    //         l_crictl_config_path
    //             .into_os_string()
    //             .to_string_lossy()
    //             .to_string(),
    //     )
    // } else {
    //     None
    // };
    // let l_bin_path = cc.bin_path.clone();
    // // let image_command = if cc.image_command == *"image" {
    // //     libcrio::ImageCommand::Images
    // // } else {
    // //     libcrio::ImageCommand::Img
    // // };
    // let cli = Cli {
    //     bin_path: l_bin_path,
    //     config_path,
    //     image_command
    // };

    // let l_pod_filename = cc.get_pod_filename().clone();
    debug!("Using runtime_file_name:{}", cc.get_pod_filename());

    match zip.start_file(cc.get_pod_filename(), options) {
        Ok(v) => v,
        Err(e) => {
            error!("Error starting pod file in zip \n{}", e);
            zip.finish()?;
            file.unlock()?;
            process::exit(1);
        }
    };

    match zip.write_all(pod_object.to_string().as_bytes()) {
        Ok(v) => v,
        Err(e) => {
            error!("Error writing pod file in zip \n{}", e);
            zip.finish()?;
            file.unlock()?;
            process::exit(1);
        }
    };

    // TODO: Check logging of more than one pod retured
    let pod_id = match pod_object["id"].as_str() {
        Some(v) => v,
        None => {
            error!("Failed to get pod id");
            zip.finish()?;
            file.unlock()?;
            process::exit(1);
        }
    };

    // With the pod_id get the runtime information from crictl
    debug!("Getting inspectp output using pod_id:{}", pod_id);

    let inspectp = match cli.inspect_pod(pod_id) {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to inspect pod {}", e);
            json!({})
        }
    };
    debug!("Starting inspectp file\n{}", cc.get_inspect_pod_filename());
    match zip.start_file(cc.get_inspect_pod_filename(), options) {
        Ok(v) => v,
        Err(e) => {
            error!("Error starting inspect pod file in zip \n{}", e);
            zip.finish()?;
            file.unlock()?;
            process::exit(1);
        }
    };
    debug!("Writing inspectp file\n{}", cc.get_inspect_pod_filename());
    match zip.write_all(inspectp.to_string().as_bytes()) {
        Ok(v) => v,
        Err(e) => {
            error!("Error writing inspect pod file in zip \n{}", e);
            zip.finish()?;
            file.unlock()?;
            process::exit(1);
        }
    };

    // Get the container_image_name based on the pod_id
    let ps_object = match cli.pod_containers(pod_id) {
        Ok(v) => v,
        Err(e) => {
            error!("{}", e);
            zip.finish()?;
            file.unlock()?;
            process::exit(1);
        }
    };

    debug!("Starting ps file \n{}", cc.get_ps_filename());
    match zip.start_file(cc.get_ps_filename(), options) {
        Ok(v) => v,
        Err(e) => {
            error!("Error starting ps file in zip \n{}", e);
            zip.finish()?;
            file.unlock()?;
            process::exit(1);
        }
    };

    debug!("Writing ps file \n{}", cc.get_ps_filename());
    match zip.write_all(ps_object.to_string().as_bytes()) {
        Ok(v) => v,
        Err(e) => {
            error!("Error writing ps file in zip \n{}", e);
            zip.finish()?;
            file.unlock()?;
            process::exit(1);
        }
    };

    debug!("Successfully got the process details {}", ps_object);

    if let Some(containers) = ps_object["containers"].as_array() {
        for container in containers {
            let counter = 0;
            let img_ref = match container["imageRef"].as_str() {
                Some(v) => v,
                None => {
                    error!("Failed to get containerid {}", "");
                    break;
                }
            };
            let log =
                match cli.tail_logs(container["id"].as_str().unwrap_or_default(), cc.log_length) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Error finding logs:\n{}", e);
                        "".to_string()
                    }
                };
            debug!("Starting log file \n{}", cc.get_log_filename(counter));
            match zip.start_file(cc.get_log_filename(counter), options) {
                Ok(v) => v,
                Err(e) => {
                    error!("Error starting log file in zip \n{}", e);
                    zip.finish()?;
                    file.unlock()?;
                    process::exit(1);
                }
            };
            debug!("Writing file output \n{}", log);
            // TODO: Should this be streamed?
            match zip.write_all(log.to_string().as_bytes()) {
                Ok(v) => v,
                Err(e) => {
                    error!("Error writing log file in zip \n{}", e);
                    zip.finish()?;
                    file.unlock()?;
                    process::exit(1);
                }
            };
            debug!("found img_id {}", img_ref);
            let image = match cli.image(img_ref) {
                Ok(v) => v,
                Err(e) => {
                    error!("Error finding image:\n{}", e);
                    json!({})
                }
            };

            debug!("Starting image file \n{}", cc.get_image_filename(counter));
            match zip.start_file(cc.get_image_filename(counter), options) {
                Ok(v) => v,
                Err(e) => {
                    error!("Error starting ps file in zip \n{}", e);
                    zip.finish()?;
                    file.unlock()?;
                    process::exit(1);
                }
            };
            debug!("Writing image file \n{}", cc.get_image_filename(counter));
            match zip.write_all(image.to_string().as_bytes()) {
                Ok(v) => v,
                Err(e) => {
                    error!("Error writing ps file in zip \n{}", e);
                    zip.finish()?;
                    file.unlock()?;
                    process::exit(1);
                }
            };
            debug!(
                "Getting logs for container id {}",
                container["id"].as_str().unwrap_or_default()
            );
        }
    };

    zip.finish()?;
    file.unlock()?;
    Ok(())
}
