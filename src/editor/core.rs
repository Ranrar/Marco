/// Analyze the text at the cursor and return an HTML-like formatting string with indentation
pub fn get_formatting_at_cursor(text: &str, line: usize, col: usize) -> String {

    use crate::markdown::basic::MarkdownParser;
    let parser = MarkdownParser::new();
    let lines: Vec<&str> = text.lines().collect();
    if line == 0 || line > lines.len() {
        return "Format:".to_string();
    }
    let line_text = lines[line - 1];
    let mut parts = Vec::new();

    // Horizontal rule detection (---, ***, ___, etc.)
    let is_hr = {
        let t = line_text.trim();
        t == "---" || t == "***" || t == "___" ||
        (t.chars().all(|c| c == '-') && t.len() >= 3) ||
        (t.chars().all(|c| c == '*') && t.len() >= 3) ||
        (t.chars().all(|c| c == '_') && t.len() >= 3)
    };
    if is_hr {
        return "Format: Horizontal Rule".to_string();
    }

    // HTML tag detection (e.g., <code>, <b>, etc.)
    let html_tag_re = regex::Regex::new(r"<[^>]+>").unwrap();
    if html_tag_re.is_match(line_text) {
        return "Format: HTML".to_string();
    }

    // Horizontal rule detection (---, ***, ___, etc.)
    let is_hr = {
        let t = line_text.trim();
        t == "---" || t == "***" || t == "___" ||
        (t.chars().all(|c| c == '-') && t.len() >= 3) ||
        (t.chars().all(|c| c == '*') && t.len() >= 3) ||
        (t.chars().all(|c| c == '_') && t.len() >= 3)
    };
    if is_hr {
        return "Format: Horizontal Rule".to_string();
    }

    // HTML tag detection (e.g., <code>, <b>, etc.)
    let html_tag_re = regex::Regex::new(r"<[^>]+>").unwrap();
    if html_tag_re.is_match(line_text) {
        return "Format: HTML".to_string();
    }

    // Fenced code block detection
    // We'll scan from the start of the document to the current line to determine if we're inside a fenced code block
    let mut in_fence = false;
    let mut fence_lang = String::new();
    let mut fence_start = 0;
    for (i, l) in lines.iter().enumerate().take(line) {
        let trimmed = l.trim_start();
        if trimmed.starts_with("```") {
            if !in_fence {
                // Opening fence
                in_fence = true;
                fence_start = i + 1;
                // Try to extract language name
                let after = trimmed.trim_start_matches("```").trim();
                if !after.is_empty() {
                    fence_lang = after.to_string();
                } else {
                    fence_lang.clear();
                }
            } else {
                // Closing fence
                in_fence = false;
                fence_lang.clear();
            }
        }
    }
    if in_fence {
        let lang = if !fence_lang.is_empty() {
            format!("{} ", fence_lang[..1].to_uppercase() + &fence_lang[1..])
        } else {
            String::new()
        };
        return format!("Format: Fencing {}code", lang);
    }
    // Indentation detection
    let indent_len = line_text.chars().take_while(|c| c.is_whitespace()).count();
    if indent_len > 0 {
        parts.push((0, "Indent".to_string()));
    }

    // Header detection
    if let Some(hashes) = parser.detect_heading(line_text) {
        // Heading hashes are always at the start
        parts.push((0, format!("Header {}", hashes)));
    }

    // List detection (unordered and ordered)
    let trimmed = line_text.trim_start();
    let indent_offset = line_text.len() - trimmed.len();
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        parts.push((indent_offset, "List".to_string()));
    } else if parser.is_ordered_list(trimmed) {
        parts.push((indent_offset, "Ordered List".to_string()));
    }

    // Blockquote detection
    if trimmed.starts_with("> ") {
        parts.push((indent_offset, "Blockquote".to_string()));
    }

    // Inline Markdown detection (all regions, not just at cursor)
    let mut inline_regions = Vec::new();
    let text_bytes = line_text.as_bytes();

    // Detect Bold and Italic (***x***)
    let mut idx = 0;
    while let Some(start) = line_text[idx..].find("***") {
        let abs_start = idx + start;
        if let Some(end) = line_text[abs_start + 3..].find("***") {
            let abs_end = abs_start + 3 + end + 3 - 1;
            // Check that the region is exactly ***x***
            let region = &line_text[abs_start..=abs_end];
            if region.starts_with("***") && region.ends_with("***") && region.len() > 6 {
                inline_regions.push((abs_start, "Bold and Italic".to_string()));
            }
            idx = abs_end + 1;
        } else {
            break;
        }
    }

    // Detect Bold (**x**), but not ***x***
    let mut idx = 0;
    while let Some(start) = line_text[idx..].find("**") {
        let abs_start = idx + start;
        // Skip if this is part of a ***x*** region
        if line_text[abs_start..].starts_with("***") {
            idx = abs_start + 3;
            continue;
        }
        if let Some(end) = line_text[abs_start + 2..].find("**") {
            let abs_end = abs_start + 2 + end + 2 - 1;
            let region = &line_text[abs_start..=abs_end];
            if region.starts_with("**") && region.ends_with("**") && region.len() > 4 {
                inline_regions.push((abs_start, "Bold".to_string()));
            }
            idx = abs_end + 1;
        } else {
            break;
        }
    }

    // Detect Italic (*x*), but not **x** or ***x***
    let mut idx = 0;
    while let Some(start) = line_text[idx..].find('*') {
        let abs_start = idx + start;
        // Skip if this is part of a ** or *** region
        if line_text[abs_start..].starts_with("***") {
            idx = abs_start + 3;
            continue;
        } else if line_text[abs_start..].starts_with("**") {
            idx = abs_start + 2;
            continue;
        }
        if let Some(end) = line_text[abs_start + 1..].find('*') {
            let abs_end = abs_start + 1 + end + 1 - 1;
            let region = &line_text[abs_start..=abs_end];
            if region.starts_with('*') && region.ends_with('*') && region.len() > 2 {
                inline_regions.push((abs_start, "Italic".to_string()));
            }
            idx = abs_end + 1;
        } else {
            break;
        }
    }

    // Other inline regions (strikethrough, code, link, image)
    let region_checks = [
        ("Strikethrough", parser.find_strikethrough(line_text)),
        ("Inline Code", parser.find_inline_code(line_text)),
        ("Link", parser.find_links(line_text)),
        ("Image", parser.find_images(line_text)),
    ];
    for (name, regions) in region_checks.iter() {
        for (start, _end) in regions {
            inline_regions.push((*start, name.to_string()));
        }
    }

    // Sort all parts (block and inline) by their position in the line
    parts.extend(inline_regions);
    parts.sort_by_key(|(pos, _)| *pos);

    // Remove duplicates but keep order
    let mut seen = std::collections::HashSet::new();
    let mut ordered = Vec::new();
    for (_pos, name) in parts {
        if seen.insert(name.clone()) {
            ordered.push(name);
        }
    }

    if ordered.is_empty() {
        if line_text.trim().is_empty() {
            "Format:".to_string()
        } else {
            // If not markdown, not html, not hr, treat as plain text
            "Format: Text".to_string()
        }
    } else {
        format!("Format: {}", ordered.join(" > "))
    }
}
use crate::editor::context_menu::ContextMenu;
use crate::editor::md_spell_check::SpellSyntaxChecker;
use crate::markdown::advanced::ExtraMarkdownSyntax;
use crate::markdown::colorize_code_blocks::CodeLanguageManager;
use crate::view::{MarkdownCodeView, MarkdownHtmlView};
use gtk4::prelude::*;
use gtk4::{HeaderBar, Label, Orientation, Paned, ScrolledWindow, Stack, Widget};
use sourceview5::prelude::*;
use sourceview5::{Buffer, LanguageManager, StyleSchemeManager, View};
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
    pub(crate) code_language_manager: Rc<RefCell<CodeLanguageManager>>,
    pub(crate) theme_manager: Rc<RefCell<Option<crate::theme::ThemeManager>>>,
    pub(crate) is_modified: Rc<RefCell<bool>>,
    pub(crate) extra_syntax: Rc<RefCell<ExtraMarkdownSyntax>>,
    pub(crate) tag_table: Rc<RefCell<HashMap<String, gtk4::TextTag>>>,
    pub(crate) context_menu: Rc<RefCell<Option<ContextMenu>>>,
    pub(crate) last_formatting_action: Rc<RefCell<Option<Instant>>>,
    pub(crate) preserve_selection: Rc<RefCell<bool>>,
    pub(crate) header_bar: HeaderBar,
    // Track the original content to determine if document is truly modified
    pub(crate) original_content: Rc<RefCell<String>>,
    // Markdown syntax checker for warnings
    pub(crate) spell_checker: Rc<RefCell<SpellSyntaxChecker>>,
    pub(crate) warnings_enabled: Rc<RefCell<bool>>,
    /// General-purpose debouncer for per-keystroke features (syntax, lint, etc)
    pub(crate) debouncer: Rc<crate::utils::debouncer::Debouncer>,
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

        // Note: SourceView syntax coloring will be controlled by settings
        // Don't set language or style scheme here - it will be set by set_editor_color_syntax()

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
        let code_language_manager = Rc::new(RefCell::new(CodeLanguageManager::new()));
        let is_modified = Rc::new(RefCell::new(false));
        let extra_syntax = Rc::new(RefCell::new(ExtraMarkdownSyntax::new()));
        let tag_table = Rc::new(RefCell::new(HashMap::new()));

        // Create header bar for title management
        let header_bar = HeaderBar::new();

        // Initialize syntax checker for markdown warnings
        let spell_check_markdown = SpellSyntaxChecker::new_with_defaults();

        let debouncer = Rc::new(crate::utils::debouncer::Debouncer::new(120)); // 120ms default debounce
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
            context_menu: Rc::new(RefCell::new(None)),
            last_formatting_action: Rc::new(RefCell::new(None)),
            preserve_selection: Rc::new(RefCell::new(false)),
            header_bar,
            original_content: Rc::new(RefCell::new(String::new())),
            spell_checker: Rc::new(RefCell::new(spell_check_markdown)),
            warnings_enabled: Rc::new(RefCell::new(true)), // Enable warnings by default
            debouncer,
        };

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
        let original_content = self.original_content.clone();
        let spell_checker = self.spell_checker.clone();
        let warnings_enabled = self.warnings_enabled.clone();
        let debouncer = self.debouncer.clone();
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
            let extra_syntax = extra_syntax.clone();
            let tag_table = tag_table.clone();
            let buffer_clone = buffer.clone();
            let prefs = crate::settings::core::get_app_preferences();
            let syntax_enabled = prefs.get_editor_color_syntax();
            let spell_checker = spell_checker.clone();
            let warnings_enabled = warnings_enabled.clone();
            let text_string_clone = text_string.clone();
            debouncer.debounce(move || {
                if syntax_enabled {
                    eprintln!("============ Applying syntax coloring (debounced) ============");
                    // Apply extra syntax coloring (underlines, colors, comments, etc.)
                    {
                        let extra_syntax_ref = extra_syntax.borrow();
                        let mut tag_table_ref = tag_table.borrow_mut();
                        extra_syntax_ref.apply_extra_syntax_coloring(&buffer_clone, &text_string_clone, &mut tag_table_ref);
                    }
                    // Apply syntect syntax coloring
                    let ui_theme = prefs.get_ui_theme();
                    let theme_name = match ui_theme.as_str() {
                        "dark" => "dark",
                        "light" => "light",
                        _ => "dark",
                    };
                    let mut tag_table_ref = tag_table.borrow_mut();
                    crate::editor::syntax::color::apply_syntax_coloring(
                        &buffer_clone,
                        &text_string_clone,
                        &mut tag_table_ref,
                        theme_name,
                    );
                } else {
                    eprintln!("============ NOT applying coloring - syntax is disabled ============");
                }
                // Clear all warning tags immediately on every keystroke (debounced)
                spell_checker.borrow_mut().clear_warnings();
                // Apply markdown warnings if enabled (debounced)
                if *warnings_enabled.borrow() {
                    let weak_checker = Rc::downgrade(&spell_checker);
                    // If you want to debounce spellcheck, call it here
                    // crate::editor::md_spell_check::SpellSyntaxChecker::trigger_spellcheck_debounced(weak_checker, text_string_clone.clone());
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
            eprintln!("============ Editor color syntax enabled ============");

            // Enable SourceView built-in syntax coloring
            let language_manager = LanguageManager::default();
            if let Some(language) = language_manager.language("markdown") {
                self.source_buffer.set_language(Some(&language));
            }

            // Apply our custom syntax coloring (tmTheme overlays)
            self.apply_syntax_coloring();
        } else {
            eprintln!("============ Editor color syntax coloring disabled ============");

            // Disable SourceView built-in syntax coloring but keep base theme
            self.source_buffer.set_language(None);

            // Remove our custom syntax coloring but keep base style scheme
            self.remove_syntax_coloring();
        }
    }

    /// Apply syntax coloring to the editor using syntect
    fn apply_syntax_coloring(&self) {
        println!("DEBUG: apply_syntax_coloring called");
        // Check if syntax coloring is enabled
        let prefs = crate::settings::core::get_app_preferences();
        if !prefs.get_editor_color_syntax() {
            println!("DEBUG: Syntax coloring disabled in apply_syntax_coloring, returning");
            return; // Do nothing if syntax coloring is disabled
        }

        let gtk_buffer = self.source_buffer.upcast_ref::<gtk4::TextBuffer>();
        let start = gtk_buffer.start_iter();
        let end = gtk_buffer.end_iter();
        let text = gtk_buffer.text(&start, &end, false).to_string();

        println!("DEBUG: Text length to highlight: {}", text.len());

        // Remove existing syntax coloring tags first
        self.remove_syntax_coloring();

        // Apply extra syntax coloring (underlines, colors, comments, etc.)
        {
            let mut tag_table = self.tag_table.borrow_mut();
            let extra_syntax = self.extra_syntax.borrow();
            println!("DEBUG: Applying extra syntax coloring");
            extra_syntax.apply_extra_syntax_coloring(&self.source_buffer, &text, &mut tag_table);
        }

        // Apply syntect coloring using ThemeManager
        if let Some(ref theme_manager) = *self.theme_manager.borrow() {
            let mut tag_table_ref = self.tag_table.borrow_mut();
            println!("DEBUG: Applying syntect coloring using ThemeManager");
            self.apply_syntect_highlighting(
                &self.source_buffer,
                &text,
                &mut tag_table_ref,
                theme_manager,
            );
            println!("DEBUG: apply_syntect_highlighting completed");
        } else {
            println!("DEBUG: No ThemeManager available, skipping syntax highlighting");
        }
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
            self.apply_syntax_coloring();
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
            println!("Markdown warnings enabled");

            // Re-check current content when enabling warnings
            let gtk_buffer = self.source_buffer.upcast_ref::<gtk4::TextBuffer>();
            let start = gtk_buffer.start_iter();
            let end = gtk_buffer.end_iter();
            let _text = gtk_buffer.text(&start, &end, false).to_string();

            // Warning checking would be done here if implemented
        } else {
            println!("Markdown warnings disabled");
        }
    }

    /// Set layout direction (for compatibility with existing code)
    pub fn set_layout_reversed(&self, _reversed: bool) {
        // This is a placeholder - the actual layout reversal would be handled
        // at the application level by rearranging the paned container
        println!("Layout reversed setting changed (handled at app level)");
    }
}
