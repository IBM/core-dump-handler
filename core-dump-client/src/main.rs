use clap::{App, Arg};
use log::{debug, error};
use std::io::prelude::*;
use std::io::Write;
use std::process;
use std::process::{Command, Stdio};
use std::{thread, time};
use uuid::Uuid;

fn main() -> Result<(), anyhow::Error> {
    let matches = match App::new("Core Dump CLI")
    .version("0.2.0")
    .author("Anton Whalley <anton@venshare.com>")
    .about("Analyse Core Dumps in a K8s System")
    .arg(
        Arg::new("corezipfilename")
            .short('c')
            .long("corezipfilename")
            .required(true)
            .takes_value(true)
            .about("the name of the zip file containing the core dump"),
    )
    .arg(
        Arg::new("exe-name")
            .short('e')
            .long("exe-name")
            .required(false)
            .takes_value(true)
            .about(
                "the name of the executable to be debugged. If not provided it will be derived from zip file name",
            ),
    )
    .arg(
        Arg::new("runtime")
            .short('r')
            .long("runtime")
            .required(false)
            .takes_value(true)
            .about(
                "the debug runtime to use. If not provided it will be derived from the exe name. java nodejs and default",
            ),
    )
    .arg(
        Arg::new("image")
            .short('i')
            .long("image")
            .required(true)
            .takes_value(true)
            .about("The image of the crashed container"),
    )
    .arg(
        Arg::new("namespace")
            .short('n')
            .long("namespace")
            .required(false)
            .takes_value(true)
            .about("The namespace containing the core-dump-handler. Defaults to observe if not supplied"),
    )
    .try_get_matches()
{
    Ok(v) => v,
    Err(e) => {
        error!("Incorrect Parameters: {}", e);
        e.exit()
    }
};

    let mut core_exe_name = matches.value_of("exe-name").unwrap_or("");
    let core_zip_name = matches.value_of("corezipfilename").unwrap_or("");
    let mut runtime = matches.value_of("runtime").unwrap_or("");
    let image = matches.value_of("image").unwrap_or("");
    let img_debug;
    let mut namespace = matches.value_of("namespace").unwrap_or("");

    // Extracting crash information based on file name:println
    // e.g.a4e4da09-2d78-4402-b191-6d3e398f7df8-dump-1631309244-segfaulter-segfaulter-1-4.zip

    let split_zip_name: Vec<&str> = core_zip_name.split('-').collect();
    let basenames: Vec<&str> = core_zip_name.split('.').collect();
    let basename = basenames[0];

    let core_location = format!("{}/{}.core", basename, basename);

    let pod_uuid = Uuid::new_v4();

    if core_exe_name.is_empty() {
        core_exe_name = split_zip_name[split_zip_name.len() - 3];
    }
    let cmd = format!(
        "cp $(which '{}' | head -n 1) /shared; sleep infinity",
        core_exe_name
    );

    if runtime.is_empty() {
        if core_exe_name == "node" {
            runtime = "nodejs";
        } else if core_exe_name == "java" {
            runtime = "java";
        } else {
            runtime = "default";
        }
    }

    if runtime == "nodejs" {
        img_debug = "quay.io/icdh/nodejs"
    } else if runtime == "java" {
        img_debug = "quay.io/icdh/java";
    } else {
        img_debug = "quay.io/icdh/default"
    }

    if namespace.is_empty() {
        namespace = "observe";
    }

    println!(
        "
Debugging: {} 
Runtime: {} 
Namespace: {}
Debug Image: {} 
App Image: {}",
        core_exe_name, runtime, namespace, img_debug, image
    );
    debug!(
        "
cmd: {}
image: {}
core_location {}",
        cmd, image, core_location
    );

    let pod = format!(
        r#"
apiVersion: v1
kind: Pod
metadata:
    name: debugger-{uuid}
spec:
  restartPolicy: Never
  volumes:
  - name: shared-data
    emptyDir: {{}}
  containers:
  - name: debug-container
    image: {img_debug}
    volumeMounts:
    - name: shared-data
      mountPath: /shared
    command: ["./init.sh"]
    env:
      - name: S3_ACCESS_KEY
        valueFrom:
          secretKeyRef:
            name: s3config
            key: s3AccessKey
      - name: S3_SECRET
        valueFrom:
          secretKeyRef:
            name: s3config
            key: s3Secret
      - name: S3_BUCKET_NAME
        valueFrom:
          secretKeyRef:
            name: s3config
            key: s3BucketName
      - name: S3_REGION
        valueFrom:
          secretKeyRef:
            name: s3config
            key: s3Region
      - name: CORE_FILE
        value: {core_zip_name}
      - name: EXE_LOCATION
        value: /shared/{core_exe_name}
      - name: CORE_LOCATION
        value: {core_location}
  - name: core-container
    image: {image}
    command: ["/bin/sh"]
    args: ["-c", {cmd}]
    volumeMounts:
    - name: shared-data
      mountPath: /shared
"#,
        uuid = pod_uuid,
        img_debug = img_debug,
        core_zip_name = core_zip_name,
        core_exe_name = core_exe_name,
        core_location = core_location,
        image = image,
        cmd = cmd
    );

    let pod_cmd = match Command::new("kubectl")
        .args(&[
            "apply",
            "-n",
            namespace,
            "--output=jsonpath={.metadata.name}",
            "-f",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Err(why) => panic!("couldn't spawn kubectl: {}", why),
        Ok(process) => process,
    };

    match pod_cmd.stdin.unwrap().write_all(pod.as_bytes()) {
        Err(why) => panic!("couldn't write to pod definition to stdin: {}", why),
        Ok(_) => println!("Sending pod config using kubectl"),
    }

    let mut kube_output = String::new();
    match pod_cmd.stdout.unwrap().read_to_string(&mut kube_output) {
        Err(why) => panic!("couldn't read kubectl stdout: {}", why),
        Ok(_) => {
            if !kube_output.is_empty() {
                println!("stdout: {}", kube_output);
            }
        }
    }

    let mut kubectl_error = String::new();
    match pod_cmd.stderr.unwrap().read_to_string(&mut kubectl_error) {
        Err(why) => panic!("couldn't read kubectl error: {}", why),
        Ok(_) => {
            if !kubectl_error.is_empty() {
                println!("stderr:\n{}", kubectl_error);
                process::exit(1);
            }
        }
    }
    let mut connected = false;

    while !connected {
        let debug_status = match Command::new("kubectl")
            .args(&[
                "exec",
                "-it",
                kube_output.as_str(),
                "-n",
                namespace,
                "--",
                "/bin/bash",
            ])
            .status()
        {
            Err(why) => panic!("couldn't spawn kubectl: {}", why),
            Ok(process) => process,
        };
        if debug_status.code().unwrap_or(1) == 0 {
            connected = true;
        } else {
            println!("\nRetrying connection...");
            thread::sleep(time::Duration::from_secs(3));
        }
    }

    let destroy_status = match Command::new("kubectl")
        .args(&["delete", "pod", kube_output.as_str(), "-n", namespace])
        .status()
    {
        Err(why) => panic!("couldn't spawn kubectl: {}", why),
        Ok(process) => process,
    };

    if destroy_status.code().unwrap_or(1) == 1 {
        println!("Failed to delete container");
        println!(
            "Try running: kubectl delete pod {} -n {}",
            kube_output.as_str(),
            namespace
        )
    }

    Ok(())
}
