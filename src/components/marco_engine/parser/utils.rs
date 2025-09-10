//! Parser utilities for Marco engine
//!
//! This module provides utilities for working with Pest parsing results,
//! rule management, and grammar analysis extracted from the debug utilities.

use crate::components::marco_engine::grammar::{MarcoParser, Rule};
use crate::components::marco_engine::parser::position::{SourceSpan, SpanExt};
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use std::collections::{HashMap, HashSet};
use std::fs;

/// Maps string rule names to their corresponding Rule enum variants
/// Extracted from debug/src/main.rs get_rule function
pub fn get_rule_by_name(rule_name: &str) -> Option<Rule> {
    match rule_name {
        // Main structure
        "file" => Some(Rule::file),
        "document" => Some(Rule::document),
        "section" => Some(Rule::section),
        "block" => Some(Rule::block),
        "paragraph" => Some(Rule::paragraph),
        "paragraph_line" => Some(Rule::paragraph_line),

        // Text and characters
        "text" => Some(Rule::text),
        "word" => Some(Rule::word),
        "safe_punct" => Some(Rule::safe_punct),
        "math_symbol" => Some(Rule::math_symbol),
        "unicode_letter" => Some(Rule::unicode_letter),
        "inner_char" => Some(Rule::inner_char),

        // Headings
        "heading" => Some(Rule::heading),
        "heading_content" => Some(Rule::heading_content),
        "H1" => Some(Rule::H1),
        "H2" => Some(Rule::H2),
        "H3" => Some(Rule::H3),
        "H4" => Some(Rule::H4),
        "H5" => Some(Rule::H5),
        "H6" => Some(Rule::H6),
        "setext_h1" => Some(Rule::setext_h1),
        "setext_h2" => Some(Rule::setext_h2),

        // Formatting
        "emphasis" => Some(Rule::emphasis),
        "bold" => Some(Rule::bold),
        "bold_asterisk" => Some(Rule::bold_asterisk),
        "bold_underscore" => Some(Rule::bold_underscore),
        "italic" => Some(Rule::italic),
        "italic_asterisk" => Some(Rule::italic_asterisk),
        "italic_underscore" => Some(Rule::italic_underscore),
        "bold_italic" => Some(Rule::bold_italic),
        "bold_italic_triple_asterisk" => Some(Rule::bold_italic_triple_asterisk),
        "bold_italic_triple_underscore" => Some(Rule::bold_italic_triple_underscore),
        "bold_italic_mixed_ast_under" => Some(Rule::bold_italic_mixed_ast_under),
        "bold_italic_mixed_under_ast" => Some(Rule::bold_italic_mixed_under_ast),
        "strikethrough" => Some(Rule::strikethrough),
        "strikethrough_tilde" => Some(Rule::strikethrough_tilde),
        "strikethrough_dash" => Some(Rule::strikethrough_dash),
        "highlight" => Some(Rule::highlight),
        "superscript" => Some(Rule::superscript),
        "subscript" => Some(Rule::subscript),
        "emoji" => Some(Rule::emoji),

        // Code and math
        "code_inline" => Some(Rule::code_inline),
        "code_block" => Some(Rule::code_block),
        "fenced_code" => Some(Rule::fenced_code),
        "indented_code" => Some(Rule::indented_code),
        "language_id" => Some(Rule::language_id),
        "math_inline" => Some(Rule::math_inline),
        "math_block" => Some(Rule::math_block),

        // Links and images
        "inline_link" => Some(Rule::inline_link),
        "bracket_link_with_title" => Some(Rule::bracket_link_with_title),
        "bracket_link_without_title" => Some(Rule::bracket_link_without_title),
        "autolink" => Some(Rule::autolink),
        "autolink_email" => Some(Rule::autolink_email),
        "autolink_url" => Some(Rule::autolink_url),
        "inline_image" => Some(Rule::inline_image),
        "inline_link_text" => Some(Rule::inline_link_text),
        "link_url" => Some(Rule::link_url),
        "link_title" => Some(Rule::link_title),
        "reference_link" => Some(Rule::reference_link),
        "reference_image" => Some(Rule::reference_image),
        "reference_definition" => Some(Rule::reference_definition),
        "block_image" => Some(Rule::block_image),
        "block_youtube" => Some(Rule::block_youtube),
        "block_caption" => Some(Rule::block_caption),
        "ref_title" => Some(Rule::ref_title),

        // URLs
        "http_url" => Some(Rule::http_url),
        "www_url" => Some(Rule::www_url),
        "mailto" => Some(Rule::mailto),
        "local_path" => Some(Rule::local_path),
        "youtube_url" => Some(Rule::youtube_url),
        "image_url" => Some(Rule::image_url),
        "image_ext" => Some(Rule::image_ext),

        // Lists
        "list" => Some(Rule::list),
        "list_item" => Some(Rule::list_item),
        "regular_list_item" => Some(Rule::regular_list_item),
        "task_list_item" => Some(Rule::task_list_item),
        "list_item_content" => Some(Rule::list_item_content),
        "list_marker" => Some(Rule::list_marker),
        "unordered_marker" => Some(Rule::unordered_marker),
        "ordered_marker" => Some(Rule::ordered_marker),
        "task_marker" => Some(Rule::task_marker),
        "task_metadata" => Some(Rule::task_metadata),
        "inline_task_item" => Some(Rule::inline_task_item),

        // Definition lists
        "def_list" => Some(Rule::def_list),
        "term_line" => Some(Rule::term_line),
        "def_line" => Some(Rule::def_line),

        // Tables
        "table" => Some(Rule::table),
        "table_header" => Some(Rule::table_header),
        "table_sep" => Some(Rule::table_sep),
        "table_row" => Some(Rule::table_row),
        "table_cell" => Some(Rule::table_cell),
        "table_sep_cell" => Some(Rule::table_sep_cell),

        // Blockquotes
        "blockquote" => Some(Rule::blockquote),
        "blockquote_line" => Some(Rule::blockquote_line),

        // Horizontal rules
        "hr" => Some(Rule::hr),

        // Footnotes
        "footnote_ref" => Some(Rule::footnote_ref),
        "footnote_def" => Some(Rule::footnote_def),
        "footnote_label" => Some(Rule::footnote_label),
        "inline_footnote_ref" => Some(Rule::inline_footnote_ref),

        // HTML and comments
        "inline_html" => Some(Rule::inline_html),
        "block_html" => Some(Rule::block_html),
        "inline_comment" => Some(Rule::inline_comment),
        "block_comment" => Some(Rule::block_comment),

        // Inline elements
        "inline" => Some(Rule::inline),
        "inline_core" => Some(Rule::inline_core),
        "escaped_char" => Some(Rule::escaped_char),
        "line_break" => Some(Rule::line_break),

        // Marco extensions
        "macro_inline" => Some(Rule::macro_inline),
        "macro_block" => Some(Rule::macro_block),
        "user_mention" => Some(Rule::user_mention),
        "username" => Some(Rule::username),
        "platform" => Some(Rule::platform),
        "display_name" => Some(Rule::display_name),

        // Admonitions
        "admonition_block" => Some(Rule::admonition_block),
        "admonition_type" => Some(Rule::admonition_type),
        "admonition_open" => Some(Rule::admonition_open),
        "admonition_emoji" => Some(Rule::admonition_emoji),
        "admonition_close" => Some(Rule::admonition_close),

        // Page and document
        "page_tag" => Some(Rule::page_tag),
        "page_format" => Some(Rule::page_format),
        "doc_ref" => Some(Rule::doc_ref),
        "bookmark" => Some(Rule::bookmark),

        // Table of contents
        "toc" => Some(Rule::toc),
        "toc_depth" => Some(Rule::toc_depth),
        "toc_doc" => Some(Rule::toc_doc),

        // Run commands
        "run_inline" => Some(Rule::run_inline),
        "run_block_fenced" => Some(Rule::run_block_fenced),
        "script_type" => Some(Rule::script_type),

        // Diagrams
        "diagram_fenced" => Some(Rule::diagram_fenced),
        "diagram_type" => Some(Rule::diagram_type),

        // Tabs
        "tab_block" => Some(Rule::tab_block),
        "tab_header" => Some(Rule::tab_header),
        "tab_title" => Some(Rule::tab_title),
        "tabs_content_I" => Some(Rule::tabs_content_I),
        "tab_content_line" => Some(Rule::tab_content_line),
        "tab" => Some(Rule::tab),
        "tab_line" => Some(Rule::tab_line),
        "tab_name" => Some(Rule::tab_name),
        "tab_content_II" => Some(Rule::tab_content_II),
        "tab_end" => Some(Rule::tab_end),

        // Keywords
        "KW_NOTE" => Some(Rule::KW_NOTE),
        "KW_TIP" => Some(Rule::KW_TIP),
        "KW_WARNING" => Some(Rule::KW_WARNING),
        "KW_DANGER" => Some(Rule::KW_DANGER),
        "KW_INFO" => Some(Rule::KW_INFO),
        "KW_BOOKMARK" => Some(Rule::KW_BOOKMARK),
        "KW_PAGE" => Some(Rule::KW_PAGE),
        "KW_DOC" => Some(Rule::KW_DOC),
        "KW_TOC" => Some(Rule::KW_TOC),
        "KW_TAB" => Some(Rule::KW_TAB),
        "KW_BASH" => Some(Rule::KW_BASH),
        "KW_ZSH" => Some(Rule::KW_ZSH),
        "KW_SH" => Some(Rule::KW_SH),
        "KW_BAT" => Some(Rule::KW_BAT),
        "KW_POWERSHELL" => Some(Rule::KW_POWERSHELL),
        "KW_PS" => Some(Rule::KW_PS),
        "KW_PYTHON" => Some(Rule::KW_PYTHON),
        "KW_PY" => Some(Rule::KW_PY),
        "KW_RUN" => Some(Rule::KW_RUN),
        "KW_MERMAID" => Some(Rule::KW_MERMAID),
        "KW_GRAPHVIZ" => Some(Rule::KW_GRAPHVIZ),

        // Error recovery
        "unknown_block" => Some(Rule::unknown_block),

        _ => None,
    }
}

/// Returns a list of all available rule names
pub fn get_all_rule_names() -> Vec<&'static str> {
    vec![
        // Main structure
        "file",
        "document",
        "section",
        "block",
        "paragraph",
        "paragraph_line",
        // Text and characters
        "text",
        "word",
        "safe_punct",
        "math_symbol",
        "unicode_letter",
        "inner_char",
        // Headings
        "heading",
        "heading_content",
        "H1",
        "H2",
        "H3",
        "H4",
        "H5",
        "H6",
        "setext_h1",
        "setext_h2",
        // Formatting
        "emphasis",
        "bold",
        "bold_asterisk",
        "bold_underscore",
        "italic",
        "italic_asterisk",
        "italic_underscore",
        "bold_italic",
        "bold_italic_triple_asterisk",
        "bold_italic_triple_underscore",
        "bold_italic_mixed_ast_under",
        "bold_italic_mixed_under_ast",
        "strikethrough",
        "strikethrough_tilde",
        "strikethrough_dash",
        "highlight",
        "superscript",
        "subscript",
        "emoji",
        // Code and math
        "code_inline",
        "code_block",
        "fenced_code",
        "indented_code",
        "language_id",
        "math_inline",
        "math_block",
        // Links and images
        "inline_link",
        "bracket_link_with_title",
        "bracket_link_without_title",
        "autolink",
        "autolink_email",
        "autolink_url",
        "inline_image",
        "inline_link_text",
        "link_url",
        "link_title",
        "reference_link",
        "reference_image",
        "reference_definition",
        "block_image",
        "block_youtube",
        "block_caption",
        "ref_title",
        // URLs
        "http_url",
        "www_url",
        "mailto",
        "local_path",
        "youtube_url",
        "image_url",
        "image_ext",
        // Lists
        "list",
        "list_item",
        "regular_list_item",
        "task_list_item",
        "list_item_content",
        "list_marker",
        "unordered_marker",
        "ordered_marker",
        "task_marker",
        "task_metadata",
        "inline_task_item",
        // Definition lists
        "def_list",
        "term_line",
        "def_line",
        // Tables
        "table",
        "table_header",
        "table_sep",
        "table_row",
        "table_cell",
        "table_sep_cell",
        // Blockquotes
        "blockquote",
        "blockquote_line",
        // Horizontal rules
        "hr",
        // Footnotes
        "footnote_ref",
        "footnote_def",
        "footnote_label",
        "inline_footnote_ref",
        // HTML and comments
        "inline_html",
        "block_html",
        "inline_comment",
        "block_comment",
        // Inline elements
        "inline",
        "inline_core",
        "escaped_char",
        "line_break",
        // Marco extensions
        "macro_inline",
        "macro_block",
        "user_mention",
        "username",
        "platform",
        "display_name",
        // Admonitions
        "admonition_block",
        "admonition_type",
        "admonition_open",
        "admonition_emoji",
        "admonition_close",
        // Page and document
        "page_tag",
        "page_format",
        "doc_ref",
        "bookmark",
        // Table of contents
        "toc",
        "toc_depth",
        "toc_doc",
        // Run commands
        "run_inline",
        "run_block_fenced",
        "script_type",
        // Diagrams
        "diagram_fenced",
        "diagram_type",
        // Tabs
        "tab_block",
        "tab_header",
        "tab_title",
        "tabs_content_I",
        "tab_content_line",
        "tab",
        "tab_line",
        "tab_name",
        "tab_content_II",
        "tab_end",
        // Keywords
        "KW_NOTE",
        "KW_TIP",
        "KW_WARNING",
        "KW_DANGER",
        "KW_INFO",
        "KW_BOOKMARK",
        "KW_PAGE",
        "KW_DOC",
        "KW_TOC",
        "KW_TAB",
        "KW_BASH",
        "KW_ZSH",
        "KW_SH",
        "KW_BAT",
        "KW_POWERSHELL",
        "KW_PS",
        "KW_PYTHON",
        "KW_PY",
        "KW_RUN",
        "KW_MERMAID",
        "KW_GRAPHVIZ",
        // Error recovery
        "unknown_block",
    ]
}

/// Categorizes a rule by its functional purpose
/// Extracted from debug/src/grammar_visualizer.rs
pub fn categorize_rule(rule_name: &str) -> &'static str {
    match rule_name {
        name if name.starts_with("H") || name.contains("heading") => "Headings",
        name if name.contains("bold") || name.contains("italic") || name.contains("emphasis") => {
            "Formatting"
        }
        name if name.contains("link") || name.contains("url") || name.contains("autolink") => {
            "Links"
        }
        name if name.contains("list") || name.contains("task") => "Lists",
        name if name.contains("code") || name.contains("math") => "Code & Math",
        name if name.contains("table") => "Tables",
        name if name.contains("image") => "Images",
        name if name.contains("admonition") || name.contains("tab") => "Marco Extensions",
        name if name.contains("text") || name.contains("word") || name.contains("unicode") => {
            "Text Processing"
        }
        name if name.starts_with("KW_") => "Keywords",
        name if name == "WHITESPACE" || name == "NEWLINE" || name.contains("INDENT") => {
            "Whitespace"
        }
        _ => "Other",
    }
}

/// Represents a simple parse tree node for analysis
#[derive(Debug, Clone)]
pub struct ParseNode {
    pub rule: Rule,
    pub text: String,
    pub span: SourceSpan,
    pub children: Vec<ParseNode>,
}

impl ParseNode {
    /// Creates a new parse node from a Pest pair
    pub fn from_pair(pair: Pair<Rule>) -> Self {
        let rule = pair.as_rule();
        let text = pair.as_str().to_string();
        let span = pair.as_span().to_source_span();

        let children: Vec<ParseNode> = pair.into_inner().map(ParseNode::from_pair).collect();

        Self {
            rule,
            text,
            span,
            children,
        }
    }

    /// Gets the rule name as a string
    pub fn rule_name(&self) -> String {
        format!("{:?}", self.rule)
    }

    /// Checks if this node is a leaf (has no children)
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Gets all descendant nodes with a specific rule
    pub fn find_nodes_by_rule(&self, target_rule: Rule) -> Vec<&ParseNode> {
        let mut results = Vec::new();
        self.find_nodes_by_rule_recursive(target_rule, &mut results);
        results
    }

    fn find_nodes_by_rule_recursive<'a>(
        &'a self,
        target_rule: Rule,
        results: &mut Vec<&'a ParseNode>,
    ) {
        if std::mem::discriminant(&self.rule) == std::mem::discriminant(&target_rule) {
            results.push(self);
        }

        for child in &self.children {
            child.find_nodes_by_rule_recursive(target_rule, results);
        }
    }

    /// Gets the depth of the tree from this node
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            0
        } else {
            1 + self.children.iter().map(|c| c.depth()).max().unwrap_or(0)
        }
    }

    /// Counts total nodes in the tree
    pub fn node_count(&self) -> usize {
        1 + self.children.iter().map(|c| c.node_count()).sum::<usize>()
    }

    /// Gets all unique rules used in this tree
    pub fn get_unique_rules(&self) -> HashSet<String> {
        let mut rules = HashSet::new();
        self.collect_rules_recursive(&mut rules);
        rules
    }

    fn collect_rules_recursive(&self, rules: &mut HashSet<String>) {
        rules.insert(self.rule_name());
        for child in &self.children {
            child.collect_rules_recursive(rules);
        }
    }
}

/// Converts Pest pairs to a simplified parse tree
pub fn pairs_to_parse_tree(pairs: Pairs<Rule>) -> Vec<ParseNode> {
    pairs.map(ParseNode::from_pair).collect()
}

/// Information about a grammar rule extracted from pest file
#[derive(Debug, Clone)]
pub struct RuleInfo {
    pub name: String,
    pub category: String,
    pub dependencies: Vec<String>,
    pub definition: String,
}

/// Extracts rule dependencies from a rule definition
/// Extracted from debug/src/grammar_visualizer.rs
pub fn extract_rule_dependencies(rule_definition: &str) -> Vec<String> {
    let mut dependencies = Vec::new();

    // Remove operators and extract identifiers
    let cleaned = rule_definition
        .replace("@{", " ")
        .replace("_{", " ")
        .replace("${", " ")
        .replace("!{", " ")
        .replace("}", " ")
        .replace("(", " ")
        .replace(")", " ")
        .replace("[", " ")
        .replace("]", " ")
        .replace("\"", " ")
        .replace("'", " ")
        .replace("~", " ")
        .replace("|", " ")
        .replace("*", " ")
        .replace("+", " ")
        .replace("?", " ")
        .replace("&", " ")
        .replace("!", " ")
        .replace("{", " ")
        .replace("^", " ");

    // Extract potential rule names (exclude literals and built-ins)
    for token in cleaned.split_whitespace() {
        if is_valid_rule_name(token) {
            dependencies.push(token.to_string());
        }
    }

    // Remove duplicates while preserving order
    let mut unique_deps = Vec::new();
    for dep in dependencies {
        if !unique_deps.contains(&dep) {
            unique_deps.push(dep);
        }
    }

    unique_deps
}

/// Checks if a token is a valid rule name (not a literal or built-in)
/// Extracted from debug/src/grammar_visualizer.rs
pub fn is_valid_rule_name(token: &str) -> bool {
    // Skip empty strings
    if token.is_empty() {
        return false;
    }

    // Skip literals (quoted strings)
    if token.starts_with('"') || token.starts_with('\'') {
        return false;
    }

    // Skip numbers and single characters
    if token.len() == 1 || token.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Skip pest built-ins
    let builtins = [
        "SOI",
        "EOI",
        "NEWLINE",
        "ANY",
        "ASCII_DIGIT",
        "ASCII_ALPHA",
        "ASCII_ALPHANUMERIC",
        "LETTER",
        "PUNCTUATION",
        "WHITESPACE",
    ];
    if builtins.contains(&token) {
        return false;
    }

    // Skip common operators and keywords
    let operators = ["=", ":", ".", "-", "+", "0", "1", "2", "3", "4", "5", "6"];
    if operators.contains(&token) {
        return false;
    }

    // Must start with letter or underscore
    if let Some(first_char) = token.chars().next() {
        first_char.is_ascii_alphabetic() || first_char == '_'
    } else {
        false
    }
}

/// Analyzes the grammar file and extracts rule information
pub fn analyze_grammar(grammar_path: &str) -> Result<Vec<RuleInfo>, std::io::Error> {
    let grammar_content = fs::read_to_string(grammar_path)?;
    let mut rules = Vec::new();

    for line in grammar_content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        // Detect rule definitions
        if let Some(eq_pos) = line.find(" = ") {
            let rule_name = line[..eq_pos].trim();
            let rule_definition = &line[eq_pos + 3..];

            // Skip built-in rules
            if rule_name.starts_with("//") {
                continue;
            }

            // Extract dependencies from rule definition
            let dependencies = extract_rule_dependencies(rule_definition);
            let category = categorize_rule(rule_name).to_string();

            rules.push(RuleInfo {
                name: rule_name.to_string(),
                category,
                dependencies,
                definition: rule_definition.to_string(),
            });
        }
    }

    Ok(rules)
}

/// Groups rules by category
pub fn group_rules_by_category(rules: &[RuleInfo]) -> HashMap<String, Vec<&RuleInfo>> {
    let mut groups = HashMap::new();

    for rule in rules {
        groups
            .entry(rule.category.clone())
            .or_insert_with(Vec::new)
            .push(rule);
    }

    groups
}

/// Parse with a specific rule and return parse tree
pub fn parse_with_rule(
    rule_name: &str,
    input: &str,
) -> Result<Vec<ParseNode>, Box<dyn std::error::Error>> {
    let rule = get_rule_by_name(rule_name).ok_or_else(|| format!("Unknown rule: {}", rule_name))?;

    let pairs = MarcoParser::parse(rule, input)?;
    Ok(pairs_to_parse_tree(pairs))
}

/// Quick validation of input against a rule
pub fn quick_validate(rule_name: &str, input: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let rule = get_rule_by_name(rule_name).ok_or_else(|| format!("Unknown rule: {}", rule_name))?;

    match MarcoParser::parse(rule, input) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Escape content for safe display (extracted from debug logic)
pub fn escape_content_for_display(content: &str) -> String {
    content
        .replace('\n', "\\n")
        .replace('\t', "\\t")
        .replace('\r', "\\r")
        .chars()
        .take(80) // Limit content length
        .collect()
}

/// Safe string truncation that respects Unicode character boundaries
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        // Find the last valid character boundary before max_len - 3 (for "...")
        let target_len = max_len.saturating_sub(3);
        let mut truncate_at = target_len.min(s.len());

        while truncate_at > 0 && !s.is_char_boundary(truncate_at) {
            truncate_at -= 1;
        }

        if truncate_at > 0 {
            format!("{}...", &s[..truncate_at])
        } else {
            "...".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_rule_by_name() {
        assert!(matches!(get_rule_by_name("file"), Some(Rule::file)));
        assert!(matches!(get_rule_by_name("heading"), Some(Rule::heading)));
        assert!(matches!(get_rule_by_name("bold"), Some(Rule::bold)));
        assert!(get_rule_by_name("nonexistent").is_none());
    }

    #[test]
    fn test_categorize_rule() {
        assert_eq!(categorize_rule("H1"), "Headings");
        assert_eq!(categorize_rule("bold"), "Formatting");
        assert_eq!(categorize_rule("inline_link"), "Links");
        assert_eq!(categorize_rule("text"), "Text Processing");
        assert_eq!(categorize_rule("KW_NOTE"), "Keywords");
    }

    #[test]
    fn test_is_valid_rule_name() {
        assert!(is_valid_rule_name("heading"));
        assert!(is_valid_rule_name("H1"));
        assert!(is_valid_rule_name("_private_rule"));

        assert!(!is_valid_rule_name(""));
        assert!(!is_valid_rule_name("123"));
        assert!(!is_valid_rule_name("\"quoted\""));
        assert!(!is_valid_rule_name("SOI"));
        assert!(!is_valid_rule_name("="));
    }

    #[test]
    fn test_extract_rule_dependencies() {
        let deps = extract_rule_dependencies("bold_asterisk | bold_underscore");
        assert!(deps.contains(&"bold_asterisk".to_string()));
        assert!(deps.contains(&"bold_underscore".to_string()));

        let deps = extract_rule_dependencies("\"**\" ~ inner_text ~ \"**\"");
        assert!(deps.contains(&"inner_text".to_string()));
        assert!(!deps.iter().any(|d| d.contains("**")));
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello", 10), "hello");
        assert_eq!(truncate_string("hello world", 8), "hello...");
        assert_eq!(truncate_string("abc", 2), "...");
    }

    #[test]
    fn test_escape_content_for_display() {
        let escaped = escape_content_for_display("hello\nworld\t!");
        assert_eq!(escaped, "hello\\nworld\\t!");
    }
}
