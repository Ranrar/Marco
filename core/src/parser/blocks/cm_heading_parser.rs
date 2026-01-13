//! Heading parser - converts grammar output to AST nodes
//!
//! Handles conversion of both ATX headings (# Header) and Setext headings (underline style)
//! from grammar layer to parser AST representation.

use super::shared::{to_parser_span, to_parser_span_range, GrammarSpan};
use crate::parser::ast::{Node, NodeKind};

/// Parse an ATX heading (# Header) into an AST node.
///
/// # Arguments
/// * `level` - Heading level (1-6)
/// * `content` - The heading text content from grammar layer
///
/// # Returns
/// A Node with NodeKind::Heading
///
/// # Note
/// The span includes only the heading text content, not the # markers.
/// For full-line highlighting including markers, the LSP layer should use
/// the full line span.
///
/// # Example
/// ```ignore
/// let content = GrammarSpan::new("Hello World");
/// let node = parse_atx_heading(1, content);
/// assert!(matches!(node.kind, NodeKind::Heading { level: 1, .. }));
/// ```
pub fn parse_atx_heading(level: u8, content: GrammarSpan) -> Node {
    let span = to_parser_span(content);
    let text = content.fragment().to_string();

    Node {
        kind: NodeKind::Heading { level, text },
        span: Some(span),
        children: Vec::new(),
    }
}

/// Parse a Setext heading (underline style) into an AST node.
///
/// # Arguments
/// * `level` - Heading level (1 for === underline, 2 for --- underline)
/// * `content` - The heading text content from grammar layer
///
/// # Returns
/// A Node with NodeKind::Heading
///
/// # Example
/// ```ignore
/// let content = GrammarSpan::new("Hello\n===");
/// let node = parse_setext_heading(1, content);
/// assert!(matches!(node.kind, NodeKind::Heading { level: 1, .. }));
/// ```
pub fn parse_setext_heading(
    level: u8,
    content: GrammarSpan,
    full_start: GrammarSpan,
    full_end: GrammarSpan,
) -> Node {
    // NOTE:
    // - `content` is the heading text *without* the underline.
    // - `full_start..full_end` covers the entire setext construct including the underline,
    //   which is what we want for highlighting.
    let span = to_parser_span_range(full_start, full_end);
    let text = content.fragment().to_string();

    Node {
        kind: NodeKind::Heading { level, text },
        span: Some(span),
        children: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::blocks as grammar;

    #[test]
    fn smoke_test_parse_atx_heading_level_1() {
        let content = GrammarSpan::new("Hello World");
        let node = parse_atx_heading(1, content);

        if let NodeKind::Heading { level, text } = node.kind {
            assert_eq!(level, 1);
            assert_eq!(text, "Hello World");
        } else {
            panic!("Expected Heading node");
        }
    }

    #[test]
    fn smoke_test_parse_atx_heading_level_6() {
        let content = GrammarSpan::new("Small heading");
        let node = parse_atx_heading(6, content);

        if let NodeKind::Heading { level, text } = node.kind {
            assert_eq!(level, 6);
            assert_eq!(text, "Small heading");
        } else {
            panic!("Expected Heading node");
        }
    }

    #[test]
    fn smoke_test_parse_setext_heading_level_1() {
        let start = GrammarSpan::new("Main Title\n===\n");
        let (rest, (level, content)) = grammar::setext_heading(start).unwrap();
        let node = parse_setext_heading(level, content, start, rest);

        if let NodeKind::Heading { level, text } = node.kind {
            assert_eq!(level, 1);
            assert_eq!(text, "Main Title");
        } else {
            panic!("Expected Heading node");
        }
    }

    #[test]
    fn smoke_test_parse_setext_heading_level_2() {
        let start = GrammarSpan::new("Subtitle\n---\n");
        let (rest, (level, content)) = grammar::setext_heading(start).unwrap();
        let node = parse_setext_heading(level, content, start, rest);

        if let NodeKind::Heading { level, text } = node.kind {
            assert_eq!(level, 2);
            assert_eq!(text, "Subtitle");
        } else {
            panic!("Expected Heading node");
        }
    }

    #[test]
    fn smoke_test_setext_heading_span_includes_underline_line() {
        let start = GrammarSpan::new("Title\n===\nNext\n");
        let (rest, (level, content)) = grammar::setext_heading(start).unwrap();
        let node = parse_setext_heading(level, content, start, rest);

        let span = node.span.expect("setext heading should have span");
        // Should span across at least 2 lines (content + underline).
        assert_eq!(span.start.line, 1);
        assert!(
            span.end.line >= 2,
            "expected underline line to be included in span"
        );
    }

    #[test]
    fn smoke_test_heading_span_tracking() {
        let content = GrammarSpan::new("Test");
        let node = parse_atx_heading(3, content);

        assert!(node.span.is_some());
        let span = node.span.unwrap();
        assert_eq!(span.start.line, 1);
        assert_eq!(span.start.column, 1);
    }

    #[test]
    fn smoke_test_heading_no_children() {
        let content = GrammarSpan::new("Test");
        let node = parse_atx_heading(2, content);

        assert!(node.children.is_empty());
    }

    #[test]
    fn smoke_test_heading_empty_text() {
        let content = GrammarSpan::new("");
        let node = parse_atx_heading(1, content);

        if let NodeKind::Heading { text, .. } = node.kind {
            assert_eq!(text, "");
        } else {
            panic!("Expected Heading node");
        }
    }
}
