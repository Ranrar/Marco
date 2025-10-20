// Inline-level grammar: emphasis, strong, links, images, code spans, inline HTML

use nom::{
    IResult,
    character::complete::char,
    bytes::complete::{take, take_until, take_while},
    multi::many1_count,
    combinator::recognize,
    Slice,
};
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;

// Backslash escape parser
// Handles backslash followed by any ASCII punctuation character
// Per CommonMark spec: !"#$%&'()*+,-./:;<=>?@[\]^_`{|}~
pub fn backslash_escape(input: Span) -> IResult<Span, char> {
    // Must start with backslash
    let (input, _) = char('\\')(input)?;
    
    // Followed by ASCII punctuation
    let (input, escaped_char) = nom::character::complete::satisfy(|c| {
        matches!(c,
            '!' | '"' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | '-' | '.' | '/' |
            ':' | ';' | '<' | '=' | '>' | '?' | '@' | '[' | '\\' | ']' | '^' | '_' | '`' | '{' | '|' | '}' | '~'
        )
    })(input)?;
    
    Ok((input, escaped_char))
}

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
    
    // Skip the opening delimiter - use slice() to preserve position
    let after_opening = input.slice(1..);
    
    // Find the closing delimiter
    let remaining_str = after_opening.fragment();
    let mut pos = 0;
    
    // Must have at least one character of content
    if remaining_str.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::TakeUntil)));
    }
    
    while pos < remaining_str.len() {
        // Skip over code spans (backtick regions) to give them precedence
        if remaining_str.as_bytes()[pos] == b'`' {
            pos += 1;
            // Find matching closing backtick
            while pos < remaining_str.len() && remaining_str.as_bytes()[pos] != b'`' {
                pos += 1;
            }
            if pos < remaining_str.len() {
                pos += 1; // Skip closing backtick
            }
            continue;
        }
        
        if remaining_str.as_bytes()[pos] == delimiter as u8 {
            // Check if next char is also the delimiter (would make it strong)
            if pos + 1 < remaining_str.len() && remaining_str.as_bytes()[pos + 1] == delimiter as u8 {
                // This is **, not a valid closing for emphasis
                pos += 2;
                continue;
            }
            
            // Found single delimiter - this is our closing
            if pos > 0 {  // Must have content
                // Use slice() to preserve absolute position information
                let content = input.slice(1..1 + pos);
                let remaining = input.slice(1 + pos + 1..);
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
    
    
    // Skip the opening delimiters using take()
    let (after_opening, _) = take(2usize)(input)?;
    let remaining_str = after_opening.fragment();
    
    
    // Must have at least one character of content
    if remaining_str.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::TakeUntil)));
    }
    
    // Find the closing delimiter pair
    let mut pos = 0;
    
    while pos < remaining_str.len() {
        // Skip over code spans (backtick regions) to give them precedence
        if remaining_str.as_bytes()[pos] == b'`' {
            pos += 1;
            // Find matching closing backtick
            while pos < remaining_str.len() && remaining_str.as_bytes()[pos] != b'`' {
                pos += 1;
            }
            if pos < remaining_str.len() {
                pos += 1; // Skip closing backtick
            }
            continue;
        }
        
        // Look for double delimiter
        if pos + 1 < remaining_str.len()
            && remaining_str.as_bytes()[pos] == delimiter as u8
            && remaining_str.as_bytes()[pos + 1] == delimiter as u8
        {
            // Found closing delimiter pair
            if pos > 0 {  // Must have content
                
                // Use take() to extract content and remaining
                let (after_content, content) = take(pos)(after_opening)?;
                let (remaining, _closing) = take(2usize)(after_content)?;
                
                
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
    
    // Find the closing ] - use byte positions for UTF-8 safety
    let bracket_pos = content_str[1..].find(']')
        .ok_or_else(|| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::TakeUntil)))?;
    
    if bracket_pos == 0 {  // Must have link text
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::TakeUntil)));
    }
    
    // Extract link text (bracket_pos is relative to position 1, so add 1 to get absolute position)
    let absolute_bracket_pos = 1 + bracket_pos;
    let link_text_str = &content_str[1..absolute_bracket_pos];
    
    // CRITICAL: Code spans have precedence over links
    // If there's an unclosed backtick in the link text, reject this as a link
    let backtick_count = link_text_str.chars().filter(|&c| c == '`').count();
    if backtick_count % 2 != 0 {
        // Odd number of backticks = unclosed code span
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    
    let link_text = LocatedSpan::new(link_text_str);
    
    // Must be followed by (
    let after_bracket = absolute_bracket_pos + 1; // Position after ']'
    if after_bracket >= content_str.len() || content_str.as_bytes()[after_bracket] != b'(' {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    
    // Find the closing )
    let url_start = after_bracket + 1;
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
        // Validate title_end doesn't exceed url_and_title length
        if title_end > url_and_title.len() {
            // Invalid title range, treat as no title
            (url_and_title.trim(), None)
        } else {
            // URL is before the first quote
            let url_end = url_and_title.rfind(" \"").unwrap_or(url_and_title.len());
            let url_part = url_and_title.get(..url_end)
                .map(|s| s.trim())
                .unwrap_or("");
            let title_part = url_and_title.get(title_start..title_end).unwrap_or("");
            (url_part, Some(LocatedSpan::new(title_part)))
        }
    } else {
        (url_and_title.trim(), None)
    };
    
    let url = LocatedSpan::new(url_str);
    
    // Calculate remaining position safely
    let remaining_pos = url_start + paren_pos + 1;
    let remaining = if remaining_pos < content_str.len() {
        LocatedSpan::new(&content_str[remaining_pos..])
    } else {
        LocatedSpan::new("")
    };
    
    log::debug!("Link parsed: text={:?}, url={:?}, title={:?}", 
                link_text.fragment(), url.fragment(), title_opt.as_ref().map(|s| s.fragment()));
    
    Ok((remaining, (link_text, url, title_opt)))
}

// Image parser (![alt](url) or ![alt](url "title"))
// Returns (alt_text, url, optional_title)
pub fn image(input: Span) -> IResult<Span, (Span, Span, Option<Span>)> {
    log::debug!("Parsing image at: {:?}", input.fragment());
    
    // Must start with ![
    let (input, _) = char('!')(input)?;
    let (input, _) = char('[')(input)?;
    
    // Parse alt text (everything until ])
    let (input, alt_text) = take_until("]")(input)?;
    let (input, _) = char(']')(input)?;
    
    // Must have opening (
    let (input, _) = char('(')(input)?;
    
    // Parse URL (everything until ) or space before title)
    let url_content = input.fragment();
    
    // Find the end - either ) or space (if there's a title)
    let url_end = url_content.find([')', ' ', '"'])
        .unwrap_or(url_content.len());
    
    if url_end == 0 {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::TakeUntil)));
    }
    
    let (input, url) = take::<_, _, nom::error::Error<_>>(url_end)(input)?;
    
    // Optional whitespace before title or closing )
    let (input, _) = take_while(|c| c == ' ')(input)?;
    
    // Optional title in quotes
    let (input, title_opt) = if input.fragment().starts_with('"') {
        let (input, _) = char('"')(input)?;
        let (input, title) = take_until("\"")(input)?;
        let (input, _) = char('"')(input)?;
        let (input, _) = take_while(|c| c == ' ')(input)?;
        (input, Some(title))
    } else {
        (input, None)
    };
    
    // Must have closing )
    let (input, _) = char(')')(input)?;
    
    log::debug!("Image parsed: alt={:?}, url={:?}, title={:?}", 
                alt_text.fragment(), url.fragment(), title_opt.as_ref().map(|s| s.fragment()));
    
    Ok((input, (alt_text, url, title_opt)))
}

// Inline HTML parser
// Matches: <tag>, </tag>, <tag />, <!-- comment -->
pub fn inline_html(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing inline HTML at: {:?}", input.fragment());
    
    let start = input;
    
    // Must start with <
    let (input, _) = char('<')(input)?;
    
    // Check for HTML comment
    if input.fragment().starts_with("!--") {
        let (input, _) = take(3usize)(input)?; // Take "!--"
        let (input, _comment_content) = take_until("-->")(input)?;
        let (input, _) = take(3usize)(input)?; // Take "-->"
        
        // Calculate how much we consumed
        let consumed_len = input.location_offset() - start.location_offset();
        let html_content = &start.fragment()[..consumed_len];
        let html_span = LocatedSpan::new(html_content);
        log::debug!("HTML comment parsed");
        return Ok((input, html_span));
    }
    
    // Check for closing tag
    let (input, is_closing) = if input.fragment().starts_with('/') {
        let (i, _) = char('/')(input)?;
        (i, true)
    } else {
        (input, false)
    };
    
    // Parse tag name (alphanumeric + hyphen)
    let tag_name_fragment = input.fragment();
    let tag_name_len: usize = tag_name_fragment
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '-')
        .map(|c| c.len_utf8())
        .sum();
    
    if tag_name_len == 0 {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Alpha)));
    }
    
    let (mut input, _tag_name) = take(tag_name_len)(input)?;
    
    // For opening tags, consume attributes and whitespace until > or />
    if !is_closing {
        loop {
            let fragment = input.fragment();
            
            if fragment.is_empty() {
                return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
            }
            
            // Check for self-closing tag
            if fragment.starts_with("/>") {
                let (rest, _) = take(2usize)(input)?;
                let consumed_len = rest.location_offset() - start.location_offset();
                let html_content = &start.fragment()[..consumed_len];
                let html_span = LocatedSpan::new(html_content);
                log::debug!("Self-closing HTML tag parsed");
                return Ok((rest, html_span));
            }
            
            // Check for closing >
            if fragment.starts_with('>') {
                let (rest, _) = char('>')(input)?;
                let consumed_len = rest.location_offset() - start.location_offset();
                let html_content = &start.fragment()[..consumed_len];
                let html_span = LocatedSpan::new(html_content);
                log::debug!("HTML tag parsed: {:?}", html_content);
                return Ok((rest, html_span));
            }
            
            // Advance one character
            let (next, _) = take(1usize)(input)?;
            input = next;
        }
    } else {
        // Closing tag must end with >
        let (input, _) = take_while(|c| c == ' ')(input)?;
        let (input, _) = char('>')(input)?;
        
        let consumed_len = input.location_offset() - start.location_offset();
        let html_content = &start.fragment()[..consumed_len];
        let html_span = LocatedSpan::new(html_content);
        log::debug!("HTML closing tag parsed: {:?}", html_content);
        Ok((input, html_span))
    }
}

// Autolink parser (<https://example.com> or <email@example.com>)
// CommonMark spec section 6.7: Autolinks
pub fn autolink(input: Span) -> IResult<Span, (Span, bool)> {
    use nom::character::complete::char as nom_char;
    
    log::debug!("Parsing autolink at: {:?}", crate::logic::logger::safe_preview(input.fragment(), 30));
    
    let start = input;
    
    // Must start with <
    let (input, _) = nom_char('<')(input)?;
    
    // Try to parse as URI autolink first
    if let Ok((input, uri)) = parse_uri_autolink(input) {
        // Must end with >
        let (remaining, _) = nom_char('>')(input)?;
        log::debug!("Parsed URI autolink: {:?}", uri.fragment());
        return Ok((remaining, (uri, false))); // false = not email
    }
    
    // Try to parse as email autolink
    if let Ok((input, email)) = parse_email_autolink(input) {
        // Must end with >
        let (remaining, _) = nom_char('>')(input)?;
        log::debug!("Parsed email autolink: {:?}", email.fragment());
        return Ok((remaining, (email, true))); // true = is email
    }
    
    Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)))
}

// Helper: Parse URI autolink content
fn parse_uri_autolink(input: Span) -> IResult<Span, Span> {
    use nom::bytes::complete::{take_while1, take_while};
    
    // Scheme: sequence of 2-32 alphanumeric/+/./- characters followed by :
    let (input, scheme) = take_while1(|c: char| c.is_ascii_alphanumeric() || c == '+' || c == '.' || c == '-')(input)?;
    
    if scheme.fragment().len() < 2 || scheme.fragment().len() > 32 {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    
    let (input, _) = nom::character::complete::char(':')(input)?;
    
    // Rest of URI: any characters except < > space and control characters
    let (input, rest) = take_while(|c: char| {
        c != '<' && c != '>' && c != ' ' && !c.is_control()
    })(input)?;
    
    // Construct full URI
    let uri_text = format!("{}:{}", scheme.fragment(), rest.fragment());
    let uri_span = Span::new(Box::leak(uri_text.into_boxed_str()));
    
    Ok((input, uri_span))
}

// Helper: Parse email autolink content
fn parse_email_autolink(input: Span) -> IResult<Span, Span> {
    use nom::bytes::complete::take_while1;
    
    let start = input;
    
    // Local part: alphanumeric, ., !, #, $, %, &, ', *, +, /, =, ?, ^, _, `, {, |, }, ~, -
    let (input, local) = take_while1(|c: char| {
        c.is_ascii_alphanumeric() || ".!#$%&'*+/=?^_`{|}~-".contains(c)
    })(input)?;
    
    // @ symbol
    let (input, _) = nom::character::complete::char('@')(input)?;
    
    // Domain: alphanumeric, -, ., with at least one .
    let (input, domain) = take_while1(|c: char| {
        c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_'
    })(input)?;
    
    // Domain must contain at least one .
    if !domain.fragment().contains('.') {
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    // Construct full email
    let email_text = format!("{}@{}", local.fragment(), domain.fragment());
    let email_span = Span::new(Box::leak(email_text.into_boxed_str()));
    
    Ok((input, email_span))
}

// Soft line break parser (newline that doesn't end a paragraph)
// CommonMark spec section 6.15: Soft line breaks
// A regular newline (not in a code span or HTML tag) is parsed as a soft line break
pub fn soft_line_break(input: Span) -> IResult<Span, ()> {
    use nom::character::complete::line_ending;
    
    log::debug!("Parsing soft line break");
    
    // Just a line ending (newline)
    let (input, _) = line_ending(input)?;
    
    Ok((input, ()))
}

// Hard line break parser (two spaces + newline, or backslash + newline)
// CommonMark spec section 6.14: Hard line breaks
pub fn hard_line_break(input: Span) -> IResult<Span, ()> {
    use nom::{branch::alt, bytes::complete::tag, character::complete::line_ending, combinator::recognize};
    
    log::debug!("Parsing hard line break");
    
    // Two or more spaces followed by newline, OR backslash followed by newline
    let (input, _) = alt((
        recognize(nom::sequence::tuple((tag("  "), line_ending))),  // 2+ spaces + newline
        recognize(nom::sequence::tuple((tag("\\"), line_ending))),   // backslash + newline
    ))(input)?;
    
    Ok((input, ()))
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
        let (_remaining, content) = result.unwrap();
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
    
    #[test]
    fn smoke_test_code_span_mismatched_backticks() {
        // Example 347: ```foo`` should NOT match (3 opening, 2 closing)
        let input = Span::new("```foo``");
        let result = code_span(input);
        
        assert!(result.is_err(), "Should NOT match code span with mismatched backtick counts (3 vs 2)");
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
    
    // Image smoke tests
    
    
    #[test]
    fn smoke_test_image_stub() {
        // Image parser should now work
        let input = Span::new("![alt text](image.png)");
        let result = image(input);
        
        assert!(result.is_ok(), "Image parser should succeed");
        let (_, (alt, url, title)) = result.unwrap();
        assert_eq!(alt.fragment(), &"alt text");
        assert_eq!(url.fragment(), &"image.png");
        assert!(title.is_none());
    }
    
    #[test]
    fn smoke_test_image_with_title_stub() {
        // Image with title attribute
        let input = Span::new("![alt](img.jpg \"title\")");
        let result = image(input);
        
        assert!(result.is_ok(), "Image parser should succeed");
        let (_, (alt, url, title)) = result.unwrap();
        assert_eq!(alt.fragment(), &"alt");
        assert_eq!(url.fragment(), &"img.jpg");
        assert!(title.is_some());
        assert_eq!(title.unwrap().fragment(), &"title");
    }
    
    // Inline HTML smoke tests
    
    #[test]
    fn smoke_test_inline_html_stub() {
        // Inline HTML parser should now work
        let input = Span::new("<span>text</span>");
        let result = inline_html(input);
        
        assert!(result.is_ok(), "Inline HTML parser should succeed");
        let (remaining, html) = result.unwrap();
        assert_eq!(html.fragment(), &"<span>");
        assert_eq!(remaining.fragment(), &"text</span>");
    }
    
    #[test]
    fn smoke_test_inline_html_self_closing_stub() {
        // Self-closing HTML tag
        let input = Span::new("<br />");
        let result = inline_html(input);
        
        assert!(result.is_ok(), "Inline HTML parser should succeed");
        let (_, html) = result.unwrap();
        assert_eq!(html.fragment(), &"<br />");
    }
    
    #[test]
    fn smoke_test_inline_html_comment_stub() {
        // HTML comment
        let input = Span::new("<!-- comment -->");
        let result = inline_html(input);
        
        assert!(result.is_ok(), "Inline HTML parser should succeed");
        let (_, html) = result.unwrap();
        assert_eq!(html.fragment(), &"<!-- comment -->");
    }
    
    // Autolink smoke tests
    
    #[test]
    fn smoke_test_autolink_uri_basic() {
        // Example 602: <https://example.com>
        let input = Span::new("<https://example.com>");
        let result = autolink(input);
        
        assert!(result.is_ok(), "Should parse URI autolink");
        let (_, (uri, is_email)) = result.unwrap();
        assert_eq!(uri.fragment(), &"https://example.com");
        assert!(!is_email);
    }
    
    #[test]
    fn smoke_test_autolink_email_basic() {
        // Example 610: <user@example.com>
        let input = Span::new("<user@example.com>");
        let result = autolink(input);
        
        assert!(result.is_ok(), "Should parse email autolink");
        let (_, (email, is_email)) = result.unwrap();
        assert_eq!(email.fragment(), &"user@example.com");
        assert!(is_email);
    }
    
    #[test]
    fn smoke_test_autolink_uri_with_path() {
        // Autolink with path
        let input = Span::new("<http://example.com/path?query=value>");
        let result = autolink(input);
        
        assert!(result.is_ok(), "Should parse URI with path");
        let (_, (uri, is_email)) = result.unwrap();
        assert!(uri.fragment().starts_with("http://example.com/path"));
        assert!(!is_email);
    }
    
    #[test]
    fn smoke_test_autolink_invalid_no_brackets() {
        // Without < >, it's not an autolink
        let input = Span::new("https://example.com");
        let result = autolink(input);
        
        assert!(result.is_err(), "Should fail without brackets");
    }
    
    // Soft line break smoke tests
    
    #[test]
    fn smoke_test_soft_line_break_basic() {
        // Single newline is a soft line break
        let input = Span::new("\ntext");
        let result = soft_line_break(input);
        
        assert!(result.is_ok(), "Should parse soft line break");
        let (remaining, _) = result.unwrap();
        assert_eq!(remaining.fragment(), &"text");
    }
    
    // Hard line break smoke tests
    
    #[test]
    fn smoke_test_hard_line_break_two_spaces() {
        // Two spaces + newline = hard line break
        let input = Span::new("  \ntext");
        let result = hard_line_break(input);
        
        assert!(result.is_ok(), "Should parse hard line break with spaces");
        let (remaining, _) = result.unwrap();
        assert_eq!(remaining.fragment(), &"text");
    }
    
    #[test]
    fn smoke_test_hard_line_break_backslash() {
        // Backslash + newline = hard line break
        let input = Span::new("\\\ntext");
        let result = hard_line_break(input);
        
        assert!(result.is_ok(), "Should parse hard line break with backslash");
        let (remaining, _) = result.unwrap();
        assert_eq!(remaining.fragment(), &"text");
    }
}


