use gtk4::prelude::*;
use gtk4::{
    Button, FileChooserAction, FileChooserDialog, HeaderBar, Label, Orientation,
    Paned, ResponseType, ScrolledWindow, TextView, Widget,
};
use sourceview5::prelude::*;
use sourceview5::{Buffer, LanguageManager, StyleSchemeManager, View};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct MarkdownEditor {
    widget: Paned,
    source_view: View,
    preview_view: TextView,
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

        // Create preview view
        let preview_view = TextView::new();
        preview_view.set_editable(false);
        preview_view.set_cursor_visible(false);

        // Create scrolled windows
        let source_scroll = ScrolledWindow::new();
        source_scroll.set_child(Some(&source_view));
        source_scroll.set_vexpand(true);
        source_scroll.set_size_request(200, -1); // Minimum width of 200px

        let preview_scroll = ScrolledWindow::new();
        preview_scroll.set_child(Some(&preview_view));
        preview_scroll.set_vexpand(true);
        preview_scroll.set_size_request(200, -1); // Minimum width of 200px

        // Add to paned
        paned.set_start_child(Some(&source_scroll));
        paned.set_end_child(Some(&preview_scroll));

        // Clamp split position logic
        // We'll clamp the position between min and max (e.g., 200px and total_width-200px)
        let paned_clone = paned.clone();
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
            preview_view,
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
        let preview_view = self.preview_view.clone();
        let footer_callbacks = self.footer_callbacks.clone();
        self.source_buffer.connect_changed(move |buffer| {
            let start = buffer.start_iter();
            let end = buffer.end_iter();
            let text = buffer.text(&start, &end, false);
            // Use your markdown_basic parser for HTML preview
            let html = crate::markdown_basic::MarkdownParser::new().to_html(&text);
            let preview_buffer = preview_view.buffer();
            preview_buffer.set_text(&html);
            
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
        self.insert_text_at_cursor(&format!("{} ", prefix));
    }

    pub fn insert_bold(&self) {
        self.wrap_selection_or_insert("**", "**", "bold text");
    }

    pub fn insert_italic(&self) {
        self.wrap_selection_or_insert("*", "*", "italic text");
    }

    pub fn insert_inline_code(&self) {
        self.wrap_selection_or_insert("`", "`", "code");
    }

    pub fn insert_strikethrough(&self) {
        self.wrap_selection_or_insert("~~", "~~", "strikethrough text");
    }

    pub fn insert_bullet_list(&self) {
        self.insert_text_at_cursor("- ");
    }

    pub fn insert_numbered_list(&self) {
        self.insert_text_at_cursor("1. ");
    }

    pub fn insert_blockquote(&self) {
        self.insert_text_at_cursor("> ");
    }

    pub fn insert_link(&self) {
        self.wrap_selection_or_insert("[", "](url)", "link text");
    }

    pub fn insert_image(&self) {
        self.wrap_selection_or_insert("![", "](image_url)", "alt text");
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

    fn wrap_selection_or_insert(&self, prefix: &str, suffix: &str, default_text: &str) {
        let gtk_buffer = self.source_buffer.upcast_ref::<gtk4::TextBuffer>();
        
        if gtk_buffer.has_selection() {
            // Get selection bounds
            let (start, end) = gtk_buffer.selection_bounds().unwrap();
            let selected_text = gtk_buffer.text(&start, &end, false);
            
            // Replace selection with wrapped text
            let wrapped_text = format!("{}{}{}", prefix, selected_text, suffix);
            gtk_buffer.delete(&mut start.clone(), &mut end.clone());
            self.source_buffer.insert(&mut start.clone(), &wrapped_text);
        } else {
            // No selection, insert template
            let template = format!("{}{}{}", prefix, default_text, suffix);
            self.insert_text_at_cursor(&template);
        }
    }

    pub fn insert_horizontal_rule(&self) {
        self.insert_text_at_cursor("\n---\n");
    }
}