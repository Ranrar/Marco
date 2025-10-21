//! Thematic break parser - converts grammar output to AST nodes
//!
//! Handles conversion of thematic breaks (horizontal rules: ---, ***, ___)
//! from grammar layer to parser AST representation.

use crate::parser::ast::{Node, NodeKind};
use super::shared::{to_parser_span, GrammarSpan};

/// Parse a thematic break into an AST node.
///
/// # Arguments
/// * `content` - The matched thematic break content from grammar layer
///
/// # Returns
/// A Node with NodeKind::ThematicBreak
///
/// # Example
/// ```ignore
/// let span = GrammarSpan::new("---");
/// let node = parse_thematic_break(span);
/// assert!(matches!(node.kind, NodeKind::ThematicBreak));
/// ```
pub fn parse_thematic_break(content: GrammarSpan) -> Node {
    let span = to_parser_span(content);
    
    Node {
        kind: NodeKind::ThematicBreak,
        span: Some(span),
        children: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn smoke_test_parse_thematic_break() {
        let content = GrammarSpan::new("---");
        let node = parse_thematic_break(content);
        
        assert!(matches!(node.kind, NodeKind::ThematicBreak));
        assert!(node.span.is_some());
        assert!(node.children.is_empty());
    }
    
    #[test]
    fn smoke_test_thematic_break_asterisks() {
        let content = GrammarSpan::new("***");
        let node = parse_thematic_break(content);
        
        assert!(matches!(node.kind, NodeKind::ThematicBreak));
    }
    
    #[test]
    fn smoke_test_thematic_break_underscores() {
        let content = GrammarSpan::new("___");
        let node = parse_thematic_break(content);
        
        assert!(matches!(node.kind, NodeKind::ThematicBreak));
    }
    
    #[test]
    fn smoke_test_thematic_break_with_spaces() {
        let content = GrammarSpan::new("- - -");
        let node = parse_thematic_break(content);
        
        assert!(matches!(node.kind, NodeKind::ThematicBreak));
    }
    
    #[test]
    fn smoke_test_thematic_break_span() {
        let content = GrammarSpan::new("---");
        let node = parse_thematic_break(content);
        
        let span = node.span.expect("Span should be present");
        assert_eq!(span.start.line, 1);
        assert_eq!(span.start.column, 1);
    }
}
