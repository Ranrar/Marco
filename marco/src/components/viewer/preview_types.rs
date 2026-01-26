use crate::components::viewer::layout_controller::SplitController;
use gtk4::{Overlay, Paned};
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

// Keep the original type but add overlay and split controller support
// WebView is now wrapped in Rc<RefCell<>> for shared ownership during reparenting
pub type EditorReturn = (
    Paned,                         // 0: Keep as Paned for backwards compatibility
    Rc<RefCell<webkit6::WebView>>, // 1: WebView wrapped for reparenting support
    Rc<RefCell<String>>,           // 2: Content string
    Box<dyn Fn()>,                 // 3: Refresh callback
    Box<dyn Fn(&str)>,             // 4: Theme update callback
    Box<dyn Fn(&str)>,             // 5: Content update callback
    sourceview5::Buffer,           // 6: Editor buffer
    sourceview5::View,             // 7: Editor view
    Rc<RefCell<bool>>,             // 8: Insert mode state
    Box<dyn Fn(ViewMode)>,         // 9: View mode switcher
    Overlay,                       // 10: Overlay widget
    SplitController,               // 11: Split position controller
);
