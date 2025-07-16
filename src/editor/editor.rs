use sourceview5::prelude::ViewExt;
use crate::viewer::{MarkdownCodeView, MarkdownHtmlView};
use gtk4::prelude::*;
use gtk4::{Paned, ScrolledWindow, Stack, Widget};
use sourceview5::{Buffer, View};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct MarkdownEditor {
    pub widget: Paned,
    pub source_view: View,
    pub view_stack: Stack,
    pub html_view: MarkdownHtmlView,
    pub code_view: MarkdownCodeView,
    pub source_buffer: Buffer,
    pub current_content: Rc<RefCell<String>>,
}

impl MarkdownEditor {
    pub fn new() -> Self {
        // Create SourceView components
        let source_buffer = Buffer::new(None);
        let source_view = View::with_buffer(&source_buffer);
        
        // Configure SourceView
        source_view.set_show_line_numbers(true);
        source_view.set_highlight_current_line(true);
        source_view.set_tab_width(4);
        source_view.set_insert_spaces_instead_of_tabs(true);
        source_view.set_auto_indent(true);

        // Create views
        let html_view = MarkdownHtmlView::new();
        let code_view = MarkdownCodeView::new();

        // Create view stack
        let view_stack = Stack::new();
        view_stack.set_vexpand(true);
        view_stack.add_named(html_view.widget(), Some("html"));
        view_stack.add_named(code_view.widget(), Some("code"));
        view_stack.set_visible_child_name("html");

        // Create scrolled window for source view
        let source_scroll = ScrolledWindow::new();
        source_scroll.set_child(Some(&source_view));
        source_scroll.set_vexpand(true);
        source_scroll.set_size_request(200, -1);

        // Create split pane
        let paned = Paned::new(gtk4::Orientation::Horizontal);
        paned.set_position(400);
        paned.set_resize_start_child(true);
        paned.set_resize_end_child(true);
        paned.set_shrink_start_child(false);
        paned.set_shrink_end_child(false);

        paned.set_start_child(Some(&source_scroll));
        paned.set_end_child(Some(&view_stack));

        MarkdownEditor {
            widget: paned,
            source_view,
            view_stack,
            html_view,
            code_view,
            source_buffer,
            current_content: Rc::new(RefCell::new(String::new())),
        }
    }

    pub fn widget(&self) -> &Widget {
        self.widget.upcast_ref()
    }

    pub fn set_content(&self, content: &str) {
        self.source_buffer.set_text(content);
        self.html_view.set_content(content);
        self.code_view.set_content(content);
        *self.current_content.borrow_mut() = content.to_string();
    }

    pub fn get_content(&self) -> String {
        let start = self.source_buffer.start_iter();
        let end = self.source_buffer.end_iter();
        self.source_buffer.text(&start, &end, false).to_string()
    }
}