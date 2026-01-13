//! Paragraph parser - converts grammar output to AST nodes with inline parsing
//!
//! Handles conversion of paragraphs from grammar layer to parser AST,
//! including recursive inline element parsing for emphasis, links, etc.

use super::shared::{to_parser_span, GrammarSpan};
use crate::parser::ast::{Node, NodeKind};

use nom::Input;

/// Parse a paragraph into an AST node with inline elements.
///
/// # Arguments
/// * `content` - The paragraph content from grammar layer
///
/// # Returns
/// A Node with NodeKind::Paragraph containing parsed inline children
///
/// # Processing
/// The function:
/// 1. Converts the grammar span to parser span
/// 2. Recursively parses inline elements (emphasis, strong, links, etc.)
/// 3. Falls back to plain text on inline parsing errors
///
/// # Example
/// ```ignore
/// let content = GrammarSpan::new("This is **bold** text.");
/// let node = parse_paragraph(content);
/// assert!(matches!(node.kind, NodeKind::Paragraph));
/// assert!(!node.children.is_empty()); // Contains inline nodes
/// ```
pub fn parse_paragraph(content: GrammarSpan) -> Node {
    let span = to_parser_span(content);

    // Support task checkbox markers at the start of a paragraph *and* at the
    // start of any subsequent line inside the same paragraph.
    //
    // This matters when the author uses hard breaks (two spaces + newline) to
    // create a checklist-like block without list markers:
    //   [ ] first
    //   [ ] second
    //
    // Those lines are still a single paragraph in CommonMark; we still want to
    // render the checkbox SVG on each line.
    let mut inline_children: Vec<Node> = Vec::new();
    let mut remaining = content;

    while let Some((start, checked, consumed)) =
        find_next_task_checkbox_marker(remaining.fragment())
    {
        // Emit any content before the marker using the inline parser.
        if start > 0 {
            let (rest, prefix) = remaining.take_split(start);
            inline_children.extend(parse_inlines_or_fallback_text(prefix));
            remaining = rest;
        }

        // `remaining` now begins at the marker.
        let (after_marker, _marker_taken) = remaining.take_split(consumed);
        inline_children.push(Node {
            kind: NodeKind::TaskCheckboxInline { checked },
            span: Some(crate::parser::shared::to_parser_span_range(
                remaining,
                after_marker,
            )),
            children: Vec::new(),
        });
        remaining = after_marker;
    }

    // Emit any trailing content after the last marker.
    inline_children.extend(parse_inlines_or_fallback_text(remaining));

    Node {
        kind: NodeKind::Paragraph,
        span: Some(span),
        children: inline_children,
    }
}

fn parse_inlines_or_fallback_text(input: GrammarSpan) -> Vec<Node> {
    if input.fragment().is_empty() {
        return Vec::new();
    }

    match crate::parser::inlines::parse_inlines_from_span(input) {
        Ok(children) => children,
        Err(e) => {
            log::warn!("Failed to parse inline elements: {}", e);
            vec![Node {
                kind: NodeKind::Text(input.fragment().to_string()),
                span: Some(to_parser_span(input)),
                children: Vec::new(),
            }]
        }
    }
}

/// Find the next task checkbox marker that appears at a line start.
///
/// Returns (byte_offset_from_start, checked, consumed_bytes).
fn find_next_task_checkbox_marker(input: &str) -> Option<(usize, bool, usize)> {
    let mut line_start = 0usize;
    loop {
        if let Some((checked, consumed)) = parse_task_checkbox_prefix_len(&input[line_start..]) {
            return Some((line_start, checked, consumed));
        }

        let rel = input[line_start..].find('\n')?;
        line_start += rel + 1;
        if line_start >= input.len() {
            return None;
        }
    }
}

/// Detect a task checkbox marker at the start of a paragraph.
///
/// Recognizes:
/// - `[ ] ` (unchecked)
/// - `[x] ` / `[X] ` (checked)
///
/// Returns (checked, consumed_bytes).
fn parse_task_checkbox_prefix_len(input: &str) -> Option<(bool, usize)> {
    let mut i = 0usize;
    for _ in 0..3 {
        if input.as_bytes().get(i) == Some(&b' ') {
            i += 1;
        } else {
            break;
        }
    }

    let rest = &input[i..];

    let (checked, after_marker) = if let Some(after) = rest.strip_prefix("[ ]") {
        (false, after)
    } else if let Some(after) = rest
        .strip_prefix("[x]")
        .or_else(|| rest.strip_prefix("[X]"))
    {
        (true, after)
    } else {
        return None;
    };

    // Must be followed by at least one whitespace character.
    let mut chars = after_marker.chars();
    match chars.next() {
        Some(' ') | Some('\t') => {
            // Consumed: leading spaces + marker + exactly one whitespace.
            // Marker is 3 bytes: "[ ]" / "[x]" / "[X]".
            Some((checked, i + 3 + 1))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_paragraph_plain_text() {
        let content = GrammarSpan::new("This is a simple paragraph.");
        let node = parse_paragraph(content);

        assert!(matches!(node.kind, NodeKind::Paragraph));
        assert!(!node.children.is_empty());
    }

    #[test]
    fn smoke_test_paragraph_with_inline_elements() {
        let content = GrammarSpan::new("This has **bold** and *italic*.");
        let node = parse_paragraph(content);

        assert!(matches!(node.kind, NodeKind::Paragraph));
        assert!(!node.children.is_empty());
    }

    #[test]
    fn smoke_test_paragraph_empty() {
        let content = GrammarSpan::new("");
        let node = parse_paragraph(content);

        assert!(matches!(node.kind, NodeKind::Paragraph));
        // Empty paragraph may have no children or empty text node
    }

    #[test]
    fn smoke_test_paragraph_span() {
        let content = GrammarSpan::new("Test paragraph");
        let node = parse_paragraph(content);

        assert!(node.span.is_some());
        let span = node.span.unwrap();
        assert_eq!(span.start.line, 1);
        assert_eq!(span.start.column, 1);
    }

    #[test]
    fn smoke_test_paragraph_multiline() {
        let content = GrammarSpan::new("Line one\nLine two\nLine three");
        let node = parse_paragraph(content);

        assert!(matches!(node.kind, NodeKind::Paragraph));
        assert!(!node.children.is_empty());
    }

    #[test]
    fn smoke_test_paragraph_with_link() {
        let content = GrammarSpan::new("Check [this link](http://example.com) out.");
        let node = parse_paragraph(content);

        assert!(matches!(node.kind, NodeKind::Paragraph));
        assert!(!node.children.is_empty());
    }
}
