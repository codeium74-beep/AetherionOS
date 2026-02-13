/// Aetherion OS - HAL Layer - Logging Infrastructure
/// Phase 3.1: Structured logging with color-coded output
/// 
/// This module implements the `log` facade for kernel-wide logging.
/// All log messages are output to the serial port with color coding.

use log::{Level, Metadata, Record, LevelFilter, SetLoggerError};

/// Simple logger implementation that outputs to serial port
pub struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level_str = match record.level() {
                Level::Error => "\x1b[31mERROR\x1b[0m", // Red
                Level::Warn  => "\x1b[33mWARN \x1b[0m", // Yellow
                Level::Info  => "\x1b[32mINFO \x1b[0m", // Green
                Level::Debug => "\x1b[36mDEBUG\x1b[0m", // Cyan
                Level::Trace => "\x1b[90mTRACE\x1b[0m", // Gray
            };

            crate::serial_println!(
                "[{}] [{}:{}] {}",
                level_str,
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

/// Initialize the logging system
/// 
/// This must be called after serial port initialization.
/// Sets the global logger to use our SimpleLogger implementation.
pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Trace))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_init() {
        // Test that logger can be initialized (may fail if already initialized)
        let _ = init();
    }
}
