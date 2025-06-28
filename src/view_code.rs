use gtk4::prelude::*;
use gtk4::{ScrolledWindow, TextView, Widget};
use pulldown_cmark::{Parser, Options, html};

#[derive(Clone)]
pub struct MarkdownCodeView {
    widget: ScrolledWindow,
    text_view: TextView,
}

impl MarkdownCodeView {
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
        // Use pulldown-cmark for HTML code preview to match the HTML view
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        
        let parser = Parser::new_ext(markdown_text, options);
        let mut html_content = String::new();
        html::push_html(&mut html_content, parser);
        
        let preview_buffer = self.text_view.buffer();
        preview_buffer.set_text(&html_content);
    }

    #[allow(dead_code)]
    pub fn get_text_view(&self) -> &TextView {
        &self.text_view
    }
}