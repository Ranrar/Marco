use crate::components::editor::font_config::{EditorConfiguration, EditorDisplaySettings};
use crate::components::editor::scroll_sync::ScrollSynchronizer;
use log::debug;
use std::collections::HashMap;
use std::rc::Rc;

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
}

impl EditorManager {
    /// Create a new editor manager
    pub fn new(settings_path: &str) -> anyhow::Result<Self> {
        let editor_config = EditorConfiguration::new(settings_path)?;
        Ok(Self {
            editor_callbacks: Rc::new(std::cell::RefCell::new(HashMap::new())),
            line_numbers_callbacks: Rc::new(std::cell::RefCell::new(HashMap::new())),
            editor_config,
            next_id: Rc::new(std::cell::RefCell::new(0)),
            scroll_synchronizer: Rc::new(ScrollSynchronizer::new()),
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

        debug!("Registered editor callback {} for settings updates", id);
        id
    }

    /// Update editor settings and notify all registered callbacks
    pub fn update_editor_settings(
        &mut self,
        editor_settings: &EditorDisplaySettings,
    ) -> anyhow::Result<()> {
        debug!(
            "Updating editor settings: {} {}px line-height:{} wrap:{}",
            editor_settings.font_family,
            editor_settings.font_size,
            editor_settings.line_height,
            editor_settings.line_wrapping
        );

        // Save settings to storage
        self.editor_config.save_editor_settings(editor_settings)?;

        // Notify all editor callbacks
        let callbacks = self.editor_callbacks.borrow();
        for (editor_id, callback) in callbacks.iter() {
            debug!("Notifying editor {} of settings update", editor_id);
            callback(editor_settings);
        }

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

        debug!("Registered line numbers callback {} for updates", id);
        id
    }

    /// Update line numbers setting and notify all registered callbacks
    pub fn update_line_numbers(&self, show_line_numbers: bool) -> anyhow::Result<()> {
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
pub fn init_editor_manager(settings_path: &str) -> anyhow::Result<()> {
    let manager = EditorManager::new(settings_path)?;

    // Log the startup editor settings for debugging
    let startup_settings = manager.get_current_editor_settings();
    debug!(
        "Initializing editor manager with startup settings: {} {}px line-height:{} wrap:{}",
        startup_settings.font_family,
        startup_settings.font_size,
        startup_settings.line_height,
        startup_settings.line_wrapping
    );

    EDITOR_MANAGER.with(|em| {
        *em.borrow_mut() = Some(Rc::new(std::cell::RefCell::new(manager)));
    });
    debug!("Global editor manager initialized successfully");
    Ok(())
}

/// Apply startup editor settings to all registered editors
pub fn apply_startup_editor_settings() -> anyhow::Result<()> {
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
        Err(anyhow::anyhow!("Editor manager not initialized"))
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
) -> anyhow::Result<()> {
    if let Some(manager) = get_editor_manager() {
        let mut mgr = manager.borrow_mut();
        mgr.update_editor_settings(editor_settings)
    } else {
        Err(anyhow::anyhow!("Editor manager not initialized"))
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
pub fn update_line_numbers_globally(show_line_numbers: bool) -> anyhow::Result<()> {
    if let Some(manager) = get_editor_manager() {
        let mgr = manager.borrow();
        mgr.update_line_numbers(show_line_numbers)
    } else {
        Err(anyhow::anyhow!("Editor manager not initialized"))
    }
}

/// Set scroll synchronization enabled/disabled globally
pub fn set_scroll_sync_enabled_globally(enabled: bool) -> anyhow::Result<()> {
    if let Some(manager) = get_editor_manager() {
        let mgr = manager.borrow();
        mgr.set_scroll_sync_enabled(enabled);
        Ok(())
    } else {
        Err(anyhow::anyhow!("Editor manager not initialized"))
    }
}
