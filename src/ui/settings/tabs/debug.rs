//! Debug settings tab
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, CheckButton, Align};
use crate::logic::swanson::Settings as AppSettings;
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

    let desc = Label::new(Some("Enable debug features and diagnostics. Toggle to show/hide debug UI components."));
    desc.set_halign(Align::Start);
    desc.set_xalign(0.0);
    desc.set_wrap(true);

    // Load current setting (default to false)
    let current = AppSettings::load_from_file(settings_path).unwrap_or_default().debug.unwrap_or(false);

    let checkbox = CheckButton::with_label("Enable debug mode (show debug UI and diagnostics)");
    checkbox.set_active(current);
    checkbox.set_halign(Align::Start);

    let settings_path_owned1 = settings_path.to_string();
    checkbox.connect_toggled(move |cb| {
        let active = cb.is_active();
        let mut settings = AppSettings::load_from_file(&settings_path_owned1).unwrap_or_default();
        settings.debug = Some(active);
        let _ = settings.save_to_file(&settings_path_owned1);
    });

    container.append(&header);
    container.append(&desc);
    container.append(&checkbox);

    // --- Program log controls ---
    let app_settings = AppSettings::load_from_file(settings_path).unwrap_or_default();
    let log_enabled = app_settings.log_to_file.unwrap_or(false);

    let prog_log_cb = CheckButton::with_label("Program log (write logs to file)");
    prog_log_cb.set_active(log_enabled);
    prog_log_cb.set_halign(Align::Start);

    // Wire checkbox to persist setting
    let settings_path_owned2 = settings_path.to_string();
    let settings_path_clone = settings_path_owned2.clone();
    prog_log_cb.connect_toggled(move |cb| {
        let active = cb.is_active();
        trace!("audit: user toggled program log: {}", active);
        let mut settings = AppSettings::load_from_file(&settings_path_clone).unwrap_or_default();
        settings.log_to_file = Some(active);
        let _ = settings.save_to_file(&settings_path_clone);
    });

    container.append(&prog_log_cb);

    container
}
