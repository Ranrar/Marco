use crate::components::editor::font_config::{EditorConfiguration, EditorDisplaySettings};
use crate::components::editor::scroll_sync::ScrollSynchronizer;
use core::logic::swanson::SettingsManager;
use log::debug;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

/// A unique identifier for editor instances
pub type EditorId = u64;

/// Editor settings change callback function
type EditorSettingsCallback = Box<dyn Fn(&EditorDisplaySettings)>;

/// Line numbers change callback function
type LineNumbersCallback = Box<dyn Fn(bool)>;

/// Editor settings manager for runtime updates
pub struct EditorManager {
    editor_callbacks: Rc<std::cell::RefCell<HashMap<EditorId, EditorSettingsCallback>>>,
    line_numbers_callbacks: Rc<std::cell::RefCell<HashMap<EditorId, LineNumbersCallback>>>,
    editor_config: EditorConfiguration,
    next_id: Rc<std::cell::RefCell<u64>>,
    scroll_synchronizer: Rc<ScrollSynchronizer>,
    /// Cache last applied settings to prevent redundant updates
    last_settings: Rc<std::cell::RefCell<Option<EditorDisplaySettings>>>,
    /// Flag to prevent infinite callback loops
    updating_settings: Rc<std::cell::RefCell<bool>>,
}

impl EditorManager {
    /// Create a new editor manager
    pub fn new(settings_manager: Arc<SettingsManager>) -> Result<Self, Box<dyn std::error::Error>> {
        let editor_config = EditorConfiguration::new(settings_manager)?;
        Ok(Self {
            editor_callbacks: Rc::new(std::cell::RefCell::new(HashMap::new())),
            line_numbers_callbacks: Rc::new(std::cell::RefCell::new(HashMap::new())),
            editor_config,
            next_id: Rc::new(std::cell::RefCell::new(0)),
            scroll_synchronizer: Rc::new(ScrollSynchronizer::new()),
            last_settings: Rc::new(std::cell::RefCell::new(None)),
            updating_settings: Rc::new(std::cell::RefCell::new(false)),
        })
    }

    /// Register an editor settings callback
    pub fn register_editor_callback<F>(&self, callback: F) -> EditorId
    where
        F: Fn(&EditorDisplaySettings) + 'static,
    {
        let mut next_id = self.next_id.borrow_mut();
        let id = *next_id;
        *next_id += 1;

        let mut callbacks = self.editor_callbacks.borrow_mut();
        callbacks.insert(id, Box::new(callback));

        debug!(
            "Registered editor callback {} for settings updates (total: {})",
            id,
            callbacks.len()
        );
        id
    }

    /// Update editor settings and notify all registered callbacks
    pub fn update_editor_settings(
        &mut self,
        editor_settings: &EditorDisplaySettings,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Prevent infinite callback loops
        if *self.updating_settings.borrow() {
            debug!("Skipping redundant settings update (already updating)");
            return Ok(());
        }

        // Check if settings actually changed to prevent redundant processing
        if let Some(ref last_settings) = *self.last_settings.borrow() {
            if last_settings == editor_settings {
                debug!("Skipping redundant settings update (settings unchanged)");
                return Ok(());
            }
        }

        debug!(
            "Updating editor settings: {} {}px line-height:{} wrap:{}",
            editor_settings.font_family,
            editor_settings.font_size,
            editor_settings.line_height,
            editor_settings.line_wrapping
        );

        // Set flag to prevent infinite loops during callback processing
        *self.updating_settings.borrow_mut() = true;

        // Save settings to storage
        self.editor_config.save_editor_settings(editor_settings)?;

        // Cache the settings to prevent redundant updates
        *self.last_settings.borrow_mut() = Some(editor_settings.clone());

        // Notify all editor callbacks
        let callbacks = self.editor_callbacks.borrow();
        debug!(
            "Notifying {} registered editors of settings update",
            callbacks.len()
        );
        for (editor_id, callback) in callbacks.iter() {
            debug!("Notifying editor {} of settings update", editor_id);
            callback(editor_settings);
        }

        // Clear the updating flag
        *self.updating_settings.borrow_mut() = false;

        Ok(())
    }

    /// Register a line numbers callback
    pub fn register_line_numbers_callback<F>(&self, callback: F) -> EditorId
    where
        F: Fn(bool) + 'static,
    {
        let mut next_id = self.next_id.borrow_mut();
        let id = *next_id;
        *next_id += 1;

        let mut callbacks = self.line_numbers_callbacks.borrow_mut();
        callbacks.insert(id, Box::new(callback));

        debug!(
            "Registered line numbers callback {} for updates (total: {})",
            id,
            callbacks.len()
        );
        id
    }

    /// Update line numbers setting and notify all registered callbacks
    pub fn update_line_numbers(&self, show_line_numbers: bool) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Updating line numbers setting: {}", show_line_numbers);

        // Notify all line numbers callbacks
        let callbacks = self.line_numbers_callbacks.borrow();
        for (editor_id, callback) in callbacks.iter() {
            debug!("Notifying editor {} of line numbers update", editor_id);
            callback(show_line_numbers);
        }

        Ok(())
    }

    /// Get current editor settings
    pub fn get_current_editor_settings(&self) -> EditorDisplaySettings {
        self.editor_config.get_current_editor_settings()
    }

    /// Get the scroll synchronizer
    pub fn get_scroll_synchronizer(&self) -> Rc<ScrollSynchronizer> {
        Rc::clone(&self.scroll_synchronizer)
    }

    /// Set scroll synchronization enabled/disabled
    pub fn set_scroll_sync_enabled(&self, enabled: bool) {
        self.scroll_synchronizer.set_enabled(enabled);
        debug!(
            "Scroll synchronization {}",
            if enabled { "enabled" } else { "disabled" }
        );
    }
}

// Global editor manager instance using thread-local storage
thread_local! {
    static EDITOR_MANAGER: std::cell::RefCell<Option<Rc<std::cell::RefCell<EditorManager>>>> = const { std::cell::RefCell::new(None) };
}

/// Initialize the global editor manager and apply startup settings
pub fn init_editor_manager(settings_manager: Arc<SettingsManager>) -> Result<(), Box<dyn std::error::Error>> {
    let manager = EditorManager::new(settings_manager.clone())?;

    // Log the startup editor settings for debugging
    let startup_settings = manager.get_current_editor_settings();
    debug!(
        "Initializing editor manager with startup settings: {} {}px line-height:{} wrap:{}",
        startup_settings.font_family,
        startup_settings.font_size,
        startup_settings.line_height,
        startup_settings.line_wrapping
    );

    // Apply initial sync_scrolling setting from settings file
    let initial_sync_scrolling = settings_manager
        .get_settings()
        .layout
        .as_ref()
        .and_then(|l| l.sync_scrolling)
        .unwrap_or(true); // Default to true if not set

    manager.set_scroll_sync_enabled(initial_sync_scrolling);
    debug!(
        "Applied initial sync scrolling setting: {}",
        initial_sync_scrolling
    );

    EDITOR_MANAGER.with(|em| {
        *em.borrow_mut() = Some(Rc::new(std::cell::RefCell::new(manager)));
    });
    debug!("Global editor manager initialized successfully");
    Ok(())
}

/// Shutdown and cleanup the global editor manager
/// This explicitly clears the thread-local storage to prevent memory retention
pub fn shutdown_editor_manager() {
    EDITOR_MANAGER.with(|em| {
        let mut em_borrow = em.borrow_mut();
        if em_borrow.is_some() {
            log::info!("Shutting down global editor manager");
            // Drop the manager instance explicitly
            *em_borrow = None;
            log::info!("Global editor manager cleaned up successfully");
        } else {
            log::info!("Editor manager was not initialized, no cleanup needed");
        }
    });
}

/// Apply startup editor settings to all registered editors
pub fn apply_startup_editor_settings() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(manager) = get_editor_manager() {
        let mgr = manager.borrow();
        let startup_settings = mgr.get_current_editor_settings();
        debug!(
            "Applying startup editor settings to all editors: {} {}px line-height:{} wrap:{}",
            startup_settings.font_family,
            startup_settings.font_size,
            startup_settings.line_height,
            startup_settings.line_wrapping
        );

        // Notify editor callbacks
        let callbacks = mgr.editor_callbacks.borrow();
        for (editor_id, callback) in callbacks.iter() {
            debug!("Applying startup settings to editor {}", editor_id);
            callback(&startup_settings);
        }

        Ok(())
    } else {
        Err(format!("Editor manager not initialized").into())
    }
}

/// Register an editor with the global editor manager using a callback
pub fn register_editor_callback_globally<F>(callback: F) -> Option<EditorId>
where
    F: Fn(&EditorDisplaySettings) + 'static,
{
    if let Some(manager) = get_editor_manager() {
        let mgr = manager.borrow();
        let id = mgr.register_editor_callback(callback);
        debug!("Globally registered editor callback with ID: {}", id);
        Some(id)
    } else {
        debug!("Cannot register editor callback: global manager not initialized");
        None
    }
}

/// Update editor settings globally and notify all callbacks
pub fn update_editor_settings_globally(
    editor_settings: &EditorDisplaySettings,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(manager) = get_editor_manager() {
        let mut mgr = manager.borrow_mut();
        mgr.update_editor_settings(editor_settings)
    } else {
        Err(format!("Editor manager not initialized").into())
    }
}

/// Get the global editor manager instance
fn get_editor_manager() -> Option<Rc<std::cell::RefCell<EditorManager>>> {
    EDITOR_MANAGER.with(|em| em.borrow().clone())
}

/// Get the global scroll synchronizer instance
pub fn get_global_scroll_synchronizer() -> Option<Rc<ScrollSynchronizer>> {
    if let Some(manager) = get_editor_manager() {
        let mgr = manager.borrow();
        Some(mgr.get_scroll_synchronizer())
    } else {
        debug!("Cannot get scroll synchronizer: global editor manager not initialized");
        None
    }
}

/// Register a line numbers callback globally
pub fn register_line_numbers_callback_globally<F>(callback: F) -> Option<EditorId>
where
    F: Fn(bool) + 'static,
{
    if let Some(manager) = get_editor_manager() {
        let mgr = manager.borrow();
        let id = mgr.register_line_numbers_callback(callback);
        debug!("Globally registered line numbers callback with ID: {}", id);
        Some(id)
    } else {
        debug!("Cannot register line numbers callback: global manager not initialized");
        None
    }
}

/// Update line numbers setting globally and notify all callbacks
pub fn update_line_numbers_globally(show_line_numbers: bool) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(manager) = get_editor_manager() {
        let mgr = manager.borrow();
        mgr.update_line_numbers(show_line_numbers)
    } else {
        Err(format!("Editor manager not initialized").into())
    }
}

/// Set scroll synchronization enabled/disabled globally
pub fn set_scroll_sync_enabled_globally(enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(manager) = get_editor_manager() {
        let mgr = manager.borrow();
        mgr.set_scroll_sync_enabled(enabled);
        Ok(())
    } else {
        Err(format!("Editor manager not initialized").into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn smoke_test_editor_manager_lifecycle() {
        // Create a temporary settings file for testing
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let settings_path = temp_dir.path().join("test_settings.ron");

        // Create a SettingsManager for testing
        let settings_manager = SettingsManager::initialize(settings_path)
            .expect("Failed to create test SettingsManager");

        // Ensure manager is not initialized initially
        assert!(
            get_editor_manager().is_none(),
            "Manager should not be initialized initially"
        );

        // Initialize manager
        init_editor_manager(settings_manager).expect("Failed to initialize editor manager");

        // Verify manager is initialized and accessible
        assert!(
            get_editor_manager().is_some(),
            "Manager should be initialized after init"
        );

        // Test that we can register a callback (simple test that doesn't require complex lifetime management)
        let _editor_id = register_editor_callback_globally(|_settings| {
            // Simple callback that doesn't capture any variables
        });
        assert!(
            _editor_id.is_some(),
            "Should be able to register callback when manager is initialized"
        );

        // Test cleanup - this is the main focus of issue #15
        shutdown_editor_manager();

        // Verify manager is cleaned up
        assert!(
            get_editor_manager().is_none(),
            "Manager should be None after shutdown"
        );

        // Verify operations fail gracefully after shutdown
        let result = register_editor_callback_globally(|_settings| {});
        assert!(
            result.is_none(),
            "Operations should fail gracefully after shutdown"
        );

        let result = update_editor_settings_globally(&EditorDisplaySettings::default());
        assert!(
            result.is_err(),
            "Update operations should fail after shutdown"
        );
    }

    #[test]
    fn smoke_test_settings_deduplication() {
        use std::sync::{Arc, Mutex};

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let settings_path = temp_dir.path().join("test_settings.ron");

        // Create a SettingsManager for testing
        let settings_manager = SettingsManager::initialize(settings_path)
            .expect("Failed to create test SettingsManager");

        // Initialize manager
        init_editor_manager(settings_manager).expect("Failed to initialize editor manager");

        // Register callback to track how many times it's called
        let callback_count = Arc::new(Mutex::new(0));
        let callback_count_clone = Arc::clone(&callback_count);

        let _editor_id = register_editor_callback_globally(move |_settings| {
            let mut count = callback_count_clone.lock().unwrap();
            *count += 1;
        })
        .expect("Should register callback");

        // Test deduplication - same settings should not trigger callbacks
        let settings = EditorDisplaySettings::default();

        // First update should trigger callback
        update_editor_settings_globally(&settings).expect("Should update settings");
        assert_eq!(
            *callback_count.lock().unwrap(),
            1,
            "First update should trigger callback"
        );

        // Identical update should be deduplicated
        update_editor_settings_globally(&settings).expect("Should update settings");
        assert_eq!(
            *callback_count.lock().unwrap(),
            1,
            "Identical update should be deduplicated"
        );

        // Different settings should trigger callback
        let mut different_settings = settings.clone();
        different_settings.font_size = 18;
        update_editor_settings_globally(&different_settings).expect("Should update settings");
        assert_eq!(
            *callback_count.lock().unwrap(),
            2,
            "Different settings should trigger callback"
        );

        // Another identical update should be deduplicated
        update_editor_settings_globally(&different_settings).expect("Should update settings");
        assert_eq!(
            *callback_count.lock().unwrap(),
            2,
            "Second identical update should be deduplicated"
        );

        // Cleanup
        shutdown_editor_manager();
    }
}
