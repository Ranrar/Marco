use super::{AstBuilder, BuilderHelpers, ErrorHandling, MAX_TEXT_LENGTH};
use crate::components::marco_engine::{
    ast::{Node, Span},
    errors::MarcoResult,
    grammar::Rule,
};
use log::debug;
use pest::iterators::Pair;

/// Trait for building link, image, and reference AST nodes
pub trait LinkBuilder: BuilderHelpers + ErrorHandling {
    /// Build link nodes
    fn build_link(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("LinkBuilder::build_link - Processing link");
        let mut text = String::new();
        let mut url = String::new();
        let mut title = None;
        let link_str = pair.as_str(); // Keep as &str, only convert to String when needed

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::inline_link_text => {
                    text = inner_pair.as_str().to_string();
                }
                Rule::link_url => {
                    url = inner_pair.as_str().to_string();
                }
                Rule::link_title => {
                    let title_str = inner_pair.as_str();
                    title = Some(title_str.trim_matches('"').trim_matches('\'').to_string());
                }
                _ => {
                    debug!(
                        "LinkBuilder::build_link - Unexpected rule: {:?}",
                        inner_pair.as_rule()
                    );
                }
            }
        }

        // Fallback: parse from the full link syntax
        if text.is_empty() || url.is_empty() {
            if let Some(captures) = Self::parse_link_syntax(link_str) {
                if text.is_empty() {
                    text = captures.0;
                }
                if url.is_empty() {
                    url = captures.1;
                }
                if title.is_none() {
                    title = captures.2;
                }
            }
        }

        // Apply validation with graceful fallback instead of strict rejection
        if let Err(e) = Self::validate_url(&url) {
            log::warn!(
                "Link URL validation failed: {}, using original text as fallback",
                e
            );
            return Ok(Node::text(link_str.to_string(), span));
        }

        if let Some(ref title_text) = title {
            if let Err(e) = Self::validate_title(title_text) {
                log::warn!("Link title validation failed: {}, removing title", e);
                title = None;
            }
        }

        // Allow empty text but provide default if needed
        if text.is_empty() {
            text = url.clone(); // Use URL as link text if none provided
        }

        // Validate text length but allow it to pass with warning if too long
        if let Err(e) = Self::validate_text_content(&text) {
            log::warn!("Link text validation failed: {}, truncating", e);
            text.truncate(MAX_TEXT_LENGTH);
        }

        Ok(Node::link(
            vec![Self::create_text_node(text, span.clone())],
            url,
            title,
            span,
        ))
    }

    /// Build image nodes
    fn build_image(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("LinkBuilder::build_image - Processing image");
        let mut alt_text = String::new();
        let mut url = String::new();
        let mut title = None;
        let image_str = pair.as_str(); // Keep as &str, only convert to String when needed

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::inline_link_text => {
                    alt_text = inner_pair.as_str().to_string();
                }
                Rule::link_url => {
                    url = inner_pair.as_str().to_string();
                }
                Rule::link_title => {
                    let title_str = inner_pair.as_str();
                    title = Some(title_str.trim_matches('"').trim_matches('\'').to_string());
                }
                _ => {
                    debug!(
                        "LinkBuilder::build_image - Unexpected rule: {:?}",
                        inner_pair.as_rule()
                    );
                }
            }
        }

        // Fallback: parse from the full image syntax
        if alt_text.is_empty() || url.is_empty() {
            if let Some(captures) = Self::parse_image_syntax(image_str) {
                if alt_text.is_empty() {
                    alt_text = captures.0;
                }
                if url.is_empty() {
                    url = captures.1;
                }
                if title.is_none() {
                    title = captures.2;
                }
            }
        }

        // Apply validation with graceful fallback instead of strict rejection
        if let Err(e) = Self::validate_url(&url) {
            log::warn!(
                "Image URL validation failed: {}, using original text as fallback",
                e
            );
            return Ok(Node::text(image_str.to_string(), span));
        }

        if let Err(e) = Self::validate_alt_text(&alt_text) {
            log::warn!("Image alt text validation failed: {}, truncating", e);
            alt_text.truncate(512); // Use constant from validation
        }

        if let Some(ref title_text) = title {
            if let Err(e) = Self::validate_title(title_text) {
                log::warn!("Image title validation failed: {}, removing title", e);
                title = None;
            }
        }

        // Allow empty alt text but provide default if needed
        if alt_text.is_empty() {
            alt_text = "Image".to_string(); // Default alt text for accessibility
        }

        Ok(Node::image(alt_text, url, title, span))
    }

    /// Build autolink nodes
    fn build_autolink(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("LinkBuilder::build_autolink - Processing autolink");
        let raw_url = pair.as_str();
        let url = raw_url.trim_matches('<').trim_matches('>').to_string();

        // Apply URL validation with fallback
        if let Err(e) = Self::validate_url(&url) {
            log::warn!("Autolink URL validation failed: {}, using text fallback", e);
            return Ok(Node::text(raw_url.to_string(), span));
        }

        Ok(Node::autolink(url, span))
    }

    /// Build reference link nodes
    fn build_reference_link(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("LinkBuilder::build_reference_link - Processing reference link");
        let mut text = String::new();
        let mut reference = String::new();
        let ref_str = pair.as_str().to_string(); // Store before consuming

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::block_caption => {
                    if text.is_empty() {
                        text = inner_pair.as_str().to_string();
                    } else {
                        reference = inner_pair.as_str().to_string();
                    }
                }
                _ => {
                    debug!(
                        "LinkBuilder::build_reference_link - Unexpected rule: {:?}",
                        inner_pair.as_rule()
                    );
                }
            }
        }
        // Fallback: parse from the full reference syntax
        if text.is_empty() || reference.is_empty() {
            if let Some(captures) = Self::parse_reference_syntax(&ref_str) {
                if text.is_empty() {
                    text = captures.0;
                }
                if reference.is_empty() {
                    reference = captures.1;
                }
            }
        }

        Ok(Node::reference_link(
            vec![Self::create_text_node(text, span.clone())],
            reference,
            span,
        ))
    }

    /// Build reference image nodes
    fn build_reference_image(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("LinkBuilder::build_reference_image - Processing reference image");
        let mut alt_text = String::new();
        let mut reference = String::new();
        let ref_str = pair.as_str().to_string(); // Store before consuming

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::block_caption => {
                    if alt_text.is_empty() {
                        alt_text = inner_pair.as_str().to_string();
                    } else {
                        reference = inner_pair.as_str().to_string();
                    }
                }
                _ => {
                    debug!(
                        "LinkBuilder::build_reference_image - Unexpected rule: {:?}",
                        inner_pair.as_rule()
                    );
                }
            }
        }

        // Fallback: parse from the full reference syntax
        if alt_text.is_empty() || reference.is_empty() {
            if let Some(captures) = Self::parse_reference_image_syntax(&ref_str) {
                if alt_text.is_empty() {
                    alt_text = captures.0;
                }
                if reference.is_empty() {
                    reference = captures.1;
                }
            }
        }

        Ok(Node::reference_image(alt_text, reference, span))
    }

    /// Build reference definition nodes
    fn build_reference_definition(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("LinkBuilder::build_reference_definition - Processing reference definition");
        let mut label = String::new();
        let mut url = String::new();
        let mut title = None;
        let def_str = pair.as_str().to_string(); // Store before consuming

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::block_caption => {
                    label = inner_pair.as_str().to_string();
                }
                Rule::inline_link => {
                    // Parse the inline link for URL
                    url = inner_pair.as_str().to_string();
                }
                Rule::ref_title => {
                    let title_str = inner_pair.as_str();
                    title = Some(title_str.trim_matches('"').trim_matches('\'').to_string());
                }
                _ => {
                    debug!(
                        "LinkBuilder::build_reference_definition - Unexpected rule: {:?}",
                        inner_pair.as_rule()
                    );
                }
            }
        }

        // Fallback: parse from the full definition syntax
        if label.is_empty() || url.is_empty() {
            if let Some(captures) = Self::parse_reference_definition_syntax(&def_str) {
                if label.is_empty() {
                    label = captures.0;
                }
                if url.is_empty() {
                    url = captures.1;
                }
                if title.is_none() {
                    title = captures.2;
                }
            }
        }

        // Apply validation with graceful fallback
        if let Err(e) = Self::validate_label(&label) {
            log::warn!(
                "Reference definition label validation failed: {}, using text fallback",
                e
            );
            return Ok(Node::text(def_str, span));
        }

        if let Err(e) = Self::validate_url(&url) {
            log::warn!(
                "Reference definition URL validation failed: {}, using text fallback",
                e
            );
            return Ok(Node::text(def_str, span));
        }

        if let Some(ref title_text) = title {
            if let Err(e) = Self::validate_title(title_text) {
                log::warn!(
                    "Reference definition title validation failed: {}, removing title",
                    e
                );
                title = None;
            }
        }

        Ok(Node::reference_definition(label, url, title, span))
    }

    /// Build footnote reference nodes
    fn build_footnote_ref(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("LinkBuilder::build_footnote_ref - Processing footnote reference");
        let mut label = String::new();
        let footnote_str = pair.as_str(); // Store before consuming

        for inner_pair in pair.into_inner() {
            let inner_text = inner_pair.as_str(); // Store before consuming
            match inner_pair.as_rule() {
                Rule::footnote_label => {
                    label = inner_text.to_string();
                }
                _ => {
                    debug!(
                        "LinkBuilder::build_footnote_ref - Unexpected rule: {:?}",
                        inner_pair.as_rule()
                    );
                }
            }
        }

        // Fallback: extract from the full footnote syntax
        if label.is_empty() {
            if let Some(captures) = Self::parse_footnote_ref_syntax(footnote_str) {
                label = captures;
            }
        }

        Ok(Node::footnote_ref(label, span))
    }

    /// Build inline footnote nodes
    fn build_inline_footnote(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("LinkBuilder::build_inline_footnote - Processing inline footnote");
        let content_str = pair.as_str();
        let content = content_str
            .trim_start_matches("^[")
            .trim_end_matches(']')
            .to_string();

        Ok(Node::inline_footnote(
            vec![Self::create_text_node(content, span.clone())],
            span,
        ))
    }

    /// Build footnote definition nodes
    fn build_footnote_definition(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("LinkBuilder::build_footnote_definition - Processing footnote definition");
        let mut label = String::new();
        let mut content = Vec::new();
        let def_str = pair.as_str(); // Store before consuming

        for inner_pair in pair.into_inner() {
            let inner_text = inner_pair.as_str(); // Store before consuming
            let inner_span = Self::create_span(&inner_pair);
            match inner_pair.as_rule() {
                Rule::footnote_label => {
                    label = inner_text.to_string();
                }
                _ => {
                    // Process content
                    match AstBuilder::build_node(inner_pair) {
                        Ok(node) => content.push(node),
                        Err(_) => {
                            let text = inner_text.trim();
                            if !text.is_empty() {
                                content.push(Self::create_text_node(text, inner_span));
                            }
                        }
                    }
                }
            }
        }

        // Fallback: parse from the full definition syntax
        if label.is_empty() || content.is_empty() {
            if let Some(captures) = Self::parse_footnote_definition_syntax(def_str) {
                if label.is_empty() {
                    label = captures.0;
                }
                if content.is_empty() {
                    content.push(Self::create_text_node(captures.1, span.clone()));
                }
            }
        }

        Ok(Node::footnote_definition(label, content, span))
    }

    // Helper methods for parsing various link/reference syntaxes

    /// Parse link syntax: [text](url "title")
    fn parse_link_syntax(input: &str) -> Option<(String, String, Option<String>)> {
        // Stricter validation: check minimum length and proper structure
        if input.len() < 4 || !input.starts_with('[') {
            return None;
        }

        if let Some(text_end) = input.find("](") {
            // Validate text content is not empty and doesn't contain invalid characters
            let text = input[1..text_end].trim();
            if text.is_empty() || text.contains('\n') {
                return None;
            }

            let rest = &input[text_end + 2..];
            if let Some(url_end) = rest.find(')') {
                let url_part = &rest[..url_end];

                // Validate that we don't have unmatched brackets in the URL part
                if url_part.contains('[') || url_part.contains(']') {
                    return None;
                }

                // Check for title with stricter validation
                if let Some(title_start) = url_part.find(' ') {
                    let url = url_part[..title_start].trim();
                    // Validate URL is not empty and doesn't contain invalid characters
                    if url.is_empty() || url.contains('\n') || url.contains('\t') {
                        return None;
                    }

                    let title_part = url_part[title_start..].trim();
                    // Validate title has proper quotes and isn't empty
                    if title_part.len() < 2
                        || (!title_part.starts_with('"') && !title_part.starts_with('\''))
                    {
                        return None;
                    }

                    let title = title_part.trim_matches('"').trim_matches('\'');
                    if title.is_empty() || title.contains('\n') {
                        return None;
                    }

                    Some((text.to_string(), url.to_string(), Some(title.to_string())))
                } else {
                    let url = url_part.trim();
                    // Validate URL is not empty and doesn't contain invalid characters
                    if url.is_empty() || url.contains('\n') || url.contains('\t') {
                        return None;
                    }
                    Some((text.to_string(), url.to_string(), None))
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Parse image syntax: ![alt](url "title")
    fn parse_image_syntax(input: &str) -> Option<(String, String, Option<String>)> {
        if input.starts_with("![") {
            Self::parse_link_syntax(&input[1..])
        } else {
            None
        }
    }

    /// Parse reference syntax: [text][ref]
    fn parse_reference_syntax(input: &str) -> Option<(String, String)> {
        // Stricter validation: check minimum length and proper structure
        if input.len() < 5 || !input.starts_with('[') {
            return None;
        }

        if let Some(text_end) = input.find("][") {
            if text_end == 1 {
                return None; // Empty text
            }

            let text = input[1..text_end].trim();
            // Validate text content doesn't contain invalid characters
            if text.is_empty() || text.contains('\n') || text.contains('[') || text.contains(']') {
                return None;
            }

            let rest = &input[text_end + 2..];

            if let Some(ref_end) = rest.find(']') {
                if ref_end == 0 {
                    return None; // Empty reference
                }

                let reference = rest[..ref_end].trim();
                // Validate reference doesn't contain invalid characters
                if reference.is_empty()
                    || reference.contains('\n')
                    || reference.contains('[')
                    || reference.contains(']')
                {
                    return None;
                }

                // Check that there's no trailing content after the reference
                if rest.len() > ref_end + 1 && !rest[ref_end + 1..].trim().is_empty() {
                    return None;
                }

                Some((text.to_string(), reference.to_string()))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Parse reference image syntax: ![alt][ref]
    fn parse_reference_image_syntax(input: &str) -> Option<(String, String)> {
        if input.starts_with("![") {
            Self::parse_reference_syntax(&input[1..])
        } else {
            None
        }
    }

    /// Parse reference definition syntax: [label]: url "title"
    fn parse_reference_definition_syntax(input: &str) -> Option<(String, String, Option<String>)> {
        if let Some(label_end) = input.find("]:") {
            let label = input[1..label_end].to_string();
            let rest = input[label_end + 2..].trim();

            // Split URL and title
            if let Some(title_start) = rest.find(' ') {
                let url = rest[..title_start].trim().to_string();
                let title_part = rest[title_start..].trim();
                let title = title_part.trim_matches('"').trim_matches('\'').to_string();
                Some((
                    label,
                    url,
                    if title.is_empty() { None } else { Some(title) },
                ))
            } else {
                let url = rest.to_string();
                Some((label, url, None))
            }
        } else {
            None
        }
    }

    /// Parse footnote reference syntax: [^label]
    fn parse_footnote_ref_syntax(input: &str) -> Option<String> {
        if input.starts_with("[^") && input.ends_with(']') {
            Some(input[2..input.len() - 1].to_string())
        } else {
            None
        }
    }

    /// Parse footnote definition syntax: [^label]: content
    fn parse_footnote_definition_syntax(input: &str) -> Option<(String, String)> {
        if let Some(label_end) = input.find("]:") {
            if input.starts_with("[^") {
                let label = input[2..label_end].to_string();
                let content = input[label_end + 2..].trim().to_string();
                Some((label, content))
            } else {
                None
            }
        } else {
            None
        }
    }
}
