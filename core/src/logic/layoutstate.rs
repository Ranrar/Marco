//! # LayoutState: Application Layout Modes
//!
//! This module defines the `LayoutState` enum, which represents all possible UI layout modes for the Marco markdown composer application.
//!
//! ## Purpose
//! - Centralizes layout state management for the main editor and preview views.
//! - Enables switching between different UI configurations (split, editor-only, view-only, and windowed modes).
//!
//! ## Enum Variants
//! - `Split`: Editor and preview are both visible in the main window (default).
//! - `EditorOnly`: Only the editor is visible in the main window.
//! - `ViewOnly`: Only the preview (right pane) is visible in the main application window. The editor (left pane) is hidden, but the main window remains open and active.
//! - `EditorAndWin`: Editor remains in the main window, preview is moved to a separate window.
//! - `ViewWinOnly`: Only the preview is shown, and it is moved to a separate window. The editor is hidden, and the main window may be minimized or closed, leaving only the detached preview window visible to the user.
//!
//! ## Usage
//! - The current `LayoutState` is typically stored in a shared `Rc<RefCell<LayoutState>>` and updated in response to user actions (menu, buttons).
//! - UI components observe the state and update their visibility/layout accordingly.
//! - Use `layout_state_label(state)` to get a human-readable description for logging, tooltips, or debugging.
//!
//! ## Example
//! ```no_run
//! use core::logic::layoutstate::{LayoutState, layout_state_label};
//! let state = LayoutState::DualView;
//! println!("Current layout: {}", layout_state_label(state));
//! ```
//!
//! ## Design Notes
//! - Enum variants are exhaustive and should be extended if new layout modes are added.
//! - The label function provides clear, user-facing descriptions for each mode.
//! - This module should not contain direct UI logic—only state and helpers.
/// LayoutState represents the current layout mode of the application.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayoutState {
    DualView,              // A: Editor + View (default)
    EditorOnly,            // B: Editor only
    ViewOnly,              // C: View only
    EditorAndViewSeparate, // D: Editor + view in separate window
}

/// Converts a LayoutState to a human-readable string label.
pub fn layout_state_label(state: LayoutState) -> &'static str {
    match state {
        LayoutState::DualView => "standard dual view",
        LayoutState::EditorOnly => "editor view only",
        LayoutState::ViewOnly => "preview view only",
        LayoutState::EditorAndViewSeparate => "editor + view in separate window",
    }
}
