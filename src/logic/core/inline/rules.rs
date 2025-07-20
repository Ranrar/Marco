//! rules.rs - Rule handlers for inline syntaxes

/// Provides modular rule implementations for individual Markdown inline syntaxes.
/// Each rule handles a syntax like emphasis, code, HTML, math, etc.

use crate::logic::core::inline::types::{Token, Delim, Bracket};
use crate::logic::ast::inlines::Inline;
use crate::logic::core::event_types::SourcePos;

/// Handles parsing of emphasis and strong delimiters
pub fn parse_emphasis(token: &Token, result: &mut Vec<(Inline, SourcePos)>, delim_stack: &mut Vec<Delim>, line: usize, column: usize) {
    let count = match token {
        Token::Star(c) => *c,
        Token::Underscore(c) => *c,
        _ => return,
    };
    let ch = if let Token::Star(_) = token { '*' } else { '_' };
    // TODO: Compute left/right flanking for can_open/can_close
    delim_stack.push(Delim {
        ch,
        count,
        pos: SourcePos { line, column },
        can_open: true, // TODO: compute flanking
        can_close: true, // TODO: compute flanking
        idx: result.len(),
        active: true,
    });
}

/// Handles parsing of code spans
pub fn parse_code_span(token: &Token, result: &mut Vec<(Inline, SourcePos)>, line: usize, column: usize) {
    if let Token::Backtick(_count) = token {
        // For now, treat as a code span (stub)
        let code = "".to_string(); // TODO: extract code content
        result.push((Inline::CodeSpan(crate::logic::ast::inlines::CodeSpan { content: code, attributes: None }), SourcePos { line, column }));
    }
}

/// Handles parsing of links and images
pub fn parse_link(token: &Token, result: &mut Vec<(Inline, SourcePos)>, bracket_stack: &mut Vec<Bracket>, line: usize, column: usize) {
    match token {
        Token::OpenBracket => {
            bracket_stack.push(Bracket {
                image: false,
                pos: SourcePos { line, column },
                idx: result.len(),
            });
        }
        Token::Bang => {
            bracket_stack.push(Bracket {
                image: true,
                pos: SourcePos { line, column },
                idx: result.len(),
            });
        }
        Token::CloseBracket => {
            if let Some(bracket) = bracket_stack.pop() {
                result.push((Inline::Text("[link/image]".to_string()), bracket.pos));
            } else {
                result.push((Inline::Text(String::from("]")), SourcePos { line, column }));
            }
        }
        _ => {}
    }
}

/// Handles parsing of math spans
pub fn parse_math(token: &Token, result: &mut Vec<(Inline, SourcePos)>, line: usize, column: usize) {
    if let Token::Dollar(_count) = token {
        let math = "".to_string();
        result.push((Inline::Math(crate::logic::ast::math::MathInline {
            content: math,
            math_type: crate::logic::ast::math::MathType::LaTeX,
            position: Some(SourcePos { line, column }),
            attributes: None,
        }), SourcePos { line, column }));
    }
}
