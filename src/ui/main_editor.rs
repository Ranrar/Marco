/// This is the markdown editor

use crate::logic::renderer::traits::Renderer;
use crate::logic::renderer::html_render::HtmlRenderer;
use crate::logic::renderer::main_editor_render::GtkSourceViewRenderer;
use crate::logic::core::inline::parser::parse_phrases;
use webkit6::prelude::*;
use gtk4::Paned;
use crate::ui::html_viewer::wrap_html_document;
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
    let initial_blocks = [parse_markdown("# Title")];
    let initial_html = wrap_html_document(HtmlRenderer::render(&initial_blocks).as_str());
    let webview = crate::ui::html_viewer::create_html_viewer(&initial_html);
    paned.set_end_child(Some(&webview));

    // Live update: on buffer change, re-render and update WebView
    let webview_clone = webview.clone();
    buffer.connect_changed(move |buf| {
        let text = buf.text(&buf.start_iter(), &buf.end_iter(), false).to_string();
        let ast = parse_markdown(&text);
        println!("[DEBUG] AST: {:#?}", ast);
        let blocks = [ast.clone()];
        let html = wrap_html_document(HtmlRenderer::render(&blocks).as_str());
        webview_clone.load_html(&html, None);
    });

// Dummy parser for now. Replace with your real Markdown parser implementation.
fn parse_markdown(input: &str) -> Block {
    use crate::logic::ast::blocks_and_inlines::{Block, LeafBlock};
    use crate::logic::ast::inlines::Inline;
    use crate::logic::core::inline::types::InlineNode;
    let mut blocks = Vec::new();

    fn convert_inlines(nodes: Vec<InlineNode>) -> Vec<(Inline, crate::logic::core::event_types::SourcePos)> {
        use crate::logic::ast::inlines::{Inline, Emphasis as EmphEnum, CodeSpan, Link as LinkStruct, LinkDestination, Image as ImageStruct};
        use crate::logic::ast::math::MathInline;
        nodes.into_iter().filter_map(|node| match node {
            InlineNode::Text { text, pos } => Some((Inline::Text(text), pos)),
            InlineNode::Emphasis { children, pos } => Some((
                Inline::Emphasis(EmphEnum::Emph(convert_inlines(children), None)),
                pos)),
            InlineNode::Strong { children, pos } => Some((
                Inline::Emphasis(EmphEnum::Strong(convert_inlines(children), None)),
                pos)),
            InlineNode::Code { text, pos } => Some((
                Inline::CodeSpan(CodeSpan { content: text, attributes: None }),
                pos)),
            InlineNode::Link { href, title, children, pos } => Some((
                Inline::Link(LinkStruct {
                    label: convert_inlines(children),
                    destination: LinkDestination::Inline(href),
                    title: if title.is_empty() { None } else { Some(title) },
                    attributes: None,
                }),
                pos)),
            InlineNode::Image { src, alt, title, pos } => Some((
                Inline::Image(ImageStruct {
                    alt: convert_inlines(alt),
                    destination: LinkDestination::Inline(src),
                    title: if title.is_empty() { None } else { Some(title) },
                    attributes: None,
                }),
                pos)),
            InlineNode::Math { text, pos } => Some((
                Inline::Math(MathInline {
                    content: text,
                    math_type: crate::logic::ast::math::MathType::LaTeX,
                    position: Some(pos),
                    attributes: None,
                }),
                pos)),
            InlineNode::Html { text, pos } => Some((Inline::RawHtml(text), pos)),
            InlineNode::SoftBreak { pos } => Some((Inline::SoftBreak, pos)),
            InlineNode::LineBreak { pos } => Some((Inline::HardBreak, pos)),
            // Entity and AttributeBlock are not directly supported in Inline; skip or map as needed
            _ => None,
        }).collect()
    }

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let hashes = trimmed.chars().take_while(|&c| c == '#').count();
        if hashes > 0 && trimmed.chars().nth(hashes) == Some(' ') {
            // Heading
            let level = hashes as u8;
            let content = &trimmed[hashes+1..];
            let inlines = parse_phrases(content);
            let inlines = convert_inlines(inlines);
            blocks.push(Block::Leaf(LeafBlock::Heading {
                level,
                content: inlines,
                attributes: None,
            }));
            continue;
        }
        // Paragraph
        let inlines = parse_phrases(trimmed);
        let inlines = convert_inlines(inlines);
        blocks.push(Block::Leaf(LeafBlock::Paragraph(inlines, None)));
    }
    let ast = if blocks.is_empty() {
        Block::Leaf(LeafBlock::Paragraph(vec![], None))
    } else if blocks.len() == 1 {
        blocks.remove(0)
    } else {
        use crate::logic::ast::blocks_and_inlines::ContainerBlock;
        Block::Container(ContainerBlock::Document(blocks, None))
    };
    println!("[PARSE DEBUG] AST: {:#?}", ast);
    ast
}

    paned
}
// src/markdown/edit.rs

use gtk4 as gtk;
// use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, ScrolledWindow};
use sourceview5 as gtk_sourceview5;
use gtk_sourceview5::{Buffer as SourceBuffer, View as SourceView};
use crate::logic::core::block_parser::EventIter;
use crate::logic::ast::blocks_and_inlines::Block;

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

    // Use GtkSourceViewRenderer for syntax highlighting and error annotation
    let mut gtk_renderer = GtkSourceViewRenderer::new();
    gtk_renderer.render(ast).unwrap();
    // TODO: Use gtk_renderer to update SourceView with highlights/errors

    // Put the SourceView in a ScrolledWindow
    let scrolled = ScrolledWindow::new();
    scrolled.set_child(Some(&source_view));
    scrolled.set_vexpand(true);

    // Add the ScrolledWindow (with SourceView) to the top
    container.append(&scrolled);

    // Use the event stream to display a label for each event (for debugging/demo)
    let mut diagnostics = crate::logic::core::diagnostics::Diagnostics::new();
    for event in EventIter::new(ast, Some(&mut diagnostics)) {
        let label = Label::new(Some(&format!("{:?}", event)));
        container.append(&label);
    }

    (container, buffer)
}

// TODO: Implement flatten_text for BlockNode/Inline structure
