//! Markdown-specific settings tab
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation};

use super::helpers::SettingsI18nRegistry;

/// Builds the Markdown tab UI.
/// Currently empty — any future markdown engine toggles go here.
pub fn build_markdown_tab(_settings_path: &str, _i18n: &SettingsI18nRegistry) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("marco-settings-tab");
    container
}
