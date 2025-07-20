// HTML renderer for basic Markdown elements
// Extensible design: add more elements as needed

use crate::logic::ast::blocks_and_inlines::{Block, LeafBlock};
use crate::logic::ast::inlines::Inline;
use crate::logic::core::event_types::SourcePos;

pub struct HtmlRenderer;

impl HtmlRenderer {
    /// Render a slice of Block AST nodes to HTML
    pub fn render(blocks: &[Block]) -> String {
        let mut html = String::new();
        for block in blocks {
            html.push_str(&Self::render_block(block));
        }
        html
    }

    /// Render a Block AST node to HTML
    pub fn render_block(block: &Block) -> String {
        match block {
            Block::Leaf(leaf) => Self::render_leaf_block(leaf),
            Block::Container(container) => {
                println!("[HTML DEBUG] Block::Container encountered: {:#?}", container);
                // For now, render all contained blocks recursively
                use crate::logic::ast::blocks_and_inlines::ContainerBlock;
                match container {
                    ContainerBlock::Document(blocks, _) => {
                        let mut html = String::new();
                        for block in blocks {
                            html.push_str(&Self::render_block(block));
                        }
                        html
                    }
                    _ => {
                        println!("[HTML DEBUG] Unknown container block type");
                        String::new()
                    }
                }
            }
        }
    }

    /// Render a LeafBlock AST node to HTML
    pub fn render_leaf_block(leaf: &LeafBlock) -> String {
        match leaf {
            LeafBlock::Paragraph(inlines, _) => {
                format!("<p>{}</p>\n", Self::render_inlines(inlines))
            }
            LeafBlock::Heading { level, content, .. } => {
                let tag = match level {
                    1 => "h1",
                    2 => "h2",
                    3 => "h3",
                    4 => "h4",
                    5 => "h5",
                    6 => "h6",
                    _ => "h1",
                };
                format!("<{tag}>{}</{tag}>\n", Self::render_inlines(content), tag=tag)
            }
            LeafBlock::IndentedCodeBlock { content, .. } => {
                format!("<pre><code>{}</code></pre>\n", html_escape::encode_text(content))
            }
            LeafBlock::FencedCodeBlock { content, .. } => {
                format!("<pre><code>{}</code></pre>\n", html_escape::encode_text(content))
            }
            other => {
                println!("[HTML DEBUG] Unknown leaf block: {:#?}", other);
                String::new()
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::ast::inlines::{Inline, CodeSpan, Link, Image, LinkDestination};
    use crate::logic::core::event_types::SourcePos;
    use crate::logic::ast::blocks_and_inlines::{Block, LeafBlock};

    #[test]
    fn test_heading() {
        assert_eq!(HtmlRenderer::render(&[Block::Leaf(LeafBlock::Heading {
            level: 1,
            content: vec![(Inline::Text("Title".into()), SourcePos::default())],
            attributes: None,
        })]), "<h1>Title</h1>\n");
        assert_eq!(HtmlRenderer::render(&[Block::Leaf(LeafBlock::Heading {
            level: 2,
            content: vec![(Inline::Text("Subtitle".into()), SourcePos::default())],
            attributes: None,
        })]), "<h2>Subtitle</h2>\n");
        assert_eq!(HtmlRenderer::render(&[Block::Leaf(LeafBlock::Heading {
            level: 3,
            content: vec![(Inline::Text("Subsubtitle".into()), SourcePos::default())],
            attributes: None,
        })]), "<h3>Subsubtitle</h3>\n");
    }

    #[test]
    fn test_bold_italic() {
        assert_eq!(HtmlRenderer::render(&[Block::Leaf(LeafBlock::Paragraph(vec![
            (Inline::Emphasis(crate::logic::ast::inlines::Emphasis::Strong(vec![(Inline::Text("bold".into()), SourcePos::default())], None)), SourcePos::default())
        ], None))]), "<p><strong>bold</strong></p>\n");
        assert_eq!(HtmlRenderer::render(&[Block::Leaf(LeafBlock::Paragraph(vec![
            (Inline::Emphasis(crate::logic::ast::inlines::Emphasis::Emph(vec![(Inline::Text("italic".into()), SourcePos::default())], None)), SourcePos::default())
        ], None))]), "<p><em>italic</em></p>\n");
    }

    #[test]
    fn test_inline_code() {
        assert_eq!(HtmlRenderer::render(&[Block::Leaf(LeafBlock::Paragraph(vec![
            (Inline::CodeSpan(CodeSpan { content: "code".into(), attributes: None }), SourcePos::default())
        ], None))]), "<p><code>code</code></p>\n");
    }

    #[test]
    fn test_link_image() {
        assert_eq!(HtmlRenderer::render(&[Block::Leaf(LeafBlock::Paragraph(vec![
            (Inline::Link(Link {
                label: vec![(Inline::Text("title".into()), SourcePos::default())],
                destination: LinkDestination::Inline("url".into()),
                title: None,
                attributes: None,
            }), SourcePos::default())
        ], None))]), "<p><a href=\"url\">title</a></p>\n");
        assert_eq!(HtmlRenderer::render(&[Block::Leaf(LeafBlock::Paragraph(vec![
            (Inline::Image(Image {
                alt: vec![(Inline::Text("alt".into()), SourcePos::default())],
                destination: LinkDestination::Inline("src".into()),
                title: None,
                attributes: None,
            }), SourcePos::default())
        ], None))]), "<p><img src=\"src\" alt=\"alt\"></p>\n");
    }

    #[test]
    fn test_paragraph() {
        assert_eq!(HtmlRenderer::render(&[Block::Leaf(LeafBlock::Paragraph(vec![
            (Inline::Text("plain text".into()), SourcePos::default())
        ], None))]), "<p>plain text</p>\n");
    }
}
