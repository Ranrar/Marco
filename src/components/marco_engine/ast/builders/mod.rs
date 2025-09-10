use crate::components::marco_engine::{
    ast::{Node, Span},
    errors::{MarcoError, MarcoResult},
    grammar::Rule,
};
use pest::iterators::{Pair, Pairs};
use std::borrow::Cow;

// Shared validation constants for consistent input validation across all builders
pub const MAX_TEXT_LENGTH: usize = 10_000; // Maximum text content length
pub const MAX_URL_LENGTH: usize = 2_048; // Maximum URL length (RFC 2616)
pub const MAX_TITLE_LENGTH: usize = 512; // Maximum link/image title length
pub const MAX_ALT_TEXT_LENGTH: usize = 512; // Maximum image alt text length
pub const MAX_LABEL_LENGTH: usize = 256; // Maximum reference label length
pub const MAX_USERNAME_LENGTH: usize = 32; // Maximum username length
pub const MAX_PLATFORM_LENGTH: usize = 20; // Maximum platform name length
pub const MAX_DISPLAY_NAME_LENGTH: usize = 64; // Maximum display name length
pub const MAX_PATH_LENGTH: usize = 512; // Maximum file path length
pub const MAX_LINE_NUMBER: u32 = 999_999; // Maximum line number
pub const MAX_HEADING_LEVEL: usize = 6; // Maximum heading level
pub const MAX_CODE_BLOCK_LENGTH: usize = 50_000; // Maximum code block length
pub const MAX_MATH_EXPRESSION_LENGTH: usize = 1_024; // Maximum math expression length
pub const MAX_TABLE_CELLS: usize = 1_000; // Maximum table cells for performance
pub const MAX_LIST_NESTING: usize = 20; // Maximum list nesting depth

// Character validation sets
pub const FORBIDDEN_CONTROL_CHARS: [char; 4] = ['\0', '\x08', '\x0C', '\x7F']; // NULL, BS, FF, DEL
pub const FORBIDDEN_URL_CHARS: [char; 7] = [' ', '\n', '\t', '<', '>', '[', ']'];
pub const FORBIDDEN_PATH_CHARS: [char; 6] = ['\0', '\n', '\t', '<', '>', '|'];
pub const ALLOWED_USERNAME_CHARS: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-";
pub const ALLOWED_PLATFORM_CHARS: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-";

// Re-export all builder modules
pub mod block;
pub mod inline;
pub mod links;
pub mod marco;
pub mod table;

// Import all builder traits
pub use block::BlockBuilder;
pub use inline::InlineBuilder;
pub use links::LinkBuilder;
pub use marco::MarcoBuilder;
pub use table::TableBuilder;

/// Shared trait for common builder functionality
pub trait BuilderHelpers {
    /// Helper function to create a span from a Pest pair
    fn create_span(pair: &Pair<Rule>) -> Span {
        Span::simple(pair.as_span().start() as u32, pair.as_span().end() as u32)
    }

    /// Helper function to create a text node with content and span
    fn create_text_node(content: impl AsRef<str>, span: Span) -> Node {
        Node::text(content.as_ref().to_string(), span)
    }

    /// Helper function to create a text node from a borrowed string, avoiding allocation if possible
    fn create_text_node_cow(content: Cow<str>, span: Span) -> Node {
        Node::text(content.into_owned(), span)
    }

    /// Build inline formatting with a generic constructor
    fn build_inline_formatting<F>(
        pair: Pair<Rule>,
        span: Span,
        node_constructor: F,
    ) -> MarcoResult<Node>
    where
        F: FnOnce(Vec<Node>, Span) -> Node,
    {
        let content_str = pair.as_str();
        let trimmed = content_str.trim_matches(|c: char| !c.is_alphanumeric() && c != ' ');
        let content = vec![Self::create_text_node(trimmed, span.clone())];
        Ok(node_constructor(content, span))
    }

    /// Build content with fallback to text node
    fn build_content_with_fallback(pair: Pair<Rule>, span: Span) -> MarcoResult<Vec<Node>> {
        let pair_str = pair.as_str(); // Get the string before consuming pair
        let mut content = Vec::new();
        for inner_pair in pair.into_inner() {
            let inner_span = Self::create_span(&inner_pair);
            let inner_context = ParseContext::new(
                &inner_pair,
                AstBuilder::get_recovery_strategy(inner_pair.as_rule()),
            );

            match AstBuilder::build_node(inner_pair) {
                Ok(node) => content.push(node),
                Err(e) => {
                    // Use enhanced error recovery
                    match AstBuilder::handle_parse_error(e, inner_context, inner_span) {
                        Ok(Some(fallback_node)) => content.push(fallback_node),
                        Ok(None) => {
                            // Skip this node
                            log::debug!("Skipped problematic node");
                        }
                        Err(critical_error) => {
                            // Re-propagate critical errors
                            return Err(critical_error);
                        }
                    }
                }
            }
        }
        if content.is_empty() {
            content.push(Self::create_text_node(pair_str, span));
        }
        Ok(content)
    }

    /// Build wrapper node by delegating to first inner pair
    fn build_wrapper_node(pair: Pair<Rule>) -> MarcoResult<Node> {
        let first_inner = Self::get_first_inner(pair)?;
        AstBuilder::build_node(first_inner)
    }

    /// Get the first inner pair or return error
    fn get_first_inner(pair: Pair<Rule>) -> MarcoResult<Pair<Rule>> {
        pair.into_inner()
            .next()
            .ok_or_else(|| MarcoError::parse_error("No inner pairs found".to_string()))
    }

    /// Extract trimmed text content efficiently
    fn extract_text_content<'a>(text: &'a str, trim_chars: &[char]) -> Cow<'a, str> {
        let mut start = 0;
        let mut end = text.len();

        // Find start position
        while start < end {
            if let Some(ch) = text.chars().nth(start) {
                if trim_chars.contains(&ch) {
                    start += ch.len_utf8();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Find end position
        while end > start {
            if let Some(ch) = text.chars().nth(end - 1) {
                if trim_chars.contains(&ch) || ch.is_whitespace() {
                    end -= ch.len_utf8();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if start == 0 && end == text.len() {
            Cow::Borrowed(text)
        } else {
            Cow::Owned(text[start..end].to_string())
        }
    }
}

/// Error recovery strategies for different types of parse failures
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorRecoveryStrategy {
    /// Critical errors that should bubble up and stop parsing
    Fail,
    /// Recoverable errors that should create fallback text nodes
    FallbackToText,
    /// Errors that should be logged but continue processing
    ContinueWithDefault,
    /// Errors that should skip the problematic content entirely
    Skip,
}

/// Error context for better error reporting
#[derive(Debug, Clone)]
pub struct ParseContext {
    pub rule: Rule,
    pub content: String,
    pub position: (u32, u32),
    pub strategy: ErrorRecoveryStrategy,
}

impl ParseContext {
    pub fn new(pair: &Pair<Rule>, strategy: ErrorRecoveryStrategy) -> Self {
        Self {
            rule: pair.as_rule(),
            content: pair.as_str().to_string(),
            position: (pair.as_span().start() as u32, pair.as_span().end() as u32),
            strategy,
        }
    }
}

/// Extended builder helpers with robust error handling
pub trait ErrorHandling: BuilderHelpers {
    /// Determine appropriate error recovery strategy based on rule type
    fn get_recovery_strategy(rule: Rule) -> ErrorRecoveryStrategy {
        match rule {
            // Critical structural elements should fail hard
            Rule::document | Rule::file => ErrorRecoveryStrategy::Fail,

            // Block elements can fallback to text
            Rule::heading
            | Rule::paragraph
            | Rule::list
            | Rule::code_block
            | Rule::math_block
            | Rule::blockquote
            | Rule::admonition_block => ErrorRecoveryStrategy::FallbackToText,

            // Inline elements should continue with defaults
            Rule::bold
            | Rule::italic
            | Rule::emphasis
            | Rule::strikethrough
            | Rule::highlight
            | Rule::superscript
            | Rule::subscript => ErrorRecoveryStrategy::ContinueWithDefault,

            // Complex elements like tables and links can fallback
            Rule::table | Rule::inline_link | Rule::inline_image => {
                ErrorRecoveryStrategy::FallbackToText
            }

            // Marco extensions are non-critical, can fallback
            Rule::user_mention
            | Rule::bookmark
            | Rule::page_tag
            | Rule::doc_ref
            | Rule::toc
            | Rule::run_inline
            | Rule::run_block_fenced
            | Rule::diagram_fenced
            | Rule::tab_block => ErrorRecoveryStrategy::FallbackToText,

            // Text and simple elements should never fail
            Rule::text | Rule::word => ErrorRecoveryStrategy::ContinueWithDefault,

            // Unknown elements should be skipped
            _ => ErrorRecoveryStrategy::Skip,
        }
    }

    /// Validate text content length and forbidden characters
    fn validate_text_content(text: &str) -> MarcoResult<()> {
        if text.len() > MAX_TEXT_LENGTH {
            return Err(MarcoError::parse_error(format!(
                "Text content exceeds maximum length of {} characters",
                MAX_TEXT_LENGTH
            )));
        }

        // Check for forbidden control characters
        for &forbidden_char in &FORBIDDEN_CONTROL_CHARS {
            if text.contains(forbidden_char) {
                return Err(MarcoError::parse_error(format!(
                    "Text contains forbidden control character: {:?}",
                    forbidden_char
                )));
            }
        }

        Ok(())
    }

    /// Validate URL format and length
    fn validate_url(url: &str) -> MarcoResult<()> {
        if url.is_empty() {
            return Err(MarcoError::parse_error("URL cannot be empty".to_string()));
        }

        if url.len() > MAX_URL_LENGTH {
            return Err(MarcoError::parse_error(format!(
                "URL exceeds maximum length of {} characters",
                MAX_URL_LENGTH
            )));
        }

        // Check for forbidden URL characters
        for &forbidden_char in &FORBIDDEN_URL_CHARS {
            if url.contains(forbidden_char) {
                return Err(MarcoError::parse_error(format!(
                    "URL contains invalid character: {:?}",
                    forbidden_char
                )));
            }
        }

        Ok(())
    }

    /// Validate link/image title length
    fn validate_title(title: &str) -> MarcoResult<()> {
        if title.len() > MAX_TITLE_LENGTH {
            return Err(MarcoError::parse_error(format!(
                "Title exceeds maximum length of {} characters",
                MAX_TITLE_LENGTH
            )));
        }
        Ok(())
    }

    /// Validate image alt text length
    fn validate_alt_text(alt_text: &str) -> MarcoResult<()> {
        if alt_text.len() > MAX_ALT_TEXT_LENGTH {
            return Err(MarcoError::parse_error(format!(
                "Alt text exceeds maximum length of {} characters",
                MAX_ALT_TEXT_LENGTH
            )));
        }
        Ok(())
    }

    /// Validate reference label
    fn validate_label(label: &str) -> MarcoResult<()> {
        if label.is_empty() {
            return Err(MarcoError::parse_error("Label cannot be empty".to_string()));
        }

        if label.len() > MAX_LABEL_LENGTH {
            return Err(MarcoError::parse_error(format!(
                "Label exceeds maximum length of {} characters",
                MAX_LABEL_LENGTH
            )));
        }

        // Labels should not contain certain characters for parsing clarity
        if label.contains('[') || label.contains(']') || label.contains('\n') {
            return Err(MarcoError::parse_error(
                "Label contains invalid characters: [, ], or newline".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate username for user mentions
    fn validate_username(username: &str) -> MarcoResult<()> {
        if username.is_empty() {
            return Err(MarcoError::parse_error(
                "Username cannot be empty".to_string(),
            ));
        }

        if username.len() > MAX_USERNAME_LENGTH {
            return Err(MarcoError::parse_error(format!(
                "Username exceeds maximum length of {} characters",
                MAX_USERNAME_LENGTH
            )));
        }

        // Check that username only contains allowed characters
        if !username.chars().all(|c| ALLOWED_USERNAME_CHARS.contains(c)) {
            return Err(MarcoError::parse_error(
                "Username contains invalid characters. Only alphanumeric, underscore, and hyphen allowed".to_string()
            ));
        }

        Ok(())
    }

    /// Validate platform name for user mentions
    fn validate_platform(platform: &str) -> MarcoResult<()> {
        if platform.is_empty() {
            return Err(MarcoError::parse_error(
                "Platform cannot be empty".to_string(),
            ));
        }

        if platform.len() > MAX_PLATFORM_LENGTH {
            return Err(MarcoError::parse_error(format!(
                "Platform exceeds maximum length of {} characters",
                MAX_PLATFORM_LENGTH
            )));
        }

        // Check that platform only contains allowed characters
        if !platform.chars().all(|c| ALLOWED_PLATFORM_CHARS.contains(c)) {
            return Err(MarcoError::parse_error(
                "Platform contains invalid characters. Only alphanumeric, underscore, and hyphen allowed".to_string()
            ));
        }

        Ok(())
    }

    /// Validate display name for user mentions
    fn validate_display_name(display_name: &str) -> MarcoResult<()> {
        if display_name.len() > MAX_DISPLAY_NAME_LENGTH {
            return Err(MarcoError::parse_error(format!(
                "Display name exceeds maximum length of {} characters",
                MAX_DISPLAY_NAME_LENGTH
            )));
        }

        // Check for forbidden control characters in display names
        for &forbidden_char in &FORBIDDEN_CONTROL_CHARS {
            if display_name.contains(forbidden_char) {
                return Err(MarcoError::parse_error(format!(
                    "Display name contains forbidden control character: {:?}",
                    forbidden_char
                )));
            }
        }

        Ok(())
    }

    /// Validate file path for bookmarks and references
    fn validate_path(path: &str) -> MarcoResult<()> {
        if path.is_empty() {
            return Err(MarcoError::parse_error("Path cannot be empty".to_string()));
        }

        if path.len() > MAX_PATH_LENGTH {
            return Err(MarcoError::parse_error(format!(
                "Path exceeds maximum length of {} characters",
                MAX_PATH_LENGTH
            )));
        }

        // Check for forbidden path characters
        for &forbidden_char in &FORBIDDEN_PATH_CHARS {
            if path.contains(forbidden_char) {
                return Err(MarcoError::parse_error(format!(
                    "Path contains invalid character: {:?}",
                    forbidden_char
                )));
            }
        }

        // Paths should not start or end with whitespace
        if path.starts_with(' ') || path.ends_with(' ') {
            return Err(MarcoError::parse_error(
                "Path cannot start or end with whitespace".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate line number
    fn validate_line_number(line_str: &str) -> MarcoResult<u32> {
        let line_num: u32 = line_str.parse().map_err(|_| {
            MarcoError::parse_error(format!("Invalid line number format: {}", line_str))
        })?;

        if line_num == 0 {
            return Err(MarcoError::parse_error(
                "Line number must be positive (1-based)".to_string(),
            ));
        }

        if line_num > MAX_LINE_NUMBER {
            return Err(MarcoError::parse_error(format!(
                "Line number {} exceeds maximum of {}",
                line_num, MAX_LINE_NUMBER
            )));
        }

        Ok(line_num)
    }

    /// Validate heading level
    fn validate_heading_level(level: usize) -> MarcoResult<()> {
        if level == 0 || level > MAX_HEADING_LEVEL {
            return Err(MarcoError::parse_error(format!(
                "Heading level {} is invalid. Must be between 1 and {}",
                level, MAX_HEADING_LEVEL
            )));
        }
        Ok(())
    }

    /// Validate code block length
    fn validate_code_block(code: &str) -> MarcoResult<()> {
        if code.len() > MAX_CODE_BLOCK_LENGTH {
            return Err(MarcoError::parse_error(format!(
                "Code block exceeds maximum length of {} characters",
                MAX_CODE_BLOCK_LENGTH
            )));
        }
        Ok(())
    }

    /// Validate math expression length
    fn validate_math_expression(math: &str) -> MarcoResult<()> {
        if math.len() > MAX_MATH_EXPRESSION_LENGTH {
            return Err(MarcoError::parse_error(format!(
                "Math expression exceeds maximum length of {} characters",
                MAX_MATH_EXPRESSION_LENGTH
            )));
        }
        Ok(())
    }

    /// Validate table cell count for performance
    fn validate_table_size(cell_count: usize) -> MarcoResult<()> {
        if cell_count > MAX_TABLE_CELLS {
            return Err(MarcoError::parse_error(format!(
                "Table has {} cells, exceeding maximum of {} for performance",
                cell_count, MAX_TABLE_CELLS
            )));
        }
        Ok(())
    }

    /// Validate list nesting depth
    fn validate_list_nesting(depth: usize) -> MarcoResult<()> {
        if depth > MAX_LIST_NESTING {
            return Err(MarcoError::parse_error(format!(
                "List nesting depth {} exceeds maximum of {}",
                depth, MAX_LIST_NESTING
            )));
        }
        Ok(())
    }

    /// Handle parse errors according to recovery strategy
    fn handle_parse_error(
        error: MarcoError,
        context: ParseContext,
        fallback_span: Span,
    ) -> MarcoResult<Option<Node>> {
        match context.strategy {
            ErrorRecoveryStrategy::Fail => {
                log::error!(
                    "Critical parse error in {:?} at {}:{}: {}",
                    context.rule,
                    context.position.0,
                    context.position.1,
                    error
                );
                Err(error)
            }

            ErrorRecoveryStrategy::FallbackToText => {
                log::warn!(
                    "Parse error in {:?}, falling back to text: {}",
                    context.rule,
                    error
                );
                Ok(Some(Node::text(context.content, fallback_span)))
            }

            ErrorRecoveryStrategy::ContinueWithDefault => {
                log::debug!(
                    "Recoverable parse error in {:?}, using default: {}",
                    context.rule,
                    error
                );
                Ok(Some(Node::text(context.content, fallback_span)))
            }

            ErrorRecoveryStrategy::Skip => {
                log::debug!(
                    "Skipping problematic content in {:?}: {}",
                    context.rule,
                    error
                );
                Ok(None)
            }
        }
    }
}

/// Main AST builder that delegates to specialized builders
pub struct AstBuilder;

impl AstBuilder {
    /// Main entry point for building AST from parsed pairs
    pub fn build(pairs: Pairs<Rule>) -> MarcoResult<Node> {
        log::debug!("AstBuilder::build - Starting AST building");
        let mut children = Vec::new();
        let mut document_span = Span::empty();
        let mut all_input = String::new();

        // Collect all input first to analyze structure
        let pairs_vec: Vec<_> = pairs.collect();
        for pair in &pairs_vec {
            all_input.push_str(pair.as_str());
        }

        for pair in pairs_vec {
            log::debug!(
                "AstBuilder::build - Processing pair: {:?} with text: '{}'",
                pair.as_rule(),
                pair.as_str()
            );

            let context = ParseContext::new(&pair, Self::get_recovery_strategy(pair.as_rule()));
            let pair_span = Self::create_span(&pair);

            match Self::build_node(pair) {
                Ok(node) => {
                    log::debug!(
                        "AstBuilder::build - Successfully built node: {:?}",
                        std::mem::discriminant(&node)
                    );
                    // Update document span to encompass all children
                    let node_span = node.span();
                    if children.is_empty() {
                        document_span = node_span.clone();
                    } else {
                        document_span = Span::simple(
                            document_span.start.min(node_span.start),
                            document_span.end.max(node_span.end),
                        );
                    }

                    // Handle document nodes specially - flatten them
                    match node {
                        Node::Document {
                            children: doc_children,
                            ..
                        } => {
                            children.extend(doc_children);
                        }
                        _ => {
                            children.push(node);
                        }
                    }
                }
                Err(e) => {
                    match Self::handle_parse_error(e, context, pair_span)? {
                        Some(fallback_node) => {
                            children.push(fallback_node);
                        }
                        None => {
                            // Node was skipped, continue processing
                        }
                    }
                }
            }
        }

        // Create final document node
        if children.is_empty() {
            document_span = Span::simple(0, all_input.len() as u32);
        }

        Ok(Node::document(children, document_span))
    }

    /// Public method for testing individual nodes
    pub fn build_node_for_testing(pair: Pair<Rule>) -> MarcoResult<Node> {
        Self::build_node(pair)
    }

    /// Main node building dispatcher - routes to appropriate builder modules
    pub fn build_node(pair: Pair<Rule>) -> MarcoResult<Node> {
        let span = Self::create_span(&pair);

        match pair.as_rule() {
            // Wrapper rules
            Rule::file | Rule::block => Self::build_wrapper_node(pair),

            // Block-level elements
            Rule::document => <AstBuilder as BlockBuilder>::build_document(pair, span),
            Rule::heading => <AstBuilder as BlockBuilder>::build_heading(pair, span),
            Rule::paragraph => <AstBuilder as BlockBuilder>::build_paragraph(pair, span),
            Rule::paragraph_line => <AstBuilder as BlockBuilder>::build_paragraph_line(pair, span),
            Rule::code_block => <AstBuilder as BlockBuilder>::build_code_block(pair, span),
            Rule::math_block => <AstBuilder as BlockBuilder>::build_math_block(pair, span),
            Rule::list => <AstBuilder as BlockBuilder>::build_list(pair, span),
            Rule::list_item | Rule::regular_list_item | Rule::task_list_item => {
                <AstBuilder as BlockBuilder>::build_list_item(pair)
            }
            Rule::admonition_block => <AstBuilder as BlockBuilder>::build_admonition(pair, span),
            Rule::blockquote => <AstBuilder as BlockBuilder>::build_blockquote(pair, span),
            Rule::hr => <AstBuilder as BlockBuilder>::build_horizontal_rule(span),
            Rule::def_list => <AstBuilder as BlockBuilder>::build_definition_list(pair, span),
            Rule::block_html => <AstBuilder as BlockBuilder>::build_block_html(pair, span),

            // Inline elements
            Rule::inline => <AstBuilder as InlineBuilder>::build_inline(pair),
            Rule::inline_core => <AstBuilder as InlineBuilder>::build_inline_core(pair),
            Rule::bold => <AstBuilder as InlineBuilder>::build_strong(pair, span),
            Rule::emphasis => <AstBuilder as InlineBuilder>::build_emphasis(pair, span),
            Rule::italic | Rule::italic_asterisk | Rule::italic_underscore => {
                <AstBuilder as InlineBuilder>::build_emphasis(pair, span)
            }
            Rule::strikethrough | Rule::strikethrough_tilde | Rule::strikethrough_dash => {
                <AstBuilder as InlineBuilder>::build_strikethrough(pair, span)
            }
            Rule::highlight => <AstBuilder as InlineBuilder>::build_highlight(pair, span),
            Rule::superscript => <AstBuilder as InlineBuilder>::build_superscript(pair, span),
            Rule::subscript | Rule::subscript_arrow | Rule::subscript_tilde => {
                <AstBuilder as InlineBuilder>::build_subscript(pair, span)
            }
            Rule::math_inline => <AstBuilder as InlineBuilder>::build_inline_math(pair, span),
            Rule::emoji => <AstBuilder as InlineBuilder>::build_emoji(pair, span),
            Rule::line_break => <AstBuilder as InlineBuilder>::build_line_break(span),
            Rule::escaped_char => <AstBuilder as InlineBuilder>::build_escaped_char(pair, span),
            Rule::code_inline => <AstBuilder as InlineBuilder>::build_code_inline(pair, span),
            Rule::inline_html => <AstBuilder as InlineBuilder>::build_inline_html(pair, span),

            // Links and references
            Rule::inline_link => <AstBuilder as LinkBuilder>::build_link(pair, span),
            Rule::inline_image => <AstBuilder as LinkBuilder>::build_image(pair, span),
            Rule::autolink | Rule::autolink_email | Rule::autolink_url => {
                <AstBuilder as LinkBuilder>::build_autolink(pair, span)
            }
            Rule::reference_link => <AstBuilder as LinkBuilder>::build_reference_link(pair, span),
            Rule::reference_image => <AstBuilder as LinkBuilder>::build_reference_image(pair, span),
            Rule::reference_definition => {
                <AstBuilder as LinkBuilder>::build_reference_definition(pair, span)
            }
            Rule::footnote_ref => <AstBuilder as LinkBuilder>::build_footnote_ref(pair, span),
            Rule::inline_footnote_ref => {
                <AstBuilder as LinkBuilder>::build_inline_footnote(pair, span)
            }
            Rule::footnote_def => {
                <AstBuilder as LinkBuilder>::build_footnote_definition(pair, span)
            }

            // Tables
            Rule::table => <AstBuilder as TableBuilder>::build_table(pair, span),

            // Marco extensions
            Rule::inline_task_item => <AstBuilder as MarcoBuilder>::build_task_item(pair, span),
            Rule::user_mention => <AstBuilder as MarcoBuilder>::build_user_mention(pair, span),
            Rule::bookmark => <AstBuilder as MarcoBuilder>::build_bookmark(pair, span),
            Rule::page_tag => <AstBuilder as MarcoBuilder>::build_page_tag(pair, span),
            Rule::doc_ref => <AstBuilder as MarcoBuilder>::build_document_reference(pair, span),
            Rule::toc => <AstBuilder as MarcoBuilder>::build_table_of_contents(pair, span),
            Rule::run_inline => <AstBuilder as MarcoBuilder>::build_run_inline(pair, span),
            Rule::run_block_fenced => <AstBuilder as MarcoBuilder>::build_run_block(pair, span),
            Rule::diagram_fenced => <AstBuilder as MarcoBuilder>::build_diagram_block(pair, span),
            Rule::tab_block => <AstBuilder as MarcoBuilder>::build_tab_block(pair, span),

            // Text and other rules
            Rule::text => {
                let text_content = pair.as_str();
                if text_content.contains('\n') {
                    // For multiline text, return as-is and let parent handle splitting
                    Ok(Node::text(text_content.to_string(), span))
                } else {
                    Ok(Node::text(text_content.to_string(), span))
                }
            }
            Rule::heading_inline => Self::build_wrapper_node(pair),
            Rule::word => Ok(Node::text(pair.as_str().to_string(), span)),

            // Fallback for unknown rules
            Rule::unknown_block => Self::build_unknown(pair, span),
            _ => {
                log::warn!("Unknown rule encountered: {:?}", pair.as_rule());
                Self::build_unknown(pair, span)
            }
        }
    }

    /// Build unknown/unhandled nodes
    fn build_unknown(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        Ok(Node::text(pair.as_str().to_string(), span))
    }
}

// Implement all helper traits for AstBuilder
impl BuilderHelpers for AstBuilder {}
impl ErrorHandling for AstBuilder {}
impl BlockBuilder for AstBuilder {}
impl InlineBuilder for AstBuilder {}
impl LinkBuilder for AstBuilder {}
impl TableBuilder for AstBuilder {}
impl MarcoBuilder for AstBuilder {}
