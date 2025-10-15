//! Block-level AST builder
//!
//! Builds AST nodes for block-level markdown elements (CommonMark only):
//! - Document, Heading, Paragraph
//! - CodeBlock, List, ListItem
//! - Blockquote, HorizontalRule
//!
//! **Phase 2.2**: Extracted from ast_builder.rs, Marco extensions removed

use crate::components::marco_engine::{
    ast_node::{Node, Span},  // Use Span from ast_node, not the new span module
    builders::{helpers, AstError},  // Use AstError from mod.rs
    grammar::Rule,
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
                    // Extract children from the document rule, filtering out NEWLINE tokens
                    for inner_pair in pair.into_inner() {
                        if inner_pair.as_rule() == Rule::NEWLINE {
                            continue;
                        }
                        let child = self.build_block_node(inner_pair)?;
                        document_children.push(child);
                    }
                }
                Rule::NEWLINE => continue,
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
    /// **Note**: This is a simplified version for Phase 2.2. It only handles
    /// CommonMark block elements. Inline content still needs full ast_builder.
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

            Rule::heading
            | Rule::H1
            | Rule::H2
            | Rule::H3
            | Rule::H4
            | Rule::H5
            | Rule::H6
            | Rule::setext_h1
            | Rule::setext_h2 => {
                let level = self.extract_heading_level(&pair)?;
                // TODO: Extract proper heading content with inline elements
                let text = pair.as_str().trim_start_matches('#').trim().to_string();
                Ok(Node::heading(level, vec![Node::text(text, span.clone())], span))
            }

            Rule::code_block | Rule::fenced_code | Rule::indented_code => {
                let indent_level = helpers::calculate_indent_from_span(&pair.as_span());
                let (language, content) = self.extract_code_content(&pair)?;
                Ok(Node::code_block(language, content, indent_level, span))
            }

            Rule::list => {
                // Process inner list directly
                let mut inner_pairs = pair.into_inner();
                if let Some(inner_pair) = inner_pairs.next() {
                    self.build_block_node(inner_pair)
                } else {
                    Ok(Node::list(false, vec![], span))
                }
            }

            Rule::unordered_list | Rule::ordered_list => {
                let ordered = matches!(pair.as_rule(), Rule::ordered_list);
                let mut items = Vec::new();

                for inner_pair in pair.into_inner() {
                    if matches!(
                        inner_pair.as_rule(),
                        Rule::unordered_list_item | Rule::ordered_list_item
                    ) {
                        items.push(self.build_block_node(inner_pair)?);
                    }
                }

                Ok(Node::list(ordered, items, span))
            }

            Rule::unordered_list_item | Rule::ordered_list_item => {
                // TODO: Proper list item building with task support
                let children = self.build_children(pair)?;
                Ok(Node::list_item(children, None, None, span))
            }

            Rule::blockquote => {
                let children = self.build_children(pair)?;
                Ok(Node::block_quote(children, None, span))
            }

            Rule::hr => Ok(Node::horizontal_rule(span)),

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

    /// Extract heading level from heading pairs
    fn extract_heading_level(&self, pair: &Pair<Rule>) -> Result<u8, AstError> {
        let text = pair.as_str();
        match pair.as_rule() {
            Rule::H1 => Ok(1),
            Rule::H2 => Ok(2),
            Rule::H3 => Ok(3),
            Rule::H4 => Ok(4),
            Rule::H5 => Ok(5),
            Rule::H6 => Ok(6),
            Rule::setext_h1 => Ok(1),
            Rule::setext_h2 => Ok(2),
            Rule::heading => {
                // Count # characters for ATX headings
                let level = text.chars().take_while(|&c| c == '#').count() as u8;
                if level > 0 && level <= 6 {
                    Ok(level)
                } else if text.contains('=') {
                    Ok(1)
                } else if text.contains('-') {
                    Ok(2)
                } else {
                    Ok(1)
                }
            }
            _ => Ok(1),
        }
    }

    /// Extract code block language and content
    fn extract_code_content(
        &self,
        pair: &Pair<Rule>,
    ) -> Result<(Option<String>, String), AstError> {
        let text = pair.as_str();

        match pair.as_rule() {
            Rule::code_block => {
                // Check inner rules
                for inner_pair in pair.clone().into_inner() {
                    match inner_pair.as_rule() {
                        Rule::fenced_code => return self.extract_code_content(&inner_pair),
                        Rule::indented_code => return self.extract_code_content(&inner_pair),
                        _ => continue,
                    }
                }
                self.extract_fenced_code_content(text)
            }
            Rule::fenced_code => self.extract_fenced_code_content(text),
            Rule::indented_code => {
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
                Ok((None, content))
            }
            _ => Ok((None, text.to_string())),
        }
    }

    /// Extract fenced code block content and language
    fn extract_fenced_code_content(&self, text: &str) -> Result<(Option<String>, String), AstError> {
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
}

impl Default for BlockBuilder {
    fn default() -> Self {
        Self::new()
    }
}
