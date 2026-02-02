//! Search State Management
//!
//! Manages search state, options, and thread-local storage for the search component.

use glib::SourceId;
use gtk4::{Label, Window};
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
pub struct AsyncSearchManager {
    pub current_timer_id: Option<SourceId>,
}

impl AsyncSearchManager {
    pub fn new() -> Self {
        Self {
            current_timer_id: None,
        }
    }

    /// Schedule a search operation with debouncing
    pub fn schedule_search<F>(&mut self, delay_ms: u32, callback: F)
    where
        F: Fn() + 'static,
    {
        use crate::logic::signal_manager::safe_source_remove;
        use glib::timeout_add_local;

        // Cancel any existing timer safely
        if let Some(timer_id) = self.current_timer_id.take() {
            safe_source_remove(timer_id);
        }

        // Schedule new search
        let timer_id = timeout_add_local(
            std::time::Duration::from_millis(delay_ms as u64),
            move || {
                callback();
                glib::ControlFlow::Break
            },
        );

        self.current_timer_id = Some(timer_id);
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
    pub static NAVIGATION_IN_PROGRESS: RefCell<bool> = const { RefCell::new(false) };
    pub static CURRENT_MATCH_POSITION: RefCell<Option<i32>> = const { RefCell::new(None) };
    pub static SEARCH_DEBOUNCE_TIMER: RefCell<Option<SourceId>> = const { RefCell::new(None) };
    pub static NAVIGATION_DEBOUNCE_TIMER: RefCell<Option<SourceId>> = const { RefCell::new(None) };
    pub static ASYNC_MANAGER: RefCell<Option<AsyncSearchManager>> = const { RefCell::new(None) };
}

/// Check if navigation is in progress
pub fn is_navigation_in_progress() -> bool {
    NAVIGATION_IN_PROGRESS.with(|flag| *flag.borrow())
}

/// Set navigation in progress flag
pub fn set_navigation_in_progress(in_progress: bool) {
    NAVIGATION_IN_PROGRESS.with(|flag| {
        *flag.borrow_mut() = in_progress;
    });
}

/// Clear search highlighting and state
pub fn clear_search_highlighting() {
    use log::trace;

    CURRENT_SEARCH_STATE.with(|state_ref| {
        *state_ref.borrow_mut() = None;
    });
    CURRENT_MATCH_POSITION.with(|pos| {
        *pos.borrow_mut() = None;
    });
    trace!("Search highlighting cleared");
}
