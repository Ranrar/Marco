// Selection logic for EditorBuffer
use crate::logic::editor::core::EditorBuffer;

impl EditorBuffer {
    /// Select all text in the buffer
    pub fn select_all(&mut self) {
        if !self.lines.is_empty() {
            self.selection_start = Some((0, 0));
            let last_row = self.lines.len() - 1;
            let last_col = self.lines[last_row].len();
            self.selection_end = Some((last_row, last_col));
        }
    }
    /// Begin mouse selection at (row, col)
    pub fn mouse_select_start(&mut self, row: usize, col: usize) {
        self.selection_start = Some((row, col));
        self.selection_end = Some((row, col));
    }
    /// Update mouse selection to (row, col)
    pub fn mouse_select_update(&mut self, row: usize, col: usize) {
        self.selection_end = Some((row, col));
    }
    /// End mouse selection
    pub fn mouse_select_end(&mut self) {
        // Selection is finalized
    }
}
