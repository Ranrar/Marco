// Marco Extended Tab Blocks Grammar
//
// Syntax:
//
// :::tab
// @tab First title
// Content...
//
// @tab Second title
// More content...
// :::
//
// Notes:
// - Tab blocks are a Marco extension.
// - `@tab` and the closing `:::` are only recognized at the top-level within the
//   container (not inside fenced code blocks).
// - Up to 3 leading spaces are allowed before markers, similar to CommonMark.

use crate::grammar::shared::Span;
use nom::Input;
use nom::{
    bytes::complete::{tag, take_while},
    character::complete::{line_ending, not_line_ending},
    combinator::opt,
    IResult, Parser,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarcoTabItem<'a> {
    pub title: String,
    pub content: Span<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarcoTabBlock<'a> {
    pub items: Vec<MarcoTabItem<'a>>,
}

/// Parse a Marco `:::tab` container block.
///
/// This is a block-level parser that captures raw panel text spans; the parser
/// layer is responsible for recursively parsing each panel's markdown content.
pub fn marco_tab_block(input: Span) -> IResult<Span, MarcoTabBlock> {
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
    let (input, _) = tag(":::tab")(input)?;

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
    let (input, _) = not_line_ending::<_, nom::error::Error<Span>>(input)?;
    let (mut input, _) = opt(line_ending).parse(input)?;

    let mut items: Vec<MarcoTabItem<'_>> = Vec::new();

    // Track the currently open tab header.
    let mut current_title: Option<String> = None;
    let mut current_content_start_offset: usize = 0;

    // Track fenced code blocks so `@tab`/`:::` inside them don't terminate panels.
    let mut in_fence: Option<(char, usize)> = None;

    loop {
        // End-of-input: invalid (requires explicit closing `:::`)
        if input.fragment().is_empty() {
            return Err(nom::Err::Error(nom::error::Error::new(
                original_input,
                nom::error::ErrorKind::Eof,
            )));
        }

        let line_start_span = input;
        let (after_line, line_span) = not_line_ending::<_, nom::error::Error<Span>>(input)?;
        let line = *line_span.fragment();

        // Helper: count leading spaces up to 3 and return (indent_len, rest_str)
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
                    // Start a fenced code block.
                    in_fence = Some((fch, fcount));
                }
                Some((open_ch, open_count)) => {
                    // Potential fence closer.
                    if fch == open_ch && fcount >= open_count && after_fence.trim().is_empty() {
                        in_fence = None;
                    }
                }
            }
        }

        // Closing marker (only when not in a fence)
        if in_fence.is_none() {
            let (_indent_len, rest) = trim_upto_3_spaces(line);
            if let Some(after) = rest.strip_prefix(":::") {
                if after.trim().is_empty() {
                    // Finalize current item if any
                    if let Some(title) = current_title.take() {
                        let content_end_offset = line_start_span.location_offset();
                        let content_span = make_slice_span(
                            original_input,
                            current_content_start_offset,
                            content_end_offset,
                        );
                        items.push(MarcoTabItem {
                            title,
                            content: content_span,
                        });
                    }

                    if items.is_empty() {
                        return Err(nom::Err::Error(nom::error::Error::new(
                            original_input,
                            nom::error::ErrorKind::Tag,
                        )));
                    }

                    // Consume closing line + optional newline and return
                    let (rest_after_close, _) = opt(line_ending).parse(after_line)?;
                    return Ok((rest_after_close, MarcoTabBlock { items }));
                }
            }
        }

        // `@tab` header (only when not in a fence)
        if in_fence.is_none() {
            let (_indent_len, rest) = trim_upto_3_spaces(line);
            if let Some(after) = rest.strip_prefix("@tab") {
                // Require at least one whitespace after `@tab`.
                let after = after.strip_prefix(' ').or_else(|| after.strip_prefix('\t'));
                let Some(after_ws) = after else {
                    // Not a tab header; treat as content.
                    input = consume_line(after_line)?;
                    continue;
                };

                let title = after_ws.trim();
                if title.is_empty() {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        original_input,
                        nom::error::ErrorKind::Tag,
                    )));
                }

                // Finalize previous tab item if any.
                if let Some(prev_title) = current_title.replace(title.to_string()) {
                    let content_end_offset = line_start_span.location_offset();
                    let content_span = make_slice_span(
                        original_input,
                        current_content_start_offset,
                        content_end_offset,
                    );
                    items.push(MarcoTabItem {
                        title: prev_title,
                        content: content_span,
                    });
                } else {
                    current_title = Some(title.to_string());
                }

                // Content starts after this header line's newline (or end-of-line).
                let after_header = consume_line(after_line)?;
                current_content_start_offset = after_header.location_offset();
                input = after_header;
                continue;
            }
        }

        // Regular line: keep scanning.
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
    fn smoke_test_parses_simple_two_tabs() {
        let input = Span::new(":::tab\n@tab One\nHello\n\n@tab Two\nWorld\n:::\n");
        let res = marco_tab_block(input);
        assert!(res.is_ok());
        let (_rest, block) = res.unwrap();
        assert_eq!(block.items.len(), 2);
        assert_eq!(block.items[0].title, "One");
        assert!(block.items[0].content.fragment().contains("Hello"));
        assert_eq!(block.items[1].title, "Two");
        assert!(block.items[1].content.fragment().contains("World"));
    }

    #[test]
    fn smoke_test_ignores_tab_markers_inside_fenced_code() {
        let input =
            Span::new(":::tab\n@tab One\n```\n@tab Not a header\n```\n\nMore\n@tab Two\nOk\n:::\n");
        let res = marco_tab_block(input);
        assert!(res.is_ok());
        let (_rest, block) = res.unwrap();
        assert_eq!(block.items.len(), 2);
        assert!(block.items[0]
            .content
            .fragment()
            .contains("@tab Not a header"));
    }

    #[test]
    fn smoke_test_requires_closing_marker() {
        let input = Span::new(":::tab\n@tab One\nHello\n");
        assert!(marco_tab_block(input).is_err());
    }
}
