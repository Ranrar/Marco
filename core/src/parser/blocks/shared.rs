// Shared utilities for block-level parsers
// Contains span conversion helpers and common types

pub use crate::parser::shared::{to_parser_span, to_parser_span_range_inclusive as to_parser_span_range, GrammarSpan};
// No local Position/Span imports required here; use canonical helpers from parser::shared

#[cfg(test)]
use nom_locate::LocatedSpan;

/// Dedent list item content by removing the specified indent width.
/// This function is used to strip the list item indentation from nested content.
///
/// # Arguments
/// * `content` - The content to dedent
/// * `content_indent` - Number of spaces to remove from each line
///
/// # Returns
/// The dedented content with proper handling of:
/// - Tab expansion to spaces (based on actual column position)
/// - Trailing newline preservation
/// - Leading space removal up to content_indent
///
/// # Tab Expansion
/// Tabs are expanded based on their actual column position in the line.
/// Starting at `content_indent` column, each tab advances to the next multiple of 4.
/// This matches the CommonMark spec for list item indentation handling.
pub fn dedent_list_item_content(content: &str, content_indent: usize) -> String {
    let had_trailing_newline = content.ends_with('\n');
    
    let mut result = content.lines()
        .map(|line| {
            // First, expand tabs to spaces based on ACTUAL column position
            // Tabs must be expanded based on their column position (content_indent + column in line)
            let mut expanded = String::with_capacity(line.len() * 2);
            let mut column = content_indent; // Start at the content_indent column
            
            for ch in line.chars() {
                if ch == '\t' {
                    // Tab advances to next multiple of 4
                    let spaces_to_add = 4 - (column % 4);
                    for _ in 0..spaces_to_add {
                        expanded.push(' ');
                        column += 1;
                    }
                } else {
                    expanded.push(ch);
                    column += 1;
                }
            }
            
            // Now count and strip leading spaces up to content_indent
            let mut spaces_to_strip = 0;
            let mut chars = expanded.chars();
            while spaces_to_strip < content_indent {
                match chars.next() {
                    Some(' ') => spaces_to_strip += 1,
                    _ => break,
                }
            }
            
            // Return the rest of the line after stripping
            expanded[spaces_to_strip..].to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");
    
    // Preserve trailing newline if original had one
    if had_trailing_newline {
        result.push('\n');
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_to_parser_span() {
        let input = "line1\nline2\nline3";
        let span = LocatedSpan::new(input);
        let parser_span = to_parser_span(span);
        assert_eq!(parser_span.start.line, 1);
        assert_eq!(parser_span.start.column, 1);
    }

    #[test]
    fn test_to_parser_span_single_line_ascii() {
        // Test: "**bold**" at start of document
        let input = LocatedSpan::new("**bold**");
        let span = to_parser_span(input);
        
        // Start should be at line 1, column 1
        assert_eq!(span.start.line, 1);
        assert_eq!(span.start.column, 1);
        
        // End should be at line 1, column 9 (8 chars + 1-based = 9)
        assert_eq!(span.end.line, 1);
        assert_eq!(span.end.column, 9);
    }

    #[test]
    fn test_to_parser_span_single_line_utf8() {
        // Test: "Tëst" where 'ë' is 2 bytes (0xC3 0xAB)
        // Byte layout: T(1) ë(2+3) s(4) t(5) = 5 bytes total
        let input = LocatedSpan::new("Tëst");
        let span = to_parser_span(input);
        
        assert_eq!(span.start.line, 1);
        assert_eq!(span.start.column, 1);
        
        // End should be at byte position 6 (5 bytes + 1-based = 6)
        assert_eq!(span.end.line, 1);
        assert_eq!(span.end.column, 6);
    }

    #[test]
    fn test_to_parser_span_single_line_emoji() {
        // Test: "🎨" emoji is 4 bytes (0xF0 0x9F 0x8E 0xA8)
        let input = LocatedSpan::new("🎨");
        let span = to_parser_span(input);
        
        assert_eq!(span.start.line, 1);
        assert_eq!(span.start.column, 1);
        
        // End should be at byte position 5 (4 bytes + 1-based = 5)
        assert_eq!(span.end.line, 1);
        assert_eq!(span.end.column, 5);
    }

    #[test]
    fn test_to_parser_span_multi_line_code_block() {
        // Test: Code block spanning 3 lines
        // "```rust\nfn main() {}\n```"
        let input = LocatedSpan::new("```rust\nfn main() {}\n```");
        let span = to_parser_span(input);
        
        // Start at line 1, column 1
        assert_eq!(span.start.line, 1);
        assert_eq!(span.start.column, 1);
        
        // End at line 3 (1 + 2 newlines = 3)
        assert_eq!(span.end.line, 3);
        
        // End column should be 4 (3 backticks + 1-based = 4)
        assert_eq!(span.end.column, 4);
    }

    #[test]
    fn test_to_parser_span_ends_with_newline() {
        // Test: Span ending with newline should have end.column = 1
        let input = LocatedSpan::new("line1\nline2\n");
        let span = to_parser_span(input);
        
        assert_eq!(span.start.line, 1);
        assert_eq!(span.start.column, 1);
        
        // End at line 3 (1 + 2 newlines = 3), column 1
        assert_eq!(span.end.line, 3);
        assert_eq!(span.end.column, 1);
    }

    #[test]
    fn test_to_parser_span_multi_line_utf8() {
        // Test: Multi-line with UTF-8 on last line
        // "Line1\nTëst" where 'ë' is 2 bytes
        let input = LocatedSpan::new("Line1\nTëst");
        let span = to_parser_span(input);
        
        assert_eq!(span.start.line, 1);
        assert_eq!(span.start.column, 1);
        
        // End at line 2
        assert_eq!(span.end.line, 2);
        
        // "Tëst" = 5 bytes, so end column = 6 (1-based)
        assert_eq!(span.end.column, 6);
    }

    #[test]
    fn test_to_parser_span_offset_correctness() {
        // Verify that absolute offsets are calculated correctly
        let input = LocatedSpan::new("abc\ndef");
        let span = to_parser_span(input);
        
        // Start offset should be 0
        assert_eq!(span.start.offset, 0);
        
        // End offset should be 7 (3 + 1 newline + 3)
        assert_eq!(span.end.offset, 7);
    }

    #[test]
    fn smoke_test_dedent_simple() {
        let content = "  Line 1\n  Line 2\n";
        let result = dedent_list_item_content(content, 2);
        assert_eq!(result, "Line 1\nLine 2\n");
    }

    #[test]
    fn smoke_test_dedent_preserves_extra_indent() {
        let content = "  Line 1\n    Indented\n";
        let result = dedent_list_item_content(content, 2);
        assert_eq!(result, "Line 1\n  Indented\n");
    }

    #[test]
    fn smoke_test_dedent_preserves_blank_lines() {
        let content = "  Line 1\n\n  Line 2\n";
        let result = dedent_list_item_content(content, 2);
        assert_eq!(result, "Line 1\n\nLine 2\n");
    }

    #[test]
    fn smoke_test_dedent_with_tabs() {
        let content = "\tLine 1\n\tLine 2\n";
        let result = dedent_list_item_content(content, 4);
        assert_eq!(result, "Line 1\nLine 2\n");
    }
}
