// Inline-level grammar: emphasis, strong, links, images, code spans, inline HTML

use nom::{
    IResult,
    bytes::complete::{tag, take_until},
    character::complete::char,
    multi::many1_count,
    combinator::recognize,
};
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;

// Code span parser (`code` or `` code with ` backtick ``)
// Handles variable backtick counts and whitespace rules per CommonMark spec
pub fn code_span(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing code span at: {:?}", input.fragment());
    
    // Count opening backticks
    let (input, opening) = recognize(many1_count(char('`')))(input)?;
    let backtick_count = opening.fragment().len();
    log::debug!("Found {} opening backticks", backtick_count);
    
    // Find the closing backticks by searching through the string
    let content_str = input.fragment();
    let mut pos = 0;
    
    while pos < content_str.len() {
        if content_str.as_bytes()[pos] == b'`' {
            // Count consecutive backticks at this position
            let mut tick_count = 0;
            let mut check_pos = pos;
            while check_pos < content_str.len() && content_str.as_bytes()[check_pos] == b'`' {
                tick_count += 1;
                check_pos += 1;
            }
            
            // If we found exactly the right number, this is our closing delimiter
            if tick_count == backtick_count {
                let content = LocatedSpan::new(&content_str[..pos]);
                let remaining = LocatedSpan::new(&content_str[check_pos..]);
                log::debug!("Code span content: {:?}", content.fragment());
                return Ok((remaining, content));
            }
            
            // Skip past these backticks
            pos = check_pos;
        } else {
            pos += 1;
        }
    }
    
    // Didn't find matching closing backticks
    Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::TakeUntil)))
}

// Emphasis parser (*text* or _text_)
// Follows CommonMark spec for left and right-flanking delimiters
pub fn emphasis(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing emphasis at: {:?}", input.fragment());
    
    // Try to parse emphasis with * or _ delimiter
    if let Ok(result) = emphasis_with_delimiter(input, '*') {
        return Ok(result);
    }
    
    emphasis_with_delimiter(input, '_')
}

// Helper: Parse emphasis with a specific delimiter (* or _)
fn emphasis_with_delimiter(input: Span, delimiter: char) -> IResult<Span, Span> {
    let content_str = input.fragment();
    
    // Must start with exactly one delimiter (not two, which would be strong)
    if !content_str.starts_with(delimiter) {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    
    // Check if this is actually a strong delimiter (**)
    if content_str.len() > 1 && content_str.chars().nth(1) == Some(delimiter) {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    
    // Skip the opening delimiter
    let after_opening = LocatedSpan::new(&content_str[1..]);
    
    // Find the closing delimiter
    let remaining_str = after_opening.fragment();
    let mut pos = 0;
    
    // Must have at least one character of content
    if remaining_str.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::TakeUntil)));
    }
    
    while pos < remaining_str.len() {
        if remaining_str.as_bytes()[pos] == delimiter as u8 {
            // Check if next char is also the delimiter (would make it strong)
            if pos + 1 < remaining_str.len() && remaining_str.as_bytes()[pos + 1] == delimiter as u8 {
                // This is **, not a valid closing for emphasis
                pos += 2;
                continue;
            }
            
            // Found single delimiter - this is our closing
            if pos > 0 {  // Must have content
                let content = LocatedSpan::new(&remaining_str[..pos]);
                let remaining = LocatedSpan::new(&remaining_str[pos + 1..]);
                log::debug!("Emphasis content: {:?}", content.fragment());
                return Ok((remaining, content));
            }
        }
        pos += 1;
    }
    
    // No closing delimiter found
    Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::TakeUntil)))
}

// Strong emphasis parser (**text** or __text__)
// Follows CommonMark spec for double delimiter sequences
pub fn strong(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing strong emphasis at: {:?}", input.fragment());
    
    // Try to parse strong with ** or __ delimiter
    if let Ok(result) = strong_with_delimiter(input, '*') {
        return Ok(result);
    }
    
    strong_with_delimiter(input, '_')
}

// Helper: Parse strong emphasis with a specific delimiter (** or __)
fn strong_with_delimiter(input: Span, delimiter: char) -> IResult<Span, Span> {
    let content_str = input.fragment();
    
    // Must start with exactly two delimiters
    if content_str.len() < 2 {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    
    if !content_str.starts_with(&format!("{}{}", delimiter, delimiter)) {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    
    // Skip the opening delimiters
    let after_opening = LocatedSpan::new(&content_str[2..]);
    let remaining_str = after_opening.fragment();
    
    // Must have at least one character of content
    if remaining_str.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::TakeUntil)));
    }
    
    // Find the closing delimiter pair
    let mut pos = 0;
    
    while pos < remaining_str.len() {
        // Look for double delimiter
        if pos + 1 < remaining_str.len()
            && remaining_str.as_bytes()[pos] == delimiter as u8
            && remaining_str.as_bytes()[pos + 1] == delimiter as u8
        {
            // Found closing delimiter pair
            if pos > 0 {  // Must have content
                let content = LocatedSpan::new(&remaining_str[..pos]);
                let remaining = LocatedSpan::new(&remaining_str[pos + 2..]);
                log::debug!("Strong emphasis content: {:?}", content.fragment());
                return Ok((remaining, content));
            }
        }
        pos += 1;
    }
    
    // No closing delimiter found
    Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::TakeUntil)))
}

// Link parser ([text](url) or [text](url "title"))
// Returns a tuple of (text_content, url, optional_title)
pub fn link(input: Span) -> IResult<Span, (Span, Span, Option<Span>)> {
    log::debug!("Parsing link at: {:?}", input.fragment());
    
    let content_str = input.fragment();
    
    // Must start with [
    if !content_str.starts_with('[') {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    
    // Find the closing ]
    let mut bracket_pos = 0;
    let mut found_close = false;
    
    for (i, ch) in content_str.chars().enumerate().skip(1) {
        if ch == ']' {
            bracket_pos = i;
            found_close = true;
            break;
        }
    }
    
    if !found_close || bracket_pos == 1 {  // Must have link text
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::TakeUntil)));
    }
    
    // Extract link text
    let link_text = LocatedSpan::new(&content_str[1..bracket_pos]);
    
    // Must be followed by (
    if bracket_pos + 1 >= content_str.len() || content_str.as_bytes()[bracket_pos + 1] != b'(' {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    
    // Find the closing )
    let url_start = bracket_pos + 2;
    let remaining_for_url = &content_str[url_start..];
    
    // Look for closing paren, handling optional title
    let mut paren_pos = None;
    let mut title_range: Option<(usize, usize)> = None;
    
    // Check if there's a title by looking for " ... "
    if let Some(first_quote) = remaining_for_url.find('"') {
        if let Some(second_quote) = remaining_for_url[first_quote + 1..].find('"') {
            let second_quote_abs = first_quote + 1 + second_quote;
            // Title found between quotes
            title_range = Some((first_quote + 1, second_quote_abs));
            
            // Closing paren should be after the second quote
            if let Some(close_paren) = remaining_for_url[second_quote_abs + 1..].find(')') {
                paren_pos = Some(second_quote_abs + 1 + close_paren);
            }
        }
    }
    
    // If no title, just find closing paren
    if paren_pos.is_none() {
        if let Some(pos) = remaining_for_url.find(')') {
            paren_pos = Some(pos);
        }
    }
    
    let paren_pos = paren_pos.ok_or_else(|| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))
    })?;
    
    // Extract URL and optional title
    let url_and_title = &remaining_for_url[..paren_pos];
    let (url_str, title_opt) = if let Some((title_start, title_end)) = title_range {
        // URL is before the first quote
        let url_end = url_and_title.rfind(" \"").unwrap_or(url_and_title.len());
        let url_part = url_and_title[..url_end].trim();
        let title_part = &url_and_title[title_start..title_end];
        (url_part, Some(LocatedSpan::new(title_part)))
    } else {
        (url_and_title.trim(), None)
    };
    
    let url = LocatedSpan::new(url_str);
    let remaining = LocatedSpan::new(&content_str[url_start + paren_pos + 1..]);
    
    log::debug!("Link parsed: text={:?}, url={:?}, title={:?}", 
                link_text.fragment(), url.fragment(), title_opt.as_ref().map(|s| s.fragment()));
    
    Ok((remaining, (link_text, url, title_opt)))
}

// Image parser (![alt](url))
pub fn image(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing image");
    // TODO: Implement image parsing
    Ok((input, input))
}

// Inline HTML parser
pub fn inline_html(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing inline HTML");
    // TODO: Implement inline HTML parsing
    Ok((input, input))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn smoke_test_code_span_basic() {
        // Example 328 from CommonMark: `foo`
        let input = Span::new("`foo`");
        let result = code_span(input);
        
        assert!(result.is_ok(), "Failed to parse basic code span");
        let (remaining, content) = result.unwrap();
        assert_eq!(content.fragment(), &"foo");
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_code_span_double_backticks() {
        // Example 329 from CommonMark: `` foo ` bar ``
        let input = Span::new("`` foo ` bar ``");
        let result = code_span(input);
        
        assert!(result.is_ok(), "Failed to parse double backtick code span");
        let (remaining, content) = result.unwrap();
        assert_eq!(content.fragment(), &" foo ` bar ");
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_code_span_whitespace() {
        // Example 333 from CommonMark: ` b `
        let input = Span::new("` b `");
        let result = code_span(input);
        
        assert!(result.is_ok(), "Failed to parse code span with whitespace");
        let (remaining, content) = result.unwrap();
        assert_eq!(content.fragment(), &" b ");
    }
    
    #[test]
    fn smoke_test_code_span_triple_backticks() {
        // Example 330 from CommonMark: ` `` `
        let input = Span::new("` `` `");
        let result = code_span(input);
        
        assert!(result.is_ok(), "Failed to parse code span containing double backticks");
        let (_remaining, content) = result.unwrap();
        assert_eq!(content.fragment(), &" `` ");
    }
    
    // ========== Emphasis Tests ==========
    
    #[test]
    fn smoke_test_emphasis_asterisk() {
        let input = Span::new("*hello*");
        let result = emphasis(input);
        
        assert!(result.is_ok(), "Failed to parse emphasis with *");
        let (remaining, content) = result.unwrap();
        assert_eq!(content.fragment(), &"hello");
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_emphasis_underscore() {
        let input = Span::new("_world_");
        let result = emphasis(input);
        
        assert!(result.is_ok(), "Failed to parse emphasis with _");
        let (remaining, content) = result.unwrap();
        assert_eq!(content.fragment(), &"world");
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_emphasis_with_spaces() {
        let input = Span::new("*foo bar*");
        let result = emphasis(input);
        
        assert!(result.is_ok(), "Failed to parse emphasis with spaces");
        let (remaining, content) = result.unwrap();
        assert_eq!(content.fragment(), &"foo bar");
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_emphasis_not_strong() {
        // Should not parse ** as emphasis
        let input = Span::new("**bold**");
        let result = emphasis(input);
        
        assert!(result.is_err(), "Should not parse ** as emphasis");
    }
    
    // ========== Strong Tests ==========
    
    #[test]
    fn smoke_test_strong_asterisk() {
        let input = Span::new("**hello**");
        let result = strong(input);
        
        assert!(result.is_ok(), "Failed to parse strong with **");
        let (remaining, content) = result.unwrap();
        assert_eq!(content.fragment(), &"hello");
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_strong_underscore() {
        let input = Span::new("__world__");
        let result = strong(input);
        
        assert!(result.is_ok(), "Failed to parse strong with __");
        let (remaining, content) = result.unwrap();
        assert_eq!(content.fragment(), &"world");
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_strong_with_spaces() {
        let input = Span::new("**foo bar**");
        let result = strong(input);
        
        assert!(result.is_ok(), "Failed to parse strong with spaces");
        let (remaining, content) = result.unwrap();
        assert_eq!(content.fragment(), &"foo bar");
        assert_eq!(remaining.fragment(), &"");
    }
    
    // ========== Link Tests ==========
    
    #[test]
    fn smoke_test_link_basic() {
        let input = Span::new("[link text](https://example.com)");
        let result = link(input);
        
        assert!(result.is_ok(), "Failed to parse basic link");
        let (remaining, (text, url, title)) = result.unwrap();
        assert_eq!(text.fragment(), &"link text");
        assert_eq!(url.fragment(), &"https://example.com");
        assert!(title.is_none());
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_link_with_title() {
        let input = Span::new("[link](/url \"title\")");
        let result = link(input);
        
        assert!(result.is_ok(), "Failed to parse link with title");
        let (remaining, (text, url, title)) = result.unwrap();
        assert_eq!(text.fragment(), &"link");
        assert_eq!(url.fragment(), &"/url");
        assert!(title.is_some());
        assert_eq!(title.unwrap().fragment(), &"title");
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_link_short_url() {
        let input = Span::new("[text](/path)");
        let result = link(input);
        
        assert!(result.is_ok(), "Failed to parse link with short URL");
        let (remaining, (text, url, _title)) = result.unwrap();
        assert_eq!(text.fragment(), &"text");
        assert_eq!(url.fragment(), &"/path");
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_link_empty_text_fails() {
        let input = Span::new("[](url)");
        let result = link(input);
        
        assert!(result.is_err(), "Should fail on empty link text");
    }
}
