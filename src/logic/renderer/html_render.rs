/// Convert a parser node to (Inline, SourcePos) for rendering
pub fn inline_node_to_inline(node: &crate::logic::core::inline::types::InlineNode) -> (Inline, SourcePos) {
    use crate::logic::core::inline::types::InlineNode;
    match node {
        InlineNode::Text { text, pos } => (Inline::Text(text.clone()), pos.clone()),
        InlineNode::Emphasis { children, pos } => {
            let inner: Vec<(Inline, SourcePos)> = children.iter().map(|n| inline_node_to_inline(n)).collect();
            (Inline::Emphasis(crate::logic::ast::inlines::Emphasis::Emph(inner, None)), pos.clone())
        }
        InlineNode::Strong { children, pos } => {
            let inner: Vec<(Inline, SourcePos)> = children.iter().map(|n| inline_node_to_inline(n)).collect();
            (Inline::Emphasis(crate::logic::ast::inlines::Emphasis::Strong(inner, None)), pos.clone())
        }
        InlineNode::Code { text, pos } => (Inline::CodeSpan(crate::logic::ast::inlines::CodeSpan {
            content: text.clone(),
            meta: None,
            attributes: None,
        }), pos.clone()),
        InlineNode::Link { href, title, children, pos } => {
            let label: Vec<(Inline, SourcePos)> = children.iter().map(|n| inline_node_to_inline(n)).collect();
            (Inline::Link(crate::logic::ast::inlines::Link {
                label,
                destination: crate::logic::ast::inlines::LinkDestination::Inline(href.clone()),
                title: if title.is_empty() { None } else { Some(title.clone()) },
                reference_type: None,
                attributes: None,
            }), pos.clone())
        }
        InlineNode::Image { src, alt, title, pos } => {
            let alt_inlines: Vec<(Inline, SourcePos)> = alt.iter().map(|n| inline_node_to_inline(n)).collect();
            (Inline::Image(crate::logic::ast::inlines::Image {
                alt: alt_inlines,
                destination: crate::logic::ast::inlines::LinkDestination::Inline(src.clone()),
                title: if title.is_empty() { None } else { Some(title.clone()) },
                attributes: None,
                alternative: None,
                resource: None,
            }), pos.clone())
        }
        InlineNode::Html { text, pos } => (Inline::RawHtml(text.clone()), pos.clone()),
        InlineNode::Entity { text, pos } => (Inline::Text(text.clone()), pos.clone()),
        InlineNode::AttributeBlock { text, pos } => (Inline::Text(text.clone()), pos.clone()),
        InlineNode::SoftBreak { pos } => (Inline::SoftBreak, pos.clone()),
        InlineNode::LineBreak { pos } => (Inline::HardBreak, pos.clone()),
        InlineNode::Math { text, pos } => (Inline::Text(text.clone()), pos.clone()),
        InlineNode::Strikethrough { children, pos } => {
            let inner: Vec<(Inline, SourcePos)> = children.iter().map(|n| inline_node_to_inline(n)).collect();
            (Inline::Strikethrough(inner, None), pos.clone())
        }
        InlineNode::TaskListItem { checked, children, pos } => {
            let inner: Vec<(Inline, SourcePos)> = children.iter().map(|n| inline_node_to_inline(n)).collect();
            (Inline::Text(format!("[{}] {}", if *checked { "x" } else { " " }, inner.iter().map(|(i, _)| format!("{:?}", i)).collect::<String>())), pos.clone())
        }
    }
}
pub struct HtmlRenderer;
// HTML renderer for basic Markdown elements
// Extensible design: add more elements as needed

use crate::logic::ast::blocks_and_inlines::{Block, LeafBlock, ContainerBlock, ListKind, AstVisitor};
use crate::logic::ast::inlines::Inline;
use crate::logic::core::event_types::SourcePos;

impl HtmlRenderer {
    /// Convert a parser node to (Inline, SourcePos) for rendering
    pub fn inline_node_to_inline(node: &crate::logic::core::inline::types::InlineNode) -> (Inline, SourcePos) {
        use crate::logic::core::inline::types::InlineNode;
        match node {
            InlineNode::Text { text, pos } => (Inline::Text(text.clone()), pos.clone()),
            InlineNode::Emphasis { children, pos } => {
                let inner: Vec<(Inline, SourcePos)> = children.iter().map(|n| HtmlRenderer::inline_node_to_inline(n)).collect();
                (Inline::Emphasis(crate::logic::ast::inlines::Emphasis::Emph(inner, None)), pos.clone())
            }
            InlineNode::Strong { children, pos } => {
                let inner: Vec<(Inline, SourcePos)> = children.iter().map(|n| HtmlRenderer::inline_node_to_inline(n)).collect();
                (Inline::Emphasis(crate::logic::ast::inlines::Emphasis::Strong(inner, None)), pos.clone())
            }
            InlineNode::Code { text, pos } => (Inline::CodeSpan(crate::logic::ast::inlines::CodeSpan { content: text.clone(), meta: None, attributes: None }), pos.clone()),
            InlineNode::Link { href, title, children, pos } => {
                let label: Vec<(Inline, SourcePos)> = children.iter().map(|n| HtmlRenderer::inline_node_to_inline(n)).collect();
                (Inline::Link(crate::logic::ast::inlines::Link {
                    label,
                    destination: crate::logic::ast::inlines::LinkDestination::Inline(href.clone()),
                    title: Some(title.clone()),
                    reference_type: None,
                    attributes: None,
                }), pos.clone())
            }
            InlineNode::Image { src, alt, title, pos } => {
                let alt_inlines: Vec<(Inline, SourcePos)> = alt.iter().map(|n| HtmlRenderer::inline_node_to_inline(n)).collect();
                (Inline::Image(crate::logic::ast::inlines::Image {
                    alt: alt_inlines,
                    destination: crate::logic::ast::inlines::LinkDestination::Inline(src.clone()),
                    title: Some(title.clone()),
                    attributes: None,
                    alternative: None,
                    resource: None,
                }), pos.clone())
            }
            InlineNode::Html { text, pos } => (Inline::RawHtml(text.clone()), pos.clone()),
            InlineNode::Entity { text, pos } => (Inline::Text(text.clone()), pos.clone()),
            InlineNode::AttributeBlock { text, pos } => (Inline::Text(text.clone()), pos.clone()),
            InlineNode::SoftBreak { pos } => (Inline::SoftBreak, pos.clone()),
            InlineNode::LineBreak { pos } => (Inline::HardBreak, pos.clone()),
            InlineNode::Math { text, pos } => (Inline::Text(text.clone()), pos.clone()),
            InlineNode::Strikethrough { children, pos } => {
                let inner: Vec<(Inline, SourcePos)> = children.iter().map(|n| HtmlRenderer::inline_node_to_inline(n)).collect();
                (Inline::Strikethrough(inner, None), pos.clone())
            }
            InlineNode::TaskListItem { checked, children, pos } => {
                let inner: Vec<(Inline, SourcePos)> = children.iter().map(|n| HtmlRenderer::inline_node_to_inline(n)).collect();
                (Inline::Text(format!("[{}] {}", if *checked { "x" } else { " " }, inner.iter().map(|(i, _)| format!("{:?}", i)).collect::<String>())), pos.clone())
            }
        }
    }

    /// Stub: Render a slice of Block AST nodes to HTML (for tests)
    pub fn render(_blocks: &[Block]) -> String {
        // TODO: Implement full block rendering logic
        String::new()
    }

    /// Render a slice of Inline AST nodes to HTML
    pub fn render_inlines(inlines: &[(Inline, SourcePos)]) -> String {
        let mut html = String::new();
        for (inline, _pos) in inlines {
            match inline {
                Inline::CodeSpan(code_span) => {
                    html.push_str(&format!("<code>{}</code>", html_escape::encode_text(&code_span.content)));
                }
                Inline::Emphasis(emph) => match emph {
                    crate::logic::ast::inlines::Emphasis::Emph(inner, _) => {
                        let inner_html = Self::render_inlines(inner);
                        if !inner_html.is_empty() {
                            html.push_str("<em>");
                            html.push_str(&inner_html);
                            html.push_str("</em>");
                        }
                    }
                    crate::logic::ast::inlines::Emphasis::Strong(inner, _) => {
                        let inner_html = Self::render_inlines(inner);
                        if !inner_html.is_empty() {
                            html.push_str("<strong>");
                            html.push_str(&inner_html);
                            html.push_str("</strong>");
                        }
                    }
                },
                Inline::Text(text) => {
                    html.push_str(&html_escape::encode_text(text));
                }
                Inline::Link(link) => {
                    let label = Self::render_inlines(&link.label);
                    let dest = match &link.destination {
                        crate::logic::ast::inlines::LinkDestination::Inline(s) => s,
                        crate::logic::ast::inlines::LinkDestination::Reference(s) => s,
                    };
                    html.push_str(&format!("<a href=\"{}\">{}</a>", html_escape::encode_double_quoted_attribute(dest), label));
                }
                Inline::Image(image) => {
                    let alt = Self::render_inlines(&image.alt);
                    let src = match &image.destination {
                        crate::logic::ast::inlines::LinkDestination::Inline(s) => s,
                        crate::logic::ast::inlines::LinkDestination::Reference(s) => s,
                    };
                    html.push_str(&format!("<img src=\"{}\" alt=\"{}\">", html_escape::encode_double_quoted_attribute(src), alt));
                }
                _ => {
                    // TODO: handle more inline types
                }
            }
        }
        html
    }
}