//! rules.rs - Rule handlers for inline syntaxes

/// Provides modular rule implementations for individual Markdown inline syntaxes.
/// Each rule handles a syntax like emphasis, code, HTML, math, etc.

use crate::logic::core::inline::types::{Token, InlineNode, SourcePos};

/// Handles parsing of emphasis and strong delimiters
pub fn parse_emphasis(token: &Token, nodes: &mut Vec<InlineNode>, line: usize, column: usize) {
    let count = match token {
        Token::Star(c) => *c,
        Token::Underscore(c) => *c,
        _ => return,
    };
    let ch = if let Token::Star(_) = token { '*' } else { '_' };
    // Represent delimiter runs as text for normalization
    nodes.push(InlineNode::Text { text: ch.to_string().repeat(count), pos: SourcePos { line, column } });
}

/// Handles parsing of code spans
pub fn parse_code_span(token: &Token, nodes: &mut Vec<InlineNode>, line: usize, column: usize) {
    if let Token::Backtick(_count) = token {
        // For now, treat as a code span with empty content (to be improved)
        nodes.push(InlineNode::Code { text: String::new(), pos: SourcePos { line, column } });
    }
}

/// Handles parsing of links and images
pub fn parse_link(token: &Token, nodes: &mut Vec<InlineNode>, line: usize, column: usize) {
    let s = match token {
        Token::OpenBracket => "[".to_string(),
        Token::Bang => "!".to_string(),
        Token::CloseBracket => "]".to_string(),
        _ => String::new(),
    };
    if !s.is_empty() {
        nodes.push(InlineNode::Text { text: s, pos: SourcePos { line, column } });
    }
}

/// Handles parsing of math spans
pub fn parse_math(token: &Token, nodes: &mut Vec<InlineNode>, line: usize, column: usize) {
    if let Token::Dollar(_count) = token {
        nodes.push(InlineNode::Math { text: String::new(), pos: SourcePos { line, column } });
    }
}
