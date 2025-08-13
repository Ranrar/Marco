// Clipboard logic for EditorBuffer
use crate::logic::editor::core::EditorBuffer;
use gtk4::gdk;
use gtk4::prelude::DisplayExt;

impl EditorBuffer {
    /// Copy selected text to system clipboard (GTK4 GDK API)
    /// Copy selected text to system clipboard (GTK4 GDK API)
    pub fn copy_to_clipboard(&self) {
        let text = self.copy();
        let display = gdk::Display::default().expect("No display found");
        let clipboard = display.clipboard();
        clipboard.set_text(&text);
    }

    /// Paste text from system clipboard at cursor (GTK4 GDK API)
    pub fn paste_from_clipboard(&mut self) {
        let display = gdk::Display::default().expect("No display found");
        let clipboard = display.clipboard();
        clipboard.read_text_async(None::<&gio::Cancellable>, move |result| {
            match result {
                Ok(Some(text)) => println!("Clipboard paste: {}", text),
                Ok(None) => println!("Clipboard is empty"),
                Err(e) => eprintln!("Clipboard error: {}", e),
            }
        });
// Provide a basic copy() method for EditorBuffer
impl EditorBuffer {
    /// Return selected text (stub: returns first line for now)
    pub fn copy(&self) -> String {
        if let (Some((row, col_start)), Some((_, col_end))) = (self.selection_start, self.selection_end) {
            if row < self.lines.len() && col_end > col_start {
                return self.lines[row][col_start..col_end].to_string();
            }
        }
        self.lines.get(0).cloned().unwrap_or_default()
    }
}
    }
}
