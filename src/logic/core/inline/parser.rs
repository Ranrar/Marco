//! parser.rs - Core inline parser: token stream → raw AST

/// Parses a token stream into raw, unprocessed inline nodes.
/// Leaves unresolved things like emphasis and links as placeholder nodes or temporary markers.
/// Constructs a flat or shallow node tree.

use crate::logic::ast::inlines::Inline;
use crate::logic::core::event_types::SourcePos;
use super::types::{Delim, Bracket, Token};
use super::tokenizer::tokenize_inline;

/// Parse a string into a sequence of Inline AST nodes (phrases) with source positions.
pub fn parse_phrases(input: &str) -> (Vec<(Inline, SourcePos)>, Vec<crate::logic::core::event_types::Event>) {
    let tokens = tokenize_inline(input);
    let mut result = Vec::new();
    let mut delim_stack: Vec<Delim> = Vec::new();
    let mut bracket_stack: Vec<Bracket> = Vec::new();
    let events = Vec::new();
    let line = 1;
    let column = 1;

    for token in tokens {
        match token {
            Token::Text(s) => {
                result.push((Inline::Text(s), SourcePos { line, column }));
            }
            Token::Star(_) | Token::Underscore(_) => {
                crate::logic::core::inline::rules::parse_emphasis(&token, &mut result, &mut delim_stack, line, column);
            }
            Token::Backtick(_) => {
                crate::logic::core::inline::rules::parse_code_span(&token, &mut result, line, column);
            }
            Token::Dollar(_) => {
                crate::logic::core::inline::rules::parse_math(&token, &mut result, line, column);
            }
            Token::OpenBracket | Token::Bang | Token::CloseBracket => {
                crate::logic::core::inline::rules::parse_link(&token, &mut result, &mut bracket_stack, line, column);
            }
            Token::OpenParen | Token::CloseParen => {
                // Parenthesis logic (to be modularized)
            }
            Token::Backslash(ch) => {
                result.push((Inline::Text(ch.to_string()), SourcePos { line, column }));
            }
            Token::Ampersand => {
                // Entity logic (to be modularized)
            }
            Token::Html(s) => {
                result.push((Inline::RawHtml(s), SourcePos { line, column }));
            }
            Token::AttributeBlock(s) => {
                result.push((Inline::Text(s), SourcePos { line, column }));
            }
            Token::SoftBreak => {
                result.push((Inline::SoftBreak, SourcePos { line, column }));
            }
            Token::HardBreak => {
                result.push((Inline::HardBreak, SourcePos { line, column }));
            }
            _ => {}
        }
    }

    // Delimiter stack post-processing for emphasis/strong
    crate::logic::core::inline::delimiters::process_delimiters(&mut delim_stack, &mut result);

    (result, events)
}
