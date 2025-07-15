// Colored text dialog - Insert colored text using HTML span tags
// Simple dialog with text and color inputs plus live preview

use crate::menu::dialogs::common::*;
use crate::{editor, language};

/// Show dialog to insert colored text
pub fn show_colored_text_dialog(window: &gtk4::Window, editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&language::tr("advanced.colored_text")),
        Some(window),
        gtk4::DialogFlags::MODAL,
        &[
            (&language::tr("table_dialog.insert"), ResponseType::Accept),
            (&language::tr("table_dialog.cancel"), ResponseType::Cancel),
        ],
    );
    let content_area = dialog.content_area();

    // Create main container
    let main_container = create_content_box(Orientation::Vertical, 12);
    main_container.set_margin_top(12);
    main_container.set_margin_bottom(12);
    main_container.set_margin_start(12);
    main_container.set_margin_end(12);

    // Add title label
    let title_label = Label::new(Some("Insert colored text using HTML span tags"));
    title_label.set_halign(gtk4::Align::Start);
    main_container.append(&title_label);

    // Create grid for input fields
    let input_grid = Grid::new();
    input_grid.set_row_spacing(8);
    input_grid.set_column_spacing(12);
    input_grid.set_margin_top(12);

    // Text input
    let text_entry =
        builders::create_labeled_entry(&input_grid, 0, "Text to color:", Some("Enter text here"));

    // Pre-fill with selected text if available
    if let Some(selected_text) = editor.get_selected_text() {
        text_entry.set_text(&selected_text);
    }

    // Color input
    let color_entry = builders::create_labeled_entry(
        &input_grid,
        1,
        "Color (hex or name):",
        Some("#ff0000 or red"),
    );
    color_entry.set_text("#ff0000");

    // Add input validation
    text_entry.connect_changed({
        let text_entry = text_entry.clone();
        let color_entry = color_entry.clone();
        move |_| {
            validation::remove_error_style(&text_entry);
            validation::remove_error_style(&color_entry);
        }
    });

    color_entry.connect_changed({
        let color_entry = color_entry.clone();
        let text_entry = text_entry.clone();
        move |_| {
            validation::remove_error_style(&color_entry);
            validation::remove_error_style(&text_entry);
        }
    });

    main_container.append(&input_grid);

    // Preview section using unified theme
    let (preview_box, preview_text) = crate::menu::dialogs::common::create_preview_area("Preview");
    main_container.append(&preview_box);
    content_area.append(&main_container);

    // Update preview function
    let update_preview = {
        let text_entry = text_entry.clone();
        let color_entry = color_entry.clone();
        let preview_buffer = preview_text.buffer();
        move || {
            let text = text_entry.text();
            let color = color_entry.text();
            let preview = format!(
                "<span style=\"color: {}\">{}</span>",
                color,
                if text.is_empty() {
                    "Sample text"
                } else {
                    &text
                }
            );
            preview_buffer.set_text(&preview);
        }
    };

    // Initial preview
    update_preview();

    // Connect change signals
    text_entry.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });
    color_entry.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });

    // Set focus to text entry
    text_entry.grab_focus();

    dialog.set_default_response(ResponseType::Accept);
    dialog.show();

    let editor_clone = editor.clone();
    let text_entry_clone = std::rc::Rc::new(text_entry);
    let color_entry_clone = std::rc::Rc::new(color_entry);

    dialog.connect_response(move |dialog, resp| {
        if resp == ResponseType::Accept {
            let text = text_entry_clone.text();
            let color = color_entry_clone.text();

            // Validate input using common validation
            let text_valid = validation::validate_not_empty(&text);
            let color_valid = validation::validate_html_color(&color);

            if validation::validate_form_fields(&[
                (text_valid, &*text_entry_clone),
                (color_valid, &*color_entry_clone),
            ]) {
                // Valid input - insert colored text and close dialog
                editor_clone.insert_colored_text(&text, &color);
                dialog.close();
            }
        } else {
            // Cancel button - close dialog
            dialog.close();
        }
    });
}
