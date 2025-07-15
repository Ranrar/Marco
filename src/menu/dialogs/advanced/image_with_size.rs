// Image with size dialog - Insert image with custom width and height
// Simple dialog with URL, alt text, and dimension inputs plus preview

use crate::menu::dialogs::common::{self, *};
use crate::{editor, language};

/// Show dialog to insert image with size
pub fn show_image_with_size_dialog(window: &gtk4::Window, editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&language::tr("advanced.image_with_size")),
        Some(window),
        gtk4::DialogFlags::MODAL,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Insert", ResponseType::Accept),
        ],
    );

    dialog.set_default_size(490, 250);

    let grid = Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(12);
    grid.set_margin_top(20);
    grid.set_margin_bottom(20);
    grid.set_margin_start(20);
    grid.set_margin_end(20);

    // URL input
    let url_label = Label::new(Some("Image URL:"));
    url_label.set_halign(gtk4::Align::Start);
    let url_entry = Entry::new();
    url_entry.set_placeholder_text(Some("https://example.com/image.jpg"));
    url_entry.set_hexpand(true);

    // File picker button
    let file_picker_button = common::create_file_picker_button_for_dialog(
        &dialog,
        "Browse...",
        "Select Image File",
        Some(vec![
            ("Image files".to_string(), "*.jpg".to_string()),
            ("Image files".to_string(), "*.jpeg".to_string()),
            ("Image files".to_string(), "*.png".to_string()),
            ("Image files".to_string(), "*.gif".to_string()),
            ("Image files".to_string(), "*.bmp".to_string()),
            ("Image files".to_string(), "*.webp".to_string()),
            ("All files".to_string(), "*".to_string()),
        ]),
        &url_entry,
    );

    // Alt text input
    let alt_label = Label::new(Some("Alt text:"));
    alt_label.set_halign(gtk4::Align::Start);
    let alt_entry = Entry::new();
    alt_entry.set_placeholder_text(Some("Description of the image"));

    // Width input
    let width_label = Label::new(Some("Width (px):"));
    width_label.set_halign(gtk4::Align::Start);
    let width_adjustment = Adjustment::new(300.0, 1.0, 2000.0, 1.0, 10.0, 0.0);
    let width_spin = SpinButton::new(Some(&width_adjustment), 1.0, 0);

    // Height input
    let height_label = Label::new(Some("Height (px):"));
    height_label.set_halign(gtk4::Align::Start);
    let height_adjustment = Adjustment::new(200.0, 1.0, 2000.0, 1.0, 10.0, 0.0);
    let height_spin = SpinButton::new(Some(&height_adjustment), 1.0, 0);

    // Preview
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    let preview_text = TextView::new();
    preview_text.set_editable(false);
    preview_text.set_size_request(400, 60);

    let update_preview = {
        let url_entry = url_entry.clone();
        let alt_entry = alt_entry.clone();
        let width_spin = width_spin.clone();
        let height_spin = height_spin.clone();
        let preview_buffer = preview_text.buffer();
        move || {
            let url = url_entry.text();
            let alt = alt_entry.text();
            let width = width_spin.value() as i32;
            let height = height_spin.value() as i32;
            let preview = format!(
                "<img src=\"{}\" alt=\"{}\" width=\"{}\" height=\"{}\">",
                if url.is_empty() { "image-url" } else { &url },
                if alt.is_empty() { "alt-text" } else { &alt },
                width,
                height
            );
            preview_buffer.set_text(&preview);
        }
    };

    // Initial preview and connect signals
    update_preview();
    url_entry.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });
    alt_entry.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });
    width_spin.connect_value_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });
    height_spin.connect_value_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });

    grid.attach(&url_label, 0, 0, 1, 1);
    grid.attach(&url_entry, 1, 0, 1, 1);
    grid.attach(&file_picker_button, 2, 0, 1, 1);
    grid.attach(&alt_label, 0, 1, 1, 1);
    grid.attach(&alt_entry, 1, 1, 2, 1);
    grid.attach(&width_label, 0, 2, 1, 1);
    grid.attach(&width_spin, 1, 2, 2, 1);
    grid.attach(&height_label, 0, 3, 1, 1);
    grid.attach(&height_spin, 1, 3, 2, 1);
    grid.attach(&preview_label, 0, 4, 3, 1);
    grid.attach(&preview_text, 0, 5, 3, 1);

    dialog.content_area().append(&grid);
    url_entry.grab_focus();

    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let url = url_entry.text();
            let alt = alt_entry.text();
            let width = width_spin.value() as i32;
            let height = height_spin.value() as i32;
            if !url.is_empty() {
                editor_clone.insert_image_with_size(
                    &url,
                    &alt,
                    Some(&width.to_string()),
                    Some(&height.to_string()),
                );
            }
        }
        dialog.close();
    });

    dialog.present();
}
