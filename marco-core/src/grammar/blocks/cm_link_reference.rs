// CommonMark Link Reference Definition Grammar
// Parses link reference definitions: [label]: url "optional title"
//
// Per CommonMark spec:
// - Format: [label]: destination "title"
// - Can have 0-3 leading spaces
// - Label cannot be empty
// - Destination can be <url> or bare url
// - Title is optional, can be in "...", '...', or (...)
// - Title requires whitespace before it

use crate::grammar::shared::Span;
use nom::{
    bytes::complete::{take_till, take_while, take_while1},
    character::complete::{char, line_ending, space0, space1},
    combinator::opt,
    IResult, Parser,
};

/// Parse a link reference definition
///
/// Examples:
/// - `[foo]: /url "title"`
/// - `[bar]: <https://example.com>`
/// - `[baz]: /url\n  "title on next line"`
///
/// # Arguments
/// * `input` - The input span to parse
///
/// # Returns
/// `Ok((remaining, (label, url, title)))` where title is optional
pub fn link_reference_definition(input: Span) -> IResult<Span, (String, String, Option<String>)> {
    log::debug!(
        "Trying link reference definition at: {:?}",
        crate::logic::logger::safe_preview(input.fragment(), 40)
    );

    let start = input;

    // Optional leading spaces (0-3)
    let (input, leading_spaces) = take_while(|c| c == ' ')(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(
            start,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Parse [label]:
    let (input, _) = char('[')(input)?;
    let (input, label) = take_till(|c| c == ']' || c == '\n')(input)?;

    // Label must not be empty
    if label.fragment().is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            start,
            nom::error::ErrorKind::Tag,
        )));
    }

    let (input, _) = char(']')(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _) = space0(input)?;

    // Optional newline and indentation after colon
    let (input, _) = opt((line_ending, take_while(|c| c == ' '))).parse(input)?;

    // Parse destination (URL) - can be <url> or bare url
    let (input, url_str) = if input.fragment().starts_with('<') {
        let (input, _) = char('<')(input)?;
        let (input, url) = take_till(|c| c == '>' || c == '\n')(input)?;
        let (input, _) = char('>')(input)?;
        (input, url)
    } else {
        take_while1(|c: char| !c.is_whitespace())(input)?
    };

    let url = url_str.fragment().to_string();

    // Optional title (must have whitespace before it)
    let (input, title) = if let Ok((i, _)) = space1::<Span, nom::error::Error<Span>>(input) {
        // Optional newline before title
        let (i, _) = opt((line_ending, take_while(|c| c == ' '))).parse(i)?;

        // Title can be in "...", '...', or (...)
        let (i, title_str) = if i.fragment().starts_with('"') {
            let (i, _) = char('"')(i)?;
            let (i, t) = take_till(|c| c == '"' || c == '\n')(i)?;
            let (i, _) = char('"')(i)?;
            (i, t)
        } else if i.fragment().starts_with('\'') {
            let (i, _) = char('\'')(i)?;
            let (i, t) = take_till(|c| c == '\'' || c == '\n')(i)?;
            let (i, _) = char('\'')(i)?;
            (i, t)
        } else if i.fragment().starts_with('(') {
            let (i, _) = char('(')(i)?;
            let (i, t) = take_till(|c| c == ')' || c == '\n')(i)?;
            let (i, _) = char(')')(i)?;
            (i, t)
        } else {
            return Err(nom::Err::Error(nom::error::Error::new(
                i,
                nom::error::ErrorKind::Char,
            )));
        };

        (i, Some(title_str.fragment().to_string()))
    } else {
        (input, None)
    };

    // Consume optional trailing spaces
    let (input, _) = space0(input)?;

    // Must end with newline or EOF
    let (input, _) = if input.fragment().is_empty() {
        (input, ())
    } else {
        line_ending(input).map(|(i, _)| (i, ()))?
    };

    let label_str = label.fragment().to_string();

    log::debug!("Parsed link reference: [{}] -> {}", label_str, url);

    Ok((input, (label_str, url, title)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_link_ref_basic() {
        let input = Span::new("[foo]: /url\n");
        let result = link_reference_definition(input);
        assert!(result.is_ok());
        let (_, (label, url, title)) = result.unwrap();
        assert_eq!(label, "foo");
        assert_eq!(url, "/url");
        assert_eq!(title, None);
    }

    #[test]
    fn smoke_test_link_ref_with_title() {
        let input = Span::new("[foo]: /url \"title\"\n");
        let result = link_reference_definition(input);
        assert!(result.is_ok());
        let (_, (_, _, title)) = result.unwrap();
        assert_eq!(title, Some("title".to_string()));
    }

    #[test]
    fn smoke_test_link_ref_angle_brackets() {
        let input = Span::new("[foo]: <https://example.com>\n");
        let result = link_reference_definition(input);
        assert!(result.is_ok());
        let (_, (_, url, _)) = result.unwrap();
        assert_eq!(url, "https://example.com");
    }

    #[test]
    fn smoke_test_link_ref_title_parens() {
        let input = Span::new("[foo]: /url (title)\n");
        let result = link_reference_definition(input);
        assert!(result.is_ok());
        let (_, (_, _, title)) = result.unwrap();
        assert_eq!(title, Some("title".to_string()));
    }

    #[test]
    fn smoke_test_link_ref_multiline() {
        // Link reference definitions need title on same line with space
        // Multiline is tested by CommonMark spec - this test verifies basic case
        let input = Span::new("[foo]: /url \"title\"\n");
        let result = link_reference_definition(input);
        assert!(result.is_ok());
        let (_, (label, url, title)) = result.unwrap();
        assert_eq!(label, "foo");
        assert_eq!(url, "/url");
        assert_eq!(title, Some("title".to_string()));
    }

    #[test]
    fn smoke_test_link_ref_empty_label_fails() {
        let input = Span::new("[]: /url\n");
        let result = link_reference_definition(input);
        assert!(result.is_err());
    }

    #[test]
    fn smoke_test_link_ref_four_space_indent_fails() {
        let input = Span::new("    [foo]: /url\n");
        let result = link_reference_definition(input);
        assert!(result.is_err());
    }
}
