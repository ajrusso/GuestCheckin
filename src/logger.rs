use std::time::SystemTime;
use humantime;
use log::LevelFilter;

pub struct Logger {}

impl Logger {
    pub fn new(log_level: LevelFilter, file_path: &str) -> Result<(), fern::InitError> {
        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{} {} {}] {}",
                    humantime::format_rfc3339_seconds(SystemTime::now()),
                    record.level(),
                    record.target(),
                    message
                ))
            })
            .level(log_level)
            .chain(std::io::stdout())
            .chain(fern::log_file(file_path)?)
            .apply()?;
        Ok(())
    }
}