use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::env;
use std::str::FromStr;

pub fn init_logger(loglevel: String) -> Result<String, anyhow::Error> {
    let logfilter = match LevelFilter::from_str(loglevel.as_str()) {
        Ok(v) => v,
        Err(_) => LevelFilter::Debug,
    };

    let mut log_path = env::current_exe()?;
    log_path.pop();
    log_path.push("composer.log");

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {d} - {m}\n")))
        .build(&log_path)?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(logfilter))?;

    log4rs::init_config(config)?;
    Ok(format!("{:?}", &log_path))
}
