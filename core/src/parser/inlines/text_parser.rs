//! Text parser - handle plain text fallback
//!
//! Parses plain text segments when no inline elements match. Handles special
//! cases like trailing spaces before newlines and consecutive backticks.

use super::shared::{GrammarSpan, to_parser_span};
use crate::parser::ast::{Node, NodeKind};
use nom::bytes::complete::take;
use nom::IResult;
use nom::Slice;

/// Parse plain text up to the next special character
///
/// Consumes text until a special inline character is found (*_`[<!\\n).
/// Handles special cases:
/// - Trailing spaces before newlines (potential hard line break)
/// - Consecutive backticks (consume all together)
///
/// # Arguments
/// * `input` - The input text as a GrammarSpan
///
/// # Returns
/// * `Ok((remaining, node))` - Successfully parsed text node
/// * `Err(_)` - No text to parse (input starts with special character)
pub fn parse_text(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let text_fragment = input.fragment();
    
    // Find the next special character
    let next_special = text_fragment
        .find(['*', '_', '`', '[', '<', '!', '\n', '\\'])
        .unwrap_or(text_fragment.len());
    
    if next_special == 0 {
        // No text - input starts with special character
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        )));
    }
    
    // Check if the upcoming character is a newline and the text ends with spaces
    // If so, don't consume trailing spaces (they might be part of a hard line break)
    let mut text_len = next_special;
    if text_fragment.chars().nth(next_special) == Some('\n') {
        // Check for trailing spaces
        let mut trailing_spaces = 0;
        for ch in text_fragment[..next_special].chars().rev() {
            if ch == ' ' {
                trailing_spaces += 1;
            } else {
                break;
            }
        }
        
        // If we have 2+ trailing spaces, don't consume them
        // (they might be part of a hard line break pattern)
        if trailing_spaces >= 2 {
            text_len = next_special - trailing_spaces;
        }
    }
    
    if text_len == 0 {
        // Only trailing spaces - don't consume them
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        )));
    }
    
    // Use slice() to properly advance by byte count (not character count!)
    let text_content = input.slice(..text_len);
    let rest = input.slice(text_len..);
    
    let span = to_parser_span(text_content);
    
    let node = Node {
        kind: NodeKind::Text(text_content.fragment().to_string()),
        span: Some(span),
        children: Vec::new(),
    };
    
    Ok((rest, node))
}

/// Parse a single special character as text (fallback for unmatched syntax)
///
/// When an inline element parser fails to match, this function consumes the
/// special character as plain text. For backticks, consumes all consecutive
/// backticks together.
///
/// # Arguments
/// * `input` - The input text as a GrammarSpan
///
/// # Returns
/// * `Ok((remaining, node))` - Successfully parsed text node with special character
/// * `Err(_)` - Input is empty
pub fn parse_special_as_text(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let text_fragment = input.fragment();
    
    if text_fragment.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Eof,
        )));
    }
    
    // Special case: if it's a backtick, consume all consecutive backticks
    // This prevents ```foo`` from being parsed as ` + ``foo``
    let char_len = if text_fragment.starts_with('`') {
        // Count all consecutive backticks
        text_fragment.chars().take_while(|&c| c == '`').count()
    } else {
        text_fragment.chars().next().map(|c| c.len_utf8()).unwrap_or(1)
    };
    
    let (rest, text_content) = take::<_, _, nom::error::Error<_>>(char_len)(input)?;
    
    let span = to_parser_span(text_content);
    
    let node = Node {
        kind: NodeKind::Text(text_content.fragment().to_string()),
        span: Some(span),
        children: Vec::new(),
    };
    
    Ok((rest, node))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_text_basic() {
        let input = GrammarSpan::new("Hello World*");
        let result = parse_text(input);
        
        assert!(result.is_ok(), "Failed to parse plain text");
        let (rest, node) = result.unwrap();
        
        assert_eq!(rest.fragment(), &"*");
        
        if let NodeKind::Text(text) = &node.kind {
            assert_eq!(text, "Hello World");
        } else {
            panic!("Expected Text node");
        }
    }

    #[test]
    fn smoke_test_parse_text_up_to_special() {
        let input = GrammarSpan::new("text with `code`");
        let result = parse_text(input);
        
        assert!(result.is_ok());
        let (rest, node) = result.unwrap();
        
        assert_eq!(rest.fragment(), &"`code`");
        
        if let NodeKind::Text(text) = &node.kind {
            assert_eq!(text, "text with ");
        }
    }

    #[test]
    fn smoke_test_parse_text_trailing_spaces() {
        let input = GrammarSpan::new("text  \n");
        let result = parse_text(input);
        
        assert!(result.is_ok());
        let (rest, node) = result.unwrap();
        
        // Should not consume trailing spaces before newline
        assert_eq!(rest.fragment(), &"  \n");
        
        if let NodeKind::Text(text) = &node.kind {
            assert_eq!(text, "text");
        }
    }

    #[test]
    fn smoke_test_parse_text_starts_with_special() {
        let input = GrammarSpan::new("*emphasis*");
        let result = parse_text(input);
        
        assert!(result.is_err(), "Should not parse text starting with special char");
    }

    #[test]
    fn smoke_test_parse_special_as_text_asterisk() {
        let input = GrammarSpan::new("* not emphasis");
        let result = parse_special_as_text(input);
        
        assert!(result.is_ok(), "Failed to parse special as text");
        let (rest, node) = result.unwrap();
        
        assert_eq!(rest.fragment(), &" not emphasis");
        
        if let NodeKind::Text(text) = &node.kind {
            assert_eq!(text, "*");
        }
    }

    #[test]
    fn smoke_test_parse_special_as_text_backticks() {
        let input = GrammarSpan::new("```not code");
        let result = parse_special_as_text(input);
        
        assert!(result.is_ok());
        let (rest, node) = result.unwrap();
        
        assert_eq!(rest.fragment(), &"not code");
        
        if let NodeKind::Text(text) = &node.kind {
            assert_eq!(text, "```");
        }
    }

    #[test]
    fn smoke_test_parse_text_position() {
        let input = GrammarSpan::new("Hello*");
        let result = parse_text(input);
        
        assert!(result.is_ok());
        let (_, node) = result.unwrap();
        
        assert!(node.span.is_some(), "Text should have position info");
        
        let span = node.span.unwrap();
        assert_eq!(span.start.offset, 0);
        assert_eq!(span.end.offset, 5); // "Hello" is 5 bytes
    }
}
