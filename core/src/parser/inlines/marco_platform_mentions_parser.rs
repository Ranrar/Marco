//! Platform mentions parser (Marco extension)
//!
//! Syntax:
//! - `@username[platform]`
//! - `@username[platform](Display Name)`
//!
//! The renderer decides whether a platform is supported and how URLs are built.
//! This parser is intentionally conservative to avoid consuming emails or other
//! @-uses. It only triggers when a `[` platform section follows the username.

use super::shared::{to_parser_span, GrammarSpan};
use crate::parser::ast::{Node, NodeKind};
use nom::IResult;
use nom::Input;

const MAX_USERNAME_LEN: usize = 128;
const MAX_PLATFORM_LEN: usize = 64;
const MAX_DISPLAY_LEN: usize = 256;

/// Parse a platform mention token starting at `@`.
pub fn parse_platform_mention(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let frag = input.fragment();
    if !frag.starts_with('@') {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Must have at least "@a[b]".
    if frag.len() < 5 {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Find the opening '[' while validating username characters.
    let mut open_bracket: Option<usize> = None;
    let mut username_end: usize = 1;

    for (rel_i, ch) in frag[1..].char_indices() {
        let i = 1 + rel_i;

        if ch == '\n' {
            break;
        }

        if ch == '[' {
            open_bracket = Some(i);
            username_end = i;
            break;
        }

        if !is_valid_username_char(ch) {
            // Not a mention token.
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }

        // Username length equals the current byte index because we only accept
        // ASCII username characters here.
        if i > MAX_USERNAME_LEN {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    }

    let Some(open_bracket) = open_bracket else {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    };

    if username_end <= 1 {
        // Empty username.
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    let Some(username) = frag.get(1..open_bracket) else {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    };

    // Parse platform inside [..]
    let platform_start = open_bracket + 1;
    let mut close_bracket: Option<usize> = None;

    for (rel_i, ch) in frag[platform_start..].char_indices() {
        let i = platform_start + rel_i;

        if ch == '\n' {
            break;
        }

        if ch == ']' {
            close_bracket = Some(i);
            break;
        }

        if !is_valid_platform_char(ch) {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }

        if i - platform_start >= MAX_PLATFORM_LEN {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    }

    let Some(close_bracket) = close_bracket else {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    };

    if close_bracket == platform_start {
        // Empty platform.
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    let Some(platform_raw) = frag.get(platform_start..close_bracket) else {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    };

    let platform = platform_raw.to_ascii_lowercase();

    // Optional display name in (...)
    let mut consumed_len = close_bracket + 1;
    let mut display: Option<String> = None;

    if frag[consumed_len..].starts_with('(') {
        let open_paren = consumed_len;
        let display_start = open_paren + 1;

        let mut close_paren: Option<usize> = None;
        for (rel_i, ch) in frag[display_start..].char_indices() {
            let i = display_start + rel_i;
            if ch == '\n' {
                break;
            }
            if ch == ')' {
                close_paren = Some(i);
                break;
            }
            if i - display_start >= MAX_DISPLAY_LEN {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Tag,
                )));
            }
        }

        let Some(close_paren) = close_paren else {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        };

        let Some(display_raw) = frag.get(display_start..close_paren) else {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        };

        let trimmed = display_raw.trim();
        if !trimmed.is_empty() {
            display = Some(trimmed.to_string());
        }

        consumed_len = close_paren + 1;
    }

    let (rest, taken) = input.take_split(consumed_len);

    Ok((
        rest,
        Node {
            kind: NodeKind::PlatformMention {
                username: username.to_string(),
                platform,
                display,
            },
            span: Some(to_parser_span(taken)),
            children: Vec::new(),
        },
    ))
}

/// Find the next offset where a platform mention token begins.
///
/// This is used by the text fallback parser so it can stop before a mention
/// in the middle of a text node.
pub fn find_next_platform_mention_start(text: &str) -> Option<usize> {
    let mut search_from = 0usize;

    while search_from < text.len() {
        while search_from < text.len() && !text.is_char_boundary(search_from) {
            search_from += 1;
        }

        let rel = text[search_from..].find('@')?;
        let start = search_from + rel;

        // Quick reject: must have at least @a[b]
        if start + 4 >= text.len() {
            return None;
        }

        let after_at = text[start + 1..].chars().next()?;

        if !is_valid_username_char(after_at) {
            search_from = start + 1;
            continue;
        }

        // Scan username until '['
        let i = start + 1;
        let mut saw_username = false;
        let mut open_bracket: Option<usize> = None;

        for (rel_i, ch) in text[i..].char_indices() {
            let j = i + rel_i;
            if ch == '\n' {
                break;
            }
            if ch == '[' {
                open_bracket = Some(j);
                break;
            }
            if !is_valid_username_char(ch) {
                open_bracket = None;
                break;
            }
            saw_username = true;
            if j - (start + 1) >= MAX_USERNAME_LEN {
                open_bracket = None;
                break;
            }
        }

        let Some(open_bracket) = open_bracket else {
            search_from = start + 1;
            continue;
        };

        if !saw_username {
            search_from = start + 1;
            continue;
        }

        // Scan platform until ']'
        let platform_start = open_bracket + 1;
        if platform_start >= text.len() {
            search_from = start + 1;
            continue;
        }

        let mut close_bracket: Option<usize> = None;
        let mut saw_platform = false;
        for (rel_i, ch) in text[platform_start..].char_indices() {
            let j = platform_start + rel_i;
            if ch == '\n' {
                break;
            }
            if ch == ']' {
                close_bracket = Some(j);
                break;
            }
            if !is_valid_platform_char(ch) {
                close_bracket = None;
                break;
            }
            saw_platform = true;
            if j - platform_start >= MAX_PLATFORM_LEN {
                close_bracket = None;
                break;
            }
        }

        let Some(close_bracket) = close_bracket else {
            search_from = start + 1;
            continue;
        };

        if !saw_platform {
            search_from = start + 1;
            continue;
        }

        // Optional (Display) must be closed if present
        let after = close_bracket + 1;
        if after < text.len() && text[after..].starts_with('(') {
            let disp_start = after + 1;
            let mut close_paren: Option<usize> = None;
            for (rel_i, ch) in text[disp_start..].char_indices() {
                let j = disp_start + rel_i;
                if ch == '\n' {
                    break;
                }
                if ch == ')' {
                    close_paren = Some(j);
                    break;
                }
                if j - disp_start >= MAX_DISPLAY_LEN {
                    close_paren = None;
                    break;
                }
            }
            if close_paren.is_none() {
                search_from = start + 1;
                continue;
            }
        }

        return Some(start);
    }

    None
}

fn is_valid_username_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.')
}

fn is_valid_platform_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_platform_mention_basic() {
        let input = GrammarSpan::new("@ranrar[github] hi");
        let (rest, node) = parse_platform_mention(input).expect("should parse");
        assert_eq!(*rest.fragment(), " hi");

        match node.kind {
            NodeKind::PlatformMention {
                username,
                platform,
                display,
            } => {
                assert_eq!(username, "ranrar");
                assert_eq!(platform, "github");
                assert!(display.is_none());
            }
            other => panic!("expected PlatformMention, got {other:?}"),
        }
    }

    #[test]
    fn smoke_test_parse_platform_mention_with_display() {
        let input = GrammarSpan::new("@ranrar[github](Kim) hi");
        let (rest, node) = parse_platform_mention(input).expect("should parse");
        assert_eq!(*rest.fragment(), " hi");

        match node.kind {
            NodeKind::PlatformMention {
                username,
                platform,
                display,
            } => {
                assert_eq!(username, "ranrar");
                assert_eq!(platform, "github");
                assert_eq!(display.as_deref(), Some("Kim"));
            }
            other => panic!("expected PlatformMention, got {other:?}"),
        }
    }

    #[test]
    fn smoke_test_find_next_platform_mention_start() {
        let s = "hello @ranrar[github] world";
        assert_eq!(find_next_platform_mention_start(s), Some(6));
    }

    #[test]
    fn smoke_test_find_next_platform_mention_start_rejects_unclosed_display() {
        let s = "a @ranrar[github](Kim world";
        assert_eq!(find_next_platform_mention_start(s), None);
    }
}
