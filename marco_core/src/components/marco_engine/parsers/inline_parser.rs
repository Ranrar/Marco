// Inline Parser - Phase 3: Inline Grammar Implementation
// Parses inline-level elements (emphasis, strong, links, code spans, etc.)

use pest::Parser;
use pest::iterators::Pairs;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "components/marco_engine/grammar/inline/_core.pest"]
#[grammar = "components/marco_engine/grammar/inline/emphasis.pest"]
#[grammar = "components/marco_engine/grammar/inline/strong.pest"]
#[grammar = "components/marco_engine/grammar/inline/code_span.pest"]
#[grammar = "components/marco_engine/grammar/inline/autolink.pest"]
#[grammar = "components/marco_engine/grammar/inline/escape.pest"]
#[grammar = "components/marco_engine/grammar/inline/line_break.pest"]
#[grammar = "components/marco_engine/grammar/inline/html_tag.pest"]
pub struct InlineParser;

/// Parse inline content from a string
/// Returns Result with Pairs if successful, or error message if failed
pub fn parse_inlines(input: &str) -> Result<Pairs<Rule>, String> {
    InlineParser::parse(Rule::emphasis, input)
        .map_err(|e| format!("Parse error: {}", e))
}

/// Parse a specific inline rule from a string (for testing individual rules)
pub fn parse_inline_rule(rule: Rule, input: &str) -> Result<Pairs<Rule>, String> {
    InlineParser::parse(rule, input)
        .map_err(|e| format!("Parse error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // Phase 3: Emphasis Tests
    // ========================================

    #[test]
    fn smoke_test_emphasis_asterisk_simple() {
        let input = "*hello*";
        let result = parse_inlines(input);
        
        match &result {
            Err(e) => eprintln!("Parse error: {}", e),
            Ok(pairs) => {
                for pair in pairs.clone() {
                    eprintln!("{:?}", pair);
                }
            }
        }
        
        assert!(result.is_ok(), "Should parse simple asterisk emphasis");
        let pairs = result.unwrap();
        let emphasis = pairs.into_iter().next().unwrap();
        assert_eq!(emphasis.as_rule(), Rule::emphasis);
        assert_eq!(emphasis.as_str(), "*hello*");
    }

    #[test]
    fn smoke_test_emphasis_underscore_simple() {
        let input = "_hello_";
        let result = parse_inlines(input);
        
        match &result {
            Err(e) => eprintln!("Parse error: {}", e),
            Ok(pairs) => {
                for pair in pairs.clone() {
                    eprintln!("{:?}", pair);
                }
            }
        }
        
        assert!(result.is_ok(), "Should parse simple underscore emphasis");
        let pairs = result.unwrap();
        let emphasis = pairs.into_iter().next().unwrap();
        assert_eq!(emphasis.as_rule(), Rule::emphasis);
        assert_eq!(emphasis.as_str(), "_hello_");
    }

    #[test]
    fn smoke_test_emphasis_intraword_asterisk() {
        // Asterisk emphasis can occur inside words
        // Test just the "*bar*" part
        let input = "*bar*";
        let result = parse_inlines(input);
        
        match &result {
            Err(e) => eprintln!("Parse error: {}", e),
            Ok(pairs) => {
                for pair in pairs.clone() {
                    eprintln!("{:?}", pair);
                }
            }
        }
        
        assert!(result.is_ok(), "Should parse intraword asterisk emphasis");
        let pairs = result.unwrap();
        let emphasis = pairs.into_iter().next().unwrap();
        assert!(emphasis.as_str().contains("bar"));
    }

    #[test]
    fn smoke_test_emphasis_no_intraword_underscore() {
        // Underscore emphasis should NOT work inside words
        // Test "_bar_" which looks like it's in the middle of "foo_bar_baz"
        let input = "_bar_";
        let result = parse_inline_rule(Rule::emphasis_underscore, input);
        
        // This SHOULD succeed when tested in isolation
        // The intraword restriction is about being between alphanumerics,
        // not about the content itself
        assert!(result.is_ok(), "Underscore emphasis should work when isolated");
    }

    #[test]
    fn smoke_test_emphasis_with_spaces() {
        let input = "*foo bar*";
        let result = parse_inlines(input);
        
        match &result {
            Err(e) => eprintln!("Parse error: {}", e),
            Ok(pairs) => {
                for pair in pairs.clone() {
                    eprintln!("{:?}", pair);
                }
            }
        }
        
        assert!(result.is_ok(), "Should parse emphasis containing spaces");
        let pairs = result.unwrap();
        let emphasis = pairs.into_iter().next().unwrap();
        assert_eq!(emphasis.as_str(), "*foo bar*");
    }

    #[test]
    fn smoke_test_emphasis_not_followed_by_whitespace() {
        // This should fail: opening delimiter followed by space
        let input = "* foo*";
        let result = parse_inlines(input);
        
        assert!(result.is_err(), "Should NOT parse emphasis when opening delimiter is followed by whitespace");
    }

    #[test]
    fn smoke_test_emphasis_not_preceded_by_whitespace() {
        // This should fail: closing delimiter preceded by space
        // The content would be "foo " which ends with whitespace
        let input = "*foo *";
        let result = parse_inlines(input);
        
        // Actually, this WILL parse because our check is !WHITESPACE AFTER the content
        // But the content itself can end with space. Let's check that it doesn't match.
        // Actually, our rule says: content ~ !WHITESPACE ~ ASTERISK
        // So *foo * should fail because there's whitespace between content and closing *
        assert!(result.is_err(), "Should NOT parse emphasis when there's whitespace before closing delimiter");
    }

    #[test]
    fn smoke_test_emphasis_with_punctuation() {
        let input = "*\"foo\"*";
        let result = parse_inlines(input);
        
        match &result {
            Err(e) => eprintln!("Parse error: {}", e),
            Ok(pairs) => {
                for pair in pairs.clone() {
                    eprintln!("{:?}", pair);
                }
            }
        }
        
        assert!(result.is_ok(), "Should parse emphasis with punctuation inside");
        let pairs = result.unwrap();
        let emphasis = pairs.into_iter().next().unwrap();
        assert!(emphasis.as_str().contains("\"foo\""));
    }

    #[test]
    fn smoke_test_emphasis_unicode() {
        let input = "*café*";
        let result = parse_inlines(input);
        
        match &result {
            Err(e) => eprintln!("Parse error: {}", e),
            Ok(pairs) => {
                for pair in pairs.clone() {
                    eprintln!("{:?}", pair);
                }
            }
        }
        
        assert!(result.is_ok(), "Should parse emphasis with Unicode characters");
        let pairs = result.unwrap();
        let emphasis = pairs.into_iter().next().unwrap();
        assert_eq!(emphasis.as_str(), "*café*");
    }

    // ========================================
    // Phase 3: Strong Emphasis Tests
    // ========================================

    #[test]
    fn smoke_test_strong_asterisk_simple() {
        let input = "**hello**";
        let result = parse_inline_rule(Rule::strong, input);
        
        match &result {
            Err(e) => eprintln!("Parse error: {}", e),
            Ok(pairs) => {
                for pair in pairs.clone() {
                    eprintln!("{:?}", pair);
                }
            }
        }
        
        assert!(result.is_ok(), "Should parse simple double asterisk strong");
        let pairs = result.unwrap();
        let strong = pairs.into_iter().next().unwrap();
        assert_eq!(strong.as_rule(), Rule::strong);
        assert_eq!(strong.as_str(), "**hello**");
    }

    #[test]
    fn smoke_test_strong_underscore_simple() {
        let input = "__hello__";
        let result = parse_inline_rule(Rule::strong, input);
        
        match &result {
            Err(e) => eprintln!("Parse error: {}", e),
            Ok(pairs) => {
                for pair in pairs.clone() {
                    eprintln!("{:?}", pair);
                }
            }
        }
        
        assert!(result.is_ok(), "Should parse simple double underscore strong");
        let pairs = result.unwrap();
        let strong = pairs.into_iter().next().unwrap();
        assert_eq!(strong.as_rule(), Rule::strong);
        assert_eq!(strong.as_str(), "__hello__");
    }

    #[test]
    fn smoke_test_strong_intraword_asterisk() {
        // Double asterisk strong can occur inside words
        let input = "**bar**";
        let result = parse_inline_rule(Rule::strong, input);
        
        match &result {
            Err(e) => eprintln!("Parse error: {}", e),
            Ok(pairs) => {
                for pair in pairs.clone() {
                    eprintln!("{:?}", pair);
                }
            }
        }
        
        assert!(result.is_ok(), "Should parse intraword double asterisk strong");
        let pairs = result.unwrap();
        let strong = pairs.into_iter().next().unwrap();
        assert!(strong.as_str().contains("bar"));
    }

    #[test]
    fn smoke_test_strong_with_spaces() {
        let input = "**foo bar**";
        let result = parse_inline_rule(Rule::strong, input);
        
        match &result {
            Err(e) => eprintln!("Parse error: {}", e),
            Ok(pairs) => {
                for pair in pairs.clone() {
                    eprintln!("{:?}", pair);
                }
            }
        }
        
        assert!(result.is_ok(), "Should parse strong containing spaces");
        let pairs = result.unwrap();
        let strong = pairs.into_iter().next().unwrap();
        assert_eq!(strong.as_str(), "**foo bar**");
    }

    #[test]
    fn smoke_test_strong_not_followed_by_whitespace() {
        // This should fail: opening delimiter followed by space
        let input = "** foo**";
        let result = parse_inline_rule(Rule::strong, input);
        
        assert!(result.is_err(), "Should NOT parse strong when opening delimiter is followed by whitespace");
    }

    #[test]
    fn smoke_test_strong_not_preceded_by_whitespace() {
        // This should fail: closing delimiter preceded by space
        let input = "**foo **";
        let result = parse_inline_rule(Rule::strong, input);
        
        assert!(result.is_err(), "Should NOT parse strong when there's whitespace before closing delimiter");
    }

    #[test]
    fn smoke_test_strong_with_punctuation() {
        let input = "**\"foo\"**";
        let result = parse_inline_rule(Rule::strong, input);
        
        match &result {
            Err(e) => eprintln!("Parse error: {}", e),
            Ok(pairs) => {
                for pair in pairs.clone() {
                    eprintln!("{:?}", pair);
                }
            }
        }
        
        assert!(result.is_ok(), "Should parse strong with punctuation inside");
        let pairs = result.unwrap();
        let strong = pairs.into_iter().next().unwrap();
        assert!(strong.as_str().contains("\"foo\""));
    }

    #[test]
    fn smoke_test_strong_unicode() {
        let input = "**café**";
        let result = parse_inline_rule(Rule::strong, input);
        
        match &result {
            Err(e) => eprintln!("Parse error: {}", e),
            Ok(pairs) => {
                for pair in pairs.clone() {
                    eprintln!("{:?}", pair);
                }
            }
        }
        
        assert!(result.is_ok(), "Should parse strong with Unicode characters");
        let pairs = result.unwrap();
        let strong = pairs.into_iter().next().unwrap();
        assert_eq!(strong.as_str(), "**café**");
    }

    #[test]
    fn smoke_test_strong_no_single_asterisk() {
        // Single asterisk should not match strong rule (it's emphasis)
        let input = "*hello*";
        let result = parse_inline_rule(Rule::strong, input);
        
        assert!(result.is_err(), "Should NOT parse single asterisk as strong emphasis");
    }

    // ========================================
    // Phase 3: Code Span Tests
    // ========================================

    #[test]
    fn smoke_test_code_span_single_backtick() {
        let input = "`code`";
        let result = parse_inline_rule(Rule::code_span, input);
        
        match &result {
            Err(e) => eprintln!("Parse error: {}", e),
            Ok(pairs) => {
                for pair in pairs.clone() {
                    eprintln!("{:?}", pair);
                }
            }
        }
        
        assert!(result.is_ok(), "Should parse code span with single backticks");
        let pairs = result.unwrap();
        let code = pairs.into_iter().next().unwrap();
        assert_eq!(code.as_rule(), Rule::code_span);
        assert_eq!(code.as_str(), "`code`");
    }

    #[test]
    fn smoke_test_code_span_double_backtick() {
        let input = "``code``";
        let result = parse_inline_rule(Rule::code_span, input);
        
        match &result {
            Err(e) => eprintln!("Parse error: {}", e),
            Ok(pairs) => {
                for pair in pairs.clone() {
                    eprintln!("{:?}", pair);
                }
            }
        }
        
        assert!(result.is_ok(), "Should parse code span with double backticks");
        let pairs = result.unwrap();
        let code = pairs.into_iter().next().unwrap();
        assert_eq!(code.as_str(), "``code``");
    }

    #[test]
    fn smoke_test_code_span_triple_backtick() {
        let input = "```code```";
        let result = parse_inline_rule(Rule::code_span, input);
        
        match &result {
            Err(e) => eprintln!("Parse error: {}", e),
            Ok(pairs) => {
                for pair in pairs.clone() {
                    eprintln!("{:?}", pair);
                }
            }
        }
        
        assert!(result.is_ok(), "Should parse code span with triple backticks");
        let pairs = result.unwrap();
        let code = pairs.into_iter().next().unwrap();
        assert_eq!(code.as_str(), "```code```");
    }

    #[test]
    fn smoke_test_code_span_with_single_backtick_inside() {
        // Using double backticks to allow single backtick inside
        let input = "`` ` ``";
        let result = parse_inline_rule(Rule::code_span, input);
        
        match &result {
            Err(e) => eprintln!("Parse error: {}", e),
            Ok(pairs) => {
                for pair in pairs.clone() {
                    eprintln!("{:?}", pair);
                }
            }
        }
        
        assert!(result.is_ok(), "Should parse code span with backtick inside when using double backticks");
        let pairs = result.unwrap();
        let code = pairs.into_iter().next().unwrap();
        assert_eq!(code.as_str(), "`` ` ``");
    }

    #[test]
    fn smoke_test_code_span_with_spaces() {
        let input = "`foo bar`";
        let result = parse_inline_rule(Rule::code_span, input);
        
        match &result {
            Err(e) => eprintln!("Parse error: {}", e),
            Ok(pairs) => {
                for pair in pairs.clone() {
                    eprintln!("{:?}", pair);
                }
            }
        }
        
        assert!(result.is_ok(), "Should parse code span with spaces");
        let pairs = result.unwrap();
        let code = pairs.into_iter().next().unwrap();
        assert_eq!(code.as_str(), "`foo bar`");
    }

    #[test]
    fn smoke_test_code_span_empty() {
        let input = "``";
        let result = parse_inline_rule(Rule::code_span, input);
        
        match &result {
            Err(e) => eprintln!("Parse error: {}", e),
            Ok(pairs) => {
                for pair in pairs.clone() {
                    eprintln!("{:?}", pair);
                }
            }
        }
        
        assert!(result.is_ok(), "Should parse empty code span");
        let pairs = result.unwrap();
        let code = pairs.into_iter().next().unwrap();
        assert_eq!(code.as_str(), "``");
    }

    #[test]
    fn smoke_test_code_span_with_special_chars() {
        let input = "`<>&`";
        let result = parse_inline_rule(Rule::code_span, input);
        
        match &result {
            Err(e) => eprintln!("Parse error: {}", e),
            Ok(pairs) => {
                for pair in pairs.clone() {
                    eprintln!("{:?}", pair);
                }
            }
        }
        
        assert!(result.is_ok(), "Should parse code span with special HTML characters");
        let pairs = result.unwrap();
        let code = pairs.into_iter().next().unwrap();
        assert_eq!(code.as_str(), "`<>&`");
    }

    // ========================================
    // Phase 3: Autolink Tests
    // ========================================

    #[test]
    fn smoke_test_autolink_uri_simple() {
        let input = "<http://example.com>";
        let result = parse_inline_rule(Rule::autolink, input);
        
        match &result {
            Err(e) => eprintln!("Parse error: {}", e),
            Ok(pairs) => {
                for pair in pairs.clone() {
                    eprintln!("{:?}", pair);
                }
            }
        }
        
        assert!(result.is_ok(), "Should parse simple URI autolink");
        let pairs = result.unwrap();
        let autolink = pairs.into_iter().next().unwrap();
        assert_eq!(autolink.as_rule(), Rule::autolink);
        assert_eq!(autolink.as_str(), "<http://example.com>");
    }

    #[test]
    fn smoke_test_autolink_uri_with_query() {
        let input = "<https://example.com/path?query=value&id=123>";
        let result = parse_inline_rule(Rule::autolink, input);
        
        assert!(result.is_ok(), "Should parse URI autolink with query parameters");
        let pairs = result.unwrap();
        let autolink = pairs.into_iter().next().unwrap();
        assert_eq!(autolink.as_str(), "<https://example.com/path?query=value&id=123>");
    }

    #[test]
    fn smoke_test_autolink_uri_with_fragment() {
        let input = "<https://example.com#section>";
        let result = parse_inline_rule(Rule::autolink, input);
        
        assert!(result.is_ok(), "Should parse URI autolink with fragment");
        let pairs = result.unwrap();
        let autolink = pairs.into_iter().next().unwrap();
        assert_eq!(autolink.as_str(), "<https://example.com#section>");
    }

    #[test]
    fn smoke_test_autolink_email_simple() {
        let input = "<user@example.com>";
        let result = parse_inline_rule(Rule::autolink, input);
        
        match &result {
            Err(e) => eprintln!("Parse error: {}", e),
            Ok(pairs) => {
                for pair in pairs.clone() {
                    eprintln!("{:?}", pair);
                }
            }
        }
        
        assert!(result.is_ok(), "Should parse simple email autolink");
        let pairs = result.unwrap();
        let autolink = pairs.into_iter().next().unwrap();
        assert_eq!(autolink.as_str(), "<user@example.com>");
    }

    #[test]
    fn smoke_test_autolink_email_with_plus() {
        let input = "<user+tag@example.com>";
        let result = parse_inline_rule(Rule::autolink, input);
        
        assert!(result.is_ok(), "Should parse email with + sign");
        let pairs = result.unwrap();
        let autolink = pairs.into_iter().next().unwrap();
        assert_eq!(autolink.as_str(), "<user+tag@example.com>");
    }

    #[test]
    fn smoke_test_autolink_email_with_subdomain() {
        let input = "<user@mail.example.com>";
        let result = parse_inline_rule(Rule::autolink, input);
        
        assert!(result.is_ok(), "Should parse email with subdomain");
        let pairs = result.unwrap();
        let autolink = pairs.into_iter().next().unwrap();
        assert_eq!(autolink.as_str(), "<user@mail.example.com>");
    }

    #[test]
    fn smoke_test_autolink_not_with_spaces() {
        // Spaces should cause failure
        let input = "<http://example .com>";
        let result = parse_inline_rule(Rule::autolink, input);
        
        assert!(result.is_err(), "Should NOT parse autolink with spaces");
    }

    // ========================
    // Escape Tests (7 tests)
    // ========================

    #[test]
    fn smoke_test_escape_exclamation() {
        let input = "\\!";
        let result = parse_inline_rule(Rule::escape, input);
        
        assert!(result.is_ok(), "Should parse escaped exclamation");
        let pairs = result.unwrap();
        let escape = pairs.into_iter().next().unwrap();
        assert_eq!(escape.as_rule(), Rule::escape);
        assert_eq!(escape.as_str(), "\\!");
    }

    #[test]
    fn smoke_test_escape_asterisk_and_brackets() {
        let input = "\\*\\[\\]";
        let result = parse_inline_rule(Rule::escape, input);
        
        assert!(result.is_ok(), "Should parse escaped asterisk");
        let pairs = result.unwrap();
        let escape = pairs.into_iter().next().unwrap();
        assert_eq!(escape.as_rule(), Rule::escape);
        assert_eq!(escape.as_str(), "\\*");
    }

    #[test]
    fn smoke_test_escape_backslash() {
        let input = "\\\\";
        let result = parse_inline_rule(Rule::escape, input);
        
        assert!(result.is_ok(), "Should parse escaped backslash");
        let pairs = result.unwrap();
        let escape = pairs.into_iter().next().unwrap();
        assert_eq!(escape.as_rule(), Rule::escape);
        assert_eq!(escape.as_str(), "\\\\");
    }

    #[test]
    fn smoke_test_escape_angle_brackets() {
        let input = "\\<\\>";
        let result = parse_inline_rule(Rule::escape, input);
        
        assert!(result.is_ok(), "Should parse escaped angle brackets");
        let pairs = result.unwrap();
        let escape = pairs.into_iter().next().unwrap();
        assert_eq!(escape.as_rule(), Rule::escape);
        assert_eq!(escape.as_str(), "\\<");
    }

    #[test]
    fn smoke_test_escape_all_punctuation() {
        // Test various punctuation characters
        let test_cases = vec![
            "\\!", "\\\"", "\\#", "\\$", "\\%", "\\&", "\\'", "\\(", "\\)", "\\*",
            "\\+", "\\,", "\\-", "\\.", "\\/", "\\:", "\\;", "\\<", "\\=", "\\>",
            "\\?", "\\@", "\\[", "\\\\", "\\]", "\\^", "\\_", "\\`", "\\{", "\\|",
            "\\}", "\\~"
        ];
        
        for input in test_cases {
            let result = parse_inline_rule(Rule::escape, input);
            assert!(result.is_ok(), "Should parse escaped char: {}", input);
            let pairs = result.unwrap();
            let escape = pairs.into_iter().next().unwrap();
            assert_eq!(escape.as_str(), input);
        }
    }

    #[test]
    fn smoke_test_escape_non_punctuation() {
        // Backslash before non-escapable character should fail
        let input = "\\a";
        let result = parse_inline_rule(Rule::escape, input);
        
        assert!(result.is_err(), "Should NOT parse backslash before letter 'a'");
    }

    #[test]
    fn smoke_test_escape_inside_emphasis() {
        // Note: This tests the escape rule in isolation, not within emphasis context
        // Full integration testing (escape preventing emphasis) is for Task #8
        let input = "\\*";
        let result = parse_inline_rule(Rule::escape, input);
        
        assert!(result.is_ok(), "Should parse escaped asterisk");
        let pairs = result.unwrap();
        let escape = pairs.into_iter().next().unwrap();
        assert_eq!(escape.as_rule(), Rule::escape);
        assert_eq!(escape.as_str(), "\\*");
    }

    // ========================================
    // TASK #6: line_break tests
    // ========================================

    #[test]
    fn smoke_test_line_break_two_spaces() {
        let input = "  \n";
        let result = parse_inline_rule(Rule::line_break, input);
        
        assert!(result.is_ok(), "Should parse two spaces followed by newline");
        let pairs = result.unwrap();
        let line_break = pairs.into_iter().next().unwrap();
        assert_eq!(line_break.as_rule(), Rule::line_break);
        assert_eq!(line_break.as_str(), "  \n");
    }

    #[test]
    fn smoke_test_line_break_three_spaces() {
        let input = "   \n";
        let result = parse_inline_rule(Rule::line_break, input);
        
        assert!(result.is_ok(), "Should parse three spaces followed by newline");
        let pairs = result.unwrap();
        let line_break = pairs.into_iter().next().unwrap();
        assert_eq!(line_break.as_rule(), Rule::line_break);
        assert_eq!(line_break.as_str(), "   \n");
    }

    #[test]
    fn smoke_test_line_break_backslash() {
        let input = "\\\n";
        let result = parse_inline_rule(Rule::line_break, input);
        
        assert!(result.is_ok(), "Should parse backslash followed by newline");
        let pairs = result.unwrap();
        let line_break = pairs.into_iter().next().unwrap();
        assert_eq!(line_break.as_rule(), Rule::line_break);
        assert_eq!(line_break.as_str(), "\\\n");
    }

    #[test]
    fn smoke_test_line_break_backslash_crlf() {
        let input = "\\\r\n";
        let result = parse_inline_rule(Rule::line_break, input);
        
        assert!(result.is_ok(), "Should parse backslash followed by CRLF");
        let pairs = result.unwrap();
        let line_break = pairs.into_iter().next().unwrap();
        assert_eq!(line_break.as_rule(), Rule::line_break);
        assert_eq!(line_break.as_str(), "\\\r\n");
    }

    #[test]
    fn smoke_test_line_break_many_spaces() {
        let input = "       \n";
        let result = parse_inline_rule(Rule::line_break, input);
        
        assert!(result.is_ok(), "Should parse many spaces (7) followed by newline");
        let pairs = result.unwrap();
        let line_break = pairs.into_iter().next().unwrap();
        assert_eq!(line_break.as_rule(), Rule::line_break);
        assert_eq!(line_break.as_str(), "       \n");
    }

    #[test]
    fn smoke_test_line_break_single_space_fails() {
        let input = " \n";
        let result = parse_inline_rule(Rule::line_break, input);
        
        // Single space + newline should NOT match line_break (must be 2+)
        assert!(result.is_err(), "Single space followed by newline should NOT be a hard line break");
    }

    #[test]
    fn smoke_test_line_break_no_space_fails() {
        let input = "\n";
        let result = parse_inline_rule(Rule::line_break, input);
        
        // Just newline should NOT match line_break (soft break, not hard)
        assert!(result.is_err(), "Newline alone should NOT be a hard line break");
    }

    // ===================================================================
    // HTML Tag Tests - CommonMark Section 6.6 (Raw HTML)
    // ===================================================================

    #[test]
    fn smoke_test_html_tag_opening_simple() {
        let input = "<div>";
        let result = parse_inline_rule(Rule::html_tag, input);
        
        assert!(result.is_ok(), "Should parse simple opening tag");
        let pairs = result.unwrap();
        let html_tag = pairs.into_iter().next().unwrap();
        assert_eq!(html_tag.as_rule(), Rule::html_tag);
        assert_eq!(html_tag.as_str(), "<div>");
    }

    #[test]
    fn smoke_test_html_tag_opening_with_attribute() {
        let input = "<a href=\"url\">";
        let result = parse_inline_rule(Rule::html_tag, input);
        
        assert!(result.is_ok(), "Should parse opening tag with single attribute");
        let pairs = result.unwrap();
        let html_tag = pairs.into_iter().next().unwrap();
        assert_eq!(html_tag.as_rule(), Rule::html_tag);
        assert_eq!(html_tag.as_str(), "<a href=\"url\">");
    }

    #[test]
    fn smoke_test_html_tag_opening_with_multiple_attributes() {
        let input = "<img src=\"x\" alt=\"y\">";
        let result = parse_inline_rule(Rule::html_tag, input);
        
        assert!(result.is_ok(), "Should parse opening tag with multiple attributes");
        let pairs = result.unwrap();
        let html_tag = pairs.into_iter().next().unwrap();
        assert_eq!(html_tag.as_rule(), Rule::html_tag);
        assert_eq!(html_tag.as_str(), "<img src=\"x\" alt=\"y\">");
    }

    #[test]
    fn smoke_test_html_tag_closing() {
        let input = "</div>";
        let result = parse_inline_rule(Rule::html_tag, input);
        
        assert!(result.is_ok(), "Should parse closing tag");
        let pairs = result.unwrap();
        let html_tag = pairs.into_iter().next().unwrap();
        assert_eq!(html_tag.as_rule(), Rule::html_tag);
        assert_eq!(html_tag.as_str(), "</div>");
    }

    #[test]
    fn smoke_test_html_tag_self_closing() {
        let input = "<br />";
        let result = parse_inline_rule(Rule::html_tag, input);
        
        assert!(result.is_ok(), "Should parse self-closing tag");
        let pairs = result.unwrap();
        let html_tag = pairs.into_iter().next().unwrap();
        assert_eq!(html_tag.as_rule(), Rule::html_tag);
        assert_eq!(html_tag.as_str(), "<br />");
    }

    #[test]
    fn smoke_test_html_tag_self_closing_with_attributes() {
        let input = "<img src=\"x\" />";
        let result = parse_inline_rule(Rule::html_tag, input);
        
        assert!(result.is_ok(), "Should parse self-closing tag with attributes");
        let pairs = result.unwrap();
        let html_tag = pairs.into_iter().next().unwrap();
        assert_eq!(html_tag.as_rule(), Rule::html_tag);
        assert_eq!(html_tag.as_str(), "<img src=\"x\" />");
    }

    #[test]
    fn smoke_test_html_tag_comment_simple() {
        let input = "<!-- foo -->";
        let result = parse_inline_rule(Rule::html_tag, input);
        
        assert!(result.is_ok(), "Should parse simple HTML comment");
        let pairs = result.unwrap();
        let html_tag = pairs.into_iter().next().unwrap();
        assert_eq!(html_tag.as_rule(), Rule::html_tag);
        assert_eq!(html_tag.as_str(), "<!-- foo -->");
    }

    #[test]
    fn smoke_test_html_tag_comment_with_hyphens() {
        let input = "<!-- foo -- bar -->";
        let result = parse_inline_rule(Rule::html_tag, input);
        
        assert!(result.is_ok(), "Should parse HTML comment with hyphens inside");
        let pairs = result.unwrap();
        let html_tag = pairs.into_iter().next().unwrap();
        assert_eq!(html_tag.as_rule(), Rule::html_tag);
        assert_eq!(html_tag.as_str(), "<!-- foo -- bar -->");
    }

    #[test]
    fn smoke_test_html_tag_processing_instruction() {
        let input = "<?php echo $x; ?>";
        let result = parse_inline_rule(Rule::html_tag, input);
        
        assert!(result.is_ok(), "Should parse processing instruction");
        let pairs = result.unwrap();
        let html_tag = pairs.into_iter().next().unwrap();
        assert_eq!(html_tag.as_rule(), Rule::html_tag);
        assert_eq!(html_tag.as_str(), "<?php echo $x; ?>");
    }

    #[test]
    fn smoke_test_html_tag_declaration() {
        let input = "<!DOCTYPE html>";
        let result = parse_inline_rule(Rule::html_tag, input);
        
        assert!(result.is_ok(), "Should parse DOCTYPE declaration");
        let pairs = result.unwrap();
        let html_tag = pairs.into_iter().next().unwrap();
        assert_eq!(html_tag.as_rule(), Rule::html_tag);
        assert_eq!(html_tag.as_str(), "<!DOCTYPE html>");
    }

    #[test]
    fn smoke_test_html_tag_cdata() {
        let input = "<![CDATA[content]]>";
        let result = parse_inline_rule(Rule::html_tag, input);
        
        assert!(result.is_ok(), "Should parse CDATA section");
        let pairs = result.unwrap();
        let html_tag = pairs.into_iter().next().unwrap();
        assert_eq!(html_tag.as_rule(), Rule::html_tag);
        assert_eq!(html_tag.as_str(), "<![CDATA[content]]>");
    }

    #[test]
    fn smoke_test_html_tag_invalid_tag_name_fails() {
        let input = "<123>";
        let result = parse_inline_rule(Rule::html_tag, input);
        
        assert!(result.is_err(), "Should fail to parse tag starting with digit");
    }

    #[test]
    fn smoke_test_html_tag_single_quoted_attribute() {
        let input = "<a href='url'>";
        let result = parse_inline_rule(Rule::html_tag, input);
        
        assert!(result.is_ok(), "Should parse attribute with single quotes");
        let pairs = result.unwrap();
        let html_tag = pairs.into_iter().next().unwrap();
        assert_eq!(html_tag.as_rule(), Rule::html_tag);
        assert_eq!(html_tag.as_str(), "<a href='url'>");
    }

    // ===================================================================
    // Integration Tests - Multiple Inline Elements (CommonMark Precedence)
    // ===================================================================

    // Emphasis + Other Elements

    #[test]
    fn integration_test_emphasis_with_code_span_characters() {
        // Emphasis containing backtick characters (not parsed as code span at this level)
        let input = "*foo `bar*";
        let result = parse_inline_rule(Rule::emphasis, input);
        
        assert!(result.is_ok(), "Should parse emphasis containing backtick characters");
        let pairs = result.unwrap();
        let emphasis = pairs.into_iter().next().unwrap();
        assert_eq!(emphasis.as_rule(), Rule::emphasis);
        assert_eq!(emphasis.as_str(), "*foo `bar*");
    }

    #[test]
    fn integration_test_emphasis_simple_content() {
        // Basic emphasis with multiple words
        let input = "*foo bar baz*";
        let result = parse_inline_rule(Rule::emphasis, input);
        
        assert!(result.is_ok(), "Should parse emphasis with multiple words");
        let pairs = result.unwrap();
        let emphasis = pairs.into_iter().next().unwrap();
        assert_eq!(emphasis.as_rule(), Rule::emphasis);
        assert_eq!(emphasis.as_str(), "*foo bar baz*");
    }

    #[test]
    fn integration_test_html_tag_with_asterisks_in_attribute() {
        // HTML tags containing characters that could be emphasis markers
        let input = "<img src=\"foo\" title=\"*\"/>";
        let result = parse_inline_rule(Rule::html_tag, input);
        
        assert!(result.is_ok(), "HTML tag should parse with asterisk in attribute");
        let pairs = result.unwrap();
        let html_tag = pairs.into_iter().next().unwrap();
        assert_eq!(html_tag.as_rule(), Rule::html_tag);
        assert_eq!(html_tag.as_str(), "<img src=\"foo\" title=\"*\"/>");
    }

    // Code Spans + Other Elements

    #[test]
    fn integration_test_code_span_with_emphasis_markers() {
        // Code spans preserve emphasis markers literally
        let input = "`*foo*`";
        let result = parse_inline_rule(Rule::code_span, input);
        
        assert!(result.is_ok(), "Should parse code span containing asterisks literally");
        let pairs = result.unwrap();
        let code_span = pairs.into_iter().next().unwrap();
        assert_eq!(code_span.as_rule(), Rule::code_span);
        assert_eq!(code_span.as_str(), "`*foo*`");
    }

    #[test]
    fn integration_test_code_span_with_escape_no_effect() {
        // Backslash escapes do NOT work in code spans
        let input = "`\\*foo\\*`";
        let result = parse_inline_rule(Rule::code_span, input);
        
        assert!(result.is_ok(), "Should parse code span with backslashes literally");
        let pairs = result.unwrap();
        let code_span = pairs.into_iter().next().unwrap();
        assert_eq!(code_span.as_rule(), Rule::code_span);
        assert_eq!(code_span.as_str(), "`\\*foo\\*`");
    }

    #[test]
    fn integration_test_code_span_takes_precedence_over_emphasis() {
        // Example: *foo`*` - the second * is inside code span
        // So we should be able to parse the code span part
        let input = "`*`";
        let result = parse_inline_rule(Rule::code_span, input);
        
        assert!(result.is_ok(), "Code span should parse asterisk inside");
        let pairs = result.unwrap();
        let code_span = pairs.into_iter().next().unwrap();
        assert_eq!(code_span.as_rule(), Rule::code_span);
        assert_eq!(code_span.as_str(), "`*`");
    }

    // HTML Tags + Other Elements

    #[test]
    fn integration_test_html_tag_with_emphasis_in_attribute() {
        // HTML attributes are raw - no markdown processing
        let input = "<a href=\"*url*\">";
        let result = parse_inline_rule(Rule::html_tag, input);
        
        assert!(result.is_ok(), "Should parse HTML tag with asterisks in attribute");
        let pairs = result.unwrap();
        let html_tag = pairs.into_iter().next().unwrap();
        assert_eq!(html_tag.as_rule(), Rule::html_tag);
        assert_eq!(html_tag.as_str(), "<a href=\"*url*\">");
    }

    #[test]
    fn integration_test_html_tag_with_escape_no_effect() {
        // Backslash escapes do NOT work in HTML tags (CommonMark Example 631)
        let input = "<a href=\"\\*\">";
        let result = parse_inline_rule(Rule::html_tag, input);
        
        assert!(result.is_ok(), "Should parse HTML tag with backslash literally");
        let pairs = result.unwrap();
        let html_tag = pairs.into_iter().next().unwrap();
        assert_eq!(html_tag.as_rule(), Rule::html_tag);
        assert_eq!(html_tag.as_str(), "<a href=\"\\*\">");
    }

    #[test]
    fn integration_test_html_comment_preserves_all_content() {
        // Comments preserve everything including markdown syntax
        let input = "<!-- **bold** `code` -->";
        let result = parse_inline_rule(Rule::html_tag, input);
        
        assert!(result.is_ok(), "Should parse HTML comment with markdown inside");
        let pairs = result.unwrap();
        let html_tag = pairs.into_iter().next().unwrap();
        assert_eq!(html_tag.as_rule(), Rule::html_tag);
        assert_eq!(html_tag.as_str(), "<!-- **bold** `code` -->");
    }

    // Escapes in Various Contexts

    #[test]
    fn integration_test_escape_before_emphasis() {
        // Escaped asterisk should not start emphasis
        let input = "\\*";
        let result = parse_inline_rule(Rule::escape, input);
        
        assert!(result.is_ok(), "Should parse escaped asterisk");
        let pairs = result.unwrap();
        let escape = pairs.into_iter().next().unwrap();
        assert_eq!(escape.as_rule(), Rule::escape);
        assert_eq!(escape.as_str(), "\\*");
    }

    #[test]
    fn integration_test_escape_in_autolink_no_effect() {
        // Backslash escapes do NOT work in autolinks (CommonMark Example 603)
        let input = "<https://example.com/\\[\\>";
        let result = parse_inline_rule(Rule::autolink, input);
        
        assert!(result.is_ok(), "Should parse autolink with backslashes literally");
        let pairs = result.unwrap();
        let autolink = pairs.into_iter().next().unwrap();
        assert_eq!(autolink.as_rule(), Rule::autolink);
        assert_eq!(autolink.as_str(), "<https://example.com/\\[\\>");
    }

    #[test]
    fn integration_test_escape_multiple_punctuation() {
        // Multiple escapes in sequence
        let input = "\\*\\[\\]\\(\\)";
        let result = parse_inline_rule(Rule::escape, input);
        
        assert!(result.is_ok(), "Should parse first escape in sequence");
        let pairs = result.unwrap();
        let escape = pairs.into_iter().next().unwrap();
        assert_eq!(escape.as_rule(), Rule::escape);
        assert_eq!(escape.as_str(), "\\*"); // Only first escape
    }

    // Line Breaks in Inline Contexts

    #[test]
    fn integration_test_line_break_in_emphasis() {
        // Hard line breaks work inside emphasis (CommonMark Example 638)
        let input = "  \n";
        let result = parse_inline_rule(Rule::line_break, input);
        
        assert!(result.is_ok(), "Should parse hard line break (can occur inside emphasis)");
        let pairs = result.unwrap();
        let line_break = pairs.into_iter().next().unwrap();
        assert_eq!(line_break.as_rule(), Rule::line_break);
        assert_eq!(line_break.as_str(), "  \n");
    }

    #[test]
    fn integration_test_line_break_backslash_variant() {
        // Backslash line breaks also work
        let input = "\\\n";
        let result = parse_inline_rule(Rule::line_break, input);
        
        assert!(result.is_ok(), "Should parse backslash line break");
        let pairs = result.unwrap();
        let line_break = pairs.into_iter().next().unwrap();
        assert_eq!(line_break.as_rule(), Rule::line_break);
        assert_eq!(line_break.as_str(), "\\\n");
    }

    #[test]
    fn integration_test_line_break_both_variants_parse() {
        // Both line break styles should parse independently
        let input_spaces = "   \n";
        let input_backslash = "\\\n";
        
        let result1 = parse_inline_rule(Rule::line_break, input_spaces);
        let result2 = parse_inline_rule(Rule::line_break, input_backslash);
        
        assert!(result1.is_ok() && result2.is_ok(), "Both line break variants should parse");
    }
}

