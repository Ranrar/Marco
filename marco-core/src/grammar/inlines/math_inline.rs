//! Inline math grammar - LaTeX math delimited by `$...$`
//!
//! This module parses inline math expressions (e.g. `$E = mc^2$`) for
//! rendering with KaTeX.

use super::Span;
use nom::{
    bytes::complete::{tag, take_until},
    IResult,
};

/// Parse an inline math span delimited by single `$`
///
/// # Grammar
/// `$` + content + `$`
///
/// # Returns
/// The LaTeX content span (without the `$` delimiters).
///
/// # Example
/// ```ignore
/// let input = Span::new("$x^2 + y^2 = r^2$ text");
/// let (rest, content) = inline_math(input).unwrap();
/// assert_eq!(*content.fragment(), "x^2 + y^2 = r^2");
/// ```
pub fn inline_math(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing inline math at: {:?}", input.fragment());

    // Opening delimiter
    let (input, _) = tag("$")(input)?;

    // Parse content until closing $
    // Allow anything except $$ (which would be display math)
    let content_str = input.fragment();

    // Check for immediate second $ (would be display math)
    if content_str.starts_with('$') {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Find the closing delimiter
    let (input, content) = take_until("$")(input)?;

    // Consume closing delimiter
    let (input, _) = tag("$")(input)?;

    log::debug!("Inline math content: {:?}", content.fragment());
    Ok((input, content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_inline_math_basic() {
        let input = Span::new("$E = mc^2$ text");
        let result = inline_math(input);
        assert!(result.is_ok());
        let (rest, content) = result.unwrap();
        assert_eq!(*content.fragment(), "E = mc^2");
        assert_eq!(*rest.fragment(), " text");
    }

    #[test]
    fn smoke_test_inline_math_with_special_chars() {
        let input = Span::new("$\\frac{a}{b}$");
        let result = inline_math(input);
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert_eq!(*content.fragment(), "\\frac{a}{b}");
    }

    #[test]
    fn smoke_test_inline_math_not_display() {
        let input = Span::new("$$x^2$$");
        let result = inline_math(input);
        assert!(result.is_err()); // Should fail - this is display math
    }
}
