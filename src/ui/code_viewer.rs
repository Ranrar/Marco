/// this is the main html code viever
use gtk4::prelude::*;
use gtk4::{ScrolledWindow, TextView, Widget};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct RenderHtmlCode {
    widget: ScrolledWindow,
    text_view: TextView,
    current_content: Rc<RefCell<String>>,
}

impl RenderHtmlCode {
    pub fn new() -> Self {
        let text_view = TextView::new();
        text_view.set_editable(false);
        text_view.set_cursor_visible(false);
        text_view.set_monospace(true);

        // Set up text view styling
        text_view.set_left_margin(15);
        text_view.set_right_margin(15);
        text_view.set_top_margin(15);
        text_view.set_bottom_margin(15);

        let scrolled = ScrolledWindow::new();
        scrolled.set_child(Some(&text_view));
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);

        RenderHtmlCode {
            widget: scrolled,
            text_view,
            current_content: Rc::new(RefCell::new(String::new())),
        }
    }

    pub fn widget(&self) -> &Widget {
        self.widget.upcast_ref()
    }

    pub fn set_content(&self, content: &str) {
        let buffer = self.text_view.buffer();
        buffer.set_text(content);
        *self.current_content.borrow_mut() = content.to_string();
    }

    pub fn get_current_content(&self) -> String {
        self.current_content.borrow().clone()
    }

    pub fn grab_focus(&self) {
        self.text_view.grab_focus();
    }
}

impl Default for RenderHtmlCode {
    fn default() -> Self {
        Self::new()
    }
}
