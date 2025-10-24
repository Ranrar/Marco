// CommonMark Setext Heading Grammar
// Parses headings with underlines (= for level 1, - for level 2)
//
// Per CommonMark spec:
// - Level 1: underlined with = (at least one =)
// - Level 2: underlined with - (at least one -)
// - Content can be multiple lines
// - Underline must be on line immediately after content
// - Can have 0-3 leading spaces on both content and underline
// - Cannot be a link reference definition
// - All lines must be in same block context (blockquote boundary check)

use crate::grammar::shared::Span;
use nom::{
    IResult,
    bytes::complete::take_while,
    combinator::recognize,
    branch::alt,
    character::complete::line_ending,
    Slice,
};

// Forward declare link_reference_definition from parent module
use crate::grammar::blocks::cm_link_reference::link_reference_definition;

/// Parse a setext heading (underline style)
/// 
/// Examples:
/// - `Heading 1\n====`
/// - `Heading 2\n----`
/// - `Multi-line\nheading\n===`
/// 
/// # Arguments
/// * `input` - The input span to parse
/// 
/// # Returns
/// `Ok((remaining, (level, content)))` where level is 1 or 2 and content is the heading text
pub fn setext_heading(input: Span) -> IResult<Span, (u8, Span)> {
    log::debug!("Parsing Setext heading: {:?}", input.fragment());
    
    let start = input;
    let start_offset = start.location_offset();
    
    // CRITICAL: Setext headings cannot have link reference definitions as content
    // Check if the first line matches the link reference pattern: ^\[.*\]:\s
    // We do a quick check before full parsing
    let first_line_end = input.fragment().find('\n').unwrap_or(input.fragment().len());
    let first_line = &input.fragment()[..first_line_end];
    let trimmed_first = first_line.trim_start_matches(' ');
    
    // If first line looks like a link reference definition, reject immediately
    // Pattern: optional spaces (0-3) + '[' + text + ']:' 
    if trimmed_first.starts_with('[') && trimmed_first.contains("]:") {
        // Do a more precise check using the link_reference_definition parser
        if link_reference_definition(input).is_ok() {
            log::debug!("Setext heading rejected: content is a link reference definition");
            return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
        }
    }
    
    // Helper: Check if a line starts with blockquote marker ('>') with 0-3 leading spaces
    fn has_blockquote_marker(line: &str) -> bool {
        let trimmed = line.trim_start_matches(' ');
        let leading_spaces = line.len() - trimmed.len();
        leading_spaces <= 3 && trimmed.starts_with('>')
    }
    
    // 1. Parse the content lines (heading text) - can be multiple lines
    // Each line cannot be indented more than 3 spaces
    // CRITICAL: All lines (content + underline) must be in the same block context
    let mut content_end_offset;
    let mut current_input = input;
    let mut has_content = false;
    let mut first_line_in_blockquote: Option<bool> = None;
    
    // Parse at least one line of content
    loop {
        let (after_spaces, leading_spaces) = take_while(|c| c == ' ')(current_input)?;
        if leading_spaces.fragment().len() > 3 {
            if !has_content {
                return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
            }
            break;
        }
        
        // Get the text line
        let (after_line, text_line) = take_while(|c| c != '\n' && c != '\r')(after_spaces)?;
        
        // Text cannot be empty on first line
        if !has_content && text_line.fragment().trim().is_empty() {
            return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
        }
        
        // Check block context: is this line in a blockquote?
        let line_start = current_input.location_offset();
        let line_end = text_line.location_offset() + text_line.fragment().len();
        let full_line_len = line_end - line_start;
        let full_line = &current_input.fragment()[..full_line_len.min(current_input.fragment().len())];
        let this_line_in_blockquote = has_blockquote_marker(full_line);
        
        // First content line sets the block context
        if first_line_in_blockquote.is_none() {
            first_line_in_blockquote = Some(this_line_in_blockquote);
        } else if let Some(first_context) = first_line_in_blockquote {
            // Subsequent lines must match first line's block context
            if first_context != this_line_in_blockquote {
                // Block context boundary crossed - setext heading cannot span this
                log::debug!("Setext heading rejected: content crosses blockquote boundary");
                return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
            }
        }
        
        has_content = true;
        content_end_offset = text_line.location_offset() + text_line.fragment().len();
        
        // Must have line ending after content (setext needs underline on next line)
        let (after_newline, _) = line_ending(after_line)?;
        
        // Check if next line is blank - if so, this is NOT a setext heading
        if after_newline.fragment().starts_with('\n') || after_newline.fragment().starts_with('\r') {
            // Blank line - setext heading must have underline immediately after content
            return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
        }
        
        // Check if next line is the underline or another content line
        // Peek at next line to see if it's an underline
        let (peek_after_spaces, underline_spaces) = take_while(|c| c == ' ')(after_newline)?;
        if underline_spaces.fragment().len() > 3 {
            // Too much indentation for underline, continue as content
            current_input = after_newline;
            continue;
        }
        
        // Check block context for the potential underline line
        let _underline_line_start = after_newline.location_offset();
        let underline_peek_len = after_newline.fragment().find('\n').unwrap_or(after_newline.fragment().len());
        let underline_full_line = &after_newline.fragment()[..underline_peek_len.min(after_newline.fragment().len())];
        let underline_in_blockquote = has_blockquote_marker(underline_full_line);
        
        // Underline MUST be in same block context as content
        if first_line_in_blockquote.unwrap() != underline_in_blockquote {
            log::debug!("Setext heading rejected: underline crosses blockquote boundary");
            return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
        }
        
        // Check if we have an underline character
        if let Ok((peek_after_char, first_char)) = nom::character::complete::one_of::<_, _, nom::error::Error<_>>("=-")(peek_after_spaces) {
            // Check if rest of line is all the same character (valid underline)
            let (after_underline, _) = take_while(|c| c == first_char)(peek_after_char)?;
            
            // Count underline characters (must be at least 1, and no spaces allowed)
            let underline_offset = peek_after_spaces.location_offset();
            let underline_len = after_underline.location_offset() - underline_offset;
            let underline_str = &peek_after_spaces.fragment()[..underline_len];
            
            // Verify underline is solid (no spaces)
            if underline_str.chars().all(|c| c == first_char) && !underline_str.is_empty() {
                // Valid underline - check it ends properly (trailing spaces/tabs allowed)
                let (after_trailing_ws, _) = take_while(|c| c == ' ' || c == '\t')(after_underline)?;
                
                // Must end with line ending or EOF
                if let Ok((remaining, _)) = alt::<_, _, nom::error::Error<Span>, _>((
                    recognize(line_ending),
                    recognize(nom::combinator::eof),
                ))(after_trailing_ws) {
                    // This is a valid setext heading!
                    let level = if first_char == '=' { 1 } else { 2 };
                    
                    // Extract content from original input - use slice to preserve position
                    let content_len = content_end_offset - start_offset;
                    let content_span = start.slice(..content_len);
                    
                    log::debug!("Setext heading parsed: level={}, text={:?}", level, content_span.fragment());
                    
                    return Ok((remaining, (level, content_span)));
                }
            }
        }
        
        // Not an underline, continue parsing content lines
        current_input = after_newline;
    }
    
    // If we get here, we didn't find a valid underline
    Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_setext_level_1() {
        let input = Span::new("Heading 1\n===\n");
        let result = setext_heading(input);
        assert!(result.is_ok());
        let (_, (level, content)) = result.unwrap();
        assert_eq!(level, 1);
        assert_eq!(*content.fragment(), "Heading 1");
    }

    #[test]
    fn smoke_test_setext_level_2() {
        let input = Span::new("Heading 2\n---\n");
        let result = setext_heading(input);
        assert!(result.is_ok());
        let (_, (level, content)) = result.unwrap();
        assert_eq!(level, 2);
        assert_eq!(*content.fragment(), "Heading 2");
    }

    #[test]
    fn smoke_test_setext_minimal_underline() {
        let input = Span::new("Title\n=\n");
        let result = setext_heading(input);
        assert!(result.is_ok());
        let (_, (level, _)) = result.unwrap();
        assert_eq!(level, 1);
    }

    #[test]
    fn smoke_test_setext_multiline_text() {
        let input = Span::new("First line\nSecond line\n===\n");
        let result = setext_heading(input);
        assert!(result.is_ok());
        let (_, (level, content)) = result.unwrap();
        assert_eq!(level, 1);
        assert!(content.fragment().contains("First line"));
        assert!(content.fragment().contains("Second line"));
    }

    #[test]
    fn smoke_test_setext_leading_spaces() {
        let input = Span::new("  Heading\n  ===\n");
        let result = setext_heading(input);
        assert!(result.is_ok());
    }

    #[test]
    fn smoke_test_setext_blank_line_before_underline_fails() {
        let input = Span::new("Heading\n\n===\n");
        let result = setext_heading(input);
        assert!(result.is_err());
    }

    #[test]
    fn smoke_test_setext_four_space_indent_fails() {
        let input = Span::new("    Heading\n===\n");
        let result = setext_heading(input);
        assert!(result.is_err());
    }

    #[test]
    fn smoke_test_setext_empty_first_line_fails() {
        let input = Span::new("\n===\n");
        let result = setext_heading(input);
        assert!(result.is_err());
    }
}
