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

use crate::logic::ast::blocks_and_inlines::{Block, LeafBlock, ContainerBlock, ListKind};
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
    pub fn render(blocks: &[Block]) -> String {
        Self::render_blocks(blocks)
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
                Inline::ImageReference(image_ref) => {
                    let alt = Self::render_inlines(&image_ref.alt);
                    let src = match &image_ref.destination {
                        crate::logic::ast::inlines::LinkDestination::Inline(s) => s,
                        crate::logic::ast::inlines::LinkDestination::Reference(s) => s,
                    };
                    html.push_str(&format!("<img src=\"{}\" alt=\"{}\">", html_escape::encode_double_quoted_attribute(src), alt));
                }
                Inline::Autolink(autolink) => match autolink {
                    crate::logic::ast::inlines::Autolink::Uri(uri) => {
                        html.push_str(&format!("<a href=\"{}\">{}</a>", html_escape::encode_double_quoted_attribute(uri), html_escape::encode_text(uri)));
                    }
                    crate::logic::ast::inlines::Autolink::Email(email) => {
                        html.push_str(&format!("<a href=\"mailto:{}\">{}</a>", html_escape::encode_double_quoted_attribute(email), html_escape::encode_text(email)));
                    }
                },
                Inline::RawHtml(raw) => {
                    html.push_str(raw);
                }
                Inline::HardBreak => {
                    html.push_str("<br />");
                }
                Inline::SoftBreak => {
                    html.push_str("\n");
                }
                Inline::Math(math) => {
                    html.push_str(&format!("<span class=\"math\">{}</span>", html_escape::encode_text(&math.content)));
                }
                Inline::Strikethrough(inner, _) => {
                    let inner_html = Self::render_inlines(inner);
                    html.push_str(&format!("<del>{}</del>", inner_html));
                }
                Inline::Emoji(shortcode, unicode, _) => {
                    html.push_str(&format!("<span class=\"emoji\" title=\"{}\">{}</span>", html_escape::encode_double_quoted_attribute(shortcode), html_escape::encode_text(unicode)));
                }
                Inline::Mention(username, _) => {
                    html.push_str(&format!("<span class=\"mention\">@{}</span>", html_escape::encode_text(username)));
                }
                Inline::TableCaption(caption, _, _) => {
                    html.push_str(&format!("<caption>{}</caption>", html_escape::encode_text(caption)));
                }
                Inline::TaskListMeta(label, _, _) => {
                    let checked = label.as_ref().map_or(false, |l| l == "x" || l == "X");
                    html.push_str(&format!("<input type=\"checkbox\" {} disabled>", if checked { "checked" } else { "" }));
                }
            }
        }
        html
    }

    /// Render a slice of Block AST nodes to HTML
    pub fn render_blocks(blocks: &[Block]) -> String {
        let mut html = String::new();
        for block in blocks {
            match block {
                Block::Container(container) => {
                    match container {
                        ContainerBlock::Document(children, _) => {
                            html.push_str(&Self::render_blocks(children));
                        }
                        ContainerBlock::BlockQuote(children, _) => {
                            html.push_str("<blockquote>");
                            html.push_str(&Self::render_blocks(children));
                            html.push_str("</blockquote>");
                        }
                        ContainerBlock::List { kind, items, .. } => {
                            let tag = match kind {
                                ListKind::Bullet { .. } => "ul",
                                ListKind::Ordered { .. } => "ol",
                            };
                            html.push_str(&format!("<{}>", tag));
                            html.push_str(&Self::render_blocks(items));
                            html.push_str(&format!("</{}>", tag));
                        }
                        ContainerBlock::ListItem { contents, .. } => {
                            html.push_str("<li>");
                            html.push_str(&Self::render_blocks(contents));
                            html.push_str("</li>");
                        }
                    }
                }
                Block::Leaf(leaf) => {
                    match leaf {
                        LeafBlock::Paragraph(inlines, _) => {
                            html.push_str("<p>");
                            html.push_str(&Self::render_inlines(inlines));
                            html.push_str("</p>");
                        }
                        LeafBlock::Heading { level, content, .. } => {
                            html.push_str(&format!("<h{}>", level));
                            html.push_str(&Self::render_inlines(content));
                            html.push_str(&format!("</h{}>", level));
                        }
                        LeafBlock::IndentedCodeBlock { content, .. } => {
                            html.push_str("<pre><code>");
                            html.push_str(&html_escape::encode_text(content));
                            html.push_str("</code></pre>");
                        }
                        LeafBlock::FencedCodeBlock { content, .. } => {
                            html.push_str("<pre><code>");
                            html.push_str(&html_escape::encode_text(content));
                            html.push_str("</code></pre>");
                        }
                        LeafBlock::ThematicBreak { .. } => {
                            html.push_str("<hr />");
                        }
                        LeafBlock::HtmlBlock { content, .. } => {
                            html.push_str(content);
                        }
                        LeafBlock::Table { header, alignments: _, rows, caption, attributes: _ } => {
                            html.push_str("<table>");
                            // Render header
                            html.push_str("<thead><tr>");
                            for cell in &header.cells {
                                html.push_str("<th>");
                                html.push_str(&Self::render_inlines(&cell.content));
                                html.push_str("</th>");
                            }
                            html.push_str("</tr></thead>");
                            // Render body
                            html.push_str("<tbody>");
                            for row in rows {
                                html.push_str("<tr>");
                                for cell in &row.cells {
                                    html.push_str("<td>");
                                    html.push_str(&Self::render_inlines(&cell.content));
                                    html.push_str("</td>");
                                }
                                html.push_str("</tr>");
                            }
                            html.push_str("</tbody>");
                            // Optional caption
                            if let Some(caption) = caption {
                                html.push_str(&format!("<caption>{}</caption>", html_escape::encode_text(caption)));
                            }
                            html.push_str("</table>");
                        }
                        LeafBlock::Math(math) => {
                            html.push_str(&format!("<div class=\"math-block\">{}</div>", html_escape::encode_text(&math.content)));
                        }
                        LeafBlock::CustomTagBlock { name, data, content, attributes: _ } => {
                            let tag = html_escape::encode_text(name);
                            let data_str = data.as_ref().map(|d| html_escape::encode_text(d)).unwrap_or_default();
                            html.push_str(&format!("<{} data=\"{}\">", tag, data_str));
                            html.push_str(&Self::render_blocks(content));
                            html.push_str(&format!("</{}>", tag));
                        }
                        LeafBlock::FootnoteDefinition { identifier, label, children, association: _, attributes: _ } => {
                            let label_str = label.as_ref().map(|l| html_escape::encode_text(l)).unwrap_or_else(|| html_escape::encode_text(identifier));
                            html.push_str(&format!("<div class=\"footnote\" id=\"fn:{}\"><sup>{}</sup> {} </div>", html_escape::encode_double_quoted_attribute(identifier), label_str, Self::render_blocks(children)));
                        }
                        LeafBlock::AtxHeading { level, raw_content, .. } => {
                            html.push_str(&format!("<h{}>", level));
                            html.push_str(&html_escape::encode_text(raw_content));
                            html.push_str(&format!("</h{}>", level));
                        }
                        LeafBlock::SetextHeading { level, raw_content, .. } => {
                            html.push_str(&format!("<h{}>", level));
                            html.push_str(&html_escape::encode_text(raw_content));
                            html.push_str(&format!("</h{}>", level));
                        }
                        LeafBlock::LinkReferenceDefinition { .. } => {
                            // Ignore link reference definitions in HTML output
                        }
                        LeafBlock::BlankLine => {
                            // Ignore blank lines in HTML output
                        }
                    }
                }
            }
        }
        html
    }

    /// Render a full document AST to HTML
    pub fn render_document(block: &Block) -> String {
        match block {
            Block::Container(ContainerBlock::Document(children, _)) => {
                Self::render_blocks(children)
            }
            _ => Self::render_blocks(std::slice::from_ref(block)),
        }
    }
}