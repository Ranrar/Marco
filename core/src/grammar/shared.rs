// Shared types and helper functions for grammar modules

use nom::IResult;
use nom_locate::LocatedSpan;

/// Span type used throughout grammar modules
/// Wraps a string slice with location information for error reporting
pub type Span<'a> = LocatedSpan<&'a str>;

/// Count spaces considering tab expansion (1 tab = 4 spaces)
/// Returns the number of effective space characters for indentation
/// 
/// # Examples
/// ```
/// use core::grammar::shared::count_indentation;
/// 
/// assert_eq!(count_indentation("    text"), 4);
/// assert_eq!(count_indentation("\ttext"), 4);
/// assert_eq!(count_indentation(" \ttext"), 4);
/// ```
pub fn count_indentation(input: &str) -> usize {
    let mut spaces = 0;
    for ch in input.chars() {
        match ch {
            ' ' => spaces += 1,
            '\t' => spaces += 4 - (spaces % 4), // Expand to next tab stop
            _ => break,
        }
    }
    spaces
}

/// Skip indentation characters (spaces and tabs) up to a certain number of effective spaces
/// Returns the remaining input and the number of spaces actually skipped
/// 
/// # Arguments
/// * `input` - The input span to process
/// * `max_spaces` - Maximum number of effective spaces to skip
/// 
/// # Returns
/// `Ok((remaining, spaces_skipped))` if successful, error otherwise
pub fn skip_indentation(input: Span, max_spaces: usize) -> IResult<Span, usize> {
    let mut spaces = 0;
    let mut bytes = 0;
    
    for ch in input.fragment().chars() {
        if spaces >= max_spaces {
            break;
        }
        match ch {
            ' ' => {
                spaces += 1;
                bytes += 1;
            }
            '\t' => {
                let tab_width = 4 - (spaces % 4);
                if spaces + tab_width <= max_spaces {
                    spaces += tab_width;
                    bytes += 1;
                } else {
                    // Tab would exceed max, stop here
                    break;
                }
            }
            _ => break,
        }
    }
    
    if bytes == 0 {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Space)));
    }
    
    // Use nom's `take` combinator to skip bytes while preserving location information
    use nom::bytes::complete::take;
    let (remaining, _skipped) = take(bytes as u32)(input)?;
    Ok((remaining, spaces))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_count_indentation() {
        assert_eq!(count_indentation("    text"), 4);
        assert_eq!(count_indentation("\ttext"), 4);
        assert_eq!(count_indentation(" \ttext"), 4);
        assert_eq!(count_indentation("  \ttext"), 4);
        assert_eq!(count_indentation("   \ttext"), 4);
        assert_eq!(count_indentation("text"), 0);
        assert_eq!(count_indentation(""), 0);
    }

    #[test]
    fn smoke_test_skip_indentation() {
        let input = Span::new("    text");
        let result = skip_indentation(input, 4);
        assert!(result.is_ok());
        let (remaining, spaces) = result.unwrap();
        assert_eq!(spaces, 4);
        assert_eq!(*remaining.fragment(), "text");
    }

    #[test]
    fn smoke_test_skip_indentation_with_tab() {
        let input = Span::new("\ttext");
        let result = skip_indentation(input, 4);
        assert!(result.is_ok());
        let (remaining, spaces) = result.unwrap();
        assert_eq!(spaces, 4);
        assert_eq!(*remaining.fragment(), "text");
    }

    #[test]
    fn smoke_test_skip_indentation_partial() {
        let input = Span::new("      text");
        let result = skip_indentation(input, 4);
        assert!(result.is_ok());
        let (remaining, spaces) = result.unwrap();
        assert_eq!(spaces, 4);
        assert_eq!(*remaining.fragment(), "  text");
    }
}
