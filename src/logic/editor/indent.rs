// Indentation and commenting logic for EditorBuffer
use crate::logic::editor::core::EditorBuffer;

impl EditorBuffer {
    /// Auto-indent current line (simple: match previous line's indent)
    pub fn auto_indent(&mut self) {
        if self.cursor_row > 0 {
            let prev = &self.lines[self.cursor_row - 1];
            let indent: String = prev.chars().take_while(|c| c.is_whitespace()).collect();
            let line = &mut self.lines[self.cursor_row];
            let rest = line.trim_start().to_string();
            *line = format!("{}{}", indent, rest);
            self.cursor_col = indent.len();
        }
    }

    /// Comment/uncomment current line (simple: add/remove //)
    pub fn toggle_comment(&mut self) {
        let line = &mut self.lines[self.cursor_row];
        if line.trim_start().starts_with("//") {
            if let Some(idx) = line.find("//") {
                line.replace_range(idx..idx+2, "");
            }
        } else {
            let idx = line.chars().take_while(|c| c.is_whitespace()).count();
            line.insert_str(idx, "//");
        }
    }
}
