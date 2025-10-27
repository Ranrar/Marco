//! Inline HTML parser - convert grammar inline HTML to AST nodes
//!
//! Parses inline HTML tags (<tag>, <tag/>, etc.) and converts them to InlineHtml nodes.
//! Inline HTML is preserved as-is without further parsing.

use super::shared::{GrammarSpan, to_parser_span_range};
use crate::grammar::inlines as grammar;
use crate::parser::ast::{Node, NodeKind};
use nom::IResult;
use nom::Slice;

/// Parse inline HTML and convert to AST node
///
/// Tries to parse inline HTML from the input. If successful, returns a Node
/// with NodeKind::InlineHtml containing the raw HTML text.
///
/// # Arguments
/// * `input` - The input text as a GrammarSpan
///
/// # Returns
/// * `Ok((remaining, node))` - Successfully parsed inline HTML node
/// * `Err(_)` - Not inline HTML at this position
pub fn parse_inline_html(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let start = input;
    let (rest, _content) = grammar::inline_html(input)?;
    
    // Calculate the full HTML span (from start to rest)
    let start_offset = start.location_offset();
    let end_offset = rest.location_offset();
    let html_len = end_offset - start_offset;

    // Extract the full HTML including brackets using LocatedSpan::slice to keep
    // proper span semantics, then convert to string.
    let html = start.slice(..html_len).fragment().to_string();

    // Create span for the full HTML tag
    let span = to_parser_span_range(start, rest);
    
    let node = Node {
        kind: NodeKind::InlineHtml(html),
        span: Some(span),
        children: Vec::new(),
    };
    
    Ok((rest, node))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_inline_html_tag() {
        let input = GrammarSpan::new("<span>");
        let result = parse_inline_html(input);
        
        assert!(result.is_ok(), "Failed to parse inline HTML");
        let (rest, node) = result.unwrap();
        
        assert_eq!(rest.fragment(), &"");
        
        if let NodeKind::InlineHtml(html) = &node.kind {
            assert_eq!(html, "<span>");
        } else {
            panic!("Expected InlineHtml node");
        }
        
        assert!(node.children.is_empty(), "Inline HTML should not have children");
    }

    #[test]
    fn smoke_test_parse_inline_html_self_closing() {
        let input = GrammarSpan::new("<br/>");
        let result = parse_inline_html(input);
        
        assert!(result.is_ok());
        let (_, node) = result.unwrap();
        
        if let NodeKind::InlineHtml(html) = &node.kind {
            assert_eq!(html, "<br/>");
        }
    }

    #[test]
    fn smoke_test_parse_inline_html_with_attributes() {
        let input = GrammarSpan::new(r#"<a href="url">"#);
        let result = parse_inline_html(input);
        
        assert!(result.is_ok());
        let (_, node) = result.unwrap();
        
        if let NodeKind::InlineHtml(html) = &node.kind {
            assert!(html.contains("href"));
        }
    }

    #[test]
    fn smoke_test_parse_inline_html_not_html() {
        let input = GrammarSpan::new("just text");
        let result = parse_inline_html(input);
        
        assert!(result.is_err(), "Should not parse non-HTML as inline HTML");
    }

    #[test]
    fn smoke_test_parse_inline_html_position() {
        let input = GrammarSpan::new("<span> and text");
        let result = parse_inline_html(input);
        
        assert!(result.is_ok());
        let (rest, node) = result.unwrap();
        
        assert_eq!(rest.fragment(), &" and text");
        assert!(node.span.is_some(), "Inline HTML should have position info");
        
        let span = node.span.unwrap();
        assert_eq!(span.start.offset, 0);
        assert!(span.end.offset > span.start.offset);
    }
}
