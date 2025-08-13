// Multi-cursor and multi-selection logic for EditorBuffer
// This module provides functions for managing multiple cursors and selections.

use crate::logic::editor::core::EditorBuffer;

impl EditorBuffer {
    /// Add a new cursor at (row, col)
    pub fn add_cursor(&mut self, row: usize, col: usize) {
        self.multi_cursors.push((row, col));
    }

    /// Remove a cursor at index
    pub fn remove_cursor(&mut self, index: usize) {
        if index < self.multi_cursors.len() {
            self.multi_cursors.remove(index);
        }
    }

    /// Clear all additional cursors
    pub fn clear_cursors(&mut self) {
        self.multi_cursors.clear();
    }

    /// Add a new selection (start, end)
    pub fn add_selection(&mut self, start: Option<(usize, usize)>, end: Option<(usize, usize)>) {
        self.multi_selections.push((start, end));
    }

    /// Remove a selection at index
    pub fn remove_selection(&mut self, index: usize) {
        if index < self.multi_selections.len() {
            self.multi_selections.remove(index);
        }
    }

    /// Clear all additional selections
    pub fn clear_selections(&mut self) {
        self.multi_selections.clear();
    }
}
