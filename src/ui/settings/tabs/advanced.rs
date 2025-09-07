use crate::logic::swanson::Settings as AppSettings;
use gtk4::prelude::*;
use gtk4::{Adjustment, Align, Box as GtkBox, Label, Orientation, SpinButton, Switch};

/// Builds the Advanced tab UI for Marco Engine settings
pub fn build_advanced_tab(settings_path: &str) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("settings-tab-advanced");
    container.set_margin_top(24);
    container.set_margin_bottom(24);
    container.set_margin_start(32);
    container.set_margin_end(32);

    // Load current engine settings
    let load_settings = || {
        AppSettings::load_from_file(settings_path)
            .unwrap_or_default()
            .engine
            .unwrap_or_default()
    };

    // === PERFORMANCE SETTINGS ===
    let perf_header = Label::new(Some("Performance Configuration"));
    perf_header.set_markup("<b>Performance Configuration</b>");
    perf_header.set_halign(Align::Start);
    perf_header.set_xalign(0.0);
    perf_header.set_margin_bottom(16);
    container.append(&perf_header);

    // AST Caching
    let ast_cache_enabled = load_settings()
        .performance
        .and_then(|p| p.cache_ast)
        .unwrap_or(false);
    let ast_cache_switch = Switch::new();
    ast_cache_switch.set_active(ast_cache_enabled);

    {
        let settings_path = settings_path.to_string();
        ast_cache_switch.connect_state_set(move |_switch, state| {
            let mut settings = AppSettings::load_from_file(&settings_path).unwrap_or_default();

            if settings.engine.is_none() {
                settings.engine = Some(crate::logic::swanson::EngineSettings::default());
            }
            if let Some(ref mut engine) = settings.engine {
                if engine.performance.is_none() {
                    engine.performance =
                        Some(crate::logic::swanson::EnginePerformanceSettings::default());
                }
                if let Some(ref mut performance) = engine.performance {
                    performance.cache_ast = Some(state);
                }
            }

            let _ = settings.save_to_file(&settings_path);
            glib::Propagation::Proceed
        });
    }

    // Enable AST Caching (Toggle)
    let ast_cache_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let ast_cache_header = Label::new(Some("Enable AST Caching"));
    ast_cache_header.set_markup("<b>Enable AST Caching</b>");
    ast_cache_header.set_halign(Align::Start);
    ast_cache_header.set_xalign(0.0);

    let ast_cache_spacer = GtkBox::new(Orientation::Horizontal, 0);
    ast_cache_spacer.set_hexpand(true);

    ast_cache_switch.set_halign(Align::End);

    ast_cache_hbox.append(&ast_cache_header);
    ast_cache_hbox.append(&ast_cache_spacer);
    ast_cache_hbox.append(&ast_cache_switch);
    ast_cache_hbox.set_margin_top(8);
    ast_cache_hbox.set_margin_bottom(4);
    container.append(&ast_cache_hbox);

    // Description text under header
    let ast_cache_description = Label::new(Some(
        "Cache Abstract Syntax Trees for improved performance with repeated operations.",
    ));
    ast_cache_description.set_halign(Align::Start);
    ast_cache_description.set_xalign(0.0);
    ast_cache_description.set_wrap(true);
    ast_cache_description.add_css_class("dim-label");
    ast_cache_description.set_margin_bottom(12);
    container.append(&ast_cache_description);

    // AST Cache Size (with MB indicator)
    let cache_size = load_settings()
        .parser
        .and_then(|p| p.max_cache_size)
        .unwrap_or(50) as f64;

    let adjustment = Adjustment::new(cache_size, 1.0, 1000.0, 1.0, 10.0, 0.0);
    let cache_size_spin = SpinButton::new(Some(&adjustment), 1.0, 0);
    cache_size_spin.set_value(cache_size);

    // Create a label to show estimated MB usage
    let mb_label = Label::new(Some(&format!("≈ {:.1} MB", cache_size * 0.1))); // Rough estimate: 0.1MB per cached AST
    mb_label.set_halign(Align::Start);
    mb_label.add_css_class("dim-label");

    // Create a horizontal box for the spin button and MB label
    let cache_size_box = GtkBox::new(Orientation::Horizontal, 8);
    cache_size_box.append(&cache_size_spin);
    cache_size_box.append(&mb_label);

    {
        let settings_path = settings_path.to_string();
        let mb_label_clone = mb_label.clone();
        cache_size_spin.connect_value_changed(move |spin| {
            let value = spin.value() as usize;
            let mut settings = AppSettings::load_from_file(&settings_path).unwrap_or_default();

            if settings.engine.is_none() {
                settings.engine = Some(crate::logic::swanson::EngineSettings::default());
            }
            if let Some(ref mut engine) = settings.engine {
                if engine.parser.is_none() {
                    engine.parser = Some(crate::logic::swanson::EngineParserSettings::default());
                }
                if let Some(ref mut parser) = engine.parser {
                    parser.max_cache_size = Some(value);
                }
            }

            // Update MB label
            mb_label_clone.set_text(&format!("≈ {:.1} MB", value as f64 * 0.1));

            let _ = settings.save_to_file(&settings_path);
        });
    }

    // AST Cache Size (SpinButton)
    let cache_size_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let cache_size_header = Label::new(Some("AST Cache Size"));
    cache_size_header.set_markup("<b>AST Cache Size</b>");
    cache_size_header.set_halign(Align::Start);
    cache_size_header.set_xalign(0.0);

    let cache_size_spacer = GtkBox::new(Orientation::Horizontal, 0);
    cache_size_spacer.set_hexpand(true);

    cache_size_box.set_halign(Align::End);

    cache_size_hbox.append(&cache_size_header);
    cache_size_hbox.append(&cache_size_spacer);
    cache_size_hbox.append(&cache_size_box);
    cache_size_hbox.set_margin_top(8);
    cache_size_hbox.set_margin_bottom(4);
    container.append(&cache_size_hbox);

    // Description text under header
    let cache_size_description = Label::new(Some("Number of Abstract Syntax Trees to keep in memory cache (higher = more memory, better performance)."));
    cache_size_description.set_halign(Align::Start);
    cache_size_description.set_xalign(0.0);
    cache_size_description.set_wrap(true);
    cache_size_description.add_css_class("dim-label");
    cache_size_description.set_margin_bottom(12);
    container.append(&cache_size_description);

    container
}
