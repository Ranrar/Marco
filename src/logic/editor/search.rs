// Search and replace logic for EditorBuffer
use crate::logic::editor::core::EditorBuffer;

impl EditorBuffer {
    /// Find the next occurrence of a string, returns (row, col) or None
    pub fn find_next(&self, query: &str, from: Option<(usize, usize)>) -> Option<(usize, usize)> {
        let (mut row, mut col) = from.unwrap_or((self.cursor_row, self.cursor_col));
        for r in row..self.lines.len() {
            let start = if r == row { col } else { 0 };
            if let Some(idx) = self.lines[r][start..].find(query) {
                return Some((r, start + idx));
            }
        }
        None
    }

    /// Replace the next occurrence of a string
    pub fn replace_next(&mut self, query: &str, replacement: &str, from: Option<(usize, usize)>) -> bool {
        if let Some((row, col)) = self.find_next(query, from) {
            self.push_undo();
            let line = &mut self.lines[row];
            line.replace_range(col..col + query.len(), replacement);
            self.cursor_row = row;
            self.cursor_col = col + replacement.len();
            return true;
        }
        false
    }
}
