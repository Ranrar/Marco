// CommonMark ATX Heading Grammar
// Parses headings with # markers: # Heading, ## Heading, etc.
//
// Per CommonMark spec:
// - 1-6 opening # characters
// - Must have space/tab after # or be at end of line
// - Can have 0-3 leading spaces
// - Can have optional closing # sequence (removed)
// - Content is trimmed

use crate::grammar::shared::Span;
use nom::{
    IResult,
    bytes::complete::{tag, take_while},
    combinator::{opt, recognize},
    multi::many1_count,
    character::complete::line_ending,
    Slice,
};

/// Parse an ATX heading (1-6 # characters)
/// 
/// Examples:
/// - `# Heading 1`
/// - `## Heading 2`
/// - `### Heading 3 ###` (with closing hashes)
/// - `  # Heading` (with leading spaces)
/// 
/// # Arguments
/// * `input` - The input span to parse
/// 
/// # Returns
/// `Ok((remaining, (level, content)))` where level is 1-6 and content is the heading text
pub fn heading(input: Span) -> IResult<Span, (u8, Span)> {
    log::debug!("Parsing ATX heading: {:?}", input.fragment());
    
    let start = input;
    
    // 1. Optional leading spaces (0-3 spaces allowed)
    let (input, leading_spaces) = take_while(|c| c == ' ')(input)?;
    if leading_spaces.fragment().len() > 3 {
        // 4+ spaces means indented code block, not heading
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    // 2. Count opening # symbols (1-6)
    let (input, hashes) = recognize(many1_count(tag("#")))(input)?;
    let level = hashes.fragment().len();
    
    if level > 6 {
        // 7+ hashes is not a valid heading
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    // 3. Require at least one space/tab or end of line after hashes
    // Check what's next without consuming
    let next_char = input.fragment().chars().next();
    let is_valid_separator = match next_char {
        None => true,  // EOF
        Some('\n') | Some('\r') => true,  // Newline
        Some(' ') | Some('\t') => true,  // Whitespace
        Some(_) => false,  // Other character - not valid
    };
    
    if !is_valid_separator {
        // No valid separator - not a valid heading (e.g., "#hashtag")
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
        // Consume whitespace (but not newlines)
    let (input, _) = take_while(|c| c == ' ' || c == '\t')(input)?;
    
    // 4. Parse content until end of line
    let (input, content) = take_while(|c| c != '\n' && c != '\r')(input)?;
    
    // 5. Trim trailing spaces and optional closing hashes
    let content_str = content.fragment();
    let trimmed = content_str.trim_end();
    
    // Remove trailing hashes if they're preceded by a space
    let final_content_str = if let Some(hash_pos) = trimmed.rfind(|c: char| c != '#' && c != ' ') {
        let after_content = &trimmed[hash_pos + 1..];
        // If everything after is spaces and hashes, remove them
        if after_content.chars().all(|c| c == ' ' || c == '#') {
            let before_trailing = &trimmed[..=hash_pos];
            before_trailing.trim_end()
        } else {
            trimmed
        }
    } else {
        // Content is all hashes/spaces or empty
        ""
    };
    
    // Slice the original content span to maintain position information
    // CRITICAL: Do NOT use LocatedSpan::new() as it resets position to 0:0
    // Instead, slice the original span to preserve line/column/offset
    let content_len = final_content_str.len();
    let content_span = content.slice(..content_len);
    
    // Consume the newline if present
    let (remaining, _) = opt(line_ending)(input)?;
    
    log::debug!("Parsed heading level {}: {:?}", level, final_content_str);
    Ok((remaining, (level as u8, content_span)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_heading_level_1() {
        let input = Span::new("# Hello World\n");
        let result = heading(input);
        assert!(result.is_ok());
        let (_, (level, content)) = result.unwrap();
        assert_eq!(level, 1);
        assert_eq!(*content.fragment(), "Hello World");
    }

    #[test]
    fn smoke_test_heading_level_6() {
        let input = Span::new("###### Heading 6\n");
        let result = heading(input);
        assert!(result.is_ok());
        let (_, (level, content)) = result.unwrap();
        assert_eq!(level, 6);
        assert_eq!(*content.fragment(), "Heading 6");
    }

    #[test]
    fn smoke_test_heading_trailing_hashes() {
        let input = Span::new("## Heading ##\n");
        let result = heading(input);
        assert!(result.is_ok());
        let (_, (_, content)) = result.unwrap();
        assert_eq!(*content.fragment(), "Heading");
    }

    #[test]
    fn smoke_test_heading_leading_spaces() {
        let input = Span::new("  # Heading\n");
        let result = heading(input);
        assert!(result.is_ok());
        let (_, (level, _)) = result.unwrap();
        assert_eq!(level, 1);
    }

    #[test]
    fn smoke_test_heading_empty_content() {
        let input = Span::new("# \n");
        let result = heading(input);
        assert!(result.is_ok());
        let (_, (level, content)) = result.unwrap();
        assert_eq!(level, 1);
        assert_eq!(*content.fragment(), "");
    }

    #[test]
    fn smoke_test_heading_seven_hashes_fails() {
        let input = Span::new("####### Not a heading\n");
        let result = heading(input);
        assert!(result.is_err());
    }

    #[test]
    fn smoke_test_heading_no_space_after_hash() {
        let input = Span::new("#NoSpace\n");
        let result = heading(input);
        assert!(result.is_err());
    }

    #[test]
    fn smoke_test_heading_four_space_indent_fails() {
        let input = Span::new("    # Not a heading\n");
        let result = heading(input);
        assert!(result.is_err());
    }
}
