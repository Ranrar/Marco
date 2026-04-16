//! Fenced code block parser - converts grammar output to AST nodes
//!
//! Handles conversion of fenced code blocks (```, ~~~) with optional language
//! from grammar layer to parser AST representation.

use super::shared::{to_parser_span, GrammarSpan};
use crate::parser::ast::{Node, NodeKind};

/// Parse a fenced code block into an AST node.
///
/// # Arguments
/// * `language` - Optional language identifier (e.g., "rust", "python", "mermaid")
/// * `content` - The code block content from grammar layer
///
/// # Returns
/// A Node with NodeKind::CodeBlock for regular code, or NodeKind::MermaidDiagram
/// when language is "mermaid"
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

    // Detect Mermaid diagrams (```mermaid ... ```)
    if let Some(ref lang) = language {
        if lang.eq_ignore_ascii_case("mermaid") {
            return Node {
                kind: NodeKind::MermaidDiagram { content: code },
                span: Some(span),
                children: Vec::new(),
            };
        }

        // Detect math blocks (```math ... ```)
        if lang.eq_ignore_ascii_case("math") {
            // Strip surrounding $$ delimiters if present
            let math_content = code.trim();
            let content = if math_content.starts_with("$$") && math_content.ends_with("$$") {
                // Remove $$ delimiters
                math_content[2..math_content.len() - 2].trim().to_string()
            } else {
                // Use content as-is (already math, no delimiters needed)
                math_content.to_string()
            };

            return Node {
                kind: NodeKind::DisplayMath { content },
                span: Some(span),
                children: Vec::new(),
            };
        }
    }

    // Regular code block
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

    #[test]
    fn smoke_test_mermaid_diagram_detection() {
        let content = GrammarSpan::new("graph TD\n    A --> B");
        let node = parse_fenced_code_block(Some("mermaid".to_string()), content);

        if let NodeKind::MermaidDiagram { content } = node.kind {
            assert!(content.contains("graph TD"));
            assert!(content.contains("A --> B"));
        } else {
            panic!("Expected MermaidDiagram node, got {:?}", node.kind);
        }
    }

    #[test]
    fn smoke_test_mermaid_case_insensitive() {
        let content = GrammarSpan::new("sequenceDiagram\n    Alice->>Bob: Hello");
        let node = parse_fenced_code_block(Some("MERMAID".to_string()), content);

        assert!(matches!(node.kind, NodeKind::MermaidDiagram { .. }));
    }

    #[test]
    fn smoke_test_mermaid_mixed_case() {
        let content = GrammarSpan::new("pie title Pets\n    \"Dogs\" : 386");
        let node = parse_fenced_code_block(Some("Mermaid".to_string()), content);

        assert!(matches!(node.kind, NodeKind::MermaidDiagram { .. }));
    }

    #[test]
    fn smoke_test_non_mermaid_remains_code_block() {
        let content = GrammarSpan::new("console.log('test');");
        let node = parse_fenced_code_block(Some("javascript".to_string()), content);

        assert!(matches!(node.kind, NodeKind::CodeBlock { .. }));
    }

    #[test]
    fn smoke_test_math_block_with_delimiters() {
        let content =
            GrammarSpan::new("$$\n\\frac{d}{dx}\\left( \\int_{0}^{x} f(u)\\,du\\right)=f(x)\n$$");
        let node = parse_fenced_code_block(Some("math".to_string()), content);

        if let NodeKind::DisplayMath { content } = node.kind {
            // Delimiters should be stripped
            assert!(!content.starts_with("$$"));
            assert!(!content.ends_with("$$"));
            assert!(content.contains("\\frac{d}{dx}"));
        } else {
            panic!("Expected DisplayMath node, got {:?}", node.kind);
        }
    }

    #[test]
    fn smoke_test_math_block_without_delimiters() {
        let content = GrammarSpan::new("E = mc^2");
        let node = parse_fenced_code_block(Some("math".to_string()), content);

        if let NodeKind::DisplayMath { content } = node.kind {
            assert_eq!(content, "E = mc^2");
        } else {
            panic!("Expected DisplayMath node, got {:?}", node.kind);
        }
    }

    #[test]
    fn smoke_test_math_case_insensitive() {
        let content = GrammarSpan::new("x^2 + y^2 = z^2");
        let node = parse_fenced_code_block(Some("MATH".to_string()), content);

        assert!(matches!(node.kind, NodeKind::DisplayMath { .. }));
    }

    #[test]
    fn smoke_test_math_multiline() {
        let content = GrammarSpan::new("$$\nx = 5\\\\\ny = 10\n$$");
        let node = parse_fenced_code_block(Some("math".to_string()), content);

        if let NodeKind::DisplayMath { content } = node.kind {
            assert!(content.contains("x = 5"));
            assert!(content.contains("y = 10"));
        } else {
            panic!("Expected DisplayMath node");
        }
    }
}
