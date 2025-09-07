//! Debug settings tab
use crate::logic::swanson::Settings as AppSettings;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, CheckButton, ComboBoxText, Label, Orientation};
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

    // Load current setting (default to false)
    let current = AppSettings::load_from_file(settings_path)
        .unwrap_or_default()
        .debug
        .unwrap_or(false);

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

    // === ENGINE DEBUG SETTINGS ===
    let engine_header = Label::new(Some("Engine Debug Settings"));
    engine_header.set_markup("<b>Engine Debug Settings</b>");
    engine_header.set_halign(Align::Start);
    engine_header.set_xalign(0.0);
    engine_header.set_margin_top(16);
    engine_header.set_margin_bottom(4);
    container.append(&engine_header);

    // Description text under header
    let engine_description = Label::new(Some("Debug features for engine development and troubleshooting. Cache parse results for better performance, enable detailed error reports with stack traces, and select JSON output format for debugging and data analysis."));
    engine_description.set_halign(Align::Start);
    engine_description.set_xalign(0.0);
    engine_description.set_wrap(true);
    engine_description.add_css_class("dim-label");
    engine_description.set_margin_bottom(12);
    container.append(&engine_description);

    // Helper function to load settings
    let load_settings = || -> crate::logic::swanson::EngineSettings {
        AppSettings::load_from_file(settings_path)
            .unwrap_or_default()
            .engine
            .unwrap_or_default()
    };

    // Parse Caching (moved from advanced)
    let cache_enabled = load_settings()
        .parser
        .and_then(|p| p.enable_cache)
        .unwrap_or(false);
    let cache_checkbox = CheckButton::with_label("Enable parse caching");
    cache_checkbox.set_active(cache_enabled);
    cache_checkbox.set_halign(Align::Start);
    cache_checkbox.set_margin_bottom(8);

    {
        let settings_path = settings_path.to_string();
        cache_checkbox.connect_toggled(move |checkbox| {
            let state = checkbox.is_active();
            let mut settings = AppSettings::load_from_file(&settings_path).unwrap_or_default();

            // Ensure engine.parser settings exist
            if settings.engine.is_none() {
                settings.engine = Some(crate::logic::swanson::EngineSettings::default());
            }
            if let Some(ref mut engine) = settings.engine {
                if engine.parser.is_none() {
                    engine.parser = Some(crate::logic::swanson::EngineParserSettings::default());
                }
                if let Some(ref mut parser) = engine.parser {
                    parser.enable_cache = Some(state);
                }
            }

            let _ = settings.save_to_file(&settings_path);
        });
    }

    container.append(&cache_checkbox);

    // Detailed Errors (moved from advanced)
    let detailed_errors = load_settings()
        .parser
        .and_then(|p| p.detailed_errors)
        .unwrap_or(true);
    let errors_checkbox = CheckButton::with_label("Enable detailed error reports");
    errors_checkbox.set_active(detailed_errors);
    errors_checkbox.set_halign(Align::Start);
    errors_checkbox.set_margin_bottom(8);

    {
        let settings_path = settings_path.to_string();
        errors_checkbox.connect_toggled(move |checkbox| {
            let state = checkbox.is_active();
            let mut settings = AppSettings::load_from_file(&settings_path).unwrap_or_default();

            if settings.engine.is_none() {
                settings.engine = Some(crate::logic::swanson::EngineSettings::default());
            }
            if let Some(ref mut engine) = settings.engine {
                if engine.parser.is_none() {
                    engine.parser = Some(crate::logic::swanson::EngineParserSettings::default());
                }
                if let Some(ref mut parser) = engine.parser {
                    parser.detailed_errors = Some(state);
                }
            }

            let _ = settings.save_to_file(&settings_path);
        });
    }

    container.append(&errors_checkbox);

    // JSON Output Format (moved from advanced)
    let current_format = load_settings()
        .render
        .and_then(|r| r.default_format)
        .unwrap_or_else(|| "html".to_string());

    // Create a simple label for the ComboBox
    let json_label = Label::new(Some("JSON Output Format:"));
    json_label.set_halign(Align::Start);
    json_label.set_xalign(0.0);
    json_label.set_margin_bottom(4);
    container.append(&json_label);

    let json_combo = ComboBoxText::new();
    json_combo.append_text("JSON");
    json_combo.append_text("Pretty JSON");
    json_combo.set_halign(Align::Start);
    json_combo.set_margin_bottom(8);

    let json_index = match current_format.as_str() {
        "json" => 0,
        "json_pretty" => 1,
        _ => 0,
    };
    json_combo.set_active(Some(json_index));

    {
        let settings_path = settings_path.to_string();
        json_combo.connect_changed(move |combo| {
            if let Some(index) = combo.active() {
                let format = match index {
                    0 => "json",
                    1 => "json_pretty",
                    _ => "json",
                };

                let mut settings = AppSettings::load_from_file(&settings_path).unwrap_or_default();

                if settings.engine.is_none() {
                    settings.engine = Some(crate::logic::swanson::EngineSettings::default());
                }
                if let Some(ref mut engine) = settings.engine {
                    if engine.render.is_none() {
                        engine.render =
                            Some(crate::logic::swanson::EngineRenderSettings::default());
                    }
                    if let Some(ref mut render) = engine.render {
                        render.default_format = Some(format.to_string());
                    }
                }

                let _ = settings.save_to_file(&settings_path);
            }
        });
    }

    container.append(&json_combo);

    container
}
