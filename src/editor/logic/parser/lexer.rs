// Tokenizer: raw Markdown → Tokens (with SourcePos for tracking)
use crate::editor::logic::parser::attributes_parser;
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
        } else if c == '{' {
            // Try to parse an attribute block
            let mut attr_block = String::new();
            let mut brace_count = 0;
            while let Some(&next) = chars.peek() {
                if next == '{' {
                    brace_count += 1;
                }
                if next == '}' {
                    brace_count -= 1;
                    attr_block.push(next);
                    chars.next();
                    if brace_count == 0 {
                        break;
                    }
                    continue;
                }
                attr_block.push(next);
                chars.next();
            }
            // Parse attributes and attach to previous inline if possible
            if let Some((last_inline, last_pos)) = result.pop() {
                let attrs = attributes_parser::parse_attributes_block(&attr_block);
                let new_inline = match last_inline {
                    Inline::Text(s) => Inline::Text(s),
                    Inline::Code(mut code) => { code.attributes = Some(attrs); Inline::Code(code) },
                    Inline::Emphasis(emph) => match emph {
                        crate::editor::logic::ast::inlines::Emphasis::Emph(inner, _) => Inline::Emphasis(crate::editor::logic::ast::inlines::Emphasis::Emph(inner, Some(attrs.clone()))),
                        crate::editor::logic::ast::inlines::Emphasis::Strong(inner, _) => Inline::Emphasis(crate::editor::logic::ast::inlines::Emphasis::Strong(inner, Some(attrs.clone()))),
                    },
                    Inline::Link(mut link) => { link.attributes = Some(attrs); Inline::Link(link) },
                    Inline::Image(mut image) => { image.attributes = Some(attrs); Inline::Image(image) },
                    other => other,
                };
                result.push((new_inline, last_pos));
            }
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
