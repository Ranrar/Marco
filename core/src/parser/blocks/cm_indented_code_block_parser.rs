//! Indented code block parser - converts grammar output to AST nodes
//!
//! Handles conversion of indented code blocks (4 spaces or 1 tab indentation)
//! from grammar layer to parser AST representation.

use crate::parser::ast::{Node, NodeKind};
use super::shared::{to_parser_span, GrammarSpan};

/// Parse an indented code block into an AST node.
///
/// # Arguments
/// * `content` - The indented code block content from grammar layer
///
/// # Returns
/// A Node with NodeKind::CodeBlock (without language identifier)
///
/// # Processing
/// The function removes the leading 4 spaces or tab from each line,
/// as indented code blocks are defined by their indentation.
///
/// # Example
/// ```ignore
/// let content = GrammarSpan::new("    code line 1\n    code line 2");
/// let node = parse_indented_code_block(content);
/// assert!(matches!(node.kind, NodeKind::CodeBlock { language: None, .. }));
/// ```
pub fn parse_indented_code_block(content: GrammarSpan) -> Node {
    let span = to_parser_span(content);
    
    // Remove indentation from the code (4 spaces or 1 tab per line)
    let code = content.fragment().lines()
        .map(|line| {
            line.strip_prefix("    ")
                .or_else(|| line.strip_prefix('\t'))
                .unwrap_or(line)
        })
        .collect::<Vec<_>>()
        .join("\n");
    
    Node {
        kind: NodeKind::CodeBlock {
            language: None, // Indented code blocks don't have language
            code,
        },
        span: Some(span),
        children: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn smoke_test_parse_indented_code_block_spaces() {
        let content = GrammarSpan::new("    code line 1\n    code line 2");
        let node = parse_indented_code_block(content);
        
        if let NodeKind::CodeBlock { language, code } = node.kind {
            assert_eq!(language, None);
            assert_eq!(code, "code line 1\ncode line 2");
        } else {
            panic!("Expected CodeBlock node");
        }
    }
    
    #[test]
    fn smoke_test_parse_indented_code_block_tabs() {
        let content = GrammarSpan::new("\tcode with tab\n\tmore code");
        let node = parse_indented_code_block(content);
        
        if let NodeKind::CodeBlock { language, code } = node.kind {
            assert_eq!(language, None);
            assert_eq!(code, "code with tab\nmore code");
        } else {
            panic!("Expected CodeBlock node");
        }
    }
    
    #[test]
    fn smoke_test_indented_code_block_mixed() {
        let content = GrammarSpan::new("    line1\n\tline2\n    line3");
        let node = parse_indented_code_block(content);
        
        if let NodeKind::CodeBlock { code, .. } = node.kind {
            assert!(code.contains("line1"));
            assert!(code.contains("line2"));
            assert!(code.contains("line3"));
        } else {
            panic!("Expected CodeBlock node");
        }
    }
    
    #[test]
    fn smoke_test_indented_code_block_no_language() {
        let content = GrammarSpan::new("    test");
        let node = parse_indented_code_block(content);
        
        if let NodeKind::CodeBlock { language, .. } = node.kind {
            assert_eq!(language, None);
        } else {
            panic!("Expected CodeBlock node");
        }
    }
    
    #[test]
    fn smoke_test_indented_code_block_span() {
        let content = GrammarSpan::new("    test");
        let node = parse_indented_code_block(content);
        
        assert!(node.span.is_some());
        assert!(node.children.is_empty());
    }
    
    #[test]
    fn smoke_test_indented_code_block_empty_lines() {
        let content = GrammarSpan::new("    code\n\n    more");
        let node = parse_indented_code_block(content);
        
        if let NodeKind::CodeBlock { code, .. } = node.kind {
            assert!(code.contains('\n'));
        } else {
            panic!("Expected CodeBlock node");
        }
    }
}
