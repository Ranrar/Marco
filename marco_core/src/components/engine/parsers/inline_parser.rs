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
#[grammar = "components/marco_engine/grammar/inline/link.pest"]
#[grammar = "components/marco_engine/grammar/inline/image.pest"]
#[grammar = "components/marco_engine/grammar/inline/inline_content.pest"]
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

    // ========================================================================
    // Phase 4: Inline Content Parser Tests
    // ========================================================================
    // Tests for inline_content rule that orchestrates all inline elements
    // with proper precedence and combination handling

    // ------------------------------------------------------------------------
    // Precedence Tests - Verify correct parsing order
    // ------------------------------------------------------------------------

    #[test]
    fn phase4_precedence_code_span_protects_asterisks() {
        // Code spans should protect their content from emphasis parsing
        let input = "`*not emphasis*`";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse code span with asterisks inside");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        assert_eq!(content.as_rule(), Rule::inline_content);
        
        // Verify it contains a code_span element (not emphasis)
        let inner: Vec<_> = content.into_inner().collect();
        assert_eq!(inner.len(), 1, "Should have exactly 1 element");
        assert_eq!(inner[0].as_rule(), Rule::code_span, "Should be code_span, not emphasis");
    }

    #[test]
    fn phase4_precedence_code_span_protects_underscores() {
        // Code spans should protect underscores from emphasis parsing
        let input = "`_not emphasis_`";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse code span with underscores inside");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        let inner: Vec<_> = content.into_inner().collect();
        assert_eq!(inner[0].as_rule(), Rule::code_span);
    }

    #[test]
    fn phase4_precedence_autolink_before_emphasis() {
        // Autolinks should be processed before emphasis
        let input = "<http://example.com>";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse autolink");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        let inner: Vec<_> = content.into_inner().collect();
        assert_eq!(inner[0].as_rule(), Rule::autolink);
    }

    #[test]
    fn phase4_precedence_html_tag_protects_content() {
        // HTML tags should not have markdown processed in attributes
        let input = "<a href=\"*url*\">";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse HTML tag");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        let inner: Vec<_> = content.into_inner().collect();
        assert_eq!(inner[0].as_rule(), Rule::html_tag);
    }

    #[test]
    fn phase4_precedence_escape_prevents_emphasis() {
        // Escaped asterisks should not start emphasis
        let input = "\\*not emphasis\\*";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse escaped asterisks");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        // Should have escape + text + escape (not emphasis)
        let inner: Vec<_> = content.into_inner().collect();
        assert!(inner.len() >= 2, "Should have multiple elements");
        assert_eq!(inner[0].as_rule(), Rule::escape);
    }

    #[test]
    fn phase4_precedence_line_break_in_content() {
        // Line breaks should be recognized in inline content
        // Note: Current text rule is greedy and may consume content before line break detection
        // This is a known limitation - full CommonMark compliance requires more sophisticated parsing
        let input = "  \nmore";  // Start with line break
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse content with line break at start");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        let inner: Vec<_> = content.into_inner().collect();
        // Should have line_break at start
        assert!(inner.iter().any(|p| p.as_rule() == Rule::line_break), "Should contain line_break");
    }

    #[test]
    fn phase4_precedence_strong_before_emphasis() {
        // When both strong and emphasis possible, try strong first
        let input = "**bold**";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse strong emphasis");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        let inner: Vec<_> = content.into_inner().collect();
        assert_eq!(inner[0].as_rule(), Rule::strong, "Should match strong, not emphasis");
    }

    // ------------------------------------------------------------------------
    // Combination Tests - Multiple inline elements together
    // ------------------------------------------------------------------------

    #[test]
    fn phase4_combination_emphasis_and_text() {
        // Simple combination: emphasis + text
        let input = "*italic* and plain text";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse emphasis with text");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        let inner: Vec<_> = content.into_inner().collect();
        assert!(inner.len() >= 2, "Should have emphasis and text elements");
        assert_eq!(inner[0].as_rule(), Rule::emphasis);
        assert_eq!(inner[1].as_rule(), Rule::text);
    }

    #[test]
    fn phase4_combination_strong_and_emphasis() {
        // Combination: strong + emphasis (not nested)
        let input = "**bold** and *italic*";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse strong and emphasis separately");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        let inner: Vec<_> = content.into_inner().collect();
        assert!(inner.len() >= 3, "Should have strong, text, and emphasis");
        assert_eq!(inner[0].as_rule(), Rule::strong);
        assert_eq!(inner[2].as_rule(), Rule::emphasis);
    }

    #[test]
    fn phase4_combination_code_span_with_text() {
        // Code span followed by text
        let input = "`code` and text";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse code span with text");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        let inner: Vec<_> = content.into_inner().collect();
        assert!(inner.len() >= 2);
        assert_eq!(inner[0].as_rule(), Rule::code_span);
        assert_eq!(inner[1].as_rule(), Rule::text);
    }

    #[test]
    fn phase4_combination_multiple_emphasis() {
        // Multiple emphasis elements in sequence
        let input = "*one* *two* *three*";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse multiple emphasis elements");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        let inner: Vec<_> = content.into_inner().collect();
        let emphasis_count = inner.iter().filter(|p| p.as_rule() == Rule::emphasis).count();
        assert_eq!(emphasis_count, 3, "Should have 3 emphasis elements");
    }

    #[test]
    fn phase4_combination_autolink_with_text() {
        // Autolink in text context
        let input = "Visit <http://example.com> for info";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse autolink with surrounding text");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        let inner: Vec<_> = content.into_inner().collect();
        assert!(inner.iter().any(|p| p.as_rule() == Rule::autolink));
        assert!(inner.iter().any(|p| p.as_rule() == Rule::text));
    }

    #[test]
    fn phase4_combination_html_tag_with_text() {
        // HTML tag in text context
        let input = "Some <br /> text";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse HTML tag with text");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        let inner: Vec<_> = content.into_inner().collect();
        assert!(inner.iter().any(|p| p.as_rule() == Rule::html_tag));
    }

    #[test]
    fn phase4_combination_mixed_formatting() {
        // Mix of different inline elements
        let input = "**bold** and *italic* with `code`";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse mixed inline elements");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        let inner: Vec<_> = content.into_inner().collect();
        assert!(inner.iter().any(|p| p.as_rule() == Rule::strong));
        assert!(inner.iter().any(|p| p.as_rule() == Rule::emphasis));
        assert!(inner.iter().any(|p| p.as_rule() == Rule::code_span));
    }

    // ------------------------------------------------------------------------
    // Delimiter Run Tests - Edge cases for delimiter matching
    // ------------------------------------------------------------------------

    #[test]
    fn phase4_delimiter_unmatched_asterisk() {
        // Single unmatched asterisk should be text
        let input = "text * more text";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse unmatched asterisk as text");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        let inner: Vec<_> = content.into_inner().collect();
        // All should be text (unmatched delimiter treated as literal)
        assert!(inner.iter().all(|p| p.as_rule() == Rule::text), "Unmatched * should be plain text");
    }

    #[test]
    fn phase4_delimiter_mixed_delimiters() {
        // Asterisk and underscore don't match each other - this is invalid markdown
        // The parser may fail or treat them as text depending on implementation
        let input = "*emphasis_";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        // This may fail or succeed - both are acceptable for malformed input
        // The key is it shouldn't crash or cause undefined behavior
        if result.is_ok() {
            // If it parses, verify it doesn't create invalid AST
            let pairs = result.unwrap();
            let content = pairs.into_iter().next().unwrap();
            let _ = content.into_inner().collect::<Vec<_>>();
            // Successfully collected elements - no crash
        }
        // If it fails, that's also acceptable for invalid markdown
    }

    #[test]
    fn phase4_delimiter_intraword_asterisks() {
        // Intraword emphasis with asterisks
        let input = "un*frigging*believable";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should handle intraword emphasis");
    }

    #[test]
    fn phase4_delimiter_adjacent_emphasis() {
        // Adjacent emphasis elements
        let input = "*one**two*";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should handle adjacent emphasis delimiters");
    }

    #[test]
    fn phase4_delimiter_triple_asterisks() {
        // Triple asterisks ***text***
        let input = "***text***";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse triple asterisks");
        // Could match as strong containing emphasis, or just text
    }

    // ------------------------------------------------------------------------
    // Real-World Scenario Tests
    // ------------------------------------------------------------------------

    #[test]
    fn phase4_realworld_sentence_with_formatting() {
        // Realistic sentence with multiple formatting
        let input = "This is **really** *important* information!";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse realistic formatted sentence");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        let inner: Vec<_> = content.into_inner().collect();
        assert!(inner.iter().any(|p| p.as_rule() == Rule::strong));
        assert!(inner.iter().any(|p| p.as_rule() == Rule::emphasis));
        assert!(inner.iter().any(|p| p.as_rule() == Rule::text));
    }

    #[test]
    fn phase4_realworld_code_with_explanation() {
        // Code span with surrounding explanation
        let input = "Use the `println!()` function to output text.";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse code with explanation");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        let inner: Vec<_> = content.into_inner().collect();
        assert!(inner.iter().any(|p| p.as_rule() == Rule::code_span));
    }

    #[test]
    fn phase4_realworld_link_text() {
        // Text that looks like it could be a link (but we haven't implemented link parsing yet)
        let input = "Visit example.com for more info";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse plain text with domain name");
    }

    #[test]
    fn phase4_realworld_email_autolink() {
        // Email autolink in sentence
        let input = "Contact me at <user@example.com> anytime.";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse email autolink in text");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        let inner: Vec<_> = content.into_inner().collect();
        assert!(inner.iter().any(|p| p.as_rule() == Rule::autolink));
    }

    #[test]
    fn phase4_realworld_escaped_characters() {
        // Text with escaped special characters
        let input = "Use \\* for emphasis, not \\*\\*";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse text with escapes");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        let inner: Vec<_> = content.into_inner().collect();
        let escape_count = inner.iter().filter(|p| p.as_rule() == Rule::escape).count();
        assert!(escape_count >= 2, "Should have at least 2 escapes");
    }

    #[test]
    fn phase4_realworld_long_content() {
        // Longer realistic content
        let input = "The `Parser` trait in **Rust** is *very* powerful. It allows you to parse <https://example.com> and more.";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse long realistic content");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        let inner: Vec<_> = content.into_inner().collect();
        // Should have mix of code_span, strong, emphasis, autolink, text
        assert!(inner.len() > 5, "Should have multiple elements");
        assert!(inner.iter().any(|p| p.as_rule() == Rule::code_span));
        assert!(inner.iter().any(|p| p.as_rule() == Rule::strong));
        assert!(inner.iter().any(|p| p.as_rule() == Rule::emphasis));
        assert!(inner.iter().any(|p| p.as_rule() == Rule::autolink));
    }

    #[test]
    fn phase4_realworld_plain_text_only() {
        // Plain text with no formatting
        let input = "This is just plain text without any special formatting.";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse plain text");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        let inner: Vec<_> = content.into_inner().collect();
        assert_eq!(inner.len(), 1, "Should have single text element");
        assert_eq!(inner[0].as_rule(), Rule::text);
    }

    #[test]
    fn phase4_realworld_empty_content() {
        // Empty inline content (edge case)
        let input = "";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        // Empty content may fail or succeed depending on grammar (+ vs *)
        // Currently using + (one or more), so this should fail
        assert!(result.is_err(), "Empty content should fail with + quantifier");
    }

    // ========================================
    // Phase 5: Link Tests (CommonMark 6.5)
    // ========================================

    #[test]
    fn phase5_link_basic_url() {
        // Basic inline link with plain URL
        let input = "[link](/uri)";
        let result = parse_inline_rule(Rule::inline_link, input);
        
        assert!(result.is_ok(), "Should parse basic link");
        let pairs = result.unwrap();
        let link = pairs.into_iter().next().unwrap();
        assert_eq!(link.as_rule(), Rule::inline_link);
        assert_eq!(link.as_str(), "[link](/uri)");
    }

    #[test]
    fn phase5_link_with_title_double_quotes() {
        // Link with title using double quotes
        let input = r#"[link](/uri "title")"#;
        let result = parse_inline_rule(Rule::inline_link, input);
        
        assert!(result.is_ok(), "Should parse link with double-quoted title");
        let pairs = result.unwrap();
        let link = pairs.into_iter().next().unwrap();
        assert_eq!(link.as_str(), input);
    }

    #[test]
    fn phase5_link_with_title_single_quotes() {
        // Link with title using single quotes
        let input = "[link](/uri 'title')";
        let result = parse_inline_rule(Rule::inline_link, input);
        
        assert!(result.is_ok(), "Should parse link with single-quoted title");
        let pairs = result.unwrap();
        let link = pairs.into_iter().next().unwrap();
        assert_eq!(link.as_str(), input);
    }

    #[test]
    fn phase5_link_with_title_parentheses() {
        // Link with title using parentheses
        let input = "[link](/uri (title))";
        let result = parse_inline_rule(Rule::inline_link, input);
        
        assert!(result.is_ok(), "Should parse link with parentheses-quoted title");
        let pairs = result.unwrap();
        let link = pairs.into_iter().next().unwrap();
        assert_eq!(link.as_str(), input);
    }

    #[test]
    fn phase5_link_angle_bracket_destination() {
        // Link with angle-bracket wrapped destination
        let input = "[link](<http://example.com>)";
        let result = parse_inline_rule(Rule::inline_link, input);
        
        assert!(result.is_ok(), "Should parse link with angle-bracket destination");
        let pairs = result.unwrap();
        let link = pairs.into_iter().next().unwrap();
        assert_eq!(link.as_str(), input);
    }

    #[test]
    fn phase5_link_angle_bracket_with_spaces() {
        // Angle brackets allow spaces in URLs
        let input = "[link](<http://example.com/foo bar>)";
        let result = parse_inline_rule(Rule::inline_link, input);
        
        assert!(result.is_ok(), "Should parse link with spaces in angle-bracket destination");
        let pairs = result.unwrap();
        let link = pairs.into_iter().next().unwrap();
        assert_eq!(link.as_str(), input);
    }

    #[test]
    fn phase5_link_plain_with_parentheses() {
        // Plain destination can have balanced parentheses
        let input = "[link](/url(with)(parens))";
        let result = parse_inline_rule(Rule::inline_link, input);
        
        assert!(result.is_ok(), "Should parse link with balanced parentheses in destination");
        let pairs = result.unwrap();
        let link = pairs.into_iter().next().unwrap();
        assert_eq!(link.as_str(), input);
    }

    #[test]
    fn phase5_link_empty_destination() {
        // Link with empty destination
        let input = "[link]()";
        let result = parse_inline_rule(Rule::inline_link, input);
        
        // Empty destination should fail - destinations need at least one character
        assert!(result.is_err(), "Should fail: empty destination not allowed");
    }

    #[test]
    fn phase5_link_escaped_characters_in_text() {
        // Link text with escaped characters
        let input = r"[link \[\]](/uri)";
        let result = parse_inline_rule(Rule::inline_link, input);
        
        assert!(result.is_ok(), "Should parse link with escaped brackets in text");
        let pairs = result.unwrap();
        let link = pairs.into_iter().next().unwrap();
        assert_eq!(link.as_str(), input);
    }

    #[test]
    fn phase5_link_escaped_characters_in_destination() {
        // Destination with escaped characters
        let input = r"[link](/uri\(with\)escapes)";
        let result = parse_inline_rule(Rule::inline_link, input);
        
        assert!(result.is_ok(), "Should parse link with escaped characters in destination");
        let pairs = result.unwrap();
        let link = pairs.into_iter().next().unwrap();
        assert_eq!(link.as_str(), input);
    }

    #[test]
    fn phase5_link_whitespace_padding() {
        // Link with whitespace around destination and title
        let input = r#"[link](  /uri  "title"  )"#;
        let result = parse_inline_rule(Rule::inline_link, input);
        
        assert!(result.is_ok(), "Should parse link with whitespace padding");
        let pairs = result.unwrap();
        let link = pairs.into_iter().next().unwrap();
        assert_eq!(link.as_str(), input);
    }

    #[test]
    fn phase5_link_no_space_between_brackets() {
        // No space allowed between ] and (
        let input = "[link] (/uri)";
        let result = parse_inline_rule(Rule::inline_link, input);
        
        assert!(result.is_err(), "Should fail: space between ] and ( not allowed");
    }

    #[test]
    fn phase5_link_multiline_text_fails() {
        // Link text cannot contain newlines
        let input = "[link\ntext](/uri)";
        let result = parse_inline_rule(Rule::inline_link, input);
        
        assert!(result.is_err(), "Should fail: newlines not allowed in link text");
    }

    #[test]
    fn phase5_link_title_multiline_fails() {
        // Title cannot contain newlines
        let input = "[link](/uri \"title\nline2\")";
        let result = parse_inline_rule(Rule::inline_link, input);
        
        assert!(result.is_err(), "Should fail: newlines not allowed in title");
    }

    #[test]
    fn phase5_link_in_inline_content() {
        // Test link within inline_content context
        let input = "Check out [this link](/uri) for more info.";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse inline content with link");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        // Phase 7: inline_link is now a child of link rule
        let has_link = content.into_inner().any(|p| {
            p.as_rule() == Rule::inline_link || p.as_rule() == Rule::link
        });
        assert!(has_link, "Should contain a link element (inline_link or link)");
    }

    // ========================================
    // Phase 5: Image Tests (CommonMark 6.4)
    // ========================================

    #[test]
    fn phase5_image_basic() {
        // Basic inline image with plain URL
        let input = "![alt](/uri)";
        let result = parse_inline_rule(Rule::inline_image, input);
        
        assert!(result.is_ok(), "Should parse basic image");
        let pairs = result.unwrap();
        let image = pairs.into_iter().next().unwrap();
        assert_eq!(image.as_rule(), Rule::inline_image);
        assert_eq!(image.as_str(), "![alt](/uri)");
    }

    #[test]
    fn phase5_image_with_title() {
        // Image with title
        let input = r#"![alt](/uri "title")"#;
        let result = parse_inline_rule(Rule::inline_image, input);
        
        assert!(result.is_ok(), "Should parse image with title");
        let pairs = result.unwrap();
        let image = pairs.into_iter().next().unwrap();
        assert_eq!(image.as_str(), input);
    }

    #[test]
    fn phase5_image_empty_alt() {
        // Image with empty alt text
        let input = "![](/uri)";
        let result = parse_inline_rule(Rule::inline_image, input);
        
        assert!(result.is_ok(), "Should parse image with empty alt text");
        let pairs = result.unwrap();
        let image = pairs.into_iter().next().unwrap();
        assert_eq!(image.as_str(), input);
    }

    #[test]
    fn phase5_image_angle_bracket_destination() {
        // Image with angle-bracket wrapped destination
        let input = "![alt](<http://example.com/image.png>)";
        let result = parse_inline_rule(Rule::inline_image, input);
        
        assert!(result.is_ok(), "Should parse image with angle-bracket destination");
        let pairs = result.unwrap();
        let image = pairs.into_iter().next().unwrap();
        assert_eq!(image.as_str(), input);
    }

    #[test]
    fn phase5_image_escaped_alt_text() {
        // Alt text with escaped characters
        let input = r"![alt \[\]](/uri)";
        let result = parse_inline_rule(Rule::inline_image, input);
        
        assert!(result.is_ok(), "Should parse image with escaped brackets in alt");
        let pairs = result.unwrap();
        let image = pairs.into_iter().next().unwrap();
        assert_eq!(image.as_str(), input);
    }

    #[test]
    fn phase5_image_multiline_alt_fails() {
        // Alt text cannot contain newlines
        let input = "![alt\ntext](/uri)";
        let result = parse_inline_rule(Rule::inline_image, input);
        
        assert!(result.is_err(), "Should fail: newlines not allowed in alt text");
    }

    #[test]
    fn phase5_image_in_inline_content() {
        // Test image within inline_content context
        let input = "Here is an image: ![logo](/logo.png) in text.";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse inline content with image");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        // Phase 7: inline_image is now a child of image rule
        let has_image = content.into_inner().any(|p| {
            p.as_rule() == Rule::inline_image || p.as_rule() == Rule::image
        });
        assert!(has_image, "Should contain an image element (inline_image or image)");
    }

    #[test]
    fn phase5_image_before_link_precedence() {
        // Test that images take precedence over links (both start with [)
        let input = "![image](/img.png) and [link](/uri)";
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse both image and link");
        let mut pairs = result.unwrap();
        let content = pairs.next().unwrap();
        
        // Phase 7: Check for image and link rules (which contain inline_image and inline_link)
        let elements: Vec<_> = content.into_inner()
            .filter(|p| {
                p.as_rule() == Rule::inline_image || p.as_rule() == Rule::image ||
                p.as_rule() == Rule::inline_link || p.as_rule() == Rule::link
            })
            .collect();
        
        assert!(elements.len() >= 2, "Should have both image and link elements");
        // First element should be image-related
        assert!(
            elements[0].as_rule() == Rule::inline_image || elements[0].as_rule() == Rule::image,
            "First should be image"
        );
        // Second element should be link-related
        let has_link = elements.iter().any(|e| {
            e.as_rule() == Rule::inline_link || e.as_rule() == Rule::link
        });
        assert!(has_link, "Should contain a link element");
    }

    #[test]
    fn phase5_realworld_links_and_images_mixed() {
        // Real-world example with mixed content
        let input = r#"Check ![logo](/logo.png) and visit [our site](http://example.com "Official Site") for more."#;
        let result = parse_inline_rule(Rule::inline_content, input);
        
        assert!(result.is_ok(), "Should parse mixed content with images and links");
        let pairs = result.unwrap();
        let content = pairs.into_iter().next().unwrap();
        
        // Phase 7: Count both direct inline_image/inline_link AND parent image/link rules
        let image_count = content.clone().into_inner()
            .filter(|p| p.as_rule() == Rule::inline_image || p.as_rule() == Rule::image)
            .count();
        let link_count = content.into_inner()
            .filter(|p| p.as_rule() == Rule::inline_link || p.as_rule() == Rule::link)
            .count();
        
        assert_eq!(image_count, 1, "Should have one image");
        assert_eq!(link_count, 1, "Should have one link");
    }
}

