use gtk4::prelude::*;
use gtk4::{
    Button, FileChooserAction, FileChooserDialog, HeaderBar, Label, Orientation,
    Paned, ResponseType, ScrolledWindow, Widget, Dialog, Entry, Grid, FileFilter,
};
use sourceview5::prelude::*;
use sourceview5::{Buffer, LanguageManager, StyleSchemeManager, View};
use std::cell::RefCell;
use std::rc::Rc;
use crate::preview::MarkdownPreview;

#[derive(Clone)]
pub struct MarkdownEditor {
    widget: Paned,
    source_view: View,
    preview: MarkdownPreview,
    source_buffer: Buffer,
    current_file: Rc<RefCell<Option<std::path::PathBuf>>>,
    footer_callbacks: Rc<RefCell<Vec<Box<dyn Fn(&str, usize, usize, usize, usize)>>>>,
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

        let editor = Self {
            widget: paned,
            source_view,
            preview,
            source_buffer,
            current_file,
            footer_callbacks,
        };

        // Connect text change signal
        editor.connect_text_changed();
        editor.connect_cursor_moved();

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

    fn update_footer(&self) {
        let start = self.source_buffer.start_iter();
        let end = self.source_buffer.end_iter();
        let text = self.source_buffer.text(&start, &end, false);
        
        // Calculate statistics
        let char_count = text.chars().count();
        let word_count = text.split_whitespace().count();
        
        // Get cursor position - use the GTK TextBuffer's get_insert mark
        let gtk_buffer = self.source_buffer.upcast_ref::<gtk4::TextBuffer>();
        let cursor_iter = gtk_buffer.iter_at_mark(&gtk_buffer.get_insert());
        let line = cursor_iter.line() + 1;
        let column = cursor_iter.line_offset() + 1;
        
        // Call all footer callbacks
        for callback in self.footer_callbacks.borrow().iter() {
            callback(&text, word_count, char_count, line as usize, column as usize);
        }
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

    pub fn create_header_bar(&self) -> HeaderBar {
        let header_bar = HeaderBar::new();
        header_bar.set_title_widget(Some(&Label::new(Some("Marco - Markdown Composer"))));

        // New button
        let new_button = Button::with_label("New");
        let source_buffer = self.source_buffer.clone();
        let current_file = self.current_file.clone();
        new_button.connect_clicked(move |_| {
            source_buffer.set_text("");
            *current_file.borrow_mut() = None;
        });
        header_bar.pack_start(&new_button);

        // Open button
        let open_button = Button::with_label("Open");
        let source_buffer = self.source_buffer.clone();
        let current_file = self.current_file.clone();
        open_button.connect_clicked(move |button| {
            let dialog = FileChooserDialog::new(
                Some("Open File"),
                button.root().and_then(|r| r.downcast::<gtk4::Window>().ok()).as_ref(),
                FileChooserAction::Open,
                &[("Cancel", ResponseType::Cancel), ("Open", ResponseType::Accept)],
            );

            let source_buffer = source_buffer.clone();
            let current_file = current_file.clone();
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
        });
        header_bar.pack_start(&open_button);

        // Save button
        let save_button = Button::with_label("Save");
        let source_buffer = self.source_buffer.clone();
        let current_file_for_save = self.current_file.clone();
        save_button.connect_clicked(move |button| {
            let current_file_borrowed = current_file_for_save.borrow().clone();
            
            if let Some(path) = current_file_borrowed {
                // Save to existing file
                let start = source_buffer.start_iter();
                let end = source_buffer.end_iter();
                let text = source_buffer.text(&start, &end, false);
                let _ = std::fs::write(&path, text);
            } else {
                // Show save dialog
                let dialog = FileChooserDialog::new(
                    Some("Save File"),
                    button.root().and_then(|r| r.downcast::<gtk4::Window>().ok()).as_ref(),
                    FileChooserAction::Save,
                    &[("Cancel", ResponseType::Cancel), ("Save", ResponseType::Accept)],
                );

                let source_buffer = source_buffer.clone();
                let current_file_for_dialog = current_file_for_save.clone();
                dialog.connect_response(move |dialog, response| {
                    if response == ResponseType::Accept {
                        if let Some(file) = dialog.file() {
                            if let Some(path) = file.path() {
                                let start = source_buffer.start_iter();
                                let end = source_buffer.end_iter();
                                let text = source_buffer.text(&start, &end, false);
                                if std::fs::write(&path, text).is_ok() {
                                    *current_file_for_dialog.borrow_mut() = Some(path);
                                }
                            }
                        }
                    }
                    dialog.close();
                });

                dialog.present();
            }
        });
        header_bar.pack_start(&save_button);

        header_bar
    }

    pub fn new_file(&self) {
        self.source_buffer.set_text("");
        *self.current_file.borrow_mut() = None;
    }

    pub fn open_file(&self, button: &Button) {
        let window = button.root().and_then(|r| r.downcast::<gtk4::Window>().ok());
        self.show_open_dialog(window.as_ref());
    }

    pub fn open_file_from_menu(&self, window: &gtk4::Window) {
        self.show_open_dialog(Some(window));
    }

    pub fn save_file(&self, button: &Button) {
        let window = button.root().and_then(|r| r.downcast::<gtk4::Window>().ok());
        self.save_current_file(window.as_ref());
    }

    pub fn save_file_from_menu(&self, window: &gtk4::Window) {
        self.save_current_file(Some(window));
    }

    pub fn save_as_file(&self, button: &Button) {
        let window = button.root().and_then(|r| r.downcast::<gtk4::Window>().ok());
        self.show_save_as_dialog(window.as_ref());
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

    fn insert_text_at_cursor(&self, text: &str) {
        let gtk_buffer = self.source_buffer.upcast_ref::<gtk4::TextBuffer>();
        let cursor_mark = gtk_buffer.get_insert();
        let mut cursor_iter = gtk_buffer.iter_at_mark(&cursor_mark);
        
        self.source_buffer.insert(&mut cursor_iter, text);
    }

    fn toggle_format(&self, prefix: &str, suffix: &str, _default_text: &str) {
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        
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
        } else {
            // No selection - just insert empty formatting tags
            let template = format!("{}{}", prefix, suffix);
            let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
            let cursor_mark = gtk_buffer.get_insert();
            let mut cursor_iter = gtk_buffer.iter_at_mark(&cursor_mark);
            
            buffer.begin_user_action();
            buffer.insert(&mut cursor_iter, &template);
            
            // Move cursor between the tags
            let new_cursor_mark = gtk_buffer.get_insert();
            let mut new_cursor_iter = gtk_buffer.iter_at_mark(&new_cursor_mark);
            new_cursor_iter.backward_chars(suffix.len() as i32);
            gtk_buffer.place_cursor(&new_cursor_iter);
            
            buffer.end_user_action();
        }
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

    fn wrap_selection_or_insert(&self, prefix: &str, suffix: &str, default_text: &str) {
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        
        if gtk_buffer.has_selection() {
            // Get selection bounds
            if let Some((start, end)) = gtk_buffer.selection_bounds() {
                let selected_text = gtk_buffer.text(&start, &end, false);
                
                // Replace selection with wrapped text
                let wrapped_text = format!("{}{}{}", prefix, selected_text, suffix);
                
                buffer.begin_user_action();
                
                // Get fresh bounds for the operation
                if let Some((mut start_iter, mut end_iter)) = gtk_buffer.selection_bounds() {
                    buffer.delete(&mut start_iter, &mut end_iter);
                    
                    // Get a fresh iterator at the insertion point
                    let insert_mark = gtk_buffer.get_insert();
                    let mut insert_iter = gtk_buffer.iter_at_mark(&insert_mark);
                    buffer.insert(&mut insert_iter, &wrapped_text);
                }
                
                buffer.end_user_action();
            }
        } else {
            // No selection, insert template with default text
            let template = format!("{}{}{}", prefix, default_text, suffix);
            self.insert_text_at_cursor(&template);
        }
    }

    pub fn insert_horizontal_rule(&self) {
        self.insert_at_new_line("---\n");
    }

    fn insert_at_line_start(&self, text: &str) {
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        
        // Get current cursor position
        let cursor_mark = gtk_buffer.get_insert();
        let cursor_iter = gtk_buffer.iter_at_mark(&cursor_mark);
        
        // Move to start of current line
        let mut line_start = cursor_iter;
        line_start.set_line_offset(0);
        
        buffer.insert(&mut line_start, text);
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
}