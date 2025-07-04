// Link open new dialog - Create a link that opens in a new tab/window
// Dialog with URL and link text inputs, uses HTML for new tab functionality

use gtk4::prelude::*;
use crate::menu::dialogs::common::*;
use crate::editor;

/// Show dialog to create a link that opens in a new tab/window
pub fn show_link_open_new_dialog(window: &gtk4::Window, editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some("Insert Link (Open in New Tab)"),
        Some(window),
        gtk4::DialogFlags::MODAL | gtk4::DialogFlags::DESTROY_WITH_PARENT,
        &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Ok)],
    );
    
    dialog.set_default_size(500, 300);
    
    let content_area = dialog.content_area();
    let main_box = gtk4::Box::new(Orientation::Vertical, 12);
    main_box.set_margin_top(16);
    main_box.set_margin_bottom(16);
    main_box.set_margin_start(16);
    main_box.set_margin_end(16);
    
    // Information section explaining HTML usage
    let info_box = gtk4::Box::new(Orientation::Vertical, 8);
    
    let info_label = Label::new(Some("About Links That Open in New Tab"));
    info_label.set_markup("<b>Info:</b>");
    info_label.set_halign(gtk4::Align::Start);
    info_box.append(&info_label);
    
    let explanation = Label::new(None);
    explanation.set_halign(gtk4::Align::Start);
    explanation.set_wrap(true);
    explanation.set_markup(
        "<i>Standard Markdown doesn't support opening links in new tabs.\n\
         This feature uses HTML. The link will work in HTML preview and export.</i>"
    );
    info_box.append(&explanation);
    
    // Separator
    let separator = gtk4::Separator::new(Orientation::Horizontal);
    separator.set_margin_top(8);
    separator.set_margin_bottom(8);
    info_box.append(&separator);
    
    main_box.append(&info_box);
    
    // Input section
    let input_grid = Grid::new();
    input_grid.set_row_spacing(12);
    input_grid.set_column_spacing(12);
    
    // URL input
    let url_label = Label::new(Some("URL:"));
    url_label.set_halign(gtk4::Align::Start);
    let url_entry = Entry::new();
    url_entry.set_placeholder_text(Some("https://example.com"));
    url_entry.set_hexpand(true);
    
    // Link text input
    let text_label = Label::new(Some("Link Text:"));
    text_label.set_halign(gtk4::Align::Start);
    let text_entry = Entry::new();
    text_entry.set_placeholder_text(Some("Click here"));
    text_entry.set_hexpand(true);
    
    // Check if we have selected text to use as default
    let buffer = &editor.source_buffer;
    let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
    if gtk_buffer.has_selection() {
        if let Some((start, end)) = gtk_buffer.selection_bounds() {
            let selected_text = gtk_buffer.text(&start, &end, false);
            text_entry.set_text(&selected_text);
        }
    }
    
    input_grid.attach(&url_label, 0, 0, 1, 1);
    input_grid.attach(&url_entry, 1, 0, 1, 1);
    input_grid.attach(&text_label, 0, 1, 1, 1);
    input_grid.attach(&text_entry, 1, 1, 1, 1);
    
    main_box.append(&input_grid);
    
    // Preview section
    let preview_box = gtk4::Box::new(Orientation::Vertical, 8);
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    preview_label.set_markup("<b>Preview:</b>");
    
    let preview_text = Label::new(Some(""));
    preview_text.set_halign(gtk4::Align::Start);
    preview_text.set_wrap(true);
    preview_text.set_selectable(true);
    preview_text.add_css_class("code-block");
    
    preview_box.append(&preview_label);
    preview_box.append(&preview_text);
    main_box.append(&preview_box);
    
    content_area.append(&main_box);
    
    // Function to update preview
    let update_preview = {
        let preview_text = preview_text.clone();
        move |url: &str, text: &str| {
            if !url.is_empty() && !text.is_empty() {
                let html = format!(r#"&lt;a href="{}" target="_blank"&gt;{}&lt;/a&gt;"#, url, text);
                preview_text.set_markup(&html);
            } else {
                preview_text.set_text("Enter URL and text to see preview");
            }
        }
    };
    
    // Connect text change events for live preview
    url_entry.connect_changed({
        let text_entry = text_entry.clone();
        let update_preview = update_preview.clone();
        move |url_entry| {
            let url = url_entry.text();
            let text = text_entry.text();
            update_preview(&url, &text);
        }
    });
    
    text_entry.connect_changed({
        let url_entry = url_entry.clone();
        let update_preview = update_preview.clone();
        move |text_entry| {
            let url = url_entry.text();
            let text = text_entry.text();
            update_preview(&url, &text);
        }
    });
    
    // Initial preview update
    update_preview("https://example.com", "Click here");
    
    // Focus on URL entry
    url_entry.grab_focus();
    
    // Handle dialog response
    dialog.connect_response({
        let editor = editor.clone();
        move |dialog, response| {
            if response == ResponseType::Ok {
                let url = url_entry.text();
                let text = text_entry.text();
                
                if !url.is_empty() && !text.is_empty() {
                    editor.insert_link_with_target(&url, &text, "_blank");
                }
            }
            dialog.close();
        }
    });
    
    dialog.present();
}
