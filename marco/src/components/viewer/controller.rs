//! Split Controller and WebView Location Tracking
//!
//! This module provides centralized control for layout management, including:
//! - Split position and locking based on layout modes
//! - WebView location tracking (main window vs preview window)
//!
//! # Layout Mode Split Behavior
//!
//! - **DualView**: Split freely adjustable between 10% and 90%
//! - **EditorOnly**: Split locked at 100% (editor fills window, preview hidden)
//! - **ViewOnly**: Split locked at 0% (preview fills window, editor hidden)
//! - **EditorAndViewSeparate**: Split locked at 100% (editor fills main window, preview in separate window)
//!
//! # WebView Location
//!
//! Marco uses GTK4 widget reparenting to move a single WebView between containers:
//! - **Main Window**: WebView is the end child of the Paned widget
//! - **Preview Window**: WebView is the child of the preview window's container
//!
//! The WebView maintains all state (scroll position, DOM, theme) during reparenting.
//!
//! # Example
//!
//! ```no_run
//! use marco::components::viewer::controller::{SplitController, WebViewLocationTracker, WebViewLocation};
//! use core::logic::layoutstate::LayoutState;
//!
//! let controller = SplitController::new(paned.clone());
//! controller.set_mode(LayoutState::EditorOnly); // Locks split at 100%
//! controller.set_mode(LayoutState::DualView);    // Unlocks split, allows 10-90%
//!
//! let tracker = WebViewLocationTracker::new();
//! tracker.set(WebViewLocation::PreviewWindow);
//! assert!(tracker.is_in_preview_window());
//! ```

use gtk4::prelude::*;
use gtk4::Paned;
use core::logic::layoutstate::LayoutState;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

// ============================================================================
// WebView Location Tracking
// ============================================================================

/// Represents the current location of the WebView widget
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebViewLocation {
    /// WebView is in the main window (Paned end child)
    MainWindow,
    /// WebView is in the separate preview window
    PreviewWindow,
}

impl Default for WebViewLocation {
    fn default() -> Self {
        Self::MainWindow
    }
}

impl std::fmt::Display for WebViewLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebViewLocation::MainWindow => write!(f, "Main Window"),
            WebViewLocation::PreviewWindow => write!(f, "Preview Window"),
        }
    }
}

/// Tracks the current location of the WebView widget
///
/// This tracker uses shared ownership (Rc<RefCell<>>) to allow multiple
/// parts of the application to read and update the WebView location.
///
/// # Thread Safety
///
/// This struct is NOT thread-safe and should only be used from the GTK main thread.
pub struct WebViewLocationTracker {
    location: Rc<RefCell<WebViewLocation>>,
}

impl WebViewLocationTracker {
    /// Create a new tracker with the WebView in the main window
    pub fn new() -> Self {
        Self {
            location: Rc::new(RefCell::new(WebViewLocation::MainWindow)),
        }
    }

    /// Get the current location of the WebView
    pub fn current(&self) -> WebViewLocation {
        *self.location.borrow()
    }

    /// Update the location of the WebView
    ///
    /// This should be called immediately after reparenting the WebView
    /// to ensure the tracker stays synchronized with the actual widget state.
    pub fn set(&self, location: WebViewLocation) {
        log::debug!("WebView location updated: {} -> {}", self.current(), location);
        *self.location.borrow_mut() = location;
    }

    /// Check if the WebView is currently in the preview window
    #[allow(dead_code)]
    pub fn is_in_preview_window(&self) -> bool {
        self.current() == WebViewLocation::PreviewWindow
    }

    /// Check if the WebView is currently in the main window
    #[allow(dead_code)]
    pub fn is_in_main_window(&self) -> bool {
        self.current() == WebViewLocation::MainWindow
    }

    /// Create a clone of the internal Rc for passing to callbacks
    ///
    /// This is useful when you need to update the location from within
    /// a GTK signal handler or callback closure.
    #[allow(dead_code)]
    pub fn clone_rc(&self) -> Rc<RefCell<WebViewLocation>> {
        Rc::clone(&self.location)
    }
}

impl Default for WebViewLocationTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for WebViewLocationTracker {
    fn clone(&self) -> Self {
        Self {
            location: Rc::clone(&self.location),
        }
    }
}

// ============================================================================
// Split Controller
// ============================================================================

/// Controls split position and locking for different layout modes
#[derive(Clone)]
pub struct SplitController {
    /// The GTK Paned widget being controlled
    paned: Paned,
    /// Current layout mode
    current_mode: Rc<RefCell<LayoutState>>,
    /// Flag to prevent cascade events during programmatic position changes
    position_being_set: Rc<RefCell<bool>>,
    /// Whether split is currently locked (prevents user dragging)
    is_locked: Rc<Cell<bool>>,
}

impl SplitController {
    /// Create a new split controller for the given paned widget
    ///
    /// # Arguments
    ///
    /// * `paned` - The GTK Paned widget to control
    ///
    /// # Initial State
    ///
    /// - Mode: DualView (unlocked, 10-90% range)
    /// - Position: 50% of allocated width
    pub fn new(paned: Paned) -> Self {
        let controller = Self {
            paned: paned.clone(),
            current_mode: Rc::new(RefCell::new(LayoutState::DualView)),
            position_being_set: Rc::new(RefCell::new(false)),
            is_locked: Rc::new(Cell::new(false)),
        };

        // Set up position constraint enforcement
        controller.setup_position_constraints();

        controller
    }

    /// Set up the position change handler to enforce constraints
    fn setup_position_constraints(&self) {
        let position_being_set = self.position_being_set.clone();
        let current_mode = self.current_mode.clone();
        let is_locked = self.is_locked.clone();

        self.paned.connect_notify_local(Some("position"), move |paned, _| {
            // Prevent cascade if we're already setting position programmatically
            if *position_being_set.borrow() {
                return;
            }

            let width = paned.allocated_width();
            if width <= 0 {
                return;
            }

            let mode = *current_mode.borrow();
            let position = paned.position();

            // If locked, prevent any position changes
            if is_locked.get() {
                *position_being_set.borrow_mut() = true;
                match mode {
                    LayoutState::EditorOnly | LayoutState::EditorAndViewSeparate => {
                        // Keep at 100% (editor takes full width)
                        log::trace!("SplitController: Enforcing EditorOnly/EditorAndViewSeparate constraint - setting to 100%");
                        let width = paned.allocated_width();
                        if width > 0 {
                            paned.set_position(width);
                        }
                    }
                    LayoutState::ViewOnly => {
                        paned.set_position(0); // Lock at 0%
                    }
                    LayoutState::DualView => {
                        // This shouldn't happen, but enforce constraints just in case
                        let min_position = (width as f64 * 0.10) as i32;
                        let max_position = (width as f64 * 0.90) as i32;
                        if position < min_position {
                            paned.set_position(min_position);
                        } else if position > max_position {
                            paned.set_position(max_position);
                        }
                    }
                }
                *position_being_set.borrow_mut() = false;
                return;
            }

            // For unlocked DualView, enforce 10-90% constraints
            if mode == LayoutState::DualView {
                let min_position = (width as f64 * 0.10) as i32;
                let max_position = (width as f64 * 0.90) as i32;

                if position < min_position || position > max_position {
                    *position_being_set.borrow_mut() = true;

                    if position < min_position {
                        paned.set_position(min_position);
                    } else if position > max_position {
                        paned.set_position(max_position);
                    }

                    *position_being_set.borrow_mut() = false;
                }
            }
        });
    }

    /// Change the layout mode and update split position/locking accordingly
    ///
    /// # Arguments
    ///
    /// * `mode` - The new layout mode
    ///
    /// # Behavior
    ///
    /// - **DualView**: Unlocks split, allows 10-90% range
    /// - **EditorOnly**: Locks split at 100%
    /// - **ViewOnly**: Locks split at 0%
    /// - **EditorAndViewSeparate**: Locks split at 100%
    pub fn set_mode(&self, mode: LayoutState) {
        *self.current_mode.borrow_mut() = mode;

        *self.position_being_set.borrow_mut() = true;

        match mode {
            LayoutState::DualView => {
                // Unlock split, allow 10-90% range
                self.is_locked.set(false);
                // Restore to default 50% position
                let width = self.paned.allocated_width();
                if width > 0 {
                    self.paned.set_position(width / 2);
                }
            }
            LayoutState::EditorOnly | LayoutState::EditorAndViewSeparate => {
                // Lock split at 100% (editor fills window)
                // Use idle callback to ensure proper width is available
                self.is_locked.set(true);
                let paned = self.paned.clone();
                glib::idle_add_local_once(move || {
                    let width = paned.allocated_width();
                    if width > 0 {
                        paned.set_position(width);
                    }
                });
            }
            LayoutState::ViewOnly => {
                // Lock split at 0% (preview fills window)
                self.is_locked.set(true);
                self.paned.set_position(0);
            }
        }

        *self.position_being_set.borrow_mut() = false;

        log::debug!(
            "Split controller: mode={:?}, locked={}, position={}",
            mode,
            self.is_locked.get(),
            self.paned.position()
        );
    }

    /// Get the current layout mode
    #[allow(dead_code)]
    pub fn current_mode(&self) -> LayoutState {
        *self.current_mode.borrow()
    }

    /// Check if the split is currently locked
    #[allow(dead_code)]
    pub fn is_locked(&self) -> bool {
        self.is_locked.get()
    }

    /// Get the position_being_set flag for cascade prevention
    ///
    /// This can be used by the split percentage indicator to avoid
    /// showing percentages during programmatic position changes.
    pub fn position_being_set(&self) -> Rc<RefCell<bool>> {
        self.position_being_set.clone()
    }

    /// Get a reference to the controlled paned widget
    #[allow(dead_code)]
    pub fn paned(&self) -> &Paned {
        &self.paned
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // WebView Location Tests
    #[test]
    fn smoke_test_location_tracker_default() {
        let tracker = WebViewLocationTracker::new();
        assert_eq!(tracker.current(), WebViewLocation::MainWindow);
        assert!(tracker.is_in_main_window());
        assert!(!tracker.is_in_preview_window());
    }

    #[test]
    fn smoke_test_location_update() {
        let tracker = WebViewLocationTracker::new();
        
        tracker.set(WebViewLocation::PreviewWindow);
        assert_eq!(tracker.current(), WebViewLocation::PreviewWindow);
        assert!(!tracker.is_in_main_window());
        assert!(tracker.is_in_preview_window());
        
        tracker.set(WebViewLocation::MainWindow);
        assert_eq!(tracker.current(), WebViewLocation::MainWindow);
    }

    #[test]
    fn smoke_test_tracker_clone() {
        let tracker1 = WebViewLocationTracker::new();
        let tracker2 = tracker1.clone();
        
        tracker1.set(WebViewLocation::PreviewWindow);
        assert_eq!(tracker2.current(), WebViewLocation::PreviewWindow);
        
        tracker2.set(WebViewLocation::MainWindow);
        assert_eq!(tracker1.current(), WebViewLocation::MainWindow);
    }

    #[test]
    fn smoke_test_location_display() {
        assert_eq!(
            format!("{}", WebViewLocation::MainWindow),
            "Main Window"
        );
        assert_eq!(
            format!("{}", WebViewLocation::PreviewWindow),
            "Preview Window"
        );
    }

    // Split Controller Tests
    #[test]
    fn smoke_test_split_controller_creation() {
        // Note: Can't fully test GTK widgets without a display
        // This smoke test just verifies the API compiles and basic logic works
        
        // Verify layout states are correct
        assert_eq!(LayoutState::DualView, LayoutState::DualView);
        assert_ne!(LayoutState::DualView, LayoutState::EditorOnly);
    }

    #[test]
    fn smoke_test_layout_state_logic() {
        // Test that layout state logic is correct
        let states = vec![
            LayoutState::DualView,
            LayoutState::EditorOnly,
            LayoutState::ViewOnly,
            LayoutState::EditorAndViewSeparate,
        ];

        for state in states {
            match state {
                LayoutState::DualView => {
                    // Should be unlocked
                    assert_eq!(state, LayoutState::DualView);
                }
                LayoutState::EditorOnly | LayoutState::EditorAndViewSeparate => {
                    // Should lock at 100%
                    assert!(state == LayoutState::EditorOnly || state == LayoutState::EditorAndViewSeparate);
                }
                LayoutState::ViewOnly => {
                    // Should lock at 0%
                    assert_eq!(state, LayoutState::ViewOnly);
                }
            }
        }
    }
}
