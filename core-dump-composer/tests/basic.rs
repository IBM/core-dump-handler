use std::env;
use std::fs;
use std::fs::File;
use std::process::{Command, Stdio};

#[test]
fn run_with_mocks() -> Result<(), std::io::Error> {
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
    // cat the test core file to kcdd
    let cat = Command::new("cat")
        .env("PATH", &new_path)
        .arg("./mocks/test.core")
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .unwrap();

    let cdc = Command::new("../target/debug/core-dump-composer")
        .env("PATH", &new_path)
        .arg("-c")
        .arg("1000000000")
        .arg("-e")
        .arg("node")
        .arg("-p")
        .arg("4")
        .arg("-s")
        .arg("10")
        .arg("-E")
        .arg(current_path)
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
        let current_path = format!("{}", path.unwrap().path().display());
        if current_path.contains("dump-info.json") {
            println!("Testing: {}", current_path);
            let file = File::open(current_path).expect("file should open read only");
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
    }
    assert_eq!(6, file_counter);
    fs::remove_dir_all("./output")?;
    Ok(())
}
