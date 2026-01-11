//! Paragraph parser - converts grammar output to AST nodes with inline parsing
//!
//! Handles conversion of paragraphs from grammar layer to parser AST,
//! including recursive inline element parsing for emphasis, links, etc.

use super::shared::{to_parser_span, GrammarSpan};
use crate::parser::ast::{Node, NodeKind};

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

    // Parse inline elements within paragraph text, preserving position
    let inline_children = match crate::parser::inlines::parse_inlines_from_span(content) {
        Ok(children) => children,
        Err(e) => {
            log::warn!("Failed to parse inline elements: {}", e);
            // Fallback to plain text
            vec![Node {
                kind: NodeKind::Text(content.fragment().to_string()),
                span: Some(span),
                children: Vec::new(),
            }]
        }
    };

    Node {
        kind: NodeKind::Paragraph,
        span: Some(span),
        children: inline_children,
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
