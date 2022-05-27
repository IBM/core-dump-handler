extern crate dotenv;

use clap::{App, Arg, ArgMatches};
use libcrio::ImageCommand;
use log::error;
use serde::Serialize;
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::str::FromStr;
use tinytemplate::TinyTemplate;
use uuid::Uuid;

#[derive(Serialize)]
pub struct CoreConfig {
    pub dot_env_path: PathBuf,
    pub base_path: PathBuf,
    pub crictl_config_path: PathBuf,
    pub log_level: String,
    pub log_length: u32,
    pub use_crio_config: bool,
    pub ignore_crio: bool,
    pub image_command: ImageCommand,
    pub bin_path: String,
    pub os_hostname: String,
    pub filename_template: String,
    pub params: CoreParams,
}

#[derive(Serialize)]
pub struct CoreParams {
    pub limit_size: String,
    pub exe_name: String,
    pub pid: String,
    pub signal: String,
    pub timestamp: String,
    pub directory: String,
    pub hostname: String,
    pub pathname: String,
    pub namespace: Option<String>,
    pub podname: Option<String>,
    pub uuid: Uuid,
}

static DEFAULT_TEMPLATE: &str = "{uuid}-dump-{timestamp}-{hostname}-{exe_name}-{pid}-{signal}";

impl CoreConfig {
    pub fn new() -> Result<CoreConfig, anyhow::Error> {
        let matches = try_get_matches()?;
        let limit_size = matches.value_of("limit-size").unwrap_or("").to_string();
        let exe_name = matches.value_of("exe-name").unwrap_or("").to_string();
        let pid = matches.value_of("pid").unwrap_or("").to_string();
        let signal = matches.value_of("signal").unwrap_or("").to_string();
        let timestamp = matches.value_of("timestamp").unwrap_or("").to_string();
        let directory = matches.value_of("directory").unwrap_or("").to_string();
        let hostname = matches.value_of("hostname").unwrap_or("").to_string();
        let pathname = matches.value_of("pathname").unwrap_or("").to_string();
        let uuid = Uuid::new_v4();

        let params = CoreParams {
            limit_size,
            exe_name,
            pid,
            signal,
            timestamp,
            directory,
            hostname,
            pathname,
            namespace: None,
            podname: None,
            uuid,
        };

        let mut dot_env_path = env::current_exe()?;
        dot_env_path.pop();
        dot_env_path.push(".env");

        match dotenv::from_path(dot_env_path.clone()) {
            Ok(v) => v,
            Err(e) => error!("error loading .env file {}", e),
        }

        let mut crictl_config_path = env::current_exe()?;
        crictl_config_path.pop();
        crictl_config_path.push("crictl.yaml");
        let mut base_path = env::current_exe()?;
        base_path.pop();

        let log_level = env::var("LOG_LEVEL").unwrap_or_default();
        let ignore_crio = env::var("IGNORE_CRIO")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            .parse::<bool>()
            .unwrap();
        let log_length = env::var("LOG_LENGTH")
            .unwrap_or_else(|_| "500".to_string())
            .parse::<u32>()
            .unwrap();
        let image_command_string = env::var("CRIO_IMAGE_CMD").unwrap_or_else(|_| "img".to_string());
        let use_crio_config = env::var("USE_CRIO_CONF")
            .unwrap_or_else(|_| "false".to_string().to_lowercase())
            .parse::<bool>()
            .unwrap();
        let os_hostname = hostname::get()
            .unwrap_or_else(|_| OsString::from_str("unknown").unwrap_or_default())
            .into_string()
            .unwrap_or_else(|_| "unknown".to_string());

        let base_path_str = base_path
            .clone()
            .into_os_string()
            .into_string()
            .unwrap_or_else(|_| "/var/mnt/core-dump-handler".to_string());

        let bin_path = format!(
            "/bin:/sbin:/usr/bin:/usr/sbin:/usr/local/bin:/home/kubernetes/bin:{}",
            base_path_str
        );
        let image_command =
            ImageCommand::from_str(&image_command_string).unwrap_or(ImageCommand::Img);
        let filename_template =
            env::var("FILENAME_TEMPLATE").unwrap_or_else(|_| String::from(DEFAULT_TEMPLATE));

        Ok(CoreConfig {
            log_level,
            ignore_crio,
            dot_env_path,
            image_command,
            use_crio_config,
            crictl_config_path,
            base_path,
            bin_path,
            os_hostname,
            filename_template,
            log_length,
            params,
        })
    }

    pub fn get_dump_info(&self) -> String {
        format!(
            "{{\"uuid\":\"{}\", \"dump_file\":\"{}\", \"timestamp\": \"{}\",
        \"hostname\": \"{}\", \"exe\": \"{}\", \"real_pid\": \"{}\", \"signal\": \"{}\", \"node_hostname\": \"{}\", \"path\": \"{}\" }}",
            self.params.uuid, self.get_core_filename(), self.params.timestamp, self.params.hostname, self.params.exe_name, self.params.pid, self.params.signal, self.os_hostname, self.params.pathname
        )
    }

    pub fn get_templated_name(&self) -> String {
        let mut tt = TinyTemplate::new();
        match tt.add_template("name", &self.filename_template) {
            Ok(v) => v,
            Err(e) => {
                error!(
                    "Adding template failed. Using uuid {} {}",
                    self.params.uuid.to_string(),
                    e
                );
                return self.params.uuid.to_string();
            }
        }
        match tt.render("name", &self.params) {
            Ok(v) => v,
            Err(e) => {
                error!(
                    "Templating name failed. Using uuid {} {}",
                    self.params.uuid.to_string(),
                    e
                );
                self.params.uuid.to_string()
            }
        }
    }
    pub fn set_namespace(&mut self, namespace: String) {
        self.params.namespace = Some(namespace)
    }

    pub fn set_podname(&mut self, podname: String) {
        self.params.podname = Some(podname)
    }

    pub fn get_dump_info_filename(&self) -> String {
        format!("{}-dump-info.json", self.get_templated_name())
    }

    pub fn get_core_filename(&self) -> String {
        format!("{}.core", self.get_templated_name())
    }

    pub fn get_pod_filename(&self) -> String {
        format!("{}-pod-info.json", self.get_templated_name())
    }

    pub fn get_inspect_pod_filename(&self) -> String {
        format!("{}-runtime-info.json", self.get_templated_name())
    }

    pub fn get_ps_filename(&self) -> String {
        format!("{}-ps-info.json", self.get_templated_name())
    }

    pub fn get_image_filename(&self, counter: u32) -> String {
        format!("{}-{}-image-info.json", self.get_templated_name(), counter)
    }

    pub fn get_log_filename(&self, counter: u32) -> String {
        format!("{}-{}.log", self.get_templated_name(), counter)
    }
    pub fn get_zip_full_path(&self) -> String {
        format!(
            "{}/{}.zip",
            self.params.directory,
            self.get_templated_name()
        )
    }
}

pub fn try_get_matches() -> clap::Result<ArgMatches> {
    App::new("Core Dump Composer")
        .version("0.1.0")
        .author("Anton Whalley <anton@venshare.com>")
        .about("Processes Core Dumps in a K8s System")
        .arg(
            Arg::new("limit-size")
                .short('c')
                .long("limit-size")
                .required(false)
                .takes_value(true)
                .help("Core file size soft resource limit of crashing process"),
        )
        .arg(
            Arg::new("exe-name")
                .short('e')
                .long("exe-name")
                .required(false)
                .takes_value(true)
                .help(
                    "The process or thread's comm value, which typically is the
            same as the executable filename (without path prefix, and
            truncated to a maximum of 15 characters)",
                ),
        )
        .arg(
            Arg::new("pid")
                .short('p')
                .long("pid")
                .required(false)
                .takes_value(true)
                .help(
                    "PID of dumped process, as seen in the PID namespace in which
            the process resides.",
                ),
        )
        .arg(
            Arg::new("signal")
                .short('s')
                .long("signal")
                .required(false)
                .takes_value(true)
                .help("Number of signal causing dump."),
        )
        .arg(
            Arg::new("timestamp")
                .short('t')
                .long("timestamp")
                .required(false)
                .takes_value(true)
                .help("Time of dump, expressed as seconds since the Epoch."),
        )
        .arg(
            Arg::new("directory")
                .short('d')
                .long("dir")
                .required(false)
                .takes_value(true)
                .help("Directory to save the core dump to."),
        )
        .arg(
            Arg::new("hostname")
                .short('h')
                .long("hostname")
                .required(false)
                .takes_value(true)
                .help("Hostname (same as nodename returned by uname(2))"),
        )
        .arg(
            Arg::new("pathname")
                .short('E')
                .long("pathname")
                .required(false)
                .takes_value(true)
                .help("Hostname (same as nodename returned by uname(2))"),
        )
        .arg(
            Arg::new("test-threads")
                .long("test-threads")
                .required(false)
                .takes_value(true)
                .help("test-threads mapped to support the test scenarios"),
        )
        .try_get_matches()
}

#[cfg(test)]
mod tests {
    use crate::config::CoreConfig;
    #[test]
    fn namespace_is_rendered() {
        let mut config = match CoreConfig::new() {
            Ok(v) => v,
            Err(e) => panic!("Generation of CoreConfig failed. {}", e),
        };
        config.set_namespace("anamespace".to_string());
        let templated_name = config.get_templated_name();
        assert!(templated_name.contains("-dump-----"));
        config.filename_template = "{namespace}".to_string();
        let just_namespace = config.get_templated_name();
        assert_eq!(just_namespace, "anamespace".to_string());
    }
    #[test]
    fn default_template_test() {
        // "{uuid}-dump-{timestamp}-{hostname}-{exe_name}-{pid}-{signal}";
        let mut config = match CoreConfig::new() {
            Ok(v) => v,
            Err(e) => panic!("Generation of CoreConfig failed. {}", e),
        };
        config.params.timestamp = "123123123".to_string();
        config.params.hostname = "ahostname".to_string();
        config.params.exe_name = "anexe".to_string();
        config.params.pid = "2".to_string();
        config.params.signal = "9".to_string();

        let templated_name = config.get_templated_name();
        assert!(templated_name.contains("-dump-123123123-ahostname-anexe-2-9"));
    }
    #[test]
    fn get_files_test() {
        let mut config = match CoreConfig::new() {
            Ok(v) => v,
            Err(e) => panic!("Generation of CoreConfig failed. {}", e),
        };
        config.params.timestamp = "123123123".to_string();
        config.params.hostname = "ahostname".to_string();
        config.params.exe_name = "anexe".to_string();
        config.params.pid = "2".to_string();
        config.params.signal = "9".to_string();
        let dump_info_name = config.get_dump_info_filename();
        assert!(dump_info_name.contains("-dump-123123123-ahostname-anexe-2-9-dump-info.json"));

        let core_file_name = config.get_core_filename();
        assert!(core_file_name.contains("-dump-123123123-ahostname-anexe-2-9.core"));

        let pod_file_name = config.get_pod_filename();
        assert!(pod_file_name.contains("-dump-123123123-ahostname-anexe-2-9-pod-info.json"));

        let inspect_file_name = config.get_inspect_pod_filename();
        assert!(inspect_file_name.contains("-dump-123123123-ahostname-anexe-2-9-runtime-info.json"));

        let inspect_file_name = config.get_inspect_pod_filename();
        assert!(inspect_file_name.contains("-dump-123123123-ahostname-anexe-2-9-runtime-info.json"));

        let ps_file_name = config.get_ps_filename();
        assert!(ps_file_name.contains("-dump-123123123-ahostname-anexe-2-9-ps-info.json"));

        let img_file_name = config.get_image_filename(0);
        assert!(img_file_name.contains("-dump-123123123-ahostname-anexe-2-9-0-image-info.json"));

        let log_file_name = config.get_log_filename(0);
        assert!(log_file_name.contains("-dump-123123123-ahostname-anexe-2-9-0.log"));

        let zip_file_name = config.get_zip_full_path();
        assert!(zip_file_name.contains("-dump-123123123-ahostname-anexe-2-9.zip"));
    }
}
