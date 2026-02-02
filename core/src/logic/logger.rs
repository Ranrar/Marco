use chrono::Local;
use log::{Level, LevelFilter, Log, Metadata, Record};
use std::boxed::Box;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

static LOGGER: OnceLock<&'static SimpleFileLogger> = OnceLock::new();



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
    pub fn init(enabled: bool, level: LevelFilter) -> Result<(), Box<dyn std::error::Error>> {
        if !enabled {
            log::set_max_level(LevelFilter::Off);
            return Ok(());
        }

        // Use platform-appropriate cache directory for logs
        // **Windows Portable Mode**: {exe_dir}\logs\
        // **Windows Installed Mode**: %LOCALAPPDATA%\Marco\logs
        // **Linux**: ~/.cache/marco/logs
        
        #[cfg(target_os = "windows")]
        let log_root = {
            // Check for portable mode first
            if let Some(portable_root) = crate::paths::install::detect_portable_mode() {
                portable_root.join("logs")
            } else if let Some(cache_dir) = dirs::cache_dir() {
                cache_dir.join("marco").join("logs")
            } else {
                // Fallback: Try LOCALAPPDATA, then TEMP
                std::env::var_os("LOCALAPPDATA")
                    .map(|p| PathBuf::from(p).join("Marco").join("logs"))
                    .or_else(|| std::env::var_os("TEMP").map(|p| PathBuf::from(p).join("marco").join("logs")))
                    .unwrap_or_else(|| PathBuf::from("C:\\Temp\\marco\\log"))
            }
        };
        
        #[cfg(target_os = "linux")]
        let log_root = if let Some(cache_dir) = dirs::cache_dir() {
            cache_dir.join("marco").join("logs")
        } else {
            // Try XDG_CACHE_HOME, then ~/.cache, then /tmp
            std::env::var_os("XDG_CACHE_HOME")
                .map(|p| PathBuf::from(p).join("marco").join("logs"))
                .or_else(|| dirs::home_dir().map(|h| h.join(".cache").join("marco").join("logs")))
                .unwrap_or_else(|| PathBuf::from("/tmp/marco/log"))
        };
        
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        let log_root = dirs::cache_dir()
            .map(|c| c.join("marco").join("logs"))
            .unwrap_or_else(|| PathBuf::from("/tmp/marco/log"));
        fs::create_dir_all(&log_root).map_err(|e| -> Box<dyn std::error::Error> { e.to_string().into() })?;

        // YYYYMM folder
        let month_folder = Local::now().format("%Y%m").to_string();
        let month_dir = log_root.join(month_folder);
        fs::create_dir_all(&month_dir).map_err(|e| -> Box<dyn std::error::Error> { e.to_string().into() })?;
        // File name: YYMMDD.log
        let file_name = Local::now().format("%y%m%d.log").to_string();
        let file_path = month_dir.join(file_name);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .map_err(|e| -> Box<dyn std::error::Error> { e.to_string().into() })?;

        let initial_size = file.metadata().map(|m| m.len()).unwrap_or(0);

        let writer = BufWriter::new(file);

        let boxed = Box::new(SimpleFileLogger {
            inner: Mutex::new(Some(writer)),
            file_path,
            level,
            bytes_written: AtomicU64::new(initial_size),
        });

        // If we already initialized our logger earlier in this process, behave idempotently
        if LOGGER.get().is_some() {
            // Update global max level and return success
            log::set_max_level(level);
            return Ok(());
        }

        // Leak the box temporarily to obtain a &'static reference required by log::set_logger.
        // If another logger is already registered, gracefully abort and drop our boxed logger.
        let leaked: &'static SimpleFileLogger = Box::leak(boxed);

        // Attempt to register; if it fails, drop the leaked box and return Ok with a warning.
        match log::set_logger(leaked) {
            Ok(()) => {
                // Successfully set our logger; record the static reference and apply level.
                // OnceLock::set returns Err if already set, but we checked above, so this should always succeed
                let _ = LOGGER.set(leaked);
                log::set_max_level(level);
                Ok(())
            }
            Err(e) => {
                // Another logger is already present (e.g., env_logger). Drop our leaked box to avoid leaking memory.
                unsafe {
                    let _ = Box::from_raw(leaked as *const SimpleFileLogger as *mut SimpleFileLogger);
                }
                // Return an error to the caller so the application can decide how to surface it.
                return Err(format!("Failed to set global logger: {}", e).into());
            }
        }
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

pub fn init_file_logger(enabled: bool, level: LevelFilter) -> Result<(), Box<dyn std::error::Error>> {
    SimpleFileLogger::init(enabled, level).map_err(|e| format!("{}", e).into())
}

/// Returns true if the file logger was successfully initialized by this library.
pub fn is_file_logger_initialized() -> bool {
    LOGGER.get().is_some()
}

/// Return the resolved root logs directory (no month folder). This is a
/// non-negotiable platform-specific location using the system cache dir and
/// the folder name `logs` per project policy.
pub fn current_log_root_dir() -> std::path::PathBuf {
    // Prefer OS cache dir when available, else fall back to a platform temp path
    if let Some(cache_dir) = dirs::cache_dir() {
        return cache_dir.join("marco").join("logs");
    }

    // Platform fallback (should be rare)
    #[cfg(target_os = "windows")]
    {
        std::path::PathBuf::from("C:\\Temp\\marco\\logs")
    }
    #[cfg(target_os = "linux")]
    {
        std::path::PathBuf::from("/tmp/marco/logs")
    }
}

/// Return the resolved log directory for the current month (YYYYMM folder).
pub fn current_log_dir() -> std::path::PathBuf {
    use chrono::Local;
    let mut root = current_log_root_dir();
    let month_folder = Local::now().format("%Y%m").to_string();
    root.push(month_folder);
    root
}

/// Convenience: return the current log file path for today (YYMMDD.log) inside
/// the resolved log directory.
pub fn current_log_file_for_today() -> std::path::PathBuf {
    use chrono::Local;
    let dir = current_log_dir();
    let file_name = Local::now().format("%y%m%d.log").to_string();
    dir.join(file_name)
}

/// Compute total size in bytes of all log files under the root logs directory.
pub fn total_log_size_bytes() -> u64 {
    use std::fs;
    let root = current_log_root_dir();
    let mut total: u64 = 0;
    if root.exists() {
        // Walk month folders and files
        if let Ok(entries) = fs::read_dir(&root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(md) = entry.metadata() {
                        total += md.len();
                    }
                } else if path.is_dir() {
                    if let Ok(subs) = fs::read_dir(&path) {
                        for s in subs.flatten() {
                            if let Ok(md) = s.metadata() {
                                if md.is_file() {
                                    total += md.len();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    total
}

/// Delete all logs under the root logs directory.
/// Best-effort: removes files and month folders, returns error on I/O failures.
pub fn delete_all_logs() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    let root = current_log_root_dir();
    if !root.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(&root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let _ = fs::remove_file(&path);
        } else if path.is_dir() {
            for sub in fs::read_dir(&path)? {
                let sub = sub?;
                let subpath = sub.path();
                if subpath.is_file() {
                    let _ = fs::remove_file(&subpath);
                }
            }
            // Try to remove the month folder if empty
            let _ = fs::remove_dir(&path);
        }
    }

    // Remove root if empty
    if root.read_dir()?.next().is_none() {
        let _ = fs::remove_dir(&root);
    }

    Ok(())
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
    if let Some(logger) = LOGGER.get() {
        logger.shutdown();
        // Note: OnceLock doesn't support clearing after initialization.
        // The logger remains set but is shut down (file handle closed).
        // This is acceptable for program shutdown.
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
