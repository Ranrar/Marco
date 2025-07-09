use super::common::{create_settings_row, create_settings_section_header};
use crate::settings::core::{OriginalSettings, SettingsChangeTracker};
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, CheckButton, ComboBoxText, Label, Notebook, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

/// Helper to create a settings row: title/subtitle left, selector right, vertically aligned
fn create_settings_row_aligned(
    title: &str,
    subtitle: Option<&str>,
    selector: &impl IsA<gtk4::Widget>,
) -> Box {
    let row = Box::new(Orientation::Horizontal, 8);

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

    let selector_widget = selector.clone().upcast::<gtk4::Widget>();
    selector_widget.set_halign(Align::End);
    selector_widget.set_valign(Align::Center);
    row.append(&selector_widget);

    row
}

/// Create the layout settings page
pub fn create_layout_settings_page(
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

    let layout_box = Box::new(Orientation::Vertical, 8);
    let editor_left_radio = CheckButton::with_label("Editor Left, Preview Right");
    let editor_right_radio = CheckButton::with_label("Editor Right, Preview Left");
    editor_right_radio.set_group(Some(&editor_left_radio));
    let current_layout = &change_tracker.borrow().layout_mode;
    if current_layout == "editor-left" {
        editor_left_radio.set_active(true);
    } else {
        editor_right_radio.set_active(true);
    }
    editor_left_radio.connect_toggled({
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let original_settings = original_settings.clone();
        move |button| {
            if button.is_active() {
                change_tracker.borrow_mut().layout_mode = "editor-left".to_string();
                let has_changes = change_tracker.borrow().has_changes(&original_settings);
                save_button.set_sensitive(has_changes);
            }
        }
    });
    editor_right_radio.connect_toggled({
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let original_settings = original_settings.clone();
        move |button| {
            if button.is_active() {
                change_tracker.borrow_mut().layout_mode = "editor-right".to_string();
                let has_changes = change_tracker.borrow().has_changes(&original_settings);
                save_button.set_sensitive(has_changes);
            }
        }
    });
    let layout_row = create_settings_row_aligned(
        "Layout mode",
        Some("Choose whether the editor or preview appears on the left side"),
        &layout_box,
    );
    layout_box.append(&editor_left_radio);
    layout_box.append(&editor_right_radio);
    page_box.append(&layout_row);

    let view_combo = ComboBoxText::new();
    view_combo.append(Some("html"), "HTML Preview");
    view_combo.append(Some("code"), "Source Code");
    let current_view = &change_tracker.borrow().view_mode;
    view_combo.set_active_id(Some(current_view));
    view_combo.connect_changed({
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let original_settings = original_settings.clone();
        move |combo| {
            if let Some(selected) = combo.active_id() {
                change_tracker.borrow_mut().view_mode = selected.to_string();
                save_button.set_sensitive(change_tracker.borrow().has_changes(&original_settings));
            }
        }
    });
    let view_row = create_settings_row_aligned(
        "Default view mode",
        Some("Default view mode when opening files"),
        &view_combo,
    );
    page_box.append(&view_row);

    // Add page to notebook
    let label = Label::new(Some("Layout"));
    notebook.append_page(&page_box, Some(&label));
}
