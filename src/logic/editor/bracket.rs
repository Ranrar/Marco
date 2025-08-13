// Bracket matching logic for EditorBuffer
use crate::logic::editor::core::EditorBuffer;

impl EditorBuffer {
    /// Find matching bracket/parenthesis for cursor position
    pub fn find_matching_bracket(&self) -> Option<(usize, usize)> {
        let pairs = [('(', ')'), ('[', ']'), ('{', '}')];
        let row = self.cursor_row;
        let col = self.cursor_col;
        if row >= self.lines.len() || col == 0 || col > self.lines[row].len() {
            return None;
        }
        let ch = self.lines[row].chars().nth(col - 1)?;
        for &(open, close) in &pairs {
            if ch == open {
                // Forward search for matching close
                let mut depth = 1;
                for r in row..self.lines.len() {
                    let start = if r == row { col } else { 0 };
                    for (i, c) in self.lines[r][start..].chars().enumerate() {
                        if c == open { depth += 1; }
                        if c == close {
                            depth -= 1;
                            if depth == 0 {
                                return Some((r, start + i));
                            }
                        }
                    }
                }
            } else if ch == close {
                // Backward search for matching open
                let mut depth = 1;
                for r in (0..=row).rev() {
                    let end = if r == row { col - 1 } else { self.lines[r].len() };
                    for (i, c) in self.lines[r][..end].chars().rev().enumerate() {
                        if c == close { depth += 1; }
                        if c == open {
                            depth -= 1;
                            if depth == 0 {
                                return Some((r, end - i - 1));
                            }
                        }
                    }
                }
            }
        }
        None
    }
}
