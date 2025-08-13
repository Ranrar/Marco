// Navigation logic for EditorBuffer
use crate::logic::editor::core::EditorBuffer;

impl EditorBuffer {
    // Move cursor left (with bounds checking)
    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
        }
    }

    // Move cursor right (with bounds checking)
    pub fn move_right(&mut self) {
        if self.cursor_col < self.lines[self.cursor_row].len() {
            self.cursor_col += 1;
        } else if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    // Move cursor up (with bounds checking)
    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
        }
    }

    // Move cursor down (with bounds checking)
    pub fn move_down(&mut self) {
        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
        }
    }

    // Move cursor to start of line
    pub fn move_home(&mut self) {
        self.cursor_col = 0;
    }

    // Move cursor to end of line
    pub fn move_end(&mut self) {
        self.cursor_col = self.lines[self.cursor_row].len();
    }
}
