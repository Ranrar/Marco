// Marco Sliders Grammar (Marco_Extended)
//
// Syntax:
//
// @slidestart
// <!-- slide 1 -->
//
// ---
//
// <!-- slide 2 -->
//
// --
//
// <!-- slide 2b (vertical) -->
//
// @slideend
//
// Notes:
// - Sliders are a Marco extension inspired by VuePress revealjs slide syntax.
// - Slide separators are:
//   - `---` for a new horizontal slide
//   - `--` for a new vertical slide (currently preserved as metadata only)
// - Markers/separators are only recognized when not inside fenced code blocks.
// - Up to 3 leading spaces are allowed before markers/separators.

use crate::grammar::shared::Span;
use nom::Input;
use nom::{
    bytes::complete::{tag, take_while},
    character::complete::{line_ending, not_line_ending},
    combinator::opt,
    IResult, Parser,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarcoSlide<'a> {
    pub content: Span<'a>,
    pub vertical: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarcoSlideDeck<'a> {
    pub timer_seconds: Option<u32>,
    pub slides: Vec<MarcoSlide<'a>>,
}

/// Parse a Marco `@slidestart ... @slideend` slide deck.
///
/// This is a block-level parser that captures raw slide body spans; the parser
/// layer is responsible for recursively parsing each slide's markdown content.
pub fn marco_slide_deck(input: Span<'_>) -> IResult<Span<'_>, MarcoSlideDeck<'_>> {
    let original_input = input;

    // Optional leading spaces (0-3 allowed)
    let (input, leading_spaces) = take_while(|c| c == ' ').parse(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(
            original_input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Opening marker
    let (input, _) = tag("@slidestart")(input)?;

    // Optional timer suffix: :t<digits>
    let mut timer_seconds: Option<u32> = None;
    let mut input = input;
    if let Some(after_colon) = input.fragment().strip_prefix(':') {
        // Re-anchor span by consuming ':'
        let (after_colon_span, _) = tag(":")(input)?;
        input = after_colon_span;

        let (after_t, _) = tag("t")(input)?;
        input = after_t;

        // Parse digits
        let (after_digits, digits_span) = take_while(|c: char| c.is_ascii_digit()).parse(input)?;
        if digits_span.fragment().is_empty() {
            return Err(nom::Err::Error(nom::error::Error::new(
                original_input,
                nom::error::ErrorKind::Tag,
            )));
        }

        let secs: u32 = digits_span.fragment().parse().map_err(|_| {
            nom::Err::Error(nom::error::Error::new(
                original_input,
                nom::error::ErrorKind::Tag,
            ))
        })?;
        if secs == 0 {
            return Err(nom::Err::Error(nom::error::Error::new(
                original_input,
                nom::error::ErrorKind::Tag,
            )));
        }
        timer_seconds = Some(secs);
        input = after_digits;

        // Silence unused variable warning (kept for clarity):
        let _ = after_colon;
    }

    // Must be followed by whitespace, newline, or end.
    if let Some(ch) = input.fragment().chars().next() {
        if ch != ' ' && ch != '\t' && ch != '\n' && ch != '\r' {
            return Err(nom::Err::Error(nom::error::Error::new(
                original_input,
                nom::error::ErrorKind::Tag,
            )));
        }
    }

    // Consume rest of opening line + optional newline
    let (input, rest_of_line) = not_line_ending::<_, nom::error::Error<Span>>(input)?;
    if !rest_of_line.fragment().trim().is_empty() {
        // We currently don't support themes/other args on the start line.
        return Err(nom::Err::Error(nom::error::Error::new(
            original_input,
            nom::error::ErrorKind::Tag,
        )));
    }
    let (mut input, _) = opt(line_ending).parse(input)?;

    let mut slides: Vec<MarcoSlide<'_>> = Vec::new();

    // Track slide body slicing within the original input.
    let mut current_slide_start_offset = input.location_offset();
    let mut next_slide_vertical = false;

    // Track fenced code blocks so separators/end markers inside them don't split.
    let mut in_fence: Option<(char, usize)> = None;

    loop {
        // End-of-input: invalid (requires explicit closing `@slideend`)
        if input.fragment().is_empty() {
            return Err(nom::Err::Error(nom::error::Error::new(
                original_input,
                nom::error::ErrorKind::Eof,
            )));
        }

        let line_start_span = input;
        let (after_line, line_span) = not_line_ending::<_, nom::error::Error<Span>>(input)?;
        let line = *line_span.fragment();

        fn trim_upto_3_spaces(s: &str) -> (usize, &str) {
            let bytes = s.as_bytes();
            let mut i = 0usize;
            for _ in 0..3 {
                if bytes.get(i) == Some(&b' ') {
                    i += 1;
                } else {
                    break;
                }
            }
            (i, &s[i..])
        }

        fn fence_prefix(rest: &str) -> Option<(char, usize, &str)> {
            let mut chars = rest.chars();
            let ch = chars.next()?;
            if ch != '`' && ch != '~' {
                return None;
            }
            let mut count = 1usize;
            for c in chars.clone() {
                if c == ch {
                    count += 1;
                } else {
                    break;
                }
            }
            if count >= 3 {
                Some((ch, count, &rest[count..]))
            } else {
                None
            }
        }

        let (_indent_len, rest) = trim_upto_3_spaces(line);

        // Fence handling
        if let Some((fch, fcount, after_fence)) = fence_prefix(rest) {
            match in_fence {
                None => {
                    in_fence = Some((fch, fcount));
                }
                Some((open_ch, open_count)) => {
                    if fch == open_ch && fcount >= open_count && after_fence.trim().is_empty() {
                        in_fence = None;
                    }
                }
            }
        }

        if in_fence.is_none() {
            // Closing marker
            let (_indent_len, rest) = trim_upto_3_spaces(line);
            if let Some(after) = rest.strip_prefix("@slideend") {
                if after.trim().is_empty() {
                    let content_end_offset = line_start_span.location_offset();
                    let content_span = make_slice_span(
                        original_input,
                        current_slide_start_offset,
                        content_end_offset,
                    );
                    slides.push(MarcoSlide {
                        content: content_span,
                        vertical: next_slide_vertical,
                    });

                    // Consume closing line + optional newline and return
                    let (rest_after_close, _) = opt(line_ending).parse(after_line)?;
                    return Ok((
                        rest_after_close,
                        MarcoSlideDeck {
                            timer_seconds,
                            slides,
                        },
                    ));
                }
            }

            // Separators
            let sep = rest.trim();
            if sep == "---" || sep == "--" {
                let content_end_offset = line_start_span.location_offset();
                let content_span = make_slice_span(
                    original_input,
                    current_slide_start_offset,
                    content_end_offset,
                );
                slides.push(MarcoSlide {
                    content: content_span,
                    vertical: next_slide_vertical,
                });

                // Next slide begins after the separator line.
                let after_sep = consume_line(after_line)?;
                current_slide_start_offset = after_sep.location_offset();
                next_slide_vertical = sep == "--";
                input = after_sep;
                continue;
            }
        }

        // Regular line: keep scanning
        input = consume_line(after_line)?;
    }
}

fn consume_line(
    input_after_not_line_ending: Span,
) -> Result<Span, nom::Err<nom::error::Error<Span>>> {
    // `not_line_ending` does not consume the newline. Consume it if present.
    opt(line_ending)
        .parse(input_after_not_line_ending)
        .map(|(rest, _)| rest)
}

fn make_slice_span<'a>(original: Span<'a>, start_offset: usize, end_offset: usize) -> Span<'a> {
    let orig_offset = original.location_offset();
    let start_rel = start_offset.saturating_sub(orig_offset);
    let end_rel = end_offset.saturating_sub(orig_offset);
    let len = end_rel.saturating_sub(start_rel);

    // Preserve location metadata by slicing from the original LocatedSpan.
    original.take_from(start_rel).take(len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parses_three_slides_with_separators() {
        let input = Span::new("@slidestart\nA\n\n---\nB\n\n---\nC\n@slideend\n");
        let res = marco_slide_deck(input);
        assert!(res.is_ok());
        let (_rest, deck) = res.unwrap();
        assert_eq!(deck.timer_seconds, None);
        assert_eq!(deck.slides.len(), 3);
        assert!(deck.slides[0].content.fragment().contains("A"));
        assert!(deck.slides[1].content.fragment().contains("B"));
        assert!(deck.slides[2].content.fragment().contains("C"));
    }

    #[test]
    fn smoke_test_parses_timer_suffix() {
        let input = Span::new("@slidestart:t5\nA\n@slideend\n");
        let res = marco_slide_deck(input);
        assert!(res.is_ok());
        let (_rest, deck) = res.unwrap();
        assert_eq!(deck.timer_seconds, Some(5));
        assert_eq!(deck.slides.len(), 1);
    }

    #[test]
    fn smoke_test_marks_vertical_split() {
        let input = Span::new("@slidestart\nA\n\n--\nB\n@slideend\n");
        let (_rest, deck) = marco_slide_deck(input).expect("parse failed");
        assert_eq!(deck.slides.len(), 2);
        assert!(!deck.slides[0].vertical);
        assert!(deck.slides[1].vertical);
    }

    #[test]
    fn smoke_test_ignores_separators_inside_fenced_code() {
        let input = Span::new("@slidestart\n```\n---\n```\n\n---\nOK\n@slideend\n");
        let (_rest, deck) = marco_slide_deck(input).expect("parse failed");
        assert_eq!(deck.slides.len(), 2);
        assert!(deck.slides[0].content.fragment().contains("```\n---\n```"));
    }

    #[test]
    fn smoke_test_requires_closing_marker() {
        let input = Span::new("@slidestart\nA\n");
        assert!(marco_slide_deck(input).is_err());
    }
}
