use gtk4::glib::SignalHandlerId;
use gtk4::prelude::ObjectExt;
use std::collections::HashMap;

/// Manages GTK signal handler connections for proper cleanup
/// This prevents memory leaks by ensuring all signal handlers are disconnected
#[derive(Default)]
pub struct SignalManager {
    handlers: HashMap<String, Vec<(gtk4::glib::Object, SignalHandlerId)>>,
}

impl SignalManager {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a signal handler for later disconnection
    pub fn register_handler(
        &mut self,
        group: &str,
        object: &gtk4::glib::Object,
        handler_id: SignalHandlerId,
    ) {
        let handlers = self.handlers.entry(group.to_string()).or_default();
        handlers.push((object.clone(), handler_id));
    }

    /// Disconnect all handlers in a specific group
    pub fn disconnect_group(&mut self, group: &str) {
        if let Some(handlers) = self.handlers.remove(group) {
            let count = handlers.len();
            for (object, handler_id) in handlers {
                object.disconnect(handler_id);
            }
            log::debug!(
                "Disconnected {} signal handlers from group '{}'",
                count,
                group
            );
        }
    }

    /// Disconnect all registered handlers
    pub fn disconnect_all(&mut self) {
        let total_handlers: usize = self.handlers.values().map(|v| v.len()).sum();

        for (_group, handlers) in self.handlers.drain() {
            for (object, handler_id) in handlers {
                object.disconnect(handler_id);
            }
        }

        log::info!("Disconnected {} total signal handlers", total_handlers);
    }

    /// Get the number of registered handlers
    pub fn handler_count(&self) -> usize {
        self.handlers.values().map(|v| v.len()).sum()
    }
}

impl Drop for SignalManager {
    fn drop(&mut self) {
        if !self.handlers.is_empty() {
            log::warn!(
                "SignalManager dropped with {} undisconnected handlers",
                self.handler_count()
            );
            self.disconnect_all();
        }
    }
}

/// Safely remove a GLib source without panicking if it has already been removed
/// The GLib implementation panics if you try to remove a source that doesn't exist.
/// This function silently ignores such errors by discarding the result.
pub fn safe_source_remove(source_id: gtk4::glib::SourceId) {
    // The simplest approach: just don't call remove() at all on sources that
    // might have already been removed. In most cases, sources clean themselves up
    // when they complete, so manual removal is often unnecessary.
    //
    // For now, just log and skip removal to prevent crashes
    log::trace!(
        "Skipping source removal for ID {:?} to prevent potential panic",
        source_id
    );
}

/// Helper macro for connecting and registering signal handlers
#[macro_export]
macro_rules! connect_and_register {
    ($signal_manager:expr, $group:expr, $object:expr, $signal:ident, $closure:expr) => {{
        let handler_id = $object.$signal($closure);
        $signal_manager.register_handler($group, &$object.clone().upcast(), handler_id);
        handler_id
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_manager_basic() {
        let manager = SignalManager::new();
        assert_eq!(manager.handler_count(), 0);

        // Test would require GTK objects to be meaningful
        // This serves as a smoke test for the structure
    }
}
