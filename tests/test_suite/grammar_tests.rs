// Grammar tests: validate nom parsers for block and inline syntax

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct CommonMarkTest {
    example: u32,
    section: Option<String>,
    markdown: String,
    html: String,
    #[serde(rename = "start_line")]
    start_line: Option<u32>,
    #[serde(rename = "end_line")]
    end_line: Option<u32>,
}

fn load_commonmark_tests() -> Vec<CommonMarkTest> {
    let json = include_str!("spec/commonmark.json");
    serde_json::from_str(json).expect("Failed to parse commonmark.json")
}

#[cfg(test)]
mod inline_tests {
    use super::*;
    use core::grammar::inline;
    use nom_locate::LocatedSpan;
    
    type Span<'a> = LocatedSpan<&'a str>;
    
    #[test]
    fn test_code_span_basic() {
        // Example 328: `foo`
        let input = Span::new("`foo`");
        let result = inline::code_span(input);
        assert!(result.is_ok(), "Failed to parse basic code span");
        
        let (remaining, content) = result.unwrap();
        assert_eq!(content.fragment(), &"foo");
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn test_code_span_double_backticks() {
        // Example 329: `` foo ` bar ``
        let input = Span::new("`` foo ` bar ``");
        let result = inline::code_span(input);
        assert!(result.is_ok(), "Failed to parse double backtick code span");
        
        let (remaining, content) = result.unwrap();
        assert_eq!(content.fragment(), &" foo ` bar ");
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn test_code_span_with_whitespace() {
        // Example 333: ` b `
        let input = Span::new("` b `");
        let result = inline::code_span(input);
        assert!(result.is_ok(), "Failed to parse code span with whitespace");
        
        let (remaining, content) = result.unwrap();
        assert_eq!(content.fragment(), &" b ");
    }
    
    #[test]
    fn test_code_span_commonmark_suite() {
        let tests = load_commonmark_tests();
        let code_span_tests: Vec<_> = tests.iter()
            .filter(|t| t.section.as_deref() == Some("Code spans"))
            .take(5) // Test first 5 examples
            .collect();
        
        for test in code_span_tests {
            println!("Testing example {}: {:?}", test.example, test.markdown);
            
            let input = Span::new(&test.markdown.trim_end());
            let result = inline::code_span(input);
            
            // Just verify it parses successfully for now
            assert!(result.is_ok(), "Failed example {}: {}", test.example, test.markdown);
        }
    }
}

#[cfg(test)]
mod block_tests {
    use super::*;
    use core::grammar::block;
    use nom_locate::LocatedSpan;
    
    type Span<'a> = LocatedSpan<&'a str>;
    
    #[test]
    fn test_heading_parse() {
        let input = Span::new("# Hello");
        let result = block::heading(input);
        assert!(result.is_ok());
    }
}
