//! Markdown syntax parser for active line trace
// Loads syntax rules from RON files and parses a line for Markdown elements

use std::collections::HashMap;
use std::fs;
use ron::de::from_str;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct SyntaxRule {
    pub node_type: String,
    #[serde(default)]
    pub depth: Option<u8>,
    #[serde(default)]
    pub ordered: Option<bool>,
    pub markdown_syntax: String,
}

#[derive(Clone)]
pub struct MarkdownSyntaxMap {
    pub rules: HashMap<String, SyntaxRule>,
}

/// Token produced by parsing a line: node_type plus optional metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxToken {
    pub node_type: String,
    pub depth: Option<u8>,
    pub ordered: Option<bool>,
}

impl MarkdownSyntaxMap {
    /// Loads syntax.ron from the given schema directory
    pub fn load_from_schema_dir(schema_dir: &str) -> anyhow::Result<Self> {
        let syntax_path = PathBuf::from(schema_dir).join("syntax.ron");
        let content = fs::read_to_string(&syntax_path)?;
        let mut rules = HashMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            if let Ok(rule) = from_str::<SyntaxRule>(line) {
                // Touch optional fields so they are considered 'used' by the compiler
                let _ = &rule.depth;
                let _ = &rule.ordered;
                rules.insert(rule.markdown_syntax.clone(), rule);
            }
        }
        Ok(Self { rules })
    }

    /// Loads the active schema from settings.ron
    pub fn load_active_schema(settings_path: &str, schema_root: &str) -> anyhow::Result<Option<Self>> {
        let settings = fs::read_to_string(settings_path)?;
        let mut active_schema: Option<String> = None;
        let mut schema_disabled = false;
        for line in settings.lines() {
            if line.contains("active_schema") {
                if let Some(val) = line.split(':').nth(1) {
                    let val = val.trim().trim_matches('"');
                    if !val.is_empty() {
                        active_schema = Some(val.to_string());
                    }
                }
            }
            if line.contains("schema_disabled") {
                if let Some(val) = line.split(':').nth(1) {
                    let val = val.trim();
                    schema_disabled = val == "true";
                }
            }
        }
        if schema_disabled {
            return Ok(None);
        }
        if let Some(schema_name) = active_schema {
            let schema_dir = PathBuf::from(schema_root).join(&schema_name);
            Self::load_from_schema_dir(schema_dir.to_str().unwrap()).map(Some)
        } else {
            Ok(None)
        }
    }
    /// Returns the SyntaxRule for a given markdown syntax
    #[allow(dead_code)]
    pub fn get(&self, syntax: &str) -> Option<&SyntaxRule> {
        self.rules.get(syntax)
    }
}

/// Parses a line and returns a left-to-right chain of Markdown syntax elements
pub fn parse_line_syntax(line: &str, syntax_map: &MarkdownSyntaxMap) -> Vec<SyntaxToken> {
    let mut chain = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut idx = 0;
    // Track which node_types we've already added (dedupe by node_type)
    let mut found = vec![];
    // Sort syntax by length descending to match longer tokens first
    let mut syntax_list: Vec<_> = syntax_map.rules.iter().collect();
    syntax_list.sort_by_key(|(s, _)| -(s.len() as isize));

    while idx < chars.len() {
        // Convert back to string slice for matching
        let remaining_chars: String = chars[idx..].iter().collect();
        let mut chars_to_skip = 1; // Default: advance by one character
        
        for (syntax, rule) in &syntax_list {
            if remaining_chars.starts_with(syntax.as_str()) {
                if !found.contains(&rule.node_type) {
                    let token = SyntaxToken {
                        node_type: rule.node_type.clone(),
                        depth: rule.depth,
                        ordered: rule.ordered,
                    };
                    chain.push(token);
                    found.push(rule.node_type.clone());
                }
                // Count how many characters this syntax spans
                chars_to_skip = syntax.chars().count();
                break;
            }
        }
        
        idx += chars_to_skip;
    }
    chain
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_map() -> MarkdownSyntaxMap {
        let mut rules = std::collections::HashMap::new();
    rules.insert("**".to_string(), SyntaxRule { node_type: "bold".to_string(), depth: None, ordered: None, markdown_syntax: "**".to_string() });
    rules.insert("#".to_string(), SyntaxRule { node_type: "heading".to_string(), depth: Some(1), ordered: None, markdown_syntax: "#".to_string() });
        MarkdownSyntaxMap { rules }
    }

    #[test]
    fn test_parse_line_syntax_finds_tokens() {
        let map = make_test_map();
        let chain = parse_line_syntax("# Hello **world**", &map);
    // heading token with depth 1 should appear
    let heading_token = SyntaxToken { node_type: "heading".to_string(), depth: Some(1), ordered: None };
    let bold_token = SyntaxToken { node_type: "bold".to_string(), depth: None, ordered: None };
    assert!(chain.contains(&heading_token));
    assert!(chain.contains(&bold_token));
    // Ensure the `get` method is exercised
    assert!(map.get("#").is_some());
    }
}
