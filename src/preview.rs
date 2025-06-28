use gtk4::prelude::*;
use gtk4::{ScrolledWindow, TextView, Widget};

#[derive(Clone)]
pub struct MarkdownPreview {
    widget: ScrolledWindow,
    text_view: TextView,
}

impl MarkdownPreview {
    pub fn new() -> Self {
        let text_view = TextView::new();
        text_view.set_editable(false);
        text_view.set_cursor_visible(false);

        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_child(Some(&text_view));
        scrolled_window.set_vexpand(true);
        scrolled_window.set_size_request(200, -1); // Minimum width of 200px

        Self {
            widget: scrolled_window,
            text_view,
        }
    }

    pub fn widget(&self) -> &Widget {
        self.widget.upcast_ref()
    }

    pub fn update_content(&self, markdown_text: &str) {
        // Use your markdown_basic parser for HTML preview
        let html = crate::syntax_basic::MarkdownParser::new().to_html(markdown_text);
        let preview_buffer = self.text_view.buffer();
        preview_buffer.set_text(&html);
    }

    #[allow(dead_code)]
    pub fn get_text_view(&self) -> &TextView {
        &self.text_view
    }
}