// ...existing code...
use crate::settings::core::{OriginalSettings, SettingsChangeTracker};
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

    let current_ratio = change_tracker.borrow().layout_ratio;

    let view_combo = ComboBoxText::new();
    view_combo.append(Some("html"), "HTML Preview");
    view_combo.append(Some("code"), "Source Code");
    
    // --- Editor/Viewer Split Ratio Slider ---

    let ratio_label = Label::new(Some(&format!("{}% editor / {}% viewer", current_ratio, 100 - current_ratio)));
    ratio_label.set_halign(Align::Start);
    ratio_label.set_valign(Align::Center);
    ratio_label.set_margin_bottom(6);

    let ratio_scale = gtk4::Scale::with_range(Orientation::Horizontal, 10.0, 90.0, 1.0);
    ratio_scale.set_value(current_ratio as f64);
    ratio_scale.set_digits(0);
    ratio_scale.set_hexpand(true);
    ratio_scale.set_valign(Align::Center);
    ratio_scale.set_tooltip_text(Some("Adjust the horizontal space for the editor (10–90%)"));
    ratio_scale.set_width_request(320);
    ratio_scale.set_margin_bottom(8);

    // Update label and change_tracker on slider move
    ratio_scale.connect_value_changed({
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let original_settings = original_settings.clone();
        let ratio_label = ratio_label.clone();
        move |scale| {
            let value = scale.value().round() as i32;
            change_tracker.borrow_mut().layout_ratio = value;
            ratio_label.set_text(&format!("{}% editor / {}% viewer", value, 100 - value));
            save_button.set_sensitive(change_tracker.borrow().has_changes(&original_settings));
        }
    });

    let ratio_col = Box::new(Orientation::Vertical, 4);
    ratio_col.append(&ratio_label);
    ratio_col.append(&ratio_scale);
    let ratio_settings_row = create_settings_row_aligned(
        "Editor/Viewer Split",
        Some("Adjust how much horizontal space the editor takes (10–90%)"),
        &ratio_col,
    );
    page_box.append(&ratio_settings_row);

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
