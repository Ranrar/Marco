//! Markdown-specific settings tab
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Switch};

// Import unified helper and section header helper
use super::helpers::add_setting_row;

/// Builds the Markdown tab UI for markdown-specific engine settings
pub fn build_markdown_tab(settings_path: &str) -> GtkBox {
    // Initialize SettingsManager for this tab
    let settings_manager = match core::logic::swanson::SettingsManager::initialize(
        std::path::PathBuf::from(settings_path),
    ) {
        Ok(sm) => sm,
        Err(_) => {
            log::warn!("Failed to initialize SettingsManager in markdown tab, using defaults");
            return GtkBox::new(Orientation::Vertical, 0);
        }
    };

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("marco-settings-tab");

    // Load current engine settings using SettingsManager
    let load_settings = || settings_manager.get_settings().engine.unwrap_or_default();

    // Table of Contents Generation
    let toc_enabled = load_settings()
        .render
        .and_then(|r| r.html)
        .and_then(|h| h.generate_toc)
        .unwrap_or(false);
    let toc_switch = Switch::new();
    toc_switch.add_css_class("marco-switch");
    toc_switch.set_active(toc_enabled);

    {
        let settings_manager_clone = settings_manager.clone();
        toc_switch.connect_state_set(move |_switch, state| {
            if let Err(e) = settings_manager_clone.update_settings(|settings| {
                if settings.engine.is_none() {
                    settings.engine = Some(core::logic::swanson::EngineSettings::default());
                }
                if let Some(ref mut engine) = settings.engine {
                    if engine.render.is_none() {
                        engine.render = Some(core::logic::swanson::EngineRenderSettings::default());
                    }
                    if let Some(ref mut render) = engine.render {
                        if render.html.is_none() {
                            render.html = Some(core::logic::swanson::EngineHtmlSettings::default());
                        }
                        if let Some(ref mut html) = render.html {
                            html.generate_toc = Some(state);
                        }
                    }
                }
            }) {
                log::error!("Failed to update TOC setting: {}", e);
            }
            glib::Propagation::Proceed
        });
    }

    // Create TOC row using unified helper (first row after section header)
    let toc_row = add_setting_row(
        "Generate Table of Contents",
        "Automatically create a navigation table of contents from document headings.",
        &toc_switch,
        true, // First row after section header - no additional top margin
    );
    container.append(&toc_row);

    // Include HTML Metadata
    let metadata_enabled = load_settings()
        .render
        .and_then(|r| r.html)
        .and_then(|h| h.include_metadata)
        .unwrap_or(false);
    let metadata_switch = Switch::new();
    metadata_switch.add_css_class("marco-switch");
    metadata_switch.set_active(metadata_enabled);

    {
        let settings_manager_clone2 = settings_manager.clone();
        metadata_switch.connect_state_set(move |_switch, state| {
            if let Err(e) = settings_manager_clone2.update_settings(|settings| {
                if settings.engine.is_none() {
                    settings.engine = Some(core::logic::swanson::EngineSettings::default());
                }
                if let Some(ref mut engine) = settings.engine {
                    if engine.render.is_none() {
                        engine.render = Some(core::logic::swanson::EngineRenderSettings::default());
                    }
                    if let Some(ref mut render) = engine.render {
                        if render.html.is_none() {
                            render.html = Some(core::logic::swanson::EngineHtmlSettings::default());
                        }
                        if let Some(ref mut html) = render.html {
                            html.include_metadata = Some(state);
                        }
                    }
                }
            }) {
                log::error!("Failed to update metadata setting: {}", e);
            }
            glib::Propagation::Proceed
        });
    }

    // Create metadata row using unified helper
    let metadata_row = add_setting_row(
        "Include HTML Metadata",
        "Add document metadata (title, author, description) to HTML head section for better SEO and document identification.",
        &metadata_switch,
        false  // Not first row
    );
    container.append(&metadata_row);

    container
}
