use crate::editor::context_menu::ContextMenu;
use crate::editor::syntax::md_spell_check::SpellSyntaxChecker;
use crate::markdown::extended::ExtendedMarkdownSyntax;
use crate::view::{MarkdownCodeView, MarkdownHtmlView};
use gtk4::prelude::*;
use gtk4::{HeaderBar, Label, Paned, ScrolledWindow, Stack, Widget};
use crate::ui::splitview;
use sourceview5::prelude::*;
use sourceview5::{Buffer, StyleSchemeManager, View};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
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
    pub(crate) theme_manager: Rc<RefCell<Option<crate::theme::ThemeManager>>>,
    pub(crate) is_modified: Rc<RefCell<bool>>,
    pub(crate) extended_syntax: Rc<RefCell<ExtendedMarkdownSyntax>>,
    pub(crate) tag_table: Rc<RefCell<HashMap<String, gtk4::TextTag>>>,
    pub(crate) context_menu: Rc<RefCell<Option<ContextMenu>>>,
    pub(crate) last_formatting_action: Rc<RefCell<Option<Instant>>>,
    pub(crate) header_bar: HeaderBar,
    // Track the original content to determine if document is truly modified
    pub(crate) original_content: Rc<RefCell<String>>,
    // Markdown syntax checker for warnings
    pub(crate) spell_checker: Rc<RefCell<SpellSyntaxChecker>>,
    pub(crate) warnings_enabled: Rc<RefCell<bool>>,
    /// General-purpose debouncer for per-keystroke features (syntax, lint, etc)
    pub(crate) debouncer: Rc<crate::utils::debouncer::Debouncer>,
    pub(crate) syntect_highlighter: Rc<RefCell<crate::editor::fencing_code_block::fencing_code_block::SyntectHighlighter>>,
}

impl MarkdownEditor {
    /// Enable or disable text wrapping in the editor
    pub fn set_text_wrap(&self, enabled: bool) {
        use gtk4::WrapMode;
        // sourceview5::View implements IsA<TextView>
        let wrap_mode = if enabled { WrapMode::Word } else { WrapMode::None };
        self.source_view.set_wrap_mode(wrap_mode);
    }
    pub fn new() -> Self {
        // Create the main paned widget using splitview helper
        let paned = splitview::create_editor_split_pane();

        // Create source view with buffer
        let source_buffer = Buffer::new(None);
        let source_view = View::with_buffer(&source_buffer);

        // Configure source view
        source_view.set_show_line_numbers(true);
        source_view.set_highlight_current_line(true);
        source_view.set_tab_width(4);
        source_view.set_insert_spaces_instead_of_tabs(true);
        source_view.set_auto_indent(true);

        // Note: SourceView syntax coloring will be controlled by settings
        // Don't set language or style scheme here - it will be set by set_editor_color_syntax()

        // Create views
        let html_view = MarkdownHtmlView::new();
        let code_view = MarkdownCodeView::new(); // Only used for view_stack, not needed as a local variable

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

        // Clamp split position logic and save ratio on user move
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
                // Save split ratio to settings
                if total_width > 0 {
                    let ratio = ((pos as f64 / total_width as f64) * 100.0).round() as i32;
                    let ratio = ratio.clamp(10, 90);
                    crate::settings::get_app_preferences().set_layout_ratio(ratio);
                }
            }
        });

        // Set up current_file and footer_callbacks
        let current_file = Rc::new(RefCell::new(None));
        let footer_callbacks = Rc::new(RefCell::new(Vec::new()));
        let is_modified = Rc::new(RefCell::new(false));
        let extended_syntax = Rc::new(RefCell::new(ExtendedMarkdownSyntax::new()));
        let tag_table = Rc::new(RefCell::new(HashMap::new()));

        // Create header bar for title management
        let header_bar = HeaderBar::new();

        // Initialize syntax checker for markdown warnings
        let spell_check_markdown = SpellSyntaxChecker::new_with_defaults();

        let debouncer = Rc::new(crate::utils::debouncer::Debouncer::new(120)); // 120ms default debounce
        let syntect_highlighter = Rc::new(RefCell::new(crate::editor::fencing_code_block::fencing_code_block::SyntectHighlighter::new()));
        let editor = Self {
            widget: paned,
            source_view,
            view_stack,
            html_view,
            code_view,
            source_buffer,
            current_file,
            footer_callbacks,
            theme_manager: Rc::new(RefCell::new(None)),
            is_modified,
            extended_syntax,
            tag_table,
            context_menu: Rc::new(RefCell::new(None)),
            last_formatting_action: Rc::new(RefCell::new(None)),
            header_bar,
            original_content: Rc::new(RefCell::new(String::new())),
            spell_checker: Rc::new(RefCell::new(spell_check_markdown)),
            warnings_enabled: Rc::new(RefCell::new(true)), // Enable warnings by default
            debouncer,
            syntect_highlighter,
        };

        // Load and apply text wrap setting at startup
        let prefs = crate::settings::core::get_app_preferences();
        let wrap_enabled = prefs.get_editor_text_wrap();
        editor.set_text_wrap(wrap_enabled);

        // Set buffer reference for syntax checker
        {
            let gtk_buffer = editor.source_buffer.upcast_ref::<gtk4::TextBuffer>();
            editor.spell_checker.borrow_mut().set_buffer(gtk_buffer);
        }

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

        // Set up preview context menus for both HTML and Code views
        editor.html_view.setup_context_menu(&editor);
        editor.code_view.setup_context_menu(&editor);

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
        // Set the split position using splitview helper
        splitview::set_split_ratio(&self.widget, total_width);
    }

    pub fn add_footer_callback<F>(&self, callback: F)
    where
        F: Fn(&str, usize, usize, usize, usize) + 'static,
    {
        self.footer_callbacks.borrow_mut().push(Box::new(callback));
    }

    fn connect_text_changed(&self) {
        let html_view = self.html_view.clone();
        let footer_callbacks = self.footer_callbacks.clone();
        let is_modified = self.is_modified.clone();
        // let extended_syntax = self.extended_syntax.clone(); // No longer needed
        let tag_table = self.tag_table.clone();
        let original_content = self.original_content.clone();
        let spell_checker = self.spell_checker.clone();
        let warnings_enabled = self.warnings_enabled.clone();
        let debouncer = self.debouncer.clone();
        // let theme_manager_cloned = self.theme_manager.clone(); // No longer needed
        // Store clone of editor for window title updates
        let editor_for_title = self.clone();

        self.source_buffer.connect_changed(move |buffer| {
            let start = buffer.start_iter();
            let end = buffer.end_iter();
            let text = buffer.text(&start, &end, false);
            let text_string = text.to_string();
            // Restore immediate HTML view update
            html_view.update_content(&text_string);

            // Smart modification tracking: compare current content with original content
            let was_modified = *is_modified.borrow();
            let current_content = &text_string;
            let original = original_content.borrow();
            let is_now_modified = current_content != &*original;

            if was_modified != is_now_modified {
                *is_modified.borrow_mut() = is_now_modified;
                println!(
                    "DEBUG: Document modification state changed to: {}",
                    is_now_modified
                );
                // Update window title when modification state changes
                editor_for_title.update_window_title();
            }

            // Debounce syntax highlighting and related per-keystroke features
            let tag_table = tag_table.clone();
            let buffer_clone = buffer.clone();
            let prefs = crate::settings::core::get_app_preferences();
            let syntax_enabled = prefs.get_editor_color_syntax();
            let spell_checker = spell_checker.clone();
            let warnings_enabled = warnings_enabled.clone();
            let text_string_clone = text_string.clone();
            // let extended_syntax = extended_syntax.clone(); // No longer needed
            // let theme_manager = theme_manager_cloned.clone(); // No longer needed
            debouncer.debounce(move || {
                if syntax_enabled {
                    eprintln!("============ Applying named/hex color parsing (debounced) ============" );
                    let mut tag_table_ref = tag_table.borrow_mut();
                    // Use regexes for color tags (define or import as needed)
                    let color_regex = regex::Regex::new(r#"color:\s*([#\w]+)"#).unwrap();
                    let font_color_regex = regex::Regex::new(r#"<font color=\"([#\w]+)\">"#).unwrap();
                    crate::editor::syntax::color_syntax::color::highlight_colored_text(
                        &buffer_clone,
                        &text_string_clone,
                        &mut tag_table_ref,
                        &color_regex,
                        &font_color_regex,
                    );
                } else {
                    eprintln!("============ NOT applying coloring - syntax is disabled ============" );
                }
                // Clear all warning tags immediately on every keystroke (debounced)
                spell_checker.borrow_mut().clear_warnings();
                // Apply markdown warnings if enabled (debounced)
                if *warnings_enabled.borrow() {
                    let weak_checker = Rc::downgrade(&spell_checker);
                    crate::editor::syntax::md_spell_check::SpellSyntaxChecker::trigger_spellcheck_debounced(weak_checker, text_string_clone.clone());
                }
            });

            // Update footer statistics immediately (no delay needed for stats)
            let char_count = text_string.chars().count();
            let word_count = text_string.split_whitespace().count();
            // Use GTK TextBuffer's get_insert mark
            let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
            let cursor_iter = gtk_buffer.iter_at_mark(&gtk_buffer.get_insert());
            let line = cursor_iter.line() + 1;
            let column = cursor_iter.line_offset() + 1;
            for callback in footer_callbacks.borrow().iter() {
                callback(
                    &text_string,
                    word_count,
                    char_count,
                    line as usize,
                    column as usize,
                );
            }
        });
    }

    fn connect_cursor_moved(&self) {
        let footer_callbacks = self.footer_callbacks.clone();

        self.source_buffer
            .connect_mark_set(move |buffer, _iter, mark| {
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
                        callback(
                            &text,
                            word_count,
                            char_count,
                            line as usize,
                            column as usize,
                        );
                    }
                }
            });
    }

    pub fn create_simple_header_bar(&self) -> &HeaderBar {
        // Return reference to the header bar that was created in new()
        &self.header_bar
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
        println!(
            "DEBUG: Marking document as saved (was modified: {})",
            self.is_modified()
        );
        *self.is_modified.borrow_mut() = false;

        // Update original content to current content since we're now saved
        let start = self.source_buffer.start_iter();
        let end = self.source_buffer.end_iter();
        let current_text = self.source_buffer.text(&start, &end, false);
        *self.original_content.borrow_mut() = current_text.to_string();

        // Update window title when save state changes
        self.update_window_title();
    }

    /// Check if the document is empty (0 characters)
    pub fn is_empty(&self) -> bool {
        let start = self.source_buffer.start_iter();
        let end = self.source_buffer.end_iter();
        let text = self.source_buffer.text(&start, &end, false);
        text.trim().is_empty()
    }

    /// Update the window title to reflect current file and modification state
    pub fn update_window_title(&self) {
        let base_title = "Marco";
        let title = if let Some(file_path) = self.current_file.borrow().as_ref() {
            let filename = file_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Untitled");
            if self.is_modified() {
                format!("{} - {}*", base_title, filename)
            } else {
                format!("{} - {}", base_title, filename)
            }
        } else {
            // New file without a path
            if self.is_modified() {
                format!("{} - Untitled*", base_title)
            } else {
                format!("{} - Untitled", base_title)
            }
        };

        // Update both window title and header bar title
        if let Some(widget) = self.widget.root() {
            if let Ok(window) = widget.downcast::<gtk4::Window>() {
                window.set_title(Some(&title));
            }
        }

        // Update header bar title widget
        self.header_bar
            .set_title_widget(Some(&Label::new(Some(&title))));
    }

    pub fn insert_text_at_cursor(&self, text: &str) {
        let gtk_buffer = self.source_buffer.upcast_ref::<gtk4::TextBuffer>();
        let cursor_mark = gtk_buffer.get_insert();
        let mut cursor_iter = gtk_buffer.iter_at_mark(&cursor_mark);

        self.source_buffer.insert(&mut cursor_iter, text);
    }

    #[allow(dead_code)]
    pub(crate) fn find_format_at_cursor(
        &self,
        line_text: &str,
        cursor_offset: i32,
        prefix: &str,
        suffix: &str,
    ) -> Option<(i32, i32)> {
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

    /// Enable or disable function coloring
    pub fn set_function_colloring(&self, enabled: bool) {
        // Store the setting for use in syntax coloring
        // This would need to be implemented in system
        if enabled {
            println!("Function coloring enabled");
        } else {
            println!("Function coloring disabled");
        }
    }

    /// Enable or disable editor color syntax
    pub fn set_editor_color_syntax(&self, enabled: bool) {
        eprintln!(
            "============ DEBUG: set_editor_color_syntax called with enabled={} ============",
            enabled
        );

        // Get the current UI theme to determine the appropriate style scheme
        let prefs = crate::settings::core::get_app_preferences();
        let ui_theme = prefs.get_ui_theme();
        let style_manager = StyleSchemeManager::default();

        // Always apply a base style scheme that matches the UI theme
        let scheme_name = if ui_theme == "dark" {
            "Adwaita-dark"
        } else {
            "Adwaita"
        };

        if let Some(scheme) = style_manager.scheme(scheme_name) {
            self.source_buffer.set_style_scheme(Some(&scheme));
            eprintln!(
                "============ Applied base style scheme: {} ============",
                scheme_name
            );
        }

        if enabled {
            eprintln!("============ Editor named/hex color parsing enabled ============");
            // Only apply named/hex color parsing
            self.apply_colored_text_highlighting();
        } else {
            eprintln!("============ Editor named/hex color parsing disabled ============");
            self.remove_syntax_coloring();
        }
    }

    /// Apply only named/hex color parsing to the editor
    fn apply_colored_text_highlighting(&self) {
        let gtk_buffer = self.source_buffer.upcast_ref::<gtk4::TextBuffer>();
        let start = gtk_buffer.start_iter();
        let end = gtk_buffer.end_iter();
        let text = gtk_buffer.text(&start, &end, false).to_string();
        self.remove_syntax_coloring();
        let mut tag_table_ref = self.tag_table.borrow_mut();
        let color_regex = regex::Regex::new(r#"color:\s*([#\w]+)"#).unwrap();
        let font_color_regex = regex::Regex::new(r#"<font color=\"([#\w]+)\">"#).unwrap();
        crate::editor::syntax::color_syntax::color::highlight_colored_text(
            &self.source_buffer,
            &text,
            &mut tag_table_ref,
            &color_regex,
            &font_color_regex,
        );
    }

    /// Update the editor theme to match the current UI theme
    pub fn update_editor_theme(&self) {
        // Use ThemeManager for theming instead of fallback style schemes
        if let Some(ref theme_manager) = *self.theme_manager.borrow() {
            // Apply CSS classes for visual theming
            let style_context = self.source_view.style_context();
            
            // Remove any existing theme classes
            style_context.remove_class("theme-light");
            style_context.remove_class("theme-dark");
            
            // Add appropriate theme class based on effective theme
            match theme_manager.get_effective_theme() {
                crate::theme::Theme::Light => style_context.add_class("theme-light"),
                crate::theme::Theme::Dark => style_context.add_class("theme-dark"),
                crate::theme::Theme::System => {
                    match crate::theme::ThemeManager::detect_system_theme() {
                        crate::theme::Theme::Dark => style_context.add_class("theme-dark"),
                        _ => style_context.add_class("theme-light"),
                    }
                }
            }
            
            eprintln!("============ Updated editor theme using ThemeManager ============");
        } else {
            eprintln!("WARNING: No ThemeManager available for editor theme update");
        }

        // If syntax coloring is enabled, reapply it with the new theme
        let prefs = crate::settings::core::get_app_preferences();
        if prefs.get_editor_color_syntax() {
            self.apply_colored_text_highlighting();
        }
    }

    /// Remove all syntax coloring from the editor
    fn remove_syntax_coloring(&self) {
        println!("DEBUG: remove_syntax_coloring called");
        let gtk_buffer = self.source_buffer.upcast_ref::<gtk4::TextBuffer>();
        let start = gtk_buffer.start_iter();
        let end = gtk_buffer.end_iter();

        // Remove ALL tags from the buffer to ensure plain text
        let tag_table = gtk_buffer.tag_table();
        let mut all_tags = Vec::new();

        // Collect all tags first (we can't modify the tag table while iterating)
        tag_table.foreach(|tag| {
            if let Some(name) = tag.name() {
                all_tags.push((tag.clone(), name.to_string()));
            }
        });

        let mut removed_tags = Vec::new();

        // Remove all tags except system/GTK built-in tags
        for (tag, name) in all_tags {
            // Skip built-in GTK tags that we shouldn't remove
            if !name.starts_with("gtk-") && !name.starts_with("selection") {
                gtk_buffer.remove_tag(&tag, &start, &end);
                removed_tags.push(name);
            }
        }

        println!(
            "DEBUG: Removed {} tags: {:?}",
            removed_tags.len(),
            removed_tags
        );
    }

    /// Enable or disable markdown warnings
    pub fn set_markdown_warnings(&self, enabled: bool) {
        *self.warnings_enabled.borrow_mut() = enabled;

        if enabled {
            println!("Markdown syntax errors enabled");

            // Re-check current content when enabling warnings
            let gtk_buffer = self.source_buffer.upcast_ref::<gtk4::TextBuffer>();
            let start = gtk_buffer.start_iter();
            let end = gtk_buffer.end_iter();
            let _text = gtk_buffer.text(&start, &end, false).to_string();

            // Warning checking would be done here if implemented
        } else {
            println!("Markdown syntax errors disabled");
        }
    }
}
