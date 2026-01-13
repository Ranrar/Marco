//! Autolink grammar - <url> or <email>
use super::Span;
use nom::{
    bytes::complete::{tag, take_while1},
    IResult,
};

/// Check if a string is a valid URI scheme according to CommonMark spec
/// Scheme: 2-32 chars, starts with ASCII letter, followed by letters/digits/+/.//-
fn is_valid_scheme(s: &str) -> bool {
    if s.len() < 2 || s.len() > 32 {
        return false;
    }

    let mut chars = s.chars();

    // First character must be ASCII letter
    if let Some(first) = chars.next() {
        if !first.is_ascii_alphabetic() {
            return false;
        }
    } else {
        return false;
    }

    // Rest must be letters, digits, +, ., or -
    for ch in chars {
        if !ch.is_ascii_alphanumeric() && ch != '+' && ch != '.' && ch != '-' {
            return false;
        }
    }

    true
}

/// Check if a string looks like an email address
/// Simple check: contains @ and has characters before and after it
fn is_valid_email(s: &str) -> bool {
    if let Some(at_pos) = s.find('@') {
        // Must have at least one char before and after @
        if at_pos > 0 && at_pos < s.len() - 1 {
            // Check for common invalid characters
            return !s.contains(' ') && !s.contains('<') && !s.contains('>');
        }
    }
    false
}

pub fn autolink(input: Span) -> IResult<Span, (Span, bool)> {
    log::debug!("Parsing autolink at: {:?}", input.fragment());
    let (input, _) = tag("<")(input)?;
    let (input, url) = take_while1(|c: char| c != '>')(input)?;
    let (input, _) = tag(">")(input)?;

    let url_str = url.fragment();

    // Check if it's an email
    if is_valid_email(url_str) {
        return Ok((input, (url, true)));
    }

    // Check if it's a valid URI with scheme
    // Must have scheme followed by colon
    if let Some(colon_pos) = url_str.find(':') {
        let scheme = &url_str[..colon_pos];
        if is_valid_scheme(scheme) {
            return Ok((input, (url, false)));
        }
    }

    // Not a valid autolink - fail the parse
    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Tag,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_autolink_valid_url() {
        let input = Span::new("<http://example.com>");
        let result = autolink(input);
        assert!(result.is_ok());
        let (_, (url, is_email)) = result.unwrap();
        assert_eq!(url.fragment(), &"http://example.com");
        assert!(!is_email);
    }

    #[test]
    fn smoke_test_autolink_valid_https() {
        let input = Span::new("<https://example.com>");
        let result = autolink(input);
        assert!(result.is_ok());
    }

    #[test]
    fn smoke_test_autolink_valid_email() {
        let input = Span::new("<user@example.com>");
        let result = autolink(input);
        assert!(result.is_ok());
        let (_, (url, is_email)) = result.unwrap();
        assert_eq!(url.fragment(), &"user@example.com");
        assert!(is_email);
    }

    #[test]
    fn smoke_test_autolink_rejects_html_img() {
        let input = Span::new(r#"<img src="test.png">"#);
        let result = autolink(input);
        assert!(result.is_err(), "Should not parse img tag as autolink");
    }

    #[test]
    fn smoke_test_autolink_rejects_html_span() {
        let input = Span::new("<span>");
        let result = autolink(input);
        assert!(result.is_err(), "Should not parse span tag as autolink");
    }

    #[test]
    fn smoke_test_autolink_rejects_html_div() {
        let input = Span::new("<div>");
        let result = autolink(input);
        assert!(result.is_err(), "Should not parse div tag as autolink");
    }

    #[test]
    fn smoke_test_autolink_rejects_closing_tag() {
        let input = Span::new("</span>");
        let result = autolink(input);
        assert!(result.is_err(), "Should not parse closing tag as autolink");
    }

    #[test]
    fn smoke_test_autolink_valid_custom_scheme() {
        let input = Span::new("<ftp://files.example.com>");
        let result = autolink(input);
        assert!(result.is_ok());
    }

    #[test]
    fn smoke_test_autolink_valid_scheme_with_plus() {
        let input = Span::new("<git+ssh://example.com>");
        let result = autolink(input);
        assert!(result.is_ok());
    }

    #[test]
    fn smoke_test_autolink_rejects_no_colon() {
        let input = Span::new("<notaurl>");
        let result = autolink(input);
        assert!(result.is_err(), "Should reject text without colon");
    }

    #[test]
    fn smoke_test_autolink_rejects_single_char_scheme() {
        let input = Span::new("<x:something>");
        let result = autolink(input);
        assert!(result.is_err(), "Should reject single-char scheme");
    }

    #[test]
    fn smoke_test_autolink_rejects_scheme_starting_with_digit() {
        let input = Span::new("<1http://example.com>");
        let result = autolink(input);
        assert!(result.is_err(), "Should reject scheme starting with digit");
    }

    #[test]
    fn smoke_test_autolink_rejects_invalid_email() {
        let input = Span::new("<not an email>");
        let result = autolink(input);
        assert!(result.is_err(), "Should reject invalid email with spaces");
    }

    #[test]
    fn smoke_test_autolink_valid_complex_url() {
        let input = Span::new("<https://example.com/path?query=value#fragment>");
        let result = autolink(input);
        assert!(result.is_ok());
    }
}
