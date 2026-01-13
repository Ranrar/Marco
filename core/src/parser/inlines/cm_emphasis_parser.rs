//! Emphasis parser - convert grammar emphasis to AST nodes
//!
//! Parses emphasis (*text* or _text_) and converts them to Emphasis nodes.
//! Emphasis nodes contain children that are recursively parsed inline elements.

use super::shared::{to_parser_span_range, GrammarSpan};
use crate::grammar::inlines as grammar;
use crate::parser::ast::{Node, NodeKind};
use nom::IResult;

/// Parse emphasis and convert to AST node
///
/// Tries to parse emphasis from the input. If successful, returns a Node with
/// NodeKind::Emphasis containing recursively parsed inline children.
///
/// # Arguments
/// * `input` - The input text as a GrammarSpan
///
/// # Returns
/// * `Ok((remaining, node))` - Successfully parsed emphasis node
/// * `Err(_)` - Not emphasis at this position
pub fn parse_emphasis(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let start = input;
    let (rest, content) = grammar::emphasis(input)?;

    // Create span for the full emphasis (including delimiters)
    let span = to_parser_span_range(start, rest);

    // Recursively parse inline elements within emphasis text
    // Parse inline content within emphasis preserving position
    let children = match crate::parser::inlines::parse_inlines_from_span(content) {
        Ok(children) => children,
        Err(e) => {
            log::warn!("Failed to parse emphasis children: {}", e);
            vec![]
        }
    };

    let node = Node {
        kind: NodeKind::Emphasis,
        span: Some(span),
        children,
    };

    Ok((rest, node))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_emphasis_asterisk() {
        let input = GrammarSpan::new("*hello*");
        let result = parse_emphasis(input);

        assert!(result.is_ok(), "Failed to parse emphasis");
        let (rest, node) = result.unwrap();

        assert_eq!(rest.fragment(), &"");
        assert!(matches!(node.kind, NodeKind::Emphasis));
        assert_eq!(node.children.len(), 1); // Should have "hello" text child
    }

    #[test]
    fn smoke_test_parse_emphasis_underscore() {
        let input = GrammarSpan::new("_hello_");
        let result = parse_emphasis(input);

        assert!(result.is_ok());
        let (_, node) = result.unwrap();

        assert!(matches!(node.kind, NodeKind::Emphasis));
        assert!(!node.children.is_empty());
    }

    #[test]
    fn smoke_test_parse_emphasis_with_nested_code() {
        let input = GrammarSpan::new("*text with `code`*");
        let result = parse_emphasis(input);

        assert!(result.is_ok());
        let (_, node) = result.unwrap();

        assert!(matches!(node.kind, NodeKind::Emphasis));
        // Should have multiple children: text + code span + text
        assert!(node.children.len() >= 2);
    }

    #[test]
    fn smoke_test_parse_emphasis_not_emphasis() {
        let input = GrammarSpan::new("just text");
        let result = parse_emphasis(input);

        assert!(result.is_err(), "Should not parse non-emphasis as emphasis");
    }

    #[test]
    fn smoke_test_parse_emphasis_unclosed() {
        let input = GrammarSpan::new("*unclosed");
        let result = parse_emphasis(input);

        assert!(result.is_err(), "Should not parse unclosed emphasis");
    }

    #[test]
    fn smoke_test_parse_emphasis_empty() {
        let input = GrammarSpan::new("**");
        let result = parse_emphasis(input);

        // This might be parsed as strong, not emphasis
        // Or might fail - either is acceptable
        let _ = result;
    }

    #[test]
    fn test_parse_emphasis_utf8_and_emoji_positions() {
        // UTF-8 (ë is 2 bytes)
        let input = GrammarSpan::new("*Tëst*");
        let result = parse_emphasis(input);
        assert!(result.is_ok());
        let (rest, node) = result.unwrap();
        assert_eq!(*rest.fragment(), "");
        assert!(node.span.is_some());
        let span = node.span.unwrap();
        // Should start at line 1, column 1
        assert_eq!(span.start.line, 1);
        assert_eq!(span.start.column, 1);
        // End offset must be greater than start offset
        assert!(span.end.offset > span.start.offset);
        assert!(span.end.column > span.start.column);

        // Emoji (😊 is multi-byte)
        let input2 = GrammarSpan::new("*😊*");
        let result2 = parse_emphasis(input2);
        assert!(result2.is_ok());
        let (_, node2) = result2.unwrap();
        assert!(node2.span.is_some());
        let span2 = node2.span.unwrap();
        assert_eq!(span2.start.line, 1);
        assert_eq!(span2.start.column, 1);
        assert!(span2.end.offset > span2.start.offset);
        assert!(span2.end.column > span2.start.column);
    }

    #[test]
    fn smoke_test_parse_emphasis_position() {
        let input = GrammarSpan::new("*hello* world");
        let result = parse_emphasis(input);

        assert!(result.is_ok());
        let (rest, node) = result.unwrap();

        assert_eq!(rest.fragment(), &" world");
        assert!(node.span.is_some(), "Emphasis should have position info");

        let span = node.span.unwrap();
        assert_eq!(span.start.offset, 0);
        assert!(span.end.offset > span.start.offset);
    }
}
