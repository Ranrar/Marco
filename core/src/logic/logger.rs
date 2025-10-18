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
        let line = format!(
            "{} [{}] {}: {}\n",
            ts,
            record.level(),
            record.target(),
            record.args()
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
