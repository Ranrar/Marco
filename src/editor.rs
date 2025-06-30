use gtk4::prelude::*;
use gtk4::{
    FileChooserAction, FileChooserDialog, HeaderBar, Label, Orientation,
    Paned, ResponseType, ScrolledWindow, Widget, Dialog, Entry, Grid, FileFilter,
    EventControllerKey, gdk, ComboBoxText, Stack,
};
use sourceview5::prelude::*;
use sourceview5::{Buffer, LanguageManager, StyleSchemeManager, View};
use std::cell::RefCell;
use std::rc::Rc;
use crate::view_code::MarkdownCodeView;
use crate::view_html::MarkdownHtmlView;
use crate::code_languages::CodeLanguageManager;
use crate::context_menu::ContextMenu;
use crate::syntax_advanced::ExtraMarkdownSyntax;
use std::collections::HashMap;

#[derive(Clone)]
pub struct MarkdownEditor {
    widget: Paned,
    source_view: View,
    view_stack: Stack,
    html_view: MarkdownHtmlView,
    code_view: MarkdownCodeView,
    source_buffer: Buffer,
    current_file: Rc<RefCell<Option<std::path::PathBuf>>>,
    footer_callbacks: Rc<RefCell<Vec<Box<dyn Fn(&str, usize, usize, usize, usize)>>>>,
    code_language_manager: Rc<RefCell<CodeLanguageManager>>,
    theme_manager: Rc<RefCell<Option<crate::theme::ThemeManager>>>,
    is_modified: Rc<RefCell<bool>>,
    extra_syntax: Rc<RefCell<ExtraMarkdownSyntax>>,
    tag_table: Rc<RefCell<HashMap<String, gtk4::TextTag>>>,
    current_css_theme: Rc<RefCell<String>>,
    context_menu: Rc<RefCell<Option<ContextMenu>>>,
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

        // Create views
        let html_view = MarkdownHtmlView::new();
        let code_view = MarkdownCodeView::new();
        
        // Create a stack to hold both views
        let view_stack = Stack::new();
        view_stack.set_vexpand(true);
        view_stack.add_named(html_view.widget(), Some("html"));
        view_stack.add_named(code_view.widget(), Some("code"));
        view_stack.set_visible_child_name("html"); // Default to HTML view

        // Create scrolled window for source view
        let source_scroll = ScrolledWindow::new();
        source_scroll.set_child(Some(&source_view));
        source_scroll.set_vexpand(true);
        source_scroll.set_size_request(200, -1); // Minimum width of 200px

        // Add to paned
        paned.set_start_child(Some(&source_scroll));
        paned.set_end_child(Some(&view_stack));

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
        let is_modified = Rc::new(RefCell::new(false));
        let extra_syntax = Rc::new(RefCell::new(ExtraMarkdownSyntax::new()));
        let tag_table = Rc::new(RefCell::new(HashMap::new()));

        let editor = Self {
            widget: paned,
            source_view,
            view_stack,
            html_view,
            code_view,
            source_buffer,
            current_file,
            footer_callbacks,
            code_language_manager,
            theme_manager: Rc::new(RefCell::new(None)),
            is_modified,
            extra_syntax,
            tag_table,
            current_css_theme: Rc::new(RefCell::new("standard".to_string())),
            context_menu: Rc::new(RefCell::new(None)),
        };

        // Connect text change signal
        editor.connect_text_changed();
        editor.connect_cursor_moved();
        
        // Set up keyboard shortcuts
        editor.setup_keyboard_shortcuts();
        
        // Set up right-click context menu
        let context_menu = ContextMenu::new(&editor);
        context_menu.setup_gesture(&editor);
        
        // Store the context menu reference for keyboard access
        *editor.context_menu.borrow_mut() = Some(context_menu);

        // Initialize with standard CSS theme
        editor.set_css_theme("standard");

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

    /// Get access to the source view for context menu integration
    pub fn source_view(&self) -> &View {
        &self.source_view
    }

    /// Get access to the source buffer for context menu integration  
    pub fn source_buffer(&self) -> &Buffer {
        &self.source_buffer
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
        let html_view = self.html_view.clone();
        let code_view = self.code_view.clone();
        let footer_callbacks = self.footer_callbacks.clone();
        let is_modified = self.is_modified.clone();
        let extra_syntax = self.extra_syntax.clone();
        let tag_table = self.tag_table.clone();
        let update_timer: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
        
        self.source_buffer.connect_changed(move |buffer| {
            // Mark document as modified
            let was_modified = *is_modified.borrow();
            *is_modified.borrow_mut() = true;
            if !was_modified {
                println!("DEBUG: Document marked as modified");
            }
            
            let start = buffer.start_iter();
            let end = buffer.end_iter();
            let text = buffer.text(&start, &end, false);
            
            // Apply extra syntax highlighting
            {
                let extra_syntax_ref = extra_syntax.borrow();
                let mut tag_table_ref = tag_table.borrow_mut();
                extra_syntax_ref.apply_extra_syntax_highlighting(buffer, &text, &mut tag_table_ref);
            }
            
            // Update footer statistics immediately (no delay needed for stats)
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
            
            // Debounce the view updates to prevent constant WebView reloads
            let html_view_clone = html_view.clone();
            let code_view_clone = code_view.clone();
            let text_clone = text.to_string();
            let timer_ref = update_timer.clone();
            
            // Cancel any existing timer
            if let Some(timer_id) = timer_ref.borrow_mut().take() {
                timer_id.remove();
            }
            
            // Set a new timer for 800ms delay (longer to reduce updates)
            let new_timer = glib::timeout_add_local(std::time::Duration::from_millis(800), move || {
                html_view_clone.update_content(&text_clone);
                code_view_clone.update_content(&text_clone);
                *timer_ref.borrow_mut() = None;
                glib::ControlFlow::Break
            });
            
            *update_timer.borrow_mut() = Some(new_timer);
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
        self.toggle_format_selection_only_with_dialog("**", "**", "bold");
    }

    pub fn insert_italic(&self) {
        self.toggle_format_selection_only_with_dialog("*", "*", "italic");
    }

    pub fn insert_inline_code(&self) {
        self.toggle_format_selection_only_with_dialog("`", "`", "inline code");
    }

    pub fn insert_strikethrough(&self) {
        self.toggle_format_selection_only_with_dialog("~~", "~~", "strikethrough");
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

    pub fn insert_single_definition(&self) {
        self.insert_at_new_line("Term\n: Definition of the term.\n");
    }

    pub fn insert_custom_definition_list(&self, count: usize) {
        let mut definition_list = String::new();
        for i in 1..=count {
            if i > 1 {
                definition_list.push('\n');
            }
            definition_list.push_str(&format!("Term {}\n: Definition of term {}.\n", i, i));
        }
        self.insert_at_new_line(&definition_list);
    }

    pub fn insert_highlight(&self) {
        self.toggle_format_selection_only_with_dialog("==", "==", "highlight");
    }

    pub fn insert_subscript(&self) {
        self.toggle_format_selection_only_with_dialog("~", "~", "subscript");
    }

    pub fn insert_superscript(&self) {
        self.toggle_format_selection_only_with_dialog("^", "^", "superscript");
    }

    #[allow(dead_code)]
    pub fn insert_emoji(&self) {
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        let cursor_mark = gtk_buffer.get_insert();
        let mut cursor_iter = gtk_buffer.iter_at_mark(&cursor_mark);
        
        // Insert some common emoji shortcodes as examples
        buffer.insert(&mut cursor_iter, ":smile: :heart: :thumbsup:");
    }

    pub fn insert_emoji_text(&self, emoji: &str) {
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        let cursor_mark = gtk_buffer.get_insert();
        let mut cursor_iter = gtk_buffer.iter_at_mark(&cursor_mark);
        
        buffer.insert(&mut cursor_iter, emoji);
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

    pub fn insert_text_at_cursor(&self, text: &str) {
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

    /// Show a dialog notifying the user that text must be selected for the formatting function
    fn show_text_selection_required_dialog(&self, parent: &gtk4::Window, feature_name: &str) {
        use gtk4::MessageDialog;
        
        let title = "Text Selection Required";
        let message = format!("Please select text in the editor before applying {} formatting.", feature_name);
        
        let dialog = MessageDialog::builder()
            .transient_for(parent)
            .modal(true)
            .message_type(gtk4::MessageType::Info)
            .text(title)
            .secondary_text(&message)
            .build();
        
        dialog.add_button("OK", gtk4::ResponseType::Ok);
        dialog.set_default_response(gtk4::ResponseType::Ok);
        
        dialog.connect_response(|dialog, _| {
            dialog.close();
        });
        
        dialog.present();
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
                
                // Smart detection: check if this specific formatting is present
                let (replacement_text, new_selection_length) = if self.has_specific_formatting(selected_str, prefix, suffix) {
                    // Remove this specific formatting
                    let inner_text = self.remove_specific_formatting(selected_str, prefix, suffix);
                    let new_length = inner_text.len();
                    (inner_text, new_length)
                } else {
                    // Add this formatting
                    let formatted_text = format!("{}{}{}", prefix, selected_str, suffix);
                    let new_length = formatted_text.len();
                    (formatted_text, new_length)
                };
                
                // Use begin/end user action for atomic operation
                buffer.begin_user_action();
                
                // Store the start position for re-selection
                let start_offset = start.offset();
                
                // Get fresh bounds for the operation
                if let Some((mut start_iter, mut end_iter)) = gtk_buffer.selection_bounds() {
                    // Replace the selected text with the formatted/unformatted version
                    buffer.delete(&mut start_iter, &mut end_iter);
                    
                    // Get a fresh iterator at the insertion point
                    let insert_mark = gtk_buffer.get_insert();
                    let mut insert_iter = gtk_buffer.iter_at_mark(&insert_mark);
                    buffer.insert(&mut insert_iter, &replacement_text);
                    
                    // Restore selection with the new text
                    let new_start_iter = gtk_buffer.iter_at_offset(start_offset);
                    let new_end_iter = gtk_buffer.iter_at_offset(start_offset + new_selection_length as i32);
                    gtk_buffer.select_range(&new_start_iter, &new_end_iter);
                }
                
                buffer.end_user_action();
            }
        }
        // If no text is selected, do nothing (for backward compatibility)
    }
    
    fn toggle_format_selection_only_with_dialog(&self, prefix: &str, suffix: &str, feature_name: &str) {
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        
        // Only work if text is selected
        if gtk_buffer.has_selection() {
            // First, try to expand selection to include formatting if user selected inner text
            let (start, end, selected_text) = if let Some((expanded_start, expanded_end, expanded_text)) = self.expand_selection_to_include_formatting() {
                // User selected inner text, but we found surrounding formatting
                gtk_buffer.select_range(&expanded_start, &expanded_end);
                (expanded_start, expanded_end, expanded_text)
            } else {
                // Use original selection
                let (start, end) = gtk_buffer.selection_bounds().unwrap();
                let selected_text = gtk_buffer.text(&start, &end, false).to_string();
                (start, end, selected_text)
            };
            
            // Smart detection and formatting application
            let (replacement_text, new_selection_length) = if self.has_specific_formatting(&selected_text, prefix, suffix) {
                // Remove this specific formatting
                let inner_text = self.remove_specific_formatting(&selected_text, prefix, suffix);
                let new_length = inner_text.len();
                (inner_text, new_length)
            } else {
                // Apply smart formatting (handles bold/italic combinations)
                let formatted_text = self.apply_smart_formatting(&selected_text, prefix, suffix);
                let new_length = formatted_text.len();
                (formatted_text, new_length)
            };
            
            // Use begin/end user action for atomic operation
            buffer.begin_user_action();
            
            // Store the start position for re-selection
            let start_offset = start.offset();
            
            // Get fresh bounds for the operation (use the potentially expanded selection)
            let mut start_iter = gtk_buffer.iter_at_offset(start.offset());
            let mut end_iter = gtk_buffer.iter_at_offset(end.offset());
            
            // Replace the selected text with the formatted/unformatted version
            buffer.delete(&mut start_iter, &mut end_iter);
            
            // Get a fresh iterator at the insertion point
            let insert_mark = gtk_buffer.get_insert();
            let mut insert_iter = gtk_buffer.iter_at_mark(&insert_mark);
            buffer.insert(&mut insert_iter, &replacement_text);
            
            // Restore selection with the new text
            let new_start_iter = gtk_buffer.iter_at_offset(start_offset);
            let new_end_iter = gtk_buffer.iter_at_offset(start_offset + new_selection_length as i32);
            gtk_buffer.select_range(&new_start_iter, &new_end_iter);
            
            buffer.end_user_action();
        } else {
            // Show error dialog if no text is selected
            if let Some(window) = self.source_view().root()
                .and_then(|w| w.downcast::<gtk4::Window>().ok()) {
                self.show_text_selection_required_dialog(&window, feature_name);
            }
        }
    }

    /// Check if the text has this specific formatting (handles mixed formatting and overlapping patterns)
    fn has_specific_formatting(&self, text: &str, prefix: &str, suffix: &str) -> bool {
        // Special handling for combined bold+italic format (***text***)
        if text.starts_with("***") && text.ends_with("***") && text.len() > 6 {
            // This text has both bold and italic
            if (prefix == "**" && suffix == "**") || (prefix == "*" && suffix == "*") {
                return true; // Both bold and italic formatting are present
            }
        }
        
        // Method 1: Check if the text is exactly the format (e.g., "**bold**")
        if text.starts_with(prefix) && text.ends_with(suffix) && text.len() > prefix.len() + suffix.len() {
            // Special handling for bold vs italic conflict
            if prefix == "*" && suffix == "*" {
                // If we're checking for italic (*) but text starts with **, this is bold, not italic
                if text.starts_with("**") {
                    return false;
                }
            }
            return true;
        }
        
        // Method 2: Check if this specific formatting exists as the outermost layer
        if text.len() > prefix.len() + suffix.len() {
            let potential_start = &text[..prefix.len()];
            let potential_end = &text[text.len() - suffix.len()..];
            
            if potential_start == prefix && potential_end == suffix {
                // Special handling for bold vs italic conflict
                if prefix == "*" && suffix == "*" {
                    // If we're checking for italic (*) but text starts with **, this is bold, not italic
                    if text.starts_with("**") {
                        return false;
                    }
                }
                return true;
            }
        }
        
        false
    }

    /// Remove specific formatting while preserving other formatting (handles overlapping patterns)
    fn remove_specific_formatting(&self, text: &str, prefix: &str, suffix: &str) -> String {
        // Special handling for combined bold+italic format (***text***)
        if text.starts_with("***") && text.ends_with("***") && text.len() > 6 {
            let inner_text = &text[3..text.len() - 3];
            if prefix == "**" && suffix == "**" {
                // Removing bold from ***text*** leaves *text* (italic only)
                return format!("*{}*", inner_text);
            } else if prefix == "*" && suffix == "*" {
                // Removing italic from ***text*** leaves **text** (bold only) 
                return format!("**{}**", inner_text);
            }
        }
        
        // Method 1: Direct match - just strip the outer layer
        if text.starts_with(prefix) && text.ends_with(suffix) && text.len() > prefix.len() + suffix.len() {
            // Special handling for bold vs italic conflict
            if prefix == "*" && suffix == "*" {
                // If we're trying to remove italic (*) but text starts with **, this is bold, not italic
                if text.starts_with("**") {
                    return text.to_string(); // Don't remove, this isn't italic
                }
            }
            return text[prefix.len()..text.len() - suffix.len()].to_string();
        }
        
        // Method 2: Check for outermost layer removal
        if text.len() > prefix.len() + suffix.len() {
            let potential_start = &text[..prefix.len()];
            let potential_end = &text[text.len() - suffix.len()..];
            
            if potential_start == prefix && potential_end == suffix {
                // Special handling for bold vs italic conflict
                if prefix == "*" && suffix == "*" {
                    // If we're trying to remove italic (*) but text starts with **, this is bold, not italic
                    if text.starts_with("**") {
                        return text.to_string(); // Don't remove, this isn't italic
                    }
                }
                return text[prefix.len()..text.len() - suffix.len()].to_string();
            }
        }
        
        // If we can't find the specific formatting, return as-is
        text.to_string()
    }

    /// Expand selection to include formatting if user selected the inner text
    fn expand_selection_to_include_formatting(&self) -> Option<(gtk4::TextIter, gtk4::TextIter, String)> {
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        
        if !gtk_buffer.has_selection() {
            return None;
        }
        
        let (start, end) = gtk_buffer.selection_bounds()?;
        let selected_text = gtk_buffer.text(&start, &end, false).to_string();
        
        // Get some context around the selection to check for formatting
        let mut expanded_start = start;
        let mut expanded_end = end;
        
        // Look backward for potential formatting markers
        for _ in 0..10 { // Look up to 10 characters back
            if !expanded_start.backward_char() {
                break;
            }
        }
        
        // Look forward for potential formatting markers  
        for _ in 0..10 { // Look up to 10 characters forward
            if !expanded_end.forward_char() {
                break;
            }
        }
        
        let expanded_text = gtk_buffer.text(&expanded_start, &expanded_end, false).to_string();
        
        // Check for formatting patterns around the selection - ORDER MATTERS!
        // Check longer patterns first to avoid substring conflicts
        let formatting_patterns = [
            ("**", "**"), // Bold (check before italic to avoid ** vs * conflict)
            ("~~", "~~"), // Strikethrough  
            ("==", "=="), // Highlight
            ("`", "`"),   // Code
            ("*", "*"),   // Italic (check after bold)
            ("~", "~"),   // Subscript
            ("^", "^"),   // Superscript
        ];
        
        // SPECIAL CASE: Check for combined bold+italic format (***text***) FIRST
        if let Some(selection_pos) = expanded_text.find(&selected_text) {
            let before_selection = &expanded_text[..selection_pos];
            let after_selection = &expanded_text[selection_pos + selected_text.len()..];
            
            if before_selection.ends_with("***") && after_selection.starts_with("***") {
                // This is combined format, expand to include all three asterisks on each side
                let format_start_pos = selection_pos - 3;
                let format_end_pos = selection_pos + selected_text.len() + 3;
                
                let new_start = gtk_buffer.iter_at_offset(expanded_start.offset() + format_start_pos as i32);
                let new_end = gtk_buffer.iter_at_offset(expanded_start.offset() + format_end_pos as i32);
                let formatted_text = gtk_buffer.text(&new_start, &new_end, false).to_string();
                
                return Some((new_start, new_end, formatted_text));
            }
        }
        
        for (prefix, suffix) in &formatting_patterns {
            // Find where our original selection fits in the expanded text
            if let Some(selection_pos) = expanded_text.find(&selected_text) {
                let before_selection = &expanded_text[..selection_pos];
                let after_selection = &expanded_text[selection_pos + selected_text.len()..];
                
                // Check if selection is surrounded by this formatting
                if before_selection.ends_with(prefix) && after_selection.starts_with(suffix) {
                    // For overlapping patterns like ** and *, make sure we don't have a longer pattern
                    if *prefix == "*" && *suffix == "*" {
                        // Check if this is actually bold formatting (**)
                        if before_selection.ends_with("**") && after_selection.starts_with("*") {
                            continue; // Skip, this is actually bold formatting
                        }
                    }
                    
                    // Calculate the new bounds that include the formatting
                    let format_start_pos = selection_pos - prefix.len();
                    let format_end_pos = selection_pos + selected_text.len() + suffix.len();
                    
                    let new_start = gtk_buffer.iter_at_offset(expanded_start.offset() + format_start_pos as i32);
                    let new_end = gtk_buffer.iter_at_offset(expanded_start.offset() + format_end_pos as i32);
                    let formatted_text = gtk_buffer.text(&new_start, &new_end, false).to_string();
                    
                    return Some((new_start, new_end, formatted_text));
                }
            }
        }
        
        None
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
        self.mark_as_saved(); // New file is not modified
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
        let is_modified = self.is_modified.clone();
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            source_buffer.set_text(&content);
                            *current_file.borrow_mut() = Some(path);
                            // Mark as not modified since we just loaded from file
                            *is_modified.borrow_mut() = false;
                        }
                    }
                }
            }
            dialog.close();
        });

        dialog.present();
    }

    fn save_current_file(&self, parent: Option<&gtk4::Window>) {
        println!("DEBUG: save_current_file called");
        if let Some(path) = self.current_file.borrow().clone() {
            // Save to existing file
            println!("DEBUG: Saving to existing file: {:?}", path);
            let start = self.source_buffer.start_iter();
            let end = self.source_buffer.end_iter();
            let text = self.source_buffer.text(&start, &end, false);
            if std::fs::write(&path, text).is_ok() {
                println!("DEBUG: File saved successfully, marking as saved");
                self.mark_as_saved(); // Mark as saved after successful write
            } else {
                println!("DEBUG: Failed to save file");
            }
        } else {
            // No file selected, show save as dialog
            println!("DEBUG: No current file, showing save as dialog");
            self.show_save_as_dialog(parent);
        }
    }

    /// Save current file with a callback that's only called on successful save
    fn save_current_file_with_callback<F>(&self, parent: Option<&gtk4::Window>, on_save_complete: F) 
    where
        F: Fn() + 'static,
    {
        println!("DEBUG: save_current_file_with_callback called");
        if let Some(path) = self.current_file.borrow().clone() {
            // Save to existing file
            println!("DEBUG: Saving to existing file: {:?}", path);
            let start = self.source_buffer.start_iter();
            let end = self.source_buffer.end_iter();
            let text = self.source_buffer.text(&start, &end, false);
            if std::fs::write(&path, text).is_ok() {
                println!("DEBUG: File saved successfully, marking as saved and calling callback");
                self.mark_as_saved(); // Mark as saved after successful write
                on_save_complete(); // Call the callback only on successful save
            } else {
                println!("DEBUG: Failed to save file, not calling callback");
            }
        } else {
            // No file selected, show save as dialog with callback
            println!("DEBUG: No current file, showing save as dialog with callback");
            self.show_save_as_dialog_with_callback(parent, on_save_complete);
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
        let is_modified = self.is_modified.clone();
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        let start = source_buffer.start_iter();
                        let end = source_buffer.end_iter();
                        let text = source_buffer.text(&start, &end, false);
                        if std::fs::write(&path, text).is_ok() {
                            *current_file.borrow_mut() = Some(path);
                            // Mark as saved after successful write
                            *is_modified.borrow_mut() = false;
                        }
                    }
                }
            }
            dialog.close();
        });

        dialog.present();
    }

    /// Show save as dialog with a callback that's only called on successful save
    fn show_save_as_dialog_with_callback<F>(&self, parent: Option<&gtk4::Window>, on_save_complete: F)
    where
        F: Fn() + 'static,
    {
        println!("DEBUG: show_save_as_dialog_with_callback called");
        let dialog = FileChooserDialog::new(
            Some("Save File"),
            parent,
            FileChooserAction::Save,
            &[("Cancel", ResponseType::Cancel), ("Save", ResponseType::Accept)],
        );

        let source_buffer = self.source_buffer.clone();
        let current_file = self.current_file.clone();
        let is_modified = self.is_modified.clone();
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                println!("DEBUG: User clicked Save in Save As dialog");
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        let start = source_buffer.start_iter();
                        let end = source_buffer.end_iter();
                        let text = source_buffer.text(&start, &end, false);
                        if std::fs::write(&path, text).is_ok() {
                            println!("DEBUG: File saved successfully via Save As, calling callback");
                            *current_file.borrow_mut() = Some(path);
                            // Mark as saved after successful write
                            *is_modified.borrow_mut() = false;
                            on_save_complete(); // Call the callback only on successful save
                        } else {
                            println!("DEBUG: Failed to save file via Save As, not calling callback");
                        }
                    }
                }
            } else {
                println!("DEBUG: User cancelled Save As dialog, not calling callback");
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
                    gdk::Key::n => {
                        // Ctrl+N for new - need to be handled by application action
                        return glib::Propagation::Proceed;
                    },
                    gdk::Key::o => {
                        // Ctrl+O for open - need to be handled by application action
                        return glib::Propagation::Proceed;
                    },
                    gdk::Key::s => {
                        // Ctrl+S for save (and Ctrl+Shift+S for save as) - need to be handled by application action
                        return glib::Propagation::Proceed;
                    },
                    gdk::Key::q => {
                        // Ctrl+Q for quit - need to be handled by application action
                        return glib::Propagation::Proceed;
                    },
                    gdk::Key::z => {
                        // Ctrl+Z for undo
                        editor_clone.undo();
                        return glib::Propagation::Stop;
                    },
                    gdk::Key::y => {
                        // Ctrl+Y for redo
                        editor_clone.redo();
                        return glib::Propagation::Stop;
                    },
                    gdk::Key::x => {
                        // Ctrl+X for cut
                        editor_clone.cut();
                        return glib::Propagation::Stop;
                    },
                    gdk::Key::c => {
                        // Ctrl+C for copy
                        editor_clone.copy();
                        return glib::Propagation::Stop;
                    },
                    gdk::Key::v => {
                        // Ctrl+V for paste
                        editor_clone.paste();
                        return glib::Propagation::Stop;
                    },
                    gdk::Key::f => {
                        // Ctrl+F for find - need window context, will be handled by menu action
                        return glib::Propagation::Proceed;
                    },
                    gdk::Key::h => {
                        // Ctrl+H for replace - need window context, will be handled by menu action  
                        return glib::Propagation::Proceed;
                    },
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
                    }
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
            
            // Check for Shift+F10 (context menu at cursor)
            if state.contains(gdk::ModifierType::SHIFT_MASK) && keyval == gdk::Key::F10 {
                if let Some(ref context_menu) = *editor_clone.context_menu.borrow() {
                    context_menu.show_at_cursor(&editor_clone);
                    return glib::Propagation::Stop;
                }
            }
            
            // Pass through unhandled keys
            glib::Propagation::Proceed
        });
        
        self.source_view.add_controller(controller);
    }

    /// Switch between HTML and code views
    pub fn set_view_mode(&self, mode: &str) {
        self.view_stack.set_visible_child_name(mode);
    }
    
    /// Get the current view mode
    #[allow(dead_code)]
    pub fn get_view_mode(&self) -> String {
        self.view_stack.visible_child_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "html".to_string())
    }

    /// Set the theme manager for both the HTML view and source editor
    pub fn set_theme_manager(&self, theme_manager: crate::theme::ThemeManager) {
        // Store the theme manager
        *self.theme_manager.borrow_mut() = Some(theme_manager.clone());
        
        // Apply to HTML view
        self.html_view.set_theme_manager(theme_manager.clone());
        
        // Apply to source editor
        self.update_source_editor_theme(&theme_manager);
    }

    /// Update the source editor theme based on the theme manager
    fn update_source_editor_theme(&self, theme_manager: &crate::theme::ThemeManager) {
        let style_manager = StyleSchemeManager::default();
        
        // Choose appropriate style scheme based on theme
        let preferred_schemes = match theme_manager.get_effective_theme() {
            crate::theme::Theme::Light => vec!["Adwaita", "classic", "tango", "kate", "solarized-light"],
            crate::theme::Theme::Dark => vec!["Adwaita-dark", "oblivion", "cobalt", "monokai", "solarized-dark"],
            crate::theme::Theme::System => {
                // For system theme, detect and choose appropriate schemes
                match crate::theme::ThemeManager::detect_system_theme() {
                    crate::theme::Theme::Dark => vec!["Adwaita-dark", "oblivion", "cobalt", "monokai", "solarized-dark"],
                    _ => vec!["Adwaita", "classic", "tango", "kate", "solarized-light"],
                }
            }
        };
        
        // Try to find the first available scheme from the preferred list
        let mut applied_scheme = false;
        for scheme_name in preferred_schemes {
            if let Some(scheme) = style_manager.scheme(scheme_name) {
                self.source_buffer.set_style_scheme(Some(&scheme));
                applied_scheme = true;
                break;
            }
        }
        
        // Ultimate fallback - use default scheme
        if !applied_scheme {
            if let Some(scheme) = style_manager.scheme("Adwaita") {
                self.source_buffer.set_style_scheme(Some(&scheme));
            }
        }
    }

    /// Refresh both the HTML view and source editor (useful after theme changes)
    pub fn refresh_html_view(&self) {
        self.html_view.refresh_with_current_content();
        
        // Also refresh source editor theme if we have a theme manager
        if let Some(ref theme_manager) = *self.theme_manager.borrow() {
            self.update_source_editor_theme(theme_manager);
        }
    }

    /// Check if the document has been modified since last save
    pub fn is_modified(&self) -> bool {
        *self.is_modified.borrow()
    }
    
    /// Mark the document as saved (not modified)
    fn mark_as_saved(&self) {
        println!("DEBUG: Marking document as saved (was modified: {})", self.is_modified());
        *self.is_modified.borrow_mut() = false;
    }

    /// Show save confirmation dialog and handle the response asynchronously
    /// Returns true immediately if no unsaved changes, false if dialog is shown (to prevent immediate quit)
    pub fn show_unsaved_changes_dialog_and_quit<F>(&self, parent: Option<&gtk4::Window>, on_confirm_quit: F) -> bool
    where
        F: Fn() + 'static,
    {
        if !self.is_modified() {
            println!("DEBUG: Document not modified, proceeding immediately");
            return true; // Not modified, safe to proceed immediately
        }

        println!("DEBUG: Document is modified, showing unsaved changes dialog");

        // Create confirmation dialog
        let title = crate::localization::tr("dialog.unsaved_changes.title");
        let message = crate::localization::tr("dialog.unsaved_changes.message");
        let cancel_text = crate::localization::tr("dialog.unsaved_changes.cancel");
        let discard_text = crate::localization::tr("dialog.unsaved_changes.discard");
        let save_text = crate::localization::tr("dialog.unsaved_changes.save");
        
        println!("DEBUG: Dialog strings - Title: '{}', Message: '{}', Cancel: '{}', Discard: '{}', Save: '{}'", 
                 title, message, cancel_text, discard_text, save_text);
        
        let dialog = gtk4::MessageDialog::builder()
            .transient_for(parent.unwrap_or(&gtk4::Window::new()))
            .modal(true)
            .message_type(gtk4::MessageType::Question)
            .text(&title)
            .secondary_text(&message)
            .build();

        dialog.add_button(&cancel_text, ResponseType::Cancel);
        dialog.add_button(&discard_text, ResponseType::No);
        dialog.add_button(&save_text, ResponseType::Yes);

        // Set default response to Save
        dialog.set_default_response(ResponseType::Yes);

        println!("DEBUG: Dialog created with buttons - Cancel: {:?}, Discard: {:?}, Save: {:?}", 
                 ResponseType::Cancel, ResponseType::No, ResponseType::Yes);

        // Handle dialog response asynchronously
        let editor_weak = self.clone();
        let parent_window = parent.map(|w| w.clone());
        
        println!("DEBUG: Setting up dialog response callback");
        
        // Clone the callback for the save case
        let on_confirm_quit_for_save = Rc::new(on_confirm_quit);
        let on_confirm_quit_for_discard = on_confirm_quit_for_save.clone();
        
        // Use a flag to ensure the dialog response is only handled once
        let response_handled = Rc::new(std::cell::RefCell::new(false));
        let response_handled_clone = response_handled.clone();
        
        dialog.connect_response(move |dialog, response| {
            println!("DEBUG: Dialog response received: {:?}", response);
            
            // Check if response was already handled
            if *response_handled_clone.borrow() {
                println!("DEBUG: Dialog response already handled, ignoring");
                return;
            }
            
            match response {
                ResponseType::Yes => {
                    // User wants to save before quitting
                    println!("DEBUG: User clicked Save button");
                    *response_handled_clone.borrow_mut() = true;
                    dialog.close();
                    
                    // Use the callback-based save method to only quit if save is successful
                    let quit_callback = on_confirm_quit_for_save.clone();
                    editor_weak.save_current_file_with_callback(parent_window.as_ref(), move || {
                        println!("DEBUG: Save completed successfully, calling quit callback");
                        quit_callback();
                        println!("DEBUG: on_confirm_quit callback completed");
                    });
                }
                ResponseType::No => {
                    // User wants to discard changes and quit
                    println!("DEBUG: User clicked Don't Save button");
                    *response_handled_clone.borrow_mut() = true;
                    dialog.close();
                    println!("DEBUG: Dialog closed, about to call quit callback");
                    (*on_confirm_quit_for_discard)();
                    println!("DEBUG: on_confirm_quit callback completed");
                }
                ResponseType::Cancel => {
                    // User explicitly clicked Cancel button
                    println!("DEBUG: User clicked Cancel button");
                    *response_handled_clone.borrow_mut() = true;
                    dialog.close();
                }
                ResponseType::DeleteEvent => {
                    // Dialog was closed by window manager (X button) - treat as cancel
                    println!("DEBUG: Dialog closed by window manager, treating as cancel");
                    *response_handled_clone.borrow_mut() = true;
                    dialog.close();
                }
                _ => {
                    // Other responses - treat as cancel
                    println!("DEBUG: Other dialog response: {:?}, treating as cancel", response);
                    *response_handled_clone.borrow_mut() = true;
                    dialog.close();
                }
            }
        });

        // Show the dialog
        println!("DEBUG: Presenting dialog to user");
        dialog.present();
        
        // Return false to indicate that quit should not proceed immediately
        // The actual quit will happen in the dialog response callback
        println!("DEBUG: Returning false - quit should wait for dialog response");
        false
    }

    // Extra Markdown Syntax Methods
    
    /// Insert underlined text
    pub fn insert_underline(&self, text: &str) {
        crate::syntax_advanced::insert_underline(self, text);
    }

    /// Insert centered text
    pub fn insert_center_text(&self, text: &str) {
        crate::syntax_advanced::insert_center_text(self, text);
    }

    /// Insert colored text
    pub fn insert_colored_text(&self, text: &str, color: &str) {
        crate::syntax_advanced::insert_colored_text(self, text, color);
    }

    /// Insert a markdown comment
    pub fn insert_comment(&self, comment: &str) {
        crate::syntax_advanced::insert_comment(self, comment);
    }

    /// Insert an admonition
    pub fn insert_admonition(&self, emoji: &str, adm_type: &str, text: &str) {
        crate::syntax_advanced::insert_admonition(self, emoji, adm_type, text);
    }

    /// Insert image with size
    pub fn insert_image_with_size(&self, src: &str, alt: &str, width: Option<&str>, height: Option<&str>) {
        crate::syntax_advanced::insert_image_with_size(self, src, alt, width, height);
    }

    /// Insert image with caption
    pub fn insert_image_with_caption(&self, src: &str, alt: &str, caption: &str) {
        crate::syntax_advanced::insert_image_with_caption(self, src, alt, caption);
    }

    /// Insert link with target
    pub fn insert_link_with_target(&self, url: &str, text: &str, target: &str) {
        crate::syntax_advanced::insert_link_with_target(self, url, text, target);
    }

    /// Insert HTML entity
    pub fn insert_html_entity(&self, entity: &str) {
        crate::syntax_advanced::insert_html_entity(self, entity);
    }

    /// Insert table of contents
    pub fn insert_table_of_contents(&self) {
        crate::syntax_advanced::insert_table_of_contents(self);
    }

    /// Insert YouTube video embed
    pub fn insert_youtube_video(&self, video_id: &str, alt_text: &str) {
        crate::syntax_advanced::insert_youtube_video(self, video_id, alt_text);
    }

    /// Insert indented text
    pub fn insert_indented_text(&self, text: &str, indent_level: usize) {
        crate::syntax_advanced::insert_indented_text(self, text, indent_level);
    }

    /// Get common HTML entities for UI
    pub fn get_common_html_entities() -> Vec<(&'static str, &'static str, &'static str)> {
        crate::syntax_advanced::get_common_html_entities()
    }

    /// Get common admonition types for UI
    pub fn get_common_admonitions() -> Vec<(&'static str, &'static str, &'static str)> {
        crate::syntax_advanced::get_common_admonitions()
    }

    /// Set the CSS theme for the preview
    pub fn set_css_theme(&self, theme_name: &str) {
        *self.current_css_theme.borrow_mut() = theme_name.to_string();
        
        // Load CSS file from the css/ directory
        let css_path = format!("css/{}.css", theme_name);
        match std::fs::read_to_string(&css_path) {
            Ok(css_content) => {
                self.html_view.set_custom_css(&css_content);
                self.refresh_html_view();
            }
            Err(e) => {
                eprintln!("Failed to load CSS theme '{}': {}", theme_name, e);
                // Fallback to standard theme
                if theme_name != "standard" {
                    self.set_css_theme("standard");
                }
            }
        }
    }
    
    /// Get the current CSS theme name
    pub fn get_current_css_theme(&self) -> String {
        self.current_css_theme.borrow().clone()
    }

    /// Get available CSS themes by scanning the css/ directory
    pub fn get_available_css_themes() -> Vec<(String, String, String)> {
        let mut themes = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir("css") {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if let Some(extension) = path.extension() {
                        if extension == "css" {
                            if let Some(filename) = path.file_stem() {
                                if let Some(theme_name) = filename.to_str() {
                                    // Create display name (capitalize first letter, replace _ with space)
                                    let display_name = theme_name
                                        .replace('_', " ")
                                        .chars()
                                        .enumerate()
                                        .map(|(i, c)| if i == 0 { c.to_uppercase().collect::<String>() } else { c.to_string() })
                                        .collect::<String>();
                                    
                                    // Create sanitized action name (replace spaces and special chars with underscores)
                                    let sanitized_name = theme_name
                                        .chars()
                                        .map(|c| if c.is_alphanumeric() { c } else { '_' })
                                        .collect::<String>();
                                    
                                    themes.push((theme_name.to_string(), display_name, sanitized_name));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Sort themes alphabetically by display name
        themes.sort_by(|a, b| a.1.cmp(&b.1));
        themes
    }

    /// Undo the last action in the buffer
    pub fn undo(&self) {
        if self.source_buffer.can_undo() {
            self.source_buffer.undo();
        }
    }

    /// Redo the last undone action in the buffer
    pub fn redo(&self) {
        if self.source_buffer.can_redo() {
            self.source_buffer.redo();
        }
    }

    /// Cut selected text to clipboard
    pub fn cut(&self) {
        if let Some(display) = gdk::Display::default() {
            let clipboard = display.clipboard();
            self.source_buffer.cut_clipboard(&clipboard, true);
        }
    }

    /// Copy selected text to clipboard
    pub fn copy(&self) {
        if let Some(display) = gdk::Display::default() {
            let clipboard = display.clipboard();
            self.source_buffer.copy_clipboard(&clipboard);
        }
    }

    /// Paste text from clipboard
    pub fn paste(&self) {
        if let Some(display) = gdk::Display::default() {
            let clipboard = display.clipboard();
            self.source_buffer.paste_clipboard(&clipboard, None, true);
        }
    }

    /// Show find dialog
    pub fn show_find_dialog(&self, window: &gtk4::Window) {
        let dialog = Dialog::builder()
            .title("Find")
            .transient_for(window)
            .modal(true)
            .build();

        let content_area = dialog.content_area();
        let grid = Grid::new();
        grid.set_margin_top(12);
        grid.set_margin_bottom(12);
        grid.set_margin_start(12);
        grid.set_margin_end(12);
        grid.set_row_spacing(6);
        grid.set_column_spacing(6);

        let find_label = Label::new(Some("Find:"));
        let find_entry = Entry::new();
        find_entry.set_hexpand(true);
        
        // Add case-sensitive checkbox
        let case_sensitive_check = gtk4::CheckButton::builder()
            .label("Case sensitive")
            .build();

        grid.attach(&find_label, 0, 0, 1, 1);
        grid.attach(&find_entry, 1, 0, 1, 1);
        grid.attach(&case_sensitive_check, 1, 1, 1, 1);

        content_area.append(&grid);

        dialog.add_button("Cancel", ResponseType::Cancel);
        let find_next_button = dialog.add_button("Find Next", ResponseType::Ok);
        find_next_button.set_css_classes(&["suggested-action"]);

        let source_buffer = self.source_buffer.clone();
        let source_view = self.source_view.clone();
        
        // Set focus to the entry
        find_entry.grab_focus();
        
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Ok {
                let entry = dialog
                    .content_area()
                    .first_child()
                    .and_then(|grid| grid.downcast::<Grid>().ok())
                    .and_then(|grid| grid.child_at(1, 0))
                    .and_then(|entry| entry.downcast::<Entry>().ok());

                let case_check = dialog
                    .content_area()
                    .first_child()
                    .and_then(|grid| grid.downcast::<Grid>().ok())
                    .and_then(|grid| grid.child_at(1, 1))
                    .and_then(|check| check.downcast::<gtk4::CheckButton>().ok());

                if let (Some(entry), Some(case_check)) = (entry, case_check) {
                    let search_text = entry.text();
                    if !search_text.is_empty() {
                        // Perform search from cursor position
                        let cursor_mark = source_buffer.get_insert();
                        let cursor_iter = source_buffer.iter_at_mark(&cursor_mark);
                        let end_iter = source_buffer.end_iter();
                        let text = source_buffer.text(&cursor_iter, &end_iter, false);
                        let text_str = text.as_str();
                        let search_str = search_text.as_str();
                        
                        let found_pos = if case_check.is_active() {
                            text_str.find(search_str)
                        } else {
                            text_str.to_lowercase().find(&search_str.to_lowercase())
                        };
                        
                        if let Some(pos) = found_pos {
                            let mut search_start = cursor_iter;
                            search_start.forward_chars(pos as i32);
                            let mut search_end = search_start;
                            search_end.forward_chars(search_str.len() as i32);
                            source_buffer.select_range(&search_start, &search_end);
                            
                            // Scroll to the found text
                            let mut scroll_iter = search_start;
                            source_view.scroll_to_iter(&mut scroll_iter, 0.0, false, 0.0, 0.0);
                        } else {
                            // Not found from cursor, search from beginning
                            let (start, _) = source_buffer.bounds();
                            let text = source_buffer.text(&start, &cursor_iter, false);
                            let text_str = text.as_str();
                            
                            let found_pos = if case_check.is_active() {
                                text_str.find(search_str)
                            } else {
                                text_str.to_lowercase().find(&search_str.to_lowercase())
                            };
                            
                            if let Some(pos) = found_pos {
                                let mut search_start = start;
                                search_start.forward_chars(pos as i32);
                                let mut search_end = search_start;
                                search_end.forward_chars(search_str.len() as i32);
                                source_buffer.select_range(&search_start, &search_end);
                                let mut scroll_iter = search_start;
                                source_view.scroll_to_iter(&mut scroll_iter, 0.0, false, 0.0, 0.0);
                            }
                        }
                    }
                }
            } else {
                dialog.close();
            }
        });

        dialog.present();
    }

    /// Show find and replace dialog
    pub fn show_replace_dialog(&self, window: &gtk4::Window) {
        let dialog = Dialog::builder()
            .title("Find and Replace")
            .transient_for(window)
            .modal(true)
            .build();

        let content_area = dialog.content_area();
        let grid = Grid::new();
        grid.set_margin_top(12);
        grid.set_margin_bottom(12);
        grid.set_margin_start(12);
        grid.set_margin_end(12);
        grid.set_row_spacing(6);
        grid.set_column_spacing(6);

        let find_label = Label::new(Some("Find:"));
        let find_entry = Entry::new();
        find_entry.set_hexpand(true);

        let replace_label = Label::new(Some("Replace:"));
        let replace_entry = Entry::new();
        replace_entry.set_hexpand(true);
        
        // Add case-sensitive checkbox
        let case_sensitive_check = gtk4::CheckButton::builder()
            .label("Case sensitive")
            .build();

        grid.attach(&find_label, 0, 0, 1, 1);
        grid.attach(&find_entry, 1, 0, 1, 1);
        grid.attach(&replace_label, 0, 1, 1, 1);
        grid.attach(&replace_entry, 1, 1, 1, 1);
        grid.attach(&case_sensitive_check, 1, 2, 1, 1);

        content_area.append(&grid);

        dialog.add_button("Cancel", ResponseType::Cancel);
        dialog.add_button("Replace All", ResponseType::Apply);
        let replace_button = dialog.add_button("Replace", ResponseType::Ok);
        replace_button.set_css_classes(&["suggested-action"]);

        // Set focus to the find entry
        find_entry.grab_focus();

        let source_buffer = self.source_buffer.clone();
        let source_view = self.source_view.clone();
        
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Ok || response == ResponseType::Apply {
                let content_area = dialog.content_area();
                let grid = content_area
                    .first_child()
                    .and_then(|grid| grid.downcast::<Grid>().ok());

                if let Some(grid) = grid {
                    let find_entry = grid.child_at(1, 0)
                        .and_then(|entry| entry.downcast::<Entry>().ok());
                    let replace_entry = grid.child_at(1, 1)
                        .and_then(|entry| entry.downcast::<Entry>().ok());
                    let case_check = grid.child_at(1, 2)
                        .and_then(|check| check.downcast::<gtk4::CheckButton>().ok());

                    if let (Some(find_entry), Some(replace_entry), Some(case_check)) = (find_entry, replace_entry, case_check) {
                        let find_text = find_entry.text();
                        let replace_text = replace_entry.text();

                        if !find_text.is_empty() {
                            let (start, end) = source_buffer.bounds();
                            let text = source_buffer.text(&start, &end, false);
                            let text_str = text.as_str();
                            let find_str = find_text.as_str();
                            let replace_str = replace_text.as_str();
                            
                            if response == ResponseType::Apply {
                                // Replace all
                                let new_text = if case_check.is_active() {
                                    text_str.replace(find_str, replace_str)
                                } else {
                                    // Case-insensitive replace all (more complex)
                                    let find_lower = find_str.to_lowercase();
                                    let mut result = String::new();
                                    let mut last_end = 0;
                                    let text_lower = text_str.to_lowercase();
                                    
                                    for m in text_lower.match_indices(&find_lower) {
                                        let start_pos = m.0;
                                        result.push_str(&text_str[last_end..start_pos]);
                                        result.push_str(replace_str);
                                        last_end = start_pos + find_str.len();
                                    }
                                    result.push_str(&text_str[last_end..]);
                                    result
                                };
                                source_buffer.set_text(&new_text);
                            } else {
                                // Replace next occurrence from current cursor position
                                let cursor_mark = source_buffer.get_insert();
                                let cursor_iter = source_buffer.iter_at_mark(&cursor_mark);
                                let end_iter = source_buffer.end_iter();
                                let text_from_cursor = source_buffer.text(&cursor_iter, &end_iter, false);
                                let text_from_cursor_str = text_from_cursor.as_str();
                                
                                let found_pos = if case_check.is_active() {
                                    text_from_cursor_str.find(find_str)
                                } else {
                                    text_from_cursor_str.to_lowercase().find(&find_str.to_lowercase())
                                };
                                
                                if let Some(pos) = found_pos {
                                    let mut search_start = cursor_iter;
                                    search_start.forward_chars(pos as i32);
                                    let mut search_end = search_start;
                                    search_end.forward_chars(find_str.len() as i32);
                                    source_buffer.delete(&mut search_start, &mut search_end);
                                    source_buffer.insert(&mut search_start, replace_str);
                                    
                                    // Position cursor after replacement and scroll to it
                                    let mut new_pos = search_start;
                                    new_pos.forward_chars(replace_str.len() as i32);
                                    source_buffer.place_cursor(&new_pos);
                                    let mut scroll_iter = new_pos;
                                    source_view.scroll_to_iter(&mut scroll_iter, 0.0, false, 0.0, 0.0);
                                } else {
                                    // Not found from cursor, search from beginning
                                    let (start, _) = source_buffer.bounds();
                                    let text_to_cursor = source_buffer.text(&start, &cursor_iter, false);
                                    let text_to_cursor_str = text_to_cursor.as_str();
                                    
                                    let found_pos = if case_check.is_active() {
                                        text_to_cursor_str.find(find_str)
                                    } else {
                                        text_to_cursor_str.to_lowercase().find(&find_str.to_lowercase())
                                    };
                                    
                                    if let Some(pos) = found_pos {
                                        let mut search_start = start;
                                        search_start.forward_chars(pos as i32);
                                        let mut search_end = search_start;
                                        search_end.forward_chars(find_str.len() as i32);
                                        source_buffer.delete(&mut search_start, &mut search_end);
                                        source_buffer.insert(&mut search_start, replace_str);
                                        
                                        // Position cursor after replacement and scroll to it
                                        let mut new_pos = search_start;
                                        new_pos.forward_chars(replace_str.len() as i32);
                                        source_buffer.place_cursor(&new_pos);
                                        let mut scroll_iter = new_pos;
                                        source_view.scroll_to_iter(&mut scroll_iter, 0.0, false, 0.0, 0.0);
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                dialog.close();
            }
        });

        dialog.present();
    }

    /// Detect existing formatting on text to handle mixed formatting correctly
    fn detect_existing_formatting(&self, text: &str) -> Vec<(&'static str, &'static str)> {
        let mut found_formats = Vec::new();
        
        // Check for formatting patterns - order matters for overlapping patterns
        // We check longer patterns first to avoid false positives
        let patterns = [
            ("**", "**", "bold"),
            ("~~", "~~", "strikethrough"),
            ("==", "==", "highlight"),
            ("`", "`", "code"),
            ("*", "*", "italic"),
            ("~", "~", "subscript"),
            ("^", "^", "superscript"),
        ];
        
        for (prefix, suffix, _name) in &patterns {
            // Use the same logic as has_specific_formatting to be consistent
            if self.has_specific_formatting(text, prefix, suffix) {
                found_formats.push((*prefix, *suffix));
            }
        }
        
        found_formats
    }
    
    /// Smart formatting application that handles mixed formatting
    fn apply_smart_formatting(&self, text: &str, target_prefix: &str, target_suffix: &str) -> String {
        // Check if we already have this specific formatting - if so, this should be removal, not addition
        if self.has_specific_formatting(text, target_prefix, target_suffix) {
            // This should not happen - if we have the formatting, remove_specific_formatting should be called
            // But as a safety net, remove it here
            return self.remove_specific_formatting(text, target_prefix, target_suffix);
        }
        
        // Special handling for bold + italic combination
        if (target_prefix == "*" && self.has_specific_formatting(text, "**", "**")) ||
           (target_prefix == "**" && self.has_specific_formatting(text, "*", "*")) {
            // Only apply combined formatting if we don't already have both
            // Check if we already have the combined format
            if text.starts_with("***") && text.ends_with("***") && text.len() > 6 {
                // We already have combined format, this should not happen here
                // Return as-is to avoid malformed formatting
                return text.to_string();
            }
            
            // Remove the existing formatting and apply combined bold+italic
            let inner_text = if self.has_specific_formatting(text, "**", "**") {
                self.remove_specific_formatting(text, "**", "**")
            } else {
                self.remove_specific_formatting(text, "*", "*")
            };
            return format!("***{}***", inner_text);
        }
        
        // Detect existing formatting
        let existing_formats = self.detect_existing_formatting(text);
        
        if existing_formats.is_empty() {
            // No existing formatting, just add the new one
            format!("{}{}{}", target_prefix, text, target_suffix)
        } else {
            // For other combinations, use the layered approach
            // Remove all existing formatting first, then reapply with the new one
            let mut inner_text = text.to_string();
            
            // Remove existing formatting from outside to inside
            for (prefix, suffix) in &existing_formats {
                if inner_text.starts_with(prefix) && inner_text.ends_with(suffix) {
                    inner_text = inner_text[prefix.len()..inner_text.len() - suffix.len()].to_string();
                }
            }
            
            // Now build the new formatting with the target at the outermost layer
            let mut result = inner_text;
            
            // Apply existing formats from innermost to outermost (reverse order)
            for (prefix, suffix) in existing_formats.iter().rev() {
                result = format!("{}{}{}", prefix, result, suffix);
            }
            
            // Apply the new target formatting as the outermost layer
            format!("{}{}{}", target_prefix, result, target_suffix)
        }
    }

    /// Get the currently selected text, if any
    pub fn get_selected_text(&self) -> Option<String> {
        let gtk_buffer = self.source_buffer.upcast_ref::<gtk4::TextBuffer>();
        if let Some((start, end)) = gtk_buffer.selection_bounds() {
            Some(gtk_buffer.text(&start, &end, false).to_string())
        } else {
            None
        }
    }

    /// Check if there is text currently selected in the editor
    pub fn has_text_selection(&self) -> bool {
        let gtk_buffer = self.source_buffer.upcast_ref::<gtk4::TextBuffer>();
        gtk_buffer.has_selection()
    }
}