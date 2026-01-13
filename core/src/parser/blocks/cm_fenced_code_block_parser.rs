//! Fenced code block parser - converts grammar output to AST nodes
//!
//! Handles conversion of fenced code blocks (```, ~~~) with optional language
//! from grammar layer to parser AST representation.

use super::shared::{to_parser_span, GrammarSpan};
use crate::parser::ast::{Node, NodeKind};

/// Parse a fenced code block into an AST node.
///
/// # Arguments
/// * `language` - Optional language identifier (e.g., "rust", "python")
/// * `content` - The code block content from grammar layer
///
/// # Returns
/// A Node with NodeKind::CodeBlock
///
/// # Example
/// ```ignore
/// let content = GrammarSpan::new("fn main() {}");
/// let node = parse_fenced_code_block(Some("rust".to_string()), content);
/// assert!(matches!(node.kind, NodeKind::CodeBlock { .. }));
/// ```
pub fn parse_fenced_code_block(language: Option<String>, content: GrammarSpan) -> Node {
    let span = to_parser_span(content);
    let code = content.fragment().to_string();

    Node {
        kind: NodeKind::CodeBlock { language, code },
        span: Some(span),
        children: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_fenced_code_block_with_language() {
        let content = GrammarSpan::new("fn main() {\n    println!(\"Hello\");\n}");
        let node = parse_fenced_code_block(Some("rust".to_string()), content);

        if let NodeKind::CodeBlock { language, code } = node.kind {
            assert_eq!(language, Some("rust".to_string()));
            assert!(code.contains("fn main()"));
        } else {
            panic!("Expected CodeBlock node");
        }
    }

    #[test]
    fn smoke_test_parse_fenced_code_block_without_language() {
        let content = GrammarSpan::new("some code\nmore code");
        let node = parse_fenced_code_block(None, content);

        if let NodeKind::CodeBlock { language, code } = node.kind {
            assert_eq!(language, None);
            assert_eq!(code, "some code\nmore code");
        } else {
            panic!("Expected CodeBlock node");
        }
    }

    #[test]
    fn smoke_test_fenced_code_block_empty() {
        let content = GrammarSpan::new("");
        let node = parse_fenced_code_block(None, content);

        if let NodeKind::CodeBlock { code, .. } = node.kind {
            assert_eq!(code, "");
        } else {
            panic!("Expected CodeBlock node");
        }
    }

    #[test]
    fn smoke_test_fenced_code_block_span() {
        let content = GrammarSpan::new("test");
        let node = parse_fenced_code_block(Some("python".to_string()), content);

        assert!(node.span.is_some());
        assert!(node.children.is_empty());
    }

    #[test]
    fn smoke_test_fenced_code_block_multiline() {
        let content = GrammarSpan::new("line1\nline2\nline3");
        let node = parse_fenced_code_block(None, content);

        if let NodeKind::CodeBlock { code, .. } = node.kind {
            assert!(code.contains('\n'));
            assert_eq!(code.lines().count(), 3);
        } else {
            panic!("Expected CodeBlock node");
        }
    }
}
