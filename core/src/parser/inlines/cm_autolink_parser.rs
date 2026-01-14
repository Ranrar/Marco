//! Autolink parser - convert grammar autolinks to AST nodes
//!
//! Parses autolinks (<url> or <email>) and converts them to Link nodes.
//! Email autolinks get "mailto:" prefix, URL autolinks are used as-is.

use super::shared::{to_parser_span, to_parser_span_range, GrammarSpan};
use crate::grammar::inlines as grammar;
use crate::parser::ast::{Node, NodeKind};
use nom::IResult;

/// Parse autolink and convert to AST node
///
/// Tries to parse an autolink from the input. If successful, returns a Node
/// with NodeKind::Link. Email autolinks get "mailto:" prefix automatically.
///
/// # Arguments
/// * `input` - The input text as a GrammarSpan
///
/// # Returns
/// * `Ok((remaining, node))` - Successfully parsed link node
/// * `Err(_)` - Not an autolink at this position
pub fn parse_autolink(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let start = input;
    let (rest, (uri, is_email)) = grammar::autolink(input)?;

    // Create span for the full autolink (including < >)
    let span = to_parser_span_range(start, rest);

    // Span for the URI text (for the child text node)
    let uri_span = to_parser_span(uri);

    let node = if is_email {
        Node {
            kind: NodeKind::Link {
                url: format!("mailto:{}", uri.fragment()),
                title: None,
            },
            span: Some(span),
            children: vec![Node {
                kind: NodeKind::Text(uri.fragment().to_string()),
                span: Some(uri_span),
                children: Vec::new(),
            }],
        }
    } else {
        Node {
            kind: NodeKind::Link {
                url: uri.fragment().to_string(),
                title: None,
            },
            span: Some(span),
            children: vec![Node {
                kind: NodeKind::Text(uri.fragment().to_string()),
                span: Some(uri_span),
                children: Vec::new(),
            }],
        }
    };

    Ok((rest, node))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_autolink_url() {
        let input = GrammarSpan::new("<https://example.com>");
        let result = parse_autolink(input);

        assert!(result.is_ok(), "Failed to parse URL autolink");
        let (rest, node) = result.unwrap();

        assert_eq!(rest.fragment(), &"");

        if let NodeKind::Link { url, title } = &node.kind {
            assert_eq!(url, "https://example.com");
            assert!(title.is_none());
        } else {
            panic!("Expected Link node");
        }

        assert_eq!(
            node.children.len(),
            1,
            "Autolink should have one text child"
        );
    }

    #[test]
    fn smoke_test_parse_autolink_email() {
        let input = GrammarSpan::new("<user@example.com>");
        let result = parse_autolink(input);

        assert!(result.is_ok(), "Failed to parse email autolink");
        let (_, node) = result.unwrap();

        if let NodeKind::Link { url, title } = &node.kind {
            assert_eq!(url, "mailto:user@example.com");
            assert!(title.is_none());
        } else {
            panic!("Expected Link node with mailto: prefix");
        }
    }

    #[test]
    fn smoke_test_parse_autolink_not_autolink() {
        let input = GrammarSpan::new("just text");
        let result = parse_autolink(input);

        assert!(result.is_err(), "Should not parse non-autolink as autolink");
    }

    #[test]
    fn smoke_test_parse_autolink_unclosed() {
        let input = GrammarSpan::new("<https://example.com");
        let result = parse_autolink(input);

        assert!(result.is_err(), "Should not parse unclosed autolink");
    }

    #[test]
    fn smoke_test_parse_autolink_position() {
        let input = GrammarSpan::new("<https://example.com> and text");
        let result = parse_autolink(input);

        assert!(result.is_ok());
        let (rest, node) = result.unwrap();

        assert_eq!(rest.fragment(), &" and text");
        assert!(node.span.is_some(), "Autolink should have position info");

        let span = node.span.unwrap();
        assert_eq!(span.start.offset, 0);
        assert!(span.end.offset > span.start.offset);
    }
}
