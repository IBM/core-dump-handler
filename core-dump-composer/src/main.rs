extern crate dotenv;

use crate::events::CoreEvent;

use advisory_lock::{AdvisoryFileLock, FileLockMode};
use libcrio::Cli;
use log::{debug, error, info, warn};
use serde_json::json;
use serde_json::Value;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::process;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use zip::write::FileOptions;
use zip::ZipWriter;

mod config;
mod events;
mod logging;

fn main() -> Result<(), anyhow::Error> {
    let (send, recv) = channel();
    let cc = config::CoreConfig::new()?;
    let recv_time: u64 = cc.timeout as u64;
    thread::spawn(move || {
        let result = handle(cc);
        send.send(result).unwrap();
    });

    let result = recv.recv_timeout(Duration::from_secs(recv_time));

    match result {
        Ok(inner_result) => inner_result,
        Err(_error) => {
            error!("Timeout error during coredump processing.");
            process::exit(32);
        }
    }
}

fn handle(mut cc: config::CoreConfig) -> Result<(), anyhow::Error> {
    cc.set_namespace("default".to_string());
    let l_log_level = cc.log_level.clone();
    let log_path = logging::init_logger(l_log_level)?;
    debug!("Arguments: {:?}", env::args());

    info!(
        "Environment config:\n IGNORE_CRIO={}\nCRIO_IMAGE_CMD={}\nUSE_CRIO_CONF={}",
        cc.ignore_crio, cc.image_command, cc.use_crio_config
    );

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

    // match the label filter if there's one, and skip the whole process if it doesn't match
    if !cc.pod_selector_label.is_empty() {
        debug!(
            "Pod selector specified. Will record only if pod has label {}",
            &cc.pod_selector_label
        );
        let pod_labels = pod_object["labels"].as_object().unwrap();
        // check if pod_labels has pod_selector_label
        if pod_labels.get(&cc.pod_selector_label).is_none() {
            info!(
                "Skipping pod as it did not match selector label {}",
                &cc.pod_selector_label
            );
            process::exit(0);
        }
    } else {
        debug!("No pod selector specified, selecting all pods");
    }

    let namespace = pod_object["metadata"]["namespace"]
        .as_str()
        .unwrap_or("unknown");

    cc.set_namespace(namespace.to_string());

    let podname = pod_object["metadata"]["name"].as_str().unwrap_or("unknown");

    cc.set_podname(podname.to_string());

    // Create the base zip file that we are going to put everything into
    let compression_method = if cc.compression {
        zip::CompressionMethod::Deflated
    } else {
        zip::CompressionMethod::Stored
    };
    let options = FileOptions::default()
        .compression_method(compression_method)
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

    match io::copy(&mut stdin, &mut zip) {
        Ok(v) => v,
        Err(e) => {
            error!("Error writing core file \n{}", e);
            process::exit(1);
        }
    };
    zip.flush()?;

    if cc.ignore_crio {
        if cc.core_events {
            let zip_name = format!("{}.zip", cc.get_templated_name());
            let evtdir = format!("{}", cc.event_location.display());
            let evt = CoreEvent::new_no_crio(cc.params, zip_name);
            evt.write_event(&evtdir)?;
        }
        zip.finish()?;
        file.unlock()?;
        process::exit(0);
    }

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
    let mut images: Vec<Value> = vec![];
    if let Some(containers) = ps_object["containers"].as_array() {
        for (counter, container) in containers.iter().enumerate() {
            let img_ref = match container["imageRef"].as_str() {
                Some(v) => v,
                None => {
                    error!("Failed to get containerid {}", "");
                    break;
                }
            };

            let container_id = container["id"].as_str().unwrap_or_default();
            debug!("Getting logs for container id {}", container_id);
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

            let img_clone = image.clone();
            images.push(img_clone);
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

            if cc.include_proc_info {
                debug!("Getting pid info for container id {}", container_id);

                let inspect = match cli.inspect_container(container_id) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Error inspecting container \n{}", e);
                        // We continue here since we do not want to interrupt the whole gathering
                        // flow because we could not inspect a container
                        continue;
                    }
                };

                let pid = match inspect["info"]["pid"].as_u64() {
                    Some(p) => p,
                    None => {
                        // We continue here since we do not want to interrupt the whole gathering
                        // flow because we could not extract pid from the container inspection
                        warn!("Failed to parse pid from inspect container, skipping");
                        continue;
                    }
                };

                debug!("Got pid {} for container", pid);

                debug!("Add proc files to the zip");

                // Gathering proc files means reading from the /proc/$pid folder to capture files
                // needed fore core2md to do its conversion. If for some reason a file is missing,
                // then that means that the container was evicted and has no longer a pid folder.
                // When this happens we do not want to abort the rest of data gathering as it can
                // contain useful information regardless of that containers process data.
                // So if any files failed to open, then log the error and continue on to the next container(s)
                let proc_folder_full_path = cc.get_proc_folder_full_path(counter);
                for filename in cc.get_proc_files_to_gather() {
                    let mut file = match File::open(format!(
                        "{}/{}/{}",
                        cc.system_proc_folder_path, pid, filename
                    )) {
                        Ok(f) => f,
                        Err(e) => {
                            warn!(
                                "Failed to open {}. Has the pod been ejected?\n{}",
                                filename, e
                            );
                            break;
                        }
                    };

                    let mut buffer = Vec::new();
                    if let Err(e) = file.read_to_end(&mut buffer) {
                        warn!("Failed read contents of the {} file \n{}", filename, e);
                        break;
                    }

                    if let Err(e) =
                        zip.start_file(format!("{}/{}", proc_folder_full_path, filename), options)
                    {
                        warn!("Error starting {} file in zip \n{}", filename, e);
                        break;
                    }

                    if let Err(e) = zip.write_all(buffer.as_slice()) {
                        warn!("Error writing {} file in zip \n{}", filename, e);
                        break;
                    }
                }

                debug!("Finished adding proc files to the zip");
            }
        }
    };

    zip.finish()?;
    file.unlock()?;
    if cc.core_events {
        let zip_name = format!("{}.zip", cc.get_templated_name());
        let evtdir = format!("{}", cc.event_location.display());
        let evt = CoreEvent::new(cc.params, zip_name, pod_object, images);
        evt.write_event(&evtdir)?;
    }
    Ok(())
}
