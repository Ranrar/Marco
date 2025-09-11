//! Simplified AST module with direct grammar mapping
//!
//! Contains only essential components:
//! - Node definitions with simplified structure
//! - Span for source position tracking  
//! - Simple AstBuilder for direct Pest-to-AST mapping

// Re-export key types from ast_node
pub use crate::components::marco_engine::ast_node::{Node, Span};

// Simple AstBuilder - implemented directly here
use crate::components::marco_engine::{errors::MarcoError, grammar::Rule};
use pest::iterators::Pairs;
pub struct AstBuilder;

impl AstBuilder {
    /// Build AST from Pest pairs - simple direct mapping
    pub fn build(pairs: Pairs<Rule>) -> Result<Node, MarcoError> {
        let mut children = Vec::new();

        for pair in pairs {
            let child = Self::build_node(pair)?;
            children.push(child);
        }

        let span = Span::new(0, 0, 1, 1);
        Ok(Node::document(children, span))
    }

    fn build_node(pair: pest::iterators::Pair<Rule>) -> Result<Node, MarcoError> {
        let span = Self::span_from_pair(&pair);

        match pair.as_rule() {
            Rule::document => {
                let children = pair
                    .into_inner()
                    .map(Self::build_node)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Node::document(children, span))
            }

            Rule::paragraph => {
                let content = pair
                    .into_inner()
                    .map(Self::build_node)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Node::paragraph(content, span))
            }

            Rule::heading => {
                // Extract heading level first
                let level = Self::extract_heading_level(&pair)?;
                let content = pair
                    .into_inner()
                    .map(Self::build_node)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Node::heading(level, content, span))
            }

            Rule::text => Ok(Node::text(pair.as_str().to_string(), span)),

            Rule::code_block => {
                let content = pair.as_str().to_string();
                // Extract language if present
                let language = Self::extract_code_language(&pair);
                Ok(Node::code_block(language, content, span))
            }

            Rule::code_inline => {
                let content = pair.as_str().trim_matches('`').to_string();
                Ok(Node::code(content, span))
            }

            Rule::emphasis => {
                // Determine if it's italic, bold, or both
                let text = pair.as_str();
                let is_bold_italic = text.starts_with("***") || text.starts_with("___");
                let is_bold = text.starts_with("**") || text.starts_with("__");

                let content = pair
                    .into_inner()
                    .map(Self::build_node)
                    .collect::<Result<Vec<_>, _>>()?;

                if is_bold_italic {
                    // Bold + italic
                    Ok(Node::strong(
                        vec![Node::emphasis(content, span.clone())],
                        span,
                    ))
                } else if is_bold {
                    Ok(Node::strong(content, span))
                } else {
                    Ok(Node::emphasis(content, span))
                }
            }

            Rule::list => {
                let ordered = Self::is_ordered_list(&pair);
                let items = pair
                    .into_inner()
                    .map(Self::build_node)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Node::list(ordered, items, span))
            }

            Rule::list_item => {
                let checked = Self::extract_task_state(&pair);
                let content = pair
                    .into_inner()
                    .map(Self::build_node)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Node::list_item(content, checked, span))
            }

            Rule::inline_link => {
                let (text, url, title) = Self::extract_link_parts(&pair)?;
                Ok(Node::link(text, url, title, span))
            }

            Rule::inline_image => {
                let (alt, url, title) = Self::extract_image_parts(&pair)?;
                Ok(Node::image(alt, url, title, span))
            }

            Rule::blockquote => {
                let content = pair
                    .into_inner()
                    .map(Self::build_node)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Node::block_quote(content, span))
            }

            Rule::table => {
                // Extract headers and rows properly
                let mut headers = Vec::new();
                let mut rows = Vec::new();

                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::table_row => {
                            let cells = inner_pair
                                .into_inner()
                                .map(Self::build_node)
                                .collect::<Result<Vec<_>, _>>()?;
                            if headers.is_empty() {
                                headers = cells;
                            } else {
                                rows.push(cells);
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Node::table(headers, rows, span))
            }

            Rule::table_row => {
                let cells = pair
                    .into_inner()
                    .map(Self::build_node)
                    .collect::<Result<Vec<_>, _>>()?;
                // This is handled by the table rule, but provide a fallback
                Ok(Node::list_item(cells, None, span))
            }

            Rule::table_cell => {
                let alignment = Self::extract_table_alignment(&pair);
                let content = pair
                    .into_inner()
                    .map(Self::build_node)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Node::table_cell(content, alignment, span))
            }

            Rule::hr => Ok(Node::horizontal_rule(span)),

            Rule::math_block => {
                let content = pair.as_str().to_string();
                Ok(Node::math_block(content, span))
            }

            Rule::math_inline => {
                let content = pair.as_str().trim_matches('$').to_string();
                Ok(Node::math_inline(content, span))
            }

            Rule::strikethrough => {
                let content = pair
                    .into_inner()
                    .map(Self::build_node)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Node::strikethrough(content, span))
            }

            Rule::highlight => {
                let content = pair
                    .into_inner()
                    .map(Self::build_node)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Node::highlight(content, span))
            }

            Rule::superscript => {
                let content = pair
                    .into_inner()
                    .map(Self::build_node)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Node::superscript(content, span))
            }

            Rule::subscript => {
                let content = pair
                    .into_inner()
                    .map(Self::build_node)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Node::subscript(content, span))
            }

            Rule::line_break => Ok(Node::line_break(span)),

            Rule::escaped_char => {
                let text = pair.as_str();
                let character = text.chars().nth(1).unwrap_or('\\'); // Skip the backslash
                Ok(Node::escaped_char(character, span))
            }

            // Handle other rules as unknown for now
            _ => {
                let content = pair.as_str().to_string();
                let rule_name = format!("{:?}", pair.as_rule());
                Ok(Node::unknown(content, rule_name, span))
            }
        }
    }

    fn span_from_pair(pair: &pest::iterators::Pair<Rule>) -> Span {
        let span = pair.as_span();
        let start = span.start() as u32;
        let end = span.end() as u32;
        Span::new(start, end, 1, 1) // Line/column calculation simplified for now
    }

    fn extract_heading_level(pair: &pest::iterators::Pair<Rule>) -> Result<u8, MarcoError> {
        let text = pair.as_str();
        let level = text.chars().take_while(|&c| c == '#').count() as u8;
        if level > 0 && level <= 6 {
            Ok(level)
        } else {
            Ok(1) // Default to level 1
        }
    }

    fn extract_code_language(pair: &pest::iterators::Pair<Rule>) -> Option<String> {
        let text = pair.as_str();
        if text.starts_with("```") {
            let first_line = text.lines().next()?;
            let lang = first_line.trim_start_matches('`').trim();
            if !lang.is_empty() {
                Some(lang.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn is_ordered_list(pair: &pest::iterators::Pair<Rule>) -> bool {
        let text = pair.as_str();
        text.trim_start()
            .chars()
            .next()
            .map_or(false, |c| c.is_ascii_digit())
    }

    fn extract_task_state(pair: &pest::iterators::Pair<Rule>) -> Option<bool> {
        let text = pair.as_str();
        if text.contains("[ ]") {
            Some(false)
        } else if text.contains("[x]") || text.contains("[X]") {
            Some(true)
        } else {
            None
        }
    }

    fn extract_link_parts(
        pair: &pest::iterators::Pair<Rule>,
    ) -> Result<(Vec<Node>, String, Option<String>), MarcoError> {
        // Simplified link parsing - just extract URL for now
        let text = pair.as_str();
        let url = text.to_string(); // Placeholder - should parse [text](url "title")
        let span = Self::span_from_pair(pair);
        Ok((vec![Node::text("link text".to_string(), span)], url, None))
    }

    fn extract_image_parts(
        pair: &pest::iterators::Pair<Rule>,
    ) -> Result<(String, String, Option<String>), MarcoError> {
        // Simplified image parsing
        let text = pair.as_str();
        Ok(("alt text".to_string(), text.to_string(), None))
    }

    fn extract_table_alignment(_pair: &pest::iterators::Pair<Rule>) -> Option<String> {
        // Simplified for now - would need to parse alignment syntax
        None
    }
}
