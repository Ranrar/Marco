// Undo/redo logic for EditorBuffer
use crate::logic::editor::core::EditorBuffer;

impl EditorBuffer {
    pub fn push_undo(&mut self) {
        self.undo_stack.push(self.lines.clone());
        self.redo_stack.clear();
    }
    pub fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.lines.clone());
            self.lines = prev;
        }
    }
    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.lines.clone());
            self.lines = next;
        }
    }
}
