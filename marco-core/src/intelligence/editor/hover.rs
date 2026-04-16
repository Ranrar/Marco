// Hover information: show link URLs, image alt text, etc.

use crate::logic::utf8::substring_by_chars;
use crate::parser::{Document, Node, NodeKind, Position, Span};

#[derive(Debug, Clone)]
pub struct HoverInfo {
    pub contents: String,
    pub range: Option<Span>,
}

pub fn get_hover_info(position: Position, document: &Document) -> Option<HoverInfo> {
    for node in &document.children {
        if let Some(hover) = find_hover_at_position(node, position) {
            return Some(hover);
        }
    }

    None
}

fn find_hover_at_position(node: &Node, position: Position) -> Option<HoverInfo> {
    // Deepest node wins: search children first.
    for child in &node.children {
        if let Some(hover) = find_hover_at_position(child, position) {
            return Some(hover);
        }
    }

    if let Some(span) = &node.span {
        if position_in_span(position, span) {
            return match &node.kind {
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
                    let lang_info = language
                        .as_ref()
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
                NodeKind::CodeSpan(code) => Some(HoverInfo {
                    contents: format!("**Code Span**\n\n`{}`", code),
                    range: Some(*span),
                }),
                NodeKind::Heading { level, text, .. } => Some(HoverInfo {
                    contents: format!("**Heading Level {}**\n\n{}", level, text),
                    range: Some(*span),
                }),
                NodeKind::Emphasis => Some(HoverInfo {
                    contents: "**Emphasis** (italic)".to_string(),
                    range: Some(*span),
                }),
                NodeKind::Strong => Some(HoverInfo {
                    contents: "**Strong** (bold)".to_string(),
                    range: Some(*span),
                }),
                NodeKind::StrongEmphasis => Some(HoverInfo {
                    contents: "**Strong + Emphasis** (bold + italic)".to_string(),
                    range: Some(*span),
                }),
                NodeKind::Strikethrough => Some(HoverInfo {
                    contents: "**Strikethrough** (deleted text)".to_string(),
                    range: Some(*span),
                }),
                NodeKind::Mark => Some(HoverInfo {
                    contents: "**Mark** (highlight)".to_string(),
                    range: Some(*span),
                }),
                NodeKind::Superscript => Some(HoverInfo {
                    contents: "**Superscript**".to_string(),
                    range: Some(*span),
                }),
                NodeKind::Subscript => Some(HoverInfo {
                    contents: "**Subscript**".to_string(),
                    range: Some(*span),
                }),
                NodeKind::InlineHtml(html) => {
                    let preview = if html.chars().count() > 50 {
                        format!("{}...", substring_by_chars(html, 0, 50))
                    } else {
                        html.clone()
                    };
                    Some(HoverInfo {
                        contents: format!("**Inline HTML**\n\n```html\n{}\n```", preview),
                        range: Some(*span),
                    })
                }
                NodeKind::HardBreak => Some(HoverInfo {
                    contents: "**Hard Line Break**\n\nForces a line break in the output (renders as `<br />`)".to_string(),
                    range: Some(*span),
                }),
                NodeKind::SoftBreak => Some(HoverInfo {
                    contents: "**Soft Line Break**\n\nRendered as a space or newline depending on context".to_string(),
                    range: Some(*span),
                }),
                NodeKind::ThematicBreak => Some(HoverInfo {
                    contents: "**Thematic Break**\n\nHorizontal rule (renders as `<hr />`)".to_string(),
                    range: Some(*span),
                }),
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
        }
    }

    None
}

/// Returns the span of the tightest (deepest) AST node covering `position`,
/// regardless of whether that node has meaningful hover content.
///
/// This is used to detect when a diagnostic's span is wider than the specific
/// node the cursor is actually on — in that case the diagnostic should be
/// suppressed rather than shown over unrelated plain text.
pub fn get_position_span(position: Position, document: &Document) -> Option<Span> {
    for node in &document.children {
        if let Some(span) = find_tightest_span_at(node, position) {
            return Some(span);
        }
    }
    None
}

fn find_tightest_span_at(node: &Node, position: Position) -> Option<Span> {
    // Deepest node (tightest span) wins: check children first.
    for child in &node.children {
        if let Some(span) = find_tightest_span_at(child, position) {
            return Some(span);
        }
    }
    if let Some(span) = &node.span {
        if position_in_span(position, span) {
            return Some(*span);
        }
    }
    None
}

fn position_in_span(position: Position, span: &Span) -> bool {
    let pos_offset = position.offset;
    // Span is [start, end) end-exclusive.
    pos_offset >= span.start.offset && pos_offset < span.end.offset
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    fn pos(line: usize, column: usize, offset: usize) -> Position {
        Position {
            line,
            column,
            offset,
        }
    }

    fn span(start_offset: usize, end_offset: usize) -> Span {
        Span {
            start: pos(1, start_offset + 1, start_offset),
            end: pos(1, end_offset + 1, end_offset),
        }
    }

    #[test]
    fn smoke_test_hover_span_start_inclusive_end_exclusive() {
        let link_span = span(5, 10);
        let heading_span = span(0, 20);

        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Heading {
                    level: 2,
                    text: "Parent heading".to_string(),
                    id: None,
                },
                span: Some(heading_span),
                children: vec![Node {
                    kind: NodeKind::Link {
                        url: "https://example.com".to_string(),
                        title: None,
                    },
                    span: Some(link_span),
                    children: vec![Node {
                        kind: NodeKind::Text("link".to_string()),
                        span: Some(link_span),
                        children: vec![],
                    }],
                }],
            }],
            ..Default::default()
        };

        // Child start offset is included.
        let at_start = get_hover_info(pos(1, 6, 5), &doc).expect("hover at child start");
        assert!(at_start.contents.contains("**Link**"));

        // Child end offset is excluded; parent hover should win here.
        let at_end = get_hover_info(pos(1, 11, 10), &doc).expect("hover at child end");
        assert!(at_end.contents.contains("**Heading Level 2**"));
        assert!(!at_end.contents.contains("**Link**"));
    }

    #[test]
    fn smoke_test_hover_deepest_node_wins_over_parent() {
        let strong_span = span(2, 15);
        let link_span = span(5, 12);

        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: Some(span(0, 20)),
                children: vec![Node {
                    kind: NodeKind::Strong,
                    span: Some(strong_span),
                    children: vec![Node {
                        kind: NodeKind::Link {
                            url: "https://deep.example".to_string(),
                            title: Some("deep".to_string()),
                        },
                        span: Some(link_span),
                        children: vec![Node {
                            kind: NodeKind::Text("deep".to_string()),
                            span: Some(link_span),
                            children: vec![],
                        }],
                    }],
                }],
            }],
            ..Default::default()
        };

        let hover = get_hover_info(pos(1, 7, 6), &doc).expect("hover inside nested nodes");
        assert!(hover.contents.contains("**Link**"));
        assert!(hover.contents.contains("https://deep.example"));
        assert!(!hover.contents.contains("**Strong**"));
    }

    #[test]
    fn smoke_test_hover_returns_none_at_top_level_end_boundary() {
        let heading_span = span(0, 4);
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Heading {
                    level: 1,
                    text: "Test".to_string(),
                    id: None,
                },
                span: Some(heading_span),
                children: vec![],
            }],
            ..Default::default()
        };

        // End boundary is exclusive, so hover should not trigger at offset=end.
        assert!(get_hover_info(pos(1, 5, 4), &doc).is_none());
    }

    fn offset_to_position(source: &str, offset: usize) -> Position {
        let mut line = 1usize;
        let mut line_start_offset = 0usize;

        for (idx, ch) in source.char_indices() {
            if idx >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                line_start_offset = idx + ch.len_utf8();
            }
        }

        Position {
            line,
            column: offset.saturating_sub(line_start_offset) + 1,
            offset,
        }
    }

    fn first_link_span_in_doc(document: &Document) -> Option<Span> {
        fn visit(node: &Node) -> Option<Span> {
            if let (NodeKind::Link { .. }, Some(span)) = (&node.kind, node.span) {
                return Some(span);
            }

            for child in &node.children {
                if let Some(span) = visit(child) {
                    return Some(span);
                }
            }

            None
        }

        for node in &document.children {
            if let Some(span) = visit(node) {
                return Some(span);
            }
        }

        None
    }

    #[test]
    fn smoke_test_parser_driven_hover_deepest_node_precedence() {
        let source = "**[deep](https://example.com)**";
        let doc = parse(source).expect("parse failed");

        let inside_link_offset = source.find("deep").expect("missing token") + 1;
        let position = offset_to_position(source, inside_link_offset);

        let hover = get_hover_info(position, &doc).expect("hover should exist");
        assert!(hover.contents.contains("**Link**"));
        assert!(hover.contents.contains("https://example.com"));
        assert!(!hover.contents.contains("**Strong**"));
    }

    #[test]
    fn smoke_test_parser_driven_hover_link_span_boundaries() {
        let source = "[hello](https://example.com) tail";
        let doc = parse(source).expect("parse failed");
        let link_span = first_link_span_in_doc(&doc).expect("link span not found");

        let at_start = get_hover_info(offset_to_position(source, link_span.start.offset), &doc)
            .expect("hover at link start");
        assert!(at_start.contents.contains("**Link**"));

        // End boundary is exclusive.
        let at_end = get_hover_info(offset_to_position(source, link_span.end.offset), &doc);
        assert!(at_end.is_none());
    }

    #[test]
    fn smoke_test_parser_driven_hover_utf8_link_text_offsets() {
        let source = "préfix [lïnk🎨](https://example.com) sufix";
        let doc = parse(source).expect("parse failed");

        let i_umlaut_offset = source.find("ï").expect("missing ï");
        let emoji_offset = source.find("🎨").expect("missing emoji");

        let hover_umlaut = get_hover_info(offset_to_position(source, i_umlaut_offset), &doc)
            .expect("hover should exist at multibyte Latin character");
        assert!(hover_umlaut.contents.contains("**Link**"));

        let hover_emoji = get_hover_info(offset_to_position(source, emoji_offset), &doc)
            .expect("hover should exist at emoji character");
        assert!(hover_emoji.contents.contains("**Link**"));
        assert!(hover_emoji.contents.contains("https://example.com"));
    }

    #[test]
    fn smoke_test_parser_driven_hover_utf8_multiline_boundaries() {
        let source = "αβγ\n[🎨x](https://example.com)\nend";
        let doc = parse(source).expect("parse failed");
        let link_span = first_link_span_in_doc(&doc).expect("link span not found");

        // Confirm hover resolves correctly on line 2 despite multibyte chars on line 1.
        let inside_offset = source.find("🎨").expect("missing emoji");
        let hover_inside = get_hover_info(offset_to_position(source, inside_offset), &doc)
            .expect("hover should exist inside utf8 multiline link");
        assert!(hover_inside.contents.contains("**Link**"));

        // End is still exclusive even with UTF-8 and line breaks involved.
        let at_end = get_hover_info(offset_to_position(source, link_span.end.offset), &doc);
        assert!(at_end.is_none());
    }

    #[test]
    fn smoke_test_hover_inline_html_preview_utf8_safe_truncation() {
        let html = format!("{}🎨{}", "a".repeat(49), "b".repeat(10));
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::InlineHtml(html),
                span: Some(span(0, 80)),
                children: vec![],
            }],
            ..Default::default()
        };

        let hover = get_hover_info(pos(1, 2, 1), &doc)
            .expect("hover should exist for inline html with utf8 preview");

        assert!(hover.contents.contains("**Inline HTML**"));
        assert!(hover.contents.contains("🎨"));
        assert!(hover.contents.contains("..."));
    }
}
