use crate::editor::logic::inline::parse_inlines;
use webkit6::WebView;
use webkit6::prelude::*;
use gtk4::Paned;
use crate::editor::logic::renderer::render;
use crate::viewer::html::wrap_html_document;
/// Create a split editor with live HTML preview using WebKit6
pub fn create_editor_with_preview(ast: &Block) -> Paned {
    let paned = Paned::new(gtk::Orientation::Horizontal);
    paned.set_position(600); // Set initial split position

    // Editor (left)
    let (editor_widget, buffer) = render_editor(ast);
    editor_widget.set_hexpand(true);
    editor_widget.set_vexpand(true);
    paned.set_start_child(Some(&editor_widget));

    // WebView (right)
    let webview = WebView::new();
    webview.set_hexpand(true);
    webview.set_vexpand(true);
    paned.set_end_child(Some(&webview));

    // Initial HTML preview
    let initial_html = wrap_html_document(&render(ast));
    webview.load_html(&initial_html, None);

    // Live update: on buffer change, re-render and update WebView
    let webview_clone = webview.clone();
    buffer.connect_changed(move |buf| {
        let text = buf.text(&buf.start_iter(), &buf.end_iter(), false).to_string();
        // Use your real Markdown parser here:
        let ast = parse_markdown(&text); // <-- Implement this function!
        println!("[DEBUG] AST: {:#?}", ast);
        let html = wrap_html_document(&render(&ast));
        println!("[DEBUG] HTML: {}", html);
        webview_clone.load_html(&html, None);
    });

// Dummy parser for now. Replace with your real Markdown parser implementation.
fn parse_markdown(input: &str) -> Block {
    use crate::editor::logic::ast::blocks_and_inlines::{Block, LeafBlock};
    if input.trim().is_empty() {
        Block::Leaf(LeafBlock::Paragraph(vec![]))
    } else {
        Block::Leaf(LeafBlock::Paragraph(parse_inlines(input)))
    }
}

    paned
}
// src/markdown/edit.rs

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, ScrolledWindow};
use sourceview5 as gtk_sourceview5;
use gtk_sourceview5::{Buffer as SourceBuffer, View as SourceView};
use crate::editor::logic::parser::EventIter;
use crate::editor::logic::ast::blocks_and_inlines::Block;

pub fn render_editor(ast: &Block) -> (GtkBox, SourceBuffer) {
    let container = GtkBox::new(gtk::Orientation::Vertical, 6);

    // Create a SourceBuffer and SourceView
    let buffer = SourceBuffer::new(None);
    buffer.set_text("Type here...");
    let source_view = SourceView::new();
    source_view.set_buffer(Some(&buffer));
    source_view.set_monospace(true);
    source_view.set_vexpand(true);
    source_view.set_editable(true);

    // Put the SourceView in a ScrolledWindow
    let scrolled = ScrolledWindow::new();
    scrolled.set_child(Some(&source_view));
    scrolled.set_vexpand(true);

    // Add the ScrolledWindow (with SourceView) to the top
    container.append(&scrolled);

    // Use the event stream to display a label for each event (for debugging/demo)
    for event in EventIter::new(ast) {
        let label = Label::new(Some(&format!("{:?}", event)));
        container.append(&label);
    }

    (container, buffer)
}

// TODO: Implement flatten_text for BlockNode/Inline structure
