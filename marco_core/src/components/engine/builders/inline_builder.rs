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
    lookbehind,  // Lookbehind validation for emphasis delimiters
};
use pest::iterators::Pair;

// AstError removed - using AstError from mod.rs

/// Builder for inline-level AST nodes
pub struct InlineBuilder {
    /// Full source text for lookbehind/lookahead validation
    source_text: Option<String>,
}

impl InlineBuilder {
    /// Create a new inline builder without source text
    pub fn new() -> Self {
        Self {
            source_text: None,
        }
    }

    /// Create a new inline builder with source text for lookbehind validation
    pub fn with_source(source_text: String) -> Self {
        Self {
            source_text: Some(source_text),
        }
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

            Rule::strong => {
                // Parent rule - find the actual variant (strong_asterisk or strong_underscore)
                let variant = pair.into_inner().next()
                    .ok_or_else(|| AstError::MissingContent("strong rule has no children".to_string()))?;
                self.build_inline_node(variant)
            }

            Rule::strong_asterisk | Rule::strong_underscore => {
                // Validate delimiters using lookbehind rules
                if !self.validate_emphasis_delimiters(&pair, &span) {
                    // Delimiters don't meet CommonMark left/right-flanking rules
                    // Return as plain text instead
                    return Ok(Node::text(pair.as_str().to_string(), span));
                }
                
                // Extract the inner content text and recursively parse it
                let content_text = self.extract_formatting_content(&pair)?;
                let children = self.parse_inline_text_recursively(&content_text, &span)?;
                Ok(Node::strong(children, span))
            }

            Rule::emphasis => {
                // Parent rule - find the actual variant (emphasis_asterisk or emphasis_underscore)
                let variant = pair.into_inner().next()
                    .ok_or_else(|| AstError::MissingContent("emphasis rule has no children".to_string()))?;
                self.build_inline_node(variant)
            }

            Rule::emphasis_asterisk | Rule::emphasis_underscore => {
                // Validate delimiters using lookbehind rules
                if !self.validate_emphasis_delimiters(&pair, &span) {
                    // Delimiters don't meet CommonMark left/right-flanking rules
                    // Return as plain text instead
                    return Ok(Node::text(pair.as_str().to_string(), span));
                }
                
                // Extract the inner content text and recursively parse it
                let content_text = self.extract_formatting_content(&pair)?;
                let children = self.parse_inline_text_recursively(&content_text, &span)?;
                Ok(Node::emphasis(children, span))
            }

            Rule::link => {
                // New grammar has link as dispatcher for inline_link, link_full_reference, etc.
                let (text_nodes, url, title) = self.extract_link_content(pair)?;
                
                // Check if this is a reference link (marked with [REF:label])
                if url.starts_with("[REF:") && url.ends_with(']') {
                    // Extract the label
                    let label = url.trim_start_matches("[REF:").trim_end_matches(']').to_string();
                    Ok(Node::ReferenceLink {
                        text: text_nodes,
                        label,
                        span,
                    })
                } else {
                    // Regular inline link
                    Ok(Node::link(text_nodes, url, title, span))
                }
            }

            Rule::image => {
                // New grammar has image as dispatcher for inline_image, image_full_reference, etc.
                let (alt_text, url, title) = self.extract_image_content(pair)?;
                
                // Check if this is a reference image (marked with [REF:label])
                if url.starts_with("[REF:") && url.ends_with(']') {
                    // Extract the label
                    let label = url.trim_start_matches("[REF:").trim_end_matches(']').to_string();
                    Ok(Node::ReferenceImage {
                        alt: alt_text,
                        label,
                        span,
                    })
                } else {
                    // Regular inline image
                    Ok(Node::image(alt_text, url, title, span))
                }
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
                // Process inner rules to determine break type
                let inner = pair.into_inner().next()
                    .ok_or_else(|| AstError::InvalidStructure("Expected hard or soft line break".to_string()))?;
                
                match inner.as_rule() {
                    Rule::hard_line_break => {
                        Ok(Node::hard_line_break(span))
                    }
                    Rule::soft_line_break => {
                        Ok(Node::soft_line_break(span))
                    }
                    _ => Err(AstError::InvalidStructure(format!("Unexpected line break type: {:?}", inner.as_rule())))
                }
            }

            Rule::escape => {
                let ch = pair.as_str().chars().nth(1).unwrap_or('\\');
                Ok(Node::escaped_char(ch, span))
            }

            Rule::entity_reference => {
                // Entity and numeric character references (Phase 4)
                let decoded = self.extract_entity_reference(&pair)?;
                Ok(Node::text(decoded, span))
            }

            Rule::html_tag => {
                // HTML tag - store as-is
                let content = pair.as_str().to_string();
                Ok(Node::text(content, span)) // TODO: Create proper HTML node type
            }

            // Inner rules that might appear (skip or handle)
            Rule::inline_content => {
                // This is a container - should NOT be called directly
                // The parse_inline_content method in BlockBuilder should iterate the children
                Err(AstError::InvalidStructure(format!(
                    "inline_content should be handled by parent, not build_inline_node. Got: {}",
                    pair.as_str()
                )))
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

    /// Extract text content from strong/emphasis formatting rules
    /// Strips the delimiter characters (**/__/*/_) and returns inner text
    /// Validate emphasis/strong delimiters using lookbehind rules
    ///
    /// Returns true if the delimiter is valid according to CommonMark left/right-flanking rules
    fn validate_emphasis_delimiters(&self, pair: &Pair<Rule>, span: &Span) -> bool {
        // If we don't have source text, skip validation (fallback to grammar-only check)
        let source = match &self.source_text {
            Some(s) => s,
            None => return true, // No validation without source text
        };

        let text = pair.as_str();
        
        let start_pos = span.start as usize;
        let end_pos = span.end as usize;
        
        match pair.as_rule() {
            Rule::emphasis_asterisk => {
                // Check if opening * is left-flanking and closing * is right-flanking
                lookbehind::can_asterisk_open_emphasis(source, start_pos)
                    && lookbehind::can_asterisk_close_emphasis(source, end_pos - 1)
            }
            Rule::emphasis_underscore => {
                // Check if opening _ is left-flanking and closing _ is right-flanking
                // (with additional intraword restrictions)
                lookbehind::can_underscore_open_emphasis(source, start_pos)
                    && lookbehind::can_underscore_close_emphasis(source, end_pos - 1)
            }
            Rule::strong_asterisk => {
                // Check if opening ** is left-flanking and closing ** is right-flanking
                lookbehind::is_left_flanking(source, start_pos, 2)
                    && lookbehind::is_right_flanking(source, end_pos - 2, 2)
            }
            Rule::strong_underscore => {
                // For __, additional intraword restrictions apply (similar to single _)
                // CommonMark spec says __ cannot be used for intraword emphasis
                if !lookbehind::is_left_flanking(source, start_pos, 2) {
                    return false;
                }
                if !lookbehind::is_right_flanking(source, end_pos - 2, 2) {
                    return false;
                }
                
                // Additional check: if opening __ is also right-flanking, must be preceded by punct
                if lookbehind::is_right_flanking(source, start_pos, 2) {
                    let prev_char = if start_pos == 0 {
                        return false;
                    } else {
                        source[..start_pos].chars().rev().next()
                    };
                    if let Some(c) = prev_char {
                        if !lookbehind::is_punctuation(c) {
                            return false;
                        }
                    }
                }
                
                // Check closing delimiter similarly
                if lookbehind::is_left_flanking(source, end_pos - 2, 2) {
                    let next_char = if end_pos >= source.len() {
                        return false;
                    } else {
                        source[end_pos..].chars().next()
                    };
                    if let Some(c) = next_char {
                        if !lookbehind::is_punctuation(c) {
                            return false;
                        }
                    }
                }
                
                true
            }
            _ => true, // Unknown rule, don't validate
        }
    }

    fn extract_formatting_content(&self, pair: &Pair<Rule>) -> Result<String, AstError> {
        let text = pair.as_str();
        
        // Determine delimiter based on rule
        let content = match pair.as_rule() {
            Rule::strong_asterisk => {
                // Remove ** from both ends
                text.strip_prefix("**")
                    .and_then(|s| s.strip_suffix("**"))
                    .ok_or_else(|| AstError::InvalidStructure("Invalid strong_asterisk format".to_string()))?
            }
            Rule::strong_underscore => {
                // Remove __ from both ends
                text.strip_prefix("__")
                    .and_then(|s| s.strip_suffix("__"))
                    .ok_or_else(|| AstError::InvalidStructure("Invalid strong_underscore format".to_string()))?
            }
            Rule::emphasis_asterisk => {
                // Remove * from both ends
                text.strip_prefix("*")
                    .and_then(|s| s.strip_suffix("*"))
                    .ok_or_else(|| AstError::InvalidStructure("Invalid emphasis_asterisk format".to_string()))?
            }
            Rule::emphasis_underscore => {
                // Remove _ from both ends
                text.strip_prefix("_")
                    .and_then(|s| s.strip_suffix("_"))
                    .ok_or_else(|| AstError::InvalidStructure("Invalid emphasis_underscore format".to_string()))?
            }
            Rule::strong | Rule::emphasis => {
                // Parent rule - should not be called directly here
                return Err(AstError::InvalidStructure(format!(
                    "extract_formatting_content called on parent rule: {:?}",
                    pair.as_rule()
                )));
            }
            _ => {
                return Err(AstError::InvalidStructure(format!(
                    "extract_formatting_content called on non-formatting rule: {:?}",
                    pair.as_rule()
                )));
            }
        };
        
        Ok(content.to_string())
    }

    /// Extract and decode entity reference
    /// 
    /// Handles three types of entities:
    /// 1. Named entities: &nbsp; &amp; &lt; etc.
    /// 2. Decimal numeric: &#35; &#169; etc.
    /// 3. Hexadecimal numeric: &#x23; &#xA9; etc.
    /// 
    /// Invalid entities render literally per CommonMark spec.
    /// 
    /// Note: The entity grammar uses atomic rules (@{}), so we parse the string directly
    /// instead of using inner pairs.
    fn extract_entity_reference(&self, pair: &Pair<Rule>) -> Result<String, AstError> {
        use crate::components::engine::entity_table::{
            decode_named_entity, decode_decimal_entity, decode_hex_entity
        };
        
        let full_text = pair.as_str(); // e.g., "&nbsp;", "&#35;", "&#x23;"
        
        // Remove & prefix and ; suffix to get inner content
        let inner_content = full_text
            .strip_prefix('&')
            .and_then(|s| s.strip_suffix(';'))
            .ok_or_else(|| AstError::InvalidStructure(
                format!("Entity missing & or ; delimiters: {}", full_text)
            ))?;
        
        // Determine entity type and decode
        if inner_content.starts_with('#') {
            // Numeric character reference
            if inner_content.len() > 1 {
                let after_hash = &inner_content[1..];
                
                if after_hash.starts_with('x') || after_hash.starts_with('X') {
                    // Hexadecimal: &#x23; or &#X23;
                    if after_hash.len() > 1 {
                        let hex_digits = &after_hash[1..];
                        match decode_hex_entity(hex_digits) {
                            Some(decoded) => Ok(decoded),
                            None => Ok(full_text.to_string()) // Invalid - render literally
                        }
                    } else {
                        // Just &#x; with no digits - render literally
                        Ok(full_text.to_string())
                    }
                } else {
                    // Decimal: &#35;
                    match decode_decimal_entity(after_hash) {
                        Some(decoded) => Ok(decoded),
                        None => Ok(full_text.to_string()) // Invalid - render literally
                    }
                }
            } else {
                // Just &# with nothing after - render literally  
                Ok(full_text.to_string())
            }
        } else {
            // Named entity: &nbsp;, &amp;, etc.
            match decode_named_entity(inner_content) {
                Some(decoded) => Ok(decoded),
                None => Ok(full_text.to_string()) // Invalid - render literally
            }
        }
    }

    /// Recursively parse inline text for nested elements
    /// Used for parsing content inside strong/emphasis that may contain other inline elements
    fn parse_inline_text_recursively(&self, text: &str, parent_span: &Span) -> Result<Vec<Node>, AstError> {
        use crate::components::engine::parsers::inline_parser::{InlineParser, Rule};
        use pest::Parser;
        
        // Handle empty text
        if text.trim().is_empty() {
            return Ok(vec![Node::text(String::new(), parent_span.clone())]);
        }
        
        // Parse as inline_content
        let mut pairs = InlineParser::parse(Rule::inline_content, text)
            .map_err(|e| AstError::ParseError(format!("Recursive inline parse failed: {}", e)))?;
        
        let inline_content_pair = pairs.next()
            .ok_or_else(|| AstError::MissingContent("No inline_content in recursive parse".to_string()))?;
        
        let mut nodes = Vec::new();
        for pair in inline_content_pair.into_inner() {
            nodes.push(self.build_inline_node(pair)?);
        }
        
        // If no nodes, return simple text
        if nodes.is_empty() {
            Ok(vec![Node::text(text.to_string(), parent_span.clone())])
        } else {
            Ok(nodes)
        }
    }

    /// Extract link content (text, URL, optional title)
    fn extract_link_content(
        &self,
        pair: Pair<Rule>,
    ) -> Result<(Vec<Node>, String, Option<String>), AstError> {
        let rule = pair.as_rule();

        match rule {
            Rule::link => {
                // Main dispatcher - extract the actual link type from inner pair
                let inner = pair.into_inner().next().ok_or_else(|| {
                    AstError::InvalidStructure("Empty link rule".to_string())
                })?;
                self.extract_link_content(inner)
            }
            Rule::inline_link => self.extract_inline_link_content(pair),
            Rule::link_full_reference => self.extract_full_reference_link(pair),
            Rule::link_collapsed_reference => self.extract_collapsed_reference_link(pair),
            Rule::link_shortcut_reference => self.extract_shortcut_reference_link(pair),
            _ => {
                Err(AstError::InvalidStructure(format!(
                    "Unexpected link type rule: {:?}",
                    rule
                )))
            }
        }
    }

    /// Extract content from inline link: [text](url "title")
    /// Returns: (parsed text nodes, url, optional title)
    fn extract_inline_link_content(
        &self,
        pair: Pair<Rule>,
    ) -> Result<(Vec<Node>, String, Option<String>), AstError> {
        let mut text_nodes = Vec::new();
        let mut url = String::new();
        let mut title: Option<String> = None;

        // inline_link = "[" ~ link_text ~ "]" ~ "(" ~ link_destination_with_title ~ ")"
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::link_text => {
                    // Parse link text recursively for inline formatting
                    let text_content = inner.as_str();
                    let span = helpers::create_span(&inner);
                    text_nodes = self.parse_inline_text_recursively(text_content, &span)?;
                }
                Rule::link_destination_with_title => {
                    // Extract destination and optional title
                    for component in inner.into_inner() {
                        match component.as_rule() {
                            Rule::link_destination => {
                                url = self.extract_link_destination(component)?;
                            }
                            Rule::link_title => {
                                title = Some(self.extract_link_title(component)?);
                            }
                            _ => {} // Ignore whitespace
                        }
                    }
                }
                _ => {} // Ignore delimiters
            }
        }

        Ok((text_nodes, url, title))
    }

    /// Extract URL from link_destination (strips <> if present)
    fn extract_link_destination(&self, pair: Pair<Rule>) -> Result<String, AstError> {
        let inner = pair.into_inner().next();
        
        // Handle empty destination: [link]() is valid
        let inner = match inner {
            Some(i) => i,
            None => return Ok(String::new()),
        };

        let url = match inner.as_rule() {
            Rule::link_angle_bracket_destination => {
                // Strip compound-atomic delimiters: <url>
                let content = inner.as_str();
                let url = content.trim_start_matches('<').trim_end_matches('>');
                // Process escapes and encode spaces
                self.process_url_escapes_and_encoding(url)
            }
            Rule::link_plain_destination => {
                // Plain destination - process escapes only (no encoding)
                let url = inner.as_str();
                self.process_url_escapes(url)
            }
            _ => {
                return Err(AstError::InvalidStructure(format!(
                    "Unexpected link destination type: {:?}",
                    inner.as_rule()
                )));
            }
        };

        Ok(url)
    }
    
    /// Process backslash escapes in URLs
    /// \( -> (, \) -> ), \\ -> \, etc.
    fn process_url_escapes(&self, url: &str) -> String {
        let mut result = String::with_capacity(url.len());
        let mut chars = url.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                // Check if next char is escapable
                if let Some(&next_ch) = chars.peek() {
                    // CommonMark: backslash escapes ASCII punctuation
                    if next_ch.is_ascii_punctuation() {
                        chars.next(); // consume the escaped char
                        result.push(next_ch);
                        continue;
                    }
                }
            }
            result.push(ch);
        }
        
        result
    }
    
    /// Process escapes and encode spaces for angle-bracket URLs
    /// Spaces become %20, plus escape processing
    fn process_url_escapes_and_encoding(&self, url: &str) -> String {
        let mut result = String::with_capacity(url.len() * 2);
        let mut chars = url.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                // Check if next char is escapable
                if let Some(&next_ch) = chars.peek() {
                    if next_ch.is_ascii_punctuation() {
                        chars.next();
                        result.push(next_ch);
                        continue;
                    }
                }
                result.push(ch);
            } else if ch == ' ' {
                // Encode spaces as %20 in angle-bracket URLs
                result.push_str("%20");
            } else {
                result.push(ch);
            }
        }
        
        result
    }

    /// Extract title from link_title (strips quotes/parens)
    fn extract_link_title(&self, pair: Pair<Rule>) -> Result<String, AstError> {
        let inner = pair.into_inner().next().ok_or_else(|| {
            AstError::MissingContent("Empty link title".to_string())
        })?;

        // All title types are compound-atomic (${}), so we strip the delimiters
        let content = inner.as_str();
        let title = match inner.as_rule() {
            Rule::link_double_quoted_title => {
                content.trim_start_matches('"').trim_end_matches('"')
            }
            Rule::link_single_quoted_title => {
                content.trim_start_matches('\'').trim_end_matches('\'')
            }
            Rule::link_paren_quoted_title => {
                content.trim_start_matches('(').trim_end_matches(')')
            }
            _ => {
                return Err(AstError::InvalidStructure(format!(
                    "Unexpected link title type: {:?}",
                    inner.as_rule()
                )));
            }
        };

        Ok(title.to_string())
    }

    /// Extract full reference link: [text][label]
    /// Returns error wrapped in Ok - will be converted to ReferenceLink node by caller
    fn extract_full_reference_link(
        &self,
        pair: Pair<Rule>,
    ) -> Result<(Vec<Node>, String, Option<String>), AstError> {
        let mut text_nodes = Vec::new();
        let mut label = String::new();
        let span = helpers::create_span(&pair);

        // link_full_reference = "[" ~ link_text ~ "]" ~ "[" ~ link_reference_label ~ "]"
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::link_text => {
                    let text_content = inner.as_str();
                    text_nodes = self.parse_inline_text_recursively(text_content, &span)?;
                }
                Rule::link_reference_label => {
                    label = inner.as_str().to_string();
                }
                _ => {}
            }
        }

        // Return special marker to indicate this is a reference that needs resolution
        // The caller (build_inline_node) will create a ReferenceLink node
        Ok((text_nodes, format!("[REF:{}]", label), None))
    }

    /// Extract collapsed reference link: [label][]
    /// Uses the label as both text and reference label
    fn extract_collapsed_reference_link(
        &self,
        pair: Pair<Rule>,
    ) -> Result<(Vec<Node>, String, Option<String>), AstError> {
        let span = helpers::create_span(&pair);
        let mut label = String::new();

        // link_collapsed_reference = "[" ~ link_reference_label ~ "]" ~ "[" ~ "]"
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::link_reference_label {
                label = inner.as_str().to_string();
                break;
            }
        }

        // Text is the same as label for collapsed references
        let text_nodes = vec![Node::text(label.clone(), span)];
        
        Ok((text_nodes, format!("[REF:{}]", label), None))
    }

    /// Extract shortcut reference link: [label]
    /// Uses the label as both text and reference label
    fn extract_shortcut_reference_link(
        &self,
        pair: Pair<Rule>,
    ) -> Result<(Vec<Node>, String, Option<String>), AstError> {
        let span = helpers::create_span(&pair);
        let mut label = String::new();

        // link_shortcut_reference = "[" ~ link_reference_label ~ "]" ~ !("(" | "[")
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::link_reference_label {
                label = inner.as_str().to_string();
                break;
            }
        }

        // Text is the same as label for shortcut references
        let text_nodes = vec![Node::text(label.clone(), span)];
        
        Ok((text_nodes, format!("[REF:{}]", label), None))
    }

    /// Extract image content (alt text as String, URL, optional title)
    fn extract_image_content(
        &self,
        pair: Pair<Rule>,
    ) -> Result<(String, String, Option<String>), AstError> {
        let rule = pair.as_rule();

        match rule {
            Rule::image => {
                // Main dispatcher - extract the actual image type from inner pair
                let inner = pair.into_inner().next().ok_or_else(|| {
                    AstError::InvalidStructure("Empty image rule".to_string())
                })?;
                self.extract_image_content(inner)
            }
            Rule::inline_image => self.extract_inline_image_content(pair),
            Rule::image_full_reference => self.extract_full_reference_image(pair),
            Rule::image_collapsed_reference => self.extract_collapsed_reference_image(pair),
            Rule::image_shortcut_reference => self.extract_shortcut_reference_image(pair),
            _ => {
                Err(AstError::InvalidStructure(format!(
                    "Unexpected image type rule: {:?}",
                    rule
                )))
            }
        }
    }

    /// Extract content from inline image: ![alt](url "title")
    /// Returns: (alt text as String, url, optional title)
    fn extract_inline_image_content(
        &self,
        pair: Pair<Rule>,
    ) -> Result<(String, String, Option<String>), AstError> {
        let mut alt_text = String::new();
        let mut url = String::new();
        let mut title: Option<String> = None;

        // inline_image = "![" ~ image_alt_text ~ "]" ~ "(" ~ image_destination_with_title ~ ")"
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::image_alt_text => {
                    // Alt text is plain text, not recursively parsed
                    alt_text = inner.as_str().to_string();
                }
                Rule::image_destination_with_title => {
                    // Extract destination and optional title
                    for component in inner.into_inner() {
                        match component.as_rule() {
                            Rule::image_destination => {
                                url = self.extract_image_destination(component)?;
                            }
                            Rule::image_title => {
                                title = Some(self.extract_image_title(component)?);
                            }
                            _ => {} // Ignore whitespace
                        }
                    }
                }
                _ => {} // Ignore delimiters
            }
        }

        Ok((alt_text, url, title))
    }

    /// Extract URL from image_destination (strips <> if present)
    fn extract_image_destination(&self, pair: Pair<Rule>) -> Result<String, AstError> {
        let inner = pair.into_inner().next();
        
        // Handle empty destination: ![alt]() is valid
        let inner = match inner {
            Some(i) => i,
            None => return Ok(String::new()),
        };

        let url = match inner.as_rule() {
            Rule::image_angle_bracket_dest => {
                // Strip compound-atomic delimiters: <url>
                let content = inner.as_str();
                let url = content.trim_start_matches('<').trim_end_matches('>');
                // Process escapes and encode spaces (reuse link processing)
                self.process_url_escapes_and_encoding(url)
            }
            Rule::image_plain_dest => {
                // Plain destination - process escapes only (no encoding)
                let url = inner.as_str();
                self.process_url_escapes(url)
            }
            _ => {
                return Err(AstError::InvalidStructure(format!(
                    "Unexpected image destination type: {:?}",
                    inner.as_rule()
                )));
            }
        };

        Ok(url)
    }

    /// Extract title from image_title (strips quotes/parens)
    fn extract_image_title(&self, pair: Pair<Rule>) -> Result<String, AstError> {
        let inner = pair.into_inner().next().ok_or_else(|| {
            AstError::MissingContent("Empty image title".to_string())
        })?;

        // All title types are compound-atomic (${}), so we strip the delimiters
        let content = inner.as_str();
        let title = match inner.as_rule() {
            Rule::image_double_quoted_title => {
                content.trim_start_matches('"').trim_end_matches('"')
            }
            Rule::image_single_quoted_title => {
                content.trim_start_matches('\'').trim_end_matches('\'')
            }
            Rule::image_paren_quoted_title => {
                content.trim_start_matches('(').trim_end_matches(')')
            }
            _ => {
                return Err(AstError::InvalidStructure(format!(
                    "Unexpected image title type: {:?}",
                    inner.as_rule()
                )));
            }
        };

        Ok(title.to_string())
    }

    /// Extract full reference image: ![alt][label]
    fn extract_full_reference_image(
        &self,
        pair: Pair<Rule>,
    ) -> Result<(String, String, Option<String>), AstError> {
        let mut alt_text = String::new();
        let mut label = String::new();

        // image_full_reference = "![" ~ image_alt_text ~ "]" ~ "[" ~ image_reference_label ~ "]"
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::image_alt_text => {
                    alt_text = inner.as_str().to_string();
                }
                Rule::image_reference_label => {
                    label = inner.as_str().to_string();
                }
                _ => {}
            }
        }

        // Return special marker to indicate this is a reference that needs resolution
        Ok((alt_text, format!("[REF:{}]", label), None))
    }

    /// Extract collapsed reference image: ![label][]
    fn extract_collapsed_reference_image(
        &self,
        pair: Pair<Rule>,
    ) -> Result<(String, String, Option<String>), AstError> {
        let mut label = String::new();

        // image_collapsed_reference = "![" ~ image_reference_label ~ "]" ~ "[" ~ "]"
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::image_reference_label {
                label = inner.as_str().to_string();
                break;
            }
        }

        // Alt text is the same as label for collapsed references
        let alt_text = label.clone();
        
        Ok((alt_text, format!("[REF:{}]", label), None))
    }

    /// Extract shortcut reference image: ![label]
    fn extract_shortcut_reference_image(
        &self,
        pair: Pair<Rule>,
    ) -> Result<(String, String, Option<String>), AstError> {
        let mut label = String::new();

        // image_shortcut_reference = "![" ~ image_reference_label ~ "]" ~ !("(" | "[")
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::image_reference_label {
                label = inner.as_str().to_string();
                break;
            }
        }

        // Alt text is the same as label for shortcut references
        let alt_text = label.clone();
        
        Ok((alt_text, format!("[REF:{}]", label), None))
    }
}

impl Default for InlineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::engine::parsers::inline_parser::{InlineParser, Rule};
    use pest::Parser;
    
    // ========================================
    // Phase 4: Entity Reference Tests
    // ========================================
    
    #[test]
    fn smoke_test_entity_named_basic() {
        let input = "&nbsp;";
        let mut pairs = InlineParser::parse(Rule::entity_reference, input).unwrap();
        let pair = pairs.next().unwrap();
        
        let builder = InlineBuilder::new();
        let node = builder.build_inline_node(pair).unwrap();
        
        match node {
            Node::Text { content, .. } => {
                assert_eq!(content, "\u{00A0}"); // non-breaking space
            }
            _ => panic!("Expected Text node, got: {:?}", node),
        }
    }
    
    #[test]
    fn smoke_test_entity_named_common() {
        let tests = vec![
            ("&amp;", "&"),
            ("&lt;", "<"),
            ("&gt;", ">"),
            ("&quot;", "\""),
            ("&copy;", "©"),
        ];
        
        let builder = InlineBuilder::new();
        
        for (input, expected) in tests {
            let mut pairs = InlineParser::parse(Rule::entity_reference, input).unwrap();
            let pair = pairs.next().unwrap();
            let node = builder.build_inline_node(pair).unwrap();
            
            match node {
                Node::Text { content, .. } => {
                    assert_eq!(content, expected, "Failed for input: {}", input);
                }
                _ => panic!("Expected Text node for {}, got: {:?}", input, node),
            }
        }
    }
    
    #[test]
    fn smoke_test_entity_decimal() {
        let input = "&#35;";
        let mut pairs = InlineParser::parse(Rule::entity_reference, input).unwrap();
        let pair = pairs.next().unwrap();
        
        let builder = InlineBuilder::new();
        let node = builder.build_inline_node(pair).unwrap();
        
        match node {
            Node::Text { content, .. } => {
                assert_eq!(content, "#");
            }
            _ => panic!("Expected Text node, got: {:?}", node),
        }
    }
    
    #[test]
    fn smoke_test_entity_hex() {
        let input = "&#x23;";
        let mut pairs = InlineParser::parse(Rule::entity_reference, input).unwrap();
        let pair = pairs.next().unwrap();
        
        let builder = InlineBuilder::new();
        let node = builder.build_inline_node(pair).unwrap();
        
        match node {
            Node::Text { content, .. } => {
                assert_eq!(content, "#");
            }
            _ => panic!("Expected Text node, got: {:?}", node),
        }
    }
    
    #[test]
    fn smoke_test_entity_invalid_renders_literally() {
        let input = "&invalidname;";
        let mut pairs = InlineParser::parse(Rule::entity_reference, input).unwrap();
        let pair = pairs.next().unwrap();
        
        let builder = InlineBuilder::new();
        let node = builder.build_inline_node(pair).unwrap();
        
        match node {
            Node::Text { content, .. } => {
                // Invalid entities render literally
                assert_eq!(content, "&invalidname;");
            }
            _ => panic!("Expected Text node, got: {:?}", node),
        }
    }
    
    #[test]
    fn smoke_test_entity_emoji() {
        let tests = vec![
            ("&#128640;", "🚀"),  // rocket (decimal)
            ("&#x1F4A9;", "💩"),   // poop (hex)
        ];
        
        let builder = InlineBuilder::new();
        
        for (input, expected) in tests {
            let mut pairs = InlineParser::parse(Rule::entity_reference, input).unwrap();
            let pair = pairs.next().unwrap();
            let node = builder.build_inline_node(pair).unwrap();
            
            match node {
                Node::Text { content, .. } => {
                    assert_eq!(content, expected, "Failed for input: {}", input);
                }
                _ => panic!("Expected Text node for {}, got: {:?}", input, node),
            }
        }
    }
}
