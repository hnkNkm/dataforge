use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// Log levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Global logger instance
pub struct Logger {
    level: LogLevel,
    file_path: Option<PathBuf>,
    file: Option<Mutex<std::fs::File>>,
}

impl Logger {
    /// Create a new logger
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            file_path: None,
            file: None,
        }
    }

    /// Enable file logging
    pub fn with_file(&mut self, path: PathBuf) -> std::io::Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        self.file_path = Some(path);
        self.file = Some(Mutex::new(file));
        Ok(())
    }

    /// Log a message
    pub fn log(&self, level: LogLevel, module: &str, message: &str) {
        if level < self.level {
            return;
        }

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let level_str = match level {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        };

        let log_line = format!("[{}] {} [{}] {}", timestamp, level_str, module, message);

        // Console output
        match level {
            LogLevel::Error => eprintln!("{}", log_line),
            _ => println!("{}", log_line),
        }

        // File output
        if let Some(ref file) = self.file {
            if let Ok(mut file) = file.lock() {
                let _ = writeln!(file, "{}", log_line);
                let _ = file.flush();
            }
        }
    }

    pub fn debug(&self, module: &str, message: &str) {
        self.log(LogLevel::Debug, module, message);
    }

    pub fn info(&self, module: &str, message: &str) {
        self.log(LogLevel::Info, module, message);
    }

    pub fn warn(&self, module: &str, message: &str) {
        self.log(LogLevel::Warn, module, message);
    }

    pub fn error(&self, module: &str, message: &str) {
        self.log(LogLevel::Error, module, message);
    }
}

/// Static logger instance
static mut LOGGER: Option<Logger> = None;
static LOGGER_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the global logger
pub fn init_logger(level: LogLevel, log_file: Option<PathBuf>) -> Result<(), std::io::Error> {
    unsafe {
        LOGGER_INIT.call_once(|| {
            let mut logger = Logger::new(level);

            if let Some(path) = log_file {
                if let Err(e) = logger.with_file(path) {
                    eprintln!("Failed to initialize file logging: {}", e);
                }
            }

            LOGGER = Some(logger);
        });
    }
    Ok(())
}

/// Get the global logger
pub fn logger() -> &'static Logger {
    unsafe {
        LOGGER.as_ref().unwrap_or_else(|| {
            panic!("Logger not initialized. Call init_logger() first.");
        })
    }
}

/// Convenience macros for logging
#[macro_export]
macro_rules! log_debug {
    ($module:expr, $($arg:tt)*) => {
        $crate::logger::logger().debug($module, &format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_info {
    ($module:expr, $($arg:tt)*) => {
        $crate::logger::logger().info($module, &format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_warn {
    ($module:expr, $($arg:tt)*) => {
        $crate::logger::logger().warn($module, &format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_error {
    ($module:expr, $($arg:tt)*) => {
        $crate::logger::logger().error($module, &format!($($arg)*));
    };
}

/// Log an error with context
pub fn log_error_with_context(module: &str, error: &crate::error::AppError, context: &str) {
    logger().error(
        module,
        &format!("{}: {}", context, error),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_logger_creation() {
        let logger = Logger::new(LogLevel::Info);
        assert_eq!(logger.level, LogLevel::Info);
    }

    #[test]
    fn test_log_levels() {
        assert!(LogLevel::Error > LogLevel::Warn);
        assert!(LogLevel::Warn > LogLevel::Info);
        assert!(LogLevel::Info > LogLevel::Debug);
    }

    #[test]
    fn test_file_logging() -> std::io::Result<()> {
        let dir = tempdir()?;
        let log_file = dir.path().join("test.log");

        let mut logger = Logger::new(LogLevel::Debug);
        logger.with_file(log_file.clone())?;

        logger.info("test", "Test message");

        let contents = std::fs::read_to_string(log_file)?;
        assert!(contents.contains("Test message"));
        assert!(contents.contains("INFO"));

        Ok(())
    }
}