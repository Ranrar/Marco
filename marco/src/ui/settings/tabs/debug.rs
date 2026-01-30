//! Debug settings tab
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, CheckButton, Orientation, Label};
use log::trace;

// Import unified helper
use super::helpers::add_setting_row;

/// Builds the Debug tab UI. Provides a simple checkbox to enable/disable debug mode.
pub fn build_debug_tab(settings_path: &str, parent: &gtk4::Window) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("marco-settings-tab");

    // Use SettingsManager to load current setting (default to false)
    let settings_manager = match core::logic::swanson::SettingsManager::initialize(
        std::path::PathBuf::from(settings_path),
    ) {
        Ok(sm) => sm,
        Err(_) => {
            log::warn!("Failed to initialize SettingsManager in debug tab, using defaults");
            return container;
        }
    };

    // --- Debug Mode Setting ---
    let current = settings_manager.get_settings().debug.unwrap_or(false);

    let debug_checkbox = CheckButton::with_label("Enable debug mode");
    debug_checkbox.add_css_class("marco-checkbutton");
    debug_checkbox.set_active(current);

    let settings_manager_clone = settings_manager.clone();
    debug_checkbox.connect_toggled(move |cb| {
        let active = cb.is_active();
        if let Err(e) = settings_manager_clone.update_settings(|settings| {
            settings.debug = Some(active);
        }) {
            log::error!("Failed to update debug setting: {}", e);
        }
    });

    // Create debug mode row using unified helper (first row)
    let debug_row = add_setting_row(
        "Debug Mode",
        "Enable debug features and diagnostics. Shows debug UI components and additional logging information.",
        &debug_checkbox,
        true  // First row - no top margin
    );
    container.append(&debug_row);

    // --- Program Log Setting ---
    let log_enabled = settings_manager.get_settings().log_to_file.unwrap_or(false);

    let log_checkbox = CheckButton::with_label("Enable file logging");
    log_checkbox.add_css_class("marco-checkbutton");
    log_checkbox.set_active(log_enabled);

    // Wire checkbox to persist setting (handler registered after UI elements below so it can update the UI immediately)
    let settings_manager_clone2 = settings_manager.clone();
    // connect handler later (after delete button is created)

    // Create program log row using unified helper
    let log_row = add_setting_row(
        "Program Log",
        "Write program logs to file for troubleshooting. Log files are stored in the application data directory.",
        &log_checkbox,
        false  // Not first row
    );
    container.append(&log_row);

    // Keep a weak reference to the parent window so closures don't require a 'static parent reference
    let parent_weak = parent.downgrade();

    // Helpful explanatory note: show platform-specific log locations and a restart tip
    let resolved_dir = core::logic::logger::current_log_dir();
    let resolved_display = resolved_dir.display().to_string();

    let log_paths_text = format!(
        "Resolved log directory: {}\n\nDefault locations:\n• Windows: %LOCALAPPDATA%\\Marco\\logs\\<YYYYMM>\\<YYMMDD.log>\n• Linux/macOS: ~/.cache/marco/logs/<YYYYMM>/<YYMMDD.log>\n\nFallback (development): ./log/<YYYYMM>/<YYMMDD.log> (used when system cache dir is unavailable).\n\nNote: File logging can be enabled/disabled immediately from this dialog without restarting the app.",
        resolved_display
    );

    let info_label = Label::new(Some(&log_paths_text));
    info_label.set_wrap(true);
    info_label.add_css_class("settings-note");
    info_label.set_margin_top(8);

    // Size label and buttons
    let size_bytes = core::logic::logger::total_log_size_bytes();
    let size_text = format!("Total logs size: {:.2} MB", size_bytes as f64 / (1024.0 * 1024.0));
    let size_label = Label::new(Some(&size_text));
    size_label.add_css_class("settings-note");
    size_label.set_margin_top(6);

    use gtk4::Button;
    let open_btn = Button::with_label("Open logs folder");
    let dir_clone = resolved_dir.clone();
    open_btn.connect_clicked(move |_| {
        // Normalize to a proper file:// URI across platforms
        let path = dir_clone.clone();
        let normalized_uri = {
            // Use canonicalized absolute path when possible
            let abs = path.canonicalize().unwrap_or(path.clone());
            #[cfg(windows)]
            {
                // Windows file URIs must start with file:/// and use forward slashes
                let s = abs.to_string_lossy().replace("\\", "/");
                format!("file:///{}", s)
            }
            #[cfg(not(windows))]
            {
                // Abs path already begins with '/'
                format!("file://{}", abs.to_string_lossy())
            }
        };

        if let Err(e) = gio::AppInfo::launch_default_for_uri(&normalized_uri, None::<&gio::AppLaunchContext>) {
            // Fallback: try platform-specific open command if GIO fails
            log::warn!("Failed to open logs folder {} via GIO: {}. Trying platform fallback...", normalized_uri, e);
            #[cfg(windows)]
            {
                if let Err(cmd_err) = std::process::Command::new("explorer").arg(dir_clone.to_string_lossy().to_string()).status() {
                    log::error!("Failed to open logs folder with explorer: {}", cmd_err);
                }
            }
            #[cfg(target_os = "macos")]
            {
                if let Err(cmd_err) = std::process::Command::new("open").arg(dir_clone.to_string_lossy().to_string()).status() {
                    log::error!("Failed to open logs folder with open: {}", cmd_err);
                }
            }
            #[cfg(all(unix, not(target_os = "macos")))]
            {
                if let Err(cmd_err) = std::process::Command::new("xdg-open").arg(dir_clone.to_string_lossy().to_string()).status() {
                    log::error!("Failed to open logs folder with xdg-open: {}", cmd_err);
                }
            }
        }
    });

    let delete_btn = Button::with_label("Delete logs");
    // Sensitivity based on whether logs exist
    delete_btn.set_sensitive(size_bytes > 0);

    // Delete action: confirm, shutdown logger, delete files, re-init if needed
    let settings_manager_clone3 = settings_manager.clone();
    let size_label_clone = size_label.clone();
    let parent_weak_for_delete = parent_weak.clone();
    delete_btn.connect_clicked(move |_| {
        // Confirmation dialog - use upgraded weak parent when available
        let maybe_parent = parent_weak_for_delete.upgrade();
        let dialog = if let Some(parent_win) = maybe_parent {
            gtk4::MessageDialog::new(
                Some(&parent_win),
                gtk4::DialogFlags::MODAL | gtk4::DialogFlags::DESTROY_WITH_PARENT,
                gtk4::MessageType::Question,
                gtk4::ButtonsType::YesNo,
                "Delete all log files?",
            )
        } else {
            // Fallback to no parent (rare)
            gtk4::MessageDialog::new(
                None::<&gtk4::Window>,
                gtk4::DialogFlags::MODAL,
                gtk4::MessageType::Question,
                gtk4::ButtonsType::YesNo,
                "Delete all log files?",
            )
        };

        let size_label_clone2 = size_label_clone.clone();
        let settings_manager_clone4 = settings_manager_clone3.clone();
        // Clone the weak parent for the response closure (avoids move issues)
        let parent_weak_for_response = parent_weak_for_delete.clone();
        dialog.connect_response(move |dlg, resp| {
            if resp == gtk4::ResponseType::Yes {
                // Shutdown logger before deleting files
                core::logic::logger::shutdown_file_logger();
                if let Err(e) = core::logic::logger::delete_all_logs() {
                    log::error!("Failed to delete logs: {}", e);
                } else {
                    log::info!("Deleted all logs via Debug settings");
                }

                // Re-init if setting enabled
                let enabled = settings_manager_clone4.get_settings().log_to_file.unwrap_or(false) || std::env::var("MARCO_LOG").is_ok();
                if enabled {
                    // Try to reinit logger with Info level by default
                    if let Err(e) = core::logic::logger::init_file_logger(true, log::LevelFilter::Info) {
                        // Show an attached dialog explaining why enable failed
                        if let Some(parent_win) = parent_weak_for_response.upgrade() {
                            let dlg = gtk4::MessageDialog::new(
                                Some(&parent_win),
                                gtk4::DialogFlags::MODAL | gtk4::DialogFlags::DESTROY_WITH_PARENT,
                                gtk4::MessageType::Warning,
                                gtk4::ButtonsType::Ok,
                                "Could not enable file logging",
                            );
                            dlg.set_secondary_text(Some("Another logger is already initialized in this process. File logging cannot be enabled."));
                            dlg.run_future();
                            dlg.close();
                        } else {
                            log::error!("Failed to reinit logger after deletion: {}", e);
                        }
                    }
                }

                // Update UI size label and button sensitivity
                let new_size = core::logic::logger::total_log_size_bytes();
                size_label_clone2.set_text(&format!("Total logs size: {:.2} MB", new_size as f64 / (1024.0 * 1024.0)));
            }
            dlg.close();
        });
        dialog.present();
    });

    // Now connect the checkbox handler so it can update UI/shutdown/init immediately
    let size_label_clone_cb = size_label.clone();
    let delete_btn_clone_cb = delete_btn.clone();
    log_checkbox.connect_toggled(move |cb| {
        let active = cb.is_active();
        trace!("audit: user toggled program log: {}", active);
        if let Err(e) = settings_manager_clone2.update_settings(|settings| {
            settings.log_to_file = Some(active);
        }) {
            log::error!("Failed to update log_to_file setting: {}", e);
        }

        // The settings listener registered in main.rs will perform the actual init/shutdown.
        // After update_settings returns the listener will have been invoked; check whether our file logger is active.
        let initialized = core::logic::logger::is_file_logger_initialized();
        if active && !initialized {
            // Show a single, attached dialog explaining why logging couldn't be enabled
            if let Some(parent_win) = parent_weak.upgrade() {
                let dlg = gtk4::MessageDialog::new(
                    Some(&parent_win),
                    gtk4::DialogFlags::MODAL | gtk4::DialogFlags::DESTROY_WITH_PARENT,
                    gtk4::MessageType::Warning,
                    gtk4::ButtonsType::Ok,
                    "Could not enable file logging",
                );
                dlg.set_secondary_text(Some("Another logger is already initialized in this process. File logging cannot be enabled."));
                dlg.run_future();
                dlg.close();
            } else {
                log::warn!("Could not enable file logging: another logger is present");
            }
        } else if !active && initialized {
            // In the rare case shutdown hasn't run, explicitly ensure we stop writing to files
            core::logic::logger::shutdown_file_logger();
            log::info!("File logger disabled via UI (ensured shutdown)");
        }

        // Update size and button sensitivity
        let new_size = core::logic::logger::total_log_size_bytes();
        size_label_clone_cb.set_text(&format!("Total logs size: {:.2} MB", new_size as f64 / (1024.0 * 1024.0)));
        delete_btn_clone_cb.set_sensitive(new_size > 0);
    });

    container.append(&info_label);
    container.append(&size_label);

    // Row with buttons
    let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    btn_box.append(&open_btn);
    btn_box.append(&delete_btn);

    // Refresh button to update size/sensitivity if logs changed externally
    let refresh_btn = Button::with_label("Refresh");
    let size_label_for_refresh = size_label.clone();
    let delete_btn_for_refresh = delete_btn.clone();
    refresh_btn.connect_clicked(move |_| {
        let new_size = core::logic::logger::total_log_size_bytes();
        size_label_for_refresh.set_text(&format!("Total logs size: {:.2} MB", new_size as f64 / (1024.0 * 1024.0)));
        delete_btn_for_refresh.set_sensitive(new_size > 0);
    });
    btn_box.append(&refresh_btn);

    container.append(&btn_box);


    container
}
