use gtk4::prelude::*;
use gtk4::{
    HeaderBar, Label, Orientation, Paned, ScrolledWindow, Widget, Stack,
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
use std::time::Instant;

#[derive(Clone)]
pub struct MarkdownEditor {
    pub(crate) widget: Paned,
    pub(crate) source_view: View,
    pub(crate) view_stack: Stack,
    pub(crate) html_view: MarkdownHtmlView,
    pub(crate) code_view: MarkdownCodeView,
    pub(crate) source_buffer: Buffer,
    pub(crate) current_file: Rc<RefCell<Option<std::path::PathBuf>>>,
    pub(crate) footer_callbacks: Rc<RefCell<Vec<Box<dyn Fn(&str, usize, usize, usize, usize)>>>>,
    pub(crate) code_language_manager: Rc<RefCell<CodeLanguageManager>>,
    pub(crate) theme_manager: Rc<RefCell<Option<crate::theme::ThemeManager>>>,
    pub(crate) is_modified: Rc<RefCell<bool>>,
    pub(crate) extra_syntax: Rc<RefCell<ExtraMarkdownSyntax>>,
    pub(crate) tag_table: Rc<RefCell<HashMap<String, gtk4::TextTag>>>,
    pub(crate) current_css_theme: Rc<RefCell<String>>,
    pub(crate) context_menu: Rc<RefCell<Option<ContextMenu>>>,
    pub(crate) last_formatting_action: Rc<RefCell<Option<Instant>>>,
    pub(crate) preserve_selection: Rc<RefCell<bool>>,
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
            last_formatting_action: Rc::new(RefCell::new(None)),
            preserve_selection: Rc::new(RefCell::new(false)),
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

    /// Check if the document has been modified since last save
    pub fn is_modified(&self) -> bool {
        *self.is_modified.borrow()
    }
    
    /// Mark the document as saved (not modified)
    pub(crate) fn mark_as_saved(&self) {
        println!("DEBUG: Marking document as saved (was modified: {})", self.is_modified());
        *self.is_modified.borrow_mut() = false;
    }

    pub fn insert_text_at_cursor(&self, text: &str) {
        let gtk_buffer = self.source_buffer.upcast_ref::<gtk4::TextBuffer>();
        let cursor_mark = gtk_buffer.get_insert();
        let mut cursor_iter = gtk_buffer.iter_at_mark(&cursor_mark);
        
        self.source_buffer.insert(&mut cursor_iter, text);
    }

    #[allow(dead_code)]
    pub(crate) fn find_format_at_cursor(&self, line_text: &str, cursor_offset: i32, prefix: &str, suffix: &str) -> Option<(i32, i32)> {
        let cursor_pos = cursor_offset as usize;
        let line_str = line_text;
        
        // Look for formatting that contains the cursor
        let mut pos = 0;
        while let Some(start_pos) = line_str[pos..].find(prefix) {
            let absolute_start = pos + start_pos;
            let search_start = absolute_start + prefix.len();
            
            if let Some(end_pos) = line_str[search_start..].find(suffix) {
                let absolute_end = search_start + end_pos;
                
                // Check if cursor is within this formatting
                if cursor_pos >= absolute_start && cursor_pos <= absolute_end + suffix.len() {
                    return Some((absolute_start as i32, (absolute_end + suffix.len()) as i32));
                }
                
                pos = absolute_end + suffix.len();
            } else {
                break;
            }
        }
        
        None
    }
}
