// Center text dialog - Insert centered text using HTML div tags
// Simple dialog with text input and live preview

use gtk4::prelude::*;
use crate::menu::dialogs::common::*;
use crate::{editor, language};

/// Show dialog to center text
pub fn show_center_text_dialog(window: &gtk4::Window, editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&language::tr("advanced.center_text")),
        Some(window),
        gtk4::DialogFlags::MODAL,
        &[(&language::tr("table_dialog.insert"), ResponseType::Accept), 
          (&language::tr("table_dialog.cancel"), ResponseType::Cancel)],
    );
    let content_area = dialog.content_area();
    
    // Create main container
    let main_container = create_content_box(Orientation::Vertical, 12);
    main_container.set_margin_top(12);
    main_container.set_margin_bottom(12);
    main_container.set_margin_start(12);
    main_container.set_margin_end(12);

    // Add title label
    let title_label = Label::new(Some("Insert centered text using HTML div tags"));
    title_label.set_halign(gtk4::Align::Start);
    main_container.append(&title_label);

    // Create grid for input fields
    let input_grid = Grid::new();
    input_grid.set_row_spacing(8);
    input_grid.set_column_spacing(12);
    input_grid.set_margin_top(12);

    // Text input
    let text_entry = builders::create_labeled_entry(
        &input_grid,
        0,
        "Text to center:",
        Some("Enter text here"),
    );
    
    // Pre-fill with selected text if available
    if let Some(selected_text) = editor.get_selected_text() {
        text_entry.set_text(&selected_text);
    }
    
    // Add input validation
    text_entry.connect_changed({
        let text_entry = text_entry.clone();
        move |_| {
            validation::remove_error_style(&text_entry);
        }
    });

    main_container.append(&input_grid);

    // Preview section
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    preview_label.set_margin_top(12);
    main_container.append(&preview_label);
    
    let preview_text = preview::create_preview_text_view();
    preview_text.set_size_request(300, 50);
    preview_text.set_margin_top(8);
    
    main_container.append(&preview_text);
    content_area.append(&main_container);
    
    let update_preview = {
        let text_entry = text_entry.clone();
        let preview_buffer = preview_text.buffer();
        move || {
            let text = text_entry.text();
            let preview = format!("<div align=\"center\">{}</div>", 
                                if text.is_empty() { "Sample text" } else { &text });
            preview_buffer.set_text(&preview);
        }
    };
    
    update_preview();
    text_entry.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });
    
    // Set focus to text entry
    text_entry.grab_focus();
    
    dialog.set_default_response(ResponseType::Accept);
    dialog.show();

    let editor_clone = editor.clone();
    let text_entry_clone = std::rc::Rc::new(text_entry);
    
    dialog.connect_response(move |dialog, resp| {
        if resp == ResponseType::Accept {
            let text = text_entry_clone.text();
            
            if validation::validate_form_fields(&[
                (validation::validate_not_empty(&text), &*text_entry_clone),
            ]) {
                // Valid input - insert centered text and close dialog
                editor_clone.insert_center_text(&text);
                dialog.close();
            }
        } else {
            // Cancel button - close dialog
            dialog.close();
        }
    });
}
