// Mouse selection and drag logic for EditorBuffer
// This module provides functions for mouse-based selection and drag support in the editor.

use crate::logic::editor::core::EditorBuffer;

impl EditorBuffer {
    /// Begin mouse selection (on mouse down)
    pub fn mouse_select_start(&mut self, row: usize, col: usize) {
        self.selection_start = Some((row, col));
        self.selection_end = Some((row, col));
    }

    /// Update mouse selection (on mouse move)
    pub fn mouse_select_update(&mut self, row: usize, col: usize) {
        self.selection_end = Some((row, col));
    }

    /// End mouse selection (on mouse up)
    pub fn mouse_select_end(&mut self, row: usize, col: usize) {
        self.selection_end = Some((row, col));
    }

    /// Clear selection (e.g., on mouse click without drag)
    pub fn mouse_clear_selection(&mut self) {
        self.selection_start = None;
        self.selection_end = None;
    }
}
