// Block-level grammar: headings, paragraphs, lists, code blocks, blockquotes, tables

use nom::{
    IResult,
    bytes::complete::{tag, take_while},
    character::complete::{space1, line_ending},
    multi::many1_count,
    combinator::{opt, recognize},
    branch::alt,
};
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;

// ATX Heading parser (# through ######)
// Returns (level, content) where level is 1-6
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
    
    // 3. Require at least one space or end of line after hashes
    let (input, space_or_eol) = alt((
        recognize(space1),
        recognize(line_ending),
        recognize(nom::combinator::eof),
    ))(input)?;
    
    // If there's no space/EOL, it's not a valid heading (e.g., "#5 bolt" or "#hashtag")
    if space_or_eol.fragment().is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    // 4. Parse content until end of line
    let (input, content) = take_while(|c| c != '\n' && c != '\r')(input)?;
    
    // 5. Trim trailing spaces and optional closing hashes
    let content_str = content.fragment();
    let trimmed = content_str.trim_end();
    
    // Remove trailing hashes if they're preceded by a space
    let final_content = if let Some(hash_pos) = trimmed.rfind(|c: char| c != '#' && c != ' ') {
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
    
    // Create a span for the final content
    let content_span = LocatedSpan::new(final_content);
    
    // Consume the newline if present
    let (input, _) = opt(line_ending)(input)?;
    
    log::debug!("Parsed heading level {}: {:?}", level, final_content);
    Ok((input, (level as u8, content_span)))
}

// Paragraph parser
// Parses a paragraph as a sequence of non-blank lines.
// A paragraph ends at a blank line or end of input.
// Leading spaces (0-3) are allowed, 4+ spaces means code block.
// Returns the paragraph content with internal newlines but no trailing newline.
pub fn paragraph(input: Span) -> IResult<Span, Span> {
    use nom::character::complete::not_line_ending;
    
    log::debug!("Parsing paragraph from: {:?}", &input.fragment()[..input.fragment().len().min(40)]);
    
    let original_input = input;
    
    // Check for leading spaces (4+ = code block)
    let (input, spaces) = take_while(|c| c == ' ')(input)?;
    if spaces.fragment().len() >= 4 {
        return Err(nom::Err::Error(nom::error::Error::new(original_input, nom::error::ErrorKind::Tag)));
    }
    
    // Parse at least one line of text
    let (input, first_line) = not_line_ending(input)?;
    
    // First line must not be empty (blank lines don't start paragraphs)
    if first_line.fragment().trim().is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(original_input, nom::error::ErrorKind::Tag)));
    }
    
    // Track the end of content (last non-blank line)
    let mut last_line_end = first_line.location_offset() + first_line.fragment().len();
    
    // Consume the newline after first line if present
    let (mut input, _) = opt(line_ending)(input)?;
    
    // Continue parsing lines until we hit a blank line or end of input
    loop {
        // Try to parse leading spaces
        let (after_spaces, spaces) = match take_while::<_, _, nom::error::Error<Span>>(|c| c == ' ')(input) {
            Ok(result) => result,
            Err(_) => break,
        };
        
        // Check for 4+ spaces (would be code block, so stop paragraph)
        if spaces.fragment().len() >= 4 {
            break;
        }
        
        // Try to parse the line content
        let (after_line, line) = match not_line_ending::<Span, nom::error::Error<Span>>(after_spaces) {
            Ok(result) => result,
            Err(_) => break,
        };
        
        // Check if line is blank (only whitespace or empty)
        if line.fragment().trim().is_empty() {
            // Blank line ends the paragraph
            break;
        }
        
        // This is a valid continuation line
        // Update the end position to include this line
        last_line_end = line.location_offset() + line.fragment().len();
        
        // Try to consume newline
        match line_ending::<Span, nom::error::Error<Span>>(after_line) {
            Ok((after_newline, _)) => {
                input = after_newline;
            }
            Err(_) => {
                // No newline, we're at end of input
                input = after_line;
                break;
            }
        }
    }
    
    // Calculate paragraph content from original input
    let start_offset = original_input.location_offset() + spaces.fragment().len();
    let content_len = last_line_end - start_offset;
    let para_content = &original_input.fragment()[spaces.fragment().len()..spaces.fragment().len() + content_len];
    let para_span = Span::new(para_content);
    
    log::debug!("Parsed paragraph: {:?}", &para_content[..para_content.len().min(40)]);
    
    Ok((input, para_span))
}

// Fenced code block parser
// Parses ``` or ~~~ code blocks with optional language info string.
// Returns (language, content) where language is the first word of info string.
pub fn fenced_code_block(input: Span) -> IResult<Span, (Option<String>, Span)> {
    use nom::character::complete::{char as nom_char, not_line_ending};
    
    log::debug!("Parsing fenced code block from: {:?}", &input.fragment()[..input.fragment().len().min(20)]);
    
    let original_input = input;
    
    // Parse optional leading spaces (0-3 allowed)
    let (input, leading_spaces) = take_while(|c| c == ' ')(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(original_input, nom::error::ErrorKind::Tag)));
    }
    
    // Parse the opening fence (``` or ~~~)
    let (input, fence_char) = alt((nom_char('`'), nom_char('~')))(input)?;
    
    // Count the fence delimiters (must be at least 3)
    let (input, fence_count) = {
        let mut count = 1; // We already parsed one
        let mut current = input;
        
        while let Ok((remaining, _)) = nom_char::<_, nom::error::Error<Span>>(fence_char)(current) {
            count += 1;
            current = remaining;
        }
        
        if count < 3 {
            return Err(nom::Err::Error(nom::error::Error::new(original_input, nom::error::ErrorKind::Tag)));
        }
        
        (current, count)
    };
    
    // Parse optional info string (rest of the line after fence)
    let (input, info_line) = not_line_ending(input)?;
    let info_string = info_line.fragment().trim();
    
    // Extract language (first word of info string)
    let language = if !info_string.is_empty() {
        Some(info_string.split_whitespace().next().unwrap_or("").to_string())
    } else {
        None
    };
    
    // Consume newline after opening fence
    let (mut input, _) = line_ending(input)?;
    
    // Track content start and end positions
    let content_start = input.location_offset();
    let mut content_end = content_start;
    
    // Collect code block content lines until we find closing fence
    let mut found_closing = false;
    
    loop {
        // Check for closing fence
        let check_input = input;
        
        // Try to parse optional leading spaces (0-3)
        if let Ok((after_spaces, spaces)) = take_while::<_, _, nom::error::Error<Span>>(|c| c == ' ')(check_input) {
            if spaces.fragment().len() <= 3 {
                // Try to match the fence character
                if let Ok((after_fence_start, _)) = nom_char::<_, nom::error::Error<Span>>(fence_char)(after_spaces) {
                    // Count closing fence delimiters
                    let mut close_count = 1;
                    let mut current = after_fence_start;
                    
                    while let Ok((remaining, _)) = nom_char::<_, nom::error::Error<Span>>(fence_char)(current) {
                        close_count += 1;
                        current = remaining;
                    }
                    
                    // Closing fence must have at least as many delimiters as opening
                    if close_count >= fence_count {
                        // Check that rest of line is whitespace only
                        if let Ok((after_line, rest)) = not_line_ending::<_, nom::error::Error<Span>>(current) {
                            if rest.fragment().trim().is_empty() {
                                // Valid closing fence!
                                found_closing = true;
                                // Consume the closing fence line and optional newline
                                input = after_line;
                                let _ = opt::<_, _, nom::error::Error<Span>, _>(line_ending)(input)?;
                                break;
                            }
                        }
                    }
                }
            }
        }
        
        // Not a closing fence, so this line is content
        // Parse the line
        match not_line_ending::<Span, nom::error::Error<Span>>(input) {
            Ok((after_line, line)) => {
                // Update content end to include this line
                content_end = line.location_offset() + line.fragment().len();
                
                // Try to consume newline
                match line_ending::<Span, nom::error::Error<Span>>(after_line) {
                    Ok((after_newline, _)) => {
                        content_end += 1; // Include newline in content
                        input = after_newline;
                    }
                    Err(_) => {
                        // No newline, end of input
                        input = after_line;
                        break;
                    }
                }
            }
            Err(_) => {
                // Can't parse line, end of input
                break;
            }
        }
    }
    
    if !found_closing {
        // Unclosed code block is still valid in CommonMark (content goes to end of document)
        log::debug!("Unclosed fenced code block");
    }
    
    // Calculate content length and create span from original input
    let content_len = content_end.saturating_sub(content_start);
    
    // Find the content in the original input
    // We need to calculate offset from original_input start
    let offset_from_original = content_start - original_input.location_offset();
    let content_fragment = if content_len > 0 && offset_from_original + content_len <= original_input.fragment().len() {
        &original_input.fragment()[offset_from_original..offset_from_original + content_len]
    } else {
        ""
    };
    
    // Remove trailing newline if present (CommonMark doesn't include trailing newline in content)
    let content_fragment = content_fragment.strip_suffix('\n').unwrap_or(content_fragment);
    let content_span = Span::new(content_fragment);
    
    log::debug!("Parsed fenced code block with language={:?}, content length={}", language, content_fragment.len());
    
    Ok((input, (language, content_span)))
}

// Legacy function name for compatibility
pub fn code_block(input: Span) -> IResult<Span, Span> {
    // Try fenced code block first
    if let Ok((remaining, (_, content))) = fenced_code_block(input) {
        Ok((remaining, content))
    } else {
        // TODO: Implement indented code blocks
        Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
    }
}

// List parser (ordered and unordered)
pub fn list(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing list");
    // TODO: Implement list parsing
    Ok((input, input))
}

// Blockquote parser (>)
pub fn blockquote(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing blockquote");
    // TODO: Implement blockquote parsing
    Ok((input, input))
}

// Table parser
pub fn table(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing table");
    // TODO: Implement table parsing
    Ok((input, input))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn smoke_test_heading_levels() {
        // Test all heading levels 1-6
        for level in 1..=6 {
            let hashes = "#".repeat(level);
            let input_str = format!("{} Heading level {}", hashes, level);
            let input = Span::new(&input_str);
            
            let result = heading(input);
            assert!(result.is_ok(), "Failed to parse heading level {}", level);
            
            let (_, (parsed_level, content)) = result.unwrap();
            assert_eq!(parsed_level, level as u8);
            assert!(content.fragment().contains(&format!("Heading level {}", level)));
        }
    }
    
    #[test]
    fn smoke_test_heading_basic() {
        let input = Span::new("# Hello World");
        let result = heading(input);
        
        assert!(result.is_ok());
        let (_, (level, content)) = result.unwrap();
        assert_eq!(level, 1);
        assert_eq!(content.fragment(), &"Hello World");
    }
    
    #[test]
    fn smoke_test_heading_trailing_hashes() {
        // Example 71: trailing hashes should be removed
        let input = Span::new("## foo ##");
        let result = heading(input);
        
        assert!(result.is_ok());
        let (_, (level, content)) = result.unwrap();
        assert_eq!(level, 2);
        assert_eq!(content.fragment(), &"foo");
    }
    
    #[test]
    fn smoke_test_heading_trim_whitespace() {
        // Example 67: multiple spaces should be trimmed
        let input = Span::new("#                  foo                     ");
        let result = heading(input);
        
        assert!(result.is_ok());
        let (_, (level, content)) = result.unwrap();
        assert_eq!(level, 1);
        assert_eq!(content.fragment(), &"foo");
    }
    
    #[test]
    fn smoke_test_heading_leading_spaces() {
        // Example 68: 0-3 leading spaces allowed
        let input = Span::new("   # foo");
        let result = heading(input);
        
        assert!(result.is_ok());
        let (_, (level, content)) = result.unwrap();
        assert_eq!(level, 1);
        assert_eq!(content.fragment(), &"foo");
    }
    
    #[test]
    fn smoke_test_heading_invalid_seven_hashes() {
        // Example 63: 7+ hashes is not a heading
        let input = Span::new("####### foo");
        let result = heading(input);
        
        assert!(result.is_err(), "Should fail with 7 hashes");
    }
    
    #[test]
    fn smoke_test_heading_invalid_no_space() {
        // Example 64: No space after # means not a heading
        let input = Span::new("#5 bolt");
        let result = heading(input);
        
        assert!(result.is_err(), "Should fail without space after #");
    }
    
    #[test]
    fn smoke_test_heading_invalid_four_space_indent() {
        // Example 69: 4+ spaces means code block
        let input = Span::new("    # foo");
        let result = heading(input);
        
        assert!(result.is_err(), "Should fail with 4+ leading spaces");
    }
    
    #[test]
    fn smoke_test_heading_empty_content() {
        // Example 79: empty heading is valid
        let input = Span::new("## ");
        let result = heading(input);
        
        assert!(result.is_ok());
        let (_, (level, content)) = result.unwrap();
        assert_eq!(level, 2);
        assert_eq!(content.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_paragraph_simple() {
        // Example 219: Simple single-line paragraph
        let input = Span::new("aaa");
        let result = paragraph(input);
        
        assert!(result.is_ok());
        let (remaining, content) = result.unwrap();
        assert_eq!(content.fragment(), &"aaa");
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_paragraph_multiline() {
        // Example 220: Multi-line paragraph
        let input = Span::new("aaa\nbbb");
        let result = paragraph(input);
        
        assert!(result.is_ok());
        let (remaining, content) = result.unwrap();
        assert_eq!(content.fragment(), &"aaa\nbbb");
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_paragraph_blank_line_terminator() {
        // Example 219: Paragraph ends at blank line
        let input = Span::new("aaa\n\nbbb");
        let result = paragraph(input);
        
        assert!(result.is_ok());
        let (remaining, content) = result.unwrap();
        assert_eq!(content.fragment(), &"aaa");
        assert!(remaining.fragment().starts_with("\n"));
    }
    
    #[test]
    fn smoke_test_paragraph_leading_spaces() {
        // Example 222: 0-3 leading spaces allowed
        let input = Span::new("  aaa\n bbb");
        let result = paragraph(input);
        
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        // Content should include the first line with its spaces
        assert!(content.fragment().contains("aaa"));
        assert!(content.fragment().contains("bbb"));
    }
    
    #[test]
    fn smoke_test_paragraph_four_spaces_rejected() {
        // Example 225: 4+ leading spaces means code block
        let input = Span::new("    aaa");
        let result = paragraph(input);
        
        assert!(result.is_err(), "Should fail with 4+ leading spaces");
    }
    
    #[test]
    fn smoke_test_paragraph_blank_line_rejected() {
        // Blank lines don't start paragraphs
        let input = Span::new("\n\naaa");
        let result = paragraph(input);
        
        assert!(result.is_err(), "Should fail on blank line");
    }
    
    #[test]
    fn smoke_test_paragraph_whitespace_only_rejected() {
        // Lines with only whitespace don't start paragraphs
        let input = Span::new("   \n");
        let result = paragraph(input);
        
        assert!(result.is_err(), "Should fail on whitespace-only line");
    }
    
    #[test]
    fn smoke_test_fenced_code_block_basic_backticks() {
        // Example 119: Basic fenced code block with backticks
        let input = Span::new("```\n<\n >\n```\n");
        let result = fenced_code_block(input);
        
        assert!(result.is_ok());
        let (_, (language, content)) = result.unwrap();
        assert_eq!(language, None);
        assert_eq!(content.fragment(), &"<\n >");
    }
    
    #[test]
    fn smoke_test_fenced_code_block_basic_tildes() {
        // Example 120: Basic fenced code block with tildes
        let input = Span::new("~~~\n<\n >\n~~~\n");
        let result = fenced_code_block(input);
        
        assert!(result.is_ok());
        let (_, (language, content)) = result.unwrap();
        assert_eq!(language, None);
        assert_eq!(content.fragment(), &"<\n >");
    }
    
    #[test]
    fn smoke_test_fenced_code_block_with_language() {
        // Example 142: Code block with language info string
        let input = Span::new("```ruby\ndef foo(x)\n  return 3\nend\n```\n");
        let result = fenced_code_block(input);
        
        assert!(result.is_ok());
        let (_, (language, content)) = result.unwrap();
        assert_eq!(language, Some("ruby".to_string()));
        assert_eq!(content.fragment(), &"def foo(x)\n  return 3\nend");
    }
    
    #[test]
    fn smoke_test_fenced_code_block_longer_closing() {
        // Closing fence can be longer than opening
        let input = Span::new("```\ncode\n````\n");
        let result = fenced_code_block(input);
        
        assert!(result.is_ok());
        let (_, (language, content)) = result.unwrap();
        assert_eq!(language, None);
        assert_eq!(content.fragment(), &"code");
    }
    
    #[test]
    fn smoke_test_fenced_code_block_nested_fences() {
        // Example 122: Different fence chars can be nested
        let input = Span::new("```\naaa\n~~~\n```\n");
        let result = fenced_code_block(input);
        
        assert!(result.is_ok());
        let (_, (_, content)) = result.unwrap();
        assert_eq!(content.fragment(), &"aaa\n~~~");
    }
    
    #[test]
    fn smoke_test_fenced_code_block_less_than_three() {
        // Example 121: Less than 3 delimiters is not a code block
        let input = Span::new("``\nfoo\n``\n");
        let result = fenced_code_block(input);
        
        assert!(result.is_err(), "Should fail with less than 3 delimiters");
    }
    
    #[test]
    fn smoke_test_fenced_code_block_unclosed() {
        // Unclosed code blocks are valid - go to end of input
        let input = Span::new("```\ncode\nmore code");
        let result = fenced_code_block(input);
        
        assert!(result.is_ok());
        let (_, (_, content)) = result.unwrap();
        assert_eq!(content.fragment(), &"code\nmore code");
    }
}
