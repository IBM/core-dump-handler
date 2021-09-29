use clap::{App, Arg};
use log::{debug, error, info, LevelFilter};
use std::io;
use std::io::prelude::*;
use std::io::Write;
use std::process;
use std::process::{Command, Stdio};
use std::str::FromStr;
use uuid::Uuid;

fn main() -> Result<(), anyhow::Error> {
    // corezipfilename - the name of the zip file containing the core dump"
    // echo "  runtime - the runtime type - nodejs rust currently supported"
    // echo "  exename - the name of the executable to be debugged"
    // echo "  image - image of the crashed container"
    // echo "  namespace - namespace of core dump handler"

    let matches = match App::new("Core Dump CLI")
    .version("0.1.0")
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
                "the debug runtime to use. If not provided it will be derived from the exe name",
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
    ).arg(
        Arg::new("silent")
            .short('s')
            .long("silent")
            .required(false)
            .takes_value(true)
            .about("Set silent to silent mode."),
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

    let split_zip_name: Vec<&str> = core_zip_name.split("-").collect();
    let basenames: Vec<&str> = core_zip_name.split(".").collect();
    let basename = basenames[0];

    let core_location = format!("{}/{}.core", basename, basename);

    let pod_uuid = Uuid::new_v4();

    if core_exe_name == "" {
        core_exe_name = split_zip_name[split_zip_name.len() - 3];
    }
    let cmd = format!(
        "cp $(which '{}' | head -n 1) /shared; sleep infinity",
        core_exe_name
    );

    if runtime == "" {
        if core_exe_name == "node" {
            runtime = "nodejs";
        } else {
            runtime = "default";
        }
    }

    if runtime == "nodejs" {
        img_debug="quay.io/icdh/nodejs@sha256:ba165eabdfd63a668f41a47f9ffcc5c7a61ed618bfd0cb1dc65e27cc64308822"
    } else {
        img_debug = "quay.io/icdh/default"
    }

    if namespace == "" {
        namespace = "observe";
    }

    println!(
        "
         Debugging: {} 
         Runtime :{} 
         Namespace: {}
         Debug Image: {} 
         App Image: {}",
        core_exe_name, runtime, namespace, img_debug, image
    );
    debug!(
        "cmd: {}
            image: {}
            core_location {}",
        cmd, image, core_location
    );
    //a4e4da09-2d78-4402-b191-6d3e398f7df8-dump-1631309244-segfaulter-segfaulter-1-4.zip

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
    //kubectl apply -n $5 --output=jsonpath={.metadata.name} -f -
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

    // {
    //     Ok(v) => v,
    //     Err(e) => {
    //         error!("failed to execute kubectl apply {}", e);
    //         process::exit(1)
    //     }
    // };
    match pod_cmd.stdin.unwrap().write_all(pod.as_bytes()) {
        Err(why) => panic!("couldn't write to pod definition to stdin: {}", why),
        Ok(_) => println!("sent pod to kubectl"),
    }

    // let kubectl_error = String::from_utf8(pod_cmd.stderr)
    // .unwrap_or_default();
    // let pod_id = String::from_utf8(pod_output.stdout)
    //                 .unwrap_or_default()
    //                 .as_str();

    let mut kube_output = String::new();
    match pod_cmd.stdout.unwrap().read_to_string(&mut kube_output) {
        Err(why) => panic!("couldn't read kubectl stdout: {}", why),
        Ok(_) => print!("kubectl responded with:\n{}", kube_output),
    }

    let mut kubectl_error = String::new();
    match pod_cmd.stderr.unwrap().read_to_string(&mut kubectl_error) {
        Err(why) => panic!("couldn't read kubectl error: {}", why),
        Ok(_) => {
            if kubectl_error != "" {
                println!("Error Applying Pod:\n{}", kubectl_error);
            }
        }
    }

    Ok(())
}