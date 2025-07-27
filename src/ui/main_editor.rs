/// This is the markdown editor

use webkit6::prelude::*;
use gtk4::Paned;
use crate::ui::html_viewer::wrap_html_document;
use markdown::to_html;
/// Create a split editor with live HTML preview using WebKit6
pub fn create_editor_with_preview() -> (Paned, webkit6::WebView) {
    let paned = Paned::new(gtk4::Orientation::Horizontal);
    paned.set_position(600);

    // Editor (left)
    let (editor_widget, buffer) = render_editor();
    editor_widget.set_hexpand(true);
    editor_widget.set_vexpand(true);
    paned.set_start_child(Some(&editor_widget));

    // WebView (right)
    let initial_html = wrap_html_document("");
    let webview = crate::ui::html_viewer::create_html_viewer(&initial_html);
    paned.set_end_child(Some(&webview));

    // Live update: on buffer change, re-render and update WebView
    let webview_clone = webview.clone();
    buffer.connect_changed(move |buf| {
        let text = buf.text(&buf.start_iter(), &buf.end_iter(), false).to_string();
        let html = wrap_html_document(&to_html(&text));
        webview_clone.load_html(&html, None);
    });

    (paned, webview)
}
// src/markdown/edit.rs



use sourceview5::prelude::*; // For set_show_line_numbers

pub fn render_editor() -> (gtk4::Box, sourceview5::Buffer) {
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

    // Put the SourceView in a ScrolledWindow
    let scrolled = gtk4::ScrolledWindow::new();
    scrolled.set_child(Some(&source_view));
    scrolled.set_vexpand(true);

    // Add the ScrolledWindow (with SourceView) to the top
    container.append(&scrolled);

    (container, buffer)
}