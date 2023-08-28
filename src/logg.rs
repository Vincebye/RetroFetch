use log::{Level, Metadata, Record};
use colored::*;

pub struct ColorLogger;

impl log::Log for ColorLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level = match record.level() {
                Level::Error => record.level().to_string().red(),
                Level::Warn => record.level().to_string().yellow(),
                Level::Info => record.level().to_string().green(),
                Level::Debug => record.level().to_string().cyan(),
                Level::Trace => record.level().to_string().blue(),
            };
            println!("{} - {}", level, record.args());
        }
    }

    fn flush(&self) {}
}