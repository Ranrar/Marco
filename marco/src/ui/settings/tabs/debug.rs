//! Debug settings tab
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, CheckButton, Orientation};
use log::trace;

// Import unified helper
use super::helpers::add_setting_row;

/// Builds the Debug tab UI. Provides a simple checkbox to enable/disable debug mode.
pub fn build_debug_tab(settings_path: &str) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("marco-settings-tab");

    // Use SettingsManager to load current setting (default to false)
    let settings_manager = match core::logic::swanson::SettingsManager::initialize(
        std::path::PathBuf::from(settings_path)
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

    // Wire checkbox to persist setting
    let settings_manager_clone2 = settings_manager.clone();
    log_checkbox.connect_toggled(move |cb| {
        let active = cb.is_active();
        trace!("audit: user toggled program log: {}", active);
        if let Err(e) = settings_manager_clone2.update_settings(|settings| {
            settings.log_to_file = Some(active);
        }) {
            log::error!("Failed to update log_to_file setting: {}", e);
        }
    });

    // Create program log row using unified helper
    let log_row = add_setting_row(
        "Program Log",
        "Write program logs to file for troubleshooting. Log files are stored in the application data directory.",
        &log_checkbox,
        false  // Not first row
    );
    container.append(&log_row);

    container
}
