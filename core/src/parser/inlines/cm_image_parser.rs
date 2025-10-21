//! Image parser - convert grammar images to AST nodes
//!
//! Parses inline images (![alt](url "title")) and converts them to Image nodes.
//! Image nodes contain URL and alt text but no children (unlike links).

use super::shared::{GrammarSpan, to_parser_span};
use crate::grammar::inlines as grammar;
use crate::parser::ast::{Node, NodeKind};
use nom::IResult;

/// Parse image and convert to AST node
///
/// Tries to parse an inline image from the input. If successful, returns a Node
/// with NodeKind::Image containing URL and alt text.
///
/// # Arguments
/// * `input` - The input text as a GrammarSpan
///
/// # Returns
/// * `Ok((remaining, node))` - Successfully parsed image node
/// * `Err(_)` - Not an image at this position
pub fn parse_image(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let (rest, (alt_text, url, _title)) = grammar::image(input)?;
    
    let span = to_parser_span(alt_text);
    
    let node = Node {
        kind: NodeKind::Image {
            url: url.fragment().to_string(),
            alt: alt_text.fragment().to_string(),
        },
        span: Some(span),
        children: Vec::new(), // Images don't have children
    };
    
    Ok((rest, node))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_image_basic() {
        let input = GrammarSpan::new("![alt text](http://example.com/image.png)");
        let result = parse_image(input);
        
        assert!(result.is_ok(), "Failed to parse image");
        let (rest, node) = result.unwrap();
        
        assert_eq!(rest.fragment(), &"");
        
        if let NodeKind::Image { url, alt } = &node.kind {
            assert_eq!(url, "http://example.com/image.png");
            assert_eq!(alt, "alt text");
        } else {
            panic!("Expected Image node");
        }
        
        assert!(node.children.is_empty(), "Image should not have children");
    }

    #[test]
    fn smoke_test_parse_image_with_title() {
        let input = GrammarSpan::new(r#"![alt](image.png "Title")"#);
        let result = parse_image(input);
        
        assert!(result.is_ok());
        let (_, node) = result.unwrap();
        
        if let NodeKind::Image { url, alt } = &node.kind {
            assert_eq!(url, "image.png");
            assert_eq!(alt, "alt");
        } else {
            panic!("Expected Image node");
        }
    }

    #[test]
    fn smoke_test_parse_image_empty_alt() {
        let input = GrammarSpan::new("![](image.png)");
        let result = parse_image(input);
        
        assert!(result.is_ok());
        let (_, node) = result.unwrap();
        
        if let NodeKind::Image { url, alt } = &node.kind {
            assert_eq!(url, "image.png");
            assert!(alt.is_empty());
        }
    }

    #[test]
    fn smoke_test_parse_image_not_image() {
        let input = GrammarSpan::new("just text");
        let result = parse_image(input);
        
        assert!(result.is_err(), "Should not parse non-image as image");
    }

    #[test]
    fn smoke_test_parse_image_missing_exclamation() {
        let input = GrammarSpan::new("[alt](image.png)");
        let result = parse_image(input);
        
        // This is a link, not an image
        assert!(result.is_err(), "Should not parse link as image");
    }

    #[test]
    fn smoke_test_parse_image_position() {
        let input = GrammarSpan::new("![alt](url) and text");
        let result = parse_image(input);
        
        assert!(result.is_ok());
        let (rest, node) = result.unwrap();
        
        assert_eq!(rest.fragment(), &" and text");
        assert!(node.span.is_some(), "Image should have position info");
        
        let span = node.span.unwrap();
        assert_eq!(span.start.offset, 0);
        assert!(span.end.offset > span.start.offset);
    }
}
