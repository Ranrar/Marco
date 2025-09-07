//! Markdown-specific settings tab
use crate::logic::swanson::Settings as AppSettings;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation, Switch};

/// Builds the Markdown tab UI for markdown-specific engine settings
pub fn build_markdown_tab(settings_path: &str) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("settings-tab-markdown");
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

    // === HTML RENDERING SETTINGS ===
    let html_header = Label::new(Some("HTML Output Configuration"));
    html_header.set_markup("<b>HTML Output Configuration</b>");
    html_header.set_halign(Align::Start);
    html_header.set_xalign(0.0);
    html_header.set_margin_bottom(16);
    container.append(&html_header);

    // Table of Contents Generation
    let toc_enabled = load_settings()
        .render
        .and_then(|r| r.html)
        .and_then(|h| h.generate_toc)
        .unwrap_or(false);
    let toc_switch = Switch::new();
    toc_switch.set_active(toc_enabled);

    {
        let settings_path = settings_path.to_string();
        toc_switch.connect_state_set(move |_switch, state| {
            let mut settings = AppSettings::load_from_file(&settings_path).unwrap_or_default();

            if settings.engine.is_none() {
                settings.engine = Some(crate::logic::swanson::EngineSettings::default());
            }
            if let Some(ref mut engine) = settings.engine {
                if engine.render.is_none() {
                    engine.render = Some(crate::logic::swanson::EngineRenderSettings::default());
                }
                if let Some(ref mut render) = engine.render {
                    if render.html.is_none() {
                        render.html = Some(crate::logic::swanson::EngineHtmlSettings::default());
                    }
                    if let Some(ref mut html) = render.html {
                        html.generate_toc = Some(state);
                    }
                }
            }

            let _ = settings.save_to_file(&settings_path);
            glib::Propagation::Proceed
        });
    }

    // Generate Table of Contents (Toggle)
    let toc_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let toc_header = Label::new(Some("Generate Table of Contents"));
    toc_header.set_markup("<b>Generate Table of Contents</b>");
    toc_header.set_halign(Align::Start);
    toc_header.set_xalign(0.0);

    let toc_spacer = GtkBox::new(Orientation::Horizontal, 0);
    toc_spacer.set_hexpand(true);

    toc_switch.set_halign(Align::End);

    toc_hbox.append(&toc_header);
    toc_hbox.append(&toc_spacer);
    toc_hbox.append(&toc_switch);
    toc_hbox.set_margin_top(8);
    toc_hbox.set_margin_bottom(4);
    container.append(&toc_hbox);

    // Description text under header
    let toc_description = Label::new(Some(
        "Automatically create a navigation table of contents from document headings.",
    ));
    toc_description.set_halign(Align::Start);
    toc_description.set_xalign(0.0);
    toc_description.set_wrap(true);
    toc_description.add_css_class("dim-label");
    toc_description.set_margin_bottom(12);
    container.append(&toc_description);

    // Include HTML Metadata
    let metadata_enabled = load_settings()
        .render
        .and_then(|r| r.html)
        .and_then(|h| h.include_metadata)
        .unwrap_or(false);
    let metadata_switch = Switch::new();
    metadata_switch.set_active(metadata_enabled);

    {
        let settings_path = settings_path.to_string();
        metadata_switch.connect_state_set(move |_switch, state| {
            let mut settings = AppSettings::load_from_file(&settings_path).unwrap_or_default();

            if settings.engine.is_none() {
                settings.engine = Some(crate::logic::swanson::EngineSettings::default());
            }
            if let Some(ref mut engine) = settings.engine {
                if engine.render.is_none() {
                    engine.render = Some(crate::logic::swanson::EngineRenderSettings::default());
                }
                if let Some(ref mut render) = engine.render {
                    if render.html.is_none() {
                        render.html = Some(crate::logic::swanson::EngineHtmlSettings::default());
                    }
                    if let Some(ref mut html) = render.html {
                        html.include_metadata = Some(state);
                    }
                }
            }

            let _ = settings.save_to_file(&settings_path);
            glib::Propagation::Proceed
        });
    }

    // Include HTML Metadata (Toggle)
    let metadata_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let metadata_header = Label::new(Some("Include HTML Metadata"));
    metadata_header.set_markup("<b>Include HTML Metadata</b>");
    metadata_header.set_halign(Align::Start);
    metadata_header.set_xalign(0.0);

    let metadata_spacer = GtkBox::new(Orientation::Horizontal, 0);
    metadata_spacer.set_hexpand(true);

    metadata_switch.set_halign(Align::End);

    metadata_hbox.append(&metadata_header);
    metadata_hbox.append(&metadata_spacer);
    metadata_hbox.append(&metadata_switch);
    metadata_hbox.set_margin_top(8);
    metadata_hbox.set_margin_bottom(4);
    container.append(&metadata_hbox);

    // Description text under header
    let metadata_description = Label::new(Some("Add document metadata (title, author, description) to HTML head section for better SEO and document identification."));
    metadata_description.set_halign(Align::Start);
    metadata_description.set_xalign(0.0);
    metadata_description.set_wrap(true);
    metadata_description.add_css_class("dim-label");
    metadata_description.set_margin_bottom(12);
    container.append(&metadata_description);

    container
}
