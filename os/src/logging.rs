use log::{self, Level, LevelFilter, Metadata, Record};
use crate::println;
struct SimpleLogger;


impl log::Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            if record.level() == Level::Error {
                println!("\x1b[31m[{:>5}] {}\x1b[0m", record.level(), record.args());
            } else if record.level() == Level::Warn {
                println!("\x1b[93m[{:>5}] {}\x1b[0m", record.level(), record.args());
            } else if record.level() == Level::Info {
                println!("\x1b[34m[{:>5}] {}\x1b[0m", record.level(), record.args());
            } else if record.level() == Level::Debug {
                println!("\x1b[32m[{:>5}] {}\x1b[0m", record.level(), record.args());
            } else if record.level() == Level::Trace {
                println!("\x1b[90m[{:>5}] {}\x1b[0m", record.level(), record.args());
            } else {
                println!("[{:>5}] {}", record.level(), record.args());
            }
        }
    }
    fn flush(&self) {}
}


pub fn init(){
    static LOGGER: SimpleLogger = SimpleLogger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG") {
        Some("ERROR") => LevelFilter::Error,
        Some("WARN") => LevelFilter::Warn,
        Some("INFO") => LevelFilter::Info,
        Some("DEBUG") => LevelFilter::Debug,
        Some("TRACE") => LevelFilter::Trace,
        _ => LevelFilter::Info,
    });
}