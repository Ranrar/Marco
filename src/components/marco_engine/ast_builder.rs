//! Enhanced AST Builder with comprehensive rule-to-node mapping
//!
//! This module provides complete AST building functionality for Marco markup:
//! - Direct Pest grammar rule to AST node mapping
//! - Full support for all Marco extensions
//! - Proper error handling and recovery with Unknown nodes
//! - Accurate span tracking for source position information
//! - Helper functions for complex parsing operations

// Re-export key types from ast_node
pub use crate::components::marco_engine::ast_node::{Node, Span};

// Enhanced AstBuilder with complete feature support
use crate::components::marco_engine::grammar::Rule;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use std::collections::HashMap;

/// Enhanced AST Builder with comprehensive rule coverage
pub struct AstBuilder {
    /// Cache for efficient span creation (currently unused but planned for optimization)
    #[allow(dead_code)]
    span_cache: HashMap<String, Span>,
    /// Current nesting depth for nested code blocks
    nesting_depth: u8,
    /// Maximum allowed nesting depth to prevent infinite recursion
    max_nesting_depth: u8,
}

/// Error type for AST building operations
#[derive(Debug, Clone)]
pub enum AstError {
    ParseError(String),
    InvalidRule(String),
    MissingContent(String),
    UnsupportedFeature(String),
}

impl Default for AstBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AstBuilder {
    /// Create a new AST builder
    pub fn new() -> Self {
        Self {
            span_cache: HashMap::new(),
            nesting_depth: 0,
            max_nesting_depth: 10, // Reasonable limit to prevent stack overflow
        }
    }

    /// Build AST from Pest pairs - enhanced with comprehensive rule mapping
    pub fn build(pairs: Pairs<Rule>) -> Result<Node, String> {
        let mut builder = Self::new();
        let mut document_children = Vec::new();

        for pair in pairs {
            match pair.as_rule() {
                Rule::document => {
                    // Extract children from the document rule, filtering out NEWLINE tokens
                    for inner_pair in pair.into_inner() {
                        // Skip NEWLINE tokens as they're only needed for parsing structure
                        if inner_pair.as_rule() == Rule::NEWLINE {
                            continue;
                        }
                        let child = builder
                            .build_node(inner_pair)
                            .map_err(|e| format!("{:?}", e))?;
                        document_children.push(child);
                    }
                }
                Rule::NEWLINE => {
                    // Skip standalone NEWLINE tokens
                    continue;
                }
                _ => {
                    // Handle single non-document rules
                    let child = builder.build_node(pair).map_err(|e| format!("{:?}", e))?;
                    document_children.push(child);
                }
            }
        }

        let span = Span::new(0, 0, 1, 1);
        Ok(Node::document(document_children, span))
    }

    /// Build a single AST node from a Pest pair with comprehensive rule handling
    fn build_node(&mut self, pair: Pair<Rule>) -> Result<Node, AstError> {
        let span = self.create_span(&pair);

        match pair.as_rule() {
            // ===========================================
            // DOCUMENT STRUCTURE
            // ===========================================
            Rule::document => {
                // Document should just collect block children, not create nested documents
                let children = self.build_children(pair)?;
                Ok(Node::document(children, span))
            }

            // ===========================================
            // BLOCK ELEMENTS
            // ===========================================
            Rule::paragraph => {
                // Get paragraph text before processing
                let paragraph_text = pair.as_str().to_string();

                // Calculate indentation from the span position
                let indent_level = self.calculate_indent_from_span(&pair.as_span());

                // Paragraph should collect inline content from paragraph_line
                let mut inline_content = Vec::new();
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::paragraph_line => {
                            // Extract inline content from paragraph_line
                            let line_content = self.extract_inline_content(inner_pair)?;
                            inline_content.extend(line_content);
                        }
                        _ => {
                            // Handle other paragraph content
                            let child = self.build_node(inner_pair)?;
                            inline_content.push(child);
                        }
                    }
                }

                // Post-process to detect hard line breaks
                // Look for pattern: text ending with 2+ spaces + soft line break
                inline_content = self.detect_hard_line_breaks(inline_content, &paragraph_text)?;

                // Fix whitespace boundary issues between text and formatting
                inline_content = self.fix_whitespace_boundaries(inline_content, &paragraph_text)?;

                Ok(Node::paragraph(inline_content, indent_level, span))
            }

            // Heading rules - unified handling for all heading types
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
                let content = self.extract_heading_content(pair)?;
                Ok(Node::heading(level, content, span))
            }

            // Code blocks - unified handling with nested support
            Rule::code_block | Rule::fenced_code | Rule::indented_code => {
                // Calculate indentation from the span position
                let indent_level = self.calculate_indent_from_span(&pair.as_span());

                let (language, content) = self.extract_code_content(&pair)?;

                // Check if this is a nested fenced code block
                if let Some((nested_node, _nesting_level)) =
                    self.try_build_nested_code_block(&language, &content, span.clone())?
                {
                    Ok(nested_node)
                } else {
                    Ok(Node::code_block(language, content, indent_level, span))
                }
            }

            // Math blocks
            Rule::math_block => {
                let content = self.extract_math_content(&pair)?;
                Ok(Node::math_block(content, span))
            }

            // Lists - now with separate ordered/unordered rules
            Rule::list => {
                // Rule::list contains either unordered_list or ordered_list
                // Process the inner list directly to avoid double nesting
                let mut inner_pairs = pair.into_inner();
                if let Some(inner_pair) = inner_pairs.next() {
                    // Should always be exactly one inner pair
                    self.build_node(inner_pair)
                } else {
                    // Fallback - empty list
                    Ok(Node::list(false, vec![], span))
                }
            }

            Rule::unordered_list | Rule::ordered_list => {
                let ordered = match pair.as_rule() {
                    Rule::ordered_list => true,
                    Rule::unordered_list => false,
                    _ => unreachable!(), // Should never happen
                };

                // Collect list items directly
                let mut items = Vec::new();
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::unordered_list_item | Rule::ordered_list_item => {
                            let item = self.build_node(inner_pair)?;
                            items.push(item);
                        }
                        _ => {
                            // Skip any other elements to prevent double nesting
                        }
                    }
                }

                Ok(Node::list(ordered, items, span))
            }

            Rule::unordered_list_item | Rule::ordered_list_item => {
                // Calculate indentation level from the span position
                let indent_level = self.calculate_indent_from_span(&pair.as_span());

                let checked = self.extract_task_state(&pair)?;
                let mut content = self.build_children(pair)?;

                // If the content is a single paragraph containing inline elements,
                // unwrap the paragraph to get direct inline content for the list item
                if content.len() == 1 {
                    if let Node::Paragraph {
                        content: para_content,
                        ..
                    } = &content[0]
                    {
                        content = para_content.clone();
                    }
                }

                Ok(Node::list_item(content, checked, indent_level, span))
            }

            // Standalone task blocks (tasks without list markers)
            Rule::task_block => {
                // Calculate indentation level from the span position
                let indent_level = self.calculate_indent_from_span(&pair.as_span());

                let checked = self.extract_task_state(&pair)?;
                let content = self.extract_task_content(&pair)?;

                // Create a list item for consistency with list-based tasks
                // This allows the same rendering logic to work for both list and standalone tasks
                Ok(Node::list_item(content, checked, indent_level, span))
            }

            // Tables
            Rule::table => {
                let (headers, rows) = self.extract_table_data(pair)?;
                Ok(Node::table(headers, rows, span))
            }

            Rule::table_row => {
                let cells = self.build_children(pair)?;
                // Return as a temporary structure - tables handle this differently
                Ok(Node::paragraph(cells, None, span)) // Temporary wrapper
            }

            Rule::table_cell => {
                let alignment = self.extract_table_alignment(&pair)?;
                let content = self.build_children(pair)?;
                Ok(Node::table_cell(content, alignment, span))
            }

            Rule::table_cell_content => {
                // Table cell content is similar to inline content
                let content = self.build_children(pair)?;
                if content.len() == 1 {
                    Ok(content.into_iter().next().unwrap())
                } else if content.is_empty() {
                    Ok(Node::text("".to_string(), span))
                } else {
                    // Multiple children - wrap in paragraph
                    Ok(Node::paragraph(content, None, span))
                }
            }

            Rule::table_safe_text => {
                let content = pair.as_str().trim().to_string();
                Ok(Node::text(content, span))
            }

            // Block quotes
            Rule::blockquote => {
                // Calculate indentation from the span position
                let indent_level = self.calculate_indent_from_span(&pair.as_span());

                let content = self.build_children(pair)?;

                // Process blockquote content to handle line breaks properly
                let mut processed_content = Vec::new();

                for child in content {
                    match child {
                        // If we have a paragraph with text + line break, flatten it
                        Node::Paragraph {
                            content: para_content,
                            ..
                        } if para_content.len() == 2 => {
                            if matches!(&para_content[1], Node::LineBreak { .. }) {
                                for item in para_content {
                                    processed_content.push(item);
                                }
                            } else {
                                processed_content.push(Node::Paragraph {
                                    content: para_content,
                                    indent_level: None,
                                    span: span.clone(),
                                });
                            }
                        }
                        _ => processed_content.push(child),
                    }
                }

                Ok(Node::block_quote(processed_content, indent_level, span))
            }

            // Blockquote content (the text after > marker)
            Rule::blockquote_content => {
                let content_str = pair.as_str();

                // Check if content ends with backslash (hard line break)
                if let Some(text_content) = content_str.strip_suffix('\\') {
                    // Remove the backslash
                    if text_content.trim().is_empty() {
                        // Just a backslash on empty line - return hard line break
                        Ok(Node::line_break(
                            crate::components::marco_engine::ast_node::LineBreakType::Hard,
                            span,
                        ))
                    } else {
                        // Text followed by backslash - return text + hard line break wrapped in paragraph for processing
                        Ok(Node::paragraph(
                            vec![
                                Node::text(text_content.to_string(), span.clone()),
                                Node::line_break(
                                    crate::components::marco_engine::ast_node::LineBreakType::Hard,
                                    span.clone(),
                                ),
                            ],
                            None,
                            span,
                        ))
                    }
                } else {
                    // Regular content - just return as text
                    if content_str.trim().is_empty() {
                        Ok(Node::text("".to_string(), span))
                    } else {
                        Ok(Node::text(content_str.to_string(), span))
                    }
                }
            }

            // Blockquote line (> marker + content) - extract just the content
            Rule::blockquote_line => {
                let content = self.build_children(pair)?;
                // Return content directly, filtering out any empty nodes
                let non_empty: Vec<Node> = content.into_iter()
                    .filter(|node| !matches!(node, Node::Text { content, .. } if content.trim().is_empty()))
                    .collect();

                if non_empty.len() == 1 {
                    Ok(non_empty.into_iter().next().unwrap())
                } else if non_empty.is_empty() {
                    // Empty blockquote line, create empty text node
                    Ok(Node::text("".to_string(), span))
                } else {
                    // Multiple items, wrap in paragraph
                    Ok(Node::paragraph(non_empty, None, span))
                }
            }

            // Horizontal rules
            Rule::hr => Ok(Node::horizontal_rule(span)),

            // ===========================================
            // INLINE ELEMENTS
            // ===========================================
            Rule::text | Rule::text_no_newline => {
                let content = pair.as_str().to_string();
                Ok(Node::text(content, span))
            }

            Rule::code_inline => {
                let content = self.extract_inline_code_content(&pair)?;
                Ok(Node::code(content, span))
            }

            Rule::math_inline => {
                let content = self.extract_inline_math_content(&pair)?;
                Ok(Node::math_inline(content, span))
            }

            Rule::emoji => {
                let (shortcode, unicode) = self.extract_emoji_content(&pair)?;
                Ok(Node::emoji(shortcode, unicode, span))
            }

            // Emphasis handling - only handle specific types to avoid nesting
            Rule::bold_asterisk | Rule::bold_underscore => {
                let content = self.extract_emphasis_content_from_structure(&pair)?;
                Ok(Node::strong(content, span))
            }

            Rule::italic_asterisk | Rule::italic_underscore => {
                let content = self.extract_emphasis_content_from_structure(&pair)?;
                Ok(Node::emphasis(content, span))
            }

            Rule::bold_italic_triple_asterisk
            | Rule::bold_italic_triple_underscore
            | Rule::bold_italic_mixed_ast_under
            | Rule::bold_italic_mixed_under_ast
            | Rule::bold_italic_triple_mixed_au
            | Rule::bold_italic_triple_mixed_ua => {
                let content = self.extract_emphasis_content_from_structure(&pair)?;
                Ok(Node::strong(
                    vec![Node::emphasis(content, span.clone())],
                    span,
                ))
            }

            // Container emphasis rules - delegate to children
            Rule::emphasis | Rule::bold | Rule::italic | Rule::bold_italic => {
                // These are container rules - process their children instead
                let text = pair.as_str().to_string(); // Get the text before moving pair
                let children = self.build_children(pair)?;
                if children.len() == 1 {
                    Ok(children.into_iter().next().unwrap())
                } else if children.is_empty() {
                    Ok(Node::text("".to_string(), span))
                } else {
                    // Multiple children - this shouldn't happen but handle gracefully
                    Ok(Node::text(text, span))
                }
            }

            Rule::strikethrough => {
                // Handle strikethrough content extraction from atomic rules
                let content = self.extract_strikethrough_content(&pair)?;
                Ok(Node::strikethrough(
                    vec![Node::text(content, span.clone())],
                    span,
                ))
            }

            Rule::highlight => {
                // Extract content from atomic highlight rule like emphasis does
                let text = pair.as_str();
                let content_text =
                    if text.len() >= 4 && text.starts_with("==") && text.ends_with("==") {
                        text[2..text.len() - 2].to_string()
                    } else if text.len() >= 2 && text.starts_with("==") {
                        // Incomplete highlight (missing end markers)
                        text[2..].to_string()
                    } else {
                        text.to_string()
                    };

                // Create a text node with the extracted content
                let content_node = Node::text(content_text, span.clone());
                Ok(Node::highlight(vec![content_node], span))
            }

            Rule::superscript => {
                let content = self.extract_superscript_content(&pair)?;
                let content_node = Node::text(content, span.clone());
                Ok(Node::superscript(vec![content_node], span))
            }

            Rule::subscript => {
                let content = self.extract_subscript_content(&pair)?;
                let content_node = Node::text(content, span.clone());
                Ok(Node::subscript(vec![content_node], span))
            }

            // Links and images
            Rule::inline_link => {
                let (text, url, title) = self.extract_link_parts(pair)?;
                Ok(Node::link(text, url, title, span))
            }

            Rule::inline_image => {
                let (alt, url, title) = self.extract_image_parts(pair)?;
                Ok(Node::image(alt, url, title, span))
            }

            Rule::autolink => {
                let url = pair.as_str().to_string();
                let text = vec![Node::text(url.clone(), span.clone())];
                Ok(Node::link(text, url, None, span))
            }

            // Line breaks and escaped characters
            Rule::hard_line_break => Ok(Node::hard_line_break(span)),

            Rule::soft_line_break => Ok(Node::soft_line_break(span)),

            Rule::escaped_char => {
                let character = self.extract_escaped_character(&pair)?;
                Ok(Node::escaped_char(character, span))
            }

            // ===========================================
            // FOOTNOTES AND REFERENCES
            // ===========================================

            // Footnote definition: [^label]: content
            Rule::footnote_def => {
                let (label, content) = self.extract_footnote_def_parts(pair)?;
                Ok(Node::FootnoteDef {
                    label,
                    content,
                    span,
                })
            }

            // Footnote reference: [^label]
            Rule::footnote_ref => {
                let label = self.extract_footnote_ref_label(&pair)?;
                Ok(Node::FootnoteRef { label, span })
            }

            // Inline footnote: ^[content]
            Rule::inline_footnote_ref => {
                let content = self.extract_inline_footnote_content(pair)?;
                Ok(Node::InlineFootnoteRef { content, span })
            }

            // Reference definition: [label]: url "title"
            Rule::reference_definition => {
                let (label, url, title) = self.extract_reference_def_parts(pair)?;
                Ok(Node::ReferenceDefinition {
                    label,
                    url,
                    title,
                    span,
                })
            }

            // Reference link: [text][label]
            Rule::reference_link => {
                let (text, label) = self.extract_reference_link_parts(pair)?;
                Ok(Node::ReferenceLink { text, label, span })
            }

            // Reference image: ![alt][label]
            Rule::reference_image => {
                let (alt, label) = self.extract_reference_image_parts(pair)?;
                Ok(Node::ReferenceImage { alt, label, span })
            }

            // ===========================================
            // HTML ELEMENTS
            // ===========================================

            // Block HTML: <div>...</div>
            Rule::block_html => {
                let content = pair.as_str().to_string();
                Ok(Node::HtmlBlock { content, span })
            }

            // ===========================================
            // DEFINITION LISTS
            // ===========================================

            // Definition list: term\n: definition
            Rule::def_list => {
                let items = self.build_children(pair)?;
                Ok(Node::DefinitionList { items, span })
            }

            // ===========================================
            // MARCO EXTENSIONS
            // ===========================================

            // User mentions: @username[platform](Display Name)
            Rule::user_mention => {
                let (username, platform, display_name) = self.extract_user_mention_parts(pair)?;
                Ok(Node::UserMention {
                    username,
                    platform,
                    display_name,
                    span,
                })
            }

            // Bookmarks: [bookmark:label](path=line)
            Rule::bookmark => {
                let (label, path, line) = self.extract_bookmark_parts(pair)?;
                Ok(Node::Bookmark {
                    label,
                    path,
                    line,
                    span,
                })
            }

            // Admonitions: :::note[title], :::warning, etc.
            Rule::admonition_block => {
                let (kind, _title, content) = self.extract_admonition_parts(pair)?;
                // Note: Current Admonition node doesn't support title, only kind and content
                Ok(Node::Admonition {
                    kind,
                    content,
                    span,
                })
            }

            // Tab blocks: :::tab[title] with @tab sections
            Rule::tab_block => {
                let (title, tabs) = self.extract_tab_block_parts(pair)?;
                Ok(Node::TabBlock { title, tabs, span })
            }

            Rule::tab => {
                let (name, content) = self.extract_tab_parts(pair)?;
                Ok(Node::Tab {
                    name,
                    content,
                    span,
                })
            }

            // Table of Contents: [toc], [toc=2], [toc=3](@doc)
            Rule::toc => {
                let (depth, document) = self.extract_toc_parts(pair)?;
                Ok(Node::TableOfContents {
                    depth,
                    document,
                    span,
                })
            }

            // Executable code: run@bash(command)
            Rule::run_inline => {
                let (script_type, command) = self.extract_run_inline_parts(pair)?;
                Ok(Node::RunInline {
                    script_type,
                    command,
                    span,
                })
            }

            // Executable code blocks: ```run@python
            Rule::run_block_fenced => {
                let (script_type, content) = self.extract_run_block_parts(pair)?;
                Ok(Node::RunBlock {
                    script_type,
                    content,
                    span,
                })
            }

            // Diagram blocks: ```mermaid, ```graphviz
            Rule::diagram_fenced => {
                let (diagram_type, content) = self.extract_diagram_parts(pair)?;
                Ok(Node::DiagramBlock {
                    diagram_type,
                    content,
                    span,
                })
            }

            // ===========================================
            // ERROR RECOVERY
            // ===========================================
            Rule::unknown_block => {
                let content = pair.as_str().to_string();
                let rule = "unknown_block".to_string();
                Ok(Node::unknown(content, rule, span))
            }

            // ===========================================
            // COMPOUND RULES (contain other rules)
            // ===========================================
            Rule::block => {
                // Block is a container - extract the actual block type
                let mut inner_pairs = pair.into_inner();
                if let Some(inner_pair) = inner_pairs.next() {
                    return self.build_node(inner_pair);
                }
                Err(AstError::MissingContent("Empty block rule".to_string()))
            }

            Rule::inline_core | Rule::inline | Rule::inline_no_newline => {
                // Inline is a container - extract the actual inline type
                let mut inner_pairs = pair.into_inner();
                if let Some(inner_pair) = inner_pairs.next() {
                    return self.build_node(inner_pair);
                }
                Err(AstError::MissingContent("Empty inline rule".to_string()))
            }

            Rule::macro_block => {
                // Macro block is a container for Marco extensions
                let mut inner_pairs = pair.into_inner();
                if let Some(inner_pair) = inner_pairs.next() {
                    return self.build_node(inner_pair);
                }
                Err(AstError::MissingContent(
                    "Empty macro_block rule".to_string(),
                ))
            }

            Rule::macro_inline => {
                // Macro inline is a container for inline Marco extensions
                let mut inner_pairs = pair.into_inner();
                if let Some(inner_pair) = inner_pairs.next() {
                    return self.build_node(inner_pair);
                }
                Err(AstError::MissingContent(
                    "Empty macro_inline rule".to_string(),
                ))
            }

            // ===========================================
            // CONTENT EXTRACTION RULES
            // ===========================================
            Rule::heading_content => {
                // heading_content now contains inline+ directly (full CommonMark inline support)
                let content = self.build_children(pair)?;
                // This is handled by heading rules - return combined content
                if content.len() == 1 {
                    Ok(content.into_iter().next().unwrap())
                } else {
                    Ok(Node::paragraph(content, None, span))
                }
            }

            Rule::heading_text => {
                // heading_text contains inline_core - process the inline content
                let content = self.build_children(pair)?;
                if content.len() == 1 {
                    Ok(content.into_iter().next().unwrap())
                } else {
                    // Multiple inline elements - wrap in a container
                    Ok(Node::paragraph(content, None, span))
                }
            }

            Rule::word => Ok(Node::text(pair.as_str().to_string(), span)),

            Rule::paragraph_line => {
                // Extract inline content instead of creating another paragraph
                let content = self.extract_inline_content(pair)?;
                if content.len() == 1 {
                    Ok(content.into_iter().next().unwrap())
                } else {
                    Ok(Node::paragraph(content, None, span))
                }
            }

            Rule::list_item_content => {
                // Parse the content as inline elements and return them directly
                let text = pair.as_str();

                // Try to parse the text content as inline markup
                if let Ok(pairs) = crate::components::marco_engine::parser::MarcoParser::parse(
                    crate::components::marco_engine::parser::Rule::paragraph_line,
                    text,
                ) {
                    // Build inline nodes from the parsed content WITH gap detection
                    let mut children = Vec::new();
                    for inline_pair in pairs {
                        // Use extract_inline_content for proper gap detection
                        let inline_content = self.extract_inline_content(inline_pair)?;
                        children.extend(inline_content);
                    }

                    // For single inline elements, return directly
                    if children.len() == 1 {
                        return Ok(children.into_iter().next().unwrap());
                    } else if !children.is_empty() {
                        // For multiple inline elements, we need to return them somehow
                        // Since we can only return one Node, we'll create a temporary container
                        // that the parent can unwrap
                        return Ok(Node::Paragraph {
                            content: children,
                            indent_level: None,
                            span,
                        });
                    }
                }

                // Fallback to plain text if inline parsing fails
                Ok(Node::text(text.to_string(), span))
            }

            Rule::unordered_regular_list_item | Rule::ordered_regular_list_item => {
                // Extract the list_item_content
                let text = pair.as_str(); // Get text before moving pair
                for inner_pair in pair.into_inner() {
                    if inner_pair.as_rule() == Rule::list_item_content {
                        return self.build_node(inner_pair);
                    }
                }
                // Fallback
                Ok(Node::text(text.to_string(), span))
            }

            Rule::unordered_task_list_item | Rule::ordered_task_list_item => {
                // Similar to regular_list_item but with task marker handling
                let text = pair.as_str(); // Get text before moving pair
                for inner_pair in pair.into_inner() {
                    if inner_pair.as_rule() == Rule::list_item_content {
                        return self.build_node(inner_pair);
                    }
                }
                Ok(Node::text(text.to_string(), span))
            }

            // ===========================================
            // TAB BLOCK INTERNAL RULES
            // ===========================================
            Rule::tab_header => {
                // Tab header should be handled by tab_block processing
                let span = self.create_span(&pair);
                let title = self.extract_tab_header_title(pair)?;
                if let Some(title_text) = title {
                    Ok(Node::text(title_text, span))
                } else {
                    Ok(Node::text("Tab".to_string(), span))
                }
            }

            Rule::tabs_content => {
                // tabs_content is a container - process its children
                let content = self.extract_tabs_content(pair)?;
                if content.len() == 1 {
                    Ok(content.into_iter().next().unwrap())
                } else {
                    Ok(Node::paragraph(content, None, span))
                }
            }

            Rule::tab_line => {
                // Tab line should be handled by tab processing
                let span = self.create_span(&pair);
                let name = self.extract_tab_line_name(pair)?;
                if let Some(tab_name) = name {
                    Ok(Node::text(format!("@tab {}", tab_name), span))
                } else {
                    Ok(Node::text("@tab".to_string(), span))
                }
            }

            Rule::tab_content_line => {
                // Tab content line is general content
                let content = pair.as_str().trim().to_string();
                Ok(Node::text(content, span))
            }

            Rule::tab_body => {
                // Tab body container - process as text for now
                let content = pair.as_str().trim().to_string();
                if content.is_empty() {
                    Ok(Node::text("".to_string(), span))
                } else {
                    Ok(Node::text(content, span))
                }
            }

            Rule::tab_title => {
                // Tab title is text content
                let content = pair.as_str().trim().to_string();
                Ok(Node::text(content, span))
            }

            Rule::tab_name => {
                // Tab name is text content
                let content = pair.as_str().trim().to_string();
                Ok(Node::text(content, span))
            }

            Rule::tab_end => {
                // Tab end delimiter - no content
                Ok(Node::text("".to_string(), span))
            }

            // ===========================================
            // SILENT RULES AND HELPERS
            // ===========================================
            
            _ => {
                // Handle any remaining rules as unknown for error recovery
                let content = pair.as_str().to_string();
                let rule_name = format!("{:?}", pair.as_rule());

                // Log unhandled rules for debugging
                log::debug!(
                    "Unhandled rule in AST builder: {} with content: {}",
                    rule_name,
                    content
                );

                Ok(Node::unknown(content, rule_name, span))
            }
        }
    }
    // ===========================================
    // HELPER FUNCTIONS FOR CONTENT EXTRACTION
    // ===========================================

    /// Build children nodes from a pair
    fn build_children(&mut self, pair: Pair<Rule>) -> Result<Vec<Node>, AstError> {
        let mut children = Vec::new();
        for inner_pair in pair.into_inner() {
            let child = self.build_node(inner_pair)?;
            children.push(child);
        }
        Ok(children)
    }

    /// Extract inline content from paragraph lines and inline containers
    fn extract_inline_content(&mut self, pair: Pair<Rule>) -> Result<Vec<Node>, AstError> {
        let mut inline_nodes = Vec::new();
        let parent_span = pair.as_span();
        let parent_content = pair.as_str();

        let mut last_end_pos = parent_span.start();

        for inner_pair in pair.into_inner() {
            let current_start = inner_pair.as_span().start();
            let current_end = inner_pair.as_span().end();

            // Check if there's a gap between last element and current element
            if current_start > last_end_pos {
                // Extract the gap content (whitespace that was consumed by Pest)
                let gap_start = last_end_pos - parent_span.start();
                let gap_end = current_start - parent_span.start();
                if gap_start < parent_content.len() && gap_end <= parent_content.len() {
                    let gap_content = &parent_content[gap_start..gap_end];
                    if !gap_content.is_empty() {
                        // Create a text node for the gap - use approximate line/column
                        let gap_span = Span::new(last_end_pos as u32, current_start as u32, 0, 0);
                        inline_nodes.push(Node::text(gap_content.to_string(), gap_span));
                    }
                }
            }

            match inner_pair.as_rule() {
                Rule::inline | Rule::inline_core | Rule::inline_no_newline => {
                    // Recursively extract from inline containers
                    let nested_content = self.extract_inline_content(inner_pair)?;
                    inline_nodes.extend(nested_content);
                }
                Rule::text | Rule::text_no_newline => {
                    let span = self.create_span(&inner_pair);
                    inline_nodes.push(Node::text(inner_pair.as_str().to_string(), span));
                }
                _ => {
                    // Handle other inline elements
                    let child = self.build_node(inner_pair)?;
                    inline_nodes.push(child);
                }
            }

            last_end_pos = current_end;
        }

        // Check if there's content after the last element
        if last_end_pos < parent_span.end() {
            let gap_start = last_end_pos - parent_span.start();
            if gap_start < parent_content.len() {
                let remaining_content = &parent_content[gap_start..];
                if !remaining_content.is_empty() {
                    let gap_span = Span::new(last_end_pos as u32, parent_span.end() as u32, 0, 0);
                    inline_nodes.push(Node::text(remaining_content.to_string(), gap_span));
                }
            }
        }

        Ok(inline_nodes)
    }

    /// Create span from pest pair with proper line/column tracking
    fn create_span(&mut self, pair: &Pair<Rule>) -> Span {
        Span::from_pest(pair)
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
                } else {
                    // Check for setext heading
                    if text.contains('=') {
                        Ok(1)
                    } else if text.contains('-') {
                        Ok(2)
                    } else {
                        Ok(1) // Default
                    }
                }
            }
            _ => Ok(1), // Default to level 1
        }
    }

    /// Count leading indentation from span column information
    /// Now supports both tabs and spaces: 1 tab = 1 indent level, 4 spaces = 1 indent level
    fn calculate_indent_from_span(&self, span: &pest::Span) -> Option<u8> {
        let (line_num, column) = span.start_pos().line_col();
        if column > 1 {
            // Get the full input to analyze the actual leading whitespace
            let full_input = span.get_input();

            // Find the line containing this span
            let lines: Vec<&str> = full_input.lines().collect();
            if let Some(current_line) = lines.get(line_num - 1) {
                // Count actual leading whitespace characters
                let mut indent_level = 0u8;
                let mut space_count = 0u8;

                for ch in current_line.chars() {
                    match ch {
                        '\t' => {
                            // Each tab counts as 1 indent level
                            indent_level += 1;
                            space_count = 0; // Reset space counting after tab
                        }
                        ' ' => {
                            space_count += 1;
                            // Every 4 spaces = 1 indent level
                            if space_count >= 4 {
                                indent_level += 1;
                                space_count = 0; // Reset for next group of spaces
                            }
                        }
                        _ => break, // Stop at first non-whitespace character
                    }
                }

                if indent_level > 0 {
                    Some(indent_level)
                } else {
                    None
                }
            } else {
                // Fallback to old column-based calculation for spaces-only content
                let spaces = (column - 1) as u8;
                let indent_level = spaces / 4;
                if indent_level > 0 {
                    Some(indent_level)
                } else {
                    None
                }
            }
        } else {
            None
        }
    }

    /// Extract heading content (inline elements)
    fn extract_heading_content(&mut self, pair: Pair<Rule>) -> Result<Vec<Node>, AstError> {
        let mut content = Vec::new();
        let original_text = pair.as_str(); // Get text before moving pair
        let span_copy = self.create_span(&pair);
        let rule = pair.as_rule();

        // Handle setext headers specially - check both direct setext rules and heading containers
        if matches!(rule, Rule::setext_h1 | Rule::setext_h2) {
            // Direct setext rule
            for inner_pair in pair.into_inner() {
                if matches!(inner_pair.as_rule(), Rule::setext_content) {
                    let setext_text = inner_pair.as_str().trim();
                    if !setext_text.is_empty() {
                        content.push(Node::text(
                            setext_text.to_string(),
                            self.create_span(&inner_pair),
                        ));
                    }
                    return Ok(content);
                }
            }
        } else if matches!(rule, Rule::heading) {
            // Heading container - look for ATX heading rules (H1-H6) inside, then heading_content inside those
            for inner_pair in pair.into_inner() {
                if matches!(inner_pair.as_rule(), Rule::setext_h1 | Rule::setext_h2) {
                    // Found setext inside heading - extract setext_content
                    for setext_inner in inner_pair.into_inner() {
                        if matches!(setext_inner.as_rule(), Rule::setext_content) {
                            let setext_text = setext_inner.as_str().trim();
                            if !setext_text.is_empty() {
                                content.push(Node::text(
                                    setext_text.to_string(),
                                    self.create_span(&setext_inner),
                                ));
                            }
                            return Ok(content);
                        }
                    }
                } else if matches!(
                    inner_pair.as_rule(),
                    Rule::H1 | Rule::H2 | Rule::H3 | Rule::H4 | Rule::H5 | Rule::H6
                ) {
                    // Found ATX heading (H1-H6) - look for heading_content inside it
                    for atx_inner in inner_pair.into_inner() {
                        if matches!(atx_inner.as_rule(), Rule::heading_content) {
                            // ATX header content - process children and preserve whitespace between tokens
                            let heading_text_pairs: Vec<_> = atx_inner.into_inner().collect();

                            for (i, heading_text_pair) in heading_text_pairs.iter().enumerate() {
                                // Add space if there's a gap between this token and the previous one
                                if i > 0 {
                                    let prev_end = heading_text_pairs[i - 1].as_span().end();
                                    let curr_start = heading_text_pair.as_span().start();

                                    if curr_start > prev_end {
                                        // There's whitespace between tokens - add a space node
                                        // Use the previous token's span info for line/column
                                        let prev_span =
                                            self.create_span(&heading_text_pairs[i - 1]);
                                        let space_span = Span::new(
                                            prev_end as u32,
                                            curr_start as u32,
                                            prev_span.line,
                                            prev_span.column,
                                        );
                                        content.push(Node::text(" ".to_string(), space_span));
                                    }
                                }

                                let child_node = self.build_node(heading_text_pair.clone())?;
                                content.push(child_node);
                            }
                        }
                    }
                }
            }
        } else {
            // Handle other heading types
            for inner_pair in pair.into_inner() {
                if matches!(inner_pair.as_rule(), Rule::heading_content) {
                    // ATX header content - process children and preserve whitespace between tokens
                    let heading_text_pairs: Vec<_> = inner_pair.into_inner().collect();

                    for (i, heading_text_pair) in heading_text_pairs.iter().enumerate() {
                        // Add space if there's a gap between this token and the previous one
                        if i > 0 {
                            let prev_end = heading_text_pairs[i - 1].as_span().end();
                            let curr_start = heading_text_pair.as_span().start();

                            if curr_start > prev_end {
                                // There's whitespace between tokens - add a space node
                                // Use the previous token's span info for line/column
                                let prev_span = self.create_span(&heading_text_pairs[i - 1]);
                                let space_span = Span::new(
                                    prev_end as u32,
                                    curr_start as u32,
                                    prev_span.line,
                                    prev_span.column,
                                );
                                content.push(Node::text(" ".to_string(), space_span));
                            }
                        }

                        let child_node = self.build_node(heading_text_pair.clone())?;
                        content.push(child_node);
                    }
                }
            }
        }

        // If no content found, extract from raw text (fallback for ATX headers)
        if content.is_empty() {
            // Remove heading markers
            let cleaned_text = original_text.trim_start_matches('#').trim();
            if !cleaned_text.is_empty() {
                content.push(Node::text(cleaned_text.to_string(), span_copy));
            }
        }

        Ok(content)
    }

    /// Extract code block language and content
    fn extract_code_content(
        &self,
        pair: &Pair<Rule>,
    ) -> Result<(Option<String>, String), AstError> {
        let text = pair.as_str();

        match pair.as_rule() {
            Rule::code_block => {
                // Code block is a container - check inner rules
                for inner_pair in pair.clone().into_inner() {
                    match inner_pair.as_rule() {
                        Rule::fenced_code => {
                            return self.extract_code_content(&inner_pair);
                        }
                        Rule::indented_code => {
                            return self.extract_code_content(&inner_pair);
                        }
                        _ => continue,
                    }
                }
                // Fallback: try to extract from the full text
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

    /// Extract emphasis content from non-atomic structure (new method for non-atomic rules)
    fn extract_emphasis_content_from_structure(
        &mut self,
        pair: &Pair<Rule>,
    ) -> Result<Vec<Node>, AstError> {
        let mut content_nodes = Vec::new();

        // Skip opening and closing markers, process content in between
        let inner_pairs: Vec<_> = pair.clone().into_inner().collect();

        if inner_pairs.is_empty() {
            // If no internal structure, extract text content manually
            let text = pair.as_str();
            let content_text = match pair.as_rule() {
                Rule::bold_asterisk => {
                    if text.len() >= 4 && text.starts_with("**") && text.ends_with("**") {
                        text[2..text.len() - 2].to_string()
                    } else if text.len() >= 2 && text.starts_with("**") {
                        text[2..].to_string()
                    } else {
                        text.to_string()
                    }
                }
                Rule::bold_underscore => {
                    if text.len() >= 4 && text.starts_with("__") && text.ends_with("__") {
                        text[2..text.len() - 2].to_string()
                    } else if text.len() >= 2 && text.starts_with("__") {
                        text[2..].to_string()
                    } else {
                        text.to_string()
                    }
                }
                Rule::italic_asterisk => {
                    if text.len() >= 2
                        && text.starts_with("*")
                        && text.ends_with("*")
                        && !text.starts_with("**")
                    {
                        text[1..text.len() - 1].to_string()
                    } else if !text.is_empty() && text.starts_with("*") && !text.starts_with("**") {
                        text[1..].to_string()
                    } else {
                        text.to_string()
                    }
                }
                Rule::italic_underscore => {
                    if text.len() >= 2
                        && text.starts_with("_")
                        && text.ends_with("_")
                        && !text.starts_with("__")
                    {
                        text[1..text.len() - 1].to_string()
                    } else if !text.is_empty() && text.starts_with("_") && !text.starts_with("__") {
                        text[1..].to_string()
                    } else {
                        text.to_string()
                    }
                }
                _ => {
                    // For bold_italic rules
                    let text = pair.as_str();
                    if (text.len() >= 6 && text.starts_with("***") && text.ends_with("***"))
                        || (text.len() >= 6 && text.starts_with("___") && text.ends_with("___"))
                    {
                        text[3..text.len() - 3].to_string()
                    } else if text.len() >= 5 && text.starts_with("**_") && text.ends_with("_**") {
                        // **_mixed bold italic_**
                        text[3..text.len() - 3].to_string()
                    } else if text.len() >= 5 && text.starts_with("__*") && text.ends_with("*__") {
                        // __*mixed bold italic*__
                        text[3..text.len() - 3].to_string()
                    } else if text.len() >= 5 && text.starts_with("***") && text.ends_with("___") {
                        // ***bold italic___
                        text[3..text.len() - 3].to_string()
                    } else if text.len() >= 5 && text.starts_with("___") && text.ends_with("***") {
                        // ___bold italic***
                        text[3..text.len() - 3].to_string()
                    } else {
                        // Fallback - try to strip any combination of * and _
                        text.to_string()
                    }
                }
            };

            if !content_text.is_empty() {
                let span = self.create_span(pair);
                content_nodes.push(Node::text(content_text, span));
            }
        } else {
            // Process internal structure if present
            for inner_pair in inner_pairs {
                let child = self.build_node(inner_pair)?;
                content_nodes.push(child);
            }
        }

        Ok(content_nodes)
    }

    /// Extract fenced code content and language properly
    fn extract_fenced_code_content(
        &self,
        text: &str,
    ) -> Result<(Option<String>, String), AstError> {
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return Ok((None, String::new()));
        }

        // Extract language from first line - improved parsing for variable-length fences
        let first_line = lines[0];
        let language = if first_line.starts_with("```") {
            // Find where backticks end (support 3-5 backticks)
            let backtick_count = first_line.chars().take_while(|&c| c == '`').count();
            if first_line.len() > backtick_count {
                let lang_part = first_line[backtick_count..].trim();
                if lang_part.is_empty() {
                    None
                } else {
                    // Remove any trailing whitespace or newlines
                    Some(lang_part.to_string())
                }
            } else {
                None
            }
        } else {
            None
        };

        // Extract content properly - skip opening and closing fence lines
        let content = if lines.len() > 2 {
            // Skip first line (opening fence) and last line (closing fence)
            let content_lines = &lines[1..lines.len() - 1];
            content_lines.join("\n")
        } else if lines.len() == 2 {
            // Empty code block (just opening and closing fence)
            String::new()
        } else {
            // Single line - shouldn't happen with proper fenced blocks, but handle gracefully
            String::new()
        };

        Ok((language, content))
    }

    /// Extract math block content
    fn extract_math_content(&self, pair: &Pair<Rule>) -> Result<String, AstError> {
        let text = pair.as_str();
        // Remove $$ delimiters
        let content = text.trim_start_matches("$$").trim_end_matches("$$");
        Ok(content.to_string())
    }

    /// Extract inline code content
    fn extract_inline_code_content(&self, pair: &Pair<Rule>) -> Result<String, AstError> {
        let text = pair.as_str();
        // Remove ` delimiters
        let content = text.trim_matches('`');
        Ok(content.to_string())
    }

    /// Extract inline math content
    fn extract_inline_math_content(&self, pair: &Pair<Rule>) -> Result<String, AstError> {
        let text = pair.as_str();
        // Remove $ delimiters
        let content = text.trim_matches('$');
        Ok(content.to_string())
    }

    /// Extract emoji content and convert to unicode
    fn extract_emoji_content(&self, pair: &Pair<Rule>) -> Result<(String, String), AstError> {
        let text = pair.as_str();
        // Remove : delimiters to get shortcode
        if text.len() >= 2 && text.starts_with(':') && text.ends_with(':') {
            let shortcode = text[1..text.len() - 1].to_string();

            // Use the emojis crate to convert shortcode to unicode
            if let Some(emoji) = emojis::get_by_shortcode(&shortcode) {
                Ok((shortcode, emoji.as_str().to_string()))
            } else {
                // If emoji not found, return the original text as both shortcode and unicode
                Ok((shortcode.clone(), format!(":{shortcode}:")))
            }
        } else {
            // Malformed emoji, return as-is
            Ok((text.to_string(), text.to_string()))
        }
    }

    /// Extract strikethrough content from atomic rules
    fn extract_strikethrough_content(&self, pair: &Pair<Rule>) -> Result<String, AstError> {
        // Look for the atomic strikethrough sub-rule
        for inner_pair in pair.clone().into_inner() {
            match inner_pair.as_rule() {
                Rule::strikethrough_tilde => {
                    let text = inner_pair.as_str();
                    // Remove ~~ delimiters
                    if text.len() >= 4 && text.starts_with("~~") && text.ends_with("~~") {
                        return Ok(text[2..text.len() - 2].to_string());
                    }
                }
                Rule::strikethrough_dash => {
                    let text = inner_pair.as_str();
                    // Remove -- delimiters
                    if text.len() >= 4 && text.starts_with("--") && text.ends_with("--") {
                        return Ok(text[2..text.len() - 2].to_string());
                    }
                }
                _ => {}
            }
        }

        // Fallback: extract from the outer pair
        let text = pair.as_str();
        if text.len() >= 4
            && ((text.starts_with("~~") && text.ends_with("~~"))
                || (text.starts_with("--") && text.ends_with("--")))
        {
            Ok(text[2..text.len() - 2].to_string())
        } else {
            Ok(text.to_string())
        }
    }

    /// Extract superscript content
    fn extract_superscript_content(&self, pair: &Pair<Rule>) -> Result<String, AstError> {
        let text = pair.as_str();
        // Remove ^ delimiters
        if text.len() >= 2 && text.starts_with("^") && text.ends_with("^") {
            Ok(text[1..text.len() - 1].to_string())
        } else {
            // Fallback for malformed superscript
            Ok(text.to_string())
        }
    }

    /// Extract subscript content
    fn extract_subscript_content(&self, pair: &Pair<Rule>) -> Result<String, AstError> {
        let text = pair.as_str();

        // Handle different subscript formats
        if text.starts_with("˅") && text.ends_with("˅") {
            // Arrow style: ˅content˅
            // Need to properly handle UTF-8 characters - ˅ is multi-byte
            let chars: Vec<char> = text.chars().collect();
            if chars.len() >= 2 && chars[0] == '˅' && chars[chars.len() - 1] == '˅' {
                let content: String = chars[1..chars.len() - 1].iter().collect();
                Ok(content)
            } else {
                Ok(text.to_string())
            }
        } else if text.len() >= 2 && text.starts_with("~") && text.ends_with("~") {
            // Tilde style: ~content~
            Ok(text[1..text.len() - 1].to_string())
        } else {
            // Fallback for malformed subscript
            Ok(text.to_string())
        }
    }

    /// Extract task list checkbox state
    fn extract_task_state(&self, pair: &Pair<Rule>) -> Result<Option<bool>, AstError> {
        let text = pair.as_str();
        if text.contains("[ ]") {
            Ok(Some(false))
        } else if text.contains("[x]") || text.contains("[X]") {
            Ok(Some(true))
        } else {
            Ok(None)
        }
    }

    /// Extract content from task_block (text after the task marker)
    fn extract_task_content(&mut self, pair: &Pair<Rule>) -> Result<Vec<Node>, AstError> {
        let full_text = pair.as_str();

        // Parse the task block: [x] Some content here
        // We need to extract "Some content here" part
        let content_text = if let Some(pos) = full_text.find(']') {
            // Skip past ']' and any whitespace to get to content
            let after_bracket = &full_text[pos + 1..];
            after_bracket.trim_start()
        } else {
            // Fallback - shouldn't happen with valid task_marker
            full_text
        };

        // Convert the text content to a Text node
        if content_text.is_empty() {
            Ok(vec![])
        } else {
            let span = self.create_span(pair);
            Ok(vec![Node::text(content_text.to_string(), span)])
        }
    }

    /// Extract table data (headers and rows)
    fn extract_table_data(
        &mut self,
        pair: Pair<Rule>,
    ) -> Result<(Vec<Node>, Vec<Vec<Node>>), AstError> {
        let mut headers = Vec::new();
        let mut rows = Vec::new();

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::table_with_header => {
                    // Recursively process table with header
                    return self.extract_table_data(inner_pair);
                }
                Rule::table_without_header => {
                    // Recursively process table without header
                    return self.extract_table_data(inner_pair);
                }
                Rule::table_header => {
                    // Explicit header found
                    let mut header_cells = self.build_children(inner_pair)?;
                    self.filter_trailing_empty_cells(&mut header_cells);
                    headers = header_cells;
                }
                Rule::table_row => {
                    // Regular data row
                    let mut cells = self.build_children(inner_pair)?;
                    self.filter_trailing_empty_cells(&mut cells);
                    rows.push(cells);
                }
                Rule::table_sep => {
                    // Skip separator row - only used for alignment info
                    continue;
                }
                _ => {}
            }
        }

        Ok((headers, rows))
    }

    /// Filter out empty trailing cells from a table row
    /// This handles cases where trailing pipes create unwanted empty cells
    fn filter_trailing_empty_cells(&self, cells: &mut Vec<Node>) {
        // Remove trailing empty cells
        while let Some(last_cell) = cells.last() {
            if let Node::TableCell { content, .. } = last_cell {
                // Check if cell is empty or contains only whitespace
                let is_empty = content.is_empty()
                    || content.iter().all(|node| {
                        if let Node::Text { content: text, .. } = node {
                            text.trim().is_empty()
                        } else {
                            false
                        }
                    });

                if is_empty {
                    cells.pop();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// Extract table cell alignment
    fn extract_table_alignment(&self, _pair: &Pair<Rule>) -> Result<Option<String>, AstError> {
        // This would be extracted from the table separator row
        // For now, return None (no alignment specified)
        Ok(None)
    }

    /// Extract link parts (text, URL, title)
    fn extract_link_parts(
        &mut self,
        pair: Pair<Rule>,
    ) -> Result<(Vec<Node>, String, Option<String>), AstError> {
        let text = pair.as_str();

        // Parse [text](url "title") format
        if let Some(close_bracket) = text.find("](") {
            let link_text = &text[1..close_bracket];
            let remaining = &text[close_bracket + 2..];

            // Find URL and optional title
            if let Some(close_paren) = remaining.rfind(')') {
                let url_and_title = &remaining[..close_paren];

                // Check for title in quotes
                if let Some(quote_pos) = url_and_title.find(" \"") {
                    let url = url_and_title[..quote_pos].to_string();
                    let title_part = &url_and_title[quote_pos + 2..];
                    let title = if let Some(stripped) = title_part.strip_suffix('"') {
                        Some(stripped.to_string())
                    } else {
                        Some(title_part.to_string())
                    };

                    // Build text nodes
                    let span = self.create_span(&pair);
                    let text_nodes = vec![Node::text(link_text.to_string(), span)];

                    return Ok((text_nodes, url, title));
                } else {
                    let url = url_and_title.to_string();
                    let span = self.create_span(&pair);
                    let text_nodes = vec![Node::text(link_text.to_string(), span)];
                    return Ok((text_nodes, url, None));
                }
            }
        }

        // Fallback
        let span = self.create_span(&pair);
        let text_nodes = vec![Node::text("link".to_string(), span.clone())];
        Ok((text_nodes, text.to_string(), None))
    }

    /// Extract image parts (alt, URL, title)
    fn extract_image_parts(
        &mut self,
        pair: Pair<Rule>,
    ) -> Result<(String, String, Option<String>), AstError> {
        let text = pair.as_str();

        // Parse ![alt](url "title") format
        if text.starts_with("![") {
            if let Some(close_bracket) = text.find("](") {
                let alt_text = &text[2..close_bracket];
                let remaining = &text[close_bracket + 2..];

                if let Some(close_paren) = remaining.rfind(')') {
                    let url_and_title = &remaining[..close_paren];

                    // Check for title in quotes
                    if let Some(quote_pos) = url_and_title.find(" \"") {
                        let url = url_and_title[..quote_pos].to_string();
                        let title_part = &url_and_title[quote_pos + 2..];
                        let title = if let Some(stripped) = title_part.strip_suffix('"') {
                            Some(stripped.to_string())
                        } else {
                            Some(title_part.to_string())
                        };
                        return Ok((alt_text.to_string(), url, title));
                    } else {
                        let url = url_and_title.to_string();
                        return Ok((alt_text.to_string(), url, None));
                    }
                }
            }
        }

        // Fallback
        Ok(("image".to_string(), text.to_string(), None))
    }

    /// Extract escaped character
    fn extract_escaped_character(&self, pair: &Pair<Rule>) -> Result<char, AstError> {
        let text = pair.as_str();
        if text.len() >= 2 && text.starts_with('\\') {
            Ok(text.chars().nth(1).unwrap_or('\\'))
        } else {
            Ok('\\')
        }
    }

    // ===========================================
    // MARCO EXTENSION EXTRACTION FUNCTIONS
    // ===========================================

    /// Extract user mention parts: @username[platform](Display Name)
    fn extract_user_mention_parts(
        &self,
        pair: Pair<Rule>,
    ) -> Result<(String, Option<String>, Option<String>), AstError> {
        let text = pair.as_str();

        // Parse @username[platform](Display Name) format
        if let Some(without_at) = text.strip_prefix('@') {
            // Find platform in brackets
            if let Some(bracket_start) = without_at.find('[') {
                let username = without_at[..bracket_start].to_string();

                if let Some(bracket_end) = without_at.find(']') {
                    let platform = without_at[bracket_start + 1..bracket_end].to_string();

                    // Check for display name in parentheses
                    let remaining = &without_at[bracket_end + 1..];
                    if remaining.starts_with('(') && remaining.ends_with(')') {
                        let display_name = remaining[1..remaining.len() - 1].to_string();
                        return Ok((username, Some(platform), Some(display_name)));
                    } else {
                        return Ok((username, Some(platform), None));
                    }
                }
            } else {
                // No platform specified
                let username = without_at.to_string();
                return Ok((username, None, None));
            }
        }

        Ok(("user".to_string(), None, None))
    }

    /// Extract bookmark parts: [bookmark:label](path=line)
    fn extract_bookmark_parts(
        &self,
        pair: Pair<Rule>,
    ) -> Result<(String, String, Option<u32>), AstError> {
        let text = pair.as_str();

        // Parse [bookmark:label](path=line) format
        if text.starts_with("[bookmark:") {
            if let Some(close_bracket) = text.find("](") {
                let label = text[10..close_bracket].to_string();

                let remaining = &text[close_bracket + 2..];
                if let Some(close_paren) = remaining.find(')') {
                    let path_and_line = &remaining[..close_paren];

                    // Check for line number after =
                    if let Some(eq_pos) = path_and_line.find('=') {
                        let path = path_and_line[..eq_pos].to_string();
                        let line_str = &path_and_line[eq_pos + 1..];
                        let line = line_str.parse::<u32>().ok();
                        return Ok((label, path, line));
                    } else {
                        let path = path_and_line.to_string();
                        return Ok((label, path, None));
                    }
                }
            }
        }

        Ok(("bookmark".to_string(), text.to_string(), None))
    }

    /// Extract admonition parts: :::note[title]
    fn extract_admonition_parts(
        &mut self,
        pair: Pair<Rule>,
    ) -> Result<(String, Option<String>, Vec<Node>), AstError> {
        let text = pair.as_str();
        let lines: Vec<&str> = text.lines().collect();

        if lines.is_empty() {
            return Ok(("note".to_string(), None, Vec::new()));
        }

        // Parse opening line: :::type[title]
        let first_line = lines[0];
        if let Some(type_and_title) = first_line.strip_prefix(":::") {
            // Check for title in brackets
            if let Some(bracket_start) = type_and_title.find('[') {
                let admonition_type = type_and_title[..bracket_start].to_string();

                if let Some(bracket_end) = type_and_title.find(']') {
                    let title = type_and_title[bracket_start + 1..bracket_end].to_string();

                    // Extract content (everything between opening and closing :::)
                    let content_lines: Vec<&str> = lines
                        .iter()
                        .skip(1)
                        .take_while(|line| !line.starts_with(":::"))
                        .copied()
                        .collect();

                    let content_text = content_lines.join("\n");
                    let span = self.create_span(&pair);
                    let content = if content_text.trim().is_empty() {
                        Vec::new()
                    } else {
                        vec![Node::text(content_text, span)]
                    };

                    return Ok((admonition_type, Some(title), content));
                }
            } else {
                let admonition_type = type_and_title.to_string();

                // Extract content without title
                let content_lines: Vec<&str> = lines
                    .iter()
                    .skip(1)
                    .take_while(|line| !line.starts_with(":::"))
                    .copied()
                    .collect();

                let content_text = content_lines.join("\n");
                let span = self.create_span(&pair);
                let content = if content_text.trim().is_empty() {
                    Vec::new()
                } else {
                    vec![Node::text(content_text, span)]
                };

                return Ok((admonition_type, None, content));
            }
        }

        Ok(("note".to_string(), None, Vec::new()))
    }

    /// Extract tab block parts
    fn extract_tab_block_parts(
        &mut self,
        pair: Pair<Rule>,
    ) -> Result<(Option<String>, Vec<Node>), AstError> {
        let mut title = None;
        let mut tabs = Vec::new();

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::tab_header => {
                    // Extract title from tab header
                    title = self.extract_tab_header_title(inner_pair)?;
                }
                Rule::tabs_content => {
                    // Extract tabs and general content from tabs_content
                    let content_nodes = self.extract_tabs_content(inner_pair)?;
                    tabs.extend(content_nodes);
                }
                Rule::tab_end => {
                    // Closing delimiter - no processing needed
                }
                _ => {
                    // Handle any other unexpected content
                    let child = self.build_node(inner_pair)?;
                    tabs.push(child);
                }
            }
        }

        Ok((title, tabs))
    }

    /// Extract tab header title
    fn extract_tab_header_title(&self, pair: Pair<Rule>) -> Result<Option<String>, AstError> {
        for inner_pair in pair.into_inner() {
            if inner_pair.as_rule() == Rule::tab_title {
                let title = inner_pair.as_str().trim().to_string();
                if !title.is_empty() {
                    return Ok(Some(title));
                }
            }
        }
        Ok(None)
    }

    /// Extract content from tabs_content (contains tab rules)
    fn extract_tabs_content(&mut self, pair: Pair<Rule>) -> Result<Vec<Node>, AstError> {
        let mut content = Vec::new();

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::tab => {
                    // Extract individual tab and create Tab node
                    let (tab_name, tab_content) = self.extract_tab_parts(inner_pair.clone())?;
                    let span = self.create_span(&inner_pair);

                    content.push(Node::Tab {
                        name: tab_name,
                        content: tab_content,
                        span,
                    });
                }
                _ => {
                    // Handle any other content (shouldn't happen with new grammar)
                    let child = self.build_node(inner_pair)?;
                    content.push(child);
                }
            }
        }

        Ok(content)
    }

    /// Extract individual tab parts
    fn extract_tab_parts(
        &mut self,
        pair: Pair<Rule>,
    ) -> Result<(Option<String>, Vec<Node>), AstError> {
        let mut name = None;
        let mut content = Vec::new();

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::tab_line => {
                    // Extract tab name from @tab [name]
                    name = self.extract_tab_line_name(inner_pair)?;
                }
                Rule::tab_body => {
                    // Extract tab content
                    let tab_content = self.extract_tab_body_content(inner_pair)?;
                    content.extend(tab_content);
                }
                _ => {
                    // Handle any other content
                    let child = self.build_node(inner_pair)?;
                    content.push(child);
                }
            }
        }

        Ok((name, content))
    }

    /// Extract tab name from tab_line (@tab [name])
    fn extract_tab_line_name(&self, pair: Pair<Rule>) -> Result<Option<String>, AstError> {
        for inner_pair in pair.into_inner() {
            if inner_pair.as_rule() == Rule::tab_name {
                let name = inner_pair.as_str().trim().to_string();
                if !name.is_empty() {
                    return Ok(Some(name));
                }
            }
        }
        Ok(None)
    }

    /// Extract content from tab_body
    fn extract_tab_body_content(&mut self, pair: Pair<Rule>) -> Result<Vec<Node>, AstError> {
        let content_text = pair.as_str().trim();
        if content_text.is_empty() {
            return Ok(Vec::new());
        }

        // Parse the content as markdown
        let span = self.create_span(&pair);

        // For now, treat tab content as plain text
        // In a more sophisticated implementation, we could parse it as markdown
        Ok(vec![Node::text(content_text.to_string(), span)])
    }

    /// Extract table of contents parts: [toc=2](@doc)
    fn extract_toc_parts(
        &self,
        pair: Pair<Rule>,
    ) -> Result<(Option<u8>, Option<String>), AstError> {
        let text = pair.as_str();

        // Parse [toc=2](@doc) format
        if text.starts_with("[toc") {
            let mut depth = None;
            let mut document = None;

            // Check for depth after =
            if let Some(eq_pos) = text.find('=') {
                if let Some(bracket_end) = text.find(']') {
                    let depth_str = &text[eq_pos + 1..bracket_end];
                    depth = depth_str.parse::<u8>().ok();
                }
            }

            // Check for document after (@
            if let Some(at_pos) = text.find("(@") {
                if let Some(close_paren) = text.find(')') {
                    let doc_str = &text[at_pos + 2..close_paren];
                    document = Some(doc_str.to_string());
                }
            }

            return Ok((depth, document));
        }

        Ok((None, None))
    }

    /// Extract run inline parts: run@bash(command)
    fn extract_run_inline_parts(&self, pair: Pair<Rule>) -> Result<(String, String), AstError> {
        let text = pair.as_str();

        // Parse run@type(command) format
        if text.starts_with("run@") {
            if let Some(paren_start) = text.find('(') {
                let script_type = text[4..paren_start].to_string();

                if let Some(paren_end) = text.rfind(')') {
                    let command = text[paren_start + 1..paren_end].to_string();
                    return Ok((script_type, command));
                }
            }
        }

        Ok(("bash".to_string(), text.to_string()))
    }

    /// Extract run block parts: ```run@python
    fn extract_run_block_parts(&self, pair: Pair<Rule>) -> Result<(String, String), AstError> {
        let text = pair.as_str();
        let lines: Vec<&str> = text.lines().collect();

        if lines.is_empty() {
            return Ok(("bash".to_string(), String::new()));
        }

        // Extract script type from first line: ```run@python
        let first_line = lines[0];
        let script_type = if let Some(type_part) = first_line.strip_prefix("```run@") {
            type_part.to_string()
        } else {
            "bash".to_string()
        };

        // Extract content (everything except first and last line)
        let content = if lines.len() > 2 {
            lines[1..lines.len() - 1].join("\n")
        } else {
            String::new()
        };

        Ok((script_type, content))
    }

    /// Extract diagram parts: ```mermaid
    fn extract_diagram_parts(&self, pair: Pair<Rule>) -> Result<(String, String), AstError> {
        let text = pair.as_str();
        let lines: Vec<&str> = text.lines().collect();

        if lines.is_empty() {
            return Ok(("mermaid".to_string(), String::new()));
        }

        // Extract diagram type from first line: ```mermaid
        let first_line = lines[0];
        let diagram_type = if let Some(type_part) = first_line.strip_prefix("```") {
            type_part.to_string()
        } else {
            "mermaid".to_string()
        };

        // Extract content (everything except first and last line)
        let content = if lines.len() > 2 {
            lines[1..lines.len() - 1].join("\n")
        } else {
            String::new()
        };

        Ok((diagram_type, content))
    }

    // ===========================================
    // FOOTNOTE AND REFERENCE EXTRACTION FUNCTIONS
    // ===========================================

    /// Extract footnote definition parts: [^label]: content
    fn extract_footnote_def_parts(
        &mut self,
        pair: Pair<Rule>,
    ) -> Result<(String, Vec<Node>), AstError> {
        let text = pair.as_str();

        // Parse [^label]: content format
        if let Some(close_bracket) = text.find("]: ") {
            if text.starts_with("[^") {
                let label = text[2..close_bracket].to_string();
                let content_text = text[close_bracket + 3..].trim();

                // Parse content as markdown - for now, treat as text
                let span = self.create_span(&pair);
                let content = if content_text.is_empty() {
                    Vec::new()
                } else {
                    vec![Node::text(content_text.to_string(), span)]
                };

                return Ok((label, content));
            }
        }

        // Fallback
        Ok(("".to_string(), Vec::new()))
    }

    /// Extract footnote reference label: [^label]
    fn extract_footnote_ref_label(&self, pair: &Pair<Rule>) -> Result<String, AstError> {
        let text = pair.as_str();

        // Parse [^label] format
        if text.starts_with("[^") && text.ends_with("]") && text.len() > 3 {
            let label = text[2..text.len() - 1].to_string();
            Ok(label)
        } else {
            Ok("".to_string())
        }
    }

    /// Extract inline footnote content: ^[content]
    fn extract_inline_footnote_content(&mut self, pair: Pair<Rule>) -> Result<Vec<Node>, AstError> {
        let text = pair.as_str();

        // Parse ^[content] format
        if text.starts_with("^[") && text.ends_with("]") && text.len() > 3 {
            let content_text = &text[2..text.len() - 1];
            let span = self.create_span(&pair);

            // For now, treat content as plain text
            // In a full implementation, you'd parse as inline markdown
            Ok(vec![Node::text(content_text.to_string(), span)])
        } else {
            Ok(Vec::new())
        }
    }

    /// Extract reference definition parts: [label]: url "title"
    fn extract_reference_def_parts(
        &self,
        pair: Pair<Rule>,
    ) -> Result<(String, String, Option<String>), AstError> {
        let text = pair.as_str();

        // Parse [label]: url "title" format
        if let Some(close_bracket) = text.find("]: ") {
            if text.starts_with("[") {
                let label = text[1..close_bracket].to_string();
                let url_and_title = &text[close_bracket + 3..];

                // Check for title in quotes
                if let Some(quote_pos) = url_and_title.find(" \"") {
                    let url = url_and_title[..quote_pos].trim().to_string();
                    let title_part = &url_and_title[quote_pos + 2..];
                    let title = if let Some(stripped) = title_part.strip_suffix('"') {
                        Some(stripped.to_string())
                    } else {
                        Some(title_part.to_string())
                    };
                    return Ok((label, url, title));
                } else {
                    let url = url_and_title.trim().to_string();
                    return Ok((label, url, None));
                }
            }
        }

        // Fallback
        Ok(("".to_string(), "".to_string(), None))
    }

    /// Extract reference link parts: [text][label]
    fn extract_reference_link_parts(
        &mut self,
        pair: Pair<Rule>,
    ) -> Result<(Vec<Node>, String), AstError> {
        let text = pair.as_str();

        // Parse [text][label] format
        if let Some(first_close) = text.find("][") {
            if text.starts_with("[") && text.ends_with("]") {
                let link_text = &text[1..first_close];
                let label = &text[first_close + 2..text.len() - 1];

                let span = self.create_span(&pair);
                let text_nodes = vec![Node::text(link_text.to_string(), span)];

                return Ok((text_nodes, label.to_string()));
            }
        }

        // Fallback
        let span = self.create_span(&pair);
        Ok((vec![Node::text("link".to_string(), span)], "".to_string()))
    }

    /// Extract reference image parts: ![alt][label]
    fn extract_reference_image_parts(
        &self,
        pair: Pair<Rule>,
    ) -> Result<(String, String), AstError> {
        let text = pair.as_str();

        // Parse ![alt][label] format
        if text.starts_with("![") {
            if let Some(first_close) = text.find("][") {
                if text.ends_with("]") {
                    let alt_text = &text[2..first_close];
                    let label = &text[first_close + 2..text.len() - 1];
                    return Ok((alt_text.to_string(), label.to_string()));
                }
            }
        }

        // Fallback
        Ok(("image".to_string(), "".to_string()))
    }

    /// Post-process inline content to detect hard line breaks
    /// Looks for text nodes ending with 2+ spaces followed by soft line breaks
    fn detect_hard_line_breaks(
        &self,
        mut content: Vec<Node>,
        source_text: &str,
    ) -> Result<Vec<Node>, AstError> {
        for i in 0..content.len() {
            if let Node::LineBreak { break_type, span } = &content[i] {
                if matches!(
                    break_type,
                    crate::components::marco_engine::ast_node::LineBreakType::Soft
                ) {
                    // Check if there's preceding text that ends with 2+ spaces or backslash
                    if let Some(text_index) =
                        self.find_preceding_text_with_trailing_spaces(&content, i, source_text)
                    {
                        // Convert this soft line break to hard line break
                        content[i] = Node::hard_line_break(span.clone());

                        // Also trim the trailing spaces/backslash from the text node
                        if let Node::Text {
                            content: text_content,
                            span: text_span,
                        } = &content[text_index]
                        {
                            let trimmed_content = if text_content.ends_with('\\') {
                                // Remove backslash
                                text_content.trim_end_matches('\\').to_string()
                            } else {
                                // Remove trailing spaces
                                text_content.trim_end().to_string()
                            };

                            // Update the text node with trimmed content
                            content[text_index] = Node::Text {
                                content: trimmed_content,
                                span: text_span.clone(),
                            };
                        }
                    }
                }
            }
        }
        Ok(content)
    }

    /// Fix whitespace boundary issues between text and formatting elements
    fn fix_whitespace_boundaries(
        &self,
        mut content: Vec<Node>,
        source_text: &str,
    ) -> Result<Vec<Node>, AstError> {
        let mut i = 0;
        while i < content.len() {
            // Look for patterns where whitespace is missing between text and formatting
            if i > 0 {
                let needs_update = {
                    let (prev_node, current_node) = (&content[i - 1], &content[i]);

                    // Check if there's missing whitespace between nodes
                    match (prev_node, current_node) {
                        (
                            Node::Text {
                                span: prev_span, ..
                            },
                            Node::Strong {
                                span: current_span, ..
                            }
                            | Node::Emphasis {
                                span: current_span, ..
                            },
                        ) => {
                            // Check if there's a gap between the spans (missing whitespace)
                            if prev_span.end < current_span.start {
                                let gap_start = prev_span.end as usize;
                                let gap_end = current_span.start as usize;

                                if gap_start < source_text.len() && gap_end <= source_text.len() {
                                    let gap_text = &source_text[gap_start..gap_end];

                                    // If the gap contains only whitespace, we should add it
                                    if gap_text.trim().is_empty() && !gap_text.is_empty() {
                                        Some((true, gap_text.to_string()))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                };

                if let Some((is_prev, gap_text)) = needs_update {
                    if is_prev {
                        // Update the previous text node
                        if let Node::Text {
                            content: prev_content,
                            span: prev_span,
                        } = &content[i - 1].clone()
                        {
                            if let Node::Strong {
                                span: current_span, ..
                            }
                            | Node::Emphasis {
                                span: current_span, ..
                            } = &content[i]
                            {
                                content[i - 1] = Node::Text {
                                    content: format!("{}{}", prev_content, gap_text),
                                    span: Span {
                                        start: prev_span.start,
                                        end: current_span.start,
                                        line: prev_span.line,
                                        column: prev_span.column,
                                    },
                                };
                            }
                        }
                    }
                }

                // Also check the reverse: formatting followed by text
                let needs_update_reverse = {
                    let (prev_node, current_node) = (&content[i - 1], &content[i]);

                    match (prev_node, current_node) {
                        (
                            Node::Strong {
                                span: prev_span, ..
                            }
                            | Node::Emphasis {
                                span: prev_span, ..
                            },
                            Node::Text {
                                span: current_span, ..
                            },
                        ) => {
                            // Check if there's a gap between the spans (missing whitespace)
                            if prev_span.end < current_span.start {
                                let gap_start = prev_span.end as usize;
                                let gap_end = current_span.start as usize;

                                if gap_start < source_text.len() && gap_end <= source_text.len() {
                                    let gap_text = &source_text[gap_start..gap_end];

                                    // If the gap contains only whitespace, we should add it
                                    if gap_text.trim().is_empty() && !gap_text.is_empty() {
                                        Some(gap_text.to_string())
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                };

                if let Some(gap_text) = needs_update_reverse {
                    // Update the current text node
                    if let Node::Text {
                        content: current_content,
                        span: current_span,
                    } = &content[i].clone()
                    {
                        if let Node::Strong {
                            span: prev_span, ..
                        }
                        | Node::Emphasis {
                            span: prev_span, ..
                        } = &content[i - 1]
                        {
                            content[i] = Node::Text {
                                content: format!("{}{}", gap_text, current_content),
                                span: Span {
                                    start: prev_span.end,
                                    end: current_span.end,
                                    line: current_span.line,
                                    column: current_span.column,
                                },
                            };
                        }
                    }
                }
            }
            i += 1;
        }
        Ok(content)
    }

    /// Check if there's text before the line break position that ends with 2+ spaces
    fn find_preceding_text_with_trailing_spaces(
        &self,
        content: &[Node],
        line_break_index: usize,
        source_text: &str,
    ) -> Option<usize> {
        if line_break_index == 0 {
            return None;
        }

        // Look at the preceding node
        if let Node::Text {
            content: text_content,
            span,
            ..
        } = &content[line_break_index - 1]
        {
            // First check if the text content itself ends with 2+ spaces or backslash
            if text_content.ends_with("\\") {
                return Some(line_break_index - 1);
            }

            let trailing_spaces = text_content.chars().rev().take_while(|&c| c == ' ').count();
            if trailing_spaces >= 2 {
                return Some(line_break_index - 1);
            }

            // If no spaces in text content, check the source text between text end and line break start
            if let Node::LineBreak { span: lb_span, .. } = &content[line_break_index] {
                let text_end = span.end as usize;
                let lb_start = lb_span.start as usize;

                // Check the text between text_end and line_break_start for spaces
                if text_end < lb_start && lb_start <= source_text.len() {
                    let between_text: String = source_text
                        .chars()
                        .skip(text_end)
                        .take(lb_start - text_end)
                        .collect();

                    // Check if there are 2+ spaces or a backslash
                    if between_text.len() >= 2
                        && (between_text.chars().all(|c| c == ' ')
                            || between_text.starts_with('\\'))
                    {
                        return Some(line_break_index - 1);
                    }
                }
            }
        }
        None
    }

    /// Try to build a nested code block if the content contains nested fenced code
    /// Returns (nested_node, nesting_level) if nested content is detected, or None for regular code blocks
    fn try_build_nested_code_block(
        &mut self,
        language: &Option<String>,
        content: &str,
        span: Span,
    ) -> Result<Option<(Node, u8)>, AstError> {
        // Check if the content contains nested fenced code blocks
        if self.has_nested_fences(content) {
            // Determine our level based on our own opening fence from the span
            let nesting_level = self.determine_current_fence_level(&span);

            // This is a nested code block - parse the content recursively
            let inner_content = self.parse_nested_content(content)?;

            let nested_node =
                Node::nested_code_block(language.clone(), nesting_level, inner_content, span);

            Ok(Some((nested_node, nesting_level)))
        } else {
            // Regular code block - no nesting detected
            Ok(None)
        }
    }

    /// Check if content has nested fence markers
    fn has_nested_fences(&self, content: &str) -> bool {
        // Don't check for nesting if we're already at max depth
        if self.nesting_depth >= self.max_nesting_depth {
            return false;
        }

        let lines: Vec<&str> = content.lines().collect();

        for line in lines {
            let trimmed = line.trim();

            // Check for fenced code block markers
            if trimmed.starts_with("```") && trimmed.len() >= 3 {
                let backtick_count = trimmed.chars().take_while(|&c| c == '`').count();
                if backtick_count >= 3 {
                    return true;
                }
            }
        }

        // If we're already at a nesting depth > 0, treat all blocks as nested for consistency
        if self.nesting_depth > 0 {
            return true;
        }

        false
    }

    /// Determine the current fence level based on the original input
    /// This is a simplified approach - in a full implementation we'd track this better
    fn determine_current_fence_level(&self, _span: &Span) -> u8 {
        // Use current nesting depth + 1
        self.nesting_depth + 1
    }

    /// Parse nested content recursively with increased nesting depth
    fn parse_nested_content(&mut self, content: &str) -> Result<Vec<Node>, AstError> {
        // Check recursion depth to prevent infinite recursion and stack overflow
        if self.nesting_depth >= self.max_nesting_depth {
            log::warn!(
                "[ast_builder] Maximum nesting depth ({}) reached, treating as plain text",
                self.max_nesting_depth
            );
            return Ok(vec![Node::text(
                content.to_string(),
                Span {
                    start: 0,
                    end: content.len() as u32,
                    line: 1,
                    column: 1,
                },
            )]);
        }

        // Increase nesting depth for recursive parsing
        let old_depth = self.nesting_depth;
        self.nesting_depth += 1;

        // Parse the content as a markdown document
        use crate::components::marco_engine::MarcoParser;
        use pest::Parser;

        let result = match MarcoParser::parse(Rule::document, content) {
            Ok(pairs) => {
                let mut children = Vec::new();
                for pair in pairs {
                    let node = self.build_node(pair)?;
                    children.push(node);
                }
                Ok(children)
            }
            Err(_e) => {
                // If parsing fails, treat as plain text
                Ok(vec![Node::text(
                    content.to_string(),
                    Span {
                        start: 0,
                        end: content.len() as u32,
                        line: 1,
                        column: 1,
                    },
                )])
            }
        };

        // Restore original nesting depth
        self.nesting_depth = old_depth;

        result
    }
}

// Implementation of error display for better debugging
impl std::fmt::Display for AstError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            AstError::InvalidRule(msg) => write!(f, "Invalid rule: {}", msg),
            AstError::MissingContent(msg) => write!(f, "Missing content: {}", msg),
            AstError::UnsupportedFeature(msg) => write!(f, "Unsupported feature: {}", msg),
        }
    }
}

impl std::error::Error for AstError {}

// Convert AstError to String for compatibility
impl From<AstError> for String {
    fn from(error: AstError) -> Self {
        format!("{}", error)
    }
}
