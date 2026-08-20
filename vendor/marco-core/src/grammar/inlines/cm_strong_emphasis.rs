//! Strong+emphasis (triple delimiter) grammar
//!
//! This is a small extension on top of our CommonMark-like emphasis/strong
//! handling.
//!
//! Parses:
//! - `***text***`
//! - `___text___`
//!
//! Returning the content span between the delimiters (without the delimiters).

use nom::{IResult, Input};

use super::Span;

/// Parse combined strong+emphasis using triple delimiters.
///
/// # Grammar
/// `***text***` or `___text___` where text cannot be empty.
///
/// This parser is intentionally conservative and does not attempt to fully
/// implement the CommonMark delimiter algorithm. It exists to ensure that the
/// "triple delimiter" case is parsed as a single inline node rather than as
/// a strong node plus a dangling delimiter.
pub fn strong_emphasis(input: Span) -> IResult<Span, Span> {
    if let Ok(result) = strong_emphasis_with_delimiter(input, '*') {
        return Ok(result);
    }
    strong_emphasis_with_delimiter(input, '_')
}

fn strong_emphasis_with_delimiter(input: Span, delimiter: char) -> IResult<Span, Span> {
    let s = input.fragment();

    // Must start with exactly three delimiters.
    if !s.starts_with(&format!("{d}{d}{d}", d = delimiter)) {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Avoid consuming 4+ delimiters as triple.
    if s.chars().nth(3) == Some(delimiter) {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Skip opening delimiters.
    let after_opening = input.take_from(3);
    let remaining = after_opening.fragment();
    if remaining.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::TakeUntil,
        )));
    }

    // Find closing delimiter triplet.
    let mut pos = 0;
    while pos < remaining.len() {
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

        if pos + 2 < remaining.len()
            && remaining.as_bytes()[pos] == delimiter as u8
            && remaining.as_bytes()[pos + 1] == delimiter as u8
            && remaining.as_bytes()[pos + 2] == delimiter as u8
        {
            if pos == 0 {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::TakeUntil,
                )));
            }

            let content = after_opening.take(pos);
            let rest = after_opening.take_from(pos + 3);
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
    fn smoke_test_triple_asterisk() {
        let input = Span::new("***bold and italic*** rest");
        let (rest, content) = strong_emphasis(input).unwrap();
        assert_eq!(*content.fragment(), "bold and italic");
        assert_eq!(*rest.fragment(), " rest");
    }

    #[test]
    fn smoke_test_triple_underscore() {
        let input = Span::new("___bold and italic___");
        let (rest, content) = strong_emphasis(input).unwrap();
        assert_eq!(*content.fragment(), "bold and italic");
        assert_eq!(*rest.fragment(), "");
    }

    #[test]
    fn smoke_test_reject_four_delimiters() {
        let input = Span::new("****nope****");
        assert!(strong_emphasis(input).is_err());
    }
}
