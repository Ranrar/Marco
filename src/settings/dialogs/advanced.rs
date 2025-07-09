use super::common::create_settings_section_header;
use crate::settings::core::{OriginalSettings, SettingsChangeTracker};
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Notebook, Orientation};
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

/// Create the advanced settings page
pub fn create_advanced_settings_page(
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

    let css_button = Button::with_label("Select CSS File");
    let current_css = &change_tracker.borrow().custom_css_file;
    if !current_css.is_empty() {
        css_button.set_label(&format!("CSS File: {}", current_css));
    }
    css_button.connect_clicked({
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let original_settings = original_settings.clone();
        move |button| {
            let file_chooser = gtk4::FileChooserDialog::new(
                Some("Select CSS File"),
                Some(
                    button
                        .root()
                        .unwrap()
                        .downcast_ref::<gtk4::Window>()
                        .unwrap(),
                ),
                gtk4::FileChooserAction::Open,
                &[
                    ("Cancel", gtk4::ResponseType::Cancel),
                    ("Select", gtk4::ResponseType::Accept),
                ],
            );
            let filter = gtk4::FileFilter::new();
            filter.set_name(Some("CSS Files"));
            filter.add_pattern("*.css");
            file_chooser.add_filter(&filter);
            let button_clone = button.clone();
            let change_tracker = change_tracker.clone();
            let save_button = save_button.clone();
            let original_settings = original_settings.clone();
            file_chooser.connect_response(move |dialog, response| {
                if response == gtk4::ResponseType::Accept {
                    if let Some(file) = dialog.file() {
                        if let Some(path) = file.path() {
                            let path_str = path.to_str().unwrap_or("");
                            change_tracker.borrow_mut().custom_css_file = path_str.to_string();
                            button_clone.set_label(&format!("CSS File: {}", path.display()));
                            save_button.set_sensitive(
                                change_tracker.borrow().has_changes(&original_settings),
                            );
                        }
                    }
                }
                dialog.close();
            });
            file_chooser.show();
        }
    });
    let css_row = create_settings_row_aligned(
        "Custom CSS file",
        Some("Path to a custom CSS file to override preview styling"),
        &css_button,
    );
    page_box.append(&css_row);

    // Add page to notebook
    let label = Label::new(Some("Advanced"));
    notebook.append_page(&page_box, Some(&label));
}
