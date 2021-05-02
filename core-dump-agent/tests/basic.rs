use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
#[test]
fn basic() -> Result<(), std::io::Error> {
    let output_folder = format!("{}/{}", ".", "output");
    match Command::new("mkdir").arg("-p").arg(&output_folder).spawn() {
        Err(why) => panic!("couldn't spawn mkdir: {}", why),
        Ok(process) => process,
    };

    let path = env::current_dir()?;

    let host_path = format!("{}/{}", path.display(), "output");
    println!("{}", host_path);
    let cdc = Command::new("../target/debug/core-dump-agent")
        .env("HOST_DIR", &host_path)
        .output()?;
    println!("{}", std::str::from_utf8(&cdc.stdout).unwrap());
    println!("{}", std::str::from_utf8(&cdc.stderr).unwrap());
    println!("{}", cdc.status);

    let core_pattern = format!("{}/core_pattern.bak", host_path);
    let core_limit = format!("{}/core_pipe_limit.bak", host_path);
    let cdc_path = format!("{}/cdc", host_path);
    assert!(Path::new(core_pattern.as_str()).exists());
    assert!(Path::new(core_limit.as_str()).exists());
    assert!(Path::new(cdc_path.as_str()).exists());

    let output = Command::new("sysctl")
        .args(&["-n", "kernel.core_pattern"])
        .output()?;

    assert_eq!(
        std::str::from_utf8(&output.stdout).unwrap(),
        format!(
            "|{}/cdc -c=%c -e=%e -p=%p -s=%s -t=%t -d=$hostvol/core -h=%h -E=%E",
            host_path
        )
        .as_str()
    );
    fs::remove_dir_all("./output")?;
    Ok(())
}
