// Preview components for dialog forms
// Common preview patterns and live update functionality

use super::*;
use gtk4::TextView;

/// Creates a preview text view with syntax highlighting
pub fn create_preview_text_view() -> TextView {
    let text_view = TextView::new();
    text_view.set_editable(false);
    text_view.set_cursor_visible(false);
    text_view.set_wrap_mode(gtk4::WrapMode::Word);
    text_view.set_vexpand(true);
    text_view.add_css_class("code-block");
    text_view
}

/// Creates a scrollable preview container
pub fn create_preview_scroll() -> gtk4::ScrolledWindow {
    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    scroll.set_vexpand(true);
    scroll.set_min_content_height(200);
    scroll
}

/// Updates preview text with proper formatting
pub fn update_preview_text(text_view: &TextView, content: &str) {
    let buffer = text_view.buffer();
    buffer.set_text(content);
}

/// Creates a preview section with label and text view
pub fn create_preview_section(title: &str) -> (gtk4::Box, TextView) {
    let preview_box = gtk4::Box::new(Orientation::Vertical, 8);
    
    let preview_label = Label::new(Some(title));
    preview_label.set_halign(gtk4::Align::Start);
    preview_label.add_css_class("heading");
    preview_box.append(&preview_label);
    
    let scroll = create_preview_scroll();
    let text_view = create_preview_text_view();
    scroll.set_child(Some(&text_view));
    preview_box.append(&scroll);
    
    (preview_box, text_view)
}

/// Creates a help label with caption styling
pub fn create_help_label(text: &str) -> Label {
    let help_label = Label::new(Some(text));
    help_label.set_halign(gtk4::Align::Start);
    help_label.set_wrap(true);
    help_label.add_css_class("caption");
    help_label
}
