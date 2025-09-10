//! AST validation module for Marco engine
//!
//! This module provides comprehensive validation of AST nodes to ensure:
//! - Node consistency and proper structure
//! - Semantic correctness
//! - Proper nesting rules
//! - Marco-specific extension validation
//! - Content validation (length limits, format requirements, etc.)

use crate::components::marco_engine::{
    ast::{Node, Span},
    errors::{MarcoError, MarcoResult},
};
use std::collections::{HashMap, HashSet};

/// Configuration for AST validation
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Maximum depth allowed for nested structures
    pub max_depth: usize,
    /// Maximum content length for text nodes
    pub max_text_length: usize,
    /// Maximum number of children for container nodes
    pub max_children: usize,
    /// Whether to enforce strict CommonMark compliance
    pub strict_commonmark: bool,
    /// Whether to validate Marco-specific extensions
    pub validate_marco_extensions: bool,
    /// Allowed admonition types
    pub allowed_admonition_types: HashSet<String>,
    /// Maximum heading level (1-6 for CommonMark)
    pub max_heading_level: u8,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        let mut allowed_admonition_types = HashSet::new();
        allowed_admonition_types.insert("note".to_string());
        allowed_admonition_types.insert("warning".to_string());
        allowed_admonition_types.insert("tip".to_string());
        allowed_admonition_types.insert("danger".to_string());
        allowed_admonition_types.insert("info".to_string());

        Self {
            max_depth: 10,
            max_text_length: 1000000, // 1MB
            max_children: 1000,
            strict_commonmark: false,
            validate_marco_extensions: true,
            allowed_admonition_types,
            max_heading_level: 6,
        }
    }
}

/// Context for validation tracking
#[derive(Debug)]
struct ValidationContext {
    current_depth: usize,
    parent_stack: Vec<String>,
    reference_ids: HashSet<String>,
    link_references: HashMap<String, String>,
    footnote_definitions: HashSet<String>,
    footnote_references: HashSet<String>,
    user_mentions: HashSet<String>,
    bookmarks: HashSet<String>,
}

impl ValidationContext {
    fn new() -> Self {
        Self {
            current_depth: 0,
            parent_stack: Vec::new(),
            reference_ids: HashSet::new(),
            link_references: HashMap::new(),
            footnote_definitions: HashSet::new(),
            footnote_references: HashSet::new(),
            user_mentions: HashSet::new(),
            bookmarks: HashSet::new(),
        }
    }

    fn enter_node(&mut self, node_type: &str) {
        self.current_depth += 1;
        self.parent_stack.push(node_type.to_string());
    }

    fn exit_node(&mut self) {
        self.current_depth -= 1;
        self.parent_stack.pop();
    }

    fn current_parent(&self) -> Option<&String> {
        self.parent_stack.last()
    }
}

/// AST validator with comprehensive validation rules
pub struct AstValidator {
    config: ValidationConfig,
}

impl AstValidator {
    /// Create a new validator with default configuration
    pub fn new() -> Self {
        Self {
            config: ValidationConfig::default(),
        }
    }

    /// Create a new validator with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Validate an entire AST
    pub fn validate(&self, node: &Node) -> MarcoResult<()> {
        let mut context = ValidationContext::new();
        self.validate_node(node, &mut context)?;

        // Final validation: check for unresolved references
        self.validate_references(&context)?;

        Ok(())
    }

    /// Validate a single node recursively
    fn validate_node(&self, node: &Node, context: &mut ValidationContext) -> MarcoResult<()> {
        // Check maximum depth
        if context.current_depth >= self.config.max_depth {
            return Err(MarcoError::max_depth_exceeded(
                context.current_depth,
                self.config.max_depth,
            ));
        }

        // Get node type for validation
        let node_type = self.get_node_type(node);
        context.enter_node(&node_type);

        // Validate based on node type
        match node {
            Node::Document { children, span } => {
                self.validate_document(children, span, context)?;
            }
            Node::Heading {
                level,
                content,
                span,
            } => {
                self.validate_heading(*level, content, span, context)?;
            }
            Node::Paragraph { content, span } => {
                self.validate_paragraph(content, span, context)?;
            }
            Node::Text { content, span } => {
                self.validate_text(content, span)?;
            }
            Node::Strong { content, span } => {
                self.validate_inline_formatting(content, span, "Strong", context)?;
            }
            Node::Emphasis { content, span } => {
                self.validate_inline_formatting(content, span, "Emphasis", context)?;
            }
            Node::Code { content, span } => {
                self.validate_code_content(content, span)?;
            }
            Node::CodeBlock {
                language,
                content,
                span,
            } => {
                self.validate_code_block(language, content, span)?;
            }
            Node::Link {
                text,
                url,
                title: _,
                span,
            } => {
                self.validate_link(url, text, span, context)?;
            }
            Node::Image {
                alt,
                url,
                title: _,
                span,
            } => {
                self.validate_image(url, alt, span)?;
            }
            Node::List {
                ordered: _,
                items,
                span,
            } => {
                self.validate_list(items, span, context)?;
            }
            Node::ListItem {
                content,
                checked: _,
                span,
            } => {
                self.validate_list_item(content, span, context)?;
            }
            Node::BlockQuote { content, span } => {
                self.validate_block_quote(content, span, context)?;
            }
            Node::HorizontalRule { span: _ } => {
                // No specific validation needed for horizontal rules
            }
            Node::LineBreak { span: _ } => {
                // No specific validation needed for line breaks
            }
            // Marco-specific extensions
            Node::Admonition {
                kind,
                content,
                span,
            } => {
                if self.config.validate_marco_extensions {
                    self.validate_admonition(kind, content, span, context)?;
                }
            }
            Node::UserMention {
                username,
                platform: _,
                display_name: _,
                span,
            } => {
                if self.config.validate_marco_extensions {
                    self.validate_user_mention(username, span, context)?;
                }
            }
            Node::Bookmark {
                label: _,
                path,
                line: _,
                span,
            } => {
                if self.config.validate_marco_extensions {
                    self.validate_bookmark(path, span, context)?;
                }
            }
            Node::TabBlock {
                title: _,
                tabs,
                span,
            } => {
                if self.config.validate_marco_extensions {
                    self.validate_tab_block_nodes(tabs, span, context)?;
                }
            }
            // Table elements
            Node::Table {
                headers,
                rows,
                span,
            } => {
                self.validate_table(headers, rows, span, context)?;
            }
            Node::TableHeader { cells, span } => {
                self.validate_table_row(cells, span, context)?; // Use same validation as row
            }
            Node::TableRow { cells, span } => {
                self.validate_table_row(cells, span, context)?;
            }
            Node::TableCell {
                content,
                alignment: _,
                span,
            } => {
                self.validate_table_cell(content, span, context)?;
            }

            // Enhanced admonitions
            Node::AdmonitionWithIcon {
                kind,
                icon: _,
                title: _,
                content,
                span,
            } => {
                if self.config.validate_marco_extensions {
                    self.validate_admonition_with_icon(kind, content, span, context)?;
                }
            }

            // Enhanced tabs
            Node::TabWithMetadata {
                name,
                icon: _,
                active: _,
                content,
                span,
            } => {
                if self.config.validate_marco_extensions {
                    self.validate_tab_with_metadata(name, content, span, context)?;
                }
            }
            Node::Tab {
                name,
                content,
                span,
            } => {
                if self.config.validate_marco_extensions {
                    self.validate_tab(name, content, span, context)?;
                }
            }

            // Macro and meta elements
            Node::Macro {
                name,
                arguments: _,
                content: _,
                span,
            } => {
                if self.config.validate_marco_extensions {
                    self.validate_macro(name, span)?;
                }
            }
            Node::PageTag { format, span } => {
                if self.config.validate_marco_extensions {
                    self.validate_page_tag(format, span)?;
                }
            }
            Node::TableOfContents {
                depth,
                document: _,
                span,
            } => {
                if self.config.validate_marco_extensions {
                    self.validate_table_of_contents(depth, span)?;
                }
            }
            Node::DocumentReference { path, span } => {
                if self.config.validate_marco_extensions {
                    self.validate_document_reference(path, span)?;
                }
            }
            Node::RunBlock {
                script_type,
                content,
                span,
            } => {
                if self.config.validate_marco_extensions {
                    self.validate_run_block(script_type, content, span)?;
                }
            }
            Node::RunInline {
                script_type,
                command,
                span,
            } => {
                if self.config.validate_marco_extensions {
                    self.validate_run_inline(script_type, command, span)?;
                }
            }
            Node::DiagramBlock {
                diagram_type,
                content,
                span,
            } => {
                if self.config.validate_marco_extensions {
                    self.validate_diagram_block(diagram_type, content, span)?;
                }
            }

            // Mathematical expressions
            Node::MathInline { content, span } => {
                self.validate_math_inline(content, span)?;
            }
            Node::MathBlock { content, span } => {
                self.validate_math_block(content, span)?;
            }
            Node::MathBlockDisplay {
                content,
                delimiter: _,
                span,
            } => {
                self.validate_math_block_display(content, span)?;
            }

            // Extended text formatting
            Node::Strikethrough { content, span } => {
                self.validate_inline_formatting(content, span, "Strikethrough", context)?;
            }
            Node::Highlight { content, span } => {
                self.validate_inline_formatting(content, span, "Highlight", context)?;
            }
            Node::Mark {
                content,
                reason: _,
                span,
            } => {
                self.validate_inline_formatting(content, span, "Mark", context)?;
            }
            Node::Superscript { content, span } => {
                self.validate_inline_formatting(content, span, "Superscript", context)?;
            }
            Node::Subscript { content, span } => {
                self.validate_inline_formatting(content, span, "Subscript", context)?;
            }

            // Links and references
            Node::ReferenceLink { text, label, span } => {
                self.validate_reference_link(text, label, span, context)?;
            }
            Node::ReferenceImage { alt, label, span } => {
                self.validate_reference_image(alt, label, span, context)?;
            }
            Node::ReferenceDefinition {
                label,
                url,
                title: _,
                span,
            } => {
                self.validate_reference_definition(label, url, span, context)?;
            }
            Node::LinkReferenceDefinition {
                label,
                destination,
                title: _,
                span,
            } => {
                self.validate_link_reference_definition(label, destination, span, context)?;
            }
            Node::AutolinkUrl { url, span } => {
                self.validate_autolink_url(url, span)?;
            }
            Node::AutolinkEmail { email, span } => {
                self.validate_autolink_email(email, span)?;
            }

            // Footnotes
            Node::FootnoteRef { label, span } => {
                self.validate_footnote_reference(label, span, context)?;
            }
            Node::InlineFootnote { content, span } => {
                self.validate_inline_footnote(content, span, context)?;
            }
            Node::FootnoteDefinition {
                label,
                content,
                span,
            } => {
                self.validate_footnote_definition(label, content, span, context)?;
            }

            // Task lists and enhanced items
            Node::TaskItem {
                content,
                checked: _,
                span,
            } => {
                self.validate_task_item(content, span, context)?;
            }

            // Extended code blocks
            Node::FencedCodeBlock {
                language: _,
                info_string,
                content,
                fence_char: _,
                fence_length: _,
                span,
            } => {
                self.validate_fenced_code_block(info_string, content, span)?;
            }
            Node::IndentedCodeBlock { content, span } => {
                self.validate_indented_code_block(content, span)?;
            }

            // HTML elements
            Node::HtmlBlock {
                html_type: _,
                content,
                span,
            } => {
                self.validate_html_block(content, span)?;
            }
            Node::InlineHTML { content, span } => {
                self.validate_inline_html(content, span)?;
            }
            Node::BlockHTML { content, span } => {
                self.validate_block_html(content, span)?;
            }
            Node::HtmlInlineTag {
                tag_name,
                attributes: _,
                content: _,
                is_self_closing: _,
                span,
            } => {
                self.validate_html_inline_tag(tag_name, span)?;
            }

            // Special text elements
            Node::EscapedChar { character, span } => {
                self.validate_escaped_char(character, span)?;
            }
            Node::CodeSpan {
                content,
                backtick_count: _,
                span,
            } => {
                self.validate_code_span(content, span)?;
            }
            Node::SoftLineBreak { span: _ } => {
                // No specific validation needed for soft line breaks
            }
            Node::HardLineBreak { span: _ } => {
                // No specific validation needed for hard line breaks
            }

            // Extended elements
            Node::Emoji { name, span } => {
                self.validate_emoji(name, span)?;
            }
            Node::Keyboard { keys, span } => {
                self.validate_keyboard(keys, span)?;
            }
            Node::Citation {
                key,
                locator: _,
                span,
            } => {
                self.validate_citation(key, span, context)?;
            }
            Node::Details {
                summary,
                content,
                open: _,
                span,
            } => {
                self.validate_details(summary, content, span, context)?;
            }

            // Headings
            Node::SetextHeading {
                level,
                content,
                underline_char: _,
                span,
            } => {
                self.validate_heading(*level, content, span, context)?;
            }

            // Lists and definitions
            Node::DefinitionList { items, span } => {
                self.validate_definition_list(items, span, context)?;
            }
            Node::DefinitionTerm { content, span } => {
                self.validate_definition_term(content, span, context)?;
            }
            Node::DefinitionDescription { content, span } => {
                self.validate_definition_description(content, span, context)?;
            }

            // Breaks
            Node::ThematicBreak { marker: _, span: _ } => {
                // No specific validation needed for thematic breaks
            }

            // Enhanced user mentions
            Node::UserMentionWithMetadata {
                username,
                platform: _,
                display_name: _,
                user_id: _,
                avatar_url: _,
                span,
            } => {
                if self.config.validate_marco_extensions {
                    self.validate_user_mention_with_metadata(username, span, context)?;
                }
            }

            // Error recovery
            Node::Unknown {
                content: _,
                rule: _,
                span: _,
            } => {
                // Unknown nodes are valid by definition (for error recovery)
            }

            // Catch-all for any remaining unhandled types
            _ => {
                // For truly unhandled node types, validate any children if they exist
                if let Some(children) = node.children() {
                    self.validate_children(children, context)?;
                }
            }
        }

        context.exit_node();
        Ok(())
    }

    /// Validate document structure
    fn validate_document(
        &self,
        children: &[Node],
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        self.validate_children_count(children, "Document")?;
        self.validate_children(children, context)?;
        Ok(())
    }

    /// Validate heading structure
    fn validate_heading(
        &self,
        level: u8,
        content: &[Node],
        span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        // Check heading level
        if level == 0 || level > self.config.max_heading_level {
            return Err(MarcoError::invalid_content_type(
                "Heading",
                format!("level 1-{}", self.config.max_heading_level),
                format!("level {}", level),
            ));
        }

        // Check for proper nesting (headings shouldn't contain block elements)
        self.validate_inline_only_content(content, "Heading", context)?;

        // Validate content is not empty
        if content.is_empty() {
            return Err(MarcoError::empty_content("Heading"));
        }

        self.validate_children(content, context)?;
        Ok(())
    }

    /// Validate paragraph structure
    fn validate_paragraph(
        &self,
        content: &[Node],
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        if content.is_empty() {
            return Err(MarcoError::empty_content("Paragraph"));
        }

        // Paragraphs should only contain inline elements
        self.validate_inline_only_content(content, "Paragraph", context)?;
        self.validate_children(content, context)?;
        Ok(())
    }

    /// Validate text content
    fn validate_text(&self, content: &str, _span: &Span) -> MarcoResult<()> {
        if content.len() > self.config.max_text_length {
            return Err(MarcoError::content_overflow(
                "Text",
                self.config.max_text_length,
            ));
        }
        Ok(())
    }

    /// Validate inline formatting elements
    fn validate_inline_formatting(
        &self,
        content: &[Node],
        _span: &Span,
        element_type: &str,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        if content.is_empty() {
            return Err(MarcoError::empty_content(element_type));
        }
        self.validate_children(content, context)?;
        Ok(())
    }

    /// Validate code content
    fn validate_code_content(&self, content: &str, _span: &Span) -> MarcoResult<()> {
        // Code content can be empty (empty code spans are valid)
        if content.len() > self.config.max_text_length {
            return Err(MarcoError::content_overflow(
                "Code",
                self.config.max_text_length,
            ));
        }
        Ok(())
    }

    /// Validate code block
    fn validate_code_block(
        &self,
        language: &Option<String>,
        content: &str,
        _span: &Span,
    ) -> MarcoResult<()> {
        if let Some(lang) = language {
            // Basic language identifier validation
            if lang.is_empty() {
                return Err(MarcoError::invalid_content_type(
                    "CodeBlock",
                    "non-empty language identifier",
                    "empty string",
                ));
            }

            // Language should not contain whitespace or special characters
            if lang
                .chars()
                .any(|c| c.is_whitespace() || "{}[]()".contains(c))
            {
                return Err(MarcoError::invalid_content_type(
                    "CodeBlock",
                    "valid language identifier",
                    format!("'{}'", lang),
                ));
            }
        }

        if content.len() > self.config.max_text_length {
            return Err(MarcoError::content_overflow(
                "CodeBlock",
                self.config.max_text_length,
            ));
        }

        Ok(())
    }

    /// Validate link
    fn validate_link(
        &self,
        url: &str,
        content: &[Node],
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        // URL validation
        if url.is_empty() {
            return Err(MarcoError::invalid_content_type(
                "Link",
                "non-empty URL",
                "empty string",
            ));
        }

        // Basic URL format validation
        if !url.starts_with("http://")
            && !url.starts_with("https://")
            && !url.starts_with("mailto:")
            && !url.starts_with("ftp://")
            && !url.starts_with("/")
            && !url.starts_with("./")
            && !url.starts_with("../")
            && !url.starts_with("#")
        {
            // Allow relative paths and anchors, but validate others
            if !url.contains("://") && !url.starts_with("www.") {
                // This might be a relative path or anchor, which is OK
            }
        }

        // Content validation
        if content.is_empty() {
            return Err(MarcoError::empty_content("Link"));
        }

        self.validate_inline_only_content(content, "Link", context)?;
        self.validate_children(content, context)?;
        Ok(())
    }

    /// Validate image
    fn validate_image(&self, url: &str, alt: &str, _span: &Span) -> MarcoResult<()> {
        if url.is_empty() {
            return Err(MarcoError::invalid_content_type(
                "Image",
                "non-empty URL",
                "empty string",
            ));
        }

        // Alt text should exist (accessibility)
        if self.config.strict_commonmark && alt.is_empty() {
            return Err(MarcoError::invalid_content_type(
                "Image",
                "non-empty alt text",
                "empty string",
            ));
        }

        Ok(())
    }

    /// Validate list
    fn validate_list(
        &self,
        items: &[Node],
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        if items.is_empty() {
            return Err(MarcoError::empty_content("List"));
        }

        self.validate_children_count(items, "List")?;

        // All children should be list items
        for item in items {
            if !matches!(item, Node::ListItem { .. }) {
                return Err(MarcoError::invalid_nesting(
                    "List",
                    self.get_node_type(item),
                ));
            }
        }

        self.validate_children(items, context)?;
        Ok(())
    }

    /// Validate list item
    fn validate_list_item(
        &self,
        content: &[Node],
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        // List items can be empty (just a marker)
        self.validate_children(content, context)?;
        Ok(())
    }

    /// Validate block quote
    fn validate_block_quote(
        &self,
        content: &[Node],
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        if content.is_empty() {
            return Err(MarcoError::empty_content("BlockQuote"));
        }
        self.validate_children(content, context)?;
        Ok(())
    }

    /// Validate admonition (Marco extension)
    fn validate_admonition(
        &self,
        kind: &str,
        content: &[Node],
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        // Check if admonition type is allowed
        if !self.config.allowed_admonition_types.contains(kind) {
            return Err(MarcoError::invalid_admonition_type(kind));
        }

        if content.is_empty() {
            return Err(MarcoError::empty_content("Admonition"));
        }

        self.validate_children(content, context)?;
        Ok(())
    }

    /// Validate user mention (Marco extension)
    fn validate_user_mention(
        &self,
        username: &str,
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        if username.is_empty() {
            return Err(MarcoError::invalid_user_mention("Username cannot be empty"));
        }

        // Basic username validation
        if username.len() > 50 {
            return Err(MarcoError::invalid_user_mention(
                "Username too long (max 50 characters)",
            ));
        }

        // Check for valid username characters
        if !username
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(MarcoError::invalid_user_mention(
                "Username can only contain alphanumeric characters, underscore, and dash",
            ));
        }

        // Track user mention for reference validation
        context.user_mentions.insert(username.to_string());
        Ok(())
    }

    /// Validate bookmark (Marco extension)
    fn validate_bookmark(
        &self,
        path: &str,
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        if path.is_empty() {
            return Err(MarcoError::invalid_bookmark(path, None));
        }

        // Basic path validation
        if path.contains("..") && self.config.strict_commonmark {
            return Err(MarcoError::invalid_bookmark(path, None));
        }

        // Track bookmark for reference validation
        context.bookmarks.insert(path.to_string());
        Ok(())
    }

    /// Validate tab block (Marco extension)  
    fn validate_tab_block_nodes(
        &self,
        tabs: &[Node],
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        if tabs.is_empty() {
            return Err(MarcoError::tabs_error(
                "Tab block must contain at least one tab",
            ));
        }

        if tabs.len() > 20 {
            return Err(MarcoError::tabs_error("Too many tabs (max 20)"));
        }

        let mut tab_names = HashSet::new();
        for tab in tabs {
            if let Node::Tab { name, content, .. } = tab {
                // Check for duplicate tab names
                if let Some(tab_name) = name {
                    if !tab_names.insert(tab_name.clone()) {
                        return Err(MarcoError::duplicate_identifier(tab_name, "TabBlock"));
                    }

                    // Validate tab name
                    if tab_name.is_empty() {
                        return Err(MarcoError::tabs_error("Tab name cannot be empty"));
                    }

                    if tab_name.len() > 50 {
                        return Err(MarcoError::tabs_error(
                            "Tab name too long (max 50 characters)",
                        ));
                    }
                }

                // Validate tab content
                self.validate_children(content, context)?;
            } else {
                return Err(MarcoError::invalid_nesting(
                    "TabBlock",
                    self.get_node_type(tab),
                ));
            }
        }

        Ok(())
    }

    /// Validate table structure
    fn validate_table(
        &self,
        headers: &[Node],
        rows: &[Vec<Node>],
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        // Validate headers
        if headers.is_empty() {
            return Err(MarcoError::empty_content("Table headers"));
        }

        // Headers should only contain table cells
        for header in headers {
            if !matches!(header, Node::TableCell { .. }) {
                return Err(MarcoError::invalid_nesting(
                    "Table header",
                    self.get_node_type(header),
                ));
            }
        }

        let header_count = headers.len();

        // Validate each row
        for (row_index, row) in rows.iter().enumerate() {
            if row.len() != header_count {
                return Err(MarcoError::invalid_node_structure(
                    "Table row",
                    format!(
                        "Expected {} cells, found {} in row {}",
                        header_count,
                        row.len(),
                        row_index
                    ),
                    row_index,     // span start
                    row_index + 1, // span end
                ));
            }

            for cell in row {
                if !matches!(cell, Node::TableCell { .. }) {
                    return Err(MarcoError::invalid_nesting(
                        "Table row",
                        self.get_node_type(cell),
                    ));
                }
            }
        }

        // Validate all cells
        for header in headers {
            self.validate_node(header, context)?;
        }

        for row in rows {
            for cell in row {
                self.validate_node(cell, context)?;
            }
        }

        Ok(())
    }

    /// Validate table row
    fn validate_table_row(
        &self,
        cells: &[Node],
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        if cells.is_empty() {
            return Err(MarcoError::empty_content("Table row"));
        }

        // All children should be table cells
        for cell in cells {
            if !matches!(cell, Node::TableCell { .. }) {
                return Err(MarcoError::invalid_nesting(
                    "Table row",
                    self.get_node_type(cell),
                ));
            }
        }

        self.validate_children(cells, context)?;
        Ok(())
    }

    /// Validate individual validation methods for new node types
    fn validate_admonition_with_icon(
        &self,
        kind: &str,
        content: &[Node],
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        // Use same validation as regular admonition
        self.validate_admonition(kind, content, _span, context)
    }

    fn validate_tab_with_metadata(
        &self,
        name: &Option<String>,
        content: &[Node],
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        self.validate_tab(name, content, _span, context)
    }

    fn validate_tab(
        &self,
        name: &Option<String>,
        content: &[Node],
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        if let Some(tab_name) = name {
            if tab_name.is_empty() {
                return Err(MarcoError::tabs_error("Tab name cannot be empty"));
            }
            if tab_name.len() > 50 {
                return Err(MarcoError::tabs_error(
                    "Tab name too long (max 50 characters)",
                ));
            }
        }
        self.validate_children(content, context)?;
        Ok(())
    }

    fn validate_macro(&self, name: &str, _span: &Span) -> MarcoResult<()> {
        if name.is_empty() {
            return Err(MarcoError::malformed_construct(
                "Macro",
                "name cannot be empty",
            ));
        }
        if name.len() > 100 {
            return Err(MarcoError::malformed_construct(
                "Macro",
                "name too long (max 100 characters)",
            ));
        }
        Ok(())
    }

    fn validate_page_tag(&self, format: &Option<String>, _span: &Span) -> MarcoResult<()> {
        if let Some(fmt) = format {
            if fmt.is_empty() {
                return Err(MarcoError::invalid_content_type(
                    "PageTag",
                    "non-empty format",
                    "empty string",
                ));
            }
        }
        Ok(())
    }

    fn validate_table_of_contents(&self, depth: &Option<u8>, _span: &Span) -> MarcoResult<()> {
        if let Some(d) = depth {
            if *d == 0 || *d > 6 {
                return Err(MarcoError::invalid_content_type(
                    "TableOfContents",
                    "depth 1-6",
                    format!("depth {}", d),
                ));
            }
        }
        Ok(())
    }

    fn validate_run_block(
        &self,
        script_type: &str,
        content: &str,
        _span: &Span,
    ) -> MarcoResult<()> {
        if script_type.is_empty() {
            return Err(MarcoError::invalid_content_type(
                "RunBlock",
                "non-empty script type",
                "empty string",
            ));
        }

        if content.len() > self.config.max_text_length {
            return Err(MarcoError::content_overflow(
                "RunBlock",
                self.config.max_text_length,
            ));
        }

        Ok(())
    }

    fn validate_run_inline(
        &self,
        script_type: &str,
        command: &str,
        _span: &Span,
    ) -> MarcoResult<()> {
        if script_type.is_empty() {
            return Err(MarcoError::invalid_content_type(
                "RunInline",
                "non-empty script type",
                "empty string",
            ));
        }

        if command.is_empty() {
            return Err(MarcoError::invalid_content_type(
                "RunInline",
                "non-empty command",
                "empty string",
            ));
        }

        if command.len() > 1000 {
            return Err(MarcoError::content_overflow("RunInline", 1000));
        }

        Ok(())
    }

    fn validate_diagram_block(
        &self,
        diagram_type: &str,
        content: &str,
        _span: &Span,
    ) -> MarcoResult<()> {
        if diagram_type.is_empty() {
            return Err(MarcoError::invalid_content_type(
                "DiagramBlock",
                "non-empty diagram type",
                "empty string",
            ));
        }

        let valid_types = ["mermaid", "graphviz", "plantuml", "dot"];
        if !valid_types.contains(&diagram_type) {
            return Err(MarcoError::invalid_content_type(
                "DiagramBlock",
                "valid diagram type (mermaid, graphviz, plantuml, dot)",
                diagram_type,
            ));
        }

        if content.len() > self.config.max_text_length {
            return Err(MarcoError::content_overflow(
                "DiagramBlock",
                self.config.max_text_length,
            ));
        }

        Ok(())
    }

    fn validate_math_block_display(&self, content: &str, _span: &Span) -> MarcoResult<()> {
        self.validate_math_block(content, _span)
    }

    fn validate_math_inline(&self, content: &str, _span: &Span) -> MarcoResult<()> {
        if content.len() > self.config.max_text_length {
            return Err(MarcoError::content_overflow(
                "MathInline",
                self.config.max_text_length,
            ));
        }

        // Basic math content validation
        if content.trim().is_empty() {
            return Err(MarcoError::empty_content("MathInline"));
        }

        Ok(())
    }

    fn validate_math_block(&self, content: &str, _span: &Span) -> MarcoResult<()> {
        if content.len() > self.config.max_text_length {
            return Err(MarcoError::content_overflow(
                "MathBlock",
                self.config.max_text_length,
            ));
        }

        // Basic math content validation - could be extended with actual math parsing
        if content.trim().is_empty() {
            return Err(MarcoError::empty_content("MathBlock"));
        }

        Ok(())
    }

    fn validate_reference_link(
        &self,
        text: &[Node],
        label: &str,
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        if label.is_empty() {
            return Err(MarcoError::invalid_content_type(
                "ReferenceLink",
                "non-empty label",
                "empty string",
            ));
        }

        if text.is_empty() {
            return Err(MarcoError::empty_content("ReferenceLink"));
        }

        self.validate_inline_only_content(text, "ReferenceLink", context)?;
        self.validate_children(text, context)?;
        Ok(())
    }

    fn validate_reference_image(
        &self,
        alt: &str,
        label: &str,
        _span: &Span,
        _context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        if label.is_empty() {
            return Err(MarcoError::invalid_content_type(
                "ReferenceImage",
                "non-empty label",
                "empty string",
            ));
        }

        if self.config.strict_commonmark && alt.is_empty() {
            return Err(MarcoError::invalid_content_type(
                "ReferenceImage",
                "non-empty alt text",
                "empty string",
            ));
        }

        Ok(())
    }

    fn validate_reference_definition(
        &self,
        label: &str,
        url: &str,
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        if label.is_empty() {
            return Err(MarcoError::invalid_content_type(
                "ReferenceDefinition",
                "non-empty label",
                "empty string",
            ));
        }

        if url.is_empty() {
            return Err(MarcoError::invalid_content_type(
                "ReferenceDefinition",
                "non-empty URL",
                "empty string",
            ));
        }

        context
            .link_references
            .insert(label.to_string(), url.to_string());
        Ok(())
    }

    fn validate_autolink_url(&self, url: &str, _span: &Span) -> MarcoResult<()> {
        if url.is_empty() {
            return Err(MarcoError::invalid_content_type(
                "AutolinkUrl",
                "non-empty URL",
                "empty string",
            ));
        }
        Ok(())
    }

    fn validate_autolink_email(&self, email: &str, _span: &Span) -> MarcoResult<()> {
        if email.is_empty() {
            return Err(MarcoError::invalid_content_type(
                "AutolinkEmail",
                "non-empty email",
                "empty string",
            ));
        }

        if !email.contains('@') {
            return Err(MarcoError::invalid_content_type(
                "AutolinkEmail",
                "valid email format",
                email,
            ));
        }

        Ok(())
    }

    fn validate_inline_footnote(
        &self,
        content: &[Node],
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        if content.is_empty() {
            return Err(MarcoError::empty_content("InlineFootnote"));
        }

        self.validate_children(content, context)?;
        Ok(())
    }

    fn validate_task_item(
        &self,
        content: &[Node],
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        self.validate_children(content, context)?;
        Ok(())
    }

    fn validate_fenced_code_block(
        &self,
        info_string: &Option<String>,
        content: &str,
        _span: &Span,
    ) -> MarcoResult<()> {
        if let Some(info) = info_string {
            if info.contains('\n') {
                return Err(MarcoError::invalid_content_type(
                    "FencedCodeBlock",
                    "info string without newlines",
                    "info string with newlines",
                ));
            }
        }

        if content.len() > self.config.max_text_length {
            return Err(MarcoError::content_overflow(
                "FencedCodeBlock",
                self.config.max_text_length,
            ));
        }

        Ok(())
    }

    fn validate_indented_code_block(&self, content: &str, _span: &Span) -> MarcoResult<()> {
        if content.len() > self.config.max_text_length {
            return Err(MarcoError::content_overflow(
                "IndentedCodeBlock",
                self.config.max_text_length,
            ));
        }
        Ok(())
    }

    fn validate_inline_html(&self, content: &str, _span: &Span) -> MarcoResult<()> {
        if content.is_empty() {
            return Err(MarcoError::empty_content("InlineHTML"));
        }

        if content.len() > self.config.max_text_length {
            return Err(MarcoError::content_overflow(
                "InlineHTML",
                self.config.max_text_length,
            ));
        }

        Ok(())
    }

    fn validate_block_html(&self, content: &str, _span: &Span) -> MarcoResult<()> {
        if content.is_empty() {
            return Err(MarcoError::empty_content("BlockHTML"));
        }

        if content.len() > self.config.max_text_length {
            return Err(MarcoError::content_overflow(
                "BlockHTML",
                self.config.max_text_length,
            ));
        }

        Ok(())
    }

    fn validate_html_inline_tag(&self, tag_name: &str, _span: &Span) -> MarcoResult<()> {
        if tag_name.is_empty() {
            return Err(MarcoError::invalid_content_type(
                "HtmlInlineTag",
                "non-empty tag name",
                "empty string",
            ));
        }

        if tag_name.contains(' ') || tag_name.contains('\t') || tag_name.contains('\n') {
            return Err(MarcoError::invalid_content_type(
                "HtmlInlineTag",
                "tag name without whitespace",
                tag_name,
            ));
        }

        Ok(())
    }

    fn validate_escaped_char(&self, character: &char, _span: &Span) -> MarcoResult<()> {
        // Common escapable characters in Markdown
        let escapable = "\\`*_{}[]()#+-.!|~";
        if !escapable.contains(*character) && !character.is_ascii_punctuation() {
            return Err(MarcoError::invalid_content_type(
                "EscapedChar",
                "escapable character",
                format!("'{}'", character),
            ));
        }
        Ok(())
    }

    fn validate_code_span(&self, content: &str, _span: &Span) -> MarcoResult<()> {
        if content.len() > self.config.max_text_length {
            return Err(MarcoError::content_overflow(
                "CodeSpan",
                self.config.max_text_length,
            ));
        }
        Ok(())
    }

    fn validate_emoji(&self, name: &str, _span: &Span) -> MarcoResult<()> {
        if name.is_empty() {
            return Err(MarcoError::invalid_content_type(
                "Emoji",
                "non-empty name",
                "empty string",
            ));
        }

        if name.len() > 50 {
            return Err(MarcoError::content_overflow("Emoji name", 50));
        }

        Ok(())
    }

    fn validate_keyboard(&self, keys: &[String], _span: &Span) -> MarcoResult<()> {
        if keys.is_empty() {
            return Err(MarcoError::empty_content("Keyboard"));
        }

        for key in keys {
            if key.is_empty() {
                return Err(MarcoError::invalid_content_type(
                    "Keyboard",
                    "non-empty key",
                    "empty string",
                ));
            }
        }

        Ok(())
    }

    fn validate_citation(
        &self,
        key: &str,
        _span: &Span,
        _context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        if key.is_empty() {
            return Err(MarcoError::invalid_content_type(
                "Citation",
                "non-empty key",
                "empty string",
            ));
        }

        if key.len() > 100 {
            return Err(MarcoError::content_overflow("Citation key", 100));
        }

        Ok(())
    }

    fn validate_details(
        &self,
        summary: &[Node],
        content: &[Node],
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        if summary.is_empty() {
            return Err(MarcoError::empty_content("Details summary"));
        }

        self.validate_inline_only_content(summary, "Details summary", context)?;
        self.validate_children(summary, context)?;
        self.validate_children(content, context)?;

        Ok(())
    }

    fn validate_definition_list(
        &self,
        items: &[Node],
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        if items.is_empty() {
            return Err(MarcoError::empty_content("DefinitionList"));
        }

        // Validate that items are DefinitionTerm or DefinitionDescription
        for item in items {
            match item {
                Node::DefinitionTerm { .. } | Node::DefinitionDescription { .. } => {}
                _ => {
                    return Err(MarcoError::invalid_nesting(
                        "DefinitionList",
                        self.get_node_type(item),
                    ))
                }
            }
        }

        self.validate_children(items, context)?;
        Ok(())
    }

    fn validate_definition_term(
        &self,
        content: &[Node],
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        if content.is_empty() {
            return Err(MarcoError::empty_content("DefinitionTerm"));
        }

        self.validate_inline_only_content(content, "DefinitionTerm", context)?;
        self.validate_children(content, context)?;
        Ok(())
    }

    fn validate_definition_description(
        &self,
        content: &[Node],
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        self.validate_children(content, context)?;
        Ok(())
    }

    fn validate_user_mention_with_metadata(
        &self,
        username: &str,
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        self.validate_user_mention(username, _span, context)
    }

    fn validate_table_cell(
        &self,
        content: &[Node],
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        // Table cells can contain inline content
        self.validate_inline_only_content(content, "TableCell", context)?;
        self.validate_children(content, context)?;
        Ok(())
    }

    fn validate_document_reference(&self, path: &str, _span: &Span) -> MarcoResult<()> {
        if path.is_empty() {
            return Err(MarcoError::invalid_content_type(
                "DocumentReference",
                "non-empty path",
                "empty string",
            ));
        }

        if path.len() > 500 {
            return Err(MarcoError::content_overflow("DocumentReference path", 500));
        }

        Ok(())
    }

    fn validate_link_reference_definition(
        &self,
        label: &str,
        destination: &str,
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        self.validate_reference_definition(label, destination, _span, context)
    }

    fn validate_footnote_reference(
        &self,
        label: &str,
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        if label.is_empty() {
            return Err(MarcoError::invalid_content_type(
                "FootnoteReference",
                "non-empty label",
                "empty string",
            ));
        }

        // Track footnote reference for later validation
        context.footnote_references.insert(label.to_string());
        Ok(())
    }

    fn validate_footnote_definition(
        &self,
        label: &str,
        content: &[Node],
        _span: &Span,
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        if label.is_empty() {
            return Err(MarcoError::invalid_content_type(
                "FootnoteDefinition",
                "non-empty label",
                "empty string",
            ));
        }

        if content.is_empty() {
            return Err(MarcoError::empty_content("FootnoteDefinition"));
        }

        // Track footnote definition
        context.footnote_definitions.insert(label.to_string());
        self.validate_children(content, context)?;
        Ok(())
    }

    fn validate_html_block(&self, content: &str, _span: &Span) -> MarcoResult<()> {
        if content.is_empty() {
            return Err(MarcoError::empty_content("HtmlBlock"));
        }

        if content.len() > self.config.max_text_length {
            return Err(MarcoError::content_overflow(
                "HtmlBlock",
                self.config.max_text_length,
            ));
        }

        Ok(())
    }

    /// Validate that content contains only inline elements
    fn validate_inline_only_content(
        &self,
        content: &[Node],
        parent_type: &str,
        _context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        for node in content {
            if self.is_block_element(node) {
                return Err(MarcoError::invalid_nesting(
                    parent_type,
                    self.get_node_type(node),
                ));
            }
        }
        Ok(())
    }

    /// Validate children recursively
    fn validate_children(
        &self,
        children: &[Node],
        context: &mut ValidationContext,
    ) -> MarcoResult<()> {
        for child in children {
            self.validate_node(child, context)?;
        }
        Ok(())
    }

    /// Validate children count
    fn validate_children_count(&self, children: &[Node], node_type: &str) -> MarcoResult<()> {
        if children.len() > self.config.max_children {
            return Err(MarcoError::content_overflow(
                node_type,
                self.config.max_children,
            ));
        }
        Ok(())
    }

    /// Validate references at the end of validation
    fn validate_references(&self, context: &ValidationContext) -> MarcoResult<()> {
        // Check for unresolved footnote references
        for footnote_ref in &context.footnote_references {
            if !context.footnote_definitions.contains(footnote_ref) {
                return Err(MarcoError::unresolved_reference(footnote_ref, "footnote"));
            }
        }

        Ok(())
    }

    /// Get node type as string
    fn get_node_type(&self, node: &Node) -> String {
        match node {
            Node::Document { .. } => "Document",
            Node::Heading { .. } => "Heading",
            Node::SetextHeading { .. } => "SetextHeading",
            Node::Paragraph { .. } => "Paragraph",
            Node::Text { .. } => "Text",
            Node::Strong { .. } => "Strong",
            Node::Emphasis { .. } => "Emphasis",
            Node::Strikethrough { .. } => "Strikethrough",
            Node::Highlight { .. } => "Highlight",
            Node::Mark { .. } => "Mark",
            Node::Superscript { .. } => "Superscript",
            Node::Subscript { .. } => "Subscript",
            Node::Code { .. } => "Code",
            Node::CodeSpan { .. } => "CodeSpan",
            Node::CodeBlock { .. } => "CodeBlock",
            Node::FencedCodeBlock { .. } => "FencedCodeBlock",
            Node::IndentedCodeBlock { .. } => "IndentedCodeBlock",
            Node::MathInline { .. } => "MathInline",
            Node::MathBlock { .. } => "MathBlock",
            Node::MathBlockDisplay { .. } => "MathBlockDisplay",
            Node::Link { .. } => "Link",
            Node::Image { .. } => "Image",
            Node::Autolink { .. } => "Autolink",
            Node::AutolinkUrl { .. } => "AutolinkUrl",
            Node::AutolinkEmail { .. } => "AutolinkEmail",
            Node::ReferenceLink { .. } => "ReferenceLink",
            Node::ReferenceImage { .. } => "ReferenceImage",
            Node::ReferenceDefinition { .. } => "ReferenceDefinition",
            Node::LinkReferenceDefinition { .. } => "LinkReferenceDefinition",
            Node::FootnoteRef { .. } => "FootnoteRef",
            Node::InlineFootnote { .. } => "InlineFootnote",
            Node::FootnoteDefinition { .. } => "FootnoteDefinition",
            Node::List { .. } => "List",
            Node::ListItem { .. } => "ListItem",
            Node::TaskItem { .. } => "TaskItem",
            Node::BlockQuote { .. } => "BlockQuote",
            Node::HorizontalRule { .. } => "HorizontalRule",
            Node::ThematicBreak { .. } => "ThematicBreak",
            Node::LineBreak { .. } => "LineBreak",
            Node::HardLineBreak { .. } => "HardLineBreak",
            Node::SoftLineBreak { .. } => "SoftLineBreak",
            Node::EscapedChar { .. } => "EscapedChar",
            Node::Emoji { .. } => "Emoji",
            Node::Keyboard { .. } => "Keyboard",
            Node::Table { .. } => "Table",
            Node::TableHeader { .. } => "TableHeader",
            Node::TableRow { .. } => "TableRow",
            Node::TableCell { .. } => "TableCell",
            Node::DefinitionList { .. } => "DefinitionList",
            Node::DefinitionTerm { .. } => "DefinitionTerm",
            Node::DefinitionDescription { .. } => "DefinitionDescription",
            Node::Admonition { .. } => "Admonition",
            Node::AdmonitionWithIcon { .. } => "AdmonitionWithIcon",
            Node::UserMention { .. } => "UserMention",
            Node::UserMentionWithMetadata { .. } => "UserMentionWithMetadata",
            Node::Bookmark { .. } => "Bookmark",
            Node::PageTag { .. } => "PageTag",
            Node::DocumentReference { .. } => "DocumentReference",
            Node::TableOfContents { .. } => "TableOfContents",
            Node::RunInline { .. } => "RunInline",
            Node::RunBlock { .. } => "RunBlock",
            Node::DiagramBlock { .. } => "DiagramBlock",
            Node::TabBlock { .. } => "TabBlock",
            Node::Tab { .. } => "Tab",
            Node::TabWithMetadata { .. } => "TabWithMetadata",
            Node::Citation { .. } => "Citation",
            Node::Details { .. } => "Details",
            Node::Macro { .. } => "Macro",
            Node::InlineHTML { .. } => "InlineHTML",
            Node::BlockHTML { .. } => "BlockHTML",
            Node::HtmlBlock { .. } => "HtmlBlock",
            Node::HtmlInlineTag { .. } => "HtmlInlineTag",
            Node::Unknown { .. } => "Unknown",
        }
        .to_string()
    }

    /// Check if node is a block element
    fn is_block_element(&self, node: &Node) -> bool {
        matches!(
            node,
            Node::Document { .. }
                | Node::Heading { .. }
                | Node::SetextHeading { .. }
                | Node::Paragraph { .. }
                | Node::CodeBlock { .. }
                | Node::FencedCodeBlock { .. }
                | Node::IndentedCodeBlock { .. }
                | Node::MathBlock { .. }
                | Node::MathBlockDisplay { .. }
                | Node::List { .. }
                | Node::ListItem { .. }
                | Node::TaskItem { .. }
                | Node::BlockQuote { .. }
                | Node::HorizontalRule { .. }
                | Node::ThematicBreak { .. }
                | Node::Table { .. }
                | Node::TableHeader { .. }
                | Node::TableRow { .. }
                | Node::DefinitionList { .. }
                | Node::DefinitionTerm { .. }
                | Node::DefinitionDescription { .. }
                | Node::Admonition { .. }
                | Node::AdmonitionWithIcon { .. }
                | Node::TabBlock { .. }
                | Node::Tab { .. }
                | Node::TabWithMetadata { .. }
                | Node::RunBlock { .. }
                | Node::DiagramBlock { .. }
                | Node::Details { .. }
                | Node::BlockHTML { .. }
                | Node::HtmlBlock { .. }
        )
    }
}

impl Default for AstValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_document() {
        let validator = AstValidator::new();
        let doc = Node::Document {
            children: vec![],
            span: Span::empty(),
        };

        // Empty document should be valid
        assert!(validator.validate(&doc).is_ok());
    }

    #[test]
    fn test_validate_heading_level() {
        let validator = AstValidator::new();

        // Valid heading
        let valid_heading = Node::Heading {
            level: 3,
            content: vec![Node::Text {
                content: "Test".to_string(),
                span: Span::empty(),
            }],
            span: Span::empty(),
        };
        assert!(validator.validate(&valid_heading).is_ok());

        // Invalid heading level
        let invalid_heading = Node::Heading {
            level: 0,
            content: vec![Node::Text {
                content: "Test".to_string(),
                span: Span::empty(),
            }],
            span: Span::empty(),
        };
        assert!(validator.validate(&invalid_heading).is_err());
    }

    #[test]
    fn test_validate_admonition_type() {
        let validator = AstValidator::new();

        // Valid admonition
        let valid_admonition = Node::Admonition {
            kind: "note".to_string(),
            content: vec![Node::Text {
                content: "Test content".to_string(),
                span: Span::empty(),
            }],
            span: Span::empty(),
        };
        assert!(validator.validate(&valid_admonition).is_ok());

        // Invalid admonition type
        let invalid_admonition = Node::Admonition {
            kind: "invalid_type".to_string(),
            content: vec![Node::Text {
                content: "Test content".to_string(),
                span: Span::empty(),
            }],
            span: Span::empty(),
        };
        assert!(validator.validate(&invalid_admonition).is_err());
    }

    #[test]
    fn test_validate_nesting() {
        let validator = AstValidator::new();

        // Invalid: paragraph inside heading
        let invalid_nesting = Node::Heading {
            level: 1,
            content: vec![Node::Paragraph {
                content: vec![Node::Text {
                    content: "Nested paragraph".to_string(),
                    span: Span::empty(),
                }],
                span: Span::empty(),
            }],
            span: Span::empty(),
        };
        assert!(validator.validate(&invalid_nesting).is_err());
    }
}
