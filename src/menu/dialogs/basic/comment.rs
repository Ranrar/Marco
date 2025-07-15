// Comment dialog - Insert HTML comment text
// Simple dialog with multi-line text input and live preview

use crate::menu::dialogs::common::*;
use crate::{editor, language};

/// Show dialog to insert a comment
pub fn show_comment_dialog(window: &gtk4::Window, editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&language::tr("advanced.comment")),
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
    let title_label = Label::new(Some("Insert HTML comment text"));
    title_label.set_halign(gtk4::Align::Start);
    main_container.append(&title_label);

    // Create grid for input fields
    let input_grid = Grid::new();
    input_grid.set_row_spacing(8);
    input_grid.set_column_spacing(12);
    input_grid.set_margin_top(12);

    // Comment text input
    let text_label = Label::new(Some("Comment text:"));
    text_label.set_halign(gtk4::Align::End);
    input_grid.attach(&text_label, 0, 0, 1, 1);

    // Use TextView for multi-line comment
    let text_view = gtk4::TextView::new();
    text_view.set_size_request(350, 80);
    let text_buffer = text_view.buffer();
    text_buffer.set_text("Your comment here...");

    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_child(Some(&text_view));
    scroll.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    scroll.set_hexpand(true);

    input_grid.attach(&scroll, 1, 0, 1, 1);

    main_container.append(&input_grid);

    // Preview section
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    preview_label.set_margin_top(12);
    main_container.append(&preview_label);

    let preview_text = preview::create_preview_text_view();
    preview_text.set_size_request(350, 50);
    preview_text.set_margin_top(8);

    main_container.append(&preview_text);
    content_area.append(&main_container);

    let update_preview = {
        let text_buffer = text_buffer.clone();
        let preview_buffer = preview_text.buffer();
        move || {
            let text = text_buffer.text(&text_buffer.start_iter(), &text_buffer.end_iter(), false);
            let preview = format!("<!-- {} -->", text);
            preview_buffer.set_text(&preview);
        }
    };

    update_preview();
    text_buffer.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });

    // Set focus to text view
    text_view.grab_focus();

    dialog.set_default_response(ResponseType::Accept);
    dialog.show();

    let editor_clone = editor.clone();
    let text_buffer_clone = text_buffer.clone();

    dialog.connect_response(move |dialog, resp| {
        if resp == ResponseType::Accept {
            let text = text_buffer_clone.text(
                &text_buffer_clone.start_iter(),
                &text_buffer_clone.end_iter(),
                false,
            );

            if !text.trim().is_empty() {
                // Valid input - insert comment and close dialog
                editor_clone.insert_comment(&text);
                dialog.close();
            }
            // Allow empty comments to be cancelled by not showing error
        } else {
            // Cancel button - close dialog
            dialog.close();
        }
    });
}
