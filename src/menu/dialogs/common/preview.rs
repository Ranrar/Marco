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
