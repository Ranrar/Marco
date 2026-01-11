// Hover information: show link URLs, image alt text, etc.

use crate::parser::{Document, Node, NodeKind, Position, Span};

#[derive(Debug, Clone)]
pub struct HoverInfo {
    pub contents: String,
    pub range: Option<Span>,
}

// Provide hover information at position
pub fn get_hover_info(position: Position, document: &Document) -> Option<HoverInfo> {
    log::debug!("Computing hover info at {:?}", position);

    // Find the node at the given position
    for node in &document.children {
        if let Some(hover) = find_hover_at_position(node, position) {
            return Some(hover);
        }
    }

    None
}

// Recursively find node at position and return hover info
fn find_hover_at_position(node: &Node, position: Position) -> Option<HoverInfo> {
    // Check if position is within this node's span
    if let Some(span) = &node.span {
        if position_in_span(position, span) {
            // Generate hover info based on node kind
            let hover = match &node.kind {
                NodeKind::Link { url, title } => {
                    let mut contents = format!("**Link**\n\nURL: `{}`", url);
                    if let Some(t) = title {
                        if !t.is_empty() {
                            contents.push_str(&format!("\n\nTitle: \"{}\"", t));
                        }
                    }
                    Some(HoverInfo {
                        contents,
                        range: Some(*span),
                    })
                }
                NodeKind::Image { url, alt } => {
                    let mut contents = format!("**Image**\n\nURL: `{}`", url);
                    if !alt.is_empty() {
                        contents.push_str(&format!("\n\nAlt text: \"{}\"", alt));
                    }
                    Some(HoverInfo {
                        contents,
                        range: Some(*span),
                    })
                }
                NodeKind::CodeBlock { language, code } => {
                    let lang_info = language.as_ref()
                        .map(|l| format!(" ({})", l))
                        .unwrap_or_default();
                    let line_count = code.lines().count();
                    Some(HoverInfo {
                        contents: format!(
                            "**Code Block{}**\n\n{} line{}",
                            lang_info,
                            line_count,
                            if line_count == 1 { "" } else { "s" }
                        ),
                        range: Some(*span),
                    })
                }
                NodeKind::CodeSpan(code) => {
                    Some(HoverInfo {
                        contents: format!("**Code Span**\n\n`{}`", code),
                        range: Some(*span),
                    })
                }
                NodeKind::Heading { level, text } => {
                    Some(HoverInfo {
                        contents: format!("**Heading Level {}**\n\n{}", level, text),
                        range: Some(*span),
                    })
                }
                NodeKind::Emphasis => {
                    Some(HoverInfo {
                        contents: "**Emphasis** (italic)".to_string(),
                        range: Some(*span),
                    })
                }
                NodeKind::Strong => {
                    Some(HoverInfo {
                        contents: "**Strong** (bold)".to_string(),
                        range: Some(*span),
                    })
                }
                NodeKind::StrongEmphasis => {
                    Some(HoverInfo {
                        contents: "**Strong + Emphasis** (bold + italic)".to_string(),
                        range: Some(*span),
                    })
                }
                NodeKind::Strikethrough => {
                    Some(HoverInfo {
                        contents: "**Strikethrough** (deleted text)".to_string(),
                        range: Some(*span),
                    })
                }
                NodeKind::Mark => {
                    Some(HoverInfo {
                        contents: "**Mark** (highlight)".to_string(),
                        range: Some(*span),
                    })
                }
                NodeKind::Superscript => {
                    Some(HoverInfo {
                        contents: "**Superscript**".to_string(),
                        range: Some(*span),
                    })
                }
                NodeKind::Subscript => {
                    Some(HoverInfo {
                        contents: "**Subscript**".to_string(),
                        range: Some(*span),
                    })
                }
                NodeKind::InlineHtml(html) => {
                    let preview = if html.len() > 50 {
                        format!("{}...", &html[..50])
                    } else {
                        html.clone()
                    };
                    Some(HoverInfo {
                        contents: format!("**Inline HTML**\n\n```html\n{}\n```", preview),
                        range: Some(*span),
                    })
                }
                NodeKind::HardBreak => {
                    Some(HoverInfo {
                        contents: "**Hard Line Break**\n\nForces a line break in the output (renders as `<br />`)".to_string(),
                        range: Some(*span),
                    })
                }
                NodeKind::SoftBreak => {
                    Some(HoverInfo {
                        contents: "**Soft Line Break**\n\nRendered as a space or newline depending on context".to_string(),
                        range: Some(*span),
                    })
                }
                NodeKind::ThematicBreak => {
                    Some(HoverInfo {
                        contents: "**Thematic Break**\n\nHorizontal rule (renders as `<hr />`)".to_string(),
                        range: Some(*span),
                    })
                }
                NodeKind::Blockquote => {
                    let child_count = node.children.len();
                    Some(HoverInfo {
                        contents: format!(
                            "**Block Quote**\n\nContains {} block element{}",
                            child_count,
                            if child_count == 1 { "" } else { "s" }
                        ),
                        range: Some(*span),
                    })
                }
                _ => None,
            };

            if hover.is_some() {
                return hover;
            }
        }
    }

    // Search children recursively
    for child in &node.children {
        if let Some(hover) = find_hover_at_position(child, position) {
            return Some(hover);
        }
    }

    None
}

// Check if position is within span
fn position_in_span(position: Position, span: &Span) -> bool {
    let pos_offset = position.offset;
    pos_offset >= span.start.offset && pos_offset <= span.end.offset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_link_hover() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![Node {
                    kind: NodeKind::Link {
                        url: "https://example.com".to_string(),
                        title: Some("Example Site".to_string()),
                    },
                    span: Some(Span {
                        start: Position {
                            line: 1,
                            column: 1,
                            offset: 0,
                        },
                        end: Position {
                            line: 1,
                            column: 30,
                            offset: 29,
                        },
                    }),
                    children: vec![],
                }],
            }],
            ..Default::default()
        };

        let position = Position {
            line: 1,
            column: 5,
            offset: 4,
        };
        let hover = get_hover_info(position, &doc);

        assert!(hover.is_some());
        let hover = hover.unwrap();
        assert!(hover.contents.contains("Link"));
        assert!(hover.contents.contains("https://example.com"));
        assert!(hover.contents.contains("Example Site"));
    }

    #[test]
    fn smoke_test_image_hover() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![Node {
                    kind: NodeKind::Image {
                        url: "image.png".to_string(),
                        alt: "A beautiful image".to_string(),
                    },
                    span: Some(Span {
                        start: Position {
                            line: 1,
                            column: 1,
                            offset: 0,
                        },
                        end: Position {
                            line: 1,
                            column: 30,
                            offset: 29,
                        },
                    }),
                    children: vec![],
                }],
            }],
            ..Default::default()
        };

        let position = Position {
            line: 1,
            column: 5,
            offset: 4,
        };
        let hover = get_hover_info(position, &doc);

        assert!(hover.is_some());
        let hover = hover.unwrap();
        assert!(hover.contents.contains("Image"));
        assert!(hover.contents.contains("image.png"));
        assert!(hover.contents.contains("A beautiful image"));
    }

    #[test]
    fn smoke_test_code_block_hover() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::CodeBlock {
                    language: Some("rust".to_string()),
                    code: "fn main() {\n    println!(\"Hello\");\n}".to_string(),
                },
                span: Some(Span {
                    start: Position {
                        line: 1,
                        column: 1,
                        offset: 0,
                    },
                    end: Position {
                        line: 4,
                        column: 4,
                        offset: 50,
                    },
                }),
                children: vec![],
            }],
            ..Default::default()
        };

        let position = Position {
            line: 1,
            column: 5,
            offset: 4,
        };
        let hover = get_hover_info(position, &doc);

        assert!(hover.is_some());
        let hover = hover.unwrap();
        assert!(hover.contents.contains("Code Block"));
        assert!(hover.contents.contains("rust"));
        assert!(hover.contents.contains("3 lines"));
    }

    #[test]
    fn smoke_test_inline_html_hover() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![Node {
                    kind: NodeKind::InlineHtml("<span class=\"highlight\">text</span>".to_string()),
                    span: Some(Span {
                        start: Position {
                            line: 1,
                            column: 1,
                            offset: 0,
                        },
                        end: Position {
                            line: 1,
                            column: 37,
                            offset: 36,
                        },
                    }),
                    children: vec![],
                }],
            }],
            ..Default::default()
        };

        let position = Position {
            line: 1,
            column: 5,
            offset: 4,
        };
        let hover = get_hover_info(position, &doc);

        assert!(hover.is_some());
        let hover = hover.unwrap();
        assert!(hover.contents.contains("Inline HTML"));
        assert!(hover.contents.contains("span"));
    }

    #[test]
    fn smoke_test_hard_break_hover() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![Node {
                    kind: NodeKind::HardBreak,
                    span: Some(Span {
                        start: Position {
                            line: 1,
                            column: 9,
                            offset: 8,
                        },
                        end: Position {
                            line: 2,
                            column: 1,
                            offset: 11,
                        },
                    }),
                    children: vec![],
                }],
            }],
            ..Default::default()
        };

        let position = Position {
            line: 1,
            column: 10,
            offset: 9,
        };
        let hover = get_hover_info(position, &doc);

        assert!(hover.is_some());
        let hover = hover.unwrap();
        assert!(hover.contents.contains("Hard Line Break"));
        assert!(hover.contents.contains("<br />"));
    }

    #[test]
    fn smoke_test_no_hover() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Heading {
                    level: 1,
                    text: "Title".to_string(),
                },
                span: Some(Span {
                    start: Position {
                        line: 1,
                        column: 1,
                        offset: 0,
                    },
                    end: Position {
                        line: 1,
                        column: 8,
                        offset: 7,
                    },
                }),
                children: vec![],
            }],
            ..Default::default()
        };

        // Position outside any node
        let position = Position {
            line: 10,
            column: 1,
            offset: 100,
        };
        let hover = get_hover_info(position, &doc);

        assert!(hover.is_none());
    }

    #[test]
    fn smoke_test_inline_style_hovers() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![
                    Node {
                        kind: NodeKind::Strikethrough,
                        span: Some(Span {
                            start: Position {
                                line: 1,
                                column: 1,
                                offset: 0,
                            },
                            end: Position {
                                line: 1,
                                column: 6,
                                offset: 5,
                            },
                        }),
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::Mark,
                        span: Some(Span {
                            start: Position {
                                line: 1,
                                column: 7,
                                offset: 6,
                            },
                            end: Position {
                                line: 1,
                                column: 11,
                                offset: 10,
                            },
                        }),
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::Superscript,
                        span: Some(Span {
                            start: Position {
                                line: 1,
                                column: 12,
                                offset: 11,
                            },
                            end: Position {
                                line: 1,
                                column: 16,
                                offset: 15,
                            },
                        }),
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::Subscript,
                        span: Some(Span {
                            start: Position {
                                line: 1,
                                column: 17,
                                offset: 16,
                            },
                            end: Position {
                                line: 1,
                                column: 21,
                                offset: 20,
                            },
                        }),
                        children: vec![],
                    },
                ],
            }],
            ..Default::default()
        };

        let strike = get_hover_info(
            Position {
                line: 1,
                column: 2,
                offset: 1,
            },
            &doc,
        )
        .expect("expected strikethrough hover");
        assert!(strike.contents.contains("Strikethrough"));

        let mark = get_hover_info(
            Position {
                line: 1,
                column: 8,
                offset: 7,
            },
            &doc,
        )
        .expect("expected mark hover");
        assert!(mark.contents.contains("Mark"));

        let sup = get_hover_info(
            Position {
                line: 1,
                column: 13,
                offset: 12,
            },
            &doc,
        )
        .expect("expected superscript hover");
        assert!(sup.contents.contains("Superscript"));

        let sub = get_hover_info(
            Position {
                line: 1,
                column: 18,
                offset: 17,
            },
            &doc,
        )
        .expect("expected subscript hover");
        assert!(sub.contents.contains("Subscript"));
    }
}
