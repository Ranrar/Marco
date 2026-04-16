// CommonMark Fenced Code Block Grammar
// Parses code blocks with ``` or ~~~ fences
//
// Per CommonMark spec:
// - Opening fence: at least 3 backticks or tildes
// - Optional info string (language) after opening fence
// - Closing fence: at least as many fence chars as opening
// - Can have 0-3 leading spaces
// - Info string cannot contain backticks if using backtick fences
// - Content between fences is preserved as-is

use crate::grammar::shared::Span;
use nom::{
    branch::alt,
    bytes::complete::take_while,
    character::complete::{char as nom_char, line_ending, not_line_ending},
    combinator::opt,
    IResult, Input, Parser,
};

/// Parse a fenced code block (``` or ~~~)
///
/// Examples:
/// - ` ```\ncode\n``` `
/// - ` ```rust\nfn main() {}\n``` `
/// - ` ~~~python\nprint("hi")\n~~~ `
///
/// # Arguments
/// * `input` - The input span to parse
///
/// # Returns
/// `Ok((remaining, (language, content)))` where language is optional
pub fn fenced_code_block(input: Span) -> IResult<Span, (Option<String>, Span)> {
    log::debug!(
        "Parsing fenced code block from: {:?}",
        crate::logic::logger::safe_preview(input.fragment(), 20)
    );

    let original_input = input;

    // Parse optional leading spaces (0-3 allowed)
    let (input, leading_spaces) = take_while(|c| c == ' ').parse(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(
            original_input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Parse the opening fence (``` or ~~~)
    let (input, fence_char) = alt((nom_char('`'), nom_char('~'))).parse(input)?;

    // Count the fence delimiters (must be at least 3)
    let (input, fence_count) = {
        let mut count = 1; // We already parsed one
        let mut current = input;

        while let Ok((remaining, _)) = nom_char::<_, nom::error::Error<Span>>(fence_char)(current) {
            count += 1;
            current = remaining;
        }

        if count < 3 {
            return Err(nom::Err::Error(nom::error::Error::new(
                original_input,
                nom::error::ErrorKind::Tag,
            )));
        }

        (current, count)
    };

    // Parse optional info string (rest of the line after fence)
    let (input, info_line) = not_line_ending(input)?;
    let info_string = info_line.fragment().trim();

    // CommonMark spec: info string cannot contain backticks if fence uses backticks
    if fence_char == '`' && info_string.contains('`') {
        return Err(nom::Err::Error(nom::error::Error::new(
            original_input,
            nom::error::ErrorKind::Tag,
        )));
    }

    // Extract language (first word of info string)
    let language = if !info_string.is_empty() {
        Some(
            info_string
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string(),
        )
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
        if let Ok((after_spaces, spaces)) =
            take_while::<_, _, nom::error::Error<Span>>(|c| c == ' ')(check_input)
        {
            if spaces.fragment().len() <= 3 {
                // Try to match the fence character
                if let Ok((after_fence_start, _)) =
                    nom_char::<_, nom::error::Error<Span>>(fence_char)(after_spaces)
                {
                    // Count closing fence delimiters
                    let mut close_count = 1;
                    let mut current = after_fence_start;

                    while let Ok((remaining, _)) =
                        nom_char::<_, nom::error::Error<Span>>(fence_char)(current)
                    {
                        close_count += 1;
                        current = remaining;
                    }

                    // Closing fence must have at least as many delimiters as opening
                    if close_count >= fence_count {
                        // Check that rest of line is whitespace only
                        if let Ok((after_line, rest)) =
                            not_line_ending::<_, nom::error::Error<Span>>(current)
                        {
                            if rest.fragment().trim().is_empty() {
                                // Valid closing fence!
                                found_closing = true;
                                // Consume the closing fence line and optional newline
                                input = after_line;
                                let _ = opt(line_ending).parse(input)?;
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

    // CRITICAL: Use slice to preserve position information
    let content_span = if content_len > 0
        && offset_from_original + content_len <= original_input.fragment().len()
    {
        let mut span = original_input
            .take_from(offset_from_original)
            .take(content_len);
        // Remove trailing newline if present (CommonMark doesn't include trailing newline in content)
        if span.fragment().ends_with('\n') {
            let len = span.fragment().len();
            span = span.take(len.saturating_sub(1));
        }
        span
    } else {
        original_input.take_from(offset_from_original).take(0usize)
    };

    log::debug!(
        "Parsed fenced code block with language={:?}, content length={}",
        language,
        content_span.fragment().len()
    );

    Ok((input, (language, content_span)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_fenced_basic_backticks() {
        let input = Span::new("```\ncode\n```\n");
        let result = fenced_code_block(input);
        assert!(result.is_ok());
        let (_, (lang, content)) = result.unwrap();
        assert_eq!(lang, None);
        assert_eq!(*content.fragment(), "code");
    }

    #[test]
    fn smoke_test_fenced_with_language() {
        let input = Span::new("```rust\nfn main() {}\n```\n");
        let result = fenced_code_block(input);
        assert!(result.is_ok());
        let (_, (lang, _)) = result.unwrap();
        assert_eq!(lang, Some("rust".to_string()));
    }

    #[test]
    fn smoke_test_fenced_tildes() {
        let input = Span::new("~~~\ncode\n~~~\n");
        let result = fenced_code_block(input);
        assert!(result.is_ok());
    }

    #[test]
    fn smoke_test_fenced_longer_closing() {
        let input = Span::new("```\ncode\n`````\n");
        let result = fenced_code_block(input);
        assert!(result.is_ok());
    }

    #[test]
    fn smoke_test_fenced_unclosed() {
        let input = Span::new("```\ncode\n");
        let result = fenced_code_block(input);
        assert!(result.is_ok());
        let (_, (_, content)) = result.unwrap();
        assert_eq!(*content.fragment(), "code");
    }

    #[test]
    fn smoke_test_fenced_nested_fences() {
        let input = Span::new("````\n```\ncode\n```\n````\n");
        let result = fenced_code_block(input);
        assert!(result.is_ok());
        let (_, (_, content)) = result.unwrap();
        assert!(content.fragment().contains("```"));
    }

    #[test]
    fn smoke_test_fenced_less_than_three_fails() {
        let input = Span::new("``\ncode\n``\n");
        let result = fenced_code_block(input);
        assert!(result.is_err());
    }

    #[test]
    fn smoke_test_fenced_backtick_in_info_fails() {
        let input = Span::new("```rust`lang\ncode\n```\n");
        let result = fenced_code_block(input);
        assert!(result.is_err());
    }
}
