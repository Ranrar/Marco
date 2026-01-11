//! Line break grammar - soft and hard line breaks
//!
//! Per CommonMark spec sections 6.14-6.15:
//! - Hard line break: two spaces + newline, or backslash + newline
//! - Soft line break: regular newline (not in code span or HTML tag)

use super::Span;
use nom::{
    branch::alt, bytes::complete::tag, character::complete::line_ending, combinator::recognize,
    IResult, Parser,
};

/// Parse a soft line break.
///
/// # Grammar
/// A regular newline that doesn't end a paragraph.
///
/// # Returns
/// Unit `()` indicating a soft line break was found.
///
/// # Example
/// ```ignore
/// let input = Span::new("\nmore text");
/// let (rest, _) = soft_line_break(input).unwrap();
/// assert_eq!(*rest.fragment(), "more text");
/// ```
pub fn soft_line_break(input: Span) -> IResult<Span, ()> {
    log::debug!("Parsing soft line break");

    // Just a line ending (newline)
    let (input, _) = line_ending(input)?;

    Ok((input, ()))
}

/// Parse a hard line break.
///
/// # Grammar
/// Two or more spaces followed by newline, OR backslash followed by newline.
///
/// # Returns
/// Unit `()` indicating a hard line break was found.
///
/// # Example
/// ```ignore
/// let input = Span::new("  \nmore text");
/// let (rest, _) = hard_line_break(input).unwrap();
/// ```
pub fn hard_line_break(input: Span) -> IResult<Span, ()> {
    log::debug!("Parsing hard line break");

    // Two or more spaces followed by newline, OR backslash followed by newline
    let (input, _) = alt((
        recognize((tag("  "), line_ending)), // 2+ spaces + newline
        recognize((tag("\\"), line_ending)), // backslash + newline
    ))
    .parse(input)?;

    Ok((input, ()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_soft_line_break() {
        let input = Span::new("\nmore text");
        let result = soft_line_break(input);
        assert!(result.is_ok());
        let (rest, _) = result.unwrap();
        assert_eq!(*rest.fragment(), "more text");
    }

    #[test]
    fn smoke_test_hard_line_break_spaces() {
        let input = Span::new("  \nmore text");
        let result = hard_line_break(input);
        assert!(result.is_ok());
        let (rest, _) = result.unwrap();
        assert_eq!(*rest.fragment(), "more text");
    }

    #[test]
    fn smoke_test_hard_line_break_backslash() {
        let input = Span::new("\\\nmore text");
        let result = hard_line_break(input);
        assert!(result.is_ok());
        let (rest, _) = result.unwrap();
        assert_eq!(*rest.fragment(), "more text");
    }

    #[test]
    fn smoke_test_hard_line_break_exact_two_spaces() {
        let input = Span::new("  \nmore text");
        let result = hard_line_break(input);
        // Exactly 2 spaces followed by newline
        assert!(result.is_ok());
    }

    #[test]
    fn smoke_test_hard_line_break_one_space_fails() {
        let input = Span::new(" \nmore text");
        let result = hard_line_break(input);
        assert!(result.is_err()); // Only 1 space should fail
    }

    #[test]
    fn smoke_test_soft_line_break_unix() {
        let input = Span::new("\ntext");
        let result = soft_line_break(input);
        assert!(result.is_ok());
    }

    #[test]
    fn smoke_test_soft_line_break_windows() {
        let input = Span::new("\r\ntext");
        let result = soft_line_break(input);
        assert!(result.is_ok());
    }
}
