
/// Thread safety and async message passing example for GTK4 Rust
///
/// All GTK4 widget creation, updates, and signal handling must occur on the main thread.
/// For background/async work (e.g., Rayon, std::thread, async tasks), use message passing to communicate results back to the main thread.
/// Use glib::Sender and glib::MainContext::channel() to send messages from worker threads to the main thread.
/// Use glib::idle_add_local() to schedule closures on the main thread for UI updates.
/// Never touch GTK widgets directly from non-main threads.
/// For shared state, use Arc<Mutex<T>> or Arc<RwLock<T>> for data, but always update GTK widgets via main thread message passing.
/// Example pattern:
///
/// use glib::MainContext;
/// use std::sync::{Arc, Mutex};
/// use rayon::prelude::*;
///
/// let (sender, receiver) = MainContext::channel(glib::PRIORITY_DEFAULT);
///
/// // In worker thread or rayon pool:
/// rayon::spawn(move || {
///     let result = do_expensive_work();
///     sender.send(result).unwrap();
/// });
///
/// // In main thread:
/// receiver.attach(None, move |result| {
///     // Update GTK widgets here
///     update_ui(result);
///     glib::Continue(true)
/// });
///
/// For async/await, use glib::MainContext::spawn_local() to run futures on the main thread.
/// GTK4-rs Editor Skeleton with Gutter, Syntax Highlighting, and Advanced Features

/// The main buffer struct for the custom code editor.
/// Holds all text, cursor, selection, undo/redo, theme, and rendering state.
pub struct EditorBuffer {
    /// Lines of text in the buffer
    pub lines: Vec<String>,
    /// Cursor row position
    pub cursor_row: usize,
    /// Cursor column position
    pub cursor_col: usize,
    /// Scroll offset for vertical scrolling
    pub scroll_offset: usize,
    /// Whether to highlight the current line
    pub highlight_line: bool,
    /// Selection start (row, col)
    pub selection_start: Option<(usize, usize)>,
    /// Selection end (row, col)
    pub selection_end: Option<(usize, usize)>,
    /// List of additional cursors (row, col)
    pub multi_cursors: Vec<(usize, usize)>,
    /// List of additional selections (start, end)
    pub multi_selections: Vec<(Option<(usize, usize)>, Option<(usize, usize)>)>,
    /// Current theme for syntax highlighting
    pub theme: syntect::highlighting::Theme,
    /// Syntax set for highlighting
    pub syntax_set: SyntaxSet,
    /// Undo stack for buffer edits
    pub undo_stack: Vec<Vec<String>>,
    /// Redo stack for buffer edits
    pub redo_stack: Vec<Vec<String>>,
    /// Word wrap enabled
    pub word_wrap: bool,
    /// Font name
    pub font: String,
    /// Font size
    pub font_size: f64,
    /// Line height
    pub line_height: f64,
    /// Character spacing
    pub character_spacing: f64,
    /// Foreground color
    pub fg_color: String,
    /// Background color
    pub bg_color: String,
    /// Gutter color
    pub gutter_color: String,
    /// Line number color
    pub line_number_color: String,
    /// Selected line number color
    pub selected_line_number_color: String,
    /// Highlight color
    pub highlight_color: String,
    /// Syntax highlighting enabled
    pub syntax_highlighting: bool,
    /// Cursor color
    pub cursor_color: String,
    /// Markdown header color
    pub markdown_header_color: String,
    /// Markdown bold color
    pub markdown_bold_color: String,
    /// Markdown italic color
    pub markdown_italic_color: String,
    /// Markdown code color
    pub markdown_code_color: String,
    /// Markdown link color
    pub markdown_link_color: String,
    /// Markdown quote color
    pub markdown_quote_color: String,
    /// Markdown list color
    pub markdown_list_color: String,
    /// Markdown syntax coloring enabled
    pub markdown_syntax_coloring: bool,
    /// Error color for diagnostics
    pub error_color: String,
    /// Warning color for diagnostics
    pub warning_color: String,
    /// Diagnostics highlighting enabled
    pub diagnostics_highlighting: bool,
    /// Search match color
    pub search_match_color: String,
    /// Whitespace guide color
    pub whitespace_guide_color: String,
    /// Show whitespace guides
    pub show_whitespace_guides: bool,
    /// Active line background color
    pub active_line_bg_color: String,
    /// Show active line background
    pub show_active_line_bg: bool,
    /// Inactive line background color
    pub inactive_line_bg_color: String,
    /// Show inactive line background
    pub show_inactive_line_bg: bool,
    /// Debug mode enabled
    pub debug_mode: bool,
    /// Diagnostics messages (row, message, kind)
    pub diagnostics: Vec<(usize, String, String)>,
    /// A4 page mode enabled
    pub a4_mode: bool,
    /// Top margin in centimeters
    pub top_margin_cm: f64,
    /// Bottom margin in centimeters
    pub bottom_margin_cm: f64,
    /// Left margin in centimeters
    pub left_margin_cm: f64,
    /// Right margin in centimeters
    pub right_margin_cm: f64,
    /// Optional redraw callback for GTK UI
    #[allow(clippy::type_complexity)]
    pub redraw_callback: Option<Box<dyn Fn()>>,
}

use std::cell::RefCell;
use std::rc::Rc;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, DrawingArea};
use gtk4::gdk;
use glib::clone;
use gtk4::pango::{AttrList, Layout};
use pangocairo::functions as pango_cairo;
use cairo::{Context, ImageSurface};
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::easy::HighlightLines;

const FONT: &str = "monospace";
const FONT_SIZE: f64 = 16.0;
const LINE_HEIGHT: f64 = 20.0;
const GUTTER_WIDTH: i32 = 40;


impl EditorBuffer {
    /// Request a redraw of the editor UI (calls the redraw_callback if set)
    pub fn request_redraw(&self) {
        if let Some(ref cb) = self.redraw_callback {
            cb();
        }
    }
    /// Move cursor up by one visible page (PgUp)
    pub fn move_page_up(&mut self, lines_per_page: usize) {
        if self.cursor_row > lines_per_page {
            self.cursor_row -= lines_per_page;
        } else {
            self.cursor_row = 0;
        }
        self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
    }

    /// Move cursor down by one visible page (PgDn)
    pub fn move_page_down(&mut self, lines_per_page: usize) {
        let max_row = self.lines.len().saturating_sub(1);
        self.cursor_row = (self.cursor_row + lines_per_page).min(max_row);
        self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
    }
    /// Cut selected text (removes and returns it)
    pub fn cut(&mut self) -> String {
        if let (Some((row, col_start)), Some((_, col_end))) = (self.selection_start, self.selection_end) {
            if row < self.lines.len() && col_end > col_start {
                let cut = self.lines[row][col_start..col_end].to_string();
                self.lines[row].replace_range(col_start..col_end, "");
                return cut;
            }
        }
        String::new()
    }
    /// Set character spacing
    fn set_character_spacing(&mut self, spacing: f64) {
        self.character_spacing = spacing;
    }
    /// Set cursor color
    fn set_cursor_color(&mut self, color: String) {
        self.cursor_color = color;
    }

    /// Set markdown syntax colors
    fn set_markdown_colors(&mut self, header: String, bold: String, italic: String, code: String, link: String, quote: String, list: String) {
        self.markdown_header_color = header;
        self.markdown_bold_color = bold;
        self.markdown_italic_color = italic;
        self.markdown_code_color = code;
        self.markdown_link_color = link;
        self.markdown_quote_color = quote;
        self.markdown_list_color = list;
    }

    /// Toggle markdown syntax coloring
    fn toggle_markdown_syntax_coloring(&mut self) {
        self.markdown_syntax_coloring = !self.markdown_syntax_coloring;
    }

    /// Set error and warning colors
    fn set_error_warning_colors(&mut self, error: String, warning: String) {
        self.error_color = error;
        self.warning_color = warning;
    }

    /// Toggle diagnostics highlighting
    fn toggle_diagnostics_highlighting(&mut self) {
        self.diagnostics_highlighting = !self.diagnostics_highlighting;
    }

    /// Set search match highlight color
    fn set_search_match_color(&mut self, color: String) {
        self.search_match_color = color;
    }

    /// Set whitespace/indent guide color
    fn set_whitespace_guide_color(&mut self, color: String) {
        self.whitespace_guide_color = color;
    }

    /// Toggle whitespace/indent guides
    fn toggle_whitespace_guides(&mut self) {
        self.show_whitespace_guides = !self.show_whitespace_guides;
    }

    /// Set active line background color
    fn set_active_line_bg_color(&mut self, color: String) {
        self.active_line_bg_color = color;
    }

    /// Toggle active line background
    fn toggle_active_line_bg(&mut self) {
        self.show_active_line_bg = !self.show_active_line_bg;
    }

    /// Set inactive line background color
    fn set_inactive_line_bg_color(&mut self, color: String) {
        self.inactive_line_bg_color = color;
    }

    /// Toggle inactive line background
    fn toggle_inactive_line_bg(&mut self) {
        self.show_inactive_line_bg = !self.show_inactive_line_bg;
    }
    /// Set gutter color
    fn set_gutter_color(&mut self, color: String) {
        self.gutter_color = color;
    }

    /// Set line number color
    fn set_line_number_color(&mut self, color: String) {
        self.line_number_color = color;
    }

    /// Set selected line number color
    fn set_selected_line_number_color(&mut self, color: String) {
        self.selected_line_number_color = color;
    }

    /// Set highlight color
    fn set_highlight_color(&mut self, color: String) {
        self.highlight_color = color;
    }
    /// Set foreground and background color
    fn set_colors(&mut self, fg: String, bg: String) {
        self.fg_color = fg;
        self.bg_color = bg;
    }

    /// Set font and font size

    /// Set line height
    fn set_line_height(&mut self, height: f64) {
        self.line_height = height;
    }

    /// Toggle syntax highlighting on/off
    fn toggle_syntax_highlighting(&mut self) {
        self.syntax_highlighting = !self.syntax_highlighting;
    }

    /// Toggle line highlight on/off
    fn toggle_line_highlight(&mut self) {
        self.highlight_line = !self.highlight_line;
    }
    /// Theme switching stub (light/dark/custom)
    fn switch_theme(&mut self, _theme_name: &str) {
        // Stub: apply theme (requires integration with GTK CSS)
    }

    /// Print/export stub (to PDF/HTML/Markdown)
    fn print_export(&self, _format: &str, _path: &str) {
        // Stub: export buffer to file (PDF/HTML/Markdown)
    }

    /// Status bar update stub (show cursor position, diagnostics, etc.)
    fn update_status_bar(&self) {
        // Stub: update status bar (requires integration with GTK UI)
    }

    /// IME (Input Method Editor) support stub
    fn ime_support(&self) {
        // Stub: handle IME events (requires GTK integration)
    }

    /// Accessibility support stub
    fn accessibility_support(&self) {
        // Stub: expose accessibility info (requires GTK integration)
    }

    /// Performance optimization stub
    fn optimize_performance(&mut self) {
        // Stub: optimize buffer and rendering for large files
    }
    /// Undo/redo selection and cursor position (stub)
    fn undo_selection_cursor(&mut self) {
        // Stub: store and restore selection/cursor history
    }
    fn redo_selection_cursor(&mut self) {
        // Stub: store and restore selection/cursor history
    }

    // ...existing code...
} // <-- Add this closing brace to end impl EditorBuffer

fn main() {
    let app = Application::builder()
        .application_id("com.example.gtkeditor")
        .build();

    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("GTK4-rs Editor")
            .default_width(800)
            .default_height(600)
            .build();

        let buffer = Rc::new(RefCell::new(EditorBuffer::new()));
        let drawing_area = DrawingArea::new();
        drawing_area.set_draw_func(move |_, ctx, width, height| {
            let buffer = buffer.borrow();
            super::render::render_editor(&buffer, ctx, width, height);
        });

        window.set_child(Some(&drawing_area));
        window.show();
    });

    app.run();
}

// ...existing code...
impl EditorBuffer {
    pub fn new() -> Self {
        use syntect::parsing::SyntaxSet;
        use syntect::highlighting::ThemeSet;
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme = ThemeSet::load_defaults().themes["InspiredGitHub"].clone();
        EditorBuffer {
            lines: vec!["fn main() {".to_string(), "    ".to_string(), "}".to_string()],
            cursor_row: 0,
            cursor_col: 0,
            scroll_offset: 0,
            highlight_line: true,
            selection_start: None,
            selection_end: None,
            multi_cursors: Vec::new(),
            multi_selections: Vec::new(),
            theme,
            syntax_set,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            word_wrap: true,
            font: "monospace".to_string(),
            font_size: 16.0,
            line_height: 20.0,
            character_spacing: 0.0,
            fg_color: "#222222".to_string(),
            bg_color: "#ffffff".to_string(),
            gutter_color: "#e0e0e0".to_string(),
            line_number_color: "#888888".to_string(),
            selected_line_number_color: "#0055aa".to_string(),
            highlight_color: "#cceeff".to_string(),
            syntax_highlighting: true,
            cursor_color: "#0055aa".to_string(),
            markdown_header_color: "#0055aa".to_string(),
            markdown_bold_color: "#222222".to_string(),
            markdown_italic_color: "#444444".to_string(),
            markdown_code_color: "#333333".to_string(),
            markdown_link_color: "#0088cc".to_string(),
            markdown_quote_color: "#888888".to_string(),
            markdown_list_color: "#0055aa".to_string(),
            markdown_syntax_coloring: true,
            error_color: "#ff3333".to_string(),
            warning_color: "#ffaa00".to_string(),
            diagnostics_highlighting: true,
            search_match_color: "#ffff99".to_string(),
            whitespace_guide_color: "#cccccc".to_string(),
            show_whitespace_guides: false,
            active_line_bg_color: "#f0f8ff".to_string(),
            show_active_line_bg: true,
            inactive_line_bg_color: "#f8f8f8".to_string(),
            show_inactive_line_bg: false,
            debug_mode: false,
            diagnostics: Vec::new(),
            a4_mode: false,
            top_margin_cm: 2.0,
            bottom_margin_cm: 2.0,
            left_margin_cm: 2.0,
            right_margin_cm: 2.0,
            redraw_callback: None,
        }
    }
}