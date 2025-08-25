use gtk4::Paned;
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

pub type EditorReturn = (
    Paned,
    webkit6::WebView,
    Rc<RefCell<String>>,
    Box<dyn Fn()>,
    Box<dyn Fn(&str)>,
    Box<dyn Fn(&str)>,
    sourceview5::Buffer,
    Rc<RefCell<bool>>,
    Box<dyn Fn(ViewMode)>,
);
