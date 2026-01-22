use chrono::Local;
use log::{Level, LevelFilter, Log, Metadata, Record};
use std::boxed::Box;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

static mut LOGGER: Option<&'static SimpleFileLogger> = None;

pub struct SimpleFileLogger {
    inner: Mutex<Option<BufWriter<File>>>,
    file_path: PathBuf,
    level: LevelFilter,
    bytes_written: AtomicU64,
}

// Keep log files reasonably sized so editors (and VS Code) can open them
// without trying to load hundreds of MB into memory.
const MAX_LOG_BYTES: u64 = 10 * 1024 * 1024; // 10 MiB

impl SimpleFileLogger {
    pub fn init(enabled: bool, level: LevelFilter) -> Result<(), String> {
        if !enabled {
            log::set_max_level(LevelFilter::Off);
            return Ok(());
        }

        // Determine log root directory based on platform and context
        let log_root = if cfg!(target_os = "windows") {
            // On Windows, prefer AppData\Local\marco\log for installed apps
            // but fall back to cwd/log for development
            if let Ok(exe_path) = std::env::current_exe() {
                // Check if running from target/ (development mode)
                let exe_str = exe_path.to_string_lossy();
                if exe_str.contains("target\\") || exe_str.contains("target/") {
                    // Development mode: use cwd/log
                    std::env::current_dir()
                        .map(|d| d.join("log"))
                        .unwrap_or_else(|_| PathBuf::from("log"))
                } else {
                    // Installed/portable mode: use AppData\Local\marco\log
                    dirs::data_local_dir()
                        .map(|d| d.join("marco").join("log"))
                        .unwrap_or_else(|| {
                            std::env::temp_dir().join("marco").join("log")
                        })
                }
            } else {
                // Fallback to temp if we can't determine exe path
                std::env::temp_dir().join("marco").join("log")
            }
        } else {
            // Unix: use cwd/log for both dev and installed (traditional behavior)
            std::env::current_dir()
                .map(|d| d.join("log"))
                .unwrap_or_else(|_| PathBuf::from("log"))
        };

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

        let initial_size = file.metadata().map(|m| m.len()).unwrap_or(0);

        let writer = BufWriter::new(file);

        let boxed = Box::new(SimpleFileLogger {
            inner: Mutex::new(Some(writer)),
            file_path,
            level,
            bytes_written: AtomicU64::new(initial_size),
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

    fn rotate_if_needed_locked(&self, guard: &mut Option<BufWriter<File>>) {
        let current = self.bytes_written.load(Ordering::Relaxed);
        if current <= MAX_LOG_BYTES {
            return;
        }

        // Best-effort rotation: flush current writer, rename the file, start a new one.
        if let Some(writer) = guard.as_mut() {
            let _ = writer.flush();
        }

        // Drop writer so the underlying file handle is released before rename on Windows.
        *guard = None;

        let ts = Local::now().format("%y%m%d-%H%M%S").to_string();
        let rotated_path =
            self.file_path
                .with_file_name(format!("{}.rotated.{}.log", ts, std::process::id()));

        if let Err(e) = fs::rename(&self.file_path, &rotated_path) {
            // If rename fails (e.g. file missing), just continue with a new file.
            eprintln!(
                "[logger] rotation rename failed ({} -> {}): {}",
                self.file_path.display(),
                rotated_path.display(),
                e
            );
        }

        match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.file_path)
        {
            Ok(file) => {
                *guard = Some(BufWriter::new(file));
                self.bytes_written.store(0, Ordering::Relaxed);
            }
            Err(e) => {
                eprintln!(
                    "[logger] failed to open new log file {}: {}",
                    self.file_path.display(),
                    e
                );
            }
        }
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
            crate::logic::utf8::InputSource::Unknown,
        );

        let line = format!(
            "{} [{}] {}: {}\n",
            ts,
            record.level(),
            record.target(),
            sanitized_message
        );

        // Track size and rotate early if needed.
        // Note: this is approximate (UTF-8 bytes). Good enough for keeping files small.
        let line_len = line.len() as u64;
        self.bytes_written.fetch_add(line_len, Ordering::Relaxed);

        if let Ok(mut guard) = self.inner.lock() {
            self.rotate_if_needed_locked(&mut guard);
            if let Some(ref mut writer) = *guard {
                let _ = writer.write_all(line.as_bytes());

                // Avoid flushing on every line (can stall UI).
                // Flush eagerly only for high-severity events.
                if record.level() <= Level::Error {
                    let _ = writer.flush();
                }
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(ref mut writer) = *guard {
                let _ = writer.flush();
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
            if let Some(ref mut writer) = *guard {
                let _ = writer.flush();
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
