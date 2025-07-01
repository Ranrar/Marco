use gtk4::prelude::*;
use crate::editor::core::MarkdownEditor;

impl MarkdownEditor {
    pub fn insert_heading(&self, level: u8) {
        let prefix = "#".repeat(level as usize);
        self.insert_at_new_line(&format!("{} ", prefix));
    }

    pub fn insert_bullet_list(&self) {
        self.insert_at_new_line("- ");
    }

    pub fn insert_numbered_list(&self) {
        self.insert_at_new_line("1. ");
    }

    pub fn insert_blockquote(&self) {
        self.insert_at_new_line("> ");
    }

    pub fn insert_link(&self) {
        self.show_link_dialog();
    }

    pub fn insert_image(&self) {
        self.show_image_dialog();
    }

    pub fn insert_table(&self) {
        let table = "\n| Header 1 | Header 2 | Header 3 |\n|----------|----------|----------|\n| Cell 1   | Cell 2   | Cell 3   |\n| Cell 4   | Cell 5   | Cell 6   |\n";
        self.insert_text_at_cursor(table);
    }

    pub fn insert_custom_table(&self, rows: usize, cols: usize) {
        let mut table = String::new();
        table.push('\n');
        
        // Create header row
        table.push('|');
        for c in 0..cols {
            table.push_str(&format!(" Header {} |", c + 1));
        }
        table.push('\n');
        
        // Create separator row
        table.push('|');
        for _ in 0..cols {
            table.push_str("----------|");
        }
        table.push('\n');
        
        // Create data rows
        for r in 0..rows {
            table.push('|');
            for c in 0..cols {
                table.push_str(&format!(" Cell {}-{} |", r + 1, c + 1));
            }
            table.push('\n');
        }
        
        self.insert_text_at_cursor(&table);
    }

    pub fn insert_code_block(&self) {
        let code_block = "\n```\ncode goes here\n```\n";
        self.insert_text_at_cursor(code_block);
    }

    // Extended Markdown syntax methods based on https://www.markdownguide.org/extended-syntax/

    pub fn insert_task_list(&self) {
        self.insert_at_new_line("[ ] Task\n[x] Completed task\n[ ] Another task\n");
    }

    pub fn insert_single_open_task(&self) {
        self.insert_at_new_line("[ ] Task\n");
    }

    pub fn insert_single_closed_task(&self) {
        self.insert_at_new_line("[x] Completed task\n");
    }

    pub fn insert_custom_task_list(&self, count: usize) {
        let mut task_list = String::new();
        for i in 0..count {
            task_list.push_str(&format!("[ ] Task {}\n", i + 1));
        }
        self.insert_at_new_line(&task_list);
    }

    pub fn insert_footnote(&self) {
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        
        if gtk_buffer.has_selection() {
            // Use selected text for footnote
            if let Some((start, end)) = gtk_buffer.selection_bounds() {
                let selected_text = gtk_buffer.text(&start, &end, false);
                let footnote_text = format!("{}[^1]", selected_text);
                
                // Create a mark to preserve the position before deletion
                let insert_mark = gtk_buffer.create_mark(None, &start, false);
                
                let mut start_mut = start;
                let mut end_mut = end;
                buffer.delete(&mut start_mut, &mut end_mut);
                
                // Get a fresh iterator from the mark after deletion
                let mut insert_iter = gtk_buffer.iter_at_mark(&insert_mark);
                buffer.insert(&mut insert_iter, &footnote_text);
                
                // Clean up the temporary mark
                gtk_buffer.delete_mark(&insert_mark);
            }
        } else {
            self.insert_text_at_cursor("text[^1]");
        }
        
        // Add footnote definition at the end
        self.insert_text_at_cursor("\n\n[^1]: Footnote text here");
    }

    pub fn insert_definition_list(&self) {
        self.insert_at_new_line("Term 1\n: Definition 1\n\nTerm 2\n: Definition 2\n");
    }

    pub fn insert_single_definition(&self) {
        self.insert_at_new_line("Term\n: Definition\n");
    }

    pub fn insert_custom_definition_list(&self, count: usize) {
        let mut definition_list = String::new();
        for i in 0..count {
            definition_list.push_str(&format!("Term {}\n: Definition {}\n", i + 1, i + 1));
            if i < count - 1 {
                definition_list.push('\n');
            }
        }
        self.insert_at_new_line(&definition_list);
    }

    #[allow(dead_code)]
    pub fn insert_emoji(&self) {
        // This could show an emoji picker in the future
        // For now, just insert a placeholder
        self.insert_text_at_cursor(":smile:");
    }

    pub fn insert_emoji_text(&self, emoji: &str) {
        self.insert_text_at_cursor(emoji);
    }

    pub fn insert_fenced_code_block(&self) {
        self.show_fenced_code_dialog();
    }

    pub fn insert_fenced_code_with_language(&self, language: &str) {
        let code_block = format!("\n```{}\ncode goes here\n```\n", language);
        self.insert_text_at_cursor(&code_block);
    }

    /// Add a new programming language to the manager
    #[allow(dead_code)]
    pub fn add_programming_language(&self, language: crate::code_languages::CodeLanguage) {
        self.code_language_manager.borrow_mut().add_language(language);
    }

    /// Get available programming languages
    #[allow(dead_code)]
    pub fn get_available_languages(&self) -> Vec<String> {
        self.code_language_manager.borrow().get_language_names()
    }

    /// Get language suggestions based on input
    #[allow(dead_code)]
    pub fn get_language_suggestions(&self, partial: &str) -> Vec<String> {
        self.code_language_manager.borrow().get_language_suggestions(partial)
    }

    pub fn insert_horizontal_rule(&self) {
        self.insert_at_new_line("---\n");
    }

    fn insert_at_new_line(&self, text: &str) {
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        
        // Get current cursor position using mark
        let cursor_mark = gtk_buffer.get_insert();
        let cursor_iter = gtk_buffer.iter_at_mark(&cursor_mark);
        
        // Check if we're at the start of a line
        let line_offset = cursor_iter.line_offset();
        
        // Prepare the text to insert
        let text_to_insert = if line_offset == 0 {
            text.to_string()
        } else {
            format!("\n{}", text)
        };
        
        // Insert at cursor mark (this is safer than using iterators)
        buffer.insert_at_cursor(&text_to_insert);
    }
}
