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
    // Section 6.1: Code spans, 6.2: Emphasis/strong emphasis
    let mut result = Vec::new();
    let mut chars = input.chars().peekable();
    let mut buffer = String::new();
    // Delimiter stack for emphasis
    #[derive(Debug, Clone)]
    struct Delim {
        pos: usize, // position in result
        ch: char,   // '*' or '_'
        count: usize, // 1 or 2
    }
    let mut delim_stack: Vec<Delim> = Vec::new();
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
                        for _ in 0..close_count { chars.next(); }
                        found = true;
                        break;
                    } else {
                        code_content.push('`');
                        chars.next();
                    }
                } else {
                    code_content.push(next);
                    chars.next();
                }
            }
            if found {
                let mut content = code_content;
                content = content.replace("\n", " ");
                if content.starts_with(' ') && content.ends_with(' ') && content.trim() != "" {
                    content = content[1..content.len()-1].to_string();
                }
                result.push(Inline::Code(crate::editor::logic::ast::inlines::CodeSpan { content }));
            } else {
                buffer.push_str(&"`".repeat(tick_count));
                buffer.push_str(&code_content);
            }
        } else if c == '*' || c == '_' {
            // Flush buffer
            if !buffer.is_empty() {
                result.push(Inline::Text(buffer.clone()));
                buffer.clear();
            }
            // Count delimiter run
            let ch = c;
            let mut count = 0;
            while let Some(&d) = chars.peek() {
                if d == ch { count += 1; chars.next(); } else { break; }
            }
            // Only allow 1 or 2 for now (CommonMark allows longer, but 1/2 covers *em* and **strong**)
            let run_count = if count > 2 { 2 } else { count };
            // Push delimiter to stack
            delim_stack.push(Delim { pos: result.len(), ch, count: run_count });
            // Insert a placeholder; will be replaced if matched
            result.push(Inline::Text(ch.to_string().repeat(run_count)));
        } else {
            buffer.push(c);
            chars.next();
        }
    }
    if !buffer.is_empty() {
        result.push(Inline::Text(buffer));
    }
    // Post-process delimiter stack to match pairs
    let mut i = 0;
    while i < delim_stack.len() {
        let open = &delim_stack[i];
        // Look for a matching close delimiter
        let mut j = i + 1;
        let mut removed = false;
        while j < delim_stack.len() {
            let close = &delim_stack[j];
            if open.ch == close.ch && open.count == close.count {
                // Matched pair: replace placeholders
                let (start, end) = (open.pos, close.pos);
                debug_assert!(start < result.len() && end < result.len(), "Delimiter positions out of bounds");
                let mut inner = Vec::new();
                for k in start+1..end {
                    inner.push(result[k].clone());
                }
                // Replace start
                result[start] = if open.count == 2 {
                    Inline::Emphasis(crate::editor::logic::ast::inlines::Emphasis::Strong(inner.clone()))
                } else {
                    Inline::Emphasis(crate::editor::logic::ast::inlines::Emphasis::Emph(inner.clone()))
                };
                // Remove inner and close placeholder
                for _ in start+1..=end {
                    result.remove(start+1);
                    // After removing at start+1, decrement all delim_stack.pos > start+1
                    for delim in delim_stack.iter_mut() {
                        if delim.pos > start+1 {
                            delim.pos -= 1;
                        }
                    }
                }
                // Remove close from stack (remove higher index first)
                delim_stack.remove(j);
                delim_stack.remove(i);
                removed = true;
                break;
            } else {
                j += 1;
            }
        }
        if removed {
            i = 0; // Restart loop after removal to avoid index confusion
        } else {
            i += 1;
        }
    }
    result
}
