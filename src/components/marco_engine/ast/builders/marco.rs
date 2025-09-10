use super::{AstBuilder, BuilderHelpers, ErrorHandling, ParseContext};
use crate::components::marco_engine::{
    ast::{Node, Span},
    errors::MarcoResult,
    grammar::Rule,
};
use pest::iterators::Pair;

// Constants for Marco-specific syntax
const TASK_CHECKED_MARKERS: [char; 2] = ['x', 'X'];
const DEFAULT_PAGE_FORMAT: &str = "A4";

/// Trait for building Marco-specific extension AST nodes
pub trait MarcoBuilder: BuilderHelpers + ErrorHandling {
    /// Build task item nodes
    fn build_task_item(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        let mut content = Vec::new();
        let mut checked = false;
        let task_text = pair.as_str(); // Store before consuming

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::task_marker => {
                    let task_str = inner_pair.as_str();
                    checked = TASK_CHECKED_MARKERS.iter().any(|&c| task_str.contains(c));
                }
                _ => {
                    // Process content with enhanced error recovery
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
                                    log::debug!("Skipped problematic task item content");
                                }
                                Err(critical_error) => {
                                    // For task items, we don't want to fail completely, so log and continue
                                    log::warn!(
                                        "Critical error in task item content, using fallback: {}",
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
        // Fallback: parse from entire task item text
        if content.is_empty() {
            if let Some((_parsed_marker, parsed_checked, parsed_content)) =
                Self::parse_task_syntax(task_text)
            {
                if !parsed_checked.is_empty() {
                    checked = TASK_CHECKED_MARKERS
                        .iter()
                        .any(|&c| parsed_checked.contains(c));
                }
                if content.is_empty() {
                    content.push(Self::create_text_node(parsed_content, span.clone()));
                }
            }
        }

        Ok(Node::task_item(checked, content, span))
    }

    /// Build user mention nodes
    fn build_user_mention(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        let mut username = String::new();
        let mut platform = None;
        let mut display_name = None;
        let mention_text = pair.as_str(); // Store before consuming

        for inner_pair in pair.into_inner() {
            let inner_text = inner_pair.as_str(); // Store before consuming
            match inner_pair.as_rule() {
                Rule::username => {
                    // Username rule doesn't include the @, so don't trim it
                    username = inner_text.to_string();
                }
                Rule::platform => {
                    platform = Some(inner_text.to_string());
                }
                Rule::display_name => {
                    display_name = Some(inner_text.to_string());
                }
                _ => {}
            }
        }

        // Fallback parsing: only use if no components were extracted
        if username.is_empty() && platform.is_none() && display_name.is_none() {
            if let Some(parsed) = Self::parse_user_mention_syntax(mention_text) {
                username = parsed.0;
                platform = parsed.1;
                display_name = parsed.2;
            }
        }

        // For now, always create basic variant since grammar doesn't support metadata fields
        // TODO: Add enhanced parsing when grammar supports user_id and avatar_url
        Ok(Node::user_mention(username, platform, display_name, span))
    }

    /// Build bookmark nodes
    fn build_bookmark(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        let mut name = String::new();
        let mut path = String::new();
        let mut line = None;
        let bookmark_text = pair.as_str(); // Store before consuming

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::KW_BOOKMARK => {
                    // This is just the "bookmark" keyword, skip it
                }
                Rule::local_path => {
                    let path_text = inner_pair.as_str();
                    // Check if the path contains a line number (=42)
                    if let Some(equals_pos) = path_text.find('=') {
                        // Split path and line number
                        path = path_text[..equals_pos].to_string();
                        let line_str = &path_text[equals_pos + 1..];
                        line = line_str.parse::<u32>().ok();
                    } else {
                        path = path_text.to_string();
                    }
                }
                _ => {
                    // This should be the bookmark name part (!"]" ~ ANY)+
                    // which captures everything between ":" and "]"
                    let inner_text = inner_pair.as_str();
                    if name.is_empty() {
                        name = inner_text.to_string();
                    }
                }
            }
        }

        // Fallback: parse from entire bookmark text if needed
        if name.is_empty() || path.is_empty() {
            if let Some(parsed) = Self::parse_bookmark_syntax(bookmark_text) {
                if name.is_empty() {
                    name = parsed.0;
                }
                if path.is_empty() {
                    path = parsed.1;
                }
                if line.is_none() {
                    line = parsed.2;
                }
            }
        }

        Ok(Node::bookmark(name, path, line, span))
    }

    /// Build page tag nodes
    fn build_page_tag(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        let mut format = None;
        let page_text = pair.as_str(); // Store before consuming

        for inner_pair in pair.into_inner() {
            let inner_text = inner_pair.as_str(); // Store before consuming
            if inner_pair.as_rule() == Rule::page_format {
                format = Some(inner_text.to_string());
            }
        }

        // Fallback parsing
        if format.is_none() {
            if let Some(parsed) = Self::parse_page_tag_syntax(page_text) {
                format = Some(parsed);
            }
        }

        Ok(Node::page_tag(format, span))
    }

    /// Build document reference nodes
    fn build_document_reference(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        let mut path = String::new();
        let doc_text = pair.as_str(); // Store before consuming

        for inner_pair in pair.into_inner() {
            if inner_pair.as_rule() == Rule::local_path {
                path = inner_pair.as_str().to_string();
            }
        }

        // Fallback: parse from entire doc ref text
        if path.is_empty() {
            if let Some(parsed_path) = Self::parse_doc_ref_syntax(doc_text) {
                path = parsed_path;
            }
        }

        Ok(Node::document_reference(path, span))
    }

    /// Build table of contents nodes
    fn build_table_of_contents(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        let mut depth = None;
        let mut document_path = None;
        let toc_text = pair.as_str(); // Store before consuming

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::toc_depth => {
                    let depth_str = inner_pair.as_str().trim_start_matches('=');
                    depth = depth_str.parse::<u8>().ok();
                }
                Rule::toc_doc => {
                    // Extract document path from toc_doc
                    let doc_text = inner_pair.as_str();
                    document_path = Self::parse_toc_doc_syntax(doc_text);
                }
                _ => {}
            }
        }

        // Fallback: parse from entire TOC text
        if depth.is_none() || document_path.is_none() {
            let parsed = Self::parse_toc_syntax(toc_text);
            if depth.is_none() {
                depth = parsed.0;
            }
            if document_path.is_none() {
                document_path = parsed.1;
            }
        }

        Ok(Node::table_of_contents(depth, document_path, span))
    }

    /// Build inline run command nodes
    fn build_run_inline(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        let mut script_type = String::new();
        let mut command = String::new();
        let run_text = pair.as_str(); // Store before consuming

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::script_type => {
                    script_type = inner_pair.as_str().to_string();
                }
                _ => {
                    // Extract command from the run syntax
                    let inner_text = inner_pair.as_str();
                    if let Some(parsed) = Self::parse_run_inline_syntax(inner_text) {
                        if script_type.is_empty() {
                            script_type = parsed.0;
                        }
                        command = parsed.1;
                    }
                }
            }
        }

        // Fallback: parse from entire run text
        if script_type.is_empty() || command.is_empty() {
            if let Some(parsed) = Self::parse_run_inline_syntax(run_text) {
                if script_type.is_empty() {
                    script_type = parsed.0;
                }
                if command.is_empty() {
                    command = parsed.1;
                }
            }
        }

        Ok(Node::run_inline(script_type, command, span))
    }

    /// Build block run command nodes
    fn build_run_block(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        let mut script_type = String::new();
        let mut command = String::new();
        let run_text = pair.as_str(); // Store before consuming

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::script_type => {
                    script_type = inner_pair.as_str().to_string();
                }
                _ => {
                    command.push_str(inner_pair.as_str());
                }
            }
        }

        // Fallback: parse from entire run block text
        if script_type.is_empty() || command.is_empty() {
            if let Some(parsed) = Self::parse_run_block_syntax(run_text) {
                if script_type.is_empty() {
                    script_type = parsed.0;
                }
                if command.is_empty() {
                    command = parsed.1;
                }
            }
        }

        Ok(Node::run_block(script_type, command, span))
    }

    /// Build diagram block nodes
    fn build_diagram_block(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        let mut diagram_type = String::new();
        let mut content = String::new();
        let diagram_text = pair.as_str(); // Store before consuming

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::diagram_type => {
                    diagram_type = inner_pair.as_str().to_string();
                }
                _ => {
                    content.push_str(inner_pair.as_str());
                }
            }
        }

        // Fallback: parse from entire diagram text
        if diagram_type.is_empty() || content.is_empty() {
            if let Some(parsed) = Self::parse_diagram_syntax(diagram_text) {
                if diagram_type.is_empty() {
                    diagram_type = parsed.0;
                }
                if content.is_empty() {
                    content = parsed.1;
                }
            }
        }

        Ok(Node::diagram_block(diagram_type, content, span))
    }

    /// Build tab block nodes
    fn build_tab_block(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        let mut title = None;
        let mut tabs = Vec::new();

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::tab_header => {
                    // Extract title from tab header
                    title = Self::parse_tab_header_title(inner_pair);
                }
                Rule::tab => {
                    if let Ok(tab) = Self::build_tab(inner_pair) {
                        tabs.push(tab);
                    }
                    // Ignore parsing errors for tabs
                }
                _ => {}
            }
        }

        Ok(Node::tab_block(title, tabs, span))
    }

    /// Build individual tab nodes
    fn build_tab(pair: Pair<Rule>) -> MarcoResult<Node> {
        let span = Self::create_span(&pair);
        let mut name = None;
        let mut content = Vec::new();
        let mut _icon: Option<String> = None;
        let mut _active = false;

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::tab_line => {
                    name = Self::parse_tab_line_name(inner_pair);
                    // TODO: Extract icon and active status from tab line when grammar supports it
                }
                Rule::tab_content_II => {
                    // Process tab content
                    for content_pair in inner_pair.into_inner() {
                        let content_text = content_pair.as_str();
                        let content_span = Self::create_span(&content_pair);
                        match AstBuilder::build_node(content_pair) {
                            Ok(node) => content.push(node),
                            Err(_) => {
                                content.push(Self::create_text_node(content_text, content_span));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // For now, always create basic variant since grammar doesn't support icon/active fields
        // TODO: Create enhanced variant when grammar supports icon and active status
        Ok(Node::tab(name, content, span))
    }

    // Helper parsing methods

    /// Parse task syntax: [x] text
    fn parse_task_syntax(input: &str) -> Option<(String, String, String)> {
        // Stricter validation: check minimum length and proper structure
        if input.len() < 3 || !input.starts_with('[') {
            return None;
        }

        if let Some(marker_end) = input.find(']') {
            // Validate marker position and content
            if marker_end == 1 || marker_end > 3 {
                return None; // Empty marker or too long
            }

            let marker = input[..=marker_end].to_string();
            let checked_char = input[1..marker_end].trim();

            // Validate checked character is valid (space, x, or X)
            if checked_char.len() != 1
                || !matches!(checked_char.chars().next(), Some(' ' | 'x' | 'X'))
            {
                return None;
            }

            let content = input[marker_end + 1..].trim();

            // Validate content is not empty and doesn't start with invalid characters
            if content.is_empty() || content.starts_with('[') || content.starts_with(']') {
                return None;
            }

            // Check for malformed nested syntax
            if content.chars().filter(|&c| c == '[').count()
                != content.chars().filter(|&c| c == ']').count()
            {
                return None;
            }

            Some((marker, checked_char.to_string(), content.to_string()))
        } else {
            None
        }
    }

    /// Parse user mention syntax: @username [platform](display)
    fn parse_user_mention_syntax(input: &str) -> Option<(String, Option<String>, Option<String>)> {
        // Stricter validation: check minimum length and proper structure
        if input.len() < 2 || !input.starts_with('@') {
            return None;
        }

        let rest = &input[1..];

        // Validate username doesn't contain invalid characters
        if rest.is_empty() || rest.starts_with(' ') || rest.contains('\n') || rest.contains('\t') {
            return None;
        }

        let parts: Vec<&str> = rest.split_whitespace().collect();
        let username_part = parts.first()?;

        // Validate username using centralized validation
        if Self::validate_username(username_part).is_err() {
            return None;
        }

        let username = username_part.to_string();

        // Look for platform and display name
        let remaining = rest[username.len()..].trim();

        // Handle different cases with early returns
        if remaining.is_empty() {
            return Some((username, None, None));
        }

        if !remaining.starts_with('[') {
            return None; // Invalid trailing content
        }

        // Parse platform and optional display syntax
        Self::parse_platform_and_display(&username, remaining)
    }

    /// Helper function to parse platform and display components
    fn parse_platform_and_display(
        username: &str,
        remaining: &str,
    ) -> Option<(String, Option<String>, Option<String>)> {
        // Validate bracket matching for platform syntax
        if !remaining.contains(']') {
            return None;
        }

        let platform_end = remaining.find(']')?;
        if platform_end == 1 {
            return None; // Empty platform
        }

        let platform = remaining[1..platform_end].trim();
        // Validate platform using centralized validation
        if Self::validate_platform(platform).is_err() {
            return None;
        }

        let after_platform = remaining[platform_end + 1..].trim();

        // Check for display name syntax
        if after_platform.starts_with('(') {
            Self::parse_display_name(username, platform, after_platform)
        } else {
            Some((username.to_string(), Some(platform.to_string()), None))
        }
    }

    /// Helper function to parse display name component
    fn parse_display_name(
        username: &str,
        platform: &str,
        after_platform: &str,
    ) -> Option<(String, Option<String>, Option<String>)> {
        let display_end = after_platform.find(')')?;
        if display_end == 1 {
            return None; // Empty display name
        }

        let display_name = after_platform[1..display_end].trim();
        // Validate display name using centralized validation
        if Self::validate_display_name(display_name).is_err() {
            return None;
        }

        Some((
            username.to_string(),
            Some(platform.to_string()),
            Some(display_name.to_string()),
        ))
    }

    /// Parse bookmark syntax: [bookmark:name](path=line)
    fn parse_bookmark_syntax(input: &str) -> Option<(String, String, Option<u32>)> {
        // Stricter validation: check minimum length and proper structure
        if input.len() < 13 || !input.starts_with("[bookmark:") {
            return None;
        }

        let name_end = input.find("](")?;
        if name_end <= 10 {
            return None; // Empty bookmark name
        }

        let name = input[10..name_end].trim();
        // Validate bookmark name doesn't contain invalid characters
        if name.is_empty()
            || name.contains('[')
            || name.contains(']')
            || name.contains('\n')
            || name.contains('\t')
        {
            return None;
        }

        let rest = &input[name_end + 2..];
        let path_end = rest.find(')')?;
        let path_part = &rest[..path_end];

        // Validate path_part is not empty
        if path_part.is_empty() {
            return None;
        }

        Self::parse_bookmark_path_and_line(name, path_part)
    }

    /// Helper function to parse bookmark path and optional line number
    fn parse_bookmark_path_and_line(
        name: &str,
        path_part: &str,
    ) -> Option<(String, String, Option<u32>)> {
        if let Some(line_start) = path_part.find('=') {
            Self::parse_bookmark_with_line(name, path_part, line_start)
        } else {
            Self::parse_bookmark_without_line(name, path_part)
        }
    }

    /// Helper function to parse bookmark with line number
    fn parse_bookmark_with_line(
        name: &str,
        path_part: &str,
        line_start: usize,
    ) -> Option<(String, String, Option<u32>)> {
        if line_start == 0 {
            return None; // Empty path before '='
        }

        let path = path_part[..line_start].trim();
        // Use enhanced path validation
        // Validate path using centralized validation
        if let Err(e) = Self::validate_path(path) {
            log::warn!("Bookmark path validation failed: {}", e);
            return None;
        }

        let line_str = path_part[line_start + 1..].trim();
        if line_str.is_empty() {
            return None; // Empty line number
        }

        // Validate line number using centralized validation
        match Self::validate_line_number(line_str) {
            Ok(line) => Some((name.to_string(), path.to_string(), Some(line))),
            Err(_) => None, // Invalid line number
        }
    }

    /// Helper function to parse bookmark without line number
    fn parse_bookmark_without_line(
        name: &str,
        path_part: &str,
    ) -> Option<(String, String, Option<u32>)> {
        let path = path_part.trim();
        // Enhanced validation using centralized validation
        if Self::validate_path(path).is_ok() {
            Some((name.to_string(), path.to_string(), None))
        } else {
            None
        }
    }

    /// Parse page tag syntax: [page=format]
    fn parse_page_tag_syntax(input: &str) -> Option<String> {
        if input.starts_with("[page=") && input.ends_with(']') {
            Some(input[6..input.len() - 1].to_string())
        } else if input == "[page]" {
            Some(DEFAULT_PAGE_FORMAT.to_string()) // Default format
        } else {
            None
        }
    }

    /// Parse document reference syntax: [@doc](path)
    fn parse_doc_ref_syntax(input: &str) -> Option<String> {
        if let Some(path_start) = input.find("](") {
            if input.starts_with("[@doc]") {
                let rest = &input[path_start + 2..];
                rest.find(')').map(|path_end| rest[..path_end].to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Parse TOC syntax: [toc=depth](@doc)
    fn parse_toc_syntax(input: &str) -> (Option<u8>, Option<String>) {
        if !input.starts_with("[toc") {
            return (None, None);
        }

        let depth = Self::parse_toc_depth(input);
        let doc_path = Self::parse_toc_document_path(input);

        (depth, doc_path)
    }

    /// Helper function to parse ToC depth parameter
    fn parse_toc_depth(input: &str) -> Option<u8> {
        let depth_start = input.find('=')?;
        let remaining = &input[depth_start..];
        let depth_end = remaining.find(']')?;

        let depth_str = &input[depth_start + 1..depth_start + depth_end];
        depth_str.parse::<u8>().ok()
    }

    /// Helper function to parse ToC document path
    fn parse_toc_document_path(input: &str) -> Option<String> {
        if input.contains("(@doc)") {
            // Simple case - no path specified
            None
        } else if let Some(doc_start) = input.find("(@doc") {
            // Path might be specified
            Self::parse_toc_doc_syntax(&input[doc_start..])
        } else {
            None
        }
    }

    /// Parse TOC document syntax: (@doc)
    fn parse_toc_doc_syntax(input: &str) -> Option<String> {
        if input == "(@doc)" {
            None // Current document
        } else {
            // For now, return None as the grammar doesn't specify path syntax
            None
        }
    }

    /// Parse inline run syntax: run script_type(command)
    fn parse_run_inline_syntax(input: &str) -> Option<(String, String)> {
        let rest = input.strip_prefix("run ")?;
        if let Some(paren_start) = rest.find('(') {
            let script_type = rest[..paren_start].trim().to_string();
            let rest2 = &rest[paren_start + 1..];
            if let Some(paren_end) = rest2.rfind(')') {
                let command = rest2[..paren_end].to_string();
                Some((script_type, command))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Parse block run syntax: ```run script_type\ncommand\n```
    fn parse_run_block_syntax(input: &str) -> Option<(String, String)> {
        if input.starts_with("```run ") {
            let lines: Vec<&str> = input.lines().collect();
            if lines.len() > 1 {
                let first_line = lines[0];
                let script_type = first_line[7..].trim().to_string(); // Skip "```run "

                let content_lines = &lines[1..];
                let last_idx = if content_lines.last() == Some(&"```") {
                    content_lines.len() - 1
                } else {
                    content_lines.len()
                };

                let command = content_lines[..last_idx].join("\n");
                Some((script_type, command))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Parse diagram syntax: ```diagram_type\ncontent\n```
    fn parse_diagram_syntax(input: &str) -> Option<(String, String)> {
        if input.starts_with("```") {
            let lines: Vec<&str> = input.lines().collect();
            if lines.len() > 1 {
                let first_line = lines[0];
                let diagram_type = first_line[3..].trim().to_string(); // Skip "```"

                let content_lines = &lines[1..];
                let last_idx = if content_lines.last() == Some(&"```") {
                    content_lines.len() - 1
                } else {
                    content_lines.len()
                };

                let content = content_lines[..last_idx].join("\n");
                Some((diagram_type, content))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Parse tab header title
    fn parse_tab_header_title(pair: Pair<Rule>) -> Option<String> {
        for inner_pair in pair.into_inner() {
            if inner_pair.as_rule() == Rule::tab_title {
                return Some(inner_pair.as_str().trim().to_string());
            }
        }
        None
    }

    /// Parse tab line name
    fn parse_tab_line_name(pair: Pair<Rule>) -> Option<String> {
        for inner_pair in pair.into_inner() {
            if inner_pair.as_rule() == Rule::tab_name {
                return Some(inner_pair.as_str().trim().to_string());
            }
        }
        None
    }
}
