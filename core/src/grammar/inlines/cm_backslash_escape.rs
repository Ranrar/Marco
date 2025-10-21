//! Backslash escape grammar - handles escape sequences
//!
//! Per CommonMark spec, a backslash before any ASCII punctuation character
//! escapes that character: !"#$%&'()*+,-./:;<=>?@[\]^_`{|}~

use nom::{
    IResult,
    character::complete::char,
};
use super::Span;

/// Parse a backslash escape sequence.
///
/// # Grammar
/// A backslash (`\`) followed by any ASCII punctuation character.
///
/// # Returns
/// The escaped character (without the backslash).
///
/// # Example
/// ```ignore
/// let input = Span::new("\\*");
/// let (rest, ch) = backslash_escape(input).unwrap();
/// assert_eq!(ch, '*');
/// ```
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn smoke_test_backslash_escape_asterisk() {
        let input = Span::new("\\*rest");
        let result = backslash_escape(input);
        assert!(result.is_ok());
        let (rest, ch) = result.unwrap();
        assert_eq!(ch, '*');
        assert_eq!(*rest.fragment(), "rest");
    }
    
    #[test]
    fn smoke_test_backslash_escape_backslash() {
        let input = Span::new("\\\\");
        let result = backslash_escape(input);
        assert!(result.is_ok());
        let (_, ch) = result.unwrap();
        assert_eq!(ch, '\\');
    }
    
    #[test]
    fn smoke_test_backslash_escape_bracket() {
        let input = Span::new("\\[link]");
        let result = backslash_escape(input);
        assert!(result.is_ok());
        let (rest, ch) = result.unwrap();
        assert_eq!(ch, '[');
        assert_eq!(*rest.fragment(), "link]");
    }
    
    #[test]
    fn smoke_test_backslash_no_escape() {
        let input = Span::new("\\a");  // 'a' is not punctuation
        let result = backslash_escape(input);
        assert!(result.is_err());  // Should fail
    }
    
    #[test]
    fn smoke_test_backslash_escape_all_punctuation() {
        let punctuation = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
        for ch in punctuation.chars() {
            let input_str = format!("\\{}", ch);
            let input = Span::new(&input_str);
            let result = backslash_escape(input);
            assert!(result.is_ok(), "Failed to escape '{}'", ch);
            let (_, escaped) = result.unwrap();
            assert_eq!(escaped, ch);
        }
    }
    
    #[test]
    fn smoke_test_no_backslash() {
        let input = Span::new("*text");
        let result = backslash_escape(input);
        assert!(result.is_err());
    }
}
