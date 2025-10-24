// CommonMark Paragraph Grammar
// Parses paragraphs as sequences of non-blank lines
//
// Per CommonMark spec:
// - Paragraph = sequence of non-blank lines
// - Leading spaces (0-3) allowed, 4+ spaces = code block
// - Ends at blank line, ATX heading, fenced code, blockquote, list, or EOF
// - Lazy continuation: indented lines continue paragraph unless blank line precedes
// - Ordered lists starting with "1" can interrupt, others can't
// - Unordered lists always interrupt

use crate::grammar::shared::{count_indentation, Span};
use nom::{
    IResult, Slice,
    bytes::complete::take_while,
    character::complete::{line_ending, not_line_ending},
    combinator::opt,
};

// Import list marker detection (needed to check if list interrupts paragraph)
use crate::grammar::blocks::cm_list::detect_list_marker;

/// Parse a paragraph
/// 
/// Examples:
/// - `"Hello world\nSecond line"`
/// - `"Text with  \n  lazy continuation"`
/// - `"Para\n\nEnds at blank"`
/// 
/// # Arguments
/// * `input` - The input span to parse
/// 
/// # Returns
/// * `Ok((remaining, paragraph_span))` - Success with remaining input and paragraph content
/// * `Err(_)` - Parse failure
pub fn paragraph(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing paragraph from: {:?}", crate::logic::logger::safe_preview(input.fragment(), 40));
    
    let original_input = input;
    
    // Check for leading indentation (4+ effective spaces = code block)
    let indentation = count_indentation(input.fragment());
    if indentation >= 4 {
        return Err(nom::Err::Error(nom::error::Error::new(original_input, nom::error::ErrorKind::Tag)));
    }
    
    // Skip the actual leading whitespace
    let (after_ws, _) = take_while(|c| c == ' ' || c == '\t')(original_input)?;
    
    // Parse at least one line of text
    let (after_first, first_line) = not_line_ending(after_ws)?;
    
    // First line must not be empty (blank lines don't start paragraphs)
    if first_line.fragment().trim().is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(original_input, nom::error::ErrorKind::Tag)));
    }
    
    // Track the end of content (last non-blank line)
    let mut last_line_end = first_line.location_offset() + first_line.fragment().len();
    
    // Consume the newline after first line if present
    let (mut input, _) = opt(line_ending)(after_first)?;
    
    // Continue parsing lines until we hit a blank line or end of input
    loop {
        // Try to parse leading spaces
        let (after_spaces, spaces) = match take_while::<_, _, nom::error::Error<Span>>(|c| c == ' ')(input) {
            Ok(result) => result,
            Err(_) => break,
        };
        
        // Check if line starts with ATX heading (# through ######)
        // ATX headings can interrupt paragraphs, but only with 0-3 leading spaces
        if spaces.fragment().len() <= 3 {
            let trimmed = after_spaces.fragment().trim_start();
            if trimmed.starts_with('#') {
                let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
                if (1..=6).contains(&hash_count) {
                    // Check if followed by space or end of line (valid ATX heading)
                    if hash_count == trimmed.len() || 
                       trimmed.chars().nth(hash_count).map(|c| c.is_whitespace()).unwrap_or(false) {
                        // This is an ATX heading, stop paragraph here
                        break;
                    }
                }
            }
            
            // Check if line starts with fenced code block (``` or ~~~)
            // Fenced code blocks can interrupt paragraphs with 0-3 leading spaces
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                let fence_char = trimmed.chars().next().unwrap();
                let fence_count = trimmed.chars().take_while(|&c| c == fence_char).count();
                if fence_count >= 3 {
                    // This is a fenced code block, stop paragraph here
                    break;
                }
            }
            
            // Check if line starts with blockquote (>)
            // Block quotes can interrupt paragraphs
            if trimmed.starts_with('>') {
                // This is a blockquote, stop paragraph here
                break;
            }
            
            // Check if line starts with list marker
            // Unordered lists can always interrupt paragraphs
            // Ordered lists can only interrupt if they start with "1"
            if detect_list_marker(after_spaces).is_ok() {
                // Check if it's unordered or ordered starting with 1
                let marker_chars: Vec<char> = trimmed.chars().take(5).collect();
                if marker_chars.first().map(|c| *c == '-' || *c == '*' || *c == '+').unwrap_or(false) {
                    // Unordered list, can interrupt
                    break;
                } else if marker_chars.first().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    // Ordered list - check if starts with "1"
                    if trimmed.starts_with("1.") || trimmed.starts_with("1)") {
                        // Can interrupt
                        break;
                    }
                    // Other numbers can't interrupt paragraphs
                }
            }
        }
        
        // Note: We allow indented lines as lazy continuation per CommonMark spec
        // Indented code blocks can only interrupt paragraphs if preceded by blank line
        
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
    // Use slice to preserve position information
    let leading_ws_len = original_input.fragment().len() - after_ws.fragment().len();
    let start_offset = original_input.location_offset() + leading_ws_len;
    let content_len = last_line_end - start_offset;
    let para_span = original_input.slice(leading_ws_len..leading_ws_len + content_len);
    
    log::debug!("Parsed paragraph: {:?}", crate::logic::logger::safe_preview(para_span.fragment(), 40));
    
    Ok((input, para_span))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_paragraph_single_line() {
        let input = Span::new("Hello world");
        let result = paragraph(input);
        assert!(result.is_ok());
        let (_, para) = result.unwrap();
        assert_eq!(para.fragment(), &"Hello world");
    }

    #[test]
    fn smoke_test_paragraph_multiline() {
        let input = Span::new("Line one\nLine two\nLine three");
        let result = paragraph(input);
        assert!(result.is_ok());
        let (_, para) = result.unwrap();
        assert_eq!(para.fragment(), &"Line one\nLine two\nLine three");
    }

    #[test]
    fn smoke_test_paragraph_ends_at_blank() {
        let input = Span::new("First para\nSecond line\n\nNext para");
        let result = paragraph(input);
        assert!(result.is_ok());
        let (remaining, para) = result.unwrap();
        assert_eq!(para.fragment(), &"First para\nSecond line");
        // Remaining starts with blank line (could be \n or \n\n depending on parser state)
        assert!(remaining.fragment().trim_start().starts_with("Next para"));
    }

    #[test]
    fn smoke_test_paragraph_with_leading_spaces() {
        let input = Span::new("  Indented para\n  Continued");
        let result = paragraph(input);
        assert!(result.is_ok());
        let (_, para) = result.unwrap();
        assert_eq!(para.fragment(), &"Indented para\n  Continued");
    }

    #[test]
    fn smoke_test_paragraph_interrupted_by_heading() {
        let input = Span::new("Para text\n# Heading");
        let result = paragraph(input);
        assert!(result.is_ok());
        let (remaining, para) = result.unwrap();
        assert_eq!(para.fragment(), &"Para text");
        assert!(remaining.fragment().starts_with("# Heading"));
    }

    #[test]
    fn smoke_test_paragraph_interrupted_by_fence() {
        let input = Span::new("Para text\n```\ncode\n```");
        let result = paragraph(input);
        assert!(result.is_ok());
        let (remaining, para) = result.unwrap();
        assert_eq!(para.fragment(), &"Para text");
        assert!(remaining.fragment().starts_with("```"));
    }

    #[test]
    fn smoke_test_paragraph_fails_with_4_spaces() {
        let input = Span::new("    Code block");
        let result = paragraph(input);
        assert!(result.is_err());
    }

    #[test]
    fn smoke_test_paragraph_lazy_continuation() {
        let input = Span::new("First line\n    Lazy indented\nThird line");
        let result = paragraph(input);
        assert!(result.is_ok());
        let (_, para) = result.unwrap();
        assert!(para.fragment().contains("Lazy indented"));
    }
}
