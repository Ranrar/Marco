use gtk4::{Paned, Overlay};
use std::cell::RefCell;
use std::rc::Rc;

/// Runtime view mode for the preview pane
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewMode {
    HtmlPreview,
    CodePreview,
}

impl std::fmt::Display for ViewMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViewMode::HtmlPreview => write!(f, "HTML Preview"),
            ViewMode::CodePreview => write!(f, "Code Preview"),
        }
    }
}

// Keep the original type but add overlay support
pub type EditorReturn = (
    Paned,  // Keep as Paned for backwards compatibility
    webkit6::WebView,
    Rc<RefCell<String>>,
    Box<dyn Fn()>,
    Box<dyn Fn(&str)>,
    Box<dyn Fn(&str)>,
    sourceview5::Buffer,
    sourceview5::View,
    Rc<RefCell<bool>>,
    Box<dyn Fn(ViewMode)>,
    Overlay, // Add overlay as the 11th element
);
