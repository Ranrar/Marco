//! Superscript grammar (Marco extension)
//!
//! Parses:
//! - `^text^`
//!
//! Returning the content span between the delimiters (without the delimiters).

use nom::{IResult, Input};

use super::Span;

/// Parse superscript using `^` delimiters.
///
/// This is a conservative extension parser (not CommonMark).
pub fn superscript(input: Span) -> IResult<Span, Span> {
    let s = input.fragment();

    if !s.starts_with('^') {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char,
        )));
    }

    // Reject "^^" (reserve for potential future meaning).
    if s.as_bytes().get(1) == Some(&b'^') {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    let after_opening = input.take_from(1);
    let remaining = after_opening.fragment();
    if remaining.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::TakeUntil,
        )));
    }

    let mut pos = 0;
    while pos < remaining.len() {
        // Skip over code spans (backtick regions).
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

        if remaining.as_bytes()[pos] == b'^' {
            if pos == 0 {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::TakeUntil,
                )));
            }

            let content = after_opening.take(pos);
            let rest = after_opening.take_from(pos + 1);
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
    fn smoke_test_superscript_basic() {
        let input = Span::new("^sup^ rest");
        let (rest, content) = superscript(input).unwrap();
        assert_eq!(*content.fragment(), "sup");
        assert_eq!(*rest.fragment(), " rest");
    }

    #[test]
    fn smoke_test_superscript_reject_double() {
        let input = Span::new("^^nope^^");
        assert!(superscript(input).is_err());
    }
}
