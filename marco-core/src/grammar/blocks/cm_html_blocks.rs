// CommonMark HTML Blocks Grammar
// Parses 7 types of HTML blocks per CommonMark spec
//
// HTML Block Types:
// 1. Special tags (script, pre, style, textarea) - consume until closing tag
// 2. Comments (<!-- ... -->)
// 3. Processing instructions (<? ... ?>)
// 4. Declarations (<!DOCTYPE ...>)
// 5. CDATA (<![CDATA[ ... ]]>)
// 6. Standard block tags (address, article, aside, etc.) - end at blank line
// 7. Complete tags (well-formed open/close tag on single line) - end at blank line

use crate::grammar::shared::Span;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    character::complete::line_ending,
    combinator::opt,
    IResult, Input, Parser,
};

// HTML Block Type 2: Comments
// Parses <!-- ... --> on its own line(s)
pub fn html_comment(input: Span) -> IResult<Span, Span> {
    log::debug!(
        "Trying HTML comment at: {:?}",
        crate::logic::logger::safe_preview(input.fragment(), 40)
    );

    let start = input;

    // Optional leading spaces (0-3 allowed, just like other block elements)
    let (input, leading_spaces) = take_while(|c| c == ' ').parse(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(
            start,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Parse <!-- ... -->
    let (input, (_, _content, _)) = (tag("<!--"), take_until("-->"), tag("-->")).parse(input)?;

    // Must be followed by newline or EOF
    let (input, _) = opt(line_ending).parse(input)?;

    // Return the whole comment including markers
    let consumed_len = input.location_offset() - start.location_offset();
    let comment_span = start.take(consumed_len);

    log::debug!(
        "Parsed HTML comment: {:?}",
        crate::logic::logger::safe_preview(comment_span.fragment(), 40)
    );
    Ok((input, comment_span))
}

// HTML Block Type 3: Processing Instructions
// Parses <?...?>
pub fn html_processing_instruction(input: Span) -> IResult<Span, Span> {
    log::debug!(
        "Trying processing instruction at: {:?}",
        crate::logic::logger::safe_preview(input.fragment(), 40)
    );

    let start = input;

    // Optional leading spaces (0-3)
    let (input, leading_spaces) = take_while(|c| c == ' ').parse(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(
            start,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Must start with <?
    let (input, _) = tag("<?").parse(input)?;

    // Consume until ?>
    let (input, _content) = take_until("?>").parse(input)?;
    let (input, _) = tag("?>").parse(input)?;

    // Must be followed by newline or EOF
    let (input, _) = opt(line_ending).parse(input)?;

    let consumed_len = input.location_offset() - start.location_offset();
    let pi_span = start.take(consumed_len);

    log::debug!(
        "Parsed processing instruction: {:?}",
        crate::logic::logger::safe_preview(pi_span.fragment(), 40)
    );
    Ok((input, pi_span))
}

// HTML Block Type 4: Declarations
// Parses <!X...> where X is ASCII letter
pub fn html_declaration(input: Span) -> IResult<Span, Span> {
    log::debug!(
        "Trying HTML declaration at: {:?}",
        crate::logic::logger::safe_preview(input.fragment(), 40)
    );

    let start = input;

    // Optional leading spaces (0-3)
    let (input, leading_spaces) = take_while(|c| c == ' ').parse(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(
            start,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Must start with <! followed by ASCII letter
    let (input, _) = tag("<!").parse(input)?;

    // Next character must be ASCII letter
    let bytes = input.fragment().as_bytes();
    if bytes.is_empty() || !bytes[0].is_ascii_alphabetic() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Alpha,
        )));
    }

    // Consume until >
    let (input, _content) = take_until(">").parse(input)?;
    let (input, _) = tag(">").parse(input)?;

    // Must be followed by newline or EOF
    let (input, _) = opt(line_ending).parse(input)?;

    let consumed_len = input.location_offset() - start.location_offset();
    let decl_span = start.take(consumed_len);

    log::debug!(
        "Parsed HTML declaration: {:?}",
        crate::logic::logger::safe_preview(decl_span.fragment(), 40)
    );
    Ok((input, decl_span))
}

// HTML Block Type 5: CDATA Sections
// Parses <![CDATA[...]]>
pub fn html_cdata(input: Span) -> IResult<Span, Span> {
    log::debug!(
        "Trying CDATA section at: {:?}",
        crate::logic::logger::safe_preview(input.fragment(), 40)
    );

    let start = input;

    // Optional leading spaces (0-3)
    let (input, leading_spaces) = take_while(|c| c == ' ').parse(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(
            start,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Must start with <![CDATA[
    let (input, _) = tag("<![CDATA[").parse(input)?;

    // Consume until ]]>
    let (input, _content) = take_until("]]>").parse(input)?;
    let (input, _) = tag("]]>").parse(input)?;

    // Must be followed by newline or EOF
    let (input, _) = opt(line_ending).parse(input)?;

    let consumed_len = input.location_offset() - start.location_offset();
    let cdata_span = start.take(consumed_len);

    log::debug!(
        "Parsed CDATA section: {:?}",
        crate::logic::logger::safe_preview(cdata_span.fragment(), 40)
    );
    Ok((input, cdata_span))
}

// HTML Block Type 1: Special Raw Content Tags (script, pre, style, textarea)
// These consume content until closing tag, can contain blank lines
pub fn html_special_tag(input: Span) -> IResult<Span, Span> {
    log::debug!(
        "Trying special HTML tag at: {:?}",
        crate::logic::logger::safe_preview(input.fragment(), 40)
    );

    let start = input;

    // Optional leading spaces (0-3)
    let (input, leading_spaces) = take_while(|c| c == ' ').parse(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(
            start,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Try to parse opening tag: <pre, <script, <style, <textarea (case-insensitive)
    let lower_input = input.fragment().to_lowercase();
    let tag_name = if lower_input.starts_with("<script") {
        "script"
    } else if lower_input.starts_with("<pre") {
        "pre"
    } else if lower_input.starts_with("<style") {
        "style"
    } else if lower_input.starts_with("<textarea") {
        "textarea"
    } else {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    };

    // Check that after tag name there's space, tab, >, or EOL
    let tag_len = tag_name.len() + 1; // +1 for '<'
    if input.fragment().len() > tag_len {
        let next_char = input.fragment().chars().nth(tag_len);
        match next_char {
            Some(' ') | Some('\t') | Some('>') | Some('\n') | Some('\r') => {}
            Some(_) => {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Tag,
                )))
            }
            None => {} // EOF is OK
        }
    }

    // Build closing tag pattern (case-insensitive)
    let closing_tag = format!("</{}>", tag_name);

    // Consume until we find the closing tag
    let mut remaining = input;

    while !remaining.fragment().is_empty() {
        // Check if current position contains closing tag (case-insensitive)
        if remaining.fragment().to_lowercase().contains(&closing_tag) {
            // Find exact position of closing tag
            if let Some(pos) = remaining.fragment().to_lowercase().find(&closing_tag) {
                // Advance to after the closing tag
                let bytes_to_consume = pos + closing_tag.len();
                remaining = remaining.take_from(bytes_to_consume);
                break;
            }
        }

        // Advance to next line
        if let Some(newline_pos) = remaining.fragment().find('\n') {
            remaining = remaining.take_from(newline_pos + 1);
        } else {
            // No more newlines, consume rest
            remaining = remaining.take_from(remaining.fragment().len());
            break;
        }
    }

    // Return the entire block (from start to after closing tag or EOF)
    let consumed_len = remaining.location_offset() - start.location_offset();
    let block_span = start.take(consumed_len);

    log::debug!(
        "Parsed special HTML tag ({}): {:?}",
        tag_name,
        crate::logic::logger::safe_preview(block_span.fragment(), 40)
    );
    Ok((remaining, block_span))
}

// HTML Block Type 6: Standard Block Tags
// CommonMark spec lists these specific tags
const BLOCK_TAGS: &[&str] = &[
    "address",
    "article",
    "aside",
    "base",
    "basefont",
    "blockquote",
    "body",
    "caption",
    "center",
    "col",
    "colgroup",
    "dd",
    "details",
    "dialog",
    "dir",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "frame",
    "frameset",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hr",
    "html",
    "iframe",
    "legend",
    "li",
    "link",
    "main",
    "menu",
    "menuitem",
    "nav",
    "noframes",
    "ol",
    "optgroup",
    "option",
    "p",
    "param",
    "search",
    "section",
    "summary",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "title",
    "tr",
    "track",
    "ul",
];

pub fn html_block_tag(input: Span) -> IResult<Span, Span> {
    log::debug!(
        "Trying block HTML tag at: {:?}",
        crate::logic::logger::safe_preview(input.fragment(), 40)
    );

    let start = input;

    // Optional leading spaces (0-3)
    let (input, leading_spaces) = take_while(|c| c == ' ').parse(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(
            start,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Must start with < or </
    let (input, _opening) = alt((tag("</"), tag("<"))).parse(input)?;

    // Try to match one of the block tag names (case-insensitive)
    let lower_input = input.fragment().to_lowercase();
    let mut matched_tag: Option<&str> = None;

    for tag_name in BLOCK_TAGS {
        if lower_input.starts_with(tag_name) {
            // Check what follows the tag name
            let tag_len = tag_name.len();
            if lower_input.len() == tag_len {
                // Tag name at EOF is valid
                matched_tag = Some(tag_name);
                break;
            }

            let next_char = lower_input.chars().nth(tag_len);
            match next_char {
                // Valid: space, tab, >, newline, or / followed by >
                Some(' ') | Some('\t') | Some('>') | Some('\n') | Some('\r') => {
                    matched_tag = Some(tag_name);
                    break;
                }
                Some('/') => {
                    // Check if followed by >
                    if lower_input.len() > tag_len + 1
                        && lower_input.chars().nth(tag_len + 1) == Some('>')
                    {
                        matched_tag = Some(tag_name);
                        break;
                    }
                }
                _ => continue, // Not a match, try next tag
            }
        }
    }

    let tag_name = matched_tag.ok_or_else(|| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))
    })?;

    // User-friendly deviation from strict CommonMark:
    // If the opening line contains a matching closing tag for the same element,
    // treat this as a single-line HTML block (only that line), instead of a
    // multi-line HTML block that runs until the next blank line.
    //
    // Why:
    // - In editor UX, a single-line `<div>...</div>` is typically expected to not
    //   swallow subsequent Markdown lines into a raw HTML block.
    // - Keeping it as a block (not inline) avoids wrapping block-level tags
    //   inside `<p>` which would be invalid HTML.
    let first_line = if let Some(newline_pos) = start.fragment().find('\n') {
        &start.fragment()[..newline_pos]
    } else {
        start.fragment()
    };
    let first_line_lower = first_line.to_lowercase();
    let closing_tag = format!("</{}>", tag_name);
    let closes_on_same_line = first_line_lower.contains(&closing_tag);

    // Consume rest of current line
    let mut remaining = input;
    if let Some(newline_pos) = remaining.fragment().find('\n') {
        remaining = remaining.take_from(newline_pos + 1);
    } else {
        // No newline, consume rest
        remaining = remaining.take_from(remaining.fragment().len());
    }

    if closes_on_same_line {
        let consumed_len = remaining.location_offset() - start.location_offset();
        let block_span = start.take(consumed_len);

        log::debug!(
            "Parsed single-line block HTML tag ({}): {:?}",
            tag_name,
            crate::logic::logger::safe_preview(block_span.fragment(), 40)
        );
        return Ok((remaining, block_span));
    }

    // Type 6 blocks end at next blank line
    // Consume lines until blank line or EOF
    while !remaining.fragment().is_empty() {
        // Check if this line is blank
        let line_content = if let Some(newline_pos) = remaining.fragment().find('\n') {
            &remaining.fragment()[..newline_pos]
        } else {
            remaining.fragment()
        };

        // If line is blank (only whitespace), end here
        if line_content.trim().is_empty() {
            break;
        }

        // Not blank, consume this line
        if let Some(newline_pos) = remaining.fragment().find('\n') {
            remaining = remaining.take_from(newline_pos + 1);
        } else {
            // No more newlines, consume rest
            remaining = remaining.take_from(remaining.fragment().len());
            break;
        }
    }

    // Return block from start to current position (before blank line)
    let consumed_len = remaining.location_offset() - start.location_offset();
    let block_span = start.take(consumed_len);

    log::debug!(
        "Parsed block HTML tag ({}): {:?}",
        tag_name,
        crate::logic::logger::safe_preview(block_span.fragment(), 40)
    );
    Ok((remaining, block_span))
}

// HTML Block Type 7: Complete Tags
// Must be a complete open or closing tag on a line by itself (followed only by spaces/tabs)
// Cannot interrupt paragraphs (must be handled specially by caller)
// IMPORTANT: This must validate that the tag is well-formed per CommonMark spec
pub fn html_complete_tag(input: Span) -> IResult<Span, Span> {
    log::debug!(
        "Trying complete HTML tag at: {:?}",
        crate::logic::logger::safe_preview(input.fragment(), 40)
    );

    let start = input;

    // Optional leading spaces (0-3)
    let (input, leading_spaces) = take_while(|c| c == ' ').parse(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(
            start,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Try to parse complete tag (open or closing)
    let line_content = if let Some(newline_pos) = input.fragment().find('\n') {
        &input.fragment()[..newline_pos]
    } else {
        input.fragment()
    };

    // Must start with < and contain >
    if !line_content.starts_with('<') || !line_content.contains('>') {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Find the > position
    let gt_pos = line_content.find('>').unwrap();

    // After >, rest of line must be only spaces/tabs
    let after_tag = &line_content[(gt_pos + 1)..];
    if !after_tag.chars().all(|c| c == ' ' || c == '\t') {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Check if it's a closing tag or opening tag
    let tag_content = &line_content[..=gt_pos];
    let is_closing = tag_content.starts_with("</");

    if is_closing {
        // Closing tag: </tagname>
        if !tag_content.starts_with("</") || tag_content.contains(' ') || tag_content.contains('\t')
        {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }

        // Extract tag name (between </ and >)
        let tag_name = &tag_content[2..(tag_content.len() - 1)];

        // Tag name must start with ASCII letter
        if tag_name.is_empty() || !tag_name.chars().next().unwrap().is_ascii_alphabetic() {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }

        // Rest of tag name must be alphanumeric or hyphen
        if !tag_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    } else {
        // Opening tag: <tagname ...>
        // Exclude special tags (pre, script, style, textarea) - those are Type 1
        let lower_tag = tag_content.to_lowercase();
        if lower_tag.starts_with("<pre")
            || lower_tag.starts_with("<script")
            || lower_tag.starts_with("<style")
            || lower_tag.starts_with("<textarea")
        {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }

        // Extract tag name (from < to first space, /, or >)
        let after_lt = &tag_content[1..];
        let tag_name_end = after_lt
            .find([' ', '\t', '/', '>'])
            .unwrap_or(after_lt.len());
        let tag_name = &after_lt[..tag_name_end];

        // Tag name must start with ASCII letter
        if tag_name.is_empty() || !tag_name.chars().next().unwrap().is_ascii_alphabetic() {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }

        // Rest of tag name must be alphanumeric or hyphen
        if !tag_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }

        // If there are attributes, validate them
        let after_tag_name = &after_lt[tag_name_end..];
        if !after_tag_name.is_empty()
            && !after_tag_name.starts_with('>')
            && !after_tag_name.starts_with("/>")
        {
            let trimmed = after_tag_name.trim_start();
            if trimmed.starts_with('/') && trimmed.len() == 2 && trimmed == "/>" {
                // Self-closing tag, OK
            } else {
                // Validate attribute format
                if after_tag_name.contains("*") || after_tag_name.contains("#") {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Tag,
                    )));
                }

                if !after_tag_name.starts_with(' ') && !after_tag_name.starts_with('\t') {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Tag,
                    )));
                }

                // Check for common malformed patterns
                if after_tag_name.contains("'") {
                    let parts: Vec<&str> = after_tag_name.split('\'').collect();
                    for (i, part) in parts.iter().enumerate().skip(1) {
                        if i % 2 == 0 && !part.is_empty() {
                            let first_char = part.chars().next().unwrap();
                            if first_char.is_alphabetic() {
                                return Err(nom::Err::Error(nom::error::Error::new(
                                    input,
                                    nom::error::ErrorKind::Tag,
                                )));
                            }
                        }
                    }
                }

                if after_tag_name.contains("\\\"") {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Tag,
                    )));
                }

                if !trimmed.starts_with(char::is_alphabetic)
                    && !trimmed.starts_with('/')
                    && !trimmed.starts_with('>')
                {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Tag,
                    )));
                }
            }
        }
    }

    // Consume current line
    let mut remaining = input;
    if let Some(newline_pos) = remaining.fragment().find('\n') {
        remaining = remaining.take_from(newline_pos + 1);
    } else {
        remaining = remaining.take_from(remaining.fragment().len());
    }

    // Type 7 blocks end at next blank line
    while !remaining.fragment().is_empty() {
        let line_content = if let Some(newline_pos) = remaining.fragment().find('\n') {
            &remaining.fragment()[..newline_pos]
        } else {
            remaining.fragment()
        };

        if line_content.trim().is_empty() {
            break;
        }

        if let Some(newline_pos) = remaining.fragment().find('\n') {
            remaining = remaining.take_from(newline_pos + 1);
        } else {
            remaining = remaining.take_from(remaining.fragment().len());
            break;
        }
    }

    // Return block from start to current position (before blank line)
    let consumed_len = remaining.location_offset() - start.location_offset();
    let block_span = start.take(consumed_len);

    log::debug!(
        "Parsed complete HTML tag: {:?}",
        crate::logic::logger::safe_preview(block_span.fragment(), 40)
    );
    Ok((remaining, block_span))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_html_comment() {
        let input = Span::new("<!-- Comment -->");
        let result = html_comment(input);
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert!(content.fragment().contains("Comment"));
    }

    #[test]
    fn smoke_test_html_processing_instruction() {
        let input = Span::new("<?xml version=\"1.0\"?>");
        let result = html_processing_instruction(input);
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert!(content.fragment().contains("xml"));
    }

    #[test]
    fn smoke_test_html_declaration() {
        let input = Span::new("<!DOCTYPE html>");
        let result = html_declaration(input);
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert!(content.fragment().contains("DOCTYPE"));
    }

    #[test]
    fn smoke_test_html_cdata() {
        let input = Span::new("<![CDATA[data here]]>");
        let result = html_cdata(input);
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert!(content.fragment().contains("data here"));
    }

    #[test]
    fn smoke_test_html_special_tag_script() {
        let input = Span::new("<script>\nalert('hi');\n</script>");
        let result = html_special_tag(input);
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert!(content.fragment().contains("alert"));
    }

    #[test]
    fn smoke_test_html_block_tag() {
        let input = Span::new("<div>\nContent\n</div>");
        let result = html_block_tag(input);
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert!(content.fragment().contains("Content"));
    }

    #[test]
    fn smoke_test_html_complete_tag_open() {
        let input = Span::new("<div>");
        let result = html_complete_tag(input);
        assert!(result.is_ok());
    }

    #[test]
    fn smoke_test_html_complete_tag_close() {
        let input = Span::new("</div>");
        let result = html_complete_tag(input);
        assert!(result.is_ok());
    }

    #[test]
    fn smoke_test_html_comment_fails_without_closing() {
        let input = Span::new("<!-- Comment");
        let result = html_comment(input);
        assert!(result.is_err());
    }

    #[test]
    fn smoke_test_html_block_tag_ends_at_blank() {
        let input = Span::new("<div>\nLine 1\n\nAfter blank");
        let result = html_block_tag(input);
        assert!(result.is_ok());
        let (remaining, content) = result.unwrap();
        assert!(content.fragment().contains("Line 1"));
        assert!(remaining.fragment().trim_start().starts_with("After blank"));
    }

    #[test]
    fn smoke_test_html_block_tag_single_line_closed_does_not_swallow_next_line() {
        let input = Span::new("<div>html</div>\n`www.example.com`\n");
        let result = html_block_tag(input);
        assert!(result.is_ok());
        let (remaining, content) = result.unwrap();

        assert!(content.fragment().contains("<div>html</div>"));
        assert!(
            remaining.fragment().starts_with("`www.example.com`"),
            "following line should remain to be parsed as Markdown"
        );
    }
}
