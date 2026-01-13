// CommonMark Thematic Break Grammar
// Parses horizontal rules: ---, ***, ___
//
// Per CommonMark spec:
// - Must have at least 3 matching characters (-, *, or _)
// - Can have spaces or tabs between characters
// - Can have 0-3 leading spaces
// - Must be on its own line (optionally followed by trailing spaces/tabs)

use crate::grammar::shared::Span;
use nom::{
    branch::alt,
    bytes::complete::take_while,
    character::complete::line_ending,
    combinator::{eof, recognize},
    IResult, Input, Parser,
};
// Note: `LocatedSpan` not used in this file; it was previously imported for tests.

/// Parse a thematic break (horizontal rule)
///
/// Examples:
/// - `---`
/// - `***`
/// - `___`
/// - `- - -` (with spaces)
/// - `  ***` (with leading spaces)
///
/// # Arguments
/// * `input` - The input span to parse
///
/// # Returns
/// `Ok((remaining, span))` if successful, where span is a dummy "---" marker
pub fn thematic_break(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing thematic break: {:?}", input.fragment());

    let start = input;

    // 1. Optional leading spaces (0-3 spaces allowed)
    let (input, leading_spaces) = take_while(|c| c == ' ').parse(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(
            start,
            nom::error::ErrorKind::Tag,
        )));
    }

    // 2. Determine which character is used (-, *, or _)
    let (input, first_char) = nom::character::complete::one_of("-*_")(input)?;

    // 3. Count matching characters with optional spaces/tabs between
    let mut remaining = input;
    let mut char_count = 1; // Already found first char

    loop {
        // Try to consume optional spaces and tabs
        let (input_after_space, _) = take_while(|c| c == ' ' || c == '\t').parse(remaining)?;

        // Try to match the same character
        if let Ok((input_after_char, _matched_char)) = nom::character::complete::char::<
            _,
            nom::error::Error<Span>,
        >(first_char)(input_after_space)
        {
            char_count += 1;
            remaining = input_after_char;
        } else {
            // No more matching chars, check if we're at end of line
            remaining = input_after_space;
            break;
        }
    }

    // 4. Must have at least 3 matching characters
    if char_count < 3 {
        return Err(nom::Err::Error(nom::error::Error::new(
            start,
            nom::error::ErrorKind::Tag,
        )));
    }

    // 5. Must be followed by whitespace or end of input (nothing else on the line)
    let (remaining, _) = take_while(|c| c == ' ' || c == '\t').parse(remaining)?;

    // Check for end of line or end of input
    let (remaining, _) = alt((recognize(line_ending), recognize(eof))).parse(remaining)?;

    log::debug!(
        "Thematic break parsed: {} matching '{}' chars",
        char_count,
        first_char
    );

    // Calculate the span of the thematic break (from start to before newline/EOF)
    // Use slice to preserve position information. The matched length is the
    // difference between the remainder's start offset and the original start
    // offset. Use saturating_sub to be defensive against malformed offsets.
    let break_len = remaining
        .location_offset()
        .saturating_sub(start.location_offset());
    let break_span = start.take(break_len);

    Ok((remaining, break_span))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_thematic_break_hyphens() {
        let input = Span::new("---\n");
        let result = thematic_break(input);
        assert!(result.is_ok());
        let (remaining, _) = result.unwrap();
        assert_eq!(*remaining.fragment(), "");
    }

    #[test]
    fn smoke_test_thematic_break_asterisks() {
        let input = Span::new("***\n");
        let result = thematic_break(input);
        assert!(result.is_ok());
        let (remaining, _) = result.unwrap();
        assert_eq!(*remaining.fragment(), "");
    }

    #[test]
    fn smoke_test_thematic_break_underscores() {
        let input = Span::new("___\n");
        let result = thematic_break(input);
        assert!(result.is_ok());
        let (remaining, _) = result.unwrap();
        assert_eq!(*remaining.fragment(), "");
    }

    #[test]
    fn smoke_test_thematic_break_with_spaces() {
        let input = Span::new("- - -\n");
        let result = thematic_break(input);
        assert!(result.is_ok());
        let (remaining, _) = result.unwrap();
        assert_eq!(*remaining.fragment(), "");
    }

    #[test]
    fn smoke_test_thematic_break_many_chars() {
        let input = Span::new("----------\n");
        let result = thematic_break(input);
        assert!(result.is_ok());
    }

    #[test]
    fn smoke_test_thematic_break_leading_spaces() {
        let input = Span::new("  ***\n");
        let result = thematic_break(input);
        assert!(result.is_ok());
    }

    #[test]
    fn smoke_test_thematic_break_two_chars_fails() {
        let input = Span::new("--\n");
        let result = thematic_break(input);
        assert!(result.is_err());
    }

    #[test]
    fn smoke_test_thematic_break_mixed_chars_fails() {
        let input = Span::new("-*-\n");
        let result = thematic_break(input);
        assert!(result.is_err());
    }
}
