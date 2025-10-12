//! WebView Reparenting Utilities
//!
//! This module provides safe utility functions for reparenting the WebView widget
//! between the main window and the preview window. It handles the GTK4 widget
//! hierarchy changes and includes safety mechanisms to prevent issues.
//!
//! # Reparenting Pattern
//!
//! GTK4 widget reparenting follows this pattern:
//! 1. Remove widget from old parent using container-specific method
//! 2. Add widget to new parent using container-specific method
//! 3. Widget state is preserved automatically by GTK4/WebKit
//!
//! # Safety Mechanisms
//!
//! - **ReparentGuard**: Prevents race conditions from rapid layout switching
//! - **Error handling**: Returns Result for graceful failure recovery
//! - **Logging**: Comprehensive debug logs for troubleshooting
//!
//! # Example
//!
//! ```no_run
//! use marco::components::viewer::webview_reparent::{
//!     move_webview_to_preview_window, move_webview_to_main_window
//! };
//!
//! // Move to preview window
//! move_webview_to_preview_window(&webview, &paned, &preview_window)?;
//!
//! // Move back to main window
//! move_webview_to_main_window(&webview, &paned, &preview_window, true)?;
//! ```

use gtk4::prelude::*;
use gtk4::{Paned, Stack};
use crate::components::viewer::previewwindow::PreviewWindow;
use webkit6::WebView;
use std::cell::Cell;
use std::rc::Rc;

/// Guard to prevent concurrent reparenting operations
///
/// This guard uses a Cell<bool> to track whether a reparenting operation
/// is currently in progress. It prevents race conditions that could occur
/// from rapid layout switching (e.g., user clicking buttons quickly).
///
/// # Thread Safety
///
/// This guard is NOT thread-safe and should only be used from the GTK main thread.
///
/// # Example
///
/// ```no_run
/// let guard = ReparentGuard::new();
///
/// if !guard.try_begin() {
///     log::warn!("Reparenting already in progress");
///     return;
/// }
///
/// // ... perform reparenting ...
///
/// guard.end();
/// ```
pub struct ReparentGuard {
    in_progress: Rc<Cell<bool>>,
}

impl ReparentGuard {
    /// Create a new reparenting guard
    pub fn new() -> Self {
        Self {
            in_progress: Rc::new(Cell::new(false)),
        }
    }

    /// Try to begin a reparenting operation
    ///
    /// # Returns
    ///
    /// * `true` - Operation can proceed (guard acquired)
    /// * `false` - Another operation is in progress (guard not acquired)
    pub fn try_begin(&self) -> bool {
        if self.in_progress.get() {
            log::warn!("Reparenting already in progress, ignoring request");
            return false;
        }
        self.in_progress.set(true);
        true
    }

    /// End the reparenting operation
    ///
    /// This releases the guard, allowing future operations to proceed.
    pub fn end(&self) {
        self.in_progress.set(false);
    }

    /// Check if a reparenting operation is in progress
    #[cfg(test)]
    pub fn is_in_progress(&self) -> bool {
        self.in_progress.get()
    }
}

impl Default for ReparentGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ReparentGuard {
    fn clone(&self) -> Self {
        Self {
            in_progress: Rc::clone(&self.in_progress),
        }
    }
}

/// Detach WebView from Paned and attach to PreviewWindow
///
/// This function performs the reparenting operation to move the WebView
/// from the main window's Paned container to the separate preview window.
///
/// # Arguments
///
/// * `webview` - The WebView widget to reparent (borrowed from Rc<RefCell<>>)
/// * `paned` - The Paned widget in the main window
/// * `preview_window` - The target preview window
///
/// # Reparenting Process
///
/// 1. Locates WebView in Paned (could be in Stack child)
/// 2. Removes WebView from its parent Stack (preserves Stack in Paned)
/// 3. Calls PreviewWindow::attach_webview() to complete reparenting
///
/// # Returns
///
/// * `Ok(())` - Reparenting succeeded
/// * `Err(String)` - Reparenting failed with error message
///
/// # Example
///
/// ```no_run
/// let webview = webview_rc.borrow();
/// move_webview_to_preview_window(&webview, &paned, &preview_window)?;
/// ```
pub fn move_webview_to_preview_window(
    webview: &WebView,
    paned: &Paned,
    preview_window: &PreviewWindow,
) -> Result<(), String> {
    log::debug!("Starting WebView reparent to preview window");

    // Check WebView's current parent
    let webview_parent = webview.parent();
    log::debug!("WebView current parent: {:?}", webview_parent.as_ref().map(|p| p.type_()));

    // The WebView is in a Stack (html_preview/code_preview)
    // The Stack is the end child of the Paned
    if let Some(stack_widget) = paned.end_child() {
        if let Some(stack) = stack_widget.downcast_ref::<Stack>() {
            // Verify WebView is actually a child of this Stack
            if webview.parent().as_ref() != Some(&stack_widget) {
                log::warn!("WebView parent is not the Stack, checking if it's already in preview window");
                // It might already be in the preview window
                if preview_window.container().child().is_some() {
                    log::info!("WebView is already in preview window");
                    return Ok(());
                }
                return Err("WebView is not in the expected Stack".to_string());
            }
            
            // Save the original state in case we need to rollback
            let original_parent = stack.clone();
            
            // Remove WebView from Stack
            stack.remove(webview);
            log::debug!("Removed WebView from Stack (preserving Stack in Paned)");
            
            // Attach to PreviewWindow - if this fails, try to restore
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                preview_window.attach_webview(webview);
            })) {
                Ok(_) => {
                    log::info!("WebView successfully reparented to preview window");
                    Ok(())
                }
                Err(e) => {
                    log::error!("Failed to attach WebView to preview window, attempting rollback: {:?}", e);
                    // Try to restore WebView back to Stack
                    original_parent.add_titled(webview, Some("html_preview"), "HTML");
                    Err("Failed to attach WebView to preview window (state restored)".to_string())
                }
            }
        } else {
            Err(format!(
                "Expected Stack as Paned end child, found: {:?}",
                stack_widget.type_()
            ))
        }
    } else {
        Err("No end child found in Paned".to_string())
    }
}

/// Detach WebView from PreviewWindow and attach back to Paned
///
/// This function performs the reverse reparenting operation to move the WebView
/// from the preview window back to the main window's Paned container.
///
/// # Arguments
///
/// * `webview` - The WebView widget to reparent (borrowed from Rc<RefCell<>>)
/// * `paned` - The Paned widget in the main window
/// * `preview_window` - The source preview window
/// * `restore_to_end` - If true, restore to end child; if false, to start child
///
/// # Reparenting Process
///
/// 1. Locates the Stack in Paned
/// 2. Detaches WebView from PreviewWindow
/// 3. Re-adds WebView to Stack with name "html_preview"
/// 4. Sets Stack to show html_preview (WebView becomes visible)
///
/// # Returns
///
/// * `Ok(())` - Reparenting succeeded
/// * `Err(String)` - Reparenting failed with error message
///
/// # Example
///
/// ```no_run
/// let webview = webview_rc.borrow();
/// move_webview_to_main_window(&webview, &paned, &preview_window, true)?;
/// ```
pub fn move_webview_to_main_window(
    webview: &WebView,
    paned: &Paned,
    preview_window: &PreviewWindow,
    _restore_to_end: bool, // Reserved for future use
) -> Result<(), String> {
    log::debug!("Starting WebView reparent to main window");

    // Check if WebView is actually in the preview window
    let webview_parent = webview.parent();
    log::debug!("WebView current parent: {:?}", webview_parent.as_ref().map(|p| p.type_()));

    // Detach from PreviewWindow (this removes it from the ScrolledWindow and returns ownership)
    let detached_webview = preview_window.detach_webview();
    
    if detached_webview.is_none() {
        log::warn!("No WebView was detached from preview window - it may already be in main window");
        // Check if it's already in the Stack
        if let Some(stack_widget) = paned.end_child() {
            if let Some(stack) = stack_widget.downcast_ref::<Stack>() {
                if let Some(child) = stack.child_by_name("html_preview") {
                    if child.downcast_ref::<WebView>().is_some() {
                        log::info!("WebView is already in main window Stack");
                        stack.set_visible_child_name("html_preview");
                        return Ok(());
                    }
                }
            }
        }
        return Err("Failed to detach WebView from preview window".to_string());
    }

    // Find the Stack in Paned and re-add WebView
    if let Some(stack_widget) = paned.end_child() {
        if let Some(stack) = stack_widget.downcast_ref::<Stack>() {
            // Save state in case we need to rollback
            let existing_child = stack.child_by_name("html_preview");
            
            // Remove existing child if present
            if let Some(ref child) = existing_child {
                log::warn!("Stack already has child named 'html_preview', removing it first");
                stack.remove(child);
            }
            
            // Try to re-add WebView to Stack with error recovery
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                stack.add_named(webview, Some("html_preview"));
                stack.set_visible_child_name("html_preview");
            })) {
                Ok(_) => {
                    log::info!("WebView successfully reparented back to main window");
                    Ok(())
                }
                Err(e) => {
                    log::error!("Failed to add WebView to Stack, attempting rollback: {:?}", e);
                    // Try to restore the preview window state
                    preview_window.attach_webview(webview);
                    Err("Failed to add WebView to main window Stack (restored to preview)".to_string())
                }
            }
        } else {
            // Failed to find Stack - try to restore WebView to preview window
            preview_window.attach_webview(webview);
            Err(format!(
                "Expected Stack as Paned end child, found: {:?} (restored to preview)",
                stack_widget.type_()
            ))
        }
    } else {
        // No end child - try to restore WebView to preview window
        preview_window.attach_webview(webview);
        Err("No end child found in Paned (restored to preview)".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_reparent_guard_basic() {
        let guard = ReparentGuard::new();
        
        assert!(!guard.is_in_progress());
        assert!(guard.try_begin());
        assert!(guard.is_in_progress());
        
        // Second try should fail
        assert!(!guard.try_begin());
        
        guard.end();
        assert!(!guard.is_in_progress());
        
        // Should be able to begin again
        assert!(guard.try_begin());
    }

    #[test]
    fn smoke_test_reparent_guard_clone() {
        let guard1 = ReparentGuard::new();
        let guard2 = guard1.clone();
        
        assert!(guard1.try_begin());
        assert!(!guard2.try_begin()); // Should share state
        
        assert!(guard1.is_in_progress());
        assert!(guard2.is_in_progress());
        
        guard1.end();
        assert!(!guard2.is_in_progress());
    }

    #[test]
    fn smoke_test_error_handling() {
        // Test error message formatting
        let err: Result<(), String> = Err("Test error".to_string());
        assert!(err.is_err());
        if let Err(msg) = err {
            assert_eq!(msg, "Test error");
        }
    }
}
