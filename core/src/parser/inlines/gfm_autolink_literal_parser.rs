//! GFM autolink literals (extension)
//!
//! Implements GitHub Flavored Markdown "Autolinks (extension)" (section 6.9):
//! - `www.` + valid domain (inserts `http://`)
//! - `http://` or `https://` + valid domain
//! - Extended email autolinks (adds `mailto:`)
//! - Extended protocol autolinks: `mailto:` / `xmpp:` + email (+ optional xmpp resource)
//!
//! This is an inline parser (not grammar) because autolink literals are recognized
//! within text nodes and depend on context/path-validation rules.

use super::shared::{to_parser_span, GrammarSpan};
use crate::parser::ast::{Node, NodeKind};
use nom::bytes::complete::take;
use nom::IResult;
use nom::Parser;

#[derive(Debug, Clone)]
struct Match {
    len: usize,   // bytes of label in the source
    href: String, // fully qualified href
}

/// Parse a GFM autolink literal at the current position.
///
/// Note: boundary conditions (start-of-line / after whitespace / after `*_~(`)
/// are handled when splitting text and are not re-validated here.
pub fn parse_gfm_autolink_literal(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let fragment = *input.fragment();

    let Some(m) = match_at_start(fragment) else {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    };

    let (rest, matched_span) = take(m.len).parse(input)?;

    let label = matched_span.fragment().to_string();
    let span = to_parser_span(matched_span);

    let node = Node {
        kind: NodeKind::Link {
            url: m.href,
            title: None,
        },
        span: Some(span),
        children: vec![Node {
            kind: NodeKind::Text(label),
            span: Some(to_parser_span(matched_span)),
            children: Vec::new(),
        }],
    };

    Ok((rest, node))
}

/// Find the next valid autolink literal start in a text fragment.
///
/// This is used by the text fallback parser to stop before link candidates so
/// `parse_gfm_autolink_literal` can run at that position.
pub(crate) fn find_next_autolink_literal_start(text: &str) -> Option<usize> {
    let mut best: Option<usize> = None;

    let mut consider = |idx: usize| {
        best = Some(best.map_or(idx, |b| b.min(idx)));
    };

    // Protocol autolinks (allowed anywhere in text nodes)
    if let Some(idx) = find_first_valid_substring(text, "mailto:", |s| match_mailto(s).is_some()) {
        consider(idx);
    }
    if let Some(idx) = find_first_valid_substring(text, "xmpp:", |s| match_xmpp(s).is_some()) {
        consider(idx);
    }

    // URL / WWW autolinks (require boundary conditions)
    if let Some(idx) = find_first_valid_substring(text, "https://", |s| match_url(s).is_some()) {
        if boundary_ok(text, idx) {
            consider(idx);
        }
    }
    if let Some(idx) = find_first_valid_substring(text, "http://", |s| match_url(s).is_some()) {
        if boundary_ok(text, idx) {
            consider(idx);
        }
    }
    if let Some(idx) = find_first_valid_substring(text, "www.", |s| match_www(s).is_some()) {
        if boundary_ok(text, idx) {
            consider(idx);
        }
    }

    // Emails (allowed anywhere in text nodes)
    if let Some(idx) = find_first_valid_email_start(text) {
        consider(idx);
    }

    best
}

fn match_at_start(s: &str) -> Option<Match> {
    // Prefer protocol forms over raw emails.
    if let Some(m) = match_mailto(s) {
        return Some(m);
    }
    if let Some(m) = match_xmpp(s) {
        return Some(m);
    }
    if let Some(m) = match_url(s) {
        return Some(m);
    }
    if let Some(m) = match_www(s) {
        return Some(m);
    }
    if let Some(m) = match_email(s) {
        return Some(m);
    }
    None
}

fn match_www(s: &str) -> Option<Match> {
    if !s.starts_with("www.") {
        return None;
    }

    let after = &s[4..];
    let domain_len = parse_domain(after, DomainRules::WwwOrHttp)?;

    let max_end = scan_nonspace_non_lt(after, domain_len);
    let candidate = &s[..(4 + max_end)];
    let final_len = apply_extended_path_validation_len(candidate);

    let label = &s[..final_len];
    Some(Match {
        len: label.len(),
        href: format!("http://{}", label),
    })
}

fn match_url(s: &str) -> Option<Match> {
    let (scheme_len, rest) = if let Some(r) = s.strip_prefix("http://") {
        ("http://".len(), r)
    } else if let Some(r) = s.strip_prefix("https://") {
        ("https://".len(), r)
    } else {
        return None;
    };

    let domain_len = parse_domain(rest, DomainRules::WwwOrHttp)?;
    let max_end = scan_nonspace_non_lt(rest, domain_len);
    let candidate = &s[..(scheme_len + max_end)];
    let final_len = apply_extended_path_validation_len(candidate);

    let label = &s[..final_len];
    Some(Match {
        len: label.len(),
        href: label.to_string(),
    })
}

fn match_email(s: &str) -> Option<Match> {
    let email_len = parse_extended_email(s)?;
    let label = &s[..email_len];
    Some(Match {
        len: label.len(),
        href: format!("mailto:{}", label),
    })
}

fn match_mailto(s: &str) -> Option<Match> {
    let after = s.strip_prefix("mailto:")?;
    let email_len = parse_extended_email(after)?;

    let full_len = "mailto:".len() + email_len;
    let label = &s[..full_len];
    Some(Match {
        len: label.len(),
        href: label.to_string(),
    })
}

fn match_xmpp(s: &str) -> Option<Match> {
    let after = s.strip_prefix("xmpp:")?;
    let email_len = parse_extended_email(after)?;

    let mut full_len = "xmpp:".len() + email_len;

    // Optional resource, introduced by a single `/`.
    let remainder = &s[full_len..];
    if let Some(rest) = remainder.strip_prefix('/') {
        let resource_len = rest
            .char_indices()
            .take_while(|&(_, c)| c.is_ascii_alphanumeric() || c == '@' || c == '.')
            .last()
            .map(|(idx, c)| idx + c.len_utf8())
            .unwrap_or(0);

        if resource_len > 0 {
            full_len += 1 + resource_len;
        }
    }

    let label = &s[..full_len];
    Some(Match {
        len: label.len(),
        href: label.to_string(),
    })
}

fn boundary_ok(text: &str, idx: usize) -> bool {
    if idx == 0 {
        return true;
    }

    let prev = text[..idx].chars().next_back();
    match prev {
        Some(c) if c.is_whitespace() => true,
        Some('*' | '_' | '~' | '(') => true,
        _ => false,
    }
}

fn find_first_valid_substring<F>(text: &str, needle: &str, mut validate: F) -> Option<usize>
where
    F: FnMut(&str) -> bool,
{
    let mut start = 0;
    while start < text.len() {
        let rel = text[start..].find(needle)?;
        let idx = start + rel;
        if validate(&text[idx..]) {
            return Some(idx);
        }
        start = idx + needle.len();
    }
    None
}

fn find_first_valid_email_start(text: &str) -> Option<usize> {
    let mut search_start = 0;
    while search_start < text.len() {
        let rel_at = text[search_start..].find('@')?;
        let at_idx = search_start + rel_at;

        // Find start of local part (scan backward).
        let local_start = text[..at_idx]
            .char_indices()
            .rev()
            .take_while(|&(_, c)| is_email_local_char(c))
            .last()
            .map(|(idx, _)| idx)
            .unwrap_or(at_idx);

        if local_start < at_idx {
            if let Some(email_len) = parse_extended_email(&text[local_start..]) {
                // Ensure the match we found actually includes this '@'.
                if local_start + email_len > at_idx {
                    return Some(local_start);
                }
            }
        }

        search_start = at_idx + 1;
    }

    None
}

fn is_email_local_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+')
}

fn is_email_domain_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '_')
}

fn parse_extended_email(s: &str) -> Option<usize> {
    // Local part: one or more of alnum or . - _ +
    let mut idx = 0;
    let mut saw_local = false;
    for (i, c) in s.char_indices() {
        if is_email_local_char(c) {
            idx = i + c.len_utf8();
            saw_local = true;
            continue;
        }
        if c == '@' {
            idx = i;
            break;
        }
        return None;
    }

    if !saw_local {
        return None;
    }

    let rest = s.get(idx..)?;
    let rest = rest.strip_prefix('@')?;

    let domain_len = parse_email_domain(rest)?;

    Some(idx + 1 + domain_len)
}

fn parse_email_domain(s: &str) -> Option<usize> {
    let mut idx = 0;

    // First segment
    let (seg_len, seg_has_underscore, seg_last_char) =
        parse_domain_segment(s, DomainSegmentRules::Email)?;
    idx += seg_len;

    let mut segments = 1usize;
    let mut last_seg_last_char = seg_last_char;
    let mut _unused = seg_has_underscore;

    while s[idx..].starts_with('.') {
        let after_dot = &s[(idx + 1)..];

        // Treat '.' as a segment separator only when followed by a valid segment.
        // This allows trailing punctuation like "foo@bar.example." to end the
        // domain cleanly.
        if !can_start_domain_segment(after_dot, DomainSegmentRules::Email) {
            break;
        }
        let (seg_len, seg_has_underscore, seg_last_char) =
            parse_domain_segment(after_dot, DomainSegmentRules::Email)?;
        idx += 1 + seg_len;
        segments += 1;
        last_seg_last_char = seg_last_char;
        _unused = seg_has_underscore;
    }

    if segments < 2 {
        return None;
    }

    // Last character must not be '-' or '_'.
    if matches!(last_seg_last_char, '-' | '_') {
        return None;
    }

    Some(idx)
}

#[derive(Copy, Clone)]
enum DomainRules {
    WwwOrHttp,
}

#[derive(Copy, Clone)]
enum DomainSegmentRules {
    Email,
    WwwOrHttp,
}

fn can_start_domain_segment(s: &str, rules: DomainSegmentRules) -> bool {
    let Some(c) = s.chars().next() else {
        return false;
    };

    match rules {
        DomainSegmentRules::Email => is_email_domain_char(c),
        DomainSegmentRules::WwwOrHttp => c.is_ascii_alphanumeric() || matches!(c, '-' | '_'),
    }
}

fn parse_domain(s: &str, rules: DomainRules) -> Option<usize> {
    let mut idx = 0;

    let (seg_len, seg_has_underscore, _last_char) =
        parse_domain_segment(s, DomainSegmentRules::WwwOrHttp)?;
    idx += seg_len;

    let mut segments = 1usize;

    // Track underscores in the last two segments.
    let mut prev_has_underscore: Option<bool> = None;
    let mut last_has_underscore: bool = seg_has_underscore;

    while s[idx..].starts_with('.') {
        let after_dot = &s[(idx + 1)..];

        // Treat '.' as a segment separator only when followed by a valid segment.
        // This prevents trailing punctuation (e.g. "www.example.org.") from
        // invalidating the domain parse.
        if !can_start_domain_segment(after_dot, DomainSegmentRules::WwwOrHttp) {
            break;
        }
        let (seg_len, seg_has_underscore, _last_char) =
            parse_domain_segment(after_dot, DomainSegmentRules::WwwOrHttp)?;
        idx += 1 + seg_len;
        segments += 1;

        prev_has_underscore = Some(last_has_underscore);
        last_has_underscore = seg_has_underscore;
    }

    if segments < 2 {
        return None;
    }

    match rules {
        DomainRules::WwwOrHttp => {
            // No underscores in the last two segments.
            if last_has_underscore {
                return None;
            }
            if let Some(prev) = prev_has_underscore {
                if prev {
                    return None;
                }
            }
        }
    }

    Some(idx)
}

fn parse_domain_segment(s: &str, rules: DomainSegmentRules) -> Option<(usize, bool, char)> {
    let mut len = 0usize;
    let mut has_underscore = false;
    let mut last_char: Option<char> = None;

    for (i, c) in s.char_indices() {
        let ok = match rules {
            DomainSegmentRules::Email => is_email_domain_char(c),
            DomainSegmentRules::WwwOrHttp => c.is_ascii_alphanumeric() || matches!(c, '-' | '_'),
        };

        if !ok {
            break;
        }

        if c == '_' {
            has_underscore = true;
        }

        len = i + c.len_utf8();
        last_char = Some(c);
    }

    let last_char = last_char?;
    Some((len, has_underscore, last_char))
}

fn scan_nonspace_non_lt(s: &str, start: usize) -> usize {
    // Return end index (relative to `s`) of the longest run where chars are
    // not whitespace and not '<'.
    let mut end = start;
    for (i, c) in s[start..].char_indices() {
        if c.is_whitespace() || c == '<' {
            break;
        }
        end = start + i + c.len_utf8();
    }
    end
}

fn apply_extended_path_validation_len(candidate: &str) -> usize {
    let mut end = candidate.len();

    // 1) Trailing punctuation is excluded.
    while end > 0 {
        let b = candidate.as_bytes()[end - 1];
        let is_punct = matches!(
            b,
            b'?' | b'!' | b'.' | b',' | b':' | b'*' | b'_' | b'~' | b']'
        );
        if is_punct {
            end -= 1;
        } else {
            break;
        }
    }

    // 2) If it ends in ')', trim unmatched trailing ')'.
    if end > 0 && candidate.as_bytes()[end - 1] == b')' {
        let mut open = 0usize;
        let mut close = 0usize;
        for &b in candidate.as_bytes().iter().take(end) {
            if b == b'(' {
                open += 1;
            } else if b == b')' {
                close += 1;
            }
        }

        while end > 0 && candidate.as_bytes()[end - 1] == b')' && close > open {
            end -= 1;
            close -= 1;
        }
    }

    // 3) If it ends in ';' and looks like an entity reference (&[alnum]+;), trim it.
    if end > 0 && candidate.as_bytes()[end - 1] == b';' {
        let mut i = end - 1;
        let mut saw_alnum = false;
        while i > 0 {
            let b = candidate.as_bytes()[i - 1];
            if b.is_ascii_alphanumeric() {
                saw_alnum = true;
                i -= 1;
                continue;
            }
            if b == b'&' {
                if saw_alnum {
                    end = i - 1;
                }
                break;
            }
            break;
        }
    }

    end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_www_trailing_punctuation_trimmed() {
        let s = "www.commonmark.org.";
        let m = match_www(s).expect("should match");
        assert_eq!(m.len, "www.commonmark.org".len());
        assert_eq!(m.href, "http://www.commonmark.org");
    }

    #[test]
    fn smoke_test_email_plus_rules() {
        assert!(match_email("hello+xyz@mail.example").is_some());
        assert!(match_email("hello@mail+xyz.example").is_none());
    }

    #[test]
    fn smoke_test_entity_suffix_trimmed() {
        let s = "www.google.com/search?q=commonmark&hl;";
        let m = match_www(s).expect("should match");
        assert_eq!(m.len, "www.google.com/search?q=commonmark".len());
        assert_eq!(m.href, "http://www.google.com/search?q=commonmark");
    }

    #[test]
    fn smoke_test_xmpp_resource_single_slash() {
        let s = "xmpp:foo@bar.baz/txt/bin";
        let m = match_xmpp(s).expect("should match");
        assert_eq!(m.len, "xmpp:foo@bar.baz/txt".len());
    }

    #[test]
    fn smoke_test_mailto_rejects_invalid_domain_endings() {
        assert!(match_mailto("mailto:a.b-c_d@a.b-").is_none());
        assert!(match_mailto("mailto:a.b-c_d@a.b_").is_none());
    }
}
