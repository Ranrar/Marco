//! Emoji shortcode parser (Markdown Guide extended syntax; Marco extension)
//!
//! Syntax: `:shortcode:`
//!
//! Notes:
//! - Only *recognized* shortcodes are converted to emoji. Unknown shortcodes are
//!   left as literal text (we return an error so the fallback text parser wins).
//! - Code spans are parsed before this, so ```:joy:``` inside backticks remains
//!   code and is not converted.

use super::shared::{to_parser_span, GrammarSpan};
use crate::parser::ast::{Node, NodeKind};
use nom::IResult;
use nom::Input;

const MAX_SHORTCODE_LEN: usize = 64;

/// Parse a recognized emoji shortcode of the form `:shortcode:`.
pub fn parse_emoji_shortcode(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let frag = input.fragment();
    if !frag.starts_with(':') {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Must have at least ":a:".
    if frag.len() < 3 {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Find the closing ':' on the same line and within a reasonable distance.
    let tail = &frag[1..];
    let close_rel = match tail.find(':') {
        Some(idx) => idx,
        None => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )))
        }
    };

    let close = 1 + close_rel;

    if let Some(nl) = frag.find('\n') {
        if nl < close {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    }

    let shortcode = &frag[1..close];
    if shortcode.is_empty() || shortcode.len() > MAX_SHORTCODE_LEN {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    if !is_valid_shortcode(shortcode) {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    let Some(emoji) = lookup_shortcode(shortcode) else {
        // Unknown shortcode: let it fall back to literal text.
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    };

    let consumed_len = close + 1;
    let (rest, taken) = input.take_split(consumed_len);

    Ok((
        rest,
        Node {
            kind: NodeKind::Text(emoji.to_string()),
            span: Some(to_parser_span(taken)),
            children: Vec::new(),
        },
    ))
}

/// Find the next offset where a *recognized* emoji shortcode starts.
///
/// This is used by the text fallback parser so it can stop before a shortcode
/// in the middle of a text node.
pub fn find_next_emoji_shortcode_start(text: &str) -> Option<usize> {
    let mut search_from = 0usize;

    while search_from < text.len() {
        let rel = text[search_from..].find(':')?;
        let start = search_from + rel;

        // Need at least ":a:" remaining.
        if start + 2 >= text.len() {
            return None;
        }

        // We only consider short candidates within a small max window.
        let window_end = (start + 1 + MAX_SHORTCODE_LEN + 1).min(text.len());
        let window = &text[start + 1..window_end];

        if let Some(close_rel) = window.find(':') {
            let close = start + 1 + close_rel;

            // Reject newlines inside the candidate.
            if let Some(nl) = text[start..=close].find('\n') {
                // Move past the newline to avoid quadratic scanning.
                search_from = start + nl + 1;
                continue;
            }

            let shortcode = &text[start + 1..close];
            if !shortcode.is_empty()
                && shortcode.len() <= MAX_SHORTCODE_LEN
                && is_valid_shortcode(shortcode)
                && lookup_shortcode(shortcode).is_some()
            {
                return Some(start);
            }
        }

        // Continue searching one byte after the ':' we just considered.
        search_from = start + 1;
    }

    None
}

fn is_valid_shortcode(s: &str) -> bool {
    // Keep this intentionally conservative (ASCII-ish) to avoid surprises.
    // GitHub supports a much larger alias set; we can expand later.
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '+' | '-'))
}

fn lookup_shortcode(s: &str) -> Option<&'static str> {
    // Minimal built-in set (common in docs). Expand as needed.
    match s {
        "+1" | "thumbsup" => Some("👍"),
        "-1" | "thumbsdown" => Some("👎"),
        "joy" => Some("😂"),
        "smile" => Some("😄"),
        "grin" => Some("😁"),
        "heart" => Some("❤️"),
        "rocket" => Some("🚀"),
        "warning" => Some("⚠️"),
        "info" => Some("ℹ️"),
        "check" | "white_check_mark" => Some("✅"),
        "x" => Some("❌"),
        "fire" => Some("🔥"),
        "star" => Some("⭐"),
        "eyes" => Some("👀"),
        "tada" => Some("🎉"),
        "clap" => Some("👏"),
        "coffee" => Some("☕"),
        "memo" => Some("📝"),
        "bug" => Some("🐛"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_emoji_shortcode_basic() {
        let input = GrammarSpan::new(":joy: test");
        let (rest, node) = parse_emoji_shortcode(input).expect("should parse");
        assert_eq!(*rest.fragment(), " test");
        match node.kind {
            NodeKind::Text(t) => assert_eq!(t, "😂"),
            other => panic!("expected Text, got {other:?}"),
        }
    }

    #[test]
    fn smoke_test_parse_emoji_shortcode_unknown_is_error() {
        let input = GrammarSpan::new(":not-a-real-one:");
        assert!(parse_emoji_shortcode(input).is_err());
    }

    #[test]
    fn smoke_test_find_next_emoji_shortcode_start() {
        let s = "a :joy: b";
        assert_eq!(find_next_emoji_shortcode_start(s), Some(2));
    }

    #[test]
    fn smoke_test_find_next_emoji_shortcode_start_ignores_unknown() {
        let s = "a :unknown: b :joy: c";
        // Should find the joy, not the unknown.
        assert_eq!(find_next_emoji_shortcode_start(s), Some(14));
    }
}
