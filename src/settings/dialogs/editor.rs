use super::common::create_settings_section_header;
use crate::settings::core::{OriginalSettings, SettingsChangeTracker};
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Notebook, Orientation, Switch};
use std::cell::RefCell;
use std::rc::Rc;

/// Helper: Create a row with label left, switch right, vertically centered, with optional subtitle below the title.
fn create_toggle_row(title: &str, subtitle: Option<&str>, switch: &Switch) -> Box {
    let row = Box::new(Orientation::Horizontal, 8);
    row.set_halign(Align::Fill);
    row.set_valign(Align::Center);
    row.set_margin_top(4);
    row.set_margin_bottom(4);

    // Left: Title and optional subtitle in a vertical box
    let label_box = Box::new(Orientation::Vertical, 2);
    let title_label = Label::new(Some(title));
    title_label.set_halign(Align::Start);
    title_label.set_valign(Align::Center);
    title_label.set_hexpand(true);
    title_label.add_css_class("settings-row-title");
    label_box.append(&title_label);

    if let Some(sub) = subtitle {
        let subtitle_label = Label::new(Some(sub));
        subtitle_label.set_halign(Align::Start);
        subtitle_label.set_valign(Align::Center);
        subtitle_label.add_css_class("settings-row-subtitle");
        label_box.append(&subtitle_label);
    }

    label_box.set_hexpand(true);
    row.append(&label_box);

    // Right: Switch
    switch.set_halign(Align::End);
    switch.set_valign(Align::Center);
    row.append(switch);

    row
}

/// Create the editor settings page
pub fn create_editor_settings_page(
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

    // Function highlighting section
    let function_switch = Switch::new();
    function_switch.set_active(change_tracker.borrow().function_highlighting);
    function_switch.connect_active_notify({
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let original_settings = original_settings.clone();
        move |switch| {
            change_tracker.borrow_mut().function_highlighting = switch.is_active();
            save_button.set_sensitive(change_tracker.borrow().has_changes(&original_settings));
        }
    });
    let function_row = create_toggle_row(
        "Function Highlighting",
        Some("Visual emphasis for function-related areas when hovering"),
        &function_switch,
    );
    page_box.append(&function_row);

    // Syntax color section
    let syntax_color_switch = Switch::new();
    syntax_color_switch.set_active(change_tracker.borrow().editor_color_syntax);
    syntax_color_switch.connect_active_notify({
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let original_settings = original_settings.clone();
        move |switch| {
            change_tracker.borrow_mut().editor_color_syntax = switch.is_active();
            save_button.set_sensitive(change_tracker.borrow().has_changes(&original_settings));
        }
    });
    let syntax_color_row = create_toggle_row(
        "Syntax Color",
        Some("Apply syntax highlighting colors to markdown text in the editor"),
        &syntax_color_switch,
    );
    page_box.append(&syntax_color_row);

    // Markdown spell check section
    let markdown_switch = Switch::new();
    markdown_switch.set_active(change_tracker.borrow().markdown_warnings);
    markdown_switch.connect_active_notify({
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let original_settings = original_settings.clone();
        move |switch| {
            change_tracker.borrow_mut().markdown_warnings = switch.is_active();
            save_button.set_sensitive(change_tracker.borrow().has_changes(&original_settings));
        }
    });
    let markdown_row = create_toggle_row(
        "Show Markdown Errors",
        Some("Display errors for misspelled syntax and formatting issues"),
        &markdown_switch,
    );
    page_box.append(&markdown_row);

    // Text wrap section
    let text_wrap_switch = Switch::new();
    text_wrap_switch.set_active(change_tracker.borrow().editor_text_wrap);
    text_wrap_switch.connect_active_notify({
        let change_tracker = change_tracker.clone();
        let save_button = save_button.clone();
        let original_settings = original_settings.clone();
        move |switch| {
            change_tracker.borrow_mut().editor_text_wrap = switch.is_active();
            save_button.set_sensitive(change_tracker.borrow().has_changes(&original_settings));
        }
    });
    let text_wrap_row = create_toggle_row(
        "Text Wrap",
        Some("Wrap long lines in the editor instead of horizontal scrolling"),
        &text_wrap_switch,
    );
    page_box.append(&text_wrap_row);

    // Add expandable explanation of errors types (keep as requested)
    let expander = gtk4::Expander::new(Some("What do the different errors types mean?"));
    expander.set_margin_top(8);
    expander.set_margin_bottom(8);
    expander.set_margin_start(20);
    expander.set_margin_end(20);

    let explanation_box = Box::new(Orientation::Vertical, 12);
    explanation_box.set_margin_top(12);
    explanation_box.set_margin_bottom(12);
    explanation_box.set_margin_start(8);
    explanation_box.set_margin_end(8);

    let warning_types = vec![
        (
            "🔴",
            "Syntax Errors",
            "Broken links, unclosed code blocks, malformed tables",
        ),
        (
            "🟠",
            "Formatting Issues",
            "Missing alt text, empty links, improper headings",
        ),
        (
            "🟡",
            "Style Warnings",
            "Raw HTML usage, inconsistent list markers",
        ),
        (
            "🔵",
            "Structure Issues",
            "Invalid references, unclosed emphasis markers",
        ),
    ];
    for (icon, category, description) in warning_types {
        let warning_box = Box::new(Orientation::Horizontal, 12);
        warning_box.set_margin_bottom(6);
        let icon_label = Label::new(Some(icon));
        icon_label.set_halign(gtk4::Align::Center);
        icon_label.set_valign(gtk4::Align::Start);
        icon_label.set_size_request(24, -1);
        let content_box = Box::new(Orientation::Vertical, 2);
        let category_label = Label::new(Some(category));
        category_label.set_markup(&format!("<b>{}</b>", category));
        category_label.set_halign(gtk4::Align::Start);
        let description_label = Label::new(Some(description));
        description_label.set_halign(gtk4::Align::Start);
        description_label.set_wrap(true);
        description_label.set_wrap_mode(gtk4::pango::WrapMode::Word);
        description_label.add_css_class("dim-label");
        content_box.append(&category_label);
        content_box.append(&description_label);
        warning_box.append(&icon_label);
        warning_box.append(&content_box);
        explanation_box.append(&warning_box);
    }
    expander.set_child(Some(&explanation_box));
    page_box.append(&expander);

    // Add page to notebook
    let label = Label::new(Some("Editor"));
    notebook.append_page(&page_box, Some(&label));
}
