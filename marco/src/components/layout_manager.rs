//! Layout Manager: Handles application layout state changes
//!
//! This module manages transitions between different layout modes:
//! - DualView: Editor and preview side by side
//! - EditorOnly: Editor with document-like styling (100% width)
//! - ViewOnly: Preview only (handled by Polo)
//! - EditorAndViewSeparate: Editor in main window + Polo viewer

use gtk4::prelude::*;
use gtk4::Paned;
use log::{debug, error, info, warn};
use marco_core::logic::layoutstate::LayoutState;
use marco_core::{SessionManager, SessionData};
use crate::beacon::{Beacon, generate_socket_name};
use std::cell::RefCell;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// Callback type for layout state reversion when Polo closes
type PoloCloseCallback = Box<dyn Fn()>;

/// Layout manager that handles state transitions and UI adjustments
pub struct LayoutManager {
    /// The paned widget containing editor and preview
    paned: Paned,
    /// Current document path (needed for Polo launching)
    current_document: Rc<RefCell<Option<PathBuf>>>,
    /// Previous layout state (for return to editor functionality)
    previous_state: Rc<RefCell<Option<LayoutState>>>,
    /// Flag to prevent cascade updates during programmatic position changes
    position_being_set: Rc<RefCell<bool>>,
    /// Active Polo process (if any)
    polo_process: Rc<RefCell<Option<Child>>>,
    /// Callback to trigger when Polo closes naturally
    polo_close_callback: Rc<RefCell<Option<PoloCloseCallback>>>,
    /// Session manager for IPC communication
    session_manager: Arc<SessionManager>,
    /// Active beacon IPC server (if any) - Arc<Mutex<>> for thread safety
    beacon: Arc<Mutex<Option<Beacon>>>,
    /// Current session key (if any)
    current_session_key: Rc<RefCell<Option<String>>>,
}

impl LayoutManager {
    /// Create a new layout manager
    pub fn new(
        paned: Paned,
        current_document: Rc<RefCell<Option<PathBuf>>>,
        session_manager: Arc<SessionManager>,
    ) -> Self {
        Self {
            paned,
            current_document,
            previous_state: Rc::new(RefCell::new(None)),
            position_being_set: Rc::new(RefCell::new(false)),
            polo_process: Rc::new(RefCell::new(None)),
            polo_close_callback: Rc::new(RefCell::new(None)),
            session_manager,
            beacon: Arc::new(Mutex::new(None)),
            current_session_key: Rc::new(RefCell::new(None)),
        }
    }

    /// Set callback to be invoked when Polo closes naturally
    pub fn set_polo_close_callback<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        *self.polo_close_callback.borrow_mut() = Some(Box::new(callback));
    }

    /// Apply a layout state change
    pub fn apply_layout_state(&self, new_state: LayoutState, current_state: LayoutState) {
        info!("Layout transition: {:?} -> {:?}", current_state, new_state);

        // If leaving EditorAndViewSeparate mode, kill Polo process
        if current_state == LayoutState::EditorAndViewSeparate 
            && new_state != LayoutState::EditorAndViewSeparate 
        {
            self.kill_polo_process();
        }

        // Store previous state for return functionality
        if new_state != current_state {
            *self.previous_state.borrow_mut() = Some(current_state);
        }

        match new_state {
            LayoutState::DualView => self.apply_dual_view(),
            LayoutState::EditorOnly => self.apply_editor_only(),
            LayoutState::ViewOnly => self.apply_view_only(),
            LayoutState::EditorAndViewSeparate => self.apply_editor_and_view_separate(),
        }
    }

    /// Apply DualView layout (50/50 split by default)
    fn apply_dual_view(&self) {
        debug!("Applying DualView layout");
        
        // Remove document styling CSS classes
        self.paned.remove_css_class("editor-document-mode");
        
        // Set split to 50% (default)
        let width = self.paned.allocated_width();
        if width > 0 {
            let position = width / 2;
            self.set_paned_position(position);
            debug!("Set paned position to 50% ({} px)", position);
        } else {
            // Widget not allocated yet, try later
            let paned = self.paned.clone();
            let position_flag = self.position_being_set.clone();
            gtk4::glib::timeout_add_local_once(
                std::time::Duration::from_millis(100),
                move || {
                    let width = paned.allocated_width();
                    if width > 0 {
                        *position_flag.borrow_mut() = true;
                        paned.set_position(width / 2);
                        *position_flag.borrow_mut() = false;
                    }
                },
            );
        }
    }

    /// Apply EditorOnly layout (hide preview, show document-style editor)
    fn apply_editor_only(&self) {
        debug!("Applying EditorOnly layout");
        
        // Add document styling CSS class
        self.paned.add_css_class("editor-document-mode");
        
        // Set paned to 100% (hide preview pane completely)
        let width = self.paned.allocated_width();
        if width > 0 {
            // Set to actual width for 100%
            self.set_paned_position(width);
            debug!("Set paned position to 100% ({} px) - preview hidden", width);
        } else {
            // Widget not allocated yet, try later
            let paned = self.paned.clone();
            let position_flag = self.position_being_set.clone();
            gtk4::glib::timeout_add_local_once(
                std::time::Duration::from_millis(100),
                move || {
                    let width = paned.allocated_width();
                    if width > 0 {
                        *position_flag.borrow_mut() = true;
                        paned.set_position(width);
                        *position_flag.borrow_mut() = false;
                        debug!("Delayed set paned position to 100% ({} px)", width);
                    }
                },
            );
        }
    }

    /// Apply ViewOnly layout (for Polo viewer application)
    fn apply_view_only(&self) {
        debug!("Applying ViewOnly layout");
        warn!("ViewOnly mode should typically be handled by Polo application");
        
        // In Marco, we can set paned to show only preview (0 px for editor - 0%)
        self.paned.remove_css_class("editor-document-mode");
        self.set_paned_position(0);
        debug!("Set paned position to 0% (0 px) - editor hidden");
    }

    /// Apply EditorAndViewSeparate layout (editor 100%, launch Polo for preview)
    fn apply_editor_and_view_separate(&self) {
        debug!("Applying EditorAndViewSeparate layout");
        
        // Add document styling CSS class for editor
        self.paned.add_css_class("editor-document-mode");
        
        // Set paned to 100% (hide preview in main window)
        let width = self.paned.allocated_width();
        if width > 0 {
            self.set_paned_position(width);
        }
        
        // Launch Polo with current document
        if let Err(e) = self.launch_polo_viewer() {
            error!("Failed to launch Polo viewer: {}", e);
        }
    }

    /// Launch Polo viewer with IPC session for current document
    fn launch_polo_viewer(&self) -> Result<(), String> {
        let document = self.current_document.borrow();
        
        // Generate session key
        let session_key = SessionManager::generate_session_key();
        info!("Generated session key: [REDACTED]");
        
        // Create session data
        let session_data = SessionData::new(
            session_key.clone(),
            document.clone(),           // Option<PathBuf>
            "github".to_string(),       // TODO: Get from settings
            "dark".to_string(),          // TODO: Get from settings
            false,                      // read_only
        );
        
        // Register session
        self.session_manager.create_session(session_data);
        
        // Generate socket name
        let socket_name = generate_socket_name(&session_key);
        debug!("IPC socket: {}", socket_name);
        
        // Create beacon IPC server
        let beacon = Beacon::new(&socket_name, self.session_manager.clone())?;
        info!("Beacon IPC server started at: {}", socket_name);
        
        // Store beacon and session key
        *self.beacon.lock().unwrap() = Some(beacon);
        *self.current_session_key.borrow_mut() = Some(session_key.clone());
        
        // Get the polo binary path (try cargo target first, then system PATH)
        let polo_path = self.find_polo_binary()?;
        
        // Build command with session key and socket name
        let mut cmd = Command::new(&polo_path);
        cmd.arg("--session").arg(&session_key);
        cmd.arg("--socket").arg(&socket_name);
        
        // Add document path if available, otherwise Polo will fetch from session
        if let Some(ref doc_path) = *document {
            info!("Launching Polo viewer with session for: {:?}", doc_path);
            cmd.arg(doc_path.to_string_lossy().as_ref());
        } else {
            info!("Launching Polo viewer with session (no document)");
        }
        
        // Launch Polo
        let child = cmd.spawn()
            .map_err(|e| {
                error!("Failed to spawn Polo process: {}", e);
                format!("Failed to launch Polo: {}", e)
            })?;

        let child_id = child.id();
        info!("Polo viewer launched successfully (PID: {})", child_id);
        
        // Update session with Polo PID
        self.session_manager.set_polo_pid(&session_key, child_id);
        
        // Store the process handle
        *self.polo_process.borrow_mut() = Some(child);
        
        // Start monitoring for Polo process exit
        self.start_polo_monitor();
        
        // Start accepting IPC connections from Polo (in background thread)
        self.start_beacon_handler();
        
        Ok(())
    }
    
    /// Start accepting IPC connections from Polo in a background thread
    fn start_beacon_handler(&self) {
        let beacon = self.beacon.clone();
        
        // Spawn a thread to handle IPC connections
        // This thread will block on accept() without freezing the GTK main loop
        std::thread::spawn(move || {
            loop {
                // Check if beacon exists, and if so, accept connections
                let should_accept = {
                    let beacon_guard = beacon.lock().unwrap();
                    beacon_guard.is_some()
                };
                
                if should_accept {
                    // Now lock again and accept (we can't hold the lock during accept)
                    let beacon_guard = beacon.lock().unwrap();
                    if let Some(ref beacon_server) = *beacon_guard {
                        // We need to release the lock before the blocking call
                        // So we can't borrow beacon_server across the drop
                        // Instead, accept while holding the lock (brief hold)
                        drop(beacon_guard);
                        
                        // Re-acquire lock for accept
                        let result = {
                            let guard = beacon.lock().unwrap();
                            if let Some(ref server) = *guard {
                                server.accept_connection()
                            } else {
                                break; // Beacon was removed
                            }
                        };
                        
                        match result {
                            Ok(()) => {
                                info!("Beacon handled IPC connection successfully");
                            }
                            Err(e) => {
                                error!("Beacon connection error: {}", e);
                                // If the listener is broken, stop the thread
                                break;
                            }
                        }
                    }
                } else {
                    // No beacon - stop the thread
                    debug!("Beacon handler thread stopping (no beacon)");
                    break;
                }
            }
        });
    }

    /// Start monitoring Polo process for natural exit
    fn start_polo_monitor(&self) {
        let polo_process = self.polo_process.clone();
        let callback = self.polo_close_callback.clone();
        
        // Poll every 500ms to check if Polo is still running
        gtk4::glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
            // Check if process has exited and gather info
            let should_trigger_callback = {
                // Scope to ensure borrow is dropped before callback
                let mut process_opt = polo_process.borrow_mut();
                
                if let Some(ref mut child) = *process_opt {
                    // Try to check if process has exited
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            // Process exited
                            info!("Polo process exited naturally: {:?}", status);
                            *process_opt = None;
                            true // Signal to trigger callback
                        }
                        Ok(None) => {
                            // Still running
                            return gtk4::glib::ControlFlow::Continue;
                        }
                        Err(e) => {
                            error!("Error checking Polo process status: {}", e);
                            *process_opt = None;
                            false // Don't trigger callback on error
                        }
                    }
                } else {
                    // No process to monitor
                    return gtk4::glib::ControlFlow::Break;
                }
            }; // Borrow dropped here
            
            // Now trigger callback outside of borrow scope
            if should_trigger_callback {
                if let Some(ref cb) = *callback.borrow() {
                    cb();
                }
            }
            
            gtk4::glib::ControlFlow::Break
        });
    }

    /// Kill the Polo process if it's running
    fn kill_polo_process(&self) {
        let mut process_opt = self.polo_process.borrow_mut();
        
        if let Some(mut child) = process_opt.take() {
            let child_id = child.id();
            info!("Killing Polo process (PID: {})", child_id);
            
            match child.kill() {
                Ok(()) => {
                    info!("Polo process killed successfully");
                    // Wait for it to actually exit
                    let _ = child.wait();
                }
                Err(e) => {
                    error!("Failed to kill Polo process: {}", e);
                }
            }
        }
        
        // Clean up session and beacon
        if let Some(session_key) = self.current_session_key.borrow_mut().take() {
            debug!("Removing session: [REDACTED]");
            self.session_manager.remove_session(&session_key);
        }
        
        // Drop the beacon (closes socket)
        *self.beacon.lock().unwrap() = None;
        debug!("Beacon IPC server stopped");
    }

    /// Find the Polo binary (checks cargo target directory first, then PATH)
    fn find_polo_binary(&self) -> Result<PathBuf, String> {
        // Try cargo target directory first (development mode)
        let target_debug = PathBuf::from("target/debug/polo");
        let target_release = PathBuf::from("target/release/polo");
        
        if target_debug.exists() {
            return Ok(target_debug);
        }
        
        if target_release.exists() {
            return Ok(target_release);
        }
        
        // Try system PATH (installed version)
        which::which("polo")
            .map_err(|e| format!("Could not find polo binary: {}", e))
    }

    /// Set paned position with cascade prevention
    fn set_paned_position(&self, position: i32) {
        *self.position_being_set.borrow_mut() = true;
        self.paned.set_position(position);
        *self.position_being_set.borrow_mut() = false;
    }
    
    /// Send RefreshContent command to active Polo instance
    ///
    /// This is called when the editor buffer changes to update the preview.
    ///
    /// # Arguments
    ///
    /// * `html` - The rendered HTML content
    /// * `scroll_position` - Optional normalized scroll position (0.0 to 1.0)
    ///
    /// # Returns
    ///
    /// Ok(()) if command sent successfully, Err if no active Polo or send failed
    pub fn send_refresh_to_polo(&self, html: String, scroll_position: Option<f64>) -> Result<(), String> {
        // Get current session key
        let session_key = match self.current_session_key.borrow().as_ref() {
            Some(key) => key.clone(),
            None => {
                debug!("No active Polo session - skipping refresh");
                return Ok(()); // Not an error, just no viewer active
            }
        };
        
        // Get beacon and send command
        let beacon_guard = self.beacon.lock()
            .map_err(|e| format!("Failed to lock beacon: {}", e))?;
        
        if let Some(ref beacon) = *beacon_guard {
            use marco_core::components::api::protocol::ServerCommand;
            beacon.send_command(&session_key, ServerCommand::refresh_content(html, scroll_position))
        } else {
            debug!("Beacon not active - skipping refresh");
            Ok(()) // Not an error, just no beacon
        }
    }
    
    /// Send UpdateTheme command to active Polo instance
    pub fn send_theme_to_polo(&self, theme: String, editor_theme: String) -> Result<(), String> {
        let session_key = match self.current_session_key.borrow().as_ref() {
            Some(key) => key.clone(),
            None => return Ok(()),
        };
        
        let beacon_guard = self.beacon.lock()
            .map_err(|e| format!("Failed to lock beacon: {}", e))?;
        
        if let Some(ref beacon) = *beacon_guard {
            use marco_core::components::api::protocol::ServerCommand;
            beacon.send_command(&session_key, ServerCommand::update_theme(theme, editor_theme))
        } else {
            Ok(())
        }
    }
    
    /// Get the previous layout state (for return functionality)
    pub fn previous_state(&self) -> Option<LayoutState> {
        *self.previous_state.borrow()
    }

    /// Update the current document path
    pub fn set_current_document(&self, path: Option<PathBuf>) {
        *self.current_document.borrow_mut() = path;
    }

    /// Shutdown and cleanup - kill Polo process if running
    pub fn shutdown(&self) {
        debug!("LayoutManager shutdown - cleaning up Polo process");
        self.kill_polo_process();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke_test_layout_manager_creation() {
        // Note: This test doesn't actually create GTK widgets as that requires
        // an initialized GTK application. This just verifies the structure compiles.
        
        // In a real application context with GTK initialized:
        // let paned = Paned::new(gtk4::Orientation::Horizontal);
        // let doc = Rc::new(RefCell::new(None));
        // let manager = LayoutManager::new(paned, doc);
        
        // For now, just verify the module compiles
    }

    #[test]
    fn smoke_test_session_key_generation() {
        // Verify session key generation works
        let key = marco_core::SessionManager::generate_session_key();
        assert!(!key.is_empty());
        assert!(key.len() >= 32); // UUID v4 format (with hyphens removed it's 32 chars)
    }
}
