//! A simple logging implementation

use log::{Level, LevelFilter, Log, Metadata, Record};

use crate::println;

struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let color = match record.level() {
            Level::Error => "31", // red
            Level::Warn => "93",  // yellow
            Level::Info => "34",  // blue
            Level::Debug => "32", // green
            Level::Trace => "90", // gray
        };
        println!(
            "\x1b[1;{}m[{}:{}][{}] {}\x1b[0m",
            color,
            record.file().unwrap_or("unknown"),
            record.line().unwrap_or(0),
            record.level(),
            record.args()
        );
    }

    fn flush(&self) {}
}

/// Initialize the logger
///
/// The log level is controlled by the `LOG_LEVEL` environment variable.
///
/// If `LOG_LEVEL` is not set, the default log level is `Info` in release mode
/// and `Debug` in debug mode.
///
/// The log level can be set to one of the following values:
/// - `error`
/// - `warn`
/// - `info`
/// - `debug`
/// - `trace`
///
/// Usage:
/// ``LOG_LEVEL=trace cargo run``
pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG_LEVEL") {
        Some("error") => LevelFilter::Error,
        Some("warn") => LevelFilter::Warn,
        Some("info") => LevelFilter::Info,
        Some("debug") => LevelFilter::Debug,
        Some("trace") => LevelFilter::Trace,
        _ => {
            if cfg!(debug_assertions) {
                LevelFilter::Debug
            } else {
                LevelFilter::Info
            }
        }
    });
}
