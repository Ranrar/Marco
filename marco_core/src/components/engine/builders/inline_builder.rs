//! Inline-level AST builder
//!
//! Builds AST nodes for inline markdown elements (CommonMark only):
//! - Text, Strong, Emphasis, Code
//! - Link, Image, Autolink
//! - HtmlTag, LineBreak, EscapedChar
//!
//! **Phase 2.2**: Extracted from ast_builder.rs, Marco extensions removed

use crate::components::engine::{
    ast_node::{Node, Span},  // Use Span from ast_node module
    builders::{helpers, AstError},  // Use centralized AstError
    grammar::Rule,
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
            Rule::text | Rule::text_no_newline => {
                Ok(Node::text(pair.as_str().to_string(), span))
            }

            Rule::code_inline => {
                let content = self.extract_inline_code_content(&pair)?;
                Ok(Node::code(content, span))
            }

            Rule::bold_asterisk | Rule::bold_underscore | Rule::bold => {
                let children = self.build_inline_children(pair)?;
                Ok(Node::strong(children, span))
            }

            Rule::italic_asterisk | Rule::italic_underscore | Rule::emphasis => {
                let children = self.build_inline_children(pair)?;
                Ok(Node::emphasis(children, span))
            }

            Rule::bold_italic_triple_asterisk
            | Rule::bold_italic_triple_underscore
            | Rule::bold_italic_mixed_ast_under
            | Rule::bold_italic_mixed_under_ast
            | Rule::bold_italic_triple_mixed_au
            | Rule::bold_italic_triple_mixed_ua
            | Rule::bold_italic => {
                // Bold + italic combination - nest them
                let children = self.build_inline_children(pair)?;
                let italic_node = Node::emphasis(children, span.clone());
                Ok(Node::strong(vec![italic_node], span))
            }

            Rule::inline_link => {
                let (text_nodes, url, title) = self.extract_link_content(pair)?;
                Ok(Node::link(text_nodes, url, title, span))
            }

            Rule::inline_image => {
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

            Rule::hard_line_break => Ok(Node::hard_line_break(span)),
            Rule::soft_line_break => Ok(Node::soft_line_break(span)),

            Rule::escaped_char => {
                let ch = pair.as_str().chars().nth(1).unwrap_or('\\');
                Ok(Node::escaped_char(ch, span))
            }

            Rule::reference_link | Rule::reference_image => {
                // TODO: Proper reference link/image support
                // For now, treat as regular link/image
                let text = pair.as_str().to_string();
                if matches!(pair.as_rule(), Rule::reference_link) {
                    Ok(Node::link(
                        vec![Node::text(text.clone(), span.clone())],
                        text,
                        None,
                        span,
                    ))
                } else {
                    Ok(Node::image(text.clone(), text, None, span))
                }
            }

            _ => {
                // Unknown or unsupported inline rule
                Err(AstError::InvalidStructure(format!(
                    "Unsupported inline rule: {:?}",
                    pair.as_rule()
                )))
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
