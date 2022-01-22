use clap::{App, Arg, ArgMatches};
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;

pub struct CoreConfig {
    pub dot_env_path: PathBuf,
    pub zip_path: PathBuf,
    pub base_path: PathBuf,
    pub crictl_config_path: PathBuf,
    pub log_level: String,
    pub use_crio_config: bool,
    pub ignore_crio: bool,
    pub image_command: String,
    pub dump_name: String,
    pub bin_path: String,
    pub os_hostname: String,
    pub params: CoreParams,
}

pub struct CoreParams {
    pub limit_size: String,
    pub exe_name: String,
    pub pid: String,
    pub signal: String,
    pub timestamp: String,
    pub directory: String,
    pub hostname: String,
    pub pathname: String,
    pub uuid: Uuid,
}

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
            uuid,
        };

        let dump_name = format!(
            "{}-dump-{}-{}-{}-{}-{}",
            uuid, params.timestamp, params.hostname, params.exe_name, params.pid, params.signal
        );
        let mut dot_env_path = env::current_exe()?;
        dot_env_path.pop();
        dot_env_path.push(".env");

        let mut crictl_config_path = env::current_exe()?;
        crictl_config_path.pop();
        crictl_config_path.push("crictl.yaml");
        let mut base_path = env::current_exe()?;
        base_path.pop();

        let zip_path = format!("{}/{}.zip", params.directory, dump_name);
        let zip_path = std::path::PathBuf::from(&zip_path);

        let log_level = env::var("LOG_LEVEL").unwrap_or_default();
        let ignore_crio = env::var("IGNORE_CRIO")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            .parse::<bool>()
            .unwrap();
        let image_command = env::var("CRIO_IMAGE_CMD").unwrap_or_else(|_| "img".to_string());
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

        Ok(CoreConfig {
            log_level,
            ignore_crio,
            dot_env_path,
            image_command,
            use_crio_config,
            zip_path,
            crictl_config_path,
            base_path,
            bin_path,
            dump_name,
            os_hostname,
            params,
        })
    }

    pub fn get_dump_info(&self) -> String {
        format!(
            "{{\"uuid\":\"{}\", \"dump_file\":\"{}.core\", \"timestamp\": \"{}\",
        \"hostname\": \"{}\", \"exe\": \"{}\", \"real_pid\": \"{}\", \"signal\": \"{}\", \"node_hostname\": \"{}\" }}",
            self.params.uuid, self.dump_name, self.params.timestamp, self.params.hostname, self.params.exe_name, self.params.pid, self.params.signal, self.os_hostname
        )
    }

    pub fn get_dump_info_filename(&self) -> String {
        format!("{}-dump-info.json", self.dump_name)
    }

    pub fn get_core_filename(&self) -> String {
        format!("{}.core", self.dump_name)
    }

    pub fn get_pod_filename(&self) -> String {
        format!("{}-pod-info.json", self.dump_name)
    }

    pub fn get_inspect_pod_filename(&self) -> String {
        format!("{}-runtime-info.json", self.dump_name)
    }

    pub fn get_ps_filename(&self) -> String {
        format!("{}-ps-info.json", self.dump_name)
    }

    pub fn get_image_filename(&self, counter: u32) -> String {
        format!("{}-{}-image-info.json", self.dump_name, counter)
    }

    pub fn get_log_filename(&self, counter: u32) -> String {
        format!("{}-{}.log", self.dump_name, counter)
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
                .required(true)
                .takes_value(true)
                .help("Core file size soft resource limit of crashing process"),
        )
        .arg(
            Arg::new("exe-name")
                .short('e')
                .long("exe-name")
                .required(true)
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
                .required(true)
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
                .required(true)
                .takes_value(true)
                .help("Number of signal causing dump."),
        )
        .arg(
            Arg::new("timestamp")
                .short('t')
                .long("timestamp")
                .required(true)
                .takes_value(true)
                .help("Time of dump, expressed as seconds since the Epoch."),
        )
        .arg(
            Arg::new("directory")
                .short('d')
                .long("dir")
                .required(true)
                .takes_value(true)
                .help("Directory to save the core dump to."),
        )
        .arg(
            Arg::new("hostname")
                .short('h')
                .long("hostname")
                .required(true)
                .takes_value(true)
                .help("Hostname (same as nodename returned by uname(2))"),
        )
        .arg(
            Arg::new("pathname")
                .short('E')
                .long("pathname")
                .required(true)
                .takes_value(true)
                .help("Hostname (same as nodename returned by uname(2))"),
        )
        .try_get_matches()
}
