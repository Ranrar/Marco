use glib::{source::SourceId, timeout_add_local, ControlFlow};
use gio::prelude::*;
use gio::Settings;
use std::time::Duration;
use std::cell::RefCell;
use std::rc::Rc;

/// General-purpose debouncer for delayed actions in GTK4, with optional dynamic timeout via gio::Settings
#[allow(dead_code)]
#[derive(Clone)]
pub struct Debouncer {
    timeout_ms: std::cell::Cell<u32>,
    source_id: Rc<RefCell<Option<SourceId>>>,
    callback: Rc<RefCell<Option<Box<dyn FnOnce()>>>>,
    settings: Option<Settings>,
    settings_key: Option<String>,
}

impl Debouncer {

    /// Set the debounce timeout in milliseconds (minimum 50ms)
    pub fn set_timeout_ms(&self, ms: u32) {
        let timeout = ms.max(50);
        self.timeout_ms.set(timeout);
    }

    /// Create a new Debouncer with a static timeout
    pub fn new(timeout_ms: u32) -> Self {
        Self {
            timeout_ms: std::cell::Cell::new(timeout_ms.max(50)),
            source_id: Rc::new(RefCell::new(None)),
            callback: Rc::new(RefCell::new(None)),
            settings: None,
            settings_key: None,
        }
    }

    /// Create a Debouncer that watches a gio::Settings key for timeout updates
    pub fn with_settings(timeout_ms: u32, settings: Settings, key: &str) -> Self {
        let debouncer = Self {
            timeout_ms: std::cell::Cell::new(timeout_ms.max(50)),
            source_id: Rc::new(RefCell::new(None)),
            callback: Rc::new(RefCell::new(None)),
            settings: Some(settings.clone()),
            settings_key: Some(key.to_string()),
        };
        // Connect to settings key changes
        let debouncer_clone = debouncer.clone();
        let key_for_closure = key.to_string();
        settings.connect_changed(Some(key), move |s, _| {
            let ms = s.int(&key_for_closure).max(50) as u32;
            debouncer_clone.set_timeout_ms(ms);
        });
        debouncer
    }

    /// Debounce the given callback. If called again before timeout, resets the timer.
    pub fn debounce<F: FnOnce() + 'static>(&self, callback: F) {
        // Remove any existing timer
        if let Some(id) = self.source_id.borrow_mut().take() {
            id.remove();
        }
        // Store the callback in a box
        *self.callback.borrow_mut() = Some(Box::new(callback));
        let source_id_cell = self.source_id.clone();
        let callback_cell = self.callback.clone();
        let timeout = self.timeout_ms.get();
        let source_id_cell_inner = source_id_cell.clone();
        *source_id_cell.borrow_mut() = Some(timeout_add_local(Duration::from_millis(timeout as u64), move || {
            // Take and call the callback if present
            if let Some(cb) = callback_cell.borrow_mut().take() {
                cb();
            }
            *source_id_cell_inner.borrow_mut() = None;
            ControlFlow::Break
        }));
    }
}
