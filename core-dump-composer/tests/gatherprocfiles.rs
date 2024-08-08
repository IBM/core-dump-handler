use std::env;
use std::fs;
use std::fs::File;
use std::process::{Command, Stdio};

#[test]
fn gather_proc_files_scenario() -> Result<(), std::io::Error> {
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
        .arg("./mocks/crictl-gatherprocfiles.sh")
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
        .env("INCLUDE_PROC_INFO", "true")
        .env(
            "OVERRIDE_PROC_FOLDER_PATH",
            format!("{}/mocks/proc", current_dir.display()),
        )
        .env("LOG_LEVEL", "Debug")
        .arg("-c")
        .arg("1000000000")
        .arg("-e")
        .arg("node")
        .arg("-p")
        .arg("4")
        .arg("-s")
        .arg("10")
        .arg("-E")
        .arg("!target!debug!core-dump-composer")
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

    let mut proc_folder_path: Option<String> = None;
    // Test to see if files are available
    let mut file_counter = 0;
    for path in paths {
        file_counter = file_counter + 1;
        let current_path = format!("{}", path.unwrap().path().display());
        if current_path.contains("dump-info.json") {
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
            let dump_name = json
                .get("dump_file")
                .expect("dump-info.json should have dump_name key");
            assert!(dump_name.to_string().contains("4-10.core"));

            let path = json
                .get("path")
                .expect("dump-info.json should have path key");
            assert!(path
                .to_string()
                .contains("!target!debug!core-dump-composer"));
        }
        if current_path.contains("image-info.json") {
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
            let l_current_path = current_path.clone();
            println!("Testing: {}", l_current_path);
            let contents = fs::read_to_string(l_current_path)?;
            assert_eq!("A LOG\n", contents.as_str());
        }
        if current_path.contains(".core") {
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

        if current_path.contains("proc") {
            proc_folder_path = Some(current_path.clone());
        }
    }
    assert_eq!(9, file_counter);
    assert_eq!(true, proc_folder_path.is_some());

    let paths = fs::read_dir(proc_folder_path.unwrap()).unwrap();
    println!("{:?}", paths);
    // Test to see if the proc files are available
    let mut file_counter = 0;
    for path in paths {
        file_counter = file_counter + 1;
        let current_path = format!("{}", path.unwrap().path().display());
        if current_path.contains("auxv") {
            let l_current_path = current_path.clone();
            println!("Testing: {}", l_current_path);
            let contents = fs::read_to_string(l_current_path)?;
            assert_eq!("auxv file\n", contents.as_str());
        }
        if current_path.contains("cmdline") {
            let l_current_path = current_path.clone();
            println!("Testing: {}", l_current_path);
            let contents = fs::read_to_string(l_current_path)?;
            assert_eq!("cmdline file\n", contents.as_str());
        }
        if current_path.contains("environ") {
            let l_current_path = current_path.clone();
            println!("Testing: {}", l_current_path);
            let contents = fs::read_to_string(l_current_path)?;
            assert_eq!("environ file\n", contents.as_str());
        }
        if current_path.contains("maps") {
            let l_current_path = current_path.clone();
            println!("Testing: {}", l_current_path);
            let contents = fs::read_to_string(l_current_path)?;
            assert_eq!("maps file\n", contents.as_str());
        }
        if current_path.contains("status") {
            let l_current_path = current_path.clone();
            println!("Testing: {}", l_current_path);
            let contents = fs::read_to_string(l_current_path)?;
            assert_eq!("status file\n", contents.as_str());
        }
    }
    assert_eq!(5, file_counter);

    fs::remove_dir_all("./output")?;
    Ok(())
}
