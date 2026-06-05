//! Minimal application-scoped file logger for `marco` and `polo`.
//!
//! This intentionally does **not** delegate to `marco_core::logic::logger`.
//! marco-core's parser emits very verbose `INFO`/`DEBUG` records (e.g.
//! `Parsed N blocks` once per block) which are not useful for the editor
//! and viewer applications — they only flood the log file and slow the
//! parse pipeline down through log I/O.
//!
//! Records with a target starting with `marco_core` are silently dropped.
//! Everything else is written under the per-user cache directory:
//! - Linux: `~/.cache/<app>/logs/YYYYMM/YYMMDD.log`
//! - Windows: `%LOCALAPPDATA%\<app>\logs\YYYYMM\YYMMDD.log`
//!
//! Falls back to `./log/YYYYMM/YYMMDD.log` when the OS cache path cannot
//! be resolved (e.g. in development or test runs).
//!
//! The logger is always installed on the first `init` call regardless of
//! whether file logging is enabled. Subsequent calls only flip the global
//! max level and re-open the writer if needed, so `Settings → Debug →
//! Enable file logging` can be toggled at runtime without ever hitting the
//! `log` crate's "logger already initialized" limit.

use chrono::Local;
use log::{LevelFilter, Log, Metadata, Record};
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

static LOGGER: OnceLock<&'static AppFileLogger> = OnceLock::new();

const MAX_LOG_BYTES: u64 = 10 * 1024 * 1024; // 10 MiB

/// Targets whose log records are dropped before being written.
fn is_filtered_target(target: &str) -> bool {
    target.starts_with("marco_core")
}

pub struct AppFileLogger {
    /// Active writer. `None` after [`shutdown`] is called; re-created on the
    /// next [`init`] call with `enabled = true`.
    inner: Mutex<Option<BufWriter<File>>>,
    /// Path of the currently-open file. Kept so the writer can be re-opened
    /// after a shutdown/disable cycle without recomputing the date layout.
    file_path: Mutex<PathBuf>,
    bytes_written: AtomicU64,
}

impl AppFileLogger {
    fn rotate_if_needed_locked(&self, guard: &mut Option<BufWriter<File>>, file_path: &PathBuf) {
        if self.bytes_written.load(Ordering::Relaxed) <= MAX_LOG_BYTES {
            return;
        }

        if let Some(writer) = guard.as_mut() {
            let _ = writer.flush();
        }
        *guard = None;

        let ts = Local::now().format("%y%m%d-%H%M%S").to_string();
        let rotated_path =
            file_path.with_file_name(format!("{}.rotated.{}.log", ts, std::process::id()));

        let _ = fs::rename(file_path, &rotated_path);

        if let Ok(file) = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file_path)
        {
            *guard = Some(BufWriter::new(file));
            self.bytes_written.store(0, Ordering::Relaxed);
        }
    }
}

impl Log for AppFileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        if is_filtered_target(metadata.target()) {
            return false;
        }
        // Honor the global max level set via `log::set_max_level`.
        metadata.level() <= log::max_level()
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

        let file_path = match self.file_path.lock() {
            Ok(p) => p.clone(),
            Err(_) => return,
        };

        if let Ok(mut guard) = self.inner.lock() {
            if guard.is_none() {
                // Writer was dropped (e.g. logging was disabled then
                // re-enabled). Best-effort reopen.
                if let Some(parent) = file_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                if let Ok(file) = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&file_path)
                {
                    let initial = file.metadata().map(|m| m.len()).unwrap_or(0);
                    self.bytes_written.store(initial, Ordering::Relaxed);
                    *guard = Some(BufWriter::new(file));
                }
            }

            self.rotate_if_needed_locked(&mut guard, &file_path);
            if let Some(writer) = guard.as_mut() {
                if writer.write_all(line.as_bytes()).is_ok() {
                    self.bytes_written
                        .fetch_add(line.len() as u64, Ordering::Relaxed);
                }
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(writer) = guard.as_mut() {
                let _ = writer.flush();
            }
        }
    }
}

/// Resolve the log root directory.
///
/// Uses the per-user OS cache directory so logs are always writable,
/// even in installed packages where the executable directory is read-only:
/// - Linux: `$XDG_CACHE_HOME/<app>/logs` → `~/.cache/<app>/logs`
/// - Windows: `%LOCALAPPDATA%\<app>\logs`
///
/// Falls back to `./log` if the OS cache path cannot be resolved.
fn resolve_log_root() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        let base = std::env::var("XDG_CACHE_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".cache"))
            });
        if let Some(cache) = base {
            return cache.join("marco").join("logs");
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(local_app_data) = std::env::var("LOCALAPPDATA").ok().map(PathBuf::from) {
            return local_app_data.join("marco").join("logs");
        }
    }
    // Fallback for dev mode / unknown OS
    std::env::current_dir()
        .map(|d| d.join("log"))
        .unwrap_or_else(|_| PathBuf::from("log"))
}

fn today_file_path() -> PathBuf {
    let root = resolve_log_root();
    let month_dir = root.join(Local::now().format("%Y%m").to_string());
    month_dir.join(Local::now().format("%y%m%d.log").to_string())
}

/// Install the global logger (idempotent) and configure file logging.
///
/// The logger is installed on the **first** call regardless of `enabled`, so
/// it always owns the `log` crate's single global slot. Subsequent calls
/// merely flip the global max level and re-open the underlying file if it
/// was previously closed by [`shutdown`].
///
/// * `enabled` — when `false`, sets the global level to `Off`. The writer is
///   kept open so re-enabling is instant.
/// * `level` — max level to record when `enabled` is `true`. Records from
///   `marco_core*` targets are dropped regardless.
pub fn init(enabled: bool, level: LevelFilter) -> Result<(), Box<dyn std::error::Error>> {
    let target_level = if enabled { level } else { LevelFilter::Off };

    // Fast path: logger already installed. Update level + re-open file if needed.
    if let Some(logger) = LOGGER.get() {
        if enabled {
            ensure_writer_open(logger)?;
        }
        log::set_max_level(target_level);
        return Ok(());
    }

    // First-time install: prepare the file only if logging is actually
    // enabled — we don't want to create an empty `log/` folder on startup
    // when the user has disabled file logging.
    let file_path = today_file_path();
    let mut initial_size: u64 = 0;
    let writer: Option<BufWriter<File>> = if enabled {
        if let Some(parent) = file_path.parent() {
            match fs::create_dir_all(parent).and_then(|_| {
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&file_path)
            }) {
                Ok(file) => {
                    initial_size = file.metadata().map(|m| m.len()).unwrap_or(0);
                    Some(BufWriter::new(file))
                }
                Err(_) => None,
            }
        } else {
            None
        }
    } else {
        None
    };

    let boxed = Box::new(AppFileLogger {
        inner: Mutex::new(writer),
        file_path: Mutex::new(file_path),
        bytes_written: AtomicU64::new(initial_size),
    });
    let leaked: &'static AppFileLogger = Box::leak(boxed);

    match log::set_logger(leaked) {
        Ok(()) => {
            let _ = LOGGER.set(leaked);
            log::set_max_level(target_level);
            Ok(())
        }
        Err(e) => {
            // Another logger occupied the global slot before we could.
            unsafe {
                let _ = Box::from_raw(leaked as *const AppFileLogger as *mut AppFileLogger);
            }
            Err(format!("Failed to set global logger: {}", e).into())
        }
    }
}

/// Best-effort: ensure the underlying writer is open.
fn ensure_writer_open(logger: &AppFileLogger) -> Result<(), Box<dyn std::error::Error>> {
    // Refresh the path so a long-running process picks up day rollovers.
    let new_path = today_file_path();
    if let Ok(mut p) = logger.file_path.lock() {
        *p = new_path.clone();
    }

    if let Some(parent) = new_path.parent() {
        fs::create_dir_all(parent)?;
    }

    if let Ok(mut guard) = logger.inner.lock() {
        if guard.is_none() {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&new_path)?;
            let initial = file.metadata().map(|m| m.len()).unwrap_or(0);
            logger.bytes_written.store(initial, Ordering::Relaxed);
            *guard = Some(BufWriter::new(file));
        }
    }
    Ok(())
}

/// Current log directory (the `YYYYMM` subfolder for today's file).
pub fn current_log_dir() -> PathBuf {
    resolve_log_root().join(Local::now().format("%Y%m").to_string())
}

/// Root logs directory (parent of the monthly subfolders).
pub fn current_log_root_dir() -> PathBuf {
    resolve_log_root()
}

/// Today's log file path (whether or not it currently exists).
pub fn current_log_file_for_today() -> PathBuf {
    today_file_path()
}

/// Total size in bytes of every file under the root log directory.
pub fn total_log_size_bytes() -> u64 {
    let root = resolve_log_root();
    if !root.exists() {
        return 0;
    }
    let mut total: u64 = 0;
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
    total
}

/// Best-effort deletion of every log file under the root log directory.
///
/// Calls [`shutdown`] first so the currently-open file handle is released
/// (mandatory on Windows where open files cannot be deleted).
pub fn delete_all_logs() -> Result<(), Box<dyn std::error::Error>> {
    shutdown();

    let root = resolve_log_root();
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
            let _ = fs::remove_dir(&path);
        }
    }

    if root.read_dir()?.next().is_none() {
        let _ = fs::remove_dir(&root);
    }
    Ok(())
}

/// Whether the file logger is installed and currently has an open writer
/// (i.e. records will be persisted).
pub fn is_initialized() -> bool {
    match LOGGER.get() {
        Some(logger) => logger.inner.lock().map(|g| g.is_some()).unwrap_or(false),
        None => false,
    }
}

/// Flush and close the active log file and stop accepting new records.
///
/// The global `Log` instance remains registered (the `log` crate only
/// permits one `set_logger` call per process), but the underlying writer is
/// dropped and the max level is set to `Off`. Re-calling [`init`] with
/// `enabled = true` reopens the file.
pub fn shutdown() {
    if let Some(logger) = LOGGER.get() {
        if let Ok(mut guard) = logger.inner.lock() {
            if let Some(ref mut writer) = *guard {
                let _ = writer.flush();
            }
            *guard = None;
        }
    }
    log::set_max_level(LevelFilter::Off);
}
