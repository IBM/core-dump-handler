use std::env;
extern crate fs_extra;
use fs_extra::copy_items;
use fs_extra::dir::copy;
use fs_extra::dir::create_all;
use fs_extra::dir::CopyOptions;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;
#[test]
fn basic() -> Result<(), std::io::Error> {
    // Currently this test creates two folders in the root of the core-dump-agent project.
    // `vendor` acts as the container folder where everything is copied from
    // Output acts as the folder that would be created on the host node.
    let path = env::current_dir()?;
    println!("AGENT_TEST RootPath: {}", path.display());
    let local_path = format!("{}", path.display());
    let local_vendor_path = format!("{}/{}", path.display(), "vendor");
    let host_path = format!("{}/{}", path.display(), "output");
    let core_path = format!("{}/{}", path.display(), "output/cores");
    let home_path = format!("{}/{}", path.display(), "output/home");
    let mocks_path = format!("{}/{}", path.display(), "mocks/vendor");
    let sysctl_path = format!("{}/{}", path.display(), "mocks/sysctl");
    let options = CopyOptions::new();
    if Path::new(&host_path).exists() {
        fs::remove_dir_all(&host_path)?;
    }
    if Path::new(&local_vendor_path).exists() {
        fs::remove_dir_all(&local_vendor_path)?;
    }
    create_all(&host_path, false).unwrap();
    println!("AGENT_TEST Made host folder : {}", host_path);
    create_all(&core_path, false).unwrap();
    println!("AGENT_TEST Made core folder : {}", core_path);
    create_all(&home_path, false).unwrap();
    println!("AGENT_TEST Made home folder : {}", home_path);

    let mut from_paths = Vec::new();
    from_paths.push(&sysctl_path);
    copy_items(&from_paths, &home_path, &options).unwrap();
    copy(&mocks_path, &local_path, &options).unwrap();
    println!("AGENT_TEST Copied mocks to : {}", home_path);

    let mut cda = Command::new("../target/debug/core-dump-agent")
        .env("HOST_DIR", &home_path)
        .env("DEPLOY_CRIO_EXE", "false")
        .env("SUID_DUMPABLE", "2")
        .env("CORE_DIR", &core_path)
        .env("LOCAL_BIN", &home_path)
        .env("S3_BUCKET_NAME", "safdasdf")
        .env("S3_REGION", "asfdasdf")
        .env("S3_ACCESS_KEY", "asdfads")
        .env("S3_SECRET", "asfdds")
        .env("SCHEDULE", "1/1 * * * * *")
        .spawn()?;
    thread::sleep(Duration::from_secs(11));
    // This is very cludgy way of proceeding with the test.
    // The initialization and the running uploader should be split into two process.
    // and most of the code should be moved into structs to make it more testable.
    // This work maybe be undertaken as part of the operator work
    // https://github.com/IBM/core-dump-operator/issues/1
    cda.kill().expect("!kill");

    let core_pattern = format!("{}/core_pattern.bak", &home_path);
    let core_limit = format!("{}/core_pipe_limit.bak", &home_path);
    let suid_dumpable = format!("{}/suid_dumpable.bak", &home_path);
    let cdc_path = format!("{}/cdc", &home_path);

    assert!(Path::new(core_pattern.as_str()).exists());
    assert!(Path::new(core_limit.as_str()).exists());
    assert!(Path::new(suid_dumpable.as_str()).exists());
    assert!(Path::new(cdc_path.as_str()).exists());

    let core_pattern_content = fs::read_to_string(&core_pattern).unwrap();
    assert_eq!(
        core_pattern_content,
        "|/usr/lib/systemd/systemd-coredump %P %u %g %s %t %c %h %e".to_string()
    );

    let core_limit_content = fs::read_to_string(&core_limit).unwrap();
    assert_eq!(core_limit_content, "16".to_string());

    let suid_dumpable_content = fs::read_to_string(&suid_dumpable).unwrap();
    assert_eq!(suid_dumpable_content, "0".to_string());

    let env_file = format!("{}/{}", &home_path, ".env");
    let env_content = fs::read_to_string(&env_file).unwrap();
    assert!(env_content.contains("LOG_LEVEL=debug"));
    assert!(env_content.contains("IGNORE_CRIO=false"));
    assert!(env_content.contains("CRIO_IMAGE_CMD=img"));
    assert!(env_content.contains("USE_CRIO_CONF=false"));
    assert!(env_content.contains(
        "FILENAME_TEMPLATE={uuid}-dump-{timestamp}-{hostname}-{exe_name}-{pid}-{signal}"
    ));
    assert!(env_content.contains("LOG_LENGTH=500"));
    assert_eq!(env_content.lines().count(), 6);
    //TODO: [No9] Test uploading of a corefile
    //TODO: [No9] Test remove option
    //TODO: [No9] Test sweep option
    fs::remove_dir_all(&host_path)?;
    fs::remove_dir_all(&local_vendor_path)?;
    Ok(())
}
