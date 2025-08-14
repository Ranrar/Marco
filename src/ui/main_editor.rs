use crate::footer::{FooterLabels, update_cursor_pos, update_line_count, update_encoding, update_insert_mode};
// No need to import source_remove; use SourceId::remove()
use glib::ControlFlow;
/// Wires up debounced footer updates to buffer and view events
pub fn wire_footer_updates(buffer: &sourceview5::Buffer, view: &sourceview5::View, labels: Rc<FooterLabels>) {
    use std::cell::Cell;
    let debounce_ms = 300;
    let timeout_id: Rc<Cell<Option<glib::SourceId>>> = Rc::new(Cell::new(None));

    let update_footer = {
        let buffer = buffer.clone();
        let view = view.clone();
        let labels = Rc::clone(&labels);
        move || {
            // Get cursor position
            let offset = buffer.cursor_position();
            let iter = buffer.iter_at_offset(offset);
            let row = iter.line() + 1;
            let col = iter.line_offset() + 1;
            update_cursor_pos(&labels, row as usize, col as usize);

            // Get line count
            let lines = buffer.line_count();
            update_line_count(&labels, lines as usize);

            // Encoding (assume UTF-8 for now)
            update_encoding(&labels, "UTF-8");

            // Insert/overwrite mode (assume insert for now, can be wired to actual mode)
            update_insert_mode(&labels, true);

            // Get buffer text for word/char count
            let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false).to_string();
            let word_count = text.split_whitespace().filter(|w| !w.is_empty()).count();
            let char_count = text.chars().count();
            crate::footer::update_word_count(&labels, word_count);
            crate::footer::update_char_count(&labels, char_count);

            // Syntax trace for current line (dummy map for now)
            let current_line = iter.line();
            let start_iter_opt = buffer.iter_at_line(current_line);
            let end_iter_opt = buffer.iter_at_line(current_line + 1);
            let line_text = match (start_iter_opt, end_iter_opt) {
                (Some(ref start), Some(ref end)) => buffer.text(start, end, false).to_string(),
                (Some(ref start), None) => buffer.text(start, &buffer.end_iter(), false).to_string(),
                _ => String::new(),
            };
            let dummy_map = crate::logic::parser::MarkdownSyntaxMap { rules: std::collections::HashMap::new() };
            crate::footer::update_syntax_trace(&labels, &line_text, &dummy_map);
        }
    };

    // Debounce logic for buffer changes
    let buffer_clone = buffer.clone();
    let timeout_id_clone: Rc<Cell<Option<glib::SourceId>>> = Rc::clone(&timeout_id);
    let update_footer_clone = update_footer.clone();
    buffer.connect_changed(move |_| {
        if let Some(id) = timeout_id_clone.take() {
            id.remove();
        }
        let update_footer_clone = update_footer_clone.clone();
        let id = glib::timeout_add_local(std::time::Duration::from_millis(debounce_ms), move || {
            update_footer_clone();
            ControlFlow::Break
        });
        timeout_id_clone.set(Some(id));
    });

    // Debounce logic for cursor movement
    let timeout_id_clone2: Rc<Cell<Option<glib::SourceId>>> = Rc::clone(&timeout_id);
    let update_footer_clone2 = update_footer.clone();
    view.connect_move_cursor(move |_, _, _, _| {
        if let Some(id) = timeout_id_clone2.take() {
            id.remove();
        }
        let update_footer_clone2 = update_footer_clone2.clone();
        let id = glib::timeout_add_local(std::time::Duration::from_millis(debounce_ms), move || {
            update_footer_clone2();
            ControlFlow::Break
        });
        timeout_id_clone2.set(Some(id));
    });

    // Initial update
    update_footer();
}
/// This is the markdown editor

use webkit6::prelude::*;
use gtk4::Paned;
use crate::ui::html_viewer::wrap_html_document;
use markdown::to_html;
/// Create a split editor with live HTML preview using WebKit6
use std::rc::Rc;
use std::cell::RefCell;
pub fn create_editor_with_preview(
    preview_theme_filename: &str,
    preview_theme_dir: &str,
    theme_manager: Rc<RefCell<crate::theme::ThemeManager>>,
    theme_mode: Rc<RefCell<String>>
) -> (Paned, webkit6::WebView, Rc<RefCell<String>>, Box<dyn Fn()>, Box<dyn Fn(&str)>, Box<dyn Fn(&str)>) {
    let paned = Paned::new(gtk4::Orientation::Horizontal);
    paned.set_position(600);

    // Get style scheme and font settings from ThemeManager
    let (style_scheme, font_family, font_size_pt) = {
        let tm = theme_manager.borrow();
        let style_scheme = tm.current_editor_scheme();
        let font_family = tm.settings.appearance.as_ref()
            .and_then(|a| a.ui_font.as_deref())
            .unwrap_or("Fira Mono").to_string();
        let font_size_pt = tm.settings.appearance.as_ref()
            .and_then(|a| a.ui_font_size)
            .map(|v| v as f64)
            .unwrap_or(14.0);
        (style_scheme, font_family, font_size_pt)
    };

    // Editor (left)
        let (editor_widget, buffer, _) = render_editor_with_view(style_scheme.as_ref(), &font_family, font_size_pt);
    editor_widget.set_hexpand(true);
    editor_widget.set_vexpand(true);
    paned.set_start_child(Some(&editor_widget));

    // Load the current HTML preview theme CSS
    use std::fs;
    use std::path::Path;
    let css_path = Path::new(preview_theme_dir).join(preview_theme_filename);
    let css = fs::read_to_string(&css_path).unwrap_or_else(|_| String::from("body { background: #fff; color: #222; }"));

    // WebView (right)
    let initial_html = wrap_html_document("", &css, &theme_mode.borrow());
    let webview = crate::ui::html_viewer::create_html_viewer(&initial_html);
    paned.set_end_child(Some(&webview));

    // Shared state for refresh
    let buffer_rc = Rc::new(buffer);
    let css_rc = Rc::new(RefCell::new(css));
    let webview_rc = Rc::new(webview.clone());
    let theme_mode_rc = Rc::clone(&theme_mode);

    // Closure to refresh preview
    let refresh_preview = {
        let buffer = Rc::clone(&buffer_rc);
        let css = Rc::clone(&css_rc);
        let webview = Rc::clone(&webview_rc);
        let theme_mode = Rc::clone(&theme_mode_rc);
        move || {
            let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false).to_string();
            let html = wrap_html_document(&to_html(&text), &css.borrow(), &theme_mode.borrow());
            webview.load_html(&html, None);
        }
    };

    // Live update: on buffer change, re-render and update WebView
    let css_clone = Rc::clone(&css_rc);
    let theme_mode = Rc::clone(&theme_mode_rc);
    let webview_clone = Rc::clone(&webview_rc);
    let buffer_for_signal = Rc::clone(&buffer_rc);
    buffer_for_signal.connect_changed(move |buf| {
        let text = buf.text(&buf.start_iter(), &buf.end_iter(), false).to_string();
        let html = wrap_html_document(&to_html(&text), &css_clone.borrow(), &theme_mode.borrow());
        webview_clone.load_html(&html, None);
    });

    // Create theme update function for editor
    let buffer_for_theme = Rc::clone(&buffer_rc);
    let theme_manager_clone = Rc::clone(&theme_manager);
    let update_theme = Box::new(move |scheme_id: &str| {
        if let Some(scheme) = theme_manager_clone.borrow().get_editor_scheme(scheme_id) {
            buffer_for_theme.set_style_scheme(Some(&scheme));
            println!("Applied theme '{}' to editor buffer", scheme_id);
        } else {
            eprintln!("Failed to find style scheme: {}", scheme_id);
        }
    }) as Box<dyn Fn(&str)>;

    // Create HTML preview theme update function
    let theme_mode_for_preview = Rc::clone(&theme_mode_rc);
    let theme_manager_for_preview = Rc::clone(&theme_manager);
    let refresh_for_preview = {
        let buffer = Rc::clone(&buffer_rc);
        let css = Rc::clone(&css_rc);
        let webview = Rc::clone(&webview_rc);
        let theme_mode = Rc::clone(&theme_mode_rc);
        move || {
            let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false).to_string();
            let html = wrap_html_document(&to_html(&text), &css.borrow(), &theme_mode.borrow());
            webview.load_html(&html, None);
        }
    };
    let update_preview_theme = Box::new(move |scheme_id: &str| {
        let new_theme_mode = theme_manager_for_preview.borrow().preview_theme_mode_from_scheme(scheme_id);
        *theme_mode_for_preview.borrow_mut() = new_theme_mode;
        // Trigger refresh to apply the new theme mode
        refresh_for_preview();
        println!("Applied preview theme mode for scheme '{}'", scheme_id);
    }) as Box<dyn Fn(&str)>;

    // Return the paned, webview, refresh closure, editor theme update, and preview theme update
    (paned, webview, css_rc, Box::new(refresh_preview) as Box<dyn Fn()>, update_theme, update_preview_theme)
}
use sourceview5::prelude::*; // For set_show_line_numbers

pub fn render_editor_with_view(
    style_scheme: Option<&sourceview5::StyleScheme>,
    font_family: &str,
    font_size_pt: f64
) -> (gtk4::Box, sourceview5::Buffer, sourceview5::View) {
    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    // Create a SourceBuffer and SourceView
    let buffer = sourceview5::Buffer::new(None);
    buffer.set_text("");
    let source_view = sourceview5::View::new();
    source_view.set_buffer(Some(&buffer));
    source_view.set_monospace(true);
    source_view.set_vexpand(true);
    source_view.set_editable(true);
    source_view.set_show_line_numbers(true);
    source_view.set_highlight_current_line(false);
    source_view.set_show_line_marks(true); //TODO for bookmarks
    
    // Apply the style scheme if available
    if let Some(scheme) = style_scheme {
        use sourceview5::prelude::*;
        buffer.set_style_scheme(Some(scheme));
    }
    
    // Apply font settings via CSS (style schemes don't control font)
    use gtk4::CssProvider;
    let css = format!(
        ".sourceview {{ font-family: '{}', 'monospace'; font-size: {}pt; }}",
        font_family,
        font_size_pt
    );
    let provider = CssProvider::new();
    provider.connect_parsing_error(|_provider, section, error| {
        eprintln!("[Theme] CSS parsing error in SourceView: {:?} at {:?}", error, section);
    });
    provider.load_from_data(&css);
    source_view.style_context().add_provider(&provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

    // Make sure CSS can override background
    use sourceview5::BackgroundPatternType;
    source_view.set_background_pattern(BackgroundPatternType::None);

    // Optionally style the ScrolledWindow for visibility (no border for clarity)
    let scrolled = gtk4::ScrolledWindow::new();
    scrolled.set_child(Some(&source_view));
    scrolled.set_vexpand(true);
    scrolled.add_css_class("sourceview-scroll");
    let scrolled_css = r#"
    .sourceview-scroll {
        background: transparent;
    }
    "#;
    let scrolled_provider = CssProvider::new();
    scrolled_provider.load_from_data(scrolled_css);
    scrolled.style_context().add_provider(&scrolled_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

    // Add the ScrolledWindow (with SourceView) to the top
    container.append(&scrolled);

    (container, buffer, source_view)
}