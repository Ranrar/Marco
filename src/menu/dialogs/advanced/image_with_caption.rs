// Image with caption dialog - Insert image with caption using HTML figure element
// Creates proper HTML structure: <figure><img src="..." alt="..."><figcaption>...</figcaption></figure>

use crate::menu::dialogs::common::{self, *};
use crate::{editor, language};

/// Show dialog to insert image with caption
pub fn show_image_with_caption_dialog(window: &gtk4::Window, editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&language::tr("advanced.image_with_caption")),
        Some(window),
        gtk4::DialogFlags::MODAL,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Insert", ResponseType::Accept),
        ],
    );

    dialog.set_default_size(520, 300);

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
    url_entry.set_placeholder_text(Some("https://example.com/image.jpg or image.png"));
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
    alt_entry.set_placeholder_text(Some("Description of the image for accessibility"));

    // Caption input
    let caption_label = Label::new(Some("Caption:"));
    caption_label.set_halign(gtk4::Align::Start);
    let caption_entry = Entry::new();
    caption_entry.set_placeholder_text(Some("Caption text that appears below the image"));

    // Help text
    let help_label = Label::new(Some(
        "Creates a figure with caption using HTML structure for better semantic markup.",
    ));
    help_label.set_halign(gtk4::Align::Start);
    help_label.set_wrap(true);
    help_label.set_max_width_chars(50);
    help_label.add_css_class("dim-label");

    // Preview
    let preview_label = Label::new(Some("HTML Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    let preview_text = TextView::new();
    preview_text.set_editable(false);
    preview_text.set_size_request(420, 80);
    preview_text.set_wrap_mode(gtk4::WrapMode::Word);

    // Scrolled window for preview
    let preview_scroll = ScrolledWindow::new();
    preview_scroll.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    preview_scroll.set_child(Some(&preview_text));

    let update_preview = {
        let url_entry = url_entry.clone();
        let alt_entry = alt_entry.clone();
        let caption_entry = caption_entry.clone();
        let preview_buffer = preview_text.buffer();
        move || {
            let url = url_entry.text();
            let alt = alt_entry.text();
            let caption = caption_entry.text();

            let preview = format!(
                "<figure>\n    <img src=\"{}\" alt=\"{}\">\n    <figcaption>{}</figcaption>\n</figure>",
                if url.is_empty() { "image-url" } else { &url },
                if alt.is_empty() { "alt-text" } else { &alt },
                if caption.is_empty() { "Image caption" } else { &caption }
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
    caption_entry.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });

    // Layout grid
    grid.attach(&url_label, 0, 0, 1, 1);
    grid.attach(&url_entry, 1, 0, 1, 1);
    grid.attach(&file_picker_button, 2, 0, 1, 1);
    grid.attach(&alt_label, 0, 1, 1, 1);
    grid.attach(&alt_entry, 1, 1, 2, 1);
    grid.attach(&caption_label, 0, 2, 1, 1);
    grid.attach(&caption_entry, 1, 2, 2, 1);
    grid.attach(&help_label, 0, 3, 3, 1);
    grid.attach(&preview_label, 0, 4, 3, 1);
    grid.attach(&preview_scroll, 0, 5, 3, 1);

    dialog.content_area().append(&grid);
    url_entry.grab_focus();

    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let url = url_entry.text();
            let alt = alt_entry.text();
            let caption = caption_entry.text();
            if !url.is_empty() {
                editor_clone.insert_image_with_caption(&url, &alt, &caption);
            }
        }
        dialog.close();
    });

    dialog.present();
}
