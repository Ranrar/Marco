// CommonMark Indented Code Block Grammar
// Parses code blocks with 4 spaces or 1 tab indentation
//
// Per CommonMark spec:
// - Lines indented with at least 4 effective spaces (tabs count as 4)
// - Can contain blank lines
// - Ends when line is not indented or end of input
// - Leading indentation is removed during parsing

use crate::grammar::shared::{Span, skip_indentation};
use nom::{
    IResult,
    bytes::complete::take_while,
    character::complete::line_ending,
};

/// Parse an indented code block (4 spaces or 1 tab)
/// 
/// Examples:
/// - `    code line 1\n    code line 2`
/// - `\tcode with tab`
/// - `    code\n\n    more code` (with blank line)
/// 
/// # Arguments
/// * `input` - The input span to parse
/// 
/// # Returns
/// `Ok((remaining, content_span))` where content_span includes indentation (removed by parser)
pub fn indented_code_block(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing indented code block: {:?}", crate::logic::logger::safe_preview(input.fragment(), 40));
    
    let start = input;
    let mut remaining = input;
    let start_offset = start.location_offset();
    let mut last_content_offset = start_offset;
    
    // Parse consecutive indented lines (need at least 4 effective spaces)
    loop {
        // Try to skip at least 4 spaces of indentation (counting tab expansion)
        let indent_result = skip_indentation(remaining, 4);
        
        match indent_result {
            Ok((after_indent, effective_spaces)) if effective_spaces >= 4 => {
                // Get the rest of the line
                let (after_line, _line) = take_while(|c| c != '\n' && c != '\r')(after_indent)?;
                
                // Update last_content_offset to include this line
                // Important: we want the offset of the END of the line content
                let line_end_offset = after_line.location_offset();
                last_content_offset = line_end_offset;
                
                log::debug!("Indented code line parsed, line_end_offset={}", line_end_offset);
                
                // Try to consume line ending
                match line_ending::<Span, nom::error::Error<Span>>(after_line) {
                    Ok((after_newline, newline)) => {
                        last_content_offset += newline.fragment().len();
                        remaining = after_newline;
                        
                        // Peek ahead: is next line blank or indented?
                        if remaining.fragment().starts_with('\n') || remaining.fragment().starts_with('\r') {
                            // Blank line - consume it and continue
                            if let Ok((after_blank, blank)) = line_ending::<Span, nom::error::Error<Span>>(remaining) {
                                last_content_offset = blank.location_offset() + blank.fragment().len();
                                remaining = after_blank;
                                continue;
                            }
                        }
                        // Continue to next iteration to check for indentation
                        continue;
                    }
                    Err(_) => {
                        // No newline, end of input
                        log::debug!("No trailing newline, end of code block");
                        break;
                    }
                }
            }
            _ => {
                // Line doesn't have 4 spaces of indentation, end of code block
                break;
            }
        }
    }
    
    // Calculate content length
    let content_len = last_content_offset.saturating_sub(start_offset);
    
    log::debug!("Indented code block: start_offset={}, last_content_offset={}, content_len={}", start_offset, last_content_offset, content_len);
    
    if content_len == 0 {
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    // Extract content from input
    let content_fragment = &start.fragment()[..content_len.min(start.fragment().len())];
    
    // Create span for the content (will be processed by parser to remove indentation)
    let content_span = Span::new(content_fragment);
    
    log::debug!("Indented code block parsed: {} bytes", content_fragment.len());
    
    Ok((remaining, content_span))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_indented_code_single_line() {
        let input = Span::new("    code\n");
        let result = indented_code_block(input);
        assert!(result.is_ok());
        let (remaining, content) = result.unwrap();
        assert!(content.fragment().contains("code"));
        assert_eq!(*remaining.fragment(), "");
    }

    #[test]
    fn smoke_test_indented_code_with_tab() {
        let input = Span::new("\tcode\n");
        let result = indented_code_block(input);
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert!(content.fragment().contains("code"));
    }

    #[test]
    fn smoke_test_indented_code_multiple_lines() {
        let input = Span::new("    line1\n    line2\n");
        let result = indented_code_block(input);
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert!(content.fragment().contains("line1"));
        assert!(content.fragment().contains("line2"));
    }

    #[test]
    fn smoke_test_indented_code_with_blank_lines() {
        let input = Span::new("    code1\n\n    code2\n");
        let result = indented_code_block(input);
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert!(content.fragment().contains("code1"));
        assert!(content.fragment().contains("code2"));
    }

    #[test]
    fn smoke_test_indented_code_no_trailing_newline() {
        let input = Span::new("    code");
        let result = indented_code_block(input);
        assert!(result.is_ok());
    }

    #[test]
    fn smoke_test_three_spaces_fails() {
        let input = Span::new("   not code\n");
        let result = indented_code_block(input);
        assert!(result.is_err());
    }

    #[test]
    fn smoke_test_no_indent_fails() {
        let input = Span::new("not code\n");
        let result = indented_code_block(input);
        assert!(result.is_err());
    }
}
