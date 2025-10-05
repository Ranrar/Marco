//! Debug settings tab
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, CheckButton, Label, Orientation};
use log::trace;

/// Builds the Debug tab UI. Provides a simple checkbox to enable/disable debug mode.
pub fn build_debug_tab(settings_path: &str) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 6);
    container.add_css_class("settings-tab-debug");
    container.set_margin_top(24);
    container.set_margin_bottom(24);
    container.set_margin_start(32);
    container.set_margin_end(32);

    let header = Label::new(Some("Debug"));
    header.set_markup("<b>Debug</b>");
    header.set_halign(Align::Start);
    header.set_xalign(0.0);

    let desc = Label::new(Some(
        "Enable debug features and diagnostics. Toggle to show/hide debug UI components.",
    ));
    desc.set_halign(Align::Start);
    desc.set_xalign(0.0);
    desc.set_wrap(true);

    // Use SettingsManager to load current setting (default to false)
    let settings_manager = match marco_core::logic::swanson::SettingsManager::initialize(
        std::path::PathBuf::from(settings_path)
    ) {
        Ok(sm) => sm,
        Err(_) => {
            log::warn!("Failed to initialize SettingsManager in debug tab, using defaults");
            return container;
        }
    };
    
    let current = settings_manager.get_settings().debug.unwrap_or(false);

    let checkbox = CheckButton::with_label("Enable debug mode (show debug UI and diagnostics)");
    checkbox.set_active(current);
    checkbox.set_halign(Align::Start);

    let settings_manager_clone = settings_manager.clone();
    checkbox.connect_toggled(move |cb| {
        let active = cb.is_active();
        if let Err(e) = settings_manager_clone.update_settings(|settings| {
            settings.debug = Some(active);
        }) {
            log::error!("Failed to update debug setting: {}", e);
        }
    });

    container.append(&header);
    container.append(&desc);
    container.append(&checkbox);

    // --- Program log controls ---
    let log_enabled = settings_manager.get_settings().log_to_file.unwrap_or(false);

    let prog_log_cb = CheckButton::with_label("Program log (write logs to file)");
    prog_log_cb.set_active(log_enabled);
    prog_log_cb.set_halign(Align::Start);

    // Wire checkbox to persist setting
    let settings_manager_clone2 = settings_manager.clone();
    prog_log_cb.connect_toggled(move |cb| {
        let active = cb.is_active();
        trace!("audit: user toggled program log: {}", active);
        if let Err(e) = settings_manager_clone2.update_settings(|settings| {
            settings.log_to_file = Some(active);
        }) {
            log::error!("Failed to update log_to_file setting: {}", e);
        }
    });

    container.append(&prog_log_cb);

    container
}
