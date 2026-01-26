//! Trailing-edge debouncing for GTK applications
//!
//! This module provides debouncing functionality optimized for expensive operations
//! like parsing, rendering, and syntax highlighting that should not run on every
//! keystroke.
//!
//! The debouncer implements trailing-edge debouncing: the function executes only
//! after calls stop for the specified timeout. Repeated calls reset the timer.

use crate::logic::signal_manager::safe_source_remove;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// A trailing-edge debouncer for GTK applications
///
/// Ideal for expensive operations (e.g. parsing, rendering, syntax highlighting)
/// where immediate execution can freeze the UI during rapid input.
///
/// # Behavior
///
/// - The function executes only after calls stop for the timeout duration
/// - Repeated calls reset the timer
/// - No leading-edge execution (unlike traditional debouncing)
pub struct Debouncer {
    /// The timeout duration for debouncing
    timeout: Duration,
    /// Handle to the currently scheduled timeout (if any)
    timeout_handle: Rc<RefCell<Option<glib::SourceId>>>,
    /// Track when the last call occurred
    last_call_time: Rc<RefCell<Option<Instant>>>,
    /// Whether we're currently in a debounce window
    is_debouncing: Rc<RefCell<bool>>,
}

impl Debouncer {
    /// Create a new debouncer with the specified timeout in milliseconds
    pub fn new(timeout_ms: u64) -> Self {
        Self {
            timeout: Duration::from_millis(timeout_ms),
            timeout_handle: Rc::new(RefCell::new(None)),
            last_call_time: Rc::new(RefCell::new(None)),
            is_debouncing: Rc::new(RefCell::new(false)),
        }
    }

    /// Execute the function with trailing-edge debouncing
    ///
    /// The function executes only after calls stop for the configured timeout.
    /// Repeated calls reset the timer, ensuring the function runs once after
    /// the user stops interacting.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let debouncer = Debouncer::new(300); // 300ms delay
    /// text_buffer.connect_changed(move |_| {
    ///     debouncer.debounce_trailing(|| {
    ///         // Expensive operation: parse and render
    ///     });
    /// });
    /// ```
    pub fn debounce_trailing<F>(&self, func: F)
    where
        F: Fn() + 'static,
    {
        let now = Instant::now();
        let mut last_call = self.last_call_time.borrow_mut();
        let mut is_debouncing = self.is_debouncing.borrow_mut();
        let mut timeout_handle = self.timeout_handle.borrow_mut();

        *last_call = Some(now);
        *is_debouncing = true;

        // Cancel any existing timeout
        if let Some(handle) = timeout_handle.take() {
            safe_source_remove(handle);
        }

        let timeout_handle_clone = Rc::clone(&self.timeout_handle);
        let is_debouncing_clone = Rc::clone(&self.is_debouncing);
        let timeout_duration = self.timeout;

        let source_id = glib::timeout_add_local(timeout_duration, move || {
            func();

            *is_debouncing_clone.borrow_mut() = false;
            *timeout_handle_clone.borrow_mut() = None;

            glib::ControlFlow::Break
        });

        *timeout_handle = Some(source_id);
    }
}
