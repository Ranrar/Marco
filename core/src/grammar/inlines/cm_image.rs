//! Image grammar - ![alt](url "title")
use nom::{IResult};
use nom_locate::LocatedSpan;
use super::Span;

pub fn image(input: Span) -> IResult<Span, (Span, Span, Option<Span>)> {
    log::debug!("Parsing image at: {:?}", input.fragment());
    let content_str = input.fragment();
    if !content_str.starts_with("![") {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    let bracket_pos = content_str[2..].find(']')
        .ok_or_else(|| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::TakeUntil)))?;
    let absolute_bracket_pos = 2 + bracket_pos;
    let alt_text_str = &content_str[2..absolute_bracket_pos];
    let alt_text = LocatedSpan::new(alt_text_str);
    let after_bracket = absolute_bracket_pos + 1;
    if after_bracket >= content_str.len() || content_str.as_bytes()[after_bracket] != b'(' {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    let url_start = after_bracket + 1;
    let remaining_for_url = &content_str[url_start..];
    let mut paren_pos = None;
    let mut title_range: Option<(usize, usize)> = None;
    if let Some(first_quote) = remaining_for_url.find('"') {
        if let Some(second_quote) = remaining_for_url[first_quote + 1..].find('"') {
            let second_quote_abs = first_quote + 1 + second_quote;
            title_range = Some((first_quote + 1, second_quote_abs));
            if let Some(close_paren) = remaining_for_url[second_quote_abs + 1..].find(')') {
                paren_pos = Some(second_quote_abs + 1 + close_paren);
            }
        }
    }
    if paren_pos.is_none() {
        if let Some(pos) = remaining_for_url.find(')') {
            paren_pos = Some(pos);
        }
    }
    let paren_pos = paren_pos.ok_or_else(|| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))
    })?;
    let url_and_title = &remaining_for_url[..paren_pos];
    let (url_str, title_opt) = if let Some((title_start, title_end)) = title_range {
        if title_end > url_and_title.len() {
            (url_and_title.trim(), None)
        } else {
            let url_end = url_and_title.rfind(" \"").unwrap_or(url_and_title.len());
            let url_part = url_and_title.get(..url_end).map(|s| s.trim()).unwrap_or("");
            let title_part = url_and_title.get(title_start..title_end).unwrap_or("");
            (url_part, Some(LocatedSpan::new(title_part)))
        }
    } else {
        (url_and_title.trim(), None)
    };
    let url = LocatedSpan::new(url_str);
    let remaining_pos = url_start + paren_pos + 1;
    let remaining = if remaining_pos < content_str.len() {
        LocatedSpan::new(&content_str[remaining_pos..])
    } else {
        LocatedSpan::new("")
    };
    Ok((remaining, (alt_text, url, title_opt)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn smoke_test_image_basic() {
        let input = Span::new("![alt](url)");
        let result = image(input);
        assert!(result.is_ok());
    }
}
