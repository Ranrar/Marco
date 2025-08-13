// Editing logic for EditorBuffer
use crate::logic::editor::core::EditorBuffer;

impl EditorBuffer {
    /// Delete character before cursor (backspace)
    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            self.push_undo();
            let line = &mut self.lines[self.cursor_row];
            line.remove(self.cursor_col - 1);
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.push_undo();
            let prev_len = self.lines[self.cursor_row - 1].len();
            let current = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = prev_len;
            self.lines[self.cursor_row].push_str(&current);
        }
    }

    /// Delete character at cursor (delete)
    pub fn delete(&mut self) {
        if self.cursor_row < self.lines.len() {
            if self.cursor_col < self.lines[self.cursor_row].len() {
                self.push_undo();
                self.lines[self.cursor_row].remove(self.cursor_col);
            } else if self.cursor_row + 1 < self.lines.len() {
                self.push_undo();
                let next_line = self.lines.remove(self.cursor_row + 1);
                self.lines[self.cursor_row].push_str(&next_line);
            }
        }
    }

    /// Paste text at cursor
    pub fn paste(&mut self, text: &str) {
        self.push_undo();
        let row = self.cursor_row;
        let col = self.cursor_col;
        if let Some(line) = self.lines.get_mut(row) {
            line.insert_str(col, text);
            self.cursor_col += text.len();
        }
    }
}
