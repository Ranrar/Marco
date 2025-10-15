//! Block-level AST builder
//!
//! Builds AST nodes for block-level markdown elements (CommonMark only):
//! - Document, Heading, Paragraph
//! - CodeBlock, List, ListItem
//! - Blockquote, HorizontalRule
//!
//! **Two-Stage Parser**: Updated to use block_parser::Rule from the new modular grammar

use crate::components::engine::{
    ast_node::{Node, Span},  // Use Span from ast_node
    builders::{helpers, AstError},  // Use AstError from mod.rs
    grammar::BlockRule as Rule,  // Use BlockRule from the new two-stage parser
};
use pest::iterators::{Pair, Pairs};
use std::collections::HashMap;

// AstError is now imported from builders/mod.rs which has Display impl

/// Builder for block-level AST nodes
pub struct BlockBuilder {
    /// Cache for efficient span creation (currently unused but planned)
    span_cache: HashMap<String, Span>,
}

impl BlockBuilder {
    /// Create a new block builder
    pub fn new() -> Self {
        Self {
            span_cache: HashMap::new(),
        }
    }

    /// Build a document node from pairs
    pub fn build_document(&mut self, pairs: Pairs<Rule>) -> Result<Node, AstError> {
        let mut document_children = Vec::new();

        for pair in pairs {
            match pair.as_rule() {
                Rule::document => {
                    // Extract children from the document rule, filtering out whitespace tokens
                    for inner_pair in pair.into_inner() {
                        match inner_pair.as_rule() {
                            Rule::NEWLINE | Rule::BLANK_LINE | Rule::EOI => continue,
                            _ => {
                                let child = self.build_block_node(inner_pair)?;
                                document_children.push(child);
                            }
                        }
                    }
                }
                Rule::NEWLINE | Rule::BLANK_LINE | Rule::EOI => continue,
                _ => {
                    let child = self.build_block_node(pair)?;
                    document_children.push(child);
                }
            }
        }

        let span = Span::new(0, 0, 1, 1);
        Ok(Node::document(document_children, span))
    }

    /// Build a single block-level AST node
    ///
    /// **Two-Stage Parser**: Updated to handle new block_parser rule names
    pub fn build_block_node(&mut self, pair: Pair<Rule>) -> Result<Node, AstError> {
        let span = helpers::create_span(&pair);

        match pair.as_rule() {
            Rule::document => {
                let children = self.build_children(pair)?;
                Ok(Node::document(children, span))
            }

            Rule::block => {
                // Block is a dispatcher rule - recurse into the actual block type
                let inner_pair = pair.into_inner().next()
                    .ok_or_else(|| AstError::MissingContent("Empty block rule".to_string()))?;
                self.build_block_node(inner_pair)
            }

            Rule::paragraph => {
                // TODO: Proper paragraph building with inline content
                // For now, create simple text-only paragraph
                let text = pair.as_str().to_string();
                let indent_level = helpers::calculate_indent_from_span(&pair.as_span());
                Ok(Node::paragraph(
                    vec![Node::text(text, span.clone())],
                    indent_level,
                    span,
                ))
            }

            Rule::atx_heading => {
                // ATX headings: # Heading
                let level = self.extract_atx_heading_level(&pair)?;
                let text = pair.as_str().trim_start_matches('#').trim().to_string();
                Ok(Node::heading(level, vec![Node::text(text, span.clone())], span))
            }

            Rule::setext_heading => {
                // Setext headings: underlined with === or ---
                let level = self.extract_setext_heading_level(&pair)?;
                let text = self.extract_setext_content(&pair)?;
                Ok(Node::heading(level, vec![Node::text(text, span.clone())], span))
            }

            Rule::fenced_code_block => {
                let indent_level = helpers::calculate_indent_from_span(&pair.as_span());
                let (language, content) = self.extract_fenced_code_content(&pair)?;
                Ok(Node::code_block(language, content, indent_level, span))
            }

            Rule::indented_code_block => {
                let indent_level = helpers::calculate_indent_from_span(&pair.as_span());
                let content = self.extract_indented_code_content(&pair)?;
                Ok(Node::code_block(None, content, indent_level, span))
            }

            Rule::list => {
                // List dispatcher - delegate to inner list type
                let mut inner_pairs = pair.into_inner();
                if let Some(inner_pair) = inner_pairs.next() {
                    self.build_list_node(inner_pair, span)
                } else {
                    Ok(Node::list(false, vec![], span))
                }
            }

            Rule::blockquote => {
                // Blockquote contains blockquote_line children
                // Extract the text content from all lines and combine them
                let mut content = String::new();
                for inner_pair in pair.clone().into_inner() {
                    if inner_pair.as_rule() == Rule::blockquote_line {
                        // Get the line_content from blockquote_line
                        for line_part in inner_pair.into_inner() {
                            if line_part.as_rule() == Rule::line_content {
                                content.push_str(line_part.as_str());
                                content.push('\n');
                            }
                        }
                    }
                }
                
                // Create a text node with the combined content
                let text_node = Node::text(content, span.clone());
                Ok(Node::block_quote(vec![text_node], None, span))
            }

            Rule::thematic_break => {
                Ok(Node::horizontal_rule(span))
            }

            Rule::reference_definition => {
                // TODO: Store reference definitions for later resolution
                // For now, skip them in the AST (they're metadata)
                Ok(Node::text(String::new(), span))
            }

            _ => {
                // Unknown rule - return error for now
                Err(AstError::InvalidStructure(format!(
                    "Unsupported block rule: {:?}",
                    pair.as_rule()
                )))
            }
        }
    }

    /// Build all children of a pair
    fn build_children(&mut self, pair: Pair<Rule>) -> Result<Vec<Node>, AstError> {
        let mut children = Vec::new();
        for inner_pair in pair.into_inner() {
            let child = self.build_block_node(inner_pair)?;
            children.push(child);
        }
        Ok(children)
    }

    /// Build a list node from list-related rules
    fn build_list_node(&mut self, pair: Pair<Rule>, span: Span) -> Result<Node, AstError> {
        // The new grammar may have different list structure
        // For now, try to extract list items
        let mut items = Vec::new();
        let ordered = pair.as_str().contains("1.") || pair.as_str().contains("1)");

        for inner_pair in pair.into_inner() {
            // Try to build each inner element as a list item or nested list
            match self.build_block_node(inner_pair) {
                Ok(node) => items.push(node),
                Err(_) => continue, // Skip unparseable items
            }
        }

        Ok(Node::list(ordered, items, span))
    }

    /// Extract ATX heading level (1-6 based on number of # characters)
    fn extract_atx_heading_level(&self, pair: &Pair<Rule>) -> Result<u8, AstError> {
        let text = pair.as_str();
        let level = text.chars().take_while(|&c| c == '#').count() as u8;
        if level > 0 && level <= 6 {
            Ok(level)
        } else {
            Ok(1) // Default to H1 if can't determine
        }
    }

    /// Extract setext heading level (1 for ===, 2 for ---)
    fn extract_setext_heading_level(&self, pair: &Pair<Rule>) -> Result<u8, AstError> {
        let text = pair.as_str();
        // Check the underline to determine level
        if text.contains('=') {
            Ok(1)
        } else if text.contains('-') {
            Ok(2)
        } else {
            Ok(1) // Default to H1
        }
    }

    /// Extract content from setext heading (text before the underline)
    fn extract_setext_content(&self, pair: &Pair<Rule>) -> Result<String, AstError> {
        let text = pair.as_str();
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() >= 2 {
            Ok(lines[0].to_string())
        } else {
            Ok(text.to_string())
        }
    }

    /// Extract fenced code block content and language
    fn extract_fenced_code_content(&self, pair: &Pair<Rule>) -> Result<(Option<String>, String), AstError> {
        let text = pair.as_str();
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return Ok((None, String::new()));
        }

        // First line contains fence and optional language
        let first_line = lines[0].trim();
        let fence_char = if first_line.starts_with("```") {
            '`'
        } else if first_line.starts_with("~~~") {
            '~'
        } else {
            return Ok((None, text.to_string()));
        };

        // Extract language (everything after the fence markers on first line)
        let language = first_line
            .trim_start_matches(fence_char)
            .trim()
            .to_string();
        let language = if language.is_empty() {
            None
        } else {
            Some(language)
        };

        // Extract content (everything between first and last line)
        let content = if lines.len() > 2 {
            lines[1..lines.len() - 1].join("\n")
        } else if lines.len() == 2 {
            String::new()
        } else {
            text.to_string()
        };

        Ok((language, content))
    }

    /// Extract indented code block content
    fn extract_indented_code_content(&self, pair: &Pair<Rule>) -> Result<String, AstError> {
        let text = pair.as_str();
        // Remove common indentation (4 spaces or 1 tab)
        let content = text
            .lines()
            .map(|line| {
                if let Some(stripped) = line.strip_prefix("    ") {
                    stripped
                } else if let Some(stripped) = line.strip_prefix('\t') {
                    stripped
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        Ok(content)
    }
}

impl Default for BlockBuilder {
    fn default() -> Self {
        Self::new()
    }
}
