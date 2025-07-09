// YouTube video dialog - Insert YouTube video with thumbnail and link
// Simple dialog with video ID and title inputs plus preview

use crate::menu::dialogs::common::*;
use crate::{editor, language};
use gtk4::prelude::*;

/// Show dialog to insert YouTube video
pub fn show_youtube_video_dialog(window: &gtk4::Window, editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&language::tr("advanced.youtube_video")),
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
    let title_label = Label::new(Some("Insert YouTube video with thumbnail and link"));
    title_label.set_halign(gtk4::Align::Start);
    main_container.append(&title_label);

    // Create grid for input fields
    let input_grid = Grid::new();
    input_grid.set_row_spacing(8);
    input_grid.set_column_spacing(12);
    input_grid.set_margin_top(12);

    // Video ID input
    let id_label = Label::new(Some("YouTube Video ID:"));
    id_label.set_halign(gtk4::Align::End);
    input_grid.attach(&id_label, 0, 0, 1, 1);

    let id_entry = Entry::new();
    id_entry.set_placeholder_text(Some("dQw4w9WgXcQ (from URL)"));
    id_entry.set_hexpand(true);

    // Add input validation for video ID
    let title_entry_for_id = Entry::new();
    id_entry.connect_changed({
        let id_entry = id_entry.clone();
        let title_entry_for_id = title_entry_for_id.clone();
        move |_| {
            // Clear error state when user types
            id_entry.remove_css_class("error");
            title_entry_for_id.remove_css_class("error");
        }
    });

    input_grid.attach(&id_entry, 1, 0, 1, 1);

    // Title input
    let title_label = Label::new(Some("Video title (optional):"));
    title_label.set_halign(gtk4::Align::End);
    input_grid.attach(&title_label, 0, 1, 1, 1);

    let title_entry = title_entry_for_id.clone();
    title_entry.set_placeholder_text(Some("Video title"));

    input_grid.attach(&title_entry, 1, 1, 1, 1);

    main_container.append(&input_grid);

    // Help text
    let help_label = Label::new(Some("Extract the video ID from the YouTube URL (e.g., from\nhttps://www.youtube.com/watch?v=dQw4w9WgXcQ extract 'dQw4w9WgXcQ')"));
    help_label.set_halign(gtk4::Align::Start);
    help_label.add_css_class("caption");
    help_label.set_margin_top(8);
    main_container.append(&help_label);

    // Preview section
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    preview_label.set_margin_top(12);
    main_container.append(&preview_label);

    let preview_text = TextView::new();
    preview_text.set_editable(false);
    preview_text.set_cursor_visible(false);
    preview_text.set_size_request(400, 60);
    preview_text.set_margin_top(8);

    main_container.append(&preview_text);
    content_area.append(&main_container);

    let update_preview = {
        let id_entry = id_entry.clone();
        let title_entry = title_entry.clone();
        let preview_buffer = preview_text.buffer();
        move || {
            let id = id_entry.text();
            let title = title_entry.text();
            let preview = if !id.is_empty() {
                let display_title = if title.is_empty() {
                    "YouTube Video"
                } else {
                    &title
                };
                format!("[![{}](https://img.youtube.com/vi/{}/0.jpg)](https://www.youtube.com/watch?v={})", 
                       display_title, id, id)
            } else {
                "[![Video Title](https://img.youtube.com/vi/VIDEO_ID/0.jpg)](https://www.youtube.com/watch?v=VIDEO_ID)".to_string()
            };
            preview_buffer.set_text(&preview);
        }
    };

    update_preview();
    id_entry.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });
    title_entry.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });

    // Set focus to video ID entry
    id_entry.grab_focus();

    dialog.set_default_response(ResponseType::Accept);
    dialog.show();

    let editor_clone = editor.clone();
    let id_entry_clone = std::rc::Rc::new(id_entry);
    let title_entry_clone = std::rc::Rc::new(title_entry);

    dialog.connect_response(move |dialog, resp| {
        if resp == ResponseType::Accept {
            let id = id_entry_clone.text();
            let title = title_entry_clone.text();

            if !id.is_empty() {
                // Valid input - insert YouTube video and close dialog
                let display_title = if title.is_empty() {
                    "YouTube Video"
                } else {
                    &title
                };
                editor_clone.insert_youtube_video(&id, display_title);
                dialog.close();
                return;
            }

            // Invalid input - add error styling and don't close dialog
            id_entry_clone.add_css_class("error");
        } else {
            // Cancel button - close dialog
            dialog.close();
        }
    });
}
