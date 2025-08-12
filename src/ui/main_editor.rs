/// This is the markdown editor

use webkit6::prelude::*;
use gtk4::Paned;
use crate::ui::html_viewer::wrap_html_document;
use markdown::to_html;
/// Create a split editor with live HTML preview using WebKit6
use std::rc::Rc;
use std::cell::RefCell;
pub fn create_editor_with_preview(preview_theme_filename: &str, preview_theme_dir: &str, theme_mode: Rc<RefCell<String>>) -> (Paned, webkit6::WebView, Rc<RefCell<String>>, Box<dyn Fn()>) {
    let paned = Paned::new(gtk4::Orientation::Horizontal);
    paned.set_position(600);

    // Editor (left)
    let (editor_widget, buffer, source_view) = render_editor_with_view();
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

    // Closure to refresh preview and update SourceView colors
    let refresh_preview = {
        let buffer = Rc::clone(&buffer_rc);
        let css = Rc::clone(&css_rc);
        let webview = Rc::clone(&webview_rc);
        let theme_mode = Rc::clone(&theme_mode_rc);
        let source_view = source_view.clone();
        move || {
            let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false).to_string();
            let html = wrap_html_document(&to_html(&text), &css.borrow(), &theme_mode.borrow());
            webview.load_html(&html, None);

            // Apply palette to SourceView
            use crate::theme::{LIGHT_PALETTE, DARK_PALETTE};
            let palette = match theme_mode.borrow().to_lowercase().as_str() {
                "dark" => &DARK_PALETTE,
                _ => &LIGHT_PALETTE,
            };
            use gtk4::CssProvider;
            let css_string = format!(
                ".sourceview {{ background-color: {}; color: {}; }}",
                palette.background, palette.text
            );
            let provider = CssProvider::new();
            provider.load_from_data(css_string.as_str());
            source_view.style_context().add_provider(
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
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

    // Return the paned, webview, and refresh closure (boxed)
    (paned, webview, css_rc, Box::new(refresh_preview) as Box<dyn Fn()>)
}
// src/markdown/edit.rs

use sourceview5::prelude::*; // For set_show_line_numbers

pub fn render_editor_with_view() -> (gtk4::Box, sourceview5::Buffer, sourceview5::View) {
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

    // Put the SourceView in a ScrolledWindow
    let scrolled = gtk4::ScrolledWindow::new();
    scrolled.set_child(Some(&source_view));
    scrolled.set_vexpand(true);

    // Add the ScrolledWindow (with SourceView) to the top
    container.append(&scrolled);

    (container, buffer, source_view)
}