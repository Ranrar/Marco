//! Code span grammar - inline code
//!
//! Per CommonMark spec section 6.3, code spans are delimited by backticks.
//! The opening and closing delimiter must have the same number of backticks.
//! Content between the delimiters is treated as literal text.

use nom::{
    IResult,
    character::complete::char,
    multi::many1_count,
    combinator::recognize,
};
use nom_locate::LocatedSpan;
use super::Span;

/// Parse a code span (inline code).
///
/// # Grammar
/// Opening backticks (1 or more) + content + closing backticks (same count).
/// The number of opening and closing backticks must match exactly.
///
/// # Returns
/// The content span between the backticks (without the backticks themselves).
///
/// # Example
/// ```ignore
/// let input = Span::new("`code` text");
/// let (rest, content) = code_span(input).unwrap();
/// assert_eq!(*content.fragment(), "code");
/// ```
pub fn code_span(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing code span at: {:?}", input.fragment());
    
    // Count opening backticks
    let (input, opening) = recognize(many1_count(char('`')))(input)?;
    let backtick_count = opening.fragment().len();
    log::debug!("Found {} opening backticks", backtick_count);
    
    // Find the closing backticks by searching through the string
    let content_str = input.fragment();
    let mut pos = 0;
    
    while pos < content_str.len() {
        if content_str.as_bytes()[pos] == b'`' {
            // Count consecutive backticks at this position
            let mut tick_count = 0;
            let mut check_pos = pos;
            while check_pos < content_str.len() && content_str.as_bytes()[check_pos] == b'`' {
                tick_count += 1;
                check_pos += 1;
            }
            
            // If we found exactly the right number, this is our closing delimiter
            if tick_count == backtick_count {
                let content = LocatedSpan::new(&content_str[..pos]);
                let remaining = LocatedSpan::new(&content_str[check_pos..]);
                log::debug!("Code span content: {:?}", content.fragment());
                return Ok((remaining, content));
            }
            
            // Skip past these backticks
            pos = check_pos;
        } else {
            pos += 1;
        }
    }
    
    // Didn't find matching closing backticks
    Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::TakeUntil)))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn smoke_test_code_span_basic() {
        let input = Span::new("`code` text");
        let result = code_span(input);
        assert!(result.is_ok());
        let (_rest, content) = result.unwrap();
        assert_eq!(*content.fragment(), "code");
        assert_eq!(*_rest.fragment(), " text");
    }
    
    #[test]
    fn smoke_test_code_span_double_backticks() {
        let input = Span::new("``code with ` backtick`` text");
        let result = code_span(input);
        assert!(result.is_ok());
        let (_rest, content) = result.unwrap();
        assert_eq!(*content.fragment(), "code with ` backtick");
    }
    
    #[test]
    fn smoke_test_code_span_triple_backticks() {
        let input = Span::new("```code```");
        let result = code_span(input);
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert_eq!(*content.fragment(), "code");
    }
    
    #[test]
    fn smoke_test_code_span_no_closing() {
        let input = Span::new("`code without closing");
        let result = code_span(input);
        assert!(result.is_err());
    }
    
    #[test]
    fn smoke_test_code_span_mismatched_backticks() {
        let input = Span::new("`code`` text");
        let result = code_span(input);
        assert!(result.is_err());  // 1 opening, 2 closing
    }
    
    #[test]
    fn smoke_test_code_span_empty() {
        let input = Span::new("` ` text");  // Space between backticks
        let result = code_span(input);
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert_eq!(*content.fragment(), " ");
    }
    
    #[test]
    fn smoke_test_code_span_with_spaces() {
        let input = Span::new("`  code  ` text");
        let result = code_span(input);
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert_eq!(*content.fragment(), "  code  ");
    }
}
