//! Display math grammar - LaTeX math delimited by `$$...$$`
//!
//! This module parses display math expressions (e.g. `$$\int x^2 dx$$`) for
//! rendering with KaTeX in display mode.

use super::Span;
use nom::{
    bytes::complete::{tag, take_until},
    IResult,
};

/// Parse a display math block delimited by double `$$`
///
/// # Grammar
/// `$$` + content + `$$`
///
/// # Returns
/// The LaTeX content span (without the `$$` delimiters).
///
/// # Example
/// ```ignore
/// let input = Span::new("$$\\int_0^\\infty e^{-x^2} dx = \\sqrt{\\pi}$$ text");
/// let (rest, content) = display_math(input).unwrap();
/// ```
pub fn display_math(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing display math at: {:?}", input.fragment());

    // Opening delimiter
    let (input, _) = tag("$$")(input)?;

    // Parse content until closing $$
    let (input, content) = take_until("$$")(input)?;

    // Consume closing delimiter
    let (input, _) = tag("$$")(input)?;

    log::debug!("Display math content: {:?}", content.fragment());
    Ok((input, content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_display_math_basic() {
        let input = Span::new("$$x^2 + y^2 = r^2$$ text");
        let result = display_math(input);
        assert!(result.is_ok());
        let (rest, content) = result.unwrap();
        assert_eq!(*content.fragment(), "x^2 + y^2 = r^2");
        assert_eq!(*rest.fragment(), " text");
    }

    #[test]
    fn smoke_test_display_math_multiline() {
        let input = Span::new("$$\\int_0^\\infty e^{-x^2} dx\n= \\sqrt{\\pi}$$");
        let result = display_math(input);
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert!(content.fragment().contains("\\int"));
        assert!(content.fragment().contains("\\sqrt"));
    }
}
