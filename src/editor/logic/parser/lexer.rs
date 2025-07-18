// Tokenizer: raw Markdown → Tokens (with SourcePos for tracking)
// For now, just move parse_phrases here
use crate::editor::logic::ast::inlines::Inline;

use crate::editor::logic::parser::event::SourcePos;

/// Parse a string into a sequence of Inline AST nodes (phrases) with source positions.
pub fn parse_phrases(input: &str) -> Vec<(Inline, SourcePos)> {
    let mut result = Vec::new();
    let mut chars = input.chars().peekable();
    let mut buffer = String::new();
    let mut line = 1;
    let mut column = 1;
    #[derive(Debug, Clone)]
    struct Delim {
        pos: usize,
        ch: char,
        count: usize,
    }
    let mut delim_stack: Vec<Delim> = Vec::new();
    while let Some(&c) = chars.peek() {
        let pos = SourcePos { line, column };
        if c == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
        if c == '`' {
            if !buffer.is_empty() {
                result.push((Inline::Text(buffer.clone()), pos.clone()));
                buffer.clear();
            }
            let mut tick_count = 0;
            while let Some('`') = chars.peek() {
                chars.next();
                tick_count += 1;
            }
            let mut code_content = String::new();
            let mut found = false;
            while let Some(&next) = chars.peek() {
                if next == '`' {
                    let mut close_count = 0;
                    let mut lookahead = chars.clone();
                    while let Some('`') = lookahead.peek() {
                        lookahead.next();
                        close_count += 1;
                    }
                }
            }
            buffer.push(c);
            chars.next();
        } else {
            buffer.push(c);
            chars.next();
        }
    }
    if !buffer.is_empty() {
        result.push((Inline::Text(buffer), SourcePos { line, column }));
    }
    // Delimiter stack post-processing omitted for brevity
    result
}
