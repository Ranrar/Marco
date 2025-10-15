//! Inline-level AST builder
//!
//! Builds AST nodes for inline markdown elements (CommonMark only):
//! - Text, Strong, Emphasis, Code
//! - Link, Image, Autolink
//! - HtmlTag, LineBreak, EscapedChar
//!
//! **Two-Stage Parser**: Updated to use inline_parser::Rule from the new modular grammar

use crate::components::engine::{
    ast_node::{Node, Span},  // Use Span from ast_node module
    builders::{helpers, AstError},  // Use centralized AstError
    grammar::InlineRule as Rule,  // Use InlineRule from two-stage parser
};
use pest::iterators::Pair;

// AstError removed - using AstError from mod.rs

/// Builder for inline-level AST nodes
pub struct InlineBuilder;

impl InlineBuilder {
    /// Create a new inline builder
    pub fn new() -> Self {
        Self
    }

    /// Build an inline node from a Pest pair
    pub fn build_inline_node(&self, pair: Pair<Rule>) -> Result<Node, AstError> {
        let span = helpers::create_span(&pair);

        match pair.as_rule() {
            Rule::text => {
                Ok(Node::text(pair.as_str().to_string(), span))
            }

            Rule::code_span => {
                let content = self.extract_inline_code_content(&pair)?;
                Ok(Node::code(content, span))
            }

            Rule::strong | Rule::strong_asterisk | Rule::strong_underscore => {
                let children = self.build_inline_children(pair)?;
                Ok(Node::strong(children, span))
            }

            Rule::emphasis | Rule::emphasis_asterisk | Rule::emphasis_underscore => {
                let children = self.build_inline_children(pair)?;
                Ok(Node::emphasis(children, span))
            }

            Rule::link => {
                // New grammar has link as dispatcher for inline_link, link_full_reference, etc.
                let (text_nodes, url, title) = self.extract_link_content(pair)?;
                Ok(Node::link(text_nodes, url, title, span))
            }

            Rule::image => {
                // New grammar has image as dispatcher for inline_image, image_full_reference, etc.
                let (alt_text, url, title) = self.extract_image_content(pair)?;
                Ok(Node::image(alt_text, url, title, span))
            }

            Rule::autolink => {
                let url = pair.as_str().trim_start_matches('<').trim_end_matches('>').to_string();
                // Autolink is just a link with URL as text
                Ok(Node::link(
                    vec![Node::text(url.clone(), span.clone())],
                    url,
                    None,
                    span,
                ))
            }

            Rule::line_break => {
                // New grammar just has "line_break" instead of hard/soft distinction
                Ok(Node::hard_line_break(span))
            }

            Rule::escape => {
                let ch = pair.as_str().chars().nth(1).unwrap_or('\\');
                Ok(Node::escaped_char(ch, span))
            }

            Rule::html_tag => {
                // HTML tag - store as-is
                let content = pair.as_str().to_string();
                Ok(Node::text(content, span)) // TODO: Create proper HTML node type
            }

            // Inner rules that might appear (skip or handle)
            Rule::inline_content => {
                // This is a container - recurse into children
                let mut children = Vec::new();
                for inner in pair.into_inner() {
                    children.push(self.build_inline_node(inner)?);
                }
                if children.len() == 1 {
                    Ok(children.into_iter().next().unwrap())
                } else {
                    // Multiple children - wrap in a text node
                    let text = children.iter().map(|n| format!("{:?}", n)).collect::<Vec<_>>().join("");
                    Ok(Node::text(text, span))
                }
            }

            _ => {
                // Unknown or unsupported inline rule - for now, convert to text
                Ok(Node::text(pair.as_str().to_string(), span))
            }
        }
    }

    /// Build all inline children of a pair
    fn build_inline_children(&self, pair: Pair<Rule>) -> Result<Vec<Node>, AstError> {
        let mut children = Vec::new();
        for inner_pair in pair.into_inner() {
            let child = self.build_inline_node(inner_pair)?;
            children.push(child);
        }
        Ok(children)
    }

    /// Extract inline code content
    fn extract_inline_code_content(&self, pair: &Pair<Rule>) -> Result<String, AstError> {
        let text = pair.as_str();
        
        // Remove backtick delimiters
        let content = text.trim_start_matches('`').trim_end_matches('`');
        
        Ok(content.to_string())
    }

    /// Extract link content (text, URL, optional title)
    fn extract_link_content(
        &self,
        pair: Pair<Rule>,
    ) -> Result<(Vec<Node>, String, Option<String>), AstError> {
        let mut text_nodes = Vec::new();
        let mut url = String::new();
        let mut title: Option<String> = None;

        // TODO: Phase 2.2 - Proper link content extraction with grammar rules
        // For now, just extract the whole text as a simple placeholder
        let text = pair.as_str().to_string();
        let span = helpers::create_span(&pair);
        text_nodes.push(Node::text(text.clone(), span));
        url = text;

        Ok((text_nodes, url, title))
    }

    /// Extract image content (alt text as String, URL, optional title)
    fn extract_image_content(
        &self,
        pair: Pair<Rule>,
    ) -> Result<(String, String, Option<String>), AstError> {
        let alt_text = pair.as_str().to_string();
        let url = alt_text.clone();
        let title: Option<String> = None;

        // TODO: Phase 2.2 - Proper image content extraction with grammar rules

        Ok((alt_text, url, title))
    }
}

impl Default for InlineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
