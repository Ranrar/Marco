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

pub struct MarkdownSyntaxMap {
    pub rules: HashMap<String, SyntaxRule>,
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
    pub fn get(&self, syntax: &str) -> Option<&SyntaxRule> {
        self.rules.get(syntax)
    }
}

/// Parses a line and returns a left-to-right chain of Markdown syntax elements
pub fn parse_line_syntax(line: &str, syntax_map: &MarkdownSyntaxMap) -> Vec<String> {
    let mut chain = Vec::new();
    let mut idx = 0;
    let line_len = line.len();
    let mut found = vec![];
    // Sort syntax by length descending to match longer tokens first
    let mut syntax_list: Vec<_> = syntax_map.rules.iter().collect();
    syntax_list.sort_by_key(|(s, _)| -(s.len() as isize));

    while idx < line_len {
        let slice = &line[idx..];
        let mut matched = false;
        for (syntax, rule) in &syntax_list {
            if slice.starts_with(syntax.as_str()) {
                if !found.contains(&rule.node_type) {
                    chain.push(rule.node_type.clone());
                    found.push(rule.node_type.clone());
                }
                idx += syntax.len();
                matched = true;
                break;
            }
        }
        if !matched {
            // Advance by one char if no syntax matched
            idx += 1;
        }
    }
    chain
}
