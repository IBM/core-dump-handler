use std::env;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::process::{Command, Stdio};
#[test]
fn namespaced_files_scenario() -> Result<(), std::io::Error> {
    let current_dir = env::current_dir()?;

    println!("The current directory is {}", current_dir.display());
    // Need to append to path
    let key = "PATH";
    let mut current_path = String::new();
    match env::var(key) {
        Ok(val) => current_path = val,
        Err(e) => println!("couldn't interpret {}: {}", key, e),
    }
    let new_path = format!(
        "{}/mocks:{}/target/debug:{}",
        current_dir.display(),
        current_dir.display(),
        current_path
    );
    println!("Running tests using this PATH: {}", new_path);
    let output_folder = format!("{}/{}", ".", "output");
    // Make a directory to store the generated zip file
    let _mkdir = match Command::new("mkdir").arg("-p").arg(&output_folder).spawn() {
        Err(why) => panic!("couldn't spawn mkdir: {}", why),
        Ok(process) => process,
    };
    // copy crictl to base_folder
    Command::new("cp")
        .arg("-f")
        .arg("./mocks/crictl-default.sh")
        .arg("../target/debug/crictl")
        .output()
        .expect("cp failed");

    // cat the test core file to process.
    let cat = Command::new("cat")
        .env("PATH", &new_path)
        .arg("./mocks/test.core")
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .unwrap();

    let cdc = Command::new("../target/debug/core-dump-composer")
        .env("FILENAME_TEMPLATE", "{namespace}")
        .arg("-c")
        .arg("1000000000")
        .arg("-e")
        .arg("node")
        .arg("-p")
        .arg("4")
        .arg("-s")
        .arg("10")
        .arg("-E")
        .arg("/target/debug/core-dump-composer")
        .arg("-d")
        .arg(&output_folder)
        .arg("-t")
        .arg("1588462466")
        .arg("-h")
        .arg("crashing-app-699c49b4ff-86wrh")
        .stdin(cat)
        .output()
        .expect("failed to execute core dump composer");

    println!("{}", String::from_utf8_lossy(&cdc.stdout));
    println!("{}", String::from_utf8_lossy(&cdc.stderr));

    Command::new("unzip")
        .arg("output/*.zip")
        .arg("-d")
        .arg("output")
        .output()
        .expect("unzip failed");

    let paths = fs::read_dir("./output").unwrap();
    println!("{:?}", paths);
    // Test to see if files are available
    let mut file_counter = 0;
    for path in paths {
        file_counter = file_counter + 1;
        let l_path = path.unwrap().path();
        let current_path = format!("{}", l_path.display());
        if current_path.contains("dump-info.json") {
            let file_stem = l_path.file_stem();
            assert_eq!(file_stem, Some(OsStr::new("default-dump-info")));
            let extension = l_path.extension();
            assert_eq!(extension, Some(OsStr::new("json")));
            let l_current_path = current_path.clone();
            println!("Testing: {}", l_current_path);
            let file = File::open(l_current_path).expect("file should open read only");
            let json: serde_json::Value =
                serde_json::from_reader(file).expect("file should be proper JSON");
            //test static properties
            let host_name = json
                .get("hostname")
                .expect("dump-info.json should have hostname key");
            assert_eq!("crashing-app-699c49b4ff-86wrh", host_name);
            let exe = json.get("exe").expect("dump-info.json should have exe key");
            assert_eq!("node", exe);
            let real_pid = json
                .get("real_pid")
                .expect("dump-info.json should have real_pid key");
            assert_eq!("4", real_pid);
            let signal = json
                .get("signal")
                .expect("dump-info.json should have signal key");
            assert_eq!("10", signal);
        }
        if current_path.contains("image-info.json") {
            let file_stem = l_path.file_stem();
            assert_eq!(file_stem, Some(OsStr::new("default-0-image-info")));
            let extension = l_path.extension();
            assert_eq!(extension, Some(OsStr::new("json")));
            let l_current_path = current_path.clone();
            println!("Testing: {}", l_current_path);
            let file = File::open(l_current_path).expect("file should open read only");
            let json: serde_json::Value =
                serde_json::from_reader(file).expect("file should be proper JSON");
            //test static properties
            let repo_digest = json
                .get("repoDigests")
                .unwrap()
                .as_array()
                .expect("image-info should have a repoDigests");
            assert_eq!("docker.io/number9/example-crashing-nodejs-app@sha256:b8fea40ed9da77307702608d1602a812c5983e0ec0b788fc6298985a40be3800", repo_digest[0].as_str().unwrap());
            let size = json.get("size").expect("image-info should have a size");
            assert_eq!("338054458", size);
            let repo_tags = json
                .get("repoTags")
                .unwrap()
                .as_array()
                .expect("image-info should have repoTags");
            assert_eq!(
                "docker.io/number9/example-crashing-nodejs-app:latest",
                repo_tags[0].as_str().unwrap()
            );
        }

        if current_path.contains(".log") {
            let file_stem = l_path.file_stem();
            assert_eq!(file_stem, Some(OsStr::new("default-0")));
            let extension = l_path.extension();
            assert_eq!(extension, Some(OsStr::new("log")));

            let l_current_path = current_path.clone();
            println!("Testing: {}", l_current_path);
            let contents = fs::read_to_string(l_current_path)?;
            assert_eq!("A LOG\n", contents.as_str());
        }
        if current_path.contains(".core") {
            let file_stem = l_path.file_stem();
            assert_eq!(file_stem, Some(OsStr::new("default")));
            let extension = l_path.extension();
            assert_eq!(extension, Some(OsStr::new("core")));

            let l_current_path = current_path.clone();
            println!("Testing: {}", l_current_path);
            let diff = Command::new("diff")
                .arg("./mocks/test.core")
                .arg(l_current_path)
                .output()
                .expect("diff failed");
            println!("{}", String::from_utf8_lossy(&diff.stdout));
            assert!(String::from_utf8_lossy(&diff.stdout).is_empty());
        }
        if current_path.contains("pod-info.json") {
            let file_stem = l_path.file_stem();
            assert_eq!(file_stem, Some(OsStr::new("default-pod-info")));
            let extension = l_path.extension();
            assert_eq!(extension, Some(OsStr::new("json")));
        }
        if current_path.contains("runtime-info.json") {
            let file_stem = l_path.file_stem();
            assert_eq!(file_stem, Some(OsStr::new("default-runtime-info")));
            let extension = l_path.extension();
            assert_eq!(extension, Some(OsStr::new("json")));
        }
        if current_path.contains("ps-info.json") {
            let file_stem = l_path.file_stem();
            assert_eq!(file_stem, Some(OsStr::new("default-ps-info")));
            let extension = l_path.extension();
            assert_eq!(extension, Some(OsStr::new("json")));
        }
        if current_path.contains(".zip") {
            let file_stem = l_path.file_stem();
            assert_eq!(file_stem, Some(OsStr::new("default")));
            let extension = l_path.extension();
            assert_eq!(extension, Some(OsStr::new("zip")));
        }
    }
    assert_eq!(8, file_counter);
    fs::remove_dir_all("./output")?;
    Ok(())
}
