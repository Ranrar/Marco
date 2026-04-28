//! Strong emphasis grammar - double asterisk or underscore delimiters
//!
//! Per CommonMark spec section 6.4, strong emphasis uses `**text**` or `__text__` syntax.
//! Delimiters must follow left/right-flanking rules.

use super::Span;
use nom::{bytes::complete::take, IResult};

/// Parse strong emphasis (double asterisk or underscore delimiters).
///
/// # Grammar
/// `**text**` or `__text__` where text cannot be empty.
///
/// # Returns
/// The content span between the delimiters (without the delimiters).
///
/// # Example
/// ```ignore
/// let input = Span::new("**strong** text");
/// let (rest, content) = strong(input).unwrap();
/// assert_eq!(*content.fragment(), "strong");
/// ```
pub fn strong(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing strong emphasis at: {:?}", input.fragment());

    // Try to parse strong with ** or __ delimiter
    if let Ok(result) = strong_with_delimiter(input, '*') {
        return Ok(result);
    }

    strong_with_delimiter(input, '_')
}

/// Helper: Parse strong emphasis with a specific delimiter (** or __)
fn strong_with_delimiter(input: Span, delimiter: char) -> IResult<Span, Span> {
    let content_str = input.fragment();

    // Must start with exactly two delimiters
    if content_str.len() < 2 {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    if !content_str.starts_with(&format!("{}{}", delimiter, delimiter)) {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Skip the opening delimiters using take()
    let (after_opening, _) = take(2usize)(input)?;
    let remaining_str = after_opening.fragment();

    // Must have at least one character of content
    if remaining_str.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::TakeUntil,
        )));
    }

    // Find the closing delimiter pair
    let mut pos = 0;

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

        // Look for double delimiter
        if pos + 1 < remaining_str.len()
            && remaining_str.as_bytes()[pos] == delimiter as u8
            && remaining_str.as_bytes()[pos + 1] == delimiter as u8
        {
            // Found closing delimiter pair
            if pos > 0 {
                // Must have content
                // Use take() to extract content and remaining
                let (after_content, content) = take(pos)(after_opening)?;
                let (remaining, _closing) = take(2usize)(after_content)?;

                return Ok((remaining, content));
            }
        }
        pos += 1;
    }

    // No closing delimiter found
    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::TakeUntil,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_strong_asterisk() {
        let input = Span::new("**strong** text");
        let result = strong(input);
        assert!(result.is_ok());
        let (rest, content) = result.unwrap();
        assert_eq!(*content.fragment(), "strong");
        assert_eq!(*rest.fragment(), " text");
    }

    #[test]
    fn smoke_test_strong_underscore() {
        let input = Span::new("__strong__ text");
        let result = strong(input);
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert_eq!(*content.fragment(), "strong");
    }

    #[test]
    fn smoke_test_strong_no_closing() {
        let input = Span::new("**no closing");
        let result = strong(input);
        assert!(result.is_err());
    }

    #[test]
    fn smoke_test_strong_empty_content() {
        let input = Span::new("****");
        let result = strong(input);
        assert!(result.is_err()); // Empty content
    }

    #[test]
    fn smoke_test_strong_with_code_span() {
        let input = Span::new("**text with `code`** more");
        let result = strong(input);
        assert!(result.is_ok());
    }

    #[test]
    fn smoke_test_strong_single_char() {
        let input = Span::new("**a** text");
        let result = strong(input);
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert_eq!(*content.fragment(), "a");
    }

    #[test]
    fn smoke_test_strong_not_emphasis() {
        let input = Span::new("*emphasis*");
        let result = strong(input);
        assert!(result.is_err()); // Should fail, this is emphasis not strong
    }
}
