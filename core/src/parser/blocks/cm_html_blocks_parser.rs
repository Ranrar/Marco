//! HTML blocks parser - converts grammar output to AST nodes
//!
//! Handles conversion of all 7 types of HTML blocks from grammar layer to parser AST:
//! - Type 1: Special raw content tags (script, pre, style, textarea)
//! - Type 2: HTML comments
//! - Type 3: Processing instructions
//! - Type 4: Declarations
//! - Type 5: CDATA sections
//! - Type 6: Standard block tags (div, table, etc.)
//! - Type 7: Complete tags (cannot interrupt paragraphs)

use super::shared::{to_parser_span, GrammarSpan};
use crate::parser::ast::{Node, NodeKind};

/// Parse an HTML block into an AST node.
///
/// This function handles all 7 types of HTML blocks defined by CommonMark.
/// The type distinction is handled by the grammar layer; this parser simply
/// converts the matched HTML content into an AST node.
///
/// # Arguments
/// * `content` - The HTML block content from grammar layer
///
/// # Returns
/// A Node with NodeKind::HtmlBlock
///
/// # Example
/// ```ignore
/// let content = GrammarSpan::new("<div>\nContent\n</div>");
/// let node = parse_html_block(content);
/// assert!(matches!(node.kind, NodeKind::HtmlBlock { .. }));
/// ```
pub fn parse_html_block(content: GrammarSpan) -> Node {
    let span = to_parser_span(content);

    Node {
        kind: NodeKind::HtmlBlock {
            html: content.fragment().to_string(),
        },
        span: Some(span),
        children: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_html_block_div() {
        let content = GrammarSpan::new("<div>\nContent\n</div>");
        let node = parse_html_block(content);

        if let NodeKind::HtmlBlock { html } = node.kind {
            assert!(html.contains("<div>"));
            assert!(html.contains("</div>"));
        } else {
            panic!("Expected HtmlBlock node");
        }
    }

    #[test]
    fn smoke_test_parse_html_block_comment() {
        let content = GrammarSpan::new("<!-- This is a comment -->");
        let node = parse_html_block(content);

        if let NodeKind::HtmlBlock { html } = node.kind {
            assert!(html.contains("<!--"));
            assert!(html.contains("-->"));
        } else {
            panic!("Expected HtmlBlock node");
        }
    }

    #[test]
    fn smoke_test_parse_html_block_script() {
        let content = GrammarSpan::new("<script>\nvar x = 1;\n</script>");
        let node = parse_html_block(content);

        if let NodeKind::HtmlBlock { html } = node.kind {
            assert!(html.contains("<script>"));
            assert!(html.contains("var x = 1;"));
        } else {
            panic!("Expected HtmlBlock node");
        }
    }

    #[test]
    fn smoke_test_parse_html_block_processing_instruction() {
        let content = GrammarSpan::new("<?xml version=\"1.0\"?>");
        let node = parse_html_block(content);

        if let NodeKind::HtmlBlock { html } = node.kind {
            assert!(html.contains("<?xml"));
        } else {
            panic!("Expected HtmlBlock node");
        }
    }

    #[test]
    fn smoke_test_parse_html_block_cdata() {
        let content = GrammarSpan::new("<![CDATA[data here]]>");
        let node = parse_html_block(content);

        if let NodeKind::HtmlBlock { html } = node.kind {
            assert!(html.contains("CDATA"));
        } else {
            panic!("Expected HtmlBlock node");
        }
    }

    #[test]
    fn smoke_test_html_block_span() {
        let content = GrammarSpan::new("<p>Test</p>");
        let node = parse_html_block(content);

        assert!(node.span.is_some());
        assert!(node.children.is_empty());
    }

    #[test]
    fn smoke_test_html_block_multiline() {
        let content = GrammarSpan::new("<table>\n<tr><td>Cell</td></tr>\n</table>");
        let node = parse_html_block(content);

        if let NodeKind::HtmlBlock { html } = node.kind {
            assert!(html.contains('\n'));
            assert_eq!(html.lines().count(), 3);
        } else {
            panic!("Expected HtmlBlock node");
        }
    }

    #[test]
    fn smoke_test_html_block_declaration() {
        let content = GrammarSpan::new("<!DOCTYPE html>");
        let node = parse_html_block(content);

        if let NodeKind::HtmlBlock { html } = node.kind {
            assert!(html.contains("DOCTYPE"));
        } else {
            panic!("Expected HtmlBlock node");
        }
    }
}
