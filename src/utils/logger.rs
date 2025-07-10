
use std::fmt;
use std::io::{self, Write};
use std::sync::{Mutex, RwLock};
use chrono::Local;
use once_cell::sync::Lazy;

static LOGGER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
static LOGGING_ENABLED: Lazy<RwLock<bool>> = Lazy::new(|| RwLock::new(false));

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Warn,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        };
        write!(f, "{}", s)
    }
}

impl LogLevel {
    fn color_code(&self) -> &'static str {
        match self {
            LogLevel::Debug => "\x1b[34m", // Blue
            LogLevel::Warn => "\x1b[33m",  // Yellow
            LogLevel::Error => "\x1b[31m", // Red
        }
    }
}

pub struct Logger {
    module_name: String,
    min_level: LogLevel,
}

impl Logger {
    pub fn new(module_name: &str, min_level: LogLevel) -> Self {
        Self {
            module_name: module_name.to_string(),
            min_level
        }
    }

    // Enable or disable all logging
    pub fn set_logging_enabled(enabled: bool) {
        let mut logging_enabled = LOGGING_ENABLED.write().unwrap();
        *logging_enabled = enabled;
    }

    // Check if logging is enabled
    pub fn is_logging_enabled() -> bool {
        *LOGGING_ENABLED.read().unwrap()
    }

    pub fn log(&self, level: LogLevel, msg: &str) {
        // First check if logging is enabled at all
        if !Logger::is_logging_enabled() {
            return;
        }

        // Then check the log level
        if level < self.min_level {
            return;
        }

        let _guard = LOGGER.lock().unwrap();

        let now = Local::now();
        let time_str = now.format("%Y-%m-%d %H:%M:%S");
        let color = level.color_code();
        let reset = "\x1b[0m";

        let output = format!(
            "{} [{}{}{}] [{}] {}",
            time_str,
            color,
            level,
            reset,
            self.module_name,
            msg
        );

        if level == LogLevel::Error {
            let _ = writeln!(&mut io::stderr(), "{}", output);
        } else {
            let _ = writeln!(&mut io::stdout(), "{}", output);
        }
    }

    pub fn debug(&self, msg: &str) {
        self.log(LogLevel::Debug, msg);
    }

    pub fn warn(&self, msg: &str) {
        self.log(LogLevel::Warn, msg);
    }

    pub fn error(&self, msg: &str) {
        self.log(LogLevel::Error, msg);
    }
}