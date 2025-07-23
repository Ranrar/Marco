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
pub fn parse_code_span(code_text: &str, nodes: &mut Vec<InlineNode>, line: usize, column: usize) {
nodes.push(InlineNode::Code { text: code_text.to_string(), pos: SourcePos { line, column } });
}

/// Handles parsing of links and images
pub fn parse_link(token: &Token, nodes: &mut Vec<InlineNode>, line: usize, column: usize) {
    match token {
        Token::OpenBracket => nodes.push(InlineNode::Link {
            href: String::new(),
            title: String::new(),
            children: vec![],
            pos: SourcePos { line, column },
        }),
        Token::Bang => nodes.push(InlineNode::Image {
            src: String::new(),
            alt: vec![],
            title: String::new(),
            pos: SourcePos { line, column },
        }),
        Token::CloseBracket => nodes.push(InlineNode::Text { text: "]".to_string(), pos: SourcePos { line, column } }),
        _ => {}
    }
}
/// Handles parsing of entities
pub fn parse_entity(token: &Token, nodes: &mut Vec<InlineNode>, line: usize, column: usize) {
    if let Token::Entity(ref text) = token {
        nodes.push(InlineNode::Entity { text: text.clone(), pos: SourcePos { line, column } });
    }
}

/// Handles parsing of HTML tags
pub fn parse_html(token: &Token, nodes: &mut Vec<InlineNode>, line: usize, column: usize) {
    if let Token::Html(ref text) = token {
        nodes.push(InlineNode::Html { text: text.clone(), pos: SourcePos { line, column } });
    }
}

/// Handles parsing of math spans
pub fn parse_math(token: &Token, nodes: &mut Vec<InlineNode>, line: usize, column: usize) {
    if let Token::Dollar(_count) = token {
        nodes.push(InlineNode::Math { text: String::new(), pos: SourcePos { line, column } });
    }
}
