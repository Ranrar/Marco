use core::logic::swanson::SettingsManager;
use std::sync::Arc;

/// Initialize the file logger according to application settings and the `RUST_LOG`
/// environment variable. Also registers a settings change listener to toggle the
/// logger at runtime.
pub fn init_logging(settings_manager: &Arc<SettingsManager>) {
    let app_settings = settings_manager.get_settings();

    // Enable logging if RUST_LOG environment variable is set or if configured in settings
    let rust_log_set = std::env::var("RUST_LOG").is_ok();
    let enabled = app_settings.log_to_file.unwrap_or(false) || rust_log_set;

    // Choose a sane default to avoid huge log files and UI stalls.
    // Trace should be opt-in.
    let level = match std::env::var("RUST_LOG") {
        Ok(v) => {
            let v = v.to_ascii_lowercase();
            if v.contains("trace") {
                log::LevelFilter::Trace
            } else if v.contains("debug") {
                log::LevelFilter::Debug
            } else if v.contains("info") {
                log::LevelFilter::Info
            } else if v.contains("warn") {
                log::LevelFilter::Warn
            } else if v.contains("error") {
                log::LevelFilter::Error
            } else {
                log::LevelFilter::Info
            }
        }
        Err(_) => log::LevelFilter::Info,
    };

    if let Err(e) = core::logic::logger::init_file_logger(enabled, level) {
        eprintln!("Failed to initialize file logger: {}", e);
    } else if enabled {
        // Show the resolved log folder to avoid confusion about "./log" vs system cache
        let resolved = core::logic::logger::current_log_dir();
        log::info!(
            "Logger initialized with level: {:?}, RUST_LOG set: {}, log_dir: {}",
            level,
            rust_log_set,
            resolved.display()
        );
        log::debug!("Debug logging is working");
        log::trace!("Trace logging is working");

        println!(
            "Logging enabled (level: {:?}), log files stored under: {}",
            level,
            resolved.display()
        );
    } else if rust_log_set {
        // RUST_LOG was set but settings did not explicitly enable file logging — still show intended path
        let resolved = core::logic::logger::current_log_dir();
        println!(
            "Logging enabled via RUST_LOG (level: {:?}), log files stored under: {}",
            level,
            resolved.display()
        );
    }

    // Register listener to toggle file logger at runtime when settings change.
    // Use the explicit settings flag `log_to_file` or the alternate env var `MARCO_LOG`.
    let settings_manager_for_logger = settings_manager.clone();
    let level_for_logger = level;
    settings_manager_for_logger.register_change_listener("logger".to_string(), move |s| {
        let enabled_now = s.log_to_file.unwrap_or(false) || std::env::var("MARCO_LOG").is_ok();
        if enabled_now {
            if let Err(e) = core::logic::logger::init_file_logger(true, level_for_logger) {
                log::warn!("Failed to init file logger from settings listener: {}", e);
            } else {
                log::info!("File logger enabled via settings listener");
            }
        } else {
            // Shutdown file logger immediately
            core::logic::logger::shutdown_file_logger();
            log::info!("File logger disabled via settings listener");
        }
    });
}
