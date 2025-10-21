//! Heading parser - converts grammar output to AST nodes
//!
//! Handles conversion of both ATX headings (# Header) and Setext headings (underline style)
//! from grammar layer to parser AST representation.

use crate::parser::ast::{Node, NodeKind};
use super::shared::{to_parser_span, GrammarSpan};

/// Parse an ATX heading (# Header) into an AST node.
///
/// # Arguments
/// * `level` - Heading level (1-6)
/// * `content` - The heading text content from grammar layer
///
/// # Returns
/// A Node with NodeKind::Heading
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
pub fn parse_setext_heading(level: u8, content: GrammarSpan) -> Node {
    let span = to_parser_span(content);
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
        let content = GrammarSpan::new("Main Title");
        let node = parse_setext_heading(1, content);
        
        if let NodeKind::Heading { level, text } = node.kind {
            assert_eq!(level, 1);
            assert_eq!(text, "Main Title");
        } else {
            panic!("Expected Heading node");
        }
    }
    
    #[test]
    fn smoke_test_parse_setext_heading_level_2() {
        let content = GrammarSpan::new("Subtitle");
        let node = parse_setext_heading(2, content);
        
        if let NodeKind::Heading { level, text } = node.kind {
            assert_eq!(level, 2);
            assert_eq!(text, "Subtitle");
        } else {
            panic!("Expected Heading node");
        }
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
