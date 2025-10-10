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
use std::cell::RefCell;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::rc::Rc;

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
}

impl LayoutManager {
    /// Create a new layout manager
    pub fn new(
        paned: Paned,
        current_document: Rc<RefCell<Option<PathBuf>>>,
    ) -> Self {
        Self {
            paned,
            current_document,
            previous_state: Rc::new(RefCell::new(None)),
            position_being_set: Rc::new(RefCell::new(false)),
            polo_process: Rc::new(RefCell::new(None)),
            polo_close_callback: Rc::new(RefCell::new(None)),
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

    /// Launch Polo viewer with API key for current document (or empty if no document)
    fn launch_polo_viewer(&self) -> Result<(), String> {
        let document = self.current_document.borrow();
        
        // Generate API key for secure Polo communication
        let api_key = marco_core::logic::api::generate_simple_view_key();
        
        // Get the polo binary path (try cargo target first, then system PATH)
        let polo_path = self.find_polo_binary()?;
        
        // Build command with API key
        let mut cmd = Command::new(&polo_path);
        cmd.arg("--api").arg(&api_key);
        
        // Add document path if available, otherwise Polo will show empty state
        if let Some(ref doc_path) = *document {
            info!("Launching Polo viewer with API key for: {:?}", doc_path);
            cmd.arg(doc_path.to_string_lossy().as_ref());
        } else {
            info!("Launching Polo viewer with API key (no document)");
        }
        
        // Launch Polo
        let child = cmd.spawn()
            .map_err(|e| {
                error!("Failed to spawn Polo process: {}", e);
                format!("Failed to launch Polo: {}", e)
            })?;

        let child_id = child.id();
        info!("Polo viewer launched successfully (PID: {})", child_id);
        
        // Store the process handle
        *self.polo_process.borrow_mut() = Some(child);
        
        // Start monitoring for Polo process exit
        self.start_polo_monitor();
        
        Ok(())
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
    fn smoke_test_api_key_generation() {
        // Verify API key generation works
        let key = marco_core::logic::api::generate_simple_view_key();
        assert!(!key.is_empty());
        assert!(key.len() > 8); // Should be a reasonable length hex string
    }
}
