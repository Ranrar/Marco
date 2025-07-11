use crate::editor::core::MarkdownEditor;
use gtk4::prelude::*;

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
        self.focus_editor_and_position_cursor();
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
        self.focus_editor_and_position_cursor();
    }

    pub fn insert_code_block(&self) {
        let code_block = "\n```\ncode goes here\n```\n";
        self.insert_text_at_cursor(code_block);
        self.focus_editor_and_position_cursor();
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

        // Get the full document text to analyze existing footnotes
        let start_iter = gtk_buffer.start_iter();
        let end_iter = gtk_buffer.end_iter();
        let full_text = gtk_buffer.text(&start_iter, &end_iter, false);

        // Find the next footnote number
        let footnote_number = self.find_next_footnote_number(&full_text);

        if gtk_buffer.has_selection() {
            // Use selected text for footnote
            if let Some((start, end)) = gtk_buffer.selection_bounds() {
                let selected_text = gtk_buffer.text(&start, &end, false);
                let footnote_text = format!("{}[^{}]", selected_text, footnote_number);

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
            // Insert footnote reference at cursor
            self.insert_text_at_cursor(&format!("text[^{}]", footnote_number));
        }

        // Add footnote definition at the end of the document
        self.add_footnote_definition_to_bottom(&full_text, footnote_number);

        // Focus editor after insertion
        self.focus_editor_and_position_cursor();
    }

    /// Find the next available footnote number by scanning existing footnotes
    fn find_next_footnote_number(&self, text: &str) -> u32 {
        use crate::utils::cache::get_regex;
        let footnote_regex = get_regex(r"\[\^(\d+)\]");
        let mut max_number = 0;

        for captures in footnote_regex.captures_iter(text) {
            if let Ok(number) = captures[1].parse::<u32>() {
                max_number = max_number.max(number);
            }
        }

        max_number + 1
    }

    /// Add footnote definition at the bottom of the document
    fn add_footnote_definition_to_bottom(&self, current_text: &str, footnote_number: u32) {
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();

        // Position cursor at the end of the document
        let mut end_iter = gtk_buffer.end_iter();

        // Check if we need to add separators
        let needs_separator =
            !current_text.ends_with("\n---\n") && !current_text.contains("\n---\n");

        let mut footnote_text = String::new();

        // Add separator if needed
        if needs_separator {
            if !current_text.is_empty() && !current_text.ends_with('\n') {
                footnote_text.push('\n');
            }
            footnote_text.push_str("\n---\n\n");
        } else if !current_text.ends_with('\n') {
            footnote_text.push('\n');
        }

        // Add the footnote definition
        footnote_text.push_str(&format!("[^{}]: Footnote text here\n", footnote_number));

        buffer.insert(&mut end_iter, &footnote_text);
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
        // Show the native GTK4 emoji picker
        crate::editor::emoji::show_emoji_picker_dialog(self);
    }

    pub fn insert_fenced_code_block(&self) {
        self.show_fenced_code_dialog();
    }

    pub fn insert_fenced_code_with_language(&self, language: &str) {
        let code_block = format!("\n```{}\ncode goes here\n```\n", language);
        self.insert_text_at_cursor(&code_block);
        self.focus_editor_and_position_cursor();
    }

    /// Add a new programming language to the manager
    #[allow(dead_code)]
    pub fn add_programming_language(&self, language: String) {
        self.code_language_manager
            .borrow_mut()
            .add_language(language);
    }

    /// Get available programming languages
    #[allow(dead_code)]
    pub fn get_available_languages(&self) -> Vec<String> {
        self.code_language_manager.borrow().get_language_names()
    }

    /// Get language suggestions based on input
    #[allow(dead_code)]
    pub fn get_language_suggestions(&self, partial: &str) -> Vec<String> {
        self.code_language_manager
            .borrow()
            .get_language_suggestions(partial)
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

        // Focus editor and position cursor
        self.focus_editor_and_position_cursor();
    }

    /// Focus the editor and move cursor to a new line if needed
    pub fn focus_editor_and_position_cursor(&self) {
        // Focus the source editor
        self.source_view.grab_focus();

        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();

        // Get current cursor position
        let cursor_mark = gtk_buffer.get_insert();
        let cursor_iter = gtk_buffer.iter_at_mark(&cursor_mark);

        // Check if we need to move to a new line
        let line_offset = cursor_iter.line_offset();
        let mut line_end_iter = cursor_iter.clone();
        line_end_iter.forward_to_line_end();
        let line_text = cursor_iter.slice(&line_end_iter);

        // If we're not at the end of the line and there's text after cursor, move to new line
        if line_offset > 0 && !line_text.trim().is_empty() {
            buffer.insert_at_cursor("\n");
        }

        // Scroll to cursor position
        self.source_view
            .scroll_to_mark(&cursor_mark, 0.0, false, 0.0, 0.0);
    }
}
