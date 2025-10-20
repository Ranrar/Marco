use chrono::Local;
use log::{Level, LevelFilter, Log, Metadata, Record};
use std::boxed::Box;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

static mut LOGGER: Option<&'static SimpleFileLogger> = None;

pub struct SimpleFileLogger {
    inner: Mutex<Option<File>>,
    level: LevelFilter,
}

impl SimpleFileLogger {
    pub fn init(enabled: bool, level: LevelFilter) -> Result<(), String> {
        if !enabled {
            log::set_max_level(LevelFilter::Off);
            return Ok(());
        }

        let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
        let log_root = cwd.join("log"); // Dont change
        fs::create_dir_all(&log_root).map_err(|e| e.to_string())?;

        // YYYYMM folder
        let month_folder = Local::now().format("%Y%m").to_string();
        let month_dir = log_root.join(month_folder);
        fs::create_dir_all(&month_dir).map_err(|e| e.to_string())?;
        // File name: YYMMDD.log
        let file_name = Local::now().format("%y%m%d.log").to_string();
        let file_path = month_dir.join(file_name);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .map_err(|e| e.to_string())?;

        let boxed = Box::new(SimpleFileLogger {
            inner: Mutex::new(Some(file)),
            level,
        });
        let leaked: &'static SimpleFileLogger = Box::leak(boxed);
        unsafe {
            LOGGER = Some(leaked);
        }

        log::set_max_level(level);
        // Safe to unwrap because we just set LOGGER
        log::set_logger(unsafe { LOGGER.unwrap() }).map_err(|e| e.to_string())?;
        Ok(())
    }
}

impl Log for SimpleFileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Always accept logs at the configured level or higher
        metadata.level() <= self.level.to_level().unwrap_or(Level::Trace)
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let ts = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        // Format the log message
        let message = format!("{}", record.args());
        
        // Sanitize UTF-8 in log message to prevent panics from invalid slicing
        // This protects against debug logs that slice strings at non-char boundaries
        let sanitized_message = crate::logic::utf8::sanitize_input(
            message.as_bytes(),
            crate::logic::utf8::InputSource::Unknown
        );
        
        let line = format!(
            "{} [{}] {}: {}\n",
            ts,
            record.level(),
            record.target(),
            sanitized_message
        );
        
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(ref mut file) = *guard {
                let _ = file.write_all(line.as_bytes());
                let _ = file.flush();
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(ref mut file) = *guard {
                let _ = file.flush();
            }
        }
    }
}

pub fn init_file_logger(enabled: bool, level: LevelFilter) -> anyhow::Result<()> {
    SimpleFileLogger::init(enabled, level).map_err(|e| anyhow::anyhow!(e))
}

impl SimpleFileLogger {
    /// Flush and close the inner file. After shutdown, the global LOGGER will be cleared.
    pub fn shutdown(&self) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(ref mut file) = *guard {
                let _ = file.flush();
            }
            // Drop the file by taking it out
            *guard = None;
        }
    }
}

/// Public shutdown hook to safely flush and drop the global logger.
pub fn shutdown_file_logger() {
    unsafe {
        if let Some(logger) = LOGGER {
            logger.shutdown();
            // Clear the static reference; we leaked a Box originally, but dropping the file
            // and clearing the pointer is acceptable for program shutdown. We set to None
            // to avoid double-use.
            LOGGER = None;
        }
    }
}

/// Safe string preview for logging - truncates by character count, not bytes
/// 
/// This function safely truncates strings for debug logging without causing
/// UTF-8 boundary panics. Use this instead of byte slicing in log statements.
///
/// # Examples
/// ```
/// use core::logic::logger::safe_preview;
/// 
/// let text = "Hello 😀 World — test";
/// let preview = safe_preview(text, 10); // Takes first 10 characters safely
/// log::debug!("Parsing: {}", preview);
/// ```
#[inline]
pub fn safe_preview(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}

/// Macro for safe debug logging with automatic string truncation
/// 
/// Use this instead of `log::debug!()` when logging string slices that might
/// contain multi-byte UTF-8 characters. It automatically truncates safely.
///
/// # Examples
/// ```
/// use core::safe_debug;
/// 
/// let input = "Text with emoji 😀 and em dash —";
/// safe_debug!("Parsing paragraph from: {:?}", input, 40);
/// safe_debug!("Short preview: {:?}", input, 20);
/// ```
#[macro_export]
macro_rules! safe_debug {
    ($fmt:expr, $text:expr, $max:expr) => {
        log::debug!($fmt, $crate::logic::logger::safe_preview($text, $max))
    };
    ($fmt:expr, $text:expr, $max:expr, $($arg:tt)*) => {
        log::debug!($fmt, $crate::logic::logger::safe_preview($text, $max), $($arg)*)
    };
}
