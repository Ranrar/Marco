//! tokenizer.rs - Converts raw Markdown text to inline tokens

/// Tokenizer for inline Markdown parsing.
/// Breaks input text into a linear token stream with source position metadata.
/// Handles punctuation, delimiters, backticks, brackets, escapes, and entities.

use super::types::Token;
use super::entities_map::HTML_ENTITIES;
use htmlentity::entity::ICodedDataTrait;

/// Tokenizes raw Markdown input into a stream of inline tokens.
pub fn tokenize_inline(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    let mut buffer = String::new();
    while let Some(c) = chars.next() {
        match c {
            '*' => {
                let mut count = 1;
                while let Some(&next) = chars.peek() {
                    if next == '*' {
                        chars.next();
                        count += 1;
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Star(count));
            }
            '_' => {
                let mut count = 1;
                while let Some(&next) = chars.peek() {
                    if next == '_' {
                        chars.next();
                        count += 1;
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Underscore(count));
            }
            '`' => {
                let mut count = 1;
                while let Some(&next) = chars.peek() {
                    if next == '`' {
                        chars.next();
                        count += 1;
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Backtick(count));
            }
            '$' => {
                let mut count = 1;
                while let Some(&next) = chars.peek() {
                    if next == '$' {
                        chars.next();
                        count += 1;
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Dollar(count));
            }
            '[' => tokens.push(Token::OpenBracket),
            ']' => tokens.push(Token::CloseBracket),
            '!' => tokens.push(Token::Bang),
            '(' => tokens.push(Token::OpenParen),
            ')' => tokens.push(Token::CloseParen),
            '\\' => {
                if let Some(next) = chars.next() {
                    tokens.push(Token::Backslash(next));
                }
            }
            '&' => {
                // Try to parse a valid entity: &name; or &#...;
                let mut entity = String::from("&");
                let mut found_semicolon = false;
                while let Some(&next) = chars.peek() {
                    entity.push(next);
                    chars.next();
                    if next == ';' {
                        found_semicolon = true;
                        break;
                    }
                    // Only allow alphanumerics, #, x, X, and ;
                    if !(next.is_alphanumeric() || next == '#' || next == 'x' || next == 'X' || next == ';') {
                        break;
                    }
                }
                // Use htmlentity crate for robust entity validation
                use htmlentity::entity::decode;
                let decoded = decode(entity.as_bytes()).to_string().unwrap_or_default();
                let is_valid_entity = found_semicolon && entity.len() > 2 && decoded != entity;
                if is_valid_entity {
                    tokens.push(Token::Entity(entity));
                } else {
                    // Not a valid entity, treat as text
                    for ch in entity.chars() {
                        tokens.push(Token::Text(ch.to_string()));
                    }
                }
            }
            '\n' => tokens.push(Token::SoftBreak),
            '{' => {
                buffer.clear();
                buffer.push('{');
                let mut depth = 1;
                while let Some(next) = chars.next() {
                    buffer.push(next);
                    if next == '{' {
                        depth += 1;
                    } else if next == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                }
                tokens.push(Token::AttributeBlock(buffer.clone()));
            }
            '<' => {
                buffer.clear();
                buffer.push('<');
                // Collect everything until the next '>' (greedy, includes all content)
                let mut html_content = String::new();
                html_content.push('<');
                while let Some(next) = chars.next() {
                    html_content.push(next);
                    if next == '>' {
                        // Continue collecting until no more '>' in the sequence
                        if !chars.clone().any(|c| c == '>') {
                            break;
                        }
                    }
                }
                tokens.push(Token::Html(html_content));
            }
            _ => {
                buffer.push(c);
                // Flush buffer as text if next char is a token boundary
                let is_boundary = match chars.peek() {
                    Some(&next) => "*_`$[]!()\\&\n{}<".contains(next),
                    None => true,
                };
                if is_boundary {
                    if !buffer.is_empty() {
                        tokens.push(Token::Text(buffer.clone()));
                        buffer.clear();
                    }
                }
            }
        }
    }
    if !buffer.is_empty() {
        tokens.push(Token::Text(buffer));
    }
    tokens
}
