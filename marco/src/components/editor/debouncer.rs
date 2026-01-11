use crate::logic::signal_manager::safe_source_remove;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// A debouncer that implements leading + trailing edge debouncing for GTK applications.
///
/// - Leading edge: Fires immediately on the first call
/// - Trailing edge: Fires after the specified delay when calls stop
/// - Batching: Ignores calls that happen within the debounce window
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
    /// Create a new debouncer with the specified timeout
    pub fn new(timeout_ms: u64) -> Self {
        Self {
            timeout: Duration::from_millis(timeout_ms),
            timeout_handle: Rc::new(RefCell::new(None)),
            last_call_time: Rc::new(RefCell::new(None)),
            is_debouncing: Rc::new(RefCell::new(false)),
        }
    }

    /// Execute the function with debouncing logic
    ///
    /// - First call executes immediately (leading edge)
    /// - Subsequent calls within the timeout window are ignored
    /// - Final call executes after timeout expires (trailing edge)
    #[allow(dead_code)]
    pub fn debounce<F>(&self, func: F)
    where
        F: Fn() + 'static,
    {
        let now = Instant::now();
        let mut last_call = self.last_call_time.borrow_mut();
        let mut is_debouncing = self.is_debouncing.borrow_mut();
        let mut timeout_handle = self.timeout_handle.borrow_mut();

        // Update last call time
        *last_call = Some(now);

        // If this is the first call or we're not currently debouncing, execute immediately (leading edge)
        if !*is_debouncing {
            func();
            *is_debouncing = true;
        }

        // Cancel any existing timeout
        if let Some(handle) = timeout_handle.take() {
            safe_source_remove(handle);
        }

        // Set up trailing edge timeout
        let timeout_handle_clone = Rc::clone(&self.timeout_handle);
        let is_debouncing_clone = Rc::clone(&self.is_debouncing);
        let timeout_duration = self.timeout;

        let source_id = glib::timeout_add_local(timeout_duration, move || {
            // Execute the function (trailing edge)
            func();

            // Reset debouncing state
            *is_debouncing_clone.borrow_mut() = false;
            *timeout_handle_clone.borrow_mut() = None;

            // Return false to remove this timeout
            glib::ControlFlow::Break
        });

        // Store the timeout handle so we can cancel it if needed
        *timeout_handle = Some(source_id);
    }

    /// Execute the function with *trailing-edge only* debouncing.
    ///
    /// This is ideal for expensive work (e.g. parsing, rendering, syntax highlighting)
    /// where calling it on the leading edge can freeze the UI while the user is typing.
    ///
    /// Behaviour:
    /// - The function is executed only after calls stop for `timeout`.
    /// - Repeated calls reset the timer.
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

    /// Reset the debouncer state (useful for testing or manual control)
    #[allow(dead_code)]
    pub fn reset(&self) {
        let mut timeout_handle = self.timeout_handle.borrow_mut();
        let mut is_debouncing = self.is_debouncing.borrow_mut();
        let mut last_call = self.last_call_time.borrow_mut();

        // Cancel any pending timeout
        if let Some(handle) = timeout_handle.take() {
            safe_source_remove(handle);
        }

        *is_debouncing = false;
        *last_call = None;
    }
}
