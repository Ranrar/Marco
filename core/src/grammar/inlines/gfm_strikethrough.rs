//! Strikethrough grammar (GFM extension)
//!
//! Parses:
//! - `~~text~~`
//!
//! Returning the content span between the delimiters (without the delimiters).

use nom::{IResult, Input};

use super::Span;

/// Parse strikethrough using `~~` delimiters.
///
/// This is a conservative extension parser (not CommonMark).
pub fn strikethrough(input: Span) -> IResult<Span, Span> {
    let s = input.fragment();

    // Must start with exactly two tildes.
    if !s.starts_with("~~") {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Avoid consuming 3+ tildes as "~~".
    if s.as_bytes().get(2) == Some(&b'~') {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    let after_opening = input.take_from(2);
    let remaining = after_opening.fragment();
    if remaining.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::TakeUntil,
        )));
    }

    // Find closing delimiter pair.
    let mut pos = 0;
    while pos + 1 < remaining.len() {
        // Skip over code spans (backtick regions) to give them precedence.
        if remaining.as_bytes()[pos] == b'`' {
            pos += 1;
            while pos < remaining.len() && remaining.as_bytes()[pos] != b'`' {
                pos += 1;
            }
            if pos < remaining.len() {
                pos += 1;
            }
            continue;
        }

        if remaining.as_bytes()[pos] == b'~' && remaining.as_bytes()[pos + 1] == b'~' {
            if pos == 0 {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::TakeUntil,
                )));
            }

            let content = after_opening.take(pos);
            let rest = after_opening.take_from(pos + 2);
            return Ok((rest, content));
        }

        pos += 1;
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
    fn smoke_test_strikethrough_basic() {
        let input = Span::new("~~strike~~ rest");
        let (rest, content) = strikethrough(input).unwrap();
        assert_eq!(*content.fragment(), "strike");
        assert_eq!(*rest.fragment(), " rest");
    }

    #[test]
    fn smoke_test_strikethrough_reject_three() {
        let input = Span::new("~~~nope~~~");
        assert!(strikethrough(input).is_err());
    }

    #[test]
    fn smoke_test_strikethrough_skips_code() {
        let input = Span::new("~~a `~~` b~~");
        let result = strikethrough(input);
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert_eq!(*content.fragment(), "a `~~` b");
    }
}
