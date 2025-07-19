// Tokenizer: raw Markdown → Tokens (with SourcePos for tracking)
use crate::logic::parser::attributes::{self, parse_attributes_block};
// For now, just move parse_phrases here
use crate::logic::ast::inlines::Inline;

use crate::logic::parser::event::SourcePos;

/// Parse a string into a sequence of Inline AST nodes (phrases) with source positions.
pub fn parse_phrases(input: &str) -> (Vec<(Inline, SourcePos)>, Vec<crate::logic::parser::event::Event>) {
    let mut result = Vec::new();
    let mut events = Vec::new();
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
        // Inline math parsing: $...$
        if c == '$' {
            // Check for escaped dollar
            let prev_char = if buffer.len() > 0 { buffer.chars().last() } else { None };
            if prev_char == Some('\\') {
                buffer.push(c);
                chars.next();
                continue;
            }
            // Flush buffer as text before math
            if !buffer.is_empty() {
                result.push((Inline::Text(buffer.clone()), pos.clone()));
                buffer.clear();
            }
            // Consume opening $
            chars.next();
            let math_start_pos = SourcePos { line, column };
            let mut math_content = String::new();
            let mut found_closing = false;
            while let Some(&next) = chars.peek() {
                // End math if next is unescaped $
                if next == '$' {
                    chars.next();
                    found_closing = true;
                    break;
                }
                // Handle newlines in math
                if next == '\n' {
                    line += 1;
                    column = 1;
                } else {
                    column += 1;
                }
                math_content.push(next);
                chars.next();
            }
            if found_closing {
                use crate::logic::ast::math::{MathInline, MathType};
                let math_inline = MathInline {
                    content: math_content,
                    math_type: MathType::LaTeX, // Default to LaTeX, could be extended
                    position: Some(math_start_pos.clone()),
                    attributes: None,
                };
                result.push((Inline::Math(math_inline), math_start_pos));
            } else {
                // Unclosed math, treat as text
                buffer.push('$');
                buffer.push_str(&math_content);
            }
        } else if c == '`' {
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
                let attrs = parse_attributes_block(&attr_block);
                let new_inline = match last_inline {
                    Inline::Text(s) => Inline::Text(s),
                    Inline::Code(mut code) => { code.attributes = Some(attrs); Inline::Code(code) },
                    Inline::Emphasis(emph) => match emph {
                        crate::logic::ast::inlines::Emphasis::Emph(inner, _) => Inline::Emphasis(crate::logic::ast::inlines::Emphasis::Emph(inner, Some(attrs.clone()))),
                        crate::logic::ast::inlines::Emphasis::Strong(inner, _) => Inline::Emphasis(crate::logic::ast::inlines::Emphasis::Strong(inner, Some(attrs.clone()))),
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
    return (result, events);
#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_phrases_inline_math_basic() {
        let input = "This is $x^2$ inline math.";
        let (inlines, _events) = parse_phrases(input);
        let mut found_math = false;
        for (inline, _pos) in inlines {
            match inline {
                Inline::Math(math) => {
                    assert_eq!(math.content, "x^2");
                    found_math = true;
                }
                _ => {}
            }
        }
        assert!(found_math);
    }

    #[test]
    fn test_parse_phrases_inline_math_escaped() {
        let input = "This is \\$notmath$ and $y^2$ is math.";
        let (inlines, _events) = parse_phrases(input);
        let mut found_math = false;
        let mut found_text = false;
        for (inline, _pos) in inlines {
            match inline {
                Inline::Math(math) => {
                    assert_eq!(math.content, "y^2");
                    found_math = true;
                }
                Inline::Text(ref s) => {
                    if s.contains("$notmath$") { found_text = true; }
                }
                _ => {}
            }
        }
        assert!(found_math);
        assert!(found_text);
    }

    #[test]
    fn test_parse_phrases_inline_math_unclosed() {
        let input = "This is $unclosed inline math.";
        let (inlines, _events) = parse_phrases(input);
        let mut found_text = false;
        for (inline, _pos) in inlines {
            match inline {
                Inline::Text(ref s) => {
                    if s.contains("$unclosed inline math.") { found_text = true; }
                }
                _ => {}
            }
        }
        assert!(found_text);
    }
    use super::*;
    use crate::logic::ast::inlines::Inline;
    #[test]
    fn test_parse_phrases_with_attributes() {
        let input = "*emph*{.important} and **strong**{#main} and [link](url){.external}";
        let (inlines, _events) = parse_phrases(input);
        let mut found_emph = false;
        let mut found_strong = false;
        let mut found_link = false;
        for (inline, _pos) in inlines {
            match inline {
                Inline::Emphasis(e) => match e {
                    crate::logic::ast::inlines::Emphasis::Emph(_, Some(attrs)) => {
                        assert!(attrs.classes.contains(&"important".to_string()));
                        found_emph = true;
                    }
                    crate::logic::ast::inlines::Emphasis::Strong(_, Some(attrs)) => {
                        assert_eq!(attrs.id, Some("main".to_string()));
                        found_strong = true;
                    }
                    _ => {}
                },
                Inline::Link(link) => {
                    if let Some(attrs) = &link.attributes {
                        assert!(attrs.classes.contains(&"external".to_string()));
                        found_link = true;
                    }
                }
                _ => {}
            }
        }
        assert!(found_emph);
        assert!(found_strong);
        assert!(found_link);
    }
}
}
