use super::{AstBuilder, BuilderHelpers, ErrorHandling, ParseContext};
use crate::components::marco_engine::{
    ast::{Node, Span},
    errors::MarcoResult,
    grammar::Rule,
};
use log::debug;
use pest::iterators::Pair;

// Constants for consistent values across the module
const DEFAULT_ADMONITION_KIND: &str = "note";
const TASK_CHECKED_MARKERS: [char; 2] = ['x', 'X'];

/// Trait for building block-level AST nodes
pub trait BlockBuilder: BuilderHelpers + ErrorHandling {
    /// Build document node containing all top-level blocks
    fn build_document(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("BlockBuilder::build_document - Processing document");
        let mut children = Vec::new();
        let mut document_span = span.clone();

        for inner_pair in pair.into_inner() {
            debug!(
                "BlockBuilder::build_document - Processing inner pair: {:?}",
                inner_pair.as_rule()
            );

            let inner_span = Self::create_span(&inner_pair);
            let inner_str = inner_pair.as_str();

            match AstBuilder::build_node(inner_pair) {
                Ok(node) => {
                    debug!(
                        "BlockBuilder::build_document - Successfully built node: {:?}",
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
                    children.push(node);
                }
                Err(e) => {
                    debug!("BlockBuilder::build_document - Error building node: {}", e);
                    // For error recovery, create a text node with the raw content
                    children.push(Self::create_text_node(inner_str, inner_span));
                }
            }
        }

        Ok(Node::document(children, document_span))
    }

    /// Build heading nodes (H1-H6)
    fn build_heading(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("BlockBuilder::build_heading - Processing heading");
        let mut level = 1u8;
        let mut content = Vec::new();

        let heading_text = pair.as_str(); // Store text before consuming

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::H1 => level = 1,
                Rule::H2 => level = 2,
                Rule::H3 => level = 3,
                Rule::H4 => level = 4,
                Rule::H5 => level = 5,
                Rule::H6 => level = 6,
                Rule::setext_h1 => level = 1,
                Rule::setext_h2 => level = 2,
                Rule::heading_inline => {
                    // Process heading content
                    content = Self::build_heading_content_with_spaces(inner_pair)?;
                }
                _ => {
                    // Handle other content with enhanced error recovery
                    let inner_span = Self::create_span(&inner_pair);
                    let inner_context = ParseContext::new(
                        &inner_pair,
                        AstBuilder::get_recovery_strategy(inner_pair.as_rule()),
                    );

                    match AstBuilder::build_node(inner_pair) {
                        Ok(node) => content.push(node),
                        Err(e) => {
                            match AstBuilder::handle_parse_error(
                                e,
                                inner_context,
                                inner_span.clone(),
                            ) {
                                Ok(Some(fallback_node)) => content.push(fallback_node),
                                Ok(None) => {
                                    // Skip this content
                                    log::debug!("Skipped problematic heading content");
                                }
                                Err(critical_error) => {
                                    // For headings, we don't want to fail completely, so log and continue
                                    log::warn!(
                                        "Critical error in heading content, using fallback: {}",
                                        critical_error
                                    );
                                    content.push(Self::create_text_node("", inner_span));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Validate heading level
        if let Err(e) = Self::validate_heading_level(level as usize) {
            log::warn!(
                "Heading level validation failed: {}, using text fallback",
                e
            );
            return Ok(Node::text(heading_text.to_string(), span));
        }

        // Fallback: if no content found, extract from original text
        if content.is_empty() {
            let cleaned = Self::extract_text_content(heading_text, &['#', '=', '-']);
            if !cleaned.is_empty() {
                content.push(Self::create_text_node_cow(cleaned, span.clone()));
            } else {
                // Empty heading, provide default content
                content.push(Self::create_text_node("Heading", span.clone()));
            }
        }

        Ok(Node::heading(level, content, span))
    }

    /// Build heading content preserving spaces
    fn build_heading_content_with_spaces(pair: Pair<Rule>) -> MarcoResult<Vec<Node>> {
        let mut content = Vec::new();
        let mut current_text = String::new();
        let span = Self::create_span(&pair);

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::text | Rule::word => {
                    current_text.push_str(inner_pair.as_str());
                }
                _ => {
                    // Flush current text if any
                    if !current_text.is_empty() {
                        content.push(Self::create_text_node(current_text.clone(), span.clone()));
                        current_text.clear();
                    }

                    // Process the non-text node
                    let inner_text = inner_pair.as_str(); // Store before consuming
                    match AstBuilder::build_node(inner_pair) {
                        Ok(node) => content.push(node),
                        Err(_) => {
                            content.push(Self::create_text_node(inner_text, span.clone()));
                        }
                    }
                }
            }
        }

        // Flush any remaining text
        if !current_text.is_empty() {
            content.push(Self::create_text_node(current_text, span));
        }

        Ok(content)
    }

    /// Build paragraph nodes
    fn build_paragraph(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("BlockBuilder::build_paragraph - Processing paragraph");
        let mut content = Vec::new();

        let paragraph_text = pair.as_str(); // Store text before consuming

        for inner_pair in pair.into_inner() {
            let inner_span = Self::create_span(&inner_pair);
            let inner_str = inner_pair.as_str();
            match AstBuilder::build_node(inner_pair) {
                Ok(node) => content.push(node),
                Err(_) => {
                    // Fallback to text node
                    content.push(Self::create_text_node(inner_str, inner_span));
                }
            }
        }

        // If no content, create from original text
        if content.is_empty() {
            let text = paragraph_text.trim();
            if !text.is_empty() {
                content.push(Self::create_text_node(text, span.clone()));
            }
        }

        Ok(Node::paragraph(content, span))
    }

    /// Build paragraph line
    fn build_paragraph_line(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("BlockBuilder::build_paragraph_line - Processing paragraph line");
        let content = Self::build_content_with_fallback(pair, span.clone())?;
        Ok(Node::paragraph(content, span))
    }

    /// Build code block nodes
    fn build_code_block(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("BlockBuilder::build_code_block - Processing code block");
        let mut language = None;
        let mut content = String::new();

        let full_text = pair.as_str(); // Store text before consuming

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::language_id => {
                    language = Some(inner_pair.as_str().to_string());
                }
                _ => {
                    content.push_str(inner_pair.as_str());
                }
            }
        }

        // Fallback: extract content from entire block
        if content.is_empty() {
            if full_text.starts_with("```") {
                // Fenced code block
                let lines: Vec<&str> = full_text.lines().collect();
                if lines.len() > 1 {
                    // Skip first and last lines (fence markers)
                    let end_idx = if lines.last().unwrap_or(&"").trim() == "```" {
                        lines.len() - 1
                    } else {
                        lines.len()
                    };
                    content = lines[1..end_idx].join("\n");

                    // Extract language from first line
                    if language.is_none() {
                        let first_line = lines[0].trim_start_matches("```").trim();
                        if !first_line.is_empty() {
                            language = Some(first_line.to_string());
                        }
                    }
                }
            } else {
                // Indented code block
                content = full_text.to_string();
            }
        }

        // Validate code block length
        if let Err(e) = Self::validate_code_block(&content) {
            log::warn!("Code block validation failed: {}, truncating", e);
            content.truncate(50_000); // Use constant from validation
        }

        Ok(Node::code_block(language, content, span))
    }

    /// Build math block nodes
    fn build_math_block(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("BlockBuilder::build_math_block - Processing math block");
        let content = pair.as_str().trim_matches('$').trim().to_string();
        Ok(Node::math_block(content, span))
    }

    /// Build list nodes
    fn build_list(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("BlockBuilder::build_list - Processing list");
        let mut items = Vec::new();
        let mut is_ordered = false;

        // Calculate nesting depth by analyzing the content structure
        let nesting_depth = Self::calculate_list_nesting_depth(&pair);

        // Validate list nesting depth
        if let Err(e) = Self::validate_list_nesting(nesting_depth) {
            log::warn!("List nesting validation failed: {}, using text fallback", e);
            return Ok(Node::text(pair.as_str().to_string(), span));
        }

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::list_item | Rule::regular_list_item | Rule::task_list_item => {
                    let inner_text = inner_pair.as_str(); // Store before consuming
                    let inner_span = Self::create_span(&inner_pair);
                    match Self::build_list_item(inner_pair) {
                        Ok(item) => {
                            // Determine if this is an ordered list
                            if let Node::ListItem { content, .. } = &item {
                                // Check if content starts with digits to detect ordered list
                                if let Some(Node::Text { content: text, .. }) = content.first() {
                                    if text.chars().any(|c| c.is_ascii_digit()) {
                                        is_ordered = true;
                                    }
                                }
                            }
                            items.push(item);
                        }
                        Err(e) => {
                            debug!("BlockBuilder::build_list - Error building list item: {}", e);
                            // Create fallback list item
                            items.push(Node::list_item(
                                vec![Self::create_text_node(inner_text, inner_span.clone())],
                                None,
                                inner_span,
                            ));
                        }
                    }
                }
                _ => {
                    debug!(
                        "BlockBuilder::build_list - Unexpected rule in list: {:?}",
                        inner_pair.as_rule()
                    );
                }
            }
        }

        Ok(Node::list(is_ordered, items, span))
    }

    /// Build list item nodes
    fn build_list_item(pair: Pair<Rule>) -> MarcoResult<Node> {
        debug!("BlockBuilder::build_list_item - Processing list item");
        let span = Self::create_span(&pair);
        let full_text = pair.as_str(); // Store before consuming
        let mut content = Vec::new();
        let mut checked = None;

        for inner_pair in pair.into_inner() {
            let inner_text = inner_pair.as_str(); // Store before consuming
            match inner_pair.as_rule() {
                Rule::list_marker => {
                    // Regular list marker - no action needed
                }
                Rule::unordered_marker => {
                    // Unordered list marker - no action needed
                }
                Rule::ordered_marker => {
                    // Ordered list marker - no action needed
                }
                Rule::task_marker => {
                    checked = Some(TASK_CHECKED_MARKERS.iter().any(|&c| inner_text.contains(c)));
                }
                Rule::list_item_content => {
                    let text_content = inner_text.trim();
                    if !text_content.is_empty() {
                        content.push(Self::create_text_node(text_content, span.clone()));
                    }
                }
                _ => match AstBuilder::build_node(inner_pair) {
                    Ok(node) => content.push(node),
                    Err(_) => {
                        content.push(Self::create_text_node(inner_text, span.clone()));
                    }
                },
            }
        }

        // Fallback: extract content from entire item
        if content.is_empty() {
            let trimmed_text = full_text.trim();
            let extracted_content = if let Some(stripped) = trimmed_text.strip_prefix("- [ ]") {
                checked = Some(false);
                stripped.trim().to_string()
            } else if let Some(stripped) = trimmed_text.strip_prefix("- [x]") {
                checked = Some(true);
                stripped.trim().to_string()
            } else if let Some(stripped) = trimmed_text.strip_prefix("- [X]") {
                checked = Some(true);
                stripped.trim().to_string()
            } else if trimmed_text.starts_with('-')
                || trimmed_text.starts_with('*')
                || trimmed_text.starts_with('+')
            {
                trimmed_text[1..].trim().to_string()
            } else if let Some(pos) = trimmed_text.find('.') {
                trimmed_text[pos + 1..].trim().to_string()
            } else {
                trimmed_text.to_string()
            };

            if !extracted_content.is_empty() {
                content.push(Self::create_text_node(extracted_content, span.clone()));
            }
        }

        if checked.is_some() {
            Ok(Node::task_item(checked.unwrap_or(false), content, span))
        } else {
            Ok(Node::list_item(content, None, span))
        }
    }

    /// Build admonition blocks
    fn build_admonition(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("BlockBuilder::build_admonition - Processing admonition");
        let mut kind = DEFAULT_ADMONITION_KIND.to_string();
        let mut content = Vec::new();
        let mut icon: Option<String> = None;
        let mut title: Option<String> = None;
        let full_text = pair.as_str(); // Store before consuming

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::admonition_type => {
                    kind = inner_pair.as_str().to_lowercase();
                }
                Rule::admonition_open => {
                    // Extract admonition type from opening tag
                    let open_text = inner_pair.as_str();
                    if let Some(stripped) = open_text.strip_prefix(":::") {
                        let type_part = stripped
                            .split_whitespace()
                            .next()
                            .unwrap_or(DEFAULT_ADMONITION_KIND);
                        kind = type_part.to_lowercase();

                        // Extract title if present after the type
                        let parts: Vec<&str> = stripped.split_whitespace().collect();
                        if parts.len() > 1 {
                            title = Some(parts[1..].join(" "));
                        }
                    }
                }
                Rule::admonition_emoji => {
                    // Handle emoji-style admonitions
                    let emoji_text = inner_pair.as_str();
                    if let Some(start) = emoji_text.find('[') {
                        if let Some(end) = emoji_text.find(']') {
                            kind = emoji_text[start + 1..end].trim().to_string();
                        }
                    }
                    // Extract icon if present (simple detection for common emojis)
                    if emoji_text.contains("🔥") {
                        icon = Some("🔥".to_string());
                    } else if emoji_text.contains("⚠️") {
                        icon = Some("⚠️".to_string());
                    } else if emoji_text.contains("💡") {
                        icon = Some("💡".to_string());
                    } else if emoji_text.contains("ℹ️") {
                        icon = Some("ℹ️".to_string());
                    }
                }
                Rule::admonition_close => {
                    // Ignore closing tag
                }
                _ => {
                    // Process content
                    let inner_text = inner_pair.as_str(); // Store before consuming
                    let inner_span = Self::create_span(&inner_pair);
                    match AstBuilder::build_node(inner_pair) {
                        Ok(node) => content.push(node),
                        Err(_) => {
                            let text = inner_text.trim();
                            if !text.is_empty() && text != ":::" {
                                content.push(Self::create_text_node(text, inner_span));
                            }
                        }
                    }
                }
            }
        }

        // Fallback content extraction
        if content.is_empty() {
            let lines: Vec<&str> = full_text.lines().collect();
            if lines.len() > 2 {
                // Skip first and last lines (opening and closing tags)
                let content_lines = &lines[1..lines.len() - 1];
                for line in content_lines {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        content.push(Self::create_text_node(trimmed, span.clone()));
                    }
                }
            }
        }

        // Create enhanced variant if we have icon or title, otherwise basic variant
        if icon.is_some() || title.is_some() {
            Ok(Node::admonition_with_icon(kind, icon, title, content, span))
        } else {
            Ok(Node::admonition(kind, content, span))
        }
    }

    /// Build blockquote nodes
    fn build_blockquote(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("BlockBuilder::build_blockquote - Processing blockquote");
        let mut content = Vec::new();
        let full_text = pair.as_str(); // Store before consuming

        for inner_pair in pair.into_inner() {
            let inner_text = inner_pair.as_str(); // Store before consuming
            match inner_pair.as_rule() {
                Rule::blockquote_line => {
                    let text = inner_text.trim();
                    if !text.is_empty() {
                        // Remove the '>' prefix if present
                        let clean_text = text.strip_prefix('>').unwrap_or(text).trim();
                        if !clean_text.is_empty() {
                            content.push(Self::create_text_node(clean_text, span.clone()));
                        }
                    }
                }
                _ => match AstBuilder::build_node(inner_pair) {
                    Ok(node) => content.push(node),
                    Err(e) => {
                        debug!("BlockBuilder::build_blockquote - Error in content: {}", e);
                        content.push(Self::create_text_node(inner_text, span.clone()));
                    }
                },
            }
        }

        // Fallback: extract content from entire blockquote
        if content.is_empty() {
            let lines: Vec<&str> = full_text
                .lines()
                .map(|line| line.trim().strip_prefix('>').unwrap_or(line).trim())
                .filter(|line| !line.is_empty())
                .collect();

            if !lines.is_empty() {
                content.push(Self::create_text_node(
                    lines.join(
                        "
",
                    ),
                    span.clone(),
                ));
            }
        }

        Ok(Node::blockquote(content, span))
    }

    /// Build horizontal rule nodes
    fn build_horizontal_rule(span: Span) -> MarcoResult<Node> {
        debug!("BlockBuilder::build_horizontal_rule - Processing horizontal rule");
        Ok(Node::horizontal_rule(span))
    }

    /// Build definition list nodes
    fn build_definition_list(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("BlockBuilder::build_definition_list - Processing definition list");
        let mut items = Vec::new();

        for inner_pair in pair.into_inner() {
            let inner_text = inner_pair.as_str(); // Store before consuming
            match inner_pair.as_rule() {
                Rule::term_line => {
                    let term_text = inner_text.trim();
                    if !term_text.is_empty() {
                        items.push(Node::definition_term(
                            vec![Self::create_text_node(term_text, span.clone())],
                            span.clone(),
                        ));
                    }
                }
                Rule::def_line => {
                    let def_text = inner_text.trim_start_matches(':').trim();
                    if !def_text.is_empty() {
                        items.push(Node::definition_description(
                            vec![Self::create_text_node(def_text, span.clone())],
                            span.clone(),
                        ));
                    }
                }
                _ => {
                    // Process other content
                    match AstBuilder::build_node(inner_pair) {
                        Ok(node) => {
                            items.push(node);
                        }
                        Err(_) => {
                            items.push(Self::create_text_node(inner_text, span.clone()));
                        }
                    }
                }
            }
        }

        Ok(Node::definition_list(items, span))
    }

    /// Build block HTML nodes
    fn build_block_html(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("BlockBuilder::build_block_html - Processing block HTML");
        let content = pair.as_str(); // Store before consuming
        Ok(Node::block_html(content.to_string(), span))
    }

    /// Calculate list nesting depth by analyzing indentation patterns
    fn calculate_list_nesting_depth(pair: &Pair<Rule>) -> usize {
        let content = pair.as_str();
        let lines: Vec<&str> = content.lines().collect();
        let mut max_depth = 0;

        for line in lines {
            if line.trim().is_empty() {
                continue;
            }

            // Count leading whitespace to determine nesting level
            let leading_spaces = line.len() - line.trim_start().len();

            // Each nesting level typically uses 2 or 4 spaces
            let depth = if leading_spaces > 0 {
                (leading_spaces / 2).max(leading_spaces / 4)
            } else {
                0
            };

            max_depth = max_depth.max(depth);
        }

        // Always return at least 1 for any list
        max_depth.max(1)
    }
}
