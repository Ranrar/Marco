//! Emphasis grammar - single asterisk or underscore delimiters
//!
//! Per CommonMark spec section 6.4, emphasis uses `*text*` or `_text_` syntax.
//! Delimiters must follow left/right-flanking rules.

use nom::{
    IResult,
    Slice,
};
use super::Span;

/// Parse emphasis (single asterisk or underscore delimiters).
///
/// # Grammar
/// `*text*` or `_text_` where text cannot be empty.
///
/// # Returns
/// The content span between the delimiters (without the delimiters).
///
/// # Example
/// ```ignore
/// let input = Span::new("*emphasized* text");
/// let (rest, content) = emphasis(input).unwrap();
/// assert_eq!(*content.fragment(), "emphasized");
/// ```
pub fn emphasis(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing emphasis at: {:?}", input.fragment());
    
    // Try to parse emphasis with * or _ delimiter
    if let Ok(result) = emphasis_with_delimiter(input, '*') {
        return Ok(result);
    }
    
    emphasis_with_delimiter(input, '_')
}

/// Helper: Parse emphasis with a specific delimiter (* or _)
fn emphasis_with_delimiter(input: Span, delimiter: char) -> IResult<Span, Span> {
    let content_str = input.fragment();
    
    // Must start with exactly one delimiter (not two, which would be strong)
    if !content_str.starts_with(delimiter) {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    
    // Check if this is actually a strong delimiter (**)
    if content_str.len() > 1 && content_str.chars().nth(1) == Some(delimiter) {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    
    // Skip the opening delimiter - use slice() to preserve position
    let after_opening = input.slice(1..);
    
    // Find the closing delimiter
    let remaining_str = after_opening.fragment();
    let mut pos = 0;
    
    // Must have at least one character of content
    if remaining_str.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::TakeUntil)));
    }
    
    while pos < remaining_str.len() {
        // Skip over code spans (backtick regions) to give them precedence
        if remaining_str.as_bytes()[pos] == b'`' {
            pos += 1;
            // Find matching closing backtick
            while pos < remaining_str.len() && remaining_str.as_bytes()[pos] != b'`' {
                pos += 1;
            }
            if pos < remaining_str.len() {
                pos += 1; // Skip closing backtick
            }
            continue;
        }
        
        if remaining_str.as_bytes()[pos] == delimiter as u8 {
            // Check if next char is also the delimiter (would make it strong)
            if pos + 1 < remaining_str.len() && remaining_str.as_bytes()[pos + 1] == delimiter as u8 {
                // This is **, not a valid closing for emphasis
                pos += 2;
                continue;
            }
            
            // Found single delimiter - this is our closing
            if pos > 0 {  // Must have content
                // Use slice() to preserve absolute position information
                let content = input.slice(1..1 + pos);
                let remaining = input.slice(1 + pos + 1..);
                log::debug!("Emphasis content: {:?}", content.fragment());
                return Ok((remaining, content));
            }
        }
        pos += 1;
    }
    
    // No closing delimiter found
    Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::TakeUntil)))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn smoke_test_emphasis_asterisk() {
        let input = Span::new("*emphasized* text");
        let result = emphasis(input);
        assert!(result.is_ok());
        let (rest, content) = result.unwrap();
        assert_eq!(*content.fragment(), "emphasized");
        assert_eq!(*rest.fragment(), " text");
    }
    
    #[test]
    fn smoke_test_emphasis_underscore() {
        let input = Span::new("_emphasized_ text");
        let result = emphasis(input);
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert_eq!(*content.fragment(), "emphasized");
    }
    
    #[test]
    fn smoke_test_emphasis_no_closing() {
        let input = Span::new("*no closing");
        let result = emphasis(input);
        assert!(result.is_err());
    }
    
    #[test]
    fn smoke_test_emphasis_empty_content() {
        let input = Span::new("**");
        let result = emphasis(input);
        assert!(result.is_err());  // Empty content or strong delimiter
    }
    
    #[test]
    fn smoke_test_emphasis_with_code_span() {
        let input = Span::new("*text with `code`* more");
        let result = emphasis(input);
        assert!(result.is_ok());
    }
    
    #[test]
    fn smoke_test_emphasis_not_strong() {
        let input = Span::new("**strong**");
        let result = emphasis(input);
        assert!(result.is_err());  // Should fail, this is strong not emphasis
    }
    
    #[test]
    fn smoke_test_emphasis_single_char() {
        let input = Span::new("*a* text");
        let result = emphasis(input);
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert_eq!(*content.fragment(), "a");
    }
}
