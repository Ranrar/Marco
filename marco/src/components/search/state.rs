//! Search State Management
//!
//! Manages search state, options, and thread-local storage for the search component.

use glib::SourceId;
use gtk4::{Entry, Label, Window};
use sourceview5::{Buffer, SearchContext, View};
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(target_os = "linux")]
use webkit6::WebView;

#[cfg(target_os = "windows")]
type WebView = gtk4::Widget;

/// Search options for controlling search behavior
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub match_case: bool,
    pub match_whole_word: bool,
    pub match_markdown_only: bool, // Not yet implemented: requires integration with Marco's grammar parser
    pub use_regex: bool,
}

/// Current search state
#[derive(Debug)]
pub struct SearchState {
    pub search_context: SearchContext,
}

/// Simple async search manager for better UI responsiveness
#[derive(Default)]
pub struct AsyncSearchManager {
    pub current_timer_id: Option<SourceId>,
}

impl AsyncSearchManager {
    pub fn new() -> Self {
        Self::default()
    }
}

// Thread-local state storage
thread_local! {
    pub static CACHED_SEARCH_WINDOW: RefCell<Option<Rc<Window>>> = const { RefCell::new(None) };
    pub static CURRENT_BUFFER: RefCell<Option<Rc<Buffer>>> = const { RefCell::new(None) };
    pub static CURRENT_SOURCE_VIEW: RefCell<Option<Rc<View>>> = const { RefCell::new(None) };
    pub static CURRENT_WEBVIEW: RefCell<Option<Rc<RefCell<WebView>>>> = const { RefCell::new(None) };
    pub static CURRENT_SEARCH_STATE: RefCell<Option<SearchState>> = const { RefCell::new(None) };
    pub static CURRENT_MATCH_LABEL: RefCell<Option<Label>> = const { RefCell::new(None) };
    pub static CURRENT_SEARCH_ENTRY: RefCell<Option<Entry>> = const { RefCell::new(None) };
    pub static NAVIGATION_IN_PROGRESS: RefCell<bool> = const { RefCell::new(false) };
    pub static CURRENT_MATCH_POSITION: RefCell<Option<i32>> = const { RefCell::new(None) };
    pub static SEARCH_DEBOUNCE_TIMER: RefCell<Option<SourceId>> = const { RefCell::new(None) };
    pub static NAVIGATION_DEBOUNCE_TIMER: RefCell<Option<SourceId>> = const { RefCell::new(None) };
    pub static ASYNC_MANAGER: RefCell<Option<AsyncSearchManager>> = const { RefCell::new(None) };
}

/// Check if navigation is in progress
pub fn set_navigation_in_progress(in_progress: bool) {
    NAVIGATION_IN_PROGRESS.with(|flag| {
        *flag.borrow_mut() = in_progress;
    });
}

/// Clear search highlighting and state
pub fn clear_search_highlighting() {
    use log::trace;

    CURRENT_SEARCH_STATE.with(|state_ref| {
        // IMPORTANT: disable highlighting on the active SearchContext before dropping it.
        // Otherwise, SourceView can leave the old highlights applied in the buffer.
        if let Some(search_state) = state_ref.borrow().as_ref() {
            search_state.search_context.set_highlight(false);
        }
        *state_ref.borrow_mut() = None;
    });
    CURRENT_MATCH_POSITION.with(|pos| {
        *pos.borrow_mut() = None;
    });
    trace!("Search highlighting cleared");
}
