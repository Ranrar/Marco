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
    builders::{helpers, AstError, InlineBuilder},  // Use AstError and InlineBuilder from mod.rs
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

        // Phase 5.1: Post-process to merge indented lists into parent lists
        document_children = self.post_process_nested_lists(document_children);

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
                // Parse inline content within the paragraph
                let text = pair.as_str().trim();
                let indent_level = helpers::calculate_indent_from_span(&pair.as_span());
                let inline_nodes = self.parse_inline_content(text, span.clone())?;
                
                Ok(Node::paragraph(
                    inline_nodes,
                    indent_level,
                    span,
                ))
            }

            Rule::atx_heading => {
                // ATX headings: # Heading
                let level = self.extract_atx_heading_level(&pair)?;
                let text = pair.as_str().trim_start_matches('#').trim();
                let inline_nodes = self.parse_inline_content(text, span.clone())?;
                
                Ok(Node::heading(level, inline_nodes, span))
            }

            Rule::setext_heading => {
                // Setext headings: underlined with === or ---
                let level = self.extract_setext_heading_level(&pair)?;
                let text = self.extract_setext_content(&pair)?;
                let inline_nodes = self.parse_inline_content(&text, span.clone())?;
                
                Ok(Node::heading(level, inline_nodes, span))
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
                // Phase 5.1: List now contains list_item+ directly, no dispatcher
                self.build_list_node(pair, span)
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
                
                // Parse the combined content as inline elements
                let inline_nodes = self.parse_inline_content(content.trim(), span.clone())?;
                Ok(Node::block_quote(inline_nodes, None, span))
            }

            Rule::thematic_break => {
                Ok(Node::horizontal_rule(span))
            }

            Rule::reference_definition => {
                // Parse reference definition: [label]: url "title"
                let mut label = String::new();
                let mut url = String::new();
                let mut title: Option<String> = None;

                for inner in pair.into_inner() {
                    match inner.as_rule() {
                        Rule::reference_label => {
                            // Extract label content (strip [ ])
                            label = inner.as_str()
                                .trim_start_matches('[')
                                .trim_end_matches(']')
                                .to_string();
                        }
                        Rule::reference_destination => {
                            // Get the destination URL
                            let dest_inner = inner.into_inner().next();
                            if let Some(dest) = dest_inner {
                                url = match dest.as_rule() {
                                    Rule::reference_angle_bracket_destination => {
                                        // Strip < > from angle bracket URLs
                                        dest.as_str()
                                            .trim_start_matches('<')
                                            .trim_end_matches('>')
                                            .to_string()
                                    }
                                    Rule::reference_plain_destination => {
                                        dest.as_str().to_string()
                                    }
                                    _ => dest.as_str().to_string(),
                                };
                            }
                        }
                        Rule::reference_title => {
                            // Get the title (strip quotes)
                            let title_inner = inner.into_inner().next();
                            if let Some(t) = title_inner {
                                let title_str = t.as_str();
                                title = Some(
                                    title_str
                                        .trim_start_matches('"')
                                        .trim_end_matches('"')
                                        .trim_start_matches('\'')
                                        .trim_end_matches('\'')
                                        .trim_start_matches('(')
                                        .trim_end_matches(')')
                                        .to_string()
                                );
                            }
                        }
                        _ => {}
                    }
                }

                // Create ReferenceDefinition node
                Ok(Node::ReferenceDefinition {
                    label,
                    url,
                    title,
                    span,
                })
            }

            Rule::html_block => {
                // HTML blocks - delegate to inner type to determine block_type
                let inner_pair = pair.into_inner().next()
                    .ok_or_else(|| AstError::MissingContent("Empty html_block rule".to_string()))?;
                self.extract_html_block(inner_pair, span)
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
    /// Phase 5.1: Enhanced to support nested lists via indentation-based tree building
    fn build_list_node(&mut self, pair: Pair<Rule>, span: Span) -> Result<Node, AstError> {
        // Detect tight vs loose list by checking for blank lines between items
        let is_tight = self.detect_list_tightness(&pair);
        
        // Extract starting number before consuming pair (for ordered lists)
        let extracted_start_number = self.extract_starting_number(&pair);

        // Phase 5.1: First, collect ALL items with their indentation levels and type
        let mut flat_items: Vec<(usize, Node, bool)> = Vec::new(); // (indent_level, node, is_ordered)
        let mut first_item_is_ordered = false;
        
        for inner_pair in pair.into_inner() {
            let inner_rule = inner_pair.as_rule();
            
            match inner_rule {
                // Phase 5.1: New unified list_item structure with nested support
                Rule::list_item => {
                    // list_item = INDENT_OPT? ~ (unordered_item | ordered_item) ~ nested_list?
                    let item_text = inner_pair.as_str();
                    let item_span = helpers::create_span(&inner_pair);
                    
                    // Calculate indentation from column position
                    let indent_level = if item_span.column > 0 {
                        (item_span.column - 1) as usize
                    } else {
                        0
                    };
                    
                    let mut is_item_ordered = false;
                    let mut content_nodes = Vec::new();
                    
                    // Process children: unordered_item/ordered_item and optional nested_list
                    for child_pair in inner_pair.into_inner() {
                        match child_pair.as_rule() {
                            Rule::unordered_item => {
                                is_item_ordered = false;
                                // Extract content from unordered_item
                                let content_str = self.extract_list_item_content(child_pair.as_str());
                                content_nodes = self.parse_inline_content(&content_str, item_span.clone())?;
                            }
                            Rule::ordered_item => {
                                is_item_ordered = true;
                                // Extract content from ordered_item
                                let content_str = self.extract_list_item_content(child_pair.as_str());
                                content_nodes = self.parse_inline_content(&content_str, item_span.clone())?;
                            }
                            Rule::nested_list => {
                                // nested_list = NEWLINE ~ NESTED_INDENT ~ list
                                // Skip NEWLINE and NESTED_INDENT (silent), get the list
                                let mut nested_children = child_pair.into_inner();
                                let _newline = nested_children.next(); // Skip NEWLINE
                                let nested_list_pair = nested_children.next()
                                    .ok_or_else(|| AstError::MissingContent("nested_list missing list rule".to_string()))?;
                                let nested_node = self.build_list_node(nested_list_pair, item_span.clone())?;
                                content_nodes.push(nested_node);
                            }
                            _ => {}
                        }
                    }
                    
                    // Create list item node
                    // Phase 5.1: Use None for indent_level since nesting is now structural
                    let item_is_loose = !is_tight;
                    let list_item = Node::list_item(content_nodes, None, None, item_is_loose, item_span);
                    
                    // Track first item type for list
                    if flat_items.is_empty() {
                        first_item_is_ordered = is_item_ordered;
                    }
                    
                    flat_items.push((indent_level, list_item, is_item_ordered));
                }
                _ => {
                    // Other nodes - add with 0 indentation and unordered
                    if let Ok(node) = self.build_block_node(inner_pair) {
                        flat_items.push((0, node, false));
                    }
                }
            }
        }
        
        // Phase 5.1: Build tree from flat items based on indentation
        // NOTE: With new grammar, nested_list is already handled recursively,
        // so we may just need to return flat_items directly as the tree
        let items: Vec<Node> = flat_items.into_iter().map(|(_, node, _)| node).collect();

        // Determine list type and starting number from first item
        let ordered = first_item_is_ordered;
        let start_number = if ordered {
            extracted_start_number
        } else {
            None
        };

        Ok(Node::list(ordered, items, is_tight, start_number, span))
    }
    
    /// Build a nested tree structure from flat items based on indentation
    /// Phase 5.1: Core nesting algorithm
    fn build_nested_list_tree(
        &mut self,
        flat_items: Vec<(usize, Node, bool)>, // (indent, node, is_ordered)
        base_indent: usize,
    ) -> Result<Vec<Node>, AstError> {
        let mut result = Vec::new();
        let mut i = 0;
        
        while i < flat_items.len() {
            let (indent, item, _item_ordered) = &flat_items[i];
            
            // Items at base level
            if *indent == base_indent {
                // Look ahead for child items (indented more)
                let mut j = i + 1;
                let mut children = Vec::new();
                
                while j < flat_items.len() && flat_items[j].0 > base_indent {
                    children.push(flat_items[j].clone());
                    j += 1;
                }
                
                if children.is_empty() {
                    // No children - add item as-is
                    result.push(item.clone());
                    i += 1;
                } else {
                    // Has children - determine nesting level
                    // Find minimum child indent (should be base + 2-4 for proper nesting)
                    let min_child_indent = children.iter().map(|(ind, _, _)| *ind).min().unwrap_or(base_indent);
                    
                    // Check if children should be nested (2-4 space increase)
                    if min_child_indent >= base_indent + 2 && min_child_indent <= base_indent + 4 {
                        // Build nested list from children (recursively)
                        let nested_items = self.build_nested_list_tree(children.clone(), min_child_indent)?;
                        
                        // Determine if nested list is ordered - check first child at min_child_indent
                        let nested_ordered = children.iter()
                            .find(|(ind, _, _)| *ind == min_child_indent)
                            .map(|(_, _, is_ord)| *is_ord)
                            .unwrap_or(false);
                        
                        // Reset indent_level for nested items (they're now relative to their parent)
                        let normalized_nested_items: Vec<Node> = nested_items.into_iter().map(|node| {
                            if let Node::ListItem { content, checked, indent_level: _, is_loose, span } = node {
                                // Reset indent to None - item is now properly nested
                                Node::list_item(content, checked, None, is_loose, span)
                            } else {
                                node
                            }
                        }).collect();
                        
                        // Create nested list node
                        let nested_span = Span { start: 0, end: 0, line: 0, column: 0 };
                        let nested_list = Node::list(
                            nested_ordered,
                            normalized_nested_items,
                            true, // Nested lists inherit tightness - simplified for now
                            None,
                            nested_span,
                        );
                        
                        // Add nested list to parent item's content
                        if let Node::ListItem { content, checked, indent_level, is_loose, span } = item {
                            let mut new_content = content.clone();
                            new_content.push(nested_list);
                            result.push(Node::list_item(new_content, *checked, *indent_level, *is_loose, span.clone()));
                        } else {
                            result.push(item.clone());
                        }
                        
                        i = j; // Skip children, we've processed them
                    } else {
                        // Children don't meet nesting criteria - treat as siblings
                        result.push(item.clone());
                        i += 1;
                    }
                }
            } else if *indent < base_indent {
                // Item has less indentation than expected - shouldn't happen
                // but handle gracefully by stopping
                break;
            } else {
                // Item has more indentation than base - skip (will be handled as child)
                i += 1;
            }
        }
        
        Ok(result)
    }
    
    /// Detect if a range of items represents an ordered or unordered list
    /// Phase 5.1: Now uses tracked is_ordered bool from grammar
    fn detect_list_type(&self, items: &[(usize, Node, bool)]) -> bool {
        // Return the is_ordered flag from the first item
        items.first().map(|(_, _, is_ordered)| *is_ordered).unwrap_or(false)
    }
    
    /// Build a list item with support for nested lists (Phase 5.1)
    fn build_list_item_with_nesting(
        &mut self,
        pair: Pair<Rule>,
        is_loose: bool,
    ) -> Result<Node, AstError> {
        let item_span = helpers::create_span(&pair);
        let item_text = pair.as_str();
        
        // Calculate base indentation (position of marker)
        let marker_indent = item_text.len() - item_text.trim_start().len();
        
        // Extract lines from the item
        let lines: Vec<&str> = item_text.lines().collect();
        if lines.is_empty() {
            return Ok(Node::list_item(vec![], None, None, is_loose, item_span));
        }
        
        // First line: extract content after marker
        let first_line_content = self.extract_list_item_content(lines[0]);
        
        // Calculate content column (where content starts after marker)
        let content_indent = self.calculate_content_indent(lines[0]);
        
        // Separate continuation lines into content vs nested lists
        let mut content_lines = vec![first_line_content];
        let mut nested_list_blocks: Vec<String> = Vec::new();
        let mut current_nested_block: Option<Vec<String>> = None;
        
        // Process continuation lines (if any)
        for (i, line) in lines.iter().skip(1).enumerate() {
            if line.trim().is_empty() {
                // Blank line - add to current context
                if let Some(ref mut block) = current_nested_block {
                    block.push(line.to_string());
                } else {
                    content_lines.push(String::new());
                }
                continue;
            }
            
            let line_indent = line.len() - line.trim_start().len();
            
            // Check if this line has a list marker
            let has_marker = self.has_list_marker(line.trim_start());
            
            // Determine if this is a nested list based on indentation
            if has_marker && line_indent >= content_indent + 2 && line_indent <= content_indent + 4 {
                // Start of nested list
                if let Some(block) = current_nested_block.take() {
                    nested_list_blocks.push(block.join("\n"));
                }
                current_nested_block = Some(vec![line.to_string()]);
            } else if let Some(ref mut block) = current_nested_block {
                // Continuation of nested list
                block.push(line.to_string());
            } else if line_indent >= content_indent {
                // Regular continuation line
                content_lines.push(line[content_indent.min(line.len())..].to_string());
            } else {
                // Line with less indentation - shouldn't happen in well-formed lists
                // but include it as content
                content_lines.push(line.trim_start().to_string());
            }
        }
        
        // Close any remaining nested block
        if let Some(block) = current_nested_block.take() {
            nested_list_blocks.push(block.join("\n"));
        }
        
        // Parse content as inline nodes
        let content_text = content_lines.join("\n");
        let mut content = self.parse_inline_content(&content_text, item_span.clone())?;
        
        // Parse nested lists and add to content
        for nested_block in nested_list_blocks {
            // Try to parse as a list
            if let Ok(nested_list) = self.parse_and_build_nested_list(&nested_block) {
                content.push(nested_list);
            }
        }
        
        Ok(Node::list_item(content, None, None, is_loose, item_span))
    }
    
    /// Check if a line starts with a list marker
    fn has_list_marker(&self, line: &str) -> bool {
        if line.is_empty() {
            return false;
        }
        
        let first_char = line.chars().next().unwrap();
        
        // Unordered markers: -, +, *
        if (first_char == '-' || first_char == '+' || first_char == '*') {
            // Check that marker is followed by space
            return line.len() > 1 && (line.chars().nth(1) == Some(' ') || line.chars().nth(1) == Some('\t'));
        }
        
        // Ordered marker: digit(s) followed by . or )
        if first_char.is_ascii_digit() {
            // Find where digits end
            let mut idx = 1;
            while idx < line.len() && line.chars().nth(idx).unwrap().is_ascii_digit() {
                idx += 1;
            }
            // Check for . or ) followed by space
            if idx < line.len() {
                let delim = line.chars().nth(idx);
                return (delim == Some('.') || delim == Some(')')) &&
                       idx + 1 < line.len() &&
                       (line.chars().nth(idx + 1) == Some(' ') || line.chars().nth(idx + 1) == Some('\t'));
            }
        }
        
        false
    }
    
    /// Calculate the column where content starts (after marker and spaces)
    fn calculate_content_indent(&self, line: &str) -> usize {
        let trimmed = line.trim_start();
        let marker_pos = line.len() - trimmed.len();
        
        // Find end of marker
        let marker_end = if trimmed.starts_with('-') || trimmed.starts_with('+') || trimmed.starts_with('*') {
            marker_pos + 1
        } else if trimmed.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            let mut idx = marker_pos;
            while idx < line.len() && line.chars().nth(idx).unwrap().is_ascii_digit() {
                idx += 1;
            }
            if idx < line.len() && (line.chars().nth(idx) == Some('.') || line.chars().nth(idx) == Some(')')) {
                idx + 1
            } else {
                marker_pos + 1
            }
        } else {
            marker_pos
        };
        
        // Skip spaces after marker
        let mut content_start = marker_end;
        while content_start < line.len() && (line.chars().nth(content_start) == Some(' ') || line.chars().nth(content_start) == Some('\t')) {
            content_start += 1;
        }
        
        content_start
    }
    
    /// Parse a nested list block (Phase 5.1)
    fn parse_and_build_nested_list(&mut self, text: &str) -> Result<Node, AstError> {
        // Legacy function - now uses unified list rule
        use crate::components::engine::parsers::block_parser;
        
        // Try to parse as unified list (supports mixed ordered/unordered)
        if let Ok(pairs) = block_parser::parse_block_rule(text, Rule::list) {
            if let Some(pair) = pairs.peek() {
                let span = helpers::create_span(&pair);
                return self.build_list_node(pair, span);
            }
        }
        
        Err(AstError::ParseError("Failed to parse nested list".to_string()))
    }
    
    /// Extract content from a list item (removes marker and indentation)
    fn extract_list_item_content(&self, item_text: &str) -> String {
        // Skip leading spaces
        let trimmed = item_text.trim_start();
        
        // Skip the list marker itself
        // For unordered: -, +, or * followed by space(s)
        // For ordered: digit(s) followed by . or ) and space(s)
        let content_start = if trimmed.starts_with('-') || trimmed.starts_with('+') || trimmed.starts_with('*') {
            // Unordered list - skip marker and following spaces
            let after_marker = &trimmed[1..];
            1 + after_marker.len() - after_marker.trim_start().len()
        } else if trimmed.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            // Ordered list - skip digits, delimiter (. or )), and spaces
            let mut idx = 0;
            while idx < trimmed.len() && trimmed.chars().nth(idx).unwrap().is_ascii_digit() {
                idx += 1;
            }
            // Skip . or )
            if idx < trimmed.len() && (trimmed.chars().nth(idx) == Some('.') || trimmed.chars().nth(idx) == Some(')')) {
                idx += 1;
            }
            // Skip spaces after delimiter
            while idx < trimmed.len() && (trimmed.chars().nth(idx) == Some(' ') || trimmed.chars().nth(idx) == Some('\t')) {
                idx += 1;
            }
            idx
        } else {
            0
        };
        
        // Return content starting after the marker, preserving newlines for lazy continuation
        if content_start < trimmed.len() {
            trimmed[content_start..].to_string()
        } else {
            String::new()
        }
    }

    /// Parse list item content, detecting and extracting nested lists
    /// Phase 5.1: This is the KEY function that makes nested lists work!
    /// 
    /// Takes raw content like "a\n  - b\n  - c" and returns:
    /// - Text node for "a"
    /// - Nested List node containing "b" and "c"
    fn parse_list_item_content_with_nesting(
        &mut self,
        content: &str,
        parent_indent: usize,
        span: Span
    ) -> Result<Vec<Node>, AstError> {
        let mut result_nodes = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        if lines.is_empty() {
            return Ok(result_nodes);
        }
        
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            let line_indent = line.len() - line.trim_start().len();
            let trimmed = line.trim_start();
            
            // Check if this line is a nested list marker (2-4 spaces more than parent)
            let indent_diff = line_indent.saturating_sub(parent_indent);
            let has_marker = trimmed.starts_with('-') || trimmed.starts_with('+') || trimmed.starts_with('*') ||
                            trimmed.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false);
            
            if has_marker && indent_diff >= 2 && indent_diff <= 4 {
                // This is a nested list item! Collect all consecutive nested items
                let mut nested_lines = vec![line];
                i += 1;
                
                while i < lines.len() {
                    let next_line = lines[i];
                    let next_indent = next_line.len() - next_line.trim_start().len();
                    let next_indent_diff = next_indent.saturating_sub(parent_indent);
                    
                    // Continue if indented at same or deeper level
                    if next_indent_diff >= indent_diff {
                        nested_lines.push(next_line);
                        i += 1;
                    } else {
                        break;
                    }
                }
                
                // Parse the nested lines as a list
                let nested_text = nested_lines.join("\n");
                if let Ok(nested_list) = self.parse_and_build_nested_list(&nested_text) {
                    result_nodes.push(nested_list);
                }
            } else {
                // Regular content line - parse as inline
                if !line.trim().is_empty() {
                    let inline_nodes = self.parse_inline_content(line, span.clone())?;
                    result_nodes.extend(inline_nodes);
                }
                i += 1;
            }
        }
        
        // If we only got text nodes and no nested lists, just parse the whole thing as inline
        if result_nodes.is_empty() || result_nodes.iter().all(|n| matches!(n, Node::Text { .. })) {
            return self.parse_inline_content(content, span);
        }
        
        Ok(result_nodes)
    }

    /// Detect if a list is tight (no blank lines between items) or loose (has blank lines)
    fn detect_list_tightness(&self, pair: &Pair<Rule>) -> bool {
        let list_text = pair.as_str();
        
        // A list is loose if it contains double newlines (blank lines) between items
        // Simple heuristic: check for "\n\n" patterns
        !list_text.contains("\n\n")
    }

    /// Extract the starting number from an ordered list's first item
    fn extract_starting_number(&self, pair: &Pair<Rule>) -> Option<usize> {
        let list_text = pair.as_str();
        
        // Find the first line and extract the number before '.' or ')'
        if let Some(first_line) = list_text.lines().next() {
            // Try to extract digits from the start of the line (after optional spaces)
            let trimmed = first_line.trim_start();
            let num_str: String = trimmed.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            
            if let Ok(num) = num_str.parse::<usize>() {
                // Only return Some if not starting at 1 (1 is default)
                if num != 1 {
                    return Some(num);
                }
            }
        }
        
        None
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

    /// Extract HTML block content and determine block type
    /// 
    /// HTML blocks have 7 types per CommonMark spec section 4.6:
    /// - Type 1: script, pre, style, textarea tags
    /// - Type 2: HTML comments
    /// - Type 3: Processing instructions
    /// - Type 4: Declarations (e.g., <!DOCTYPE>)
    /// - Type 5: CDATA sections
    /// - Type 6: Block-level tags (div, table, etc.)
    /// - Type 7: Generic tags
    fn extract_html_block(&self, pair: Pair<Rule>, span: Span) -> Result<Node, AstError> {
        let block_type = match pair.as_rule() {
            Rule::html_block_type1 => 1,
            Rule::html_block_type2 => 2,
            Rule::html_block_type3 => 3,
            Rule::html_block_type4 => 4,
            Rule::html_block_type5 => 5,
            Rule::html_block_type6 => 6,
            Rule::html_block_type7 => 7,
            _ => {
                return Err(AstError::InvalidStructure(format!(
                    "Unknown HTML block type: {:?}",
                    pair.as_rule()
                )))
            }
        };
        
        // Get the raw HTML content - preserve it exactly as-is
        let content = pair.as_str().to_string();
        
        Ok(Node::html_block(block_type, content, span))
    }

    /// Parse inline content using InlineParser
    /// 
    /// This method integrates the two-stage parser by taking block-level text content
    /// and parsing it for inline elements (bold, italic, links, images, etc.)
    fn parse_inline_content(&self, text: &str, span: Span) -> Result<Vec<Node>, AstError> {
        use crate::components::engine::parsers::inline_parser::{InlineParser, Rule as InlineRule};
        use pest::Parser;
        
        // Handle empty text
        if text.trim().is_empty() {
            return Ok(vec![Node::text(String::new(), span)]);
        }
        
        // Parse inline content
        let mut pairs = InlineParser::parse(InlineRule::inline_content, text)
            .map_err(|e| AstError::ParseError(format!("Inline parse failed: {}", e)))?;
        
        // Get the inline_content pair (the top-level rule)
        let inline_content_pair = pairs.next()
            .ok_or_else(|| AstError::MissingContent("No inline_content pair found".to_string()))?;
        
        // Build inline AST nodes from the children of inline_content
        let inline_builder = InlineBuilder::new();
        let mut nodes = Vec::new();
        
        // Iterate through the children of inline_content
        for pair in inline_content_pair.into_inner() {
            // Try to build each inline node
            match inline_builder.build_inline_node(pair) {
                Ok(node) => nodes.push(node),
                Err(e) => {
                    // On error, fallback to raw text with warning
                    eprintln!("Warning: Inline parse error: {}, falling back to raw text", e);
                    return Ok(vec![Node::text(text.to_string(), span)]);
                }
            }
        }
        
        // If no nodes were produced, create a text node
        if nodes.is_empty() {
            Ok(vec![Node::text(text.to_string(), span)])
        } else {
            Ok(nodes)
        }
    }

    /// Phase 5.1: Post-process document to merge indented lists into parent lists
    /// This handles nested lists with different marker types (e.g., "- a\n  1. b")
    fn post_process_nested_lists(&mut self, nodes: Vec<Node>) -> Vec<Node> {
        let mut result = Vec::new();
        let mut i = 0;
        
        while i < nodes.len() {
            if let Node::List { items, ordered, is_tight, start_number, span } = &nodes[i] {
                // Look ahead for a consecutive list that might be nested
                if i + 1 < nodes.len() {
                    if let Node::List { items: child_items, ordered: child_ordered, is_tight: child_tight, start_number: child_start, span: child_span } = &nodes[i + 1] {
                        // Check if child list is indented relative to parent
                        if let Some(parent_last_indent) = self.get_first_item_indent(items) {
                            if let Some(child_first_indent) = self.get_first_item_indent(child_items) {
                                // Nested if child is indented 2-4 spaces more than parent
                                if child_first_indent >= parent_last_indent + 2 && child_first_indent <= parent_last_indent + 4 {
                                    // Merge child list into parent's last item
                                    let merged = self.merge_child_list_into_parent(
                                        items.clone(), *ordered, *is_tight, *start_number, span.clone(),
                                        child_items.clone(), *child_ordered, *child_tight, *child_start, child_span.clone()
                                    );
                                    result.push(merged);
                                    i += 2; // Skip both lists
                                    continue;
                                }
                            }
                        }
                    }
                }
                // Not merged, add as-is
                result.push(nodes[i].clone());
                i += 1;
            } else {
                // Not a list, add as-is
                result.push(nodes[i].clone());
                i += 1;
            }
        }
        
        result
    }

    /// Get indentation level of first item in a list
    fn get_first_item_indent(&self, items: &[Node]) -> Option<usize> {
        items.first().and_then(|item| {
            if let Node::ListItem { indent_level, .. } = item {
                // indent_level is Option<u8>, convert to usize
                indent_level.map(|l| l as usize)
            } else {
                None
            }
        }).or(Some(0)) // Default to 0 if not set
    }

    /// Merge child list into parent list's last item
    fn merge_child_list_into_parent(
        &mut self,
        parent_items: Vec<Node>,
        parent_ordered: bool,
        parent_tight: bool,
        parent_start: Option<usize>,
        parent_span: Span,
        child_items: Vec<Node>,
        child_ordered: bool,
        child_tight: bool,
        child_start: Option<usize>,
        child_span: Span,
    ) -> Node {
        let mut new_parent_items = parent_items;
        
        // Get last item and add child list to its content
        if let Some(last_item) = new_parent_items.pop() {
            if let Node::ListItem { content, checked, indent_level, is_loose, span } = last_item {
                let mut new_content = content;
                
                // Create nested list node
                let nested_list = Node::List {
                    items: child_items,
                    ordered: child_ordered,
                    is_tight: child_tight,
                    start_number: child_start,
                    span: child_span,
                };
                
                new_content.push(nested_list);
                
                // Recreate parent item with nested list
                let updated_item = Node::list_item(new_content, checked, indent_level, is_loose, span);
                new_parent_items.push(updated_item);
            }
        }
        
        Node::List {
            items: new_parent_items,
            ordered: parent_ordered,
            is_tight: parent_tight,
            start_number: parent_start,
            span: parent_span,
        }
    }
}

impl Default for BlockBuilder {
    fn default() -> Self {
        Self::new()
    }
}
