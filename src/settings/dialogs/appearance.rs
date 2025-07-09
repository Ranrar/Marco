use super::common::create_settings_section_header;
use crate::settings::core::{OriginalSettings, SettingsChangeTracker};
use crate::settings::ui::{get_available_css_themes, get_available_languages};
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, ComboBoxText, Label, Notebook, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

/// Helper to create a settings row: title/subtitle left, selector right, vertically aligned
fn create_settings_row_aligned(
    title: &str,
    subtitle: Option<&str>,
    selector: &impl IsA<gtk4::Widget>,
) -> Box {
    let row = Box::new(Orientation::Horizontal, 8);

    // Left: title + subtitle in a vertical box
    let left = Box::new(Orientation::Vertical, 2);
    let title_label = Label::new(Some(title));
    title_label.set_halign(Align::Start);
    title_label.set_valign(Align::Center);
    title_label.add_css_class("settings-row-title");
    left.append(&title_label);

    if let Some(sub) = subtitle {
        let subtitle_label = Label::new(Some(sub));
        subtitle_label.set_halign(Align::Start);
        subtitle_label.set_valign(Align::Center);
        subtitle_label.add_css_class("settings-row-subtitle");
        left.append(&subtitle_label);
    }

    left.set_hexpand(true);
    row.append(&left);

    // Right: selector
    let selector_widget = selector.clone().upcast::<gtk4::Widget>();
    selector_widget.set_halign(Align::End);
    selector_widget.set_valign(Align::Center);
    row.append(&selector_widget);

    row
}

/// Create the appearance settings page
pub fn create_appearance_settings_page(
    notebook: &Notebook,
    change_tracker: &Rc<RefCell<SettingsChangeTracker>>,
    save_button: &Button,
    original_settings: &OriginalSettings,
) {
    let page_box = Box::new(Orientation::Vertical, 16);
    page_box.set_margin_top(24);
    page_box.set_margin_bottom(24);
    page_box.set_margin_start(24);
    page_box.set_margin_end(24);
    page_box.add_css_class("settings-page");

    let ui_theme_combo = ComboBoxText::new();
    ui_theme_combo.append(Some("system"), "Follow System");
    ui_theme_combo.append(Some("light"), "Light");
    ui_theme_combo.append(Some("dark"), "Dark");
    let current_ui_theme = &change_tracker.borrow().ui_theme;
    ui_theme_combo.set_active_id(Some(current_ui_theme));
    ui_theme_combo.connect_changed({
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let original_settings = original_settings.clone();
        move |combo| {
            if let Some(selected) = combo.active_id() {
                change_tracker.borrow_mut().ui_theme = selected.to_string();
                save_button.set_sensitive(change_tracker.borrow().has_changes(&original_settings));
            }
        }
    });
    let ui_theme_row = create_settings_row_aligned(
        "Application theme",
        Some("System will follow your desktop environment's theme"),
        &ui_theme_combo,
    );
    page_box.append(&ui_theme_row);

    let css_theme_combo = ComboBoxText::new();
    let available_themes = get_available_css_themes();
    for theme in &available_themes {
        css_theme_combo.append(Some(theme), theme);
    }
    let current_css_theme = &change_tracker.borrow().css_theme;
    css_theme_combo.set_active_id(Some(current_css_theme));
    css_theme_combo.connect_changed({
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let original_settings = original_settings.clone();
        move |combo| {
            if let Some(selected) = combo.active_id() {
                change_tracker.borrow_mut().css_theme = selected.to_string();
                save_button.set_sensitive(change_tracker.borrow().has_changes(&original_settings));
            }
        }
    });
    let css_theme_row = create_settings_row_aligned(
        "Preview theme",
        Some("CSS theme used for rendering the Markdown preview"),
        &css_theme_combo,
    );
    page_box.append(&css_theme_row);

    // Add page to notebook
    let label = Label::new(Some("Appearance"));
    notebook.append_page(&page_box, Some(&label));
}
