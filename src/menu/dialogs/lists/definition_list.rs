// Definition list dialog - Create custom definition list with specified number of items
// Simple dialog with number input and preview

use crate::menu::dialogs::common::*;
use crate::{editor, language};
use gtk4::prelude::*;

/// Show dialog to create a custom definition list with specified number of items
pub fn show_definition_list_custom_dialog(window: &gtk4::Window, editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&language::tr("insert.definition_list_custom")),
        Some(window),
        gtk4::DialogFlags::MODAL,
        &[
            (&language::tr("table_dialog.insert"), ResponseType::Accept),
            (&language::tr("table_dialog.cancel"), ResponseType::Cancel),
        ],
    );
    let content_area = dialog.content_area();

    // Create main container
    let main_container = gtk4::Box::new(Orientation::Vertical, 12);
    main_container.set_margin_top(12);
    main_container.set_margin_bottom(12);
    main_container.set_margin_start(12);
    main_container.set_margin_end(12);

    // Add title label
    let title_label = Label::new(Some(
        "Create a custom definition list with a specified number of definition pairs",
    ));
    title_label.set_halign(gtk4::Align::Start);
    main_container.append(&title_label);

    // Create grid for input fields
    let input_grid = Grid::new();
    input_grid.set_row_spacing(8);
    input_grid.set_column_spacing(12);
    input_grid.set_margin_top(12);

    // Number of definition pairs input
    let pairs_label = Label::new(Some("Number of definition pairs:"));
    pairs_label.set_halign(gtk4::Align::End);
    input_grid.attach(&pairs_label, 0, 0, 1, 1);

    // Create spin button with range 1-20, default 2
    let adjustment = Adjustment::new(2.0, 1.0, 20.0, 1.0, 5.0, 0.0);
    let items_spin = SpinButton::new(Some(&adjustment), 1.0, 0);
    items_spin.set_hexpand(true);
    input_grid.attach(&items_spin, 1, 0, 1, 1);

    main_container.append(&input_grid);

    // Preview section
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    preview_label.set_margin_top(12);
    main_container.append(&preview_label);

    // Preview text with scrolled window
    let preview_text = TextView::new();
    preview_text.set_editable(false);
    preview_text.set_cursor_visible(false);

    let preview_scroll = ScrolledWindow::new();
    preview_scroll.set_child(Some(&preview_text));
    preview_scroll.set_size_request(350, 120);
    preview_scroll.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    preview_scroll.set_margin_top(8);

    main_container.append(&preview_scroll);
    content_area.append(&main_container);

    // Update preview when spin button value changes
    let preview_buffer = preview_text.buffer();
    let update_preview = {
        let items_spin = items_spin.clone();
        let preview_buffer = preview_buffer.clone();
        move || {
            let count = items_spin.value() as usize;
            let mut preview = String::new();
            for i in 0..count.min(8) {
                // Show max 8 in preview
                if i > 0 {
                    preview.push('\n');
                }
                preview.push_str(&format!(
                    "Term {}\n: Definition of term {}.\n",
                    i + 1,
                    i + 1
                ));
            }
            if count > 8 {
                preview.push_str(&format!("\n... and {} more definition pairs", count - 8));
            }
            preview_buffer.set_text(&preview);
        }
    };

    // Initial preview update
    update_preview();

    // Connect value changed signal
    items_spin.connect_value_changed({
        let update_preview = update_preview.clone();
        move |_| {
            update_preview();
        }
    });

    // Set focus to spin button
    items_spin.grab_focus();

    dialog.set_default_response(ResponseType::Accept);
    dialog.show();

    let editor_clone = editor.clone();
    let items_spin_clone = std::rc::Rc::new(items_spin);

    dialog.connect_response(move |dialog, resp| {
        if resp == ResponseType::Accept {
            let count = items_spin_clone.value() as usize;
            if count > 0 && count <= 20 {
                // Valid input - create definition list and close dialog
                editor_clone.insert_custom_definition_list(count);
                dialog.close();
                return;
            }

            // Invalid input - add error styling and don't close dialog
            items_spin_clone.add_css_class("error");
        } else {
            // Cancel button - close dialog
            dialog.close();
        }
    });
}
