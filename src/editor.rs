use gtk4::prelude::*;
use gtk4::{
    FileChooserAction, FileChooserDialog, HeaderBar, Label, Orientation,
    Paned, ResponseType, ScrolledWindow, Widget, Dialog, Entry, Grid, FileFilter,
    EventControllerKey, gdk, ComboBoxText,
};
use sourceview5::prelude::*;
use sourceview5::{Buffer, LanguageManager, StyleSchemeManager, View};
use std::cell::RefCell;
use std::rc::Rc;
use crate::preview::MarkdownPreview;
use crate::code_languages::CodeLanguageManager;

#[derive(Clone)]
pub struct MarkdownEditor {
    widget: Paned,
    #[allow(dead_code)]
    source_view: View,
    preview: MarkdownPreview,
    source_buffer: Buffer,
    current_file: Rc<RefCell<Option<std::path::PathBuf>>>,
    footer_callbacks: Rc<RefCell<Vec<Box<dyn Fn(&str, usize, usize, usize, usize)>>>>,
    code_language_manager: Rc<RefCell<CodeLanguageManager>>,
}

impl MarkdownEditor {
    pub fn new() -> Self {
        // Create the main paned widget
        let paned = Paned::new(Orientation::Horizontal);
        // Set initial position to 50% - will be properly set after window is shown
        paned.set_position(400);
        
        // Set resize behavior: both panes can resize
        paned.set_resize_start_child(true);
        paned.set_resize_end_child(true);
        
        // Set shrink behavior: both panes can shrink but have minimum sizes
        paned.set_shrink_start_child(false);
        paned.set_shrink_end_child(false);

        // Create source view with buffer
        let source_buffer = Buffer::new(None);
        let source_view = View::with_buffer(&source_buffer);

        // Configure source view
        source_view.set_show_line_numbers(true);
        source_view.set_highlight_current_line(true);
        source_view.set_tab_width(4);
        source_view.set_insert_spaces_instead_of_tabs(true);
        source_view.set_auto_indent(true);

        // Set up syntax highlighting for markdown
        let language_manager = LanguageManager::default();
        if let Some(language) = language_manager.language("markdown") {
            source_buffer.set_language(Some(&language));
        }

        // Set up style scheme
        let style_manager = StyleSchemeManager::default();
        if let Some(scheme) = style_manager.scheme("Adwaita") {
            source_buffer.set_style_scheme(Some(&scheme));
        }

        // Create preview
        let preview = MarkdownPreview::new();

        // Create scrolled window for source view
        let source_scroll = ScrolledWindow::new();
        source_scroll.set_child(Some(&source_view));
        source_scroll.set_vexpand(true);
        source_scroll.set_size_request(200, -1); // Minimum width of 200px

        // Add to paned
        paned.set_start_child(Some(&source_scroll));
        paned.set_end_child(Some(preview.widget()));

        // Clamp split position logic
        // We'll clamp the position between min and max (e.g., 200px and total_width-200px)
        let _paned_clone = paned.clone();
        paned.connect_notify(Some("position"), move |paned, _pspec| {
            if let Some(window) = paned.root().and_then(|w| w.downcast::<gtk4::Window>().ok()) {
                let total_width = window.allocated_width();
                let min = 200;
                let max = (total_width - 200).max(min);
                let pos = paned.position();
                if pos < min {
                    paned.set_position(min);
                } else if pos > max {
                    paned.set_position(max);
                }
            }
        });

        // Set up current_file and footer_callbacks
        let current_file = Rc::new(RefCell::new(None));
        let footer_callbacks = Rc::new(RefCell::new(Vec::new()));
        let code_language_manager = Rc::new(RefCell::new(CodeLanguageManager::new()));

        let editor = Self {
            widget: paned,
            source_view,
            preview,
            source_buffer,
            current_file,
            footer_callbacks,
            code_language_manager,
        };

        // Connect text change signal
        editor.connect_text_changed();
        editor.connect_cursor_moved();
        
        // Set up keyboard shortcuts
        editor.setup_keyboard_shortcuts();

        // Ensure 50/50 split on window realize
        let paned = editor.widget.clone();
        paned.connect_realize(move |paned| {
            if let Some(window) = paned.root().and_then(|w| w.downcast::<gtk4::Window>().ok()) {
                let total_width = window.allocated_width();
                paned.set_position(total_width / 2);
            }
        });

        editor
    }

    pub fn widget(&self) -> &Widget {
        self.widget.upcast_ref()
    }

    /// Get access to the source buffer for external cursor tracking
    pub fn get_source_buffer(&self) -> &Buffer {
        &self.source_buffer
    }

    pub fn set_split_ratio(&self, total_width: i32) {
        // Set the split position to 50% of the total width
        self.widget.set_position(total_width / 2);
        // No min/max position methods in GTK4
    }

    pub fn add_footer_callback<F>(&self, callback: F)
    where
        F: Fn(&str, usize, usize, usize, usize) + 'static,
    {
        self.footer_callbacks.borrow_mut().push(Box::new(callback));
    }

    fn connect_text_changed(&self) {
        let preview = self.preview.clone();
        let footer_callbacks = self.footer_callbacks.clone();
        self.source_buffer.connect_changed(move |buffer| {
            let start = buffer.start_iter();
            let end = buffer.end_iter();
            let text = buffer.text(&start, &end, false);
            
            // Update preview
            preview.update_content(&text);
            
            // Update footer statistics
            let char_count = text.chars().count();
            let word_count = text.split_whitespace().count();
            
            // Use GTK TextBuffer's get_insert mark
            let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
            let cursor_iter = gtk_buffer.iter_at_mark(&gtk_buffer.get_insert());
            let line = cursor_iter.line() + 1;
            let column = cursor_iter.line_offset() + 1;
            
            for callback in footer_callbacks.borrow().iter() {
                callback(&text, word_count, char_count, line as usize, column as usize);
            }
        });
    }

    fn connect_cursor_moved(&self) {
        let footer_callbacks = self.footer_callbacks.clone();
        
        self.source_buffer.connect_mark_set(move |buffer, _iter, mark| {
            // Use GTK TextBuffer's get_insert mark
            let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
            if mark == &gtk_buffer.get_insert() {
                let start = buffer.start_iter();
                let end = buffer.end_iter();
                let text = buffer.text(&start, &end, false);
                
                let char_count = text.chars().count();
                let word_count = text.split_whitespace().count();
                let cursor_iter = gtk_buffer.iter_at_mark(&gtk_buffer.get_insert());
                let line = cursor_iter.line() + 1;
                let column = cursor_iter.line_offset() + 1;
                
                for callback in footer_callbacks.borrow().iter() {
                    callback(&text, word_count, char_count, line as usize, column as usize);
                }
            }
        });
    }

    pub fn create_simple_header_bar(&self) -> HeaderBar {
        let header_bar = HeaderBar::new();
        header_bar.set_title_widget(Some(&Label::new(Some("Marco - Markdown Composer"))));
        header_bar
    }

    pub fn insert_heading(&self, level: u8) {
        let prefix = "#".repeat(level as usize);
        self.insert_at_new_line(&format!("{} ", prefix));
    }

    pub fn insert_bold(&self) {
        self.toggle_format_selection_only("**", "**");
    }

    pub fn insert_italic(&self) {
        self.toggle_format_selection_only("*", "*");
    }

    pub fn insert_inline_code(&self) {
        self.toggle_format_selection_only("`", "`");
    }

    pub fn insert_strikethrough(&self) {
        self.toggle_format_selection_only("~~", "~~");
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
        self.insert_at_new_line("- [ ] Task\n- [x] Completed task\n- [ ] Another task\n");
    }

    pub fn insert_single_open_task(&self) {
        self.insert_at_new_line("- [ ] Task\n");
    }

    pub fn insert_single_closed_task(&self) {
        self.insert_at_new_line("- [x] Completed task\n");
    }

    pub fn insert_custom_task_list(&self, count: usize) {
        let mut task_list = String::new();
        for i in 0..count {
            task_list.push_str(&format!("- [ ] Task {}\n", i + 1));
        }
        self.insert_at_new_line(&task_list);
    }

    pub fn insert_footnote(&self) {
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        let cursor_mark = gtk_buffer.get_insert();
        let mut cursor_iter = gtk_buffer.iter_at_mark(&cursor_mark);
        
        // Insert footnote reference at cursor
        buffer.insert(&mut cursor_iter, "[^1]");
        
        // Add footnote definition at the end
        let mut end_iter = gtk_buffer.end_iter();
        buffer.insert(&mut end_iter, "\n\n[^1]: Your footnote here.");
    }

    pub fn insert_definition_list(&self) {
        self.insert_at_new_line("First Term\n: This is the definition of the first term.\n\nSecond Term\n: This is one definition of the second term.\n: This is another definition of the second term.\n");
    }

    pub fn insert_highlight(&self) {
        self.toggle_format_selection_only("==", "==");
    }

    pub fn insert_subscript(&self) {
        self.toggle_format_selection_only("~", "~");
    }

    pub fn insert_superscript(&self) {
        self.toggle_format_selection_only("^", "^");
    }

    pub fn insert_emoji(&self) {
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        let cursor_mark = gtk_buffer.get_insert();
        let mut cursor_iter = gtk_buffer.iter_at_mark(&cursor_mark);
        
        // Insert some common emoji shortcodes as examples
        buffer.insert(&mut cursor_iter, ":smile: :heart: :thumbsup:");
    }

    pub fn insert_fenced_code_block(&self) {
        self.show_fenced_code_dialog();
    }

    pub fn insert_fenced_code_with_language(&self, language: &str) {
        let code_block = format!("```{}\nYour {} code here\n```\n", language, language);
        self.insert_at_new_line(&code_block);
    }

    /// Show dialog to select programming language for fenced code block
    fn show_fenced_code_dialog(&self) {
        let dialog = Dialog::with_buttons(
            Some("Insert Fenced Code Block"),
            None::<&gtk4::Window>,
            gtk4::DialogFlags::MODAL,
            &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
        );
        
        dialog.set_default_size(400, 200);
        
        // Create the grid layout
        let grid = Grid::new();
        grid.set_row_spacing(12);
        grid.set_column_spacing(12);
        grid.set_margin_top(20);
        grid.set_margin_bottom(20);
        grid.set_margin_start(20);
        grid.set_margin_end(20);
        
        // Language selection
        let lang_label = Label::new(Some("Programming Language:"));
        lang_label.set_halign(gtk4::Align::Start);
        
        let lang_combo = ComboBoxText::new();
        lang_combo.set_hexpand(true);
        
        // Populate with available languages
        let languages = self.code_language_manager.borrow().get_language_names();
        for language in &languages {
            lang_combo.append_text(language);
        }
        
        // Set default to "text" or first language
        if !languages.is_empty() {
            lang_combo.set_active(Some(0));
        }
        
        // Add common languages that might not be in the list
        lang_combo.append_text("text");
        lang_combo.append_text("bash");
        lang_combo.append_text("shell");
        lang_combo.append_text("sql");
        lang_combo.append_text("json");
        lang_combo.append_text("xml");
        lang_combo.append_text("html");
        lang_combo.append_text("css");
        lang_combo.append_text("markdown");
        lang_combo.append_text("yaml");
        lang_combo.append_text("toml");
        
        // Code sample input
        let code_label = Label::new(Some("Code Sample (optional):"));
        code_label.set_halign(gtk4::Align::Start);
        
        let code_entry = gtk4::TextView::new();
        code_entry.set_vexpand(true);
        code_entry.set_hexpand(true);
        
        let code_scroll = ScrolledWindow::new();
        code_scroll.set_child(Some(&code_entry));
        code_scroll.set_size_request(350, 100);
        code_scroll.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
        
        // Add placeholder text
        let code_buffer = code_entry.buffer();
        code_buffer.set_text("// Your code here...");
        
        // Add to grid
        grid.attach(&lang_label, 0, 0, 1, 1);
        grid.attach(&lang_combo, 1, 0, 1, 1);
        grid.attach(&code_label, 0, 1, 2, 1);
        grid.attach(&code_scroll, 0, 2, 2, 1);
        
        // Add grid to dialog
        dialog.content_area().append(&grid);
        
        // Focus on language combo
        lang_combo.grab_focus();
        
        // Connect response
        let buffer_clone = self.source_buffer.clone();
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                let language = lang_combo.active_text()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "text".to_string());
                
                let code_buffer = code_entry.buffer();
                let start = code_buffer.start_iter();
                let end = code_buffer.end_iter();
                let code_sample = code_buffer.text(&start, &end, false);
                
                let code_content = if code_sample.trim().is_empty() || code_sample.trim() == "// Your code here..." {
                    format!("Your {} code here", language)
                } else {
                    code_sample.to_string()
                };
                
                let fenced_block = format!("```{}\n{}\n```\n", language, code_content);
                
                // Insert the fenced code block
                let gtk_buffer = buffer_clone.upcast_ref::<gtk4::TextBuffer>();
                let cursor_mark = gtk_buffer.get_insert();
                let mut cursor_iter = gtk_buffer.iter_at_mark(&cursor_mark);
                buffer_clone.insert(&mut cursor_iter, &fenced_block);
            }
            dialog.close();
        });
        
        dialog.present();
    }

    /// Add a new programming language to the manager
    pub fn add_programming_language(&self, language: crate::code_languages::CodeLanguage) {
        self.code_language_manager.borrow_mut().add_language(language);
    }

    /// Get available programming languages
    pub fn get_available_languages(&self) -> Vec<String> {
        self.code_language_manager.borrow().get_language_names()
    }

    /// Get language suggestions based on input
    pub fn get_language_suggestions(&self, partial: &str) -> Vec<String> {
        self.code_language_manager.borrow().get_language_suggestions(partial)
    }

    fn insert_text_at_cursor(&self, text: &str) {
        let gtk_buffer = self.source_buffer.upcast_ref::<gtk4::TextBuffer>();
        let cursor_mark = gtk_buffer.get_insert();
        let mut cursor_iter = gtk_buffer.iter_at_mark(&cursor_mark);
        
        self.source_buffer.insert(&mut cursor_iter, text);
    }

    #[allow(dead_code)]
    fn find_format_at_cursor(&self, line_text: &str, cursor_offset: i32, prefix: &str, suffix: &str) -> Option<(i32, i32)> {
        let cursor_pos = cursor_offset as usize;
        let line_str = line_text;
        
        // Look for formatting that contains the cursor
        let mut pos = 0;
        while let Some(start_pos) = line_str[pos..].find(prefix) {
            let absolute_start = pos + start_pos;
            let search_from = absolute_start + prefix.len();
            
            if let Some(end_pos) = line_str[search_from..].find(suffix) {
                let absolute_end = search_from + end_pos + suffix.len();
                
                // Check if cursor is within this formatting
                if cursor_pos >= absolute_start && cursor_pos <= absolute_end {
                    return Some((absolute_start as i32, absolute_end as i32));
                }
                
                pos = absolute_end;
            } else {
                break;
            }
        }
        
        None
    }

    pub fn insert_horizontal_rule(&self) {
        self.insert_at_new_line("---\n");
    }

    fn insert_at_new_line(&self, text: &str) {
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        
        // Get current cursor position
        let cursor_mark = gtk_buffer.get_insert();
        let mut cursor_iter = gtk_buffer.iter_at_mark(&cursor_mark);
        
        // Check if we're at the start of a line
        let line_offset = cursor_iter.line_offset();
        
        if line_offset == 0 {
            // We're at the start of a line, just insert
            buffer.insert(&mut cursor_iter, text);
        } else {
            // We're in the middle of a line, add a newline first
            buffer.insert(&mut cursor_iter, &format!("\n{}", text));
        }
    }

    fn toggle_format_selection_only(&self, prefix: &str, suffix: &str) {
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        
        // Only work if text is selected
        if gtk_buffer.has_selection() {
            // Get selection bounds
            if let Some((start, end)) = gtk_buffer.selection_bounds() {
                let selected_text = gtk_buffer.text(&start, &end, false);
                let selected_str = selected_text.as_str();
                
                // Create the replacement text
                let replacement_text = if selected_str.starts_with(prefix) && selected_str.ends_with(suffix) && selected_str.len() > prefix.len() + suffix.len() {
                    // Remove formatting - extract inner text
                    selected_str[prefix.len()..selected_str.len() - suffix.len()].to_string()
                } else {
                    // Add formatting - wrap the selected text
                    format!("{}{}{}", prefix, selected_str, suffix)
                };
                
                // Use begin/end user action for atomic operation
                buffer.begin_user_action();
                
                // Get fresh bounds for the operation
                if let Some((mut start_iter, mut end_iter)) = gtk_buffer.selection_bounds() {
                    // Replace the selected text with the formatted/unformatted version
                    buffer.delete(&mut start_iter, &mut end_iter);
                    
                    // Get a fresh iterator at the insertion point
                    let insert_mark = gtk_buffer.get_insert();
                    let mut insert_iter = gtk_buffer.iter_at_mark(&insert_mark);
                    buffer.insert(&mut insert_iter, &replacement_text);
                }
                
                buffer.end_user_action();
            }
        }
        // If no text is selected, do nothing
    }

    /// Check if cursor is within a specific formatting pattern
    /// Returns true if the cursor position contains the formatting
    #[allow(dead_code)]
    pub fn is_cursor_in_format(&self, prefix: &str, suffix: &str) -> bool {
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        let cursor_iter = gtk_buffer.iter_at_mark(&gtk_buffer.get_insert());
        
        let line_start = gtk_buffer.iter_at_line(cursor_iter.line()).unwrap_or_else(|| cursor_iter);
        let mut line_end = gtk_buffer.iter_at_line(cursor_iter.line()).unwrap_or_else(|| cursor_iter);
        
        if !line_end.ends_line() {
            line_end.forward_to_line_end();
        }
        
        let line_text = gtk_buffer.text(&line_start, &line_end, false);
        let cursor_offset = cursor_iter.line_offset();
        
        self.find_format_at_cursor(&line_text, cursor_offset, prefix, suffix).is_some()
    }

    /// Check if cursor is on a heading line and return the heading level
    #[allow(dead_code)]
    pub fn get_heading_level_at_cursor(&self) -> Option<usize> {
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        let cursor_iter = gtk_buffer.iter_at_mark(&gtk_buffer.get_insert());
        
        let line_start = gtk_buffer.iter_at_line(cursor_iter.line()).unwrap_or_else(|| cursor_iter);
        let mut line_end = gtk_buffer.iter_at_line(cursor_iter.line()).unwrap_or_else(|| cursor_iter);
        
        if !line_end.ends_line() {
            line_end.forward_to_line_end();
        }
        
        let line_text = gtk_buffer.text(&line_start, &line_end, false);
        let trimmed = line_text.trim();
        
        // Check for heading pattern
        if trimmed.starts_with('#') {
            let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
            if hash_count <= 6 && trimmed.len() > hash_count && trimmed.chars().nth(hash_count) == Some(' ') {
                return Some(hash_count);
            }
        }
        
        None
    }

    fn show_link_dialog(&self) {
        // Create the dialog
        let dialog = Dialog::with_buttons(
            Some("Insert Link"),
            None::<&gtk4::Window>,
            gtk4::DialogFlags::MODAL,
            &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
        );
        
        // Create the grid layout
        let grid = Grid::new();
        grid.set_row_spacing(10);
        grid.set_column_spacing(10);
        grid.set_margin_top(20);
        grid.set_margin_bottom(20);
        grid.set_margin_start(20);
        grid.set_margin_end(20);
        
        // URL input
        let url_label = Label::new(Some("URL:"));
        url_label.set_halign(gtk4::Align::Start);
        let url_entry = Entry::new();
        url_entry.set_placeholder_text(Some("https://example.com"));
        url_entry.set_hexpand(true);
        
        // Link text input
        let text_label = Label::new(Some("Link Text:"));
        text_label.set_halign(gtk4::Align::Start);
        let text_entry = Entry::new();
        text_entry.set_placeholder_text(Some("Link description"));
        text_entry.set_hexpand(true);
        
        // Check if we have selected text to use as default
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        if gtk_buffer.has_selection() {
            if let Some((start, end)) = gtk_buffer.selection_bounds() {
                let selected_text = gtk_buffer.text(&start, &end, false);
                text_entry.set_text(&selected_text);
            }
        }
        
        // Add to grid
        grid.attach(&url_label, 0, 0, 1, 1);
        grid.attach(&url_entry, 1, 0, 1, 1);
        grid.attach(&text_label, 0, 1, 1, 1);
        grid.attach(&text_entry, 1, 1, 1, 1);
        
        // Add grid to dialog
        dialog.content_area().append(&grid);
        
        // Focus on URL entry
        url_entry.grab_focus();
        
        // Connect response
        let buffer_clone = self.source_buffer.clone();
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                let url = url_entry.text();
                let text = text_entry.text();
                
                if !url.is_empty() {
                    let link_text = if text.is_empty() { url.clone() } else { text };
                    let link_markdown = format!("[{}]({})", link_text, url);
                    
                    // Insert the link
                    let gtk_buffer = buffer_clone.upcast_ref::<gtk4::TextBuffer>();
                    if gtk_buffer.has_selection() {
                        // Replace selected text
                        if let Some((mut start, mut end)) = gtk_buffer.selection_bounds() {
                            buffer_clone.delete(&mut start, &mut end);
                            let insert_mark = gtk_buffer.get_insert();
                            let mut insert_iter = gtk_buffer.iter_at_mark(&insert_mark);
                            buffer_clone.insert(&mut insert_iter, &link_markdown);
                        }
                    } else {
                        // Insert at cursor
                        let cursor_mark = gtk_buffer.get_insert();
                        let mut cursor_iter = gtk_buffer.iter_at_mark(&cursor_mark);
                        buffer_clone.insert(&mut cursor_iter, &link_markdown);
                    }
                }
            }
            dialog.close();
        });
        
        dialog.present();
    }

    fn show_image_dialog(&self) {
        // Create file chooser dialog
        let dialog = FileChooserDialog::new(
            Some("Select Image"),
            None::<&gtk4::Window>,
            FileChooserAction::Open,
            &[("Cancel", ResponseType::Cancel), ("Open", ResponseType::Accept)],
        );
        
        // Add image file filters
        let filter = FileFilter::new();
        filter.set_name(Some("Image Files"));
        filter.add_mime_type("image/*");
        filter.add_pattern("*.png");
        filter.add_pattern("*.jpg");
        filter.add_pattern("*.jpeg");
        filter.add_pattern("*.gif");
        filter.add_pattern("*.bmp");
        filter.add_pattern("*.svg");
        filter.add_pattern("*.webp");
        dialog.add_filter(&filter);
        
        let all_filter = FileFilter::new();
        all_filter.set_name(Some("All Files"));
        all_filter.add_pattern("*");
        dialog.add_filter(&all_filter);
        
        // Connect response
        let buffer_clone = self.source_buffer.clone();
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        let path_str = path.to_string_lossy();
                        
                        // Get filename for alt text
                        let filename = path.file_name()
                            .and_then(|f| f.to_str())
                            .unwrap_or("image");
                        
                        let image_markdown = format!("![{}]({})", filename, path_str);
                        
                        // Insert the image
                        let gtk_buffer = buffer_clone.upcast_ref::<gtk4::TextBuffer>();
                        let cursor_mark = gtk_buffer.get_insert();
                        let mut cursor_iter = gtk_buffer.iter_at_mark(&cursor_mark);
                        buffer_clone.insert(&mut cursor_iter, &image_markdown);
                    }
                }
            }
            dialog.close();
        });
        
        dialog.present();
    }

    pub fn new_file(&self) {
        self.source_buffer.set_text("");
        *self.current_file.borrow_mut() = None;
    }

    pub fn open_file_from_menu(&self, window: &gtk4::Window) {
        self.show_open_dialog(Some(window));
    }

    pub fn save_file_from_menu(&self, window: &gtk4::Window) {
        self.save_current_file(Some(window));
    }

    pub fn save_as_file_from_menu(&self, window: &gtk4::Window) {
        self.show_save_as_dialog(Some(window));
    }

    fn show_open_dialog(&self, parent: Option<&gtk4::Window>) {
        let dialog = FileChooserDialog::new(
            Some("Open File"),
            parent,
            FileChooserAction::Open,
            &[("Cancel", ResponseType::Cancel), ("Open", ResponseType::Accept)],
        );

        let source_buffer = self.source_buffer.clone();
        let current_file = self.current_file.clone();
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            source_buffer.set_text(&content);
                            *current_file.borrow_mut() = Some(path);
                        }
                    }
                }
            }
            dialog.close();
        });

        dialog.present();
    }

    fn save_current_file(&self, parent: Option<&gtk4::Window>) {
        if let Some(path) = self.current_file.borrow().clone() {
            // Save to existing file
            let start = self.source_buffer.start_iter();
            let end = self.source_buffer.end_iter();
            let text = self.source_buffer.text(&start, &end, false);
            let _ = std::fs::write(&path, text);
        } else {
            // No file selected, show save as dialog
            self.show_save_as_dialog(parent);
        }
    }

    fn show_save_as_dialog(&self, parent: Option<&gtk4::Window>) {
        let dialog = FileChooserDialog::new(
            Some("Save File"),
            parent,
            FileChooserAction::Save,
            &[("Cancel", ResponseType::Cancel), ("Save", ResponseType::Accept)],
        );

        let source_buffer = self.source_buffer.clone();
        let current_file = self.current_file.clone();
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        let start = source_buffer.start_iter();
                        let end = source_buffer.end_iter();
                        let text = source_buffer.text(&start, &end, false);
                        if std::fs::write(&path, text).is_ok() {
                            *current_file.borrow_mut() = Some(path);
                        }
                    }
                }
            }
            dialog.close();
        });

        dialog.present();
    }

    /// Set up keyboard shortcuts for common formatting operations
    fn setup_keyboard_shortcuts(&self) {
        let controller = EventControllerKey::new();
        let editor_clone = self.clone();
        
        controller.connect_key_pressed(move |_, keyval, _, state| {
            // Check for Ctrl modifier
            if state.contains(gdk::ModifierType::CONTROL_MASK) {
                match keyval {
                    gdk::Key::b => {
                        // Ctrl+B for bold
                        editor_clone.insert_bold();
                        return glib::Propagation::Stop;
                    },
                    gdk::Key::i => {
                        // Ctrl+I for italic
                        editor_clone.insert_italic();
                        return glib::Propagation::Stop;
                    },
                    gdk::Key::u => {
                        // Ctrl+U for underline (strikethrough instead)
                        editor_clone.insert_strikethrough();
                        return glib::Propagation::Stop;
                    },
                    gdk::Key::k => {
                        // Ctrl+K for link
                        editor_clone.insert_link();
                        return glib::Propagation::Stop;
                    },
                    gdk::Key::_1 => {
                        // Ctrl+1 for heading 1
                        editor_clone.insert_heading(1);
                        return glib::Propagation::Stop;
                    },
                    gdk::Key::_2 => {
                        // Ctrl+2 for heading 2
                        editor_clone.insert_heading(2);
                        return glib::Propagation::Stop;
                    },
                    gdk::Key::_3 => {
                        // Ctrl+3 for heading 3
                        editor_clone.insert_heading(3);
                        return glib::Propagation::Stop;
                    },
                    gdk::Key::_4 => {
                        // Ctrl+4 for heading 4
                        editor_clone.insert_heading(4);
                        return glib::Propagation::Stop;
                    },
                    gdk::Key::_5 => {
                        // Ctrl+5 for heading 5
                        editor_clone.insert_heading(5);
                        return glib::Propagation::Stop;
                    },
                    gdk::Key::_6 => {
                        // Ctrl+6 for heading 6
                        editor_clone.insert_heading(6);
                        return glib::Propagation::Stop;
                    },
                    gdk::Key::grave => {
                        // Ctrl+` for inline code
                        editor_clone.insert_inline_code();
                        return glib::Propagation::Stop;
                    },
                    _ => {}
                }
                
                // Check for Ctrl+Shift combinations
                if state.contains(gdk::ModifierType::SHIFT_MASK) {
                    match keyval {
                        gdk::Key::period => {
                            // Ctrl+Shift+. for blockquote
                            editor_clone.insert_blockquote();
                            return glib::Propagation::Stop;
                        },
                        gdk::Key::_8 => {
                            // Ctrl+Shift+8 for unordered list
                            editor_clone.insert_bullet_list();
                            return glib::Propagation::Stop;
                        },
                        gdk::Key::_7 => {
                            // Ctrl+Shift+7 for ordered list
                            editor_clone.insert_numbered_list();
                            return glib::Propagation::Stop;
                        },
                        _ => {}
                    }
                }
            }
            
            // Pass through unhandled keys
            glib::Propagation::Proceed
        });
        
        self.source_view.add_controller(controller);
    }
}