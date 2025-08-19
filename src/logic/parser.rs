//! Markdown syntax parser for active line trace
// Loads syntax rules from RON files and parses a line for Markdown elements

use std::collections::HashMap;
use std::fs;
use regex::Regex;
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
    #[serde(skip)]
    pub is_regex: bool,
    #[serde(skip)]
    pub regex: Option<Regex>,
}

#[derive(Clone)]
pub struct MarkdownSyntaxMap {
    pub rules: HashMap<String, SyntaxRule>,
    /// Optional display hints loaded from display_hints.ron in the schema dir
    pub display_hints: Option<DisplayHints>,
}

impl Default for MarkdownSyntaxMap {
    fn default() -> Self {
        Self { rules: HashMap::new(), display_hints: None }
    }
}

/// Data-driven display hints loaded from the schema dir (optional)
type DisplayHints = std::collections::HashMap<String, Vec<String>>;

impl MarkdownSyntaxMap {
    /// Try to read display_hints.ron from the given schema dir. If missing or
    /// invalid, return an empty map.
    fn load_display_hints(schema_dir: &str) -> DisplayHints {
        let mut hints: DisplayHints = DisplayHints::new();
        let path = PathBuf::from(schema_dir).join("display_hints.ron");
        if path.exists() {
            if let Ok(s) = fs::read_to_string(&path) {
                // The expected format is a simple RON map of string -> [string,...]
                // We'll attempt a very small and forgiving parse using serde and ron.
                if let Ok(parsed) = ron::from_str::<DisplayHints>(&s) {
                    hints = parsed;
                } else if std::env::var("MARCO_DEBUG_PARSER").is_ok() {
                    eprintln!("[parser debug] failed to parse display_hints.ron at {}", path.display());
                }
            }
        }
        hints
    }

}

/// Token produced by parsing a line: node_type plus optional metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxToken {
    pub node_type: String,
    pub depth: Option<u8>,
    pub ordered: Option<bool>,
    /// Optional map of capture names/indices to captured values from regex matches.
    pub captures: Option<HashMap<String, String>>,
}

impl MarkdownSyntaxMap {
    /// Loads syntax.ron from the given schema directory
    pub fn load_from_schema_dir(schema_dir: &str) -> anyhow::Result<Self> {
        let syntax_path = PathBuf::from(schema_dir).join("syntax.ron");
        let content = fs::read_to_string(&syntax_path)?;
    let mut rules = HashMap::new();
    // Try to load optional display hints in the same schema dir
    let display_hints: DisplayHints = Self::load_display_hints(schema_dir);
        for (lineno, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }

            if std::env::var("MARCO_DEBUG_PARSER").is_ok() {
                eprintln!("[parser debug] line {}: {}", lineno + 1, line);
            }

            // Extract body between braces { ... } if present, otherwise use the whole line
            let body = if let Some(start) = line.find('{') {
                if let Some(end) = line.rfind('}') {
                    &line[start + 1..end]
                } else {
                    &line[start + 1..]
                }
            } else {
                line
            };

            // Helper closures to extract fields like node_type: "value", depth: 1, ordered: true
            let extract_str = |key: &str| -> Option<String> {
                if let Some(pos) = body.find(key) {
                    if let Some(q1) = body[pos..].find('"') {
                        let start = pos + q1 + 1;
                        if let Some(q2_rel) = body[start..].find('"') {
                            let val = &body[start..start + q2_rel];
                            return Some(val.to_string());
                        }
                    }
                }
                None
            };

            let extract_u8 = |key: &str| -> Option<u8> {
                if let Some(pos) = body.find(key) {
                    let after = &body[pos + key.len()..];
                    // find digits
                    let digits: String = after.chars().skip_while(|c| !c.is_digit(10)).take_while(|c| c.is_digit(10)).collect();
                    if !digits.is_empty() {
                        if let Ok(n) = digits.parse::<u8>() {
                            return Some(n);
                        }
                    }
                }
                None
            };

            let extract_bool = |key: &str| -> Option<bool> {
                if let Some(pos) = body.find(key) {
                    let after = &body[pos + key.len()..];
                    if after.contains("true") { return Some(true); }
                    if after.contains("false") { return Some(false); }
                }
                None
            };

            let node_type = extract_str("node_type:").or_else(|| extract_str("node_type"));
            let markdown_syntax = extract_str("markdown_syntax:").or_else(|| extract_str("markdown_syntax"));
            let depth = extract_u8("depth:");
            let ordered = extract_bool("ordered:");

            if let Some(ms) = markdown_syntax {
                // Skip empty-match rules: they would match zero characters and cause
                // the parsing loop to make no progress, which leads to infinite loops
                // or crashes when parsing ordinary text.
                if ms.is_empty() {
                    if std::env::var("MARCO_DEBUG_PARSER").is_ok() {
                        eprintln!("[parser debug] skipping empty markdown_syntax on line {}", lineno + 1);
                    }
                } else {
                    // Use node_type fallback to ms if missing
                    let nt = node_type.unwrap_or_else(|| ms.clone());
                    // Detect regex-marked syntaxes. Use prefix `re:` to indicate a regular expression.
                    let mut is_re = false;
                    let mut compiled: Option<Regex> = None;
                    let ms_trimmed = ms.trim().to_string();
                    if ms_trimmed.starts_with("re:") {
                        is_re = true;
                        let pat = ms_trimmed.trim_start_matches("re:").trim();
                        // Anchor the pattern to the start so it behaves like starts_with
                        let anchored = format!("^(?:{})", pat);
                        match Regex::new(&anchored) {
                            Ok(r) => compiled = Some(r),
                            Err(err) => {
                                if std::env::var("MARCO_DEBUG_PARSER").is_ok() {
                                    eprintln!("[parser debug] invalid regex for '{}' on line {}: {}", ms, lineno + 1, err);
                                }
                                // Skip invalid regex rules
                                continue;
                            }
                        }
                    }

                    let rule = SyntaxRule { node_type: nt, depth, ordered, markdown_syntax: ms.clone(), is_regex: is_re, regex: compiled };
                    if std::env::var("MARCO_DEBUG_PARSER").is_ok() {
                        eprintln!("[parser debug] inserted rule for syntax '{}' -> {:?}", ms, rule);
                    }
                    // Use the raw markdown_syntax string as the map key to preserve uniqueness
                    rules.insert(ms, rule);
                }
            } else {
                if std::env::var("MARCO_DEBUG_PARSER").is_ok() {
                    eprintln!("[parser debug] skipping line {}: no markdown_syntax found", lineno + 1);
                }
            }
        }
        let mut map = Self { rules, display_hints: None };
        // Attach loaded display hints to the map (if any)
        if !display_hints.is_empty() {
            map.display_hints = Some(display_hints);
        }
        Ok(map)
    }

    /// Build a simple map of node_type -> ordered list of preferred capture names.
    /// We inspect each rule's compiled regex (if any) for named capture groups and
    /// use them as hints for UI formatting. We also provide a small set of
    /// sensible fallbacks for common node types.
    pub fn build_display_hints(&self) -> std::collections::HashMap<String, Vec<String>> {
        // If explicit display hints were loaded from the schema, prefer them.
        if let Some(h) = &self.display_hints {
            return h.clone();
        }

        let mut hints: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        for (_ms, rule) in &self.rules {
            let mut names: Vec<String> = Vec::new();
            if let Some(re) = &rule.regex {
                for n in re.capture_names().flatten() {
                    names.push(n.to_string());
                }
            }

            // If no named captures were found, provide small node_type based fallbacks
            if names.is_empty() {
                match rule.node_type.as_str() {
                    "def_description" | "definition" => names.push("desc".to_string()),
                    "frontmatter" => names.push("value".to_string()),
                    "heading" => names.push("text".to_string()),
                    "image-size" => names.push("w".to_string()),
                    "video" => { names.push("id1".to_string()); names.push("id2".to_string()); },
                    "link-target" | "link" => { names.push("h".to_string()); names.push("t".to_string()); },
                    _ => {}
                }
            }

            hints.insert(rule.node_type.clone(), names);
        }
        hints
    }

    /// Loads the active schema from settings.ron
    pub fn load_active_schema(settings_path: &str, schema_root: &str) -> anyhow::Result<Option<Self>> {
        let settings = fs::read_to_string(settings_path)?;

        // Heuristic parse the RON settings file for active_schema and schema_disabled.
        // The project's settings.ron typically uses `active_schema:Some("Name")` and
        // `schema_disabled:Some(false)`. We'll extract the inner string if present.
        let mut active_schema: Option<String> = None;
        let mut schema_disabled = false;

        // Try to find active_schema:Some("Name") pattern
        if let Some(pos) = settings.find("active_schema:Some(") {
            if let Some(start_quote) = settings[pos..].find('"') {
                let start = pos + start_quote + 1;
                if let Some(end_quote_rel) = settings[start..].find('"') {
                    let name = &settings[start..start + end_quote_rel];
                    if !name.is_empty() {
                        active_schema = Some(name.to_string());
                    }
                }
            }
        } else if let Some(pos) = settings.find("active_schema:") {
            // Fallback: look for a quoted value after the key
            if let Some(start_quote) = settings[pos..].find('"') {
                let start = pos + start_quote + 1;
                if let Some(end_quote_rel) = settings[start..].find('"') {
                    let name = &settings[start..start + end_quote_rel];
                    if !name.is_empty() {
                        active_schema = Some(name.to_string());
                    }
                }
            }
        }

        if settings.contains("schema_disabled:Some(true)") || settings.contains("schema_disabled:true") {
            schema_disabled = true;
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

/// Collect reference-style link definitions from a document.
/// Returns a map id -> (url, optional title).
pub fn collect_link_definitions(doc: &str) -> HashMap<String, (String, Option<String>)> {
    let mut defs = HashMap::new();
    let re = Regex::new(r#"^\s*\[(?P<id>[^\]]+)\]:\s*(?P<url>\S+)(?:\s+"(?P<title>.+?)")?"#).unwrap();
    for line in doc.lines() {
        if let Some(caps) = re.captures(line) {
            let id = caps.name("id").unwrap().as_str().to_string().to_lowercase();
            let url = caps.name("url").unwrap().as_str().to_string();
            let title = caps.name("title").map(|m| m.as_str().to_string());
            defs.insert(id, (url, title));
        }
    }
    defs
}

/// Parse a whole document with a small block-level pre-pass to support:
/// - Setext heading retroactive conversion (underline on following line)
/// - Multi-line frontmatter parsing (YAML delimited by `---`)
/// - Aggregation of reference-style link definitions
/// Returns (tokens, link_definitions)
pub fn parse_document_blocks(doc: &str, syntax_map: &MarkdownSyntaxMap) -> (Vec<SyntaxToken>, HashMap<String, (String, Option<String>)>) {
    let mut tokens: Vec<SyntaxToken> = Vec::new();
    // Aggregate link definitions for the whole doc using helper (covers multi-line defs)
    let link_defs: HashMap<String, (String, Option<String>)> = collect_link_definitions(doc);

    let setext_re = Regex::new(r"^(?P<ch>=+|-+)\s*$").unwrap();

    let mut in_front = false;
    let mut front_buf: Vec<String> = Vec::new();
    let mut prev_text_line: Option<String> = None;

    for line in doc.lines() {
        let trimmed = line.trim_end();

        // Frontmatter handling: start when a `---` appears (typically at document start)
        if !in_front && trimmed == "---" && tokens.is_empty() {
            in_front = true;
            front_buf.push(trimmed.to_string());
            continue;
        }
        if in_front {
            front_buf.push(trimmed.to_string());
            if trimmed == "---" {
                // end of frontmatter
                let value = front_buf.join("\n");
                let mut caps = HashMap::new();
                caps.insert("value".to_string(), value);
                tokens.push(SyntaxToken { node_type: "frontmatter".to_string(), depth: None, ordered: None, captures: Some(caps) });
                front_buf.clear();
                in_front = false;
            }
            continue;
        }

        // Link definition lines are handled by the initial collect_link_definitions pass;
        // skip emitting tokens for those lines when encountered.
        if collect_link_definitions(trimmed).len() > 0 {
            // If this trimmed line contains a link definition, skip it.
            // (This is a cheap check that avoids re-parsing the full doc.)
            if Regex::new(r"^\s*\[(?P<id>[^\]]+)\]:").unwrap().is_match(trimmed) {
                continue;
            }
        }

        // Setext underline retroactive heading conversion
        if let Some(caps) = setext_re.captures(trimmed) {
            // ch is '=' or '-'
            if let Some(prev) = prev_text_line.take() {
                let ch = caps.name("ch").unwrap().as_str().chars().next().unwrap();
                let depth = if ch == '=' { 1u8 } else { 2u8 };
                let mut capmap = HashMap::new();
                capmap.insert("text".to_string(), prev);
                tokens.push(SyntaxToken { node_type: "heading".to_string(), depth: Some(depth), ordered: None, captures: Some(capmap) });
            }
            continue;
        }

        // Normal line: run line-level parser and record tokens
        let line_tokens = parse_line_syntax(trimmed, syntax_map);
        if !trimmed.trim().is_empty() {
            prev_text_line = Some(trimmed.trim().to_string());
        }
        tokens.extend(line_tokens);
    }

    (tokens, link_defs)
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
            // Skip any empty-match rules (e.g. markdown_syntax == "") to avoid
            // zero-length matches which would cause the main loop to make no
            // progress and hang/crash on ordinary text input.
            if syntax.is_empty() {
                continue;
            }
                    if rule.is_regex {
                if let Some(re) = &rule.regex {
                    if let Some(mat) = re.find(&remaining_chars) {
                        if mat.start() == 0 {
                                    // Build captures map from the regex match
                                    let mut captures_map: Option<HashMap<String, String>> = None;
                                    if let Some(caps) = re.captures(&remaining_chars) {
                                        let mut map = HashMap::new();
                                        // Named captures
                                        for name in re.capture_names().flatten() {
                                            if let Some(m) = caps.name(name) {
                                                map.insert(name.to_string(), m.as_str().to_string());
                                            }
                                        }
                                        // Unnamed numeric groups (1..n)
                                        for i in 1..caps.len() {
                                            if let Some(m) = caps.get(i) {
                                                let key = format!("g{}", i);
                                                map.insert(key, m.as_str().to_string());
                                            }
                                        }
                                        if !map.is_empty() {
                                            captures_map = Some(map);
                                        }
                                    }

                                    if !found.contains(&rule.node_type) {
                                        let token = SyntaxToken {
                                            node_type: rule.node_type.clone(),
                                            depth: rule.depth,
                                            ordered: rule.ordered,
                                            captures: captures_map,
                                        };
                                        chain.push(token);
                                        found.push(rule.node_type.clone());
                                    }
                                    chars_to_skip = mat.end();
                            break;
                        }
                    }
                }
                    } else {
                if remaining_chars.starts_with(syntax.as_str()) {
                    if !found.contains(&rule.node_type) {
                        let token = SyntaxToken {
                            node_type: rule.node_type.clone(),
                            depth: rule.depth,
                            ordered: rule.ordered,
                            captures: None,
                        };
                        chain.push(token);
                        found.push(rule.node_type.clone());
                    }
                    // Count how many characters this syntax spans
                    chars_to_skip = syntax.chars().count();
                    break;
                }
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
    rules.insert("**".to_string(), SyntaxRule { node_type: "bold".to_string(), depth: None, ordered: None, markdown_syntax: "**".to_string(), is_regex: false, regex: None });
    rules.insert("#".to_string(), SyntaxRule { node_type: "heading".to_string(), depth: Some(1), ordered: None, markdown_syntax: "#".to_string(), is_regex: false, regex: None });
    MarkdownSyntaxMap { rules, display_hints: None }
    }

    #[test]
    fn test_parse_line_syntax_finds_tokens() {
        let map = make_test_map();
        let chain = parse_line_syntax("# Hello **world**", &map);
    // heading token with depth 1 should appear
    let heading_token = SyntaxToken { node_type: "heading".to_string(), depth: Some(1), ordered: None, captures: None };
    let bold_token = SyntaxToken { node_type: "bold".to_string(), depth: None, ordered: None, captures: None };
    assert!(chain.contains(&heading_token));
    assert!(chain.contains(&bold_token));
    // Ensure the `get` method is exercised
    assert!(map.get("#").is_some());
    }

    #[test]
    fn test_regex_captures_video_image_link() {
        // Build a map with regex rules compiled manually for testing
        let mut rules = std::collections::HashMap::new();
    // Video embed pattern: capture the id in two separate named groups (id1 and id2)
    // Rust's regex crate does not support backreferences like (?P=id), so we capture both
    // occurrences and compare them in the test.
    let video_pat = "re:\\[!\\[.*?\\]\\(https?://img\\.youtube\\.com/vi/(?P<id1>[A-Za-z0-9_-]+)/0\\.jpg\\)\\]\\(https?://(www\\.)?youtube\\.com/watch\\?v=(?P<id2>[A-Za-z0-9_-]+)\\)".to_string();
    let video_re = regex::Regex::new("^(?:\\[!\\[.*?\\]\\(https?://img\\.youtube\\.com/vi/(?P<id1>[A-Za-z0-9_-]+)/0\\.jpg\\)\\]\\(https?://(www\\.)?youtube\\.com/watch\\?v=(?P<id2>[A-Za-z0-9_-]+)\\))").unwrap();
    rules.insert(video_pat.clone(), SyntaxRule { node_type: "video".to_string(), depth: None, ordered: None, markdown_syntax: video_pat, is_regex: true, regex: Some(video_re) });

        // Image width pattern
        let img_pat = "re:<img\\s+[^>]*width=[\"']?(?P<w>\\d+)[\"']?[^>]*>".to_string();
        let img_re = regex::Regex::new("^(?:<img\\s+[^>]*width=[\"']?(?P<w>\\d+)[\"']?[^>]*>)").unwrap();
        rules.insert(img_pat.clone(), SyntaxRule { node_type: "image-size".to_string(), depth: None, ordered: None, markdown_syntax: img_pat, is_regex: true, regex: Some(img_re) });

        // Link target pattern
        let link_pat = "re:<a\\s+[^>]*href=\\\"(?P<h>[^\\\"]+)\\\"[^>]*target=\\\"(?P<t>[^\\\"]+)\\\"[^>]*>".to_string();
        let link_re = regex::Regex::new("^(?:<a\\s+[^>]*href=\\\"(?P<h>[^\\\"]+)\\\"[^>]*target=\\\"(?P<t>[^\\\"]+)\\\"[^>]*>)").unwrap();
        rules.insert(link_pat.clone(), SyntaxRule { node_type: "link-target".to_string(), depth: None, ordered: None, markdown_syntax: link_pat, is_regex: true, regex: Some(link_re) });

    let map = MarkdownSyntaxMap { rules, display_hints: None };

        // Test video
        let line = "[![Image alt](https://img.youtube.com/vi/AbC123/0.jpg)](https://www.youtube.com/watch?v=AbC123)";
        let chain = parse_line_syntax(line, &map);
    assert!(!chain.is_empty());
    assert_eq!(chain[0].node_type, "video");
    let caps = chain[0].captures.as_ref().unwrap();
    // ensure both captured id1 and id2 exist and are equal
    let id1 = caps.get("id1").map(|s| s.as_str());
    let id2 = caps.get("id2").map(|s| s.as_str());
    assert!(id1.is_some() && id2.is_some());
    assert_eq!(id1, id2);

        // Test image width
        let line2 = "<img src=\"x.png\" width=300>";
        let chain2 = parse_line_syntax(line2, &map);
        assert!(!chain2.is_empty());
        assert_eq!(chain2[0].node_type, "image-size");
        let caps2 = chain2[0].captures.as_ref().unwrap();
        assert_eq!(caps2.get("w").map(|s| s.as_str()), Some("300"));

        // Test link target
        let line3 = "<a href=\"https://example.com\" target=\"_blank\">";
        let chain3 = parse_line_syntax(line3, &map);
        assert!(!chain3.is_empty());
        assert_eq!(chain3[0].node_type, "link-target");
        let caps3 = chain3[0].captures.as_ref().unwrap();
        assert_eq!(caps3.get("h").map(|s| s.as_str()), Some("https://example.com"));
        assert_eq!(caps3.get("t").map(|s| s.as_str()), Some("_blank"));
    }
}
