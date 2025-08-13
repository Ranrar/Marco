// File I/O logic for EditorBuffer
use crate::logic::editor::core::EditorBuffer;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

impl EditorBuffer {
    /// Open a file and load its contents into the buffer
    pub fn open_file(&mut self, path: &str) -> std::io::Result<()> {
        match File::open(path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                match reader.lines().collect::<Result<_, _>>() {
                    Ok(lines) => {
                        self.lines = lines;
                        self.cursor_row = 0;
                        self.cursor_col = 0;
                        self.scroll_offset = 0;
                        self.selection_start = None;
                        self.selection_end = None;
                        self.undo_stack.clear();
                        self.redo_stack.clear();
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Save buffer contents to a file
    pub fn save_file(&self, path: &str) -> std::io::Result<()> {
        match File::create(path) {
            Ok(mut file) => {
                for (i, line) in self.lines.iter().enumerate() {
                    if i > 0 {
                        if let Err(e) = writeln!(file) {
                            return Err(e);
                        }
                    }
                    if let Err(e) = write!(file, "{}", line) {
                        return Err(e);
                    }
                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
