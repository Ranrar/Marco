//! Link parser - convert grammar links to AST nodes
//!
//! Parses inline links ([text](url "title")) and converts them to Link nodes.
//! Link nodes contain URL, optional title, and recursively parsed inline children.

use super::shared::{to_parser_span, GrammarSpan};
use crate::grammar::inlines as grammar;
use crate::parser::ast::{Node, NodeKind};
use nom::IResult;

/// Parse link and convert to AST node
///
/// Tries to parse an inline link from the input. If successful, returns a Node
/// with NodeKind::Link containing URL, optional title, and parsed inline children.
///
/// # Arguments
/// * `input` - The input text as a GrammarSpan
///
/// # Returns
/// * `Ok((remaining, node))` - Successfully parsed link node
/// * `Err(_)` - Not a link at this position
pub fn parse_link(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let (rest, (link_text, url, title)) = grammar::link(input)?;

    let span = to_parser_span(link_text);

    // Parse inline content within link text preserving position
    // Use the span directly instead of fragment() to maintain position context
    let children = match crate::parser::inlines::parse_inlines_from_span(link_text) {
        Ok(children) => children,
        Err(e) => {
            log::warn!("Failed to parse link text children: {}", e);
            vec![]
        }
    };

    let node = Node {
        kind: NodeKind::Link {
            url: url.fragment().to_string(),
            title: title.map(|s| s.fragment().to_string()),
        },
        span: Some(span),
        children,
    };

    Ok((rest, node))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_link_basic() {
        let input = GrammarSpan::new("[link text](http://example.com)");
        let result = parse_link(input);

        assert!(result.is_ok(), "Failed to parse link");
        let (rest, node) = result.unwrap();

        assert_eq!(rest.fragment(), &"");

        if let NodeKind::Link { url, title } = &node.kind {
            assert_eq!(url, "http://example.com");
            assert!(title.is_none());
        } else {
            panic!("Expected Link node");
        }

        assert!(!node.children.is_empty(), "Link should have text children");
    }

    #[test]
    fn smoke_test_parse_link_with_title() {
        let input = GrammarSpan::new(r#"[link](http://example.com "Title")"#);
        let result = parse_link(input);

        assert!(result.is_ok());
        let (_, node) = result.unwrap();

        if let NodeKind::Link { url, title } = &node.kind {
            assert_eq!(url, "http://example.com");
            assert_eq!(title.as_deref(), Some("Title"));
        } else {
            panic!("Expected Link node");
        }
    }

    #[test]
    fn smoke_test_parse_link_with_emphasis() {
        let input = GrammarSpan::new("[*emphasized* text](http://example.com)");
        let result = parse_link(input);

        assert!(result.is_ok());
        let (_, node) = result.unwrap();

        // Should have multiple children including emphasis
        assert!(node.children.len() >= 2);
    }

    #[test]
    fn smoke_test_parse_link_not_link() {
        let input = GrammarSpan::new("just text");
        let result = parse_link(input);

        assert!(result.is_err(), "Should not parse non-link as link");
    }

    #[test]
    fn smoke_test_parse_link_unclosed_bracket() {
        let input = GrammarSpan::new("[unclosed text");
        let result = parse_link(input);

        assert!(result.is_err(), "Should not parse unclosed bracket as link");
    }

    #[test]
    fn smoke_test_parse_link_position() {
        let input = GrammarSpan::new("[link](url) and text");
        let result = parse_link(input);

        assert!(result.is_ok());
        let (rest, node) = result.unwrap();

        assert_eq!(rest.fragment(), &" and text");
        assert!(node.span.is_some(), "Link should have position info");

        let span = node.span.unwrap();
        // Link text starts after '[' at position 1
        assert_eq!(span.start.offset, 1);
        assert!(span.end.offset > span.start.offset);
    }
}
