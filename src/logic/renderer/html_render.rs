// HTML renderer for basic Markdown elements
// Extensible design: add more elements as needed

use crate::logic::ast::blocks_and_inlines::{Block, LeafBlock, ContainerBlock, ListKind, AstVisitor};
use crate::logic::ast::inlines::Inline;
use crate::logic::core::event_types::SourcePos;

pub struct HtmlRenderer {
    pub html: String,
}

impl HtmlRenderer {
    pub fn new() -> Self {
        HtmlRenderer { html: String::new() }
    }
    pub fn render(blocks: &[Block]) -> String {
        let mut renderer = HtmlRenderer::new();
        for block in blocks {
            block.accept(&mut renderer);
        }
        renderer.html
    }
}

impl AstVisitor for HtmlRenderer {
    fn visit_block(&mut self, block: &Block) {
        match block {
            Block::Leaf(leaf) => self.visit_leaf_block(leaf),
            Block::Container(container) => self.visit_container_block(container),
        }
    }

    fn visit_container_block(&mut self, container: &ContainerBlock) {
        match container {
            ContainerBlock::Document(blocks, _) => {
                for block in blocks {
                    block.accept(self);
                }
            }
            ContainerBlock::List { kind, items, .. } => {
                let tag = match kind {
                    ListKind::Bullet { .. } => "ul",
                    ListKind::Ordered { .. } => "ol",
                };
                self.html.push_str(&format!("<{tag}>", tag=tag));
                for item in items {
                    item.accept(self);
                }
                self.html.push_str(&format!("</{tag}>\n", tag=tag));
            }
            ContainerBlock::ListItem { contents, task_checked, .. } => {
                if let Some(checked) = task_checked {
                    self.html.push_str("<li class=\"task-list-item\">");
                    self.html.push_str(&format!(
                        "<input type=\"checkbox\" disabled{}> ",
                        if *checked { " checked" } else { "" }
                    ));
                } else {
                    self.html.push_str("<li>");
                }
                for block in contents {
                    block.accept(self);
                }
                self.html.push_str("</li>\n");
            }
            _ => {}
        }
    }

    fn visit_leaf_block(&mut self, leaf: &LeafBlock) {
        match leaf {
            LeafBlock::Paragraph(inlines, _) => {
                self.html.push_str("<p>");
                self.html.push_str(&HtmlRenderer::render_inlines(inlines));
                self.html.push_str("</p>\n");
            }
            LeafBlock::Heading { level, content, .. } => {
                self.html.push_str(&format!("<h{lvl}>", lvl=level));
                self.html.push_str(&HtmlRenderer::render_inlines(content));
                self.html.push_str(&format!("</h{lvl}>\n", lvl=level));
            }
            LeafBlock::AtxHeading { level, raw_content, .. } => {
                self.html.push_str(&format!("<h{lvl}>{}</h{lvl}>\n", html_escape::encode_text(raw_content), lvl=level));
            }
            LeafBlock::SetextHeading { level, raw_content, .. } => {
                self.html.push_str(&format!("<h{lvl}>{}</h{lvl}>\n", html_escape::encode_text(raw_content), lvl=level));
            }
            LeafBlock::IndentedCodeBlock { content, .. } | LeafBlock::FencedCodeBlock { content, .. } => {
                self.html.push_str("<pre><code>");
                self.html.push_str(&html_escape::encode_text(content));
                self.html.push_str("</code></pre>\n");
            }
            LeafBlock::ThematicBreak { .. } => {
                self.html.push_str("<hr />\n");
            }
            LeafBlock::HtmlBlock { content, .. } => {
                self.html.push_str(content);
                self.html.push_str("\n");
            }
            LeafBlock::LinkReferenceDefinition { .. } => {
                // No direct HTML output
            }
            LeafBlock::BlankLine => {
                // No output for blank lines
            }
            LeafBlock::Math(math_block) => {
                self.html.push_str("<div class=\"math\">");
                self.html.push_str(&html_escape::encode_text(&math_block.content));
                self.html.push_str("</div>\n");
            }
            LeafBlock::CustomTagBlock { name, data, content, .. } => {
                self.html.push_str(&format!("<div class=\"custom-tag {}\">", html_escape::encode_text(name)));
                if let Some(d) = data {
                    self.html.push_str(&html_escape::encode_text(d));
                }
                for block in content {
                    block.accept(self);
                }
                self.html.push_str("</div>\n");
            }
            LeafBlock::Table { .. } => {
                // TODO: Implement table rendering
            }
        }
    }
    // ...existing code...
}

impl HtmlRenderer {

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
    use crate::logic::ast::blocks_and_inlines::{ContainerBlock, ListMarker, ListKind};

    #[test]
    fn test_gfm_task_list_items() {
        let checked_item = Block::Container(ContainerBlock::ListItem {
            marker: ListMarker::Bullet { char: '-' },
            contents: vec![Block::Leaf(LeafBlock::Paragraph(vec![(Inline::Text("checked item".into()), SourcePos::default())], None))],
            task_checked: Some(true),
            attributes: None,
        });
        let unchecked_item = Block::Container(ContainerBlock::ListItem {
            marker: ListMarker::Bullet { char: '-' },
            contents: vec![Block::Leaf(LeafBlock::Paragraph(vec![(Inline::Text("unchecked item".into()), SourcePos::default())], None))],
            task_checked: Some(false),
            attributes: None,
        });
        let regular_item = Block::Container(ContainerBlock::ListItem {
            marker: ListMarker::Bullet { char: '-' },
            contents: vec![Block::Leaf(LeafBlock::Paragraph(vec![(Inline::Text("regular item".into()), SourcePos::default())], None))],
            task_checked: None,
            attributes: None,
        });
        let list = Block::Container(ContainerBlock::List {
            kind: ListKind::Bullet { char: '-' },
            tight: true,
            items: vec![checked_item, unchecked_item, regular_item],
            attributes: None,
        });
        let html = HtmlRenderer::render(&[list]);
        assert!(html.contains("<input type=\"checkbox\" disabled checked> "));
        assert!(html.contains("<input type=\"checkbox\" disabled> "));
        assert!(html.contains("regular item"));
        assert!(html.contains("class=\"task-list-item\""));
    }
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
