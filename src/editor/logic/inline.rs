//! Inline parsing phase for CommonMark (Section 6)
//
// This module implements the inline parsing phase, converting raw text in paragraphs,
// headings, etc. into a Vec<Inline> AST nodes as per CommonMark 0.31.2 Section 6.
//
// - 6.1 Code spans
// - 6.2 Emphasis and strong emphasis
// - 6.3 Links
// - 6.4 Images
// - 6.5 Autolinks
// - 6.6 Raw HTML
// - 6.7 Hard line breaks
// - 6.8 Soft line breaks
// - 6.9 Textual content

use crate::editor::logic::ast::inlines::Inline;

/// Parse a string into a sequence of Inline AST nodes.
pub fn parse_inlines(input: &str) -> Vec<Inline> {
    // Section 6.1: Code spans
    // This implementation parses backtick-delimited code spans, handling multiple backticks and whitespace normalization.
    let mut result = Vec::new();
    let mut chars = input.chars().peekable();
    let mut buffer = String::new();
    while let Some(&c) = chars.peek() {
        if c == '`' {
            // Flush any accumulated text
            if !buffer.is_empty() {
                result.push(Inline::Text(buffer.clone()));
                buffer.clear();
            }
            // Count the number of backticks
            let mut tick_count = 0;
            while let Some('`') = chars.peek() {
                chars.next();
                tick_count += 1;
            }
            // Search for a matching closing run of backticks
            let mut code_content = String::new();
            let mut found = false;
            let mut temp = String::new();
            while let Some(&next) = chars.peek() {
                if next == '`' {
                    // Potential closing run
                    let mut close_count = 0;
                    let mut lookahead = chars.clone();
                    while let Some('`') = lookahead.peek() {
                        lookahead.next();
                        close_count += 1;
                    }
                    if close_count == tick_count {
                        // Found closing run
                        // Consume the closing backticks
                        for _ in 0..close_count { chars.next(); }
                        found = true;
                        break;
                    } else {
                        // Not a match, treat as literal
                        code_content.push('`');
                        chars.next();
                    }
                } else {
                    code_content.push(next);
                    chars.next();
                }
            }
            if found {
                // Normalize code span content: trim one leading/trailing space if present, replace newlines with spaces
                let mut content = code_content;
                // Collapse newlines to spaces
                content = content.replace("\n", " ");
                // Only strip one leading and one trailing space if not all spaces
                if content.starts_with(' ') && content.ends_with(' ') && content.trim() != "" {
                    content = content[1..content.len()-1].to_string();
                }
                result.push(Inline::Code(crate::editor::logic::ast::inlines::CodeSpan { content }));
            } else {
                // No closing run found, treat as literal backticks
                buffer.push_str(&"`".repeat(tick_count));
                buffer.push_str(&code_content);
            }
        } else {
            buffer.push(c);
            chars.next();
        }
    }
    if !buffer.is_empty() {
        result.push(Inline::Text(buffer));
    }
    result
}
