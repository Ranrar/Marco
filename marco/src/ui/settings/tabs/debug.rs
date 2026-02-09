//! Debug settings tab
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, CheckButton, Label, Orientation};
use log::trace;
use std::rc::Rc;

// Import unified helper
use super::helpers::{add_setting_row_i18n, SettingsI18nRegistry};
use crate::components::language::SettingsDebugTranslations;
use crate::components::language::Translations;
use crate::ui::dialogs::welcome_screen;

/// Builds the Debug tab UI. Provides a simple checkbox to enable/disable debug mode.
pub fn build_debug_tab(
    settings_path: &str,
    parent: &gtk4::Window,
    translations: &SettingsDebugTranslations,
    i18n: &SettingsI18nRegistry,
) -> GtkBox {
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

    let debug_checkbox = CheckButton::with_label(&translations.debug_checkbox);
    debug_checkbox.add_css_class("marco-checkbutton");
    debug_checkbox.set_active(current);

    i18n.bind_check_button_label(
        &debug_checkbox,
        Rc::new(|t: &Translations| t.settings.debug.debug_checkbox.clone()),
    );

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
    let debug_row = add_setting_row_i18n(
        i18n,
        &translations.debug_label,
        &translations.debug_description,
        Rc::new(|t: &Translations| t.settings.debug.debug_label.clone()),
        Rc::new(|t: &Translations| t.settings.debug.debug_description.clone()),
        &debug_checkbox,
        true, // First row - no top margin
    );
    container.append(&debug_row);

    // --- Program Log Setting ---
    let log_enabled = settings_manager.get_settings().log_to_file.unwrap_or(false);

    let log_checkbox = CheckButton::with_label(&translations.log_checkbox);
    log_checkbox.add_css_class("marco-checkbutton");
    log_checkbox.set_active(log_enabled);

    i18n.bind_check_button_label(
        &log_checkbox,
        Rc::new(|t: &Translations| t.settings.debug.log_checkbox.clone()),
    );

    // Wire checkbox to persist setting (handler registered after UI elements below so it can update the UI immediately)
    let settings_manager_clone2 = settings_manager.clone();
    // connect handler later (after delete button is created)

    // Create program log row using unified helper
    let log_row = add_setting_row_i18n(
        i18n,
        &translations.log_label,
        &translations.log_description,
        Rc::new(|t: &Translations| t.settings.debug.log_label.clone()),
        Rc::new(|t: &Translations| t.settings.debug.log_description.clone()),
        &log_checkbox,
        false, // Not first row
    );
    container.append(&log_row);

    // --- Welcome Screen (debug) ---
    let show_welcome_button = gtk4::Button::with_label(&translations.welcome_button);
    show_welcome_button.add_css_class("marco-btn");
    show_welcome_button.add_css_class("marco-btn-blue");
    i18n.bind_button_label(
        &show_welcome_button,
        Rc::new(|t: &Translations| t.settings.debug.welcome_button.clone()),
    );

    // Use weak parent so the debug tab doesn't hold the settings window alive.
    let parent_weak_for_welcome = parent.downgrade();
    let settings_manager_for_welcome = settings_manager.clone();
    show_welcome_button.connect_clicked(move |_| {
        let parent = parent_weak_for_welcome.upgrade();
        let parent_ref = parent.as_ref().map(|w| w.upcast_ref());
        welcome_screen::show_welcome_screen(&settings_manager_for_welcome, parent_ref, None);
    });

    let welcome_row = add_setting_row_i18n(
        i18n,
        &translations.welcome_label,
        &translations.welcome_description,
        Rc::new(|t: &Translations| t.settings.debug.welcome_label.clone()),
        Rc::new(|t: &Translations| t.settings.debug.welcome_description.clone()),
        &show_welcome_button,
        false,
    );
    container.append(&welcome_row);

    // Keep a weak reference to the parent window so closures don't require a 'static parent reference
    let parent_weak = parent.downgrade();

    // Helpful explanatory note: show platform-specific log locations and a restart tip
    let resolved_dir = core::logic::logger::current_log_dir();
    let resolved_display = resolved_dir.display().to_string();

    let log_paths_text = translations
        .log_paths_template
        .replace("{resolved}", &resolved_display);

    let info_label = Label::new(Some(&log_paths_text));
    info_label.set_wrap(true);
    info_label.add_css_class("settings-note");
    info_label.set_margin_top(8);

    {
        let resolved_display = resolved_display.clone();
        i18n.bind_label_text(
            &info_label,
            Rc::new(move |t: &Translations| {
                t.settings
                    .debug
                    .log_paths_template
                    .replace("{resolved}", &resolved_display)
            }),
        );
    }

    // Size label and buttons
    let size_bytes = core::logic::logger::total_log_size_bytes();
    let size_text = {
        let size_mb = size_bytes as f64 / (1024.0 * 1024.0);
        if translations.log_size_template.contains("{size_mb:.2}") {
            translations
                .log_size_template
                .replace("{size_mb:.2}", &format!("{:.2}", size_mb))
        } else if translations.log_size_template.contains("{size_mb}") {
            translations
                .log_size_template
                .replace("{size_mb}", &format!("{:.2}", size_mb))
        } else {
            format!("{} {:.2}", translations.log_size_template, size_mb)
        }
    };
    let size_label = Label::new(Some(&size_text));
    size_label.add_css_class("settings-note");
    size_label.set_margin_top(6);

    i18n.bind_label_text(
        &size_label,
        Rc::new(|t: &Translations| {
            let size_bytes = core::logic::logger::total_log_size_bytes();
            let size_mb = size_bytes as f64 / (1024.0 * 1024.0);
            let template = &t.settings.debug.log_size_template;
            if template.contains("{size_mb:.2}") {
                template.replace("{size_mb:.2}", &format!("{:.2}", size_mb))
            } else if template.contains("{size_mb}") {
                template.replace("{size_mb}", &format!("{:.2}", size_mb))
            } else {
                format!("{} {:.2}", template, size_mb)
            }
        }),
    );

    use gtk4::Button;
    let open_btn = Button::with_label(&translations.open_logs_button);
    i18n.bind_button_label(
        &open_btn,
        Rc::new(|t: &Translations| t.settings.debug.open_logs_button.clone()),
    );
    let dir_clone = resolved_dir.clone();
    open_btn.connect_clicked(move |_| {
        // Normalize to a proper file:// URI across platforms
        let path = dir_clone.clone();
        let normalized_uri = {
            // Use canonicalized absolute path when possible
            let abs = path.canonicalize().unwrap_or(path.clone());
            #[cfg(target_os = "windows")]
            {
                // Windows file URIs must start with file:/// and use forward slashes
                let s = abs.to_string_lossy().replace("\\", "/");
                format!("file:///{}", s)
            }
            #[cfg(target_os = "linux")]
            {
                // Abs path already begins with '/'
                format!("file://{}", abs.to_string_lossy())
            }
        };

        if let Err(e) =
            gio::AppInfo::launch_default_for_uri(&normalized_uri, None::<&gio::AppLaunchContext>)
        {
            // Fallback: try platform-specific open command if GIO fails
            log::warn!(
                "Failed to open logs folder {} via GIO: {}. Trying platform fallback...",
                normalized_uri,
                e
            );

            #[cfg(target_os = "windows")]
            {
                if let Err(cmd_err) = std::process::Command::new("explorer")
                    .arg(dir_clone.to_string_lossy().to_string())
                    .status()
                {
                    log::error!("Failed to open logs folder with explorer: {}", cmd_err);
                }
            }

            #[cfg(target_os = "linux")]
            {
                if let Err(cmd_err) = std::process::Command::new("xdg-open")
                    .arg(dir_clone.to_string_lossy().to_string())
                    .status()
                {
                    log::error!("Failed to open logs folder with xdg-open: {}", cmd_err);
                }
            }
        }
    });

    let delete_btn = Button::with_label(&translations.delete_logs_button);
    i18n.bind_button_label(
        &delete_btn,
        Rc::new(|t: &Translations| t.settings.debug.delete_logs_button.clone()),
    );
    // Sensitivity based on whether logs exist
    delete_btn.set_sensitive(size_bytes > 0);

    // Delete action: confirm, shutdown logger, delete files, re-init if needed
    let settings_manager_clone3 = settings_manager.clone();
    let size_label_clone = size_label.clone();
    let parent_weak_for_delete = parent_weak.clone();
    let translations_for_delete = translations.clone();
    delete_btn.connect_clicked(move |_| {
        // Confirmation dialog - use upgraded weak parent when available
        let maybe_parent = parent_weak_for_delete.upgrade();
        let dialog = if let Some(parent_win) = maybe_parent {
            gtk4::MessageDialog::new(
                Some(&parent_win),
                gtk4::DialogFlags::MODAL | gtk4::DialogFlags::DESTROY_WITH_PARENT,
                gtk4::MessageType::Question,
                gtk4::ButtonsType::YesNo,
                &translations_for_delete.delete_logs_confirm,
            )
        } else {
            // Fallback to no parent (rare)
            gtk4::MessageDialog::new(
                None::<&gtk4::Window>,
                gtk4::DialogFlags::MODAL,
                gtk4::MessageType::Question,
                gtk4::ButtonsType::YesNo,
                &translations_for_delete.delete_logs_confirm,
            )
        };

        let size_label_clone2 = size_label_clone.clone();
        let settings_manager_clone4 = settings_manager_clone3.clone();
        // Clone the weak parent for the response closure (avoids move issues)
        let parent_weak_for_response = parent_weak_for_delete.clone();
        let translations_for_response = translations_for_delete.clone();
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
                let enabled = settings_manager_clone4
                    .get_settings()
                    .log_to_file
                    .unwrap_or(false)
                    || std::env::var("MARCO_LOG").is_ok();
                if enabled {
                    // Try to reinit logger with Info level by default
                    if let Err(e) =
                        core::logic::logger::init_file_logger(true, log::LevelFilter::Info)
                    {
                        // Show an attached dialog explaining why enable failed
                        if let Some(parent_win) = parent_weak_for_response.upgrade() {
                            let dlg = gtk4::MessageDialog::new(
                                Some(&parent_win),
                                gtk4::DialogFlags::MODAL | gtk4::DialogFlags::DESTROY_WITH_PARENT,
                                gtk4::MessageType::Warning,
                                gtk4::ButtonsType::Ok,
                                &translations_for_response.log_enable_failed_title,
                            );
                            dlg.set_secondary_text(Some(
                                &translations_for_response.log_enable_failed_message,
                            ));
                            dlg.connect_response(|dlg, _resp| {
                                dlg.close();
                            });
                            dlg.present();
                        } else {
                            log::error!("Failed to reinit logger after deletion: {}", e);
                        }
                    }
                }

                // Update UI size label and button sensitivity
                let new_size = core::logic::logger::total_log_size_bytes();
                let updated_size_mb = new_size as f64 / (1024.0 * 1024.0);
                let updated_text = if translations_for_response
                    .log_size_template
                    .contains("{size_mb:.2}")
                {
                    translations_for_response
                        .log_size_template
                        .replace("{size_mb:.2}", &format!("{:.2}", updated_size_mb))
                } else if translations_for_response
                    .log_size_template
                    .contains("{size_mb}")
                {
                    translations_for_response
                        .log_size_template
                        .replace("{size_mb}", &format!("{:.2}", updated_size_mb))
                } else {
                    format!(
                        "{} {:.2}",
                        translations_for_response.log_size_template, updated_size_mb
                    )
                };
                size_label_clone2.set_text(&updated_text);
            }
            dlg.close();
        });
        dialog.present();
    });

    // Now connect the checkbox handler so it can update UI/shutdown/init immediately
    let size_label_clone_cb = size_label.clone();
    let delete_btn_clone_cb = delete_btn.clone();
    let translations_for_log = translations.clone();
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
                    &translations_for_log.log_enable_failed_title,
                );
                dlg.set_secondary_text(Some(&translations_for_log.log_enable_failed_message));
                dlg.connect_response(|dlg, _resp| {
                    dlg.close();
                });
                dlg.present();
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
        let updated_size_mb = new_size as f64 / (1024.0 * 1024.0);
        let updated_text = if translations_for_log
            .log_size_template
            .contains("{size_mb:.2}")
        {
            translations_for_log
                .log_size_template
                .replace("{size_mb:.2}", &format!("{:.2}", updated_size_mb))
        } else if translations_for_log.log_size_template.contains("{size_mb}") {
            translations_for_log
                .log_size_template
                .replace("{size_mb}", &format!("{:.2}", updated_size_mb))
        } else {
            format!(
                "{} {:.2}",
                translations_for_log.log_size_template, updated_size_mb
            )
        };
        size_label_clone_cb.set_text(&updated_text);
        delete_btn_clone_cb.set_sensitive(new_size > 0);
    });

    container.append(&info_label);
    container.append(&size_label);

    // Row with buttons
    let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    btn_box.append(&open_btn);
    btn_box.append(&delete_btn);

    // Refresh button to update size/sensitivity if logs changed externally
    let refresh_btn = Button::with_label(&translations.refresh_button);
    i18n.bind_button_label(
        &refresh_btn,
        Rc::new(|t: &Translations| t.settings.debug.refresh_button.clone()),
    );
    let size_label_for_refresh = size_label.clone();
    let delete_btn_for_refresh = delete_btn.clone();
    let translations_for_refresh = translations.clone();
    refresh_btn.connect_clicked(move |_| {
        let new_size = core::logic::logger::total_log_size_bytes();
        let updated_size_mb = new_size as f64 / (1024.0 * 1024.0);
        let updated_text = if translations_for_refresh
            .log_size_template
            .contains("{size_mb:.2}")
        {
            translations_for_refresh
                .log_size_template
                .replace("{size_mb:.2}", &format!("{:.2}", updated_size_mb))
        } else if translations_for_refresh
            .log_size_template
            .contains("{size_mb}")
        {
            translations_for_refresh
                .log_size_template
                .replace("{size_mb}", &format!("{:.2}", updated_size_mb))
        } else {
            format!(
                "{} {:.2}",
                translations_for_refresh.log_size_template, updated_size_mb
            )
        };
        size_label_for_refresh.set_text(&updated_text);
        delete_btn_for_refresh.set_sensitive(new_size > 0);
    });
    btn_box.append(&refresh_btn);

    container.append(&btn_box);

    container
}
