//! Subscript grammar (arrow-style Marco extension)
//!
//! Parses:
//! - `˅text˅`
//!
//! Returning the content span between the delimiters (without the delimiters).

use nom::{IResult, Input};

use super::Span;

const ARROW: char = '˅';

/// Parse subscript using `˅` delimiters.
///
/// This is a conservative extension parser (not CommonMark).
pub fn subscript_arrow(input: Span) -> IResult<Span, Span> {
    let s = input.fragment();

    if !s.starts_with(ARROW) {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char,
        )));
    }

    // Reject "˅˅" (reserve for potential future meaning).
    let mut chars = s.chars();
    chars.next();
    if chars.next() == Some(ARROW) {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    let opening_len = ARROW.len_utf8();
    let after_opening = input.take_from(opening_len);
    let remaining = after_opening.fragment();
    if remaining.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::TakeUntil,
        )));
    }

    // Find closing delimiter (˅), skipping over code spans.
    let mut pos = 0;
    while pos < remaining.len() {
        // Skip over code spans (backtick regions).
        if remaining[pos..].starts_with('`') {
            // Move to the next backtick.
            pos += 1;
            while pos < remaining.len() && !remaining[pos..].starts_with('`') {
                let Some(ch) = remaining[pos..].chars().next() else {
                    break;
                };
                pos += ch.len_utf8();
            }
            if pos < remaining.len() {
                pos += 1; // consume closing backtick
            }
            continue;
        }

        if remaining[pos..].starts_with(ARROW) {
            if pos == 0 {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::TakeUntil,
                )));
            }

            let content = after_opening.take(pos);
            let rest = after_opening.take_from(pos + opening_len);
            return Ok((rest, content));
        }

        let Some(ch) = remaining[pos..].chars().next() else {
            break;
        };
        pos += ch.len_utf8();
    }

    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::TakeUntil,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_subscript_arrow_basic() {
        let input = Span::new("˅sub˅ rest");
        let (rest, content) = subscript_arrow(input).unwrap();
        assert_eq!(*content.fragment(), "sub");
        assert_eq!(*rest.fragment(), " rest");
    }

    #[test]
    fn smoke_test_subscript_arrow_reject_double() {
        let input = Span::new("˅˅nope˅˅");
        assert!(subscript_arrow(input).is_err());
    }

    #[test]
    fn smoke_test_subscript_arrow_skips_code() {
        let input = Span::new("˅a `˅` b˅");
        let result = subscript_arrow(input);
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert_eq!(*content.fragment(), "a `˅` b");
    }
}
