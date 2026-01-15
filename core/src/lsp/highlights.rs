// Syntax highlighting: map AST nodes to SourceView5 text tags

use crate::parser::{Document, Node, NodeKind, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct Highlight {
    pub span: Span,
    pub tag: HighlightTag,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HighlightTag {
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
    Emphasis,
    Strong,
    Strikethrough,
    Mark,
    Superscript,
    Subscript,
    Link,
    Image,
    CodeSpan,
    CodeBlock,
    InlineHtml,
    HardBreak,
    SoftBreak,
    ThematicBreak,
    Blockquote,
    Admonition,
    HtmlBlock,
    List,
    ListItem,
    TaskCheckboxChecked,
    TaskCheckboxUnchecked,
    Table,
    TableRow,
    TableRowHeader,
    TableCell,
    TableCellHeader,
    LinkReference,
    DefinitionList,
    DefinitionTerm,
    DefinitionDescription,
}

// Generate highlights from AST by walking all nodes
pub fn compute_highlights(document: &Document) -> Vec<Highlight> {
    log::debug!(
        "Computing syntax highlights for {} nodes",
        document.children.len()
    );

    let mut highlights = Vec::new();

    for node in &document.children {
        collect_highlights(node, &mut highlights);
    }

    // Make the output deterministic and robust for UI application.
    //
    // Why:
    // - Some UI layers apply tags in the order provided; stable ordering prevents
    //   flicker when different traversal orders happen (e.g., future refactors).
    // - Invalid/empty spans can cause confusing no-op or out-of-bounds behaviour
    //   during TextIter conversion.
    // - Dedup prevents redundant tag applications for identical ranges.
    highlights.retain(|h| is_non_empty_span(&h.span));
    highlights.sort_by(|a, b| {
        // Primary: start position (ascending)
        let start_cmp =
            (a.span.start.line, a.span.start.column).cmp(&(b.span.start.line, b.span.start.column));
        if start_cmp != std::cmp::Ordering::Equal {
            return start_cmp;
        }

        // Secondary: end position (descending) so outer spans come before inner
        // spans when they share the same start (useful for nested styling).
        let end_cmp =
            (b.span.end.line, b.span.end.column).cmp(&(a.span.end.line, a.span.end.column));
        if end_cmp != std::cmp::Ordering::Equal {
            return end_cmp;
        }

        // Tertiary: stable tag ordering
        tag_rank(&a.tag).cmp(&tag_rank(&b.tag))
    });
    highlights.dedup_by(|a, b| a.tag == b.tag && a.span == b.span);

    log::info!("Generated {} highlights", highlights.len());
    highlights
}

/// Return true if span describes a non-empty forward range.
///
/// `Span` is defined as [start, end) (end exclusive). In practice we only need
/// to guarantee that:
/// - start <= end in document order, and
/// - start != end
fn is_non_empty_span(span: &Span) -> bool {
    let start = (span.start.line, span.start.column);
    let end = (span.end.line, span.end.column);
    start < end
}

/// Provide a stable ordering for highlight tags.
///
/// This is used for deterministic sorting only; it has no semantic meaning
/// beyond ensuring a total ordering without allocations.
fn tag_rank(tag: &HighlightTag) -> u8 {
    match tag {
        HighlightTag::Heading1 => 1,
        HighlightTag::Heading2 => 2,
        HighlightTag::Heading3 => 3,
        HighlightTag::Heading4 => 4,
        HighlightTag::Heading5 => 5,
        HighlightTag::Heading6 => 6,
        HighlightTag::Emphasis => 10,
        HighlightTag::Strong => 11,
        HighlightTag::Strikethrough => 12,
        HighlightTag::Mark => 13,
        HighlightTag::Superscript => 14,
        HighlightTag::Subscript => 15,
        HighlightTag::Link => 16,
        HighlightTag::Image => 17,
        HighlightTag::CodeSpan => 20,
        HighlightTag::CodeBlock => 21,
        HighlightTag::InlineHtml => 30,
        HighlightTag::HardBreak => 40,
        HighlightTag::SoftBreak => 41,
        HighlightTag::ThematicBreak => 42,
        HighlightTag::Blockquote => 50,
        HighlightTag::Admonition => 51,
        HighlightTag::HtmlBlock => 52,
        HighlightTag::List => 60,
        HighlightTag::ListItem => 61,
        HighlightTag::TaskCheckboxUnchecked => 62,
        HighlightTag::TaskCheckboxChecked => 63,
        HighlightTag::Table => 70,
        HighlightTag::TableRowHeader => 71,
        HighlightTag::TableRow => 72,
        HighlightTag::TableCellHeader => 73,
        HighlightTag::TableCell => 74,
        HighlightTag::LinkReference => 80,
        HighlightTag::DefinitionList => 90,
        HighlightTag::DefinitionTerm => 91,
        HighlightTag::DefinitionDescription => 92,
    }
}

// Recursively collect highlights from a node and its children
fn collect_highlights(node: &Node, highlights: &mut Vec<Highlight>) {
    // Add highlight for this node if it has a span and is highlightable
    if let Some(span) = &node.span {
        match &node.kind {
            NodeKind::Heading { level, .. } => {
                let tag = match level {
                    1 => HighlightTag::Heading1,
                    2 => HighlightTag::Heading2,
                    3 => HighlightTag::Heading3,
                    4 => HighlightTag::Heading4,
                    5 => HighlightTag::Heading5,
                    6 => HighlightTag::Heading6,
                    _ => HighlightTag::Heading1, // Fallback for invalid levels
                };

                // For headings, highlight the entire line including the # markers
                // Expand the span to start from column 1 (beginning of line)
                // Use the parser `Span` helper to compute the absolute line start
                // offset instead of doing manual arithmetic which is fragile.
                let full_line_span = Span::new(
                    Position::new(span.start.line, 1, span.start_line_offset()),
                    span.end,
                );

                highlights.push(Highlight {
                    span: full_line_span,
                    tag,
                });
            }
            NodeKind::Emphasis => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::Emphasis,
                });
            }
            NodeKind::Strong => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::Strong,
                });
            }
            NodeKind::StrongEmphasis => {
                // Treat combined strong+emphasis as strong for coloring purposes.
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::Strong,
                });
            }
            NodeKind::Strikethrough => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::Strikethrough,
                });
            }
            NodeKind::Mark => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::Mark,
                });
            }
            NodeKind::Superscript => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::Superscript,
                });
            }
            NodeKind::Subscript => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::Subscript,
                });
            }
            NodeKind::Link { .. } => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::Link,
                });
            }
            NodeKind::PlatformMention { .. } => {
                // Mentions behave like navigational links.
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::Link,
                });
            }
            NodeKind::Image { .. } => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::Image,
                });
            }
            NodeKind::CodeSpan(_) => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::CodeSpan,
                });
            }
            NodeKind::CodeBlock { .. } => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::CodeBlock,
                });
            }
            NodeKind::InlineHtml(_) => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::InlineHtml,
                });
            }
            NodeKind::ThematicBreak => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::ThematicBreak,
                });
            }
            NodeKind::HtmlBlock { .. } => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::HtmlBlock,
                });
            }
            NodeKind::Blockquote => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::Blockquote,
                });
            }
            NodeKind::Admonition { .. } => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::Admonition,
                });
            }
            NodeKind::List { .. } => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::List,
                });
            }
            NodeKind::ListItem => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::ListItem,
                });
            }
            NodeKind::TaskCheckbox { checked } | NodeKind::TaskCheckboxInline { checked } => {
                highlights.push(Highlight {
                    span: *span,
                    tag: if *checked {
                        HighlightTag::TaskCheckboxChecked
                    } else {
                        HighlightTag::TaskCheckboxUnchecked
                    },
                });
            }
            NodeKind::Table { .. } => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::Table,
                });
            }
            NodeKind::TableRow { header } => {
                highlights.push(Highlight {
                    span: *span,
                    tag: if *header {
                        HighlightTag::TableRowHeader
                    } else {
                        HighlightTag::TableRow
                    },
                });
            }
            NodeKind::TableCell { header, .. } => {
                highlights.push(Highlight {
                    span: *span,
                    tag: if *header {
                        HighlightTag::TableCellHeader
                    } else {
                        HighlightTag::TableCell
                    },
                });
            }
            NodeKind::LinkReference { .. } => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::LinkReference,
                });
            }
            NodeKind::FootnoteReference { .. } => {
                // Footnote references behave like navigational links.
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::Link,
                });
            }
            NodeKind::FootnoteDefinition { .. } => {
                // Definition blocks are rendered out-of-band; keep them unstyled.
            }
            NodeKind::DefinitionList => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::DefinitionList,
                });
            }
            NodeKind::DefinitionTerm => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::DefinitionTerm,
                });
            }
            NodeKind::DefinitionDescription => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::DefinitionDescription,
                });
            }
            // SKIP only structural nodes without visual representation
            NodeKind::Paragraph | NodeKind::Text(_) => {
                // These are pure containers, no visual styling needed
            }
            // SKIP line breaks - they're invisible whitespace
            NodeKind::HardBreak | NodeKind::SoftBreak => {
                // Line breaks are formatting, not content
                // Don't highlight them
            }
        }
    }

    // Recursively process children
    for child in &node.children {
        collect_highlights(child, highlights);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Position;

    #[test]
    fn test_is_non_empty_span_basic() {
        let s = Span {
            start: Position {
                line: 1,
                column: 1,
                offset: 0,
            },
            end: Position {
                line: 1,
                column: 2,
                offset: 1,
            },
        };
        assert!(is_non_empty_span(&s));

        let empty = Span {
            start: Position {
                line: 1,
                column: 1,
                offset: 0,
            },
            end: Position {
                line: 1,
                column: 1,
                offset: 0,
            },
        };
        assert!(!is_non_empty_span(&empty));
    }

    #[test]
    fn smoke_test_heading_highlights() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Heading {
                        level: 1,
                        text: "Title".to_string(),
                        id: None,
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
                },
                Node {
                    kind: NodeKind::Heading {
                        level: 2,
                        text: "Subtitle".to_string(),
                        id: None,
                    },
                    span: Some(Span {
                        start: Position {
                            line: 3,
                            column: 1,
                            offset: 9,
                        },
                        end: Position {
                            line: 3,
                            column: 12,
                            offset: 20,
                        },
                    }),
                    children: vec![],
                },
            ],
            ..Default::default()
        };

        let highlights = compute_highlights(&doc);

        assert_eq!(highlights.len(), 2);
        assert_eq!(highlights[0].tag, HighlightTag::Heading1);
        assert_eq!(highlights[1].tag, HighlightTag::Heading2);
    }

    #[test]
    fn test_heading_full_line_offset_using_helper() {
        // Simulate a heading whose span starts at column 5 on line 2 with
        // an absolute offset of 10. The start_line_offset should compute
        // the absolute offset of the beginning of line 2.
        let start = Position {
            line: 2,
            column: 5,
            offset: 10,
        };
        let end = Position {
            line: 2,
            column: 12,
            offset: 17,
        };
        let node_span = Span { start, end };

        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Heading {
                    level: 1,
                    text: "Title".to_string(),
                    id: None,
                },
                span: Some(node_span),
                children: vec![],
            }],
            ..Default::default()
        };

        let highlights = compute_highlights(&doc);
        assert_eq!(highlights.len(), 1);
        // The highlight span start offset should equal the span's computed
        // line start offset (start.offset - (start.column - 1)).
        let hl_span = &highlights[0].span;
        assert_eq!(hl_span.start.offset, node_span.start_line_offset());
        assert_eq!(hl_span.start.column, 1);
        assert_eq!(hl_span.start.line, node_span.start.line);
    }

    #[test]
    fn smoke_test_inline_highlights() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
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
                children: vec![
                    Node {
                        kind: NodeKind::Text("Text with ".to_string()),
                        span: None,
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::Emphasis,
                        span: Some(Span {
                            start: Position {
                                line: 1,
                                column: 11,
                                offset: 10,
                            },
                            end: Position {
                                line: 1,
                                column: 18,
                                offset: 17,
                            },
                        }),
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::Text(" and ".to_string()),
                        span: None,
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::Strong,
                        span: Some(Span {
                            start: Position {
                                line: 1,
                                column: 24,
                                offset: 23,
                            },
                            end: Position {
                                line: 1,
                                column: 30,
                                offset: 29,
                            },
                        }),
                        children: vec![],
                    },
                ],
            }],
            ..Default::default()
        };

        let highlights = compute_highlights(&doc);

        assert_eq!(highlights.len(), 2);
        assert_eq!(highlights[0].tag, HighlightTag::Emphasis);
        assert_eq!(highlights[1].tag, HighlightTag::Strong);
    }

    #[test]
    fn test_inline_highlights_with_utf8_and_emoji() {
        // Construct document manually with inline nodes that contain multi-byte
        // UTF-8 characters and emoji in their spans. We don't rely on the
        // parser here; this ensures highlight mapping treats those spans the same.
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![
                    Node {
                        kind: NodeKind::Emphasis,
                        span: Some(Span {
                            // Simulate emphasis covering "Tëst" (multi-byte)
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
                        kind: NodeKind::Strong,
                        span: Some(Span {
                            // Simulate strong covering an emoji "😊"
                            start: Position {
                                line: 2,
                                column: 1,
                                offset: 6,
                            },
                            end: Position {
                                line: 2,
                                column: 5,
                                offset: 10,
                            },
                        }),
                        children: vec![],
                    },
                ],
            }],
            ..Default::default()
        };

        let highlights = compute_highlights(&doc);
        // Should include both Emphasis and Strong tags
        assert!(highlights.iter().any(|h| h.tag == HighlightTag::Emphasis));
        assert!(highlights.iter().any(|h| h.tag == HighlightTag::Strong));
    }

    #[test]
    fn smoke_test_code_highlights() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::CodeBlock {
                        language: Some("rust".to_string()),
                        code: "fn main() {}".to_string(),
                    },
                    span: Some(Span {
                        start: Position {
                            line: 1,
                            column: 1,
                            offset: 0,
                        },
                        end: Position {
                            line: 3,
                            column: 4,
                            offset: 25,
                        },
                    }),
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Paragraph,
                    span: None,
                    children: vec![Node {
                        kind: NodeKind::CodeSpan("code()".to_string()),
                        span: Some(Span {
                            start: Position {
                                line: 5,
                                column: 5,
                                offset: 30,
                            },
                            end: Position {
                                line: 5,
                                column: 13,
                                offset: 38,
                            },
                        }),
                        children: vec![],
                    }],
                },
            ],
            ..Default::default()
        };

        let highlights = compute_highlights(&doc);

        assert_eq!(highlights.len(), 2);
        assert_eq!(highlights[0].tag, HighlightTag::CodeBlock);
        assert_eq!(highlights[1].tag, HighlightTag::CodeSpan);
    }

    #[test]
    fn smoke_test_link_highlights() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![Node {
                    kind: NodeKind::Link {
                        url: "https://example.com".to_string(),
                        title: None,
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

        let highlights = compute_highlights(&doc);

        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].tag, HighlightTag::Link);
    }

    #[test]
    fn smoke_test_image_highlights() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![Node {
                    kind: NodeKind::Image {
                        url: "image.png".to_string(),
                        alt: "Alt text".to_string(),
                    },
                    span: Some(Span {
                        start: Position {
                            line: 1,
                            column: 1,
                            offset: 0,
                        },
                        end: Position {
                            line: 1,
                            column: 25,
                            offset: 24,
                        },
                    }),
                    children: vec![],
                }],
            }],
            ..Default::default()
        };

        let highlights = compute_highlights(&doc);

        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].tag, HighlightTag::Image);
    }

    #[test]
    fn smoke_test_inline_html_highlights() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![Node {
                    kind: NodeKind::InlineHtml("<span>text</span>".to_string()),
                    span: Some(Span {
                        start: Position {
                            line: 1,
                            column: 1,
                            offset: 0,
                        },
                        end: Position {
                            line: 1,
                            column: 18,
                            offset: 17,
                        },
                    }),
                    children: vec![],
                }],
            }],
            ..Default::default()
        };

        let highlights = compute_highlights(&doc);

        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].tag, HighlightTag::InlineHtml);
    }

    #[test]
    fn smoke_test_hard_break_highlights() {
        // Hard breaks are now skipped (they're invisible whitespace)
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![
                    Node {
                        kind: NodeKind::Text("Line one".to_string()),
                        span: None,
                        children: vec![],
                    },
                    Node {
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
                    },
                    Node {
                        kind: NodeKind::Text("Line two".to_string()),
                        span: None,
                        children: vec![],
                    },
                ],
            }],
            ..Default::default()
        };

        let highlights = compute_highlights(&doc);

        // Hard breaks are no longer highlighted (invisible whitespace)
        assert_eq!(highlights.len(), 0);
    }

    #[test]
    fn smoke_test_soft_break_highlights() {
        // Soft breaks are now skipped (they're invisible whitespace)
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![
                    Node {
                        kind: NodeKind::Text("Line one".to_string()),
                        span: None,
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::SoftBreak,
                        span: Some(Span {
                            start: Position {
                                line: 1,
                                column: 9,
                                offset: 8,
                            },
                            end: Position {
                                line: 2,
                                column: 1,
                                offset: 9,
                            },
                        }),
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::Text("Line two".to_string()),
                        span: None,
                        children: vec![],
                    },
                ],
            }],
            ..Default::default()
        };

        let highlights = compute_highlights(&doc);

        // Soft breaks are no longer highlighted (invisible whitespace)
        assert_eq!(highlights.len(), 0);
    }

    #[test]
    fn smoke_test_thematic_break_highlights() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::ThematicBreak,
                span: Some(Span {
                    start: Position {
                        line: 1,
                        column: 1,
                        offset: 0,
                    },
                    end: Position {
                        line: 1,
                        column: 4,
                        offset: 3,
                    },
                }),
                children: vec![],
            }],
            ..Default::default()
        };

        let highlights = compute_highlights(&doc);

        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].tag, HighlightTag::ThematicBreak);
    }

    #[test]
    fn smoke_test_blockquote_highlights() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Blockquote,
                span: Some(Span {
                    start: Position {
                        line: 1,
                        column: 1,
                        offset: 0,
                    },
                    end: Position {
                        line: 2,
                        column: 15,
                        offset: 30,
                    },
                }),
                children: vec![Node {
                    kind: NodeKind::Paragraph,
                    span: None,
                    children: vec![],
                }],
            }],
            ..Default::default()
        };

        let highlights = compute_highlights(&doc);

        // Blockquotes ARE highlighted so themes can style them
        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].tag, HighlightTag::Blockquote);
    }

    #[test]
    fn smoke_test_html_block_highlights() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::HtmlBlock {
                    html: "<div>content</div>".to_string(),
                },
                span: Some(Span {
                    start: Position {
                        line: 1,
                        column: 1,
                        offset: 0,
                    },
                    end: Position {
                        line: 1,
                        column: 19,
                        offset: 18,
                    },
                }),
                children: vec![],
            }],
            ..Default::default()
        };

        let highlights = compute_highlights(&doc);

        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].tag, HighlightTag::HtmlBlock);
    }

    #[test]
    fn smoke_test_list_highlights() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::List {
                    ordered: false,
                    start: None,
                    tight: true,
                },
                span: Some(Span {
                    start: Position {
                        line: 1,
                        column: 1,
                        offset: 0,
                    },
                    end: Position {
                        line: 3,
                        column: 10,
                        offset: 25,
                    },
                }),
                children: vec![
                    Node {
                        kind: NodeKind::ListItem,
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
                    },
                    Node {
                        kind: NodeKind::ListItem,
                        span: Some(Span {
                            start: Position {
                                line: 2,
                                column: 1,
                                offset: 8,
                            },
                            end: Position {
                                line: 2,
                                column: 8,
                                offset: 15,
                            },
                        }),
                        children: vec![],
                    },
                ],
            }],
            ..Default::default()
        };

        let highlights = compute_highlights(&doc);

        // Lists and list items ARE highlighted so themes can style them
        assert_eq!(highlights.len(), 3);
        assert_eq!(highlights[0].tag, HighlightTag::List);
        assert_eq!(highlights[1].tag, HighlightTag::ListItem);
        assert_eq!(highlights[2].tag, HighlightTag::ListItem);
    }

    #[test]
    fn test_compute_highlights_is_sorted_and_deduped() {
        // Intentionally add duplicates and out-of-order spans.
        let span_a = Span {
            start: Position {
                line: 2,
                column: 1,
                offset: 10,
            },
            end: Position {
                line: 2,
                column: 5,
                offset: 14,
            },
        };
        let span_b = Span {
            start: Position {
                line: 1,
                column: 1,
                offset: 0,
            },
            end: Position {
                line: 1,
                column: 3,
                offset: 2,
            },
        };

        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Strong,
                    span: Some(span_a),
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Strong,
                    span: Some(span_a),
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Emphasis,
                    span: Some(span_b),
                    children: vec![],
                },
            ],
            ..Default::default()
        };

        let highlights = compute_highlights(&doc);
        // Duplicate Strong should be removed.
        assert_eq!(highlights.len(), 2);
        // Sorted by start position: emphasis (line 1) comes before strong (line 2).
        assert_eq!(highlights[0].tag, HighlightTag::Emphasis);
        assert_eq!(highlights[1].tag, HighlightTag::Strong);
    }

    #[test]
    fn smoke_test_strike_mark_sup_sub_highlights() {
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
                                column: 12,
                                offset: 11,
                            },
                        }),
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::Superscript,
                        span: Some(Span {
                            start: Position {
                                line: 1,
                                column: 13,
                                offset: 12,
                            },
                            end: Position {
                                line: 1,
                                column: 18,
                                offset: 17,
                            },
                        }),
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::Subscript,
                        span: Some(Span {
                            start: Position {
                                line: 1,
                                column: 19,
                                offset: 18,
                            },
                            end: Position {
                                line: 1,
                                column: 24,
                                offset: 23,
                            },
                        }),
                        children: vec![],
                    },
                ],
            }],
            ..Default::default()
        };

        let highlights = compute_highlights(&doc);

        assert!(highlights
            .iter()
            .any(|h| h.tag == HighlightTag::Strikethrough));
        assert!(highlights.iter().any(|h| h.tag == HighlightTag::Mark));
        assert!(highlights
            .iter()
            .any(|h| h.tag == HighlightTag::Superscript));
        assert!(highlights.iter().any(|h| h.tag == HighlightTag::Subscript));
    }

    #[test]
    fn smoke_test_task_checkbox_highlights_checked_unchecked() {
        let checked_span = Span {
            start: Position {
                line: 1,
                column: 1,
                offset: 0,
            },
            end: Position {
                line: 1,
                column: 4,
                offset: 3,
            },
        };
        let unchecked_span = Span {
            start: Position {
                line: 2,
                column: 1,
                offset: 4,
            },
            end: Position {
                line: 2,
                column: 4,
                offset: 7,
            },
        };

        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::TaskCheckbox { checked: true },
                    span: Some(checked_span),
                    children: vec![],
                },
                Node {
                    kind: NodeKind::TaskCheckboxInline { checked: false },
                    span: Some(unchecked_span),
                    children: vec![],
                },
            ],
            ..Default::default()
        };

        let highlights = compute_highlights(&doc);
        assert!(highlights
            .iter()
            .any(|h| h.tag == HighlightTag::TaskCheckboxChecked));
        assert!(highlights
            .iter()
            .any(|h| h.tag == HighlightTag::TaskCheckboxUnchecked));
    }

    #[test]
    fn smoke_test_table_highlights_row_and_cell() {
        // Minimal table with a header row and one cell.
        let table_span = Span {
            start: Position {
                line: 1,
                column: 1,
                offset: 0,
            },
            end: Position {
                line: 2,
                column: 1,
                offset: 10,
            },
        };
        let row_span = Span {
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
        };
        let cell_span = Span {
            start: Position {
                line: 1,
                column: 3,
                offset: 2,
            },
            end: Position {
                line: 1,
                column: 4,
                offset: 3,
            },
        };

        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Table { alignments: vec![] },
                span: Some(table_span),
                children: vec![Node {
                    kind: NodeKind::TableRow { header: true },
                    span: Some(row_span),
                    children: vec![Node {
                        kind: NodeKind::TableCell {
                            header: true,
                            alignment: crate::parser::ast::TableAlignment::None,
                        },
                        span: Some(cell_span),
                        children: vec![],
                    }],
                }],
            }],
            ..Default::default()
        };

        let highlights = compute_highlights(&doc);

        assert!(highlights.iter().any(|h| h.tag == HighlightTag::Table));
        assert!(highlights
            .iter()
            .any(|h| h.tag == HighlightTag::TableRowHeader));
        assert!(highlights
            .iter()
            .any(|h| h.tag == HighlightTag::TableCellHeader));
    }

    #[test]
    fn smoke_test_definition_list_highlights() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::DefinitionList,
                span: Some(Span {
                    start: Position {
                        line: 1,
                        column: 1,
                        offset: 0,
                    },
                    end: Position {
                        line: 3,
                        column: 1,
                        offset: 20,
                    },
                }),
                children: vec![
                    Node {
                        kind: NodeKind::DefinitionTerm,
                        span: Some(Span {
                            start: Position {
                                line: 1,
                                column: 1,
                                offset: 0,
                            },
                            end: Position {
                                line: 1,
                                column: 5,
                                offset: 4,
                            },
                        }),
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::DefinitionDescription,
                        span: Some(Span {
                            start: Position {
                                line: 2,
                                column: 1,
                                offset: 5,
                            },
                            end: Position {
                                line: 3,
                                column: 1,
                                offset: 20,
                            },
                        }),
                        children: vec![],
                    },
                ],
            }],
            ..Default::default()
        };

        let highlights = compute_highlights(&doc);

        assert!(highlights
            .iter()
            .any(|h| h.tag == HighlightTag::DefinitionList));
        assert!(highlights
            .iter()
            .any(|h| h.tag == HighlightTag::DefinitionTerm));
        assert!(highlights
            .iter()
            .any(|h| h.tag == HighlightTag::DefinitionDescription));
    }
}
