use webkit6::prelude::WebViewExt;
use gtk4::prelude::*;
use gtk4::{ScrolledWindow, Widget};
use webkit6::{WebView};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct MarkdownHtmlView {
    widget: ScrolledWindow,
    webview: WebView,
    current_content: Rc<RefCell<String>>,
}

impl MarkdownHtmlView {
    pub fn new() -> Self {
        // Create webkit6 webview
        let webview = WebView::new();
        
        // Create scrolled window container
        let scrolled = ScrolledWindow::new();
        scrolled.set_child(Some(&webview));
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);

        MarkdownHtmlView {
            widget: scrolled,
            webview,
            current_content: Rc::new(RefCell::new(String::new())),
        }
    }

    pub fn widget(&self) -> &Widget {
        self.widget.upcast_ref()
    }

    pub fn set_content(&self, markdown: &str) {
        // Basic HTML structure for webkit6 display
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Preview</title>
    <style>
        body {{ font-family: sans-serif; padding: 20px; }}
        pre {{ background: #f5f5f5; padding: 10px; border-radius: 4px; }}
        code {{ background: #f0f0f0; padding: 2px 4px; border-radius: 2px; }}
    </style>
</head>
<body>
    <pre>{}</pre>
</body>
</html>"#,
            markdown
        );

        self.webview.load_html(&html, None);
        *self.current_content.borrow_mut() = markdown.to_string();
    }

    pub fn get_current_content(&self) -> String {
        self.current_content.borrow().clone()
    }
}