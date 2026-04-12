//! Embedded diagnostics catalog loaded from RON at compile time.
//!
//! Catalog sources live next to this module and are embedded via `include_str!`:
//! - Marco-native catalog: `core/src/intelligence/diagnostics_catalog_marco.ron`
//! - markdownlint baseline catalog: `core/src/intelligence/diagnostics_catalog_markdownlint.ron`

use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct DiagnosticsCatalog {
    pub version: u32,
    #[serde(default)]
    pub settings: DiagnosticsCatalogSettings,
    #[serde(default)]
    pub groups: Vec<DiagnosticsCatalogGroup>,
    #[serde(default)]
    pub features: Vec<MarkdownFeatureCoverage>,
    pub entries: Vec<DiagnosticsCatalogEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarkdownFeatureCoverage {
    pub key: String,
    pub title: String,
    pub category: String,
    pub status: String,
    #[serde(default)]
    pub node_kinds: Vec<String>,
    pub showcase_doc: Option<String>,
    #[serde(default)]
    pub related_diagnostics: Vec<String>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiagnosticsCatalogGroup {
    pub id: String,
    pub title: String,
    pub description: String,
    pub code_prefix: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiagnosticsCatalogSettings {
    pub heading_too_long_threshold: usize,
    pub unsafe_protocols: Vec<String>,
    pub insecure_link_prefixes: Vec<String>,
    pub script_tag_markers: Vec<String>,
    pub unknown_code_fallback: String,
    pub unknown_message_fallback: String,
    pub unknown_fix_suggestion_fallback: String,
    pub unknown_protocol_label: String,
}

impl Default for DiagnosticsCatalogSettings {
    fn default() -> Self {
        Self {
            heading_too_long_threshold: 120,
            unsafe_protocols: vec!["javascript".to_string(), "data".to_string()],
            insecure_link_prefixes: vec!["http://".to_string()],
            script_tag_markers: vec!["<script".to_string()],
            unknown_code_fallback: "UNKNOWN".to_string(),
            unknown_message_fallback: "Unknown diagnostic".to_string(),
            unknown_fix_suggestion_fallback: "No fix suggestion available.".to_string(),
            unknown_protocol_label: "unknown".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiagnosticsCatalogEntry {
    pub key: String,
    pub code: String,
    pub title: String,
    #[serde(default)]
    pub message_template: Option<String>,
    pub default_severity: String,
    pub fix_suggestion: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub examples: Vec<String>,
}

const DIAGNOSTICS_CATALOG_MARCO_RON: &str = include_str!("diagnostics_catalog_marco.ron");
const DIAGNOSTICS_CATALOG_MARKDOWNLINT_RON: &str =
    include_str!("diagnostics_catalog_markdownlint.ron");

fn parse_catalog(source_name: &str, ron_src: &str) -> Option<DiagnosticsCatalog> {
    match ron::de::from_str::<DiagnosticsCatalog>(ron_src) {
        Ok(catalog) => Some(catalog),
        Err(err) => {
            log::error!(
                "Failed to parse embedded diagnostics catalog ({}): {}",
                source_name,
                err
            );
            None
        }
    }
}

fn merge_catalogs(
    mut marco: DiagnosticsCatalog,
    markdownlint: DiagnosticsCatalog,
) -> DiagnosticsCatalog {
    // Keep Marco settings as authoritative for runtime policy.
    marco.version = marco.version.max(markdownlint.version);

    for group in markdownlint.groups {
        if marco.groups.iter().all(|g| g.id != group.id) {
            marco.groups.push(group);
        }
    }

    for feature in markdownlint.features {
        if marco.features.iter().all(|f| f.key != feature.key) {
            marco.features.push(feature);
        }
    }

    for entry in markdownlint.entries {
        let duplicate_key = marco.entries.iter().any(|e| e.key == entry.key);
        let duplicate_code = marco.entries.iter().any(|e| e.code == entry.code);
        if !(duplicate_key || duplicate_code) {
            marco.entries.push(entry);
        }
    }

    marco
}

static DIAGNOSTICS_CATALOG: LazyLock<DiagnosticsCatalog> = LazyLock::new(|| {
    let marco = parse_catalog("marco", DIAGNOSTICS_CATALOG_MARCO_RON);
    let markdownlint = parse_catalog("markdownlint", DIAGNOSTICS_CATALOG_MARKDOWNLINT_RON);

    match (marco, markdownlint) {
        (Some(marco), Some(markdownlint)) => merge_catalogs(marco, markdownlint),
        (Some(marco), None) => marco,
        (None, Some(markdownlint)) => markdownlint,
        (None, None) => DiagnosticsCatalog::default(),
    }
});

/// Returns the embedded diagnostics catalog parsed from RON.
pub fn diagnostics_catalog() -> &'static DiagnosticsCatalog {
    &DIAGNOSTICS_CATALOG
}

/// Returns shared diagnostics analysis policy settings.
pub fn diagnostics_catalog_settings() -> &'static DiagnosticsCatalogSettings {
    &diagnostics_catalog().settings
}

/// Returns diagnostics groups metadata from the embedded catalog.
pub fn diagnostics_catalog_groups() -> &'static [DiagnosticsCatalogGroup] {
    &diagnostics_catalog().groups
}

/// Lookup a diagnostics group by id (e.g. `links`, `html`).
pub fn find_catalog_group(id: &str) -> Option<&'static DiagnosticsCatalogGroup> {
    diagnostics_catalog_groups()
        .iter()
        .find(|group| group.id == id)
}

/// Lookup a diagnostics group by code id prefix (e.g. `MD2` for links).
pub fn find_catalog_group_by_code(code: &str) -> Option<&'static DiagnosticsCatalogGroup> {
    diagnostics_catalog_groups()
        .iter()
        .filter(|group| code.starts_with(group.code_prefix.as_str()))
        .max_by_key(|group| group.code_prefix.len())
}

/// Returns markdown feature coverage metadata from the embedded catalog.
pub fn diagnostics_markdown_features() -> &'static [MarkdownFeatureCoverage] {
    &diagnostics_catalog().features
}

/// Lookup a markdown feature coverage record by key.
pub fn find_markdown_feature(key: &str) -> Option<&'static MarkdownFeatureCoverage> {
    diagnostics_markdown_features()
        .iter()
        .find(|feature| feature.key == key)
}

/// Fast lookup by diagnostic code id (e.g. `MD203`).
pub fn find_catalog_entry(code: &str) -> Option<&'static DiagnosticsCatalogEntry> {
    diagnostics_catalog()
        .entries
        .iter()
        .find(|entry| entry.code == code)
}

/// Fast lookup by diagnostic enum key (e.g. `EmptyImageUrl`).
pub fn find_catalog_entry_by_key(key: &str) -> Option<&'static DiagnosticsCatalogEntry> {
    diagnostics_catalog()
        .entries
        .iter()
        .find(|entry| entry.key == key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn is_valid_severity(value: &str) -> bool {
        matches!(value, "Error" | "Warning" | "Info" | "Hint")
    }

    fn is_md_three_digit_code(code: &str) -> bool {
        let mut chars = code.chars();
        matches!(
            (
                chars.next(),
                chars.next(),
                chars.next(),
                chars.next(),
                chars.next(),
                chars.next(),
            ),
            (Some('M'), Some('D'), Some(a), Some(b), Some(c), None)
                if a.is_ascii_digit() && b.is_ascii_digit() && c.is_ascii_digit()
        )
    }

    #[test]
    fn smoke_test_embedded_catalog_parses() {
        let catalog = diagnostics_catalog();
        assert!(catalog.version >= 1);
        assert!(!catalog.entries.is_empty());
    }

    #[test]
    fn smoke_test_catalog_has_known_code() {
        let md060 = find_catalog_entry("MD060");
        assert!(md060.is_some());
    }

    #[test]
    fn smoke_test_markdownlint_code_present() {
        let md060 = find_catalog_entry("MD060");
        assert!(md060.is_some());
    }

    #[test]
    fn smoke_test_catalog_has_known_key() {
        let entry = find_catalog_entry_by_key("EmptyImageUrl");
        assert!(entry.is_some());
    }

    #[test]
    fn smoke_test_catalog_settings_have_defaults() {
        let settings = diagnostics_catalog_settings();
        assert!(settings.heading_too_long_threshold > 0);
        assert!(!settings.unsafe_protocols.is_empty());
        assert!(!settings.insecure_link_prefixes.is_empty());
        assert!(!settings.script_tag_markers.is_empty());
        assert!(!settings.unknown_code_fallback.is_empty());
        assert!(!settings.unknown_message_fallback.is_empty());
        assert!(!settings.unknown_fix_suggestion_fallback.is_empty());
        assert!(!settings.unknown_protocol_label.is_empty());
    }

    #[test]
    fn smoke_test_catalog_has_groups() {
        assert!(!diagnostics_catalog_groups().is_empty());
        assert!(find_catalog_group("links").is_some());
        assert!(find_catalog_group_by_code("MD203").is_some());
    }

    #[test]
    fn smoke_test_group_lookup_prefers_longest_prefix_match() {
        // MD101 should resolve to the Marco parse group (prefix MD1)
        // instead of the broad markdownlint baseline group (prefix MD).
        let group = find_catalog_group_by_code("MD101").expect("expected group for MD101");
        assert_eq!(group.id, "parse");
    }

    #[test]
    fn smoke_test_catalog_has_markdown_feature_coverage() {
        let features = diagnostics_markdown_features();
        assert!(!features.is_empty());
        assert!(find_markdown_feature("math").is_some());
        assert!(find_markdown_feature("task-lists").is_some());
        assert!(
            features.iter().all(|feature| !feature.examples.is_empty()),
            "all markdown feature records should include at least one example"
        );
    }

    #[test]
    fn smoke_test_feature_node_kinds_match_known_ast_variants() {
        let known_node_kinds: HashSet<&'static str> = [
            "Heading",
            "Paragraph",
            "CodeBlock",
            "ThematicBreak",
            "List",
            "ListItem",
            "DefinitionList",
            "DefinitionTerm",
            "DefinitionDescription",
            "TaskCheckbox",
            "Blockquote",
            "Admonition",
            "TabGroup",
            "TabItem",
            "SliderDeck",
            "Slide",
            "Table",
            "TableRow",
            "TableCell",
            "HtmlBlock",
            "FootnoteDefinition",
            "Text",
            "TaskCheckboxInline",
            "Emphasis",
            "Strong",
            "StrongEmphasis",
            "Strikethrough",
            "Mark",
            "Superscript",
            "Subscript",
            "Link",
            "LinkReference",
            "FootnoteReference",
            "Image",
            "CodeSpan",
            "InlineHtml",
            "HardBreak",
            "SoftBreak",
            "PlatformMention",
            "InlineMath",
            "DisplayMath",
            "MermaidDiagram",
        ]
        .into_iter()
        .collect();

        for feature in diagnostics_markdown_features() {
            for kind in &feature.node_kinds {
                assert!(
                    known_node_kinds.contains(kind.as_str()),
                    "unknown node kind '{}' in feature '{}'",
                    kind,
                    feature.key
                );
            }
        }
    }

    #[test]
    fn smoke_test_marco_catalog_entries_use_supported_prefixes() {
        let marco = parse_catalog("marco", DIAGNOSTICS_CATALOG_MARCO_RON)
            .expect("marco catalog should parse in tests");

        for entry in &marco.entries {
            assert!(
                entry.code.starts_with("MD")
                    || entry.code.starts_with("MO")
                    || entry.code.starts_with("MG"),
                "unsupported diagnostics prefix for {} ({})",
                entry.key,
                entry.code
            );
        }
    }

    #[test]
    fn smoke_test_marco_catalog_has_no_code_overlap_with_markdownlint() {
        let marco = parse_catalog("marco", DIAGNOSTICS_CATALOG_MARCO_RON)
            .expect("marco catalog should parse in tests");
        let markdownlint = parse_catalog("markdownlint", DIAGNOSTICS_CATALOG_MARKDOWNLINT_RON)
            .expect("markdownlint catalog should parse in tests");

        let marco_codes: HashSet<&str> = marco
            .entries
            .iter()
            .map(|entry| entry.code.as_str())
            .collect();
        let markdownlint_codes: HashSet<&str> = markdownlint
            .entries
            .iter()
            .map(|entry| entry.code.as_str())
            .collect();

        let overlaps: Vec<&str> = marco_codes
            .intersection(&markdownlint_codes)
            .copied()
            .collect();

        assert!(
            overlaps.is_empty(),
            "marco/markdownlint code overlap detected: {:?}",
            overlaps
        );
    }

    #[test]
    fn smoke_test_all_catalog_entries_have_editor_required_fields() {
        let marco = parse_catalog("marco", DIAGNOSTICS_CATALOG_MARCO_RON)
            .expect("marco catalog should parse in tests");
        let markdownlint = parse_catalog("markdownlint", DIAGNOSTICS_CATALOG_MARKDOWNLINT_RON)
            .expect("markdownlint catalog should parse in tests");

        for (source, catalog) in [("marco", marco), ("markdownlint", markdownlint)] {
            for entry in &catalog.entries {
                assert!(
                    !entry.key.trim().is_empty(),
                    "{} entry has empty key (code={})",
                    source,
                    entry.code
                );
                assert!(
                    !entry.code.trim().is_empty(),
                    "{} entry has empty code (key={})",
                    source,
                    entry.key
                );
                assert!(
                    !entry.title.trim().is_empty(),
                    "{} entry {} has empty title",
                    source,
                    entry.code
                );
                assert!(
                    !entry.description.trim().is_empty(),
                    "{} entry {} has empty description",
                    source,
                    entry.code
                );
                assert!(
                    !entry.fix_suggestion.trim().is_empty(),
                    "{} entry {} has empty fix_suggestion",
                    source,
                    entry.code
                );
                assert!(
                    is_valid_severity(entry.default_severity.as_str()),
                    "{} entry {} has unsupported severity {}",
                    source,
                    entry.code,
                    entry.default_severity
                );
                if let Some(template) = &entry.message_template {
                    assert!(
                        !template.trim().is_empty(),
                        "{} entry {} has empty message_template",
                        source,
                        entry.code
                    );
                }
                assert!(
                    !entry.examples.is_empty(),
                    "{} entry {} must include at least one example",
                    source,
                    entry.code
                );
                assert!(
                    entry.examples.iter().all(|e| !e.trim().is_empty()),
                    "{} entry {} has blank example text",
                    source,
                    entry.code
                );
            }
        }
    }

    #[test]
    fn smoke_test_markdownlint_entries_have_editor_friendly_content() {
        let markdownlint = parse_catalog("markdownlint", DIAGNOSTICS_CATALOG_MARKDOWNLINT_RON)
            .expect("markdownlint catalog should parse in tests");

        for entry in &markdownlint.entries {
            assert!(
                is_md_three_digit_code(&entry.code),
                "markdownlint entry has invalid code format: {}",
                entry.code
            );
            assert!(
                entry.key.starts_with("MarkdownlintMD"),
                "markdownlint entry key must start with MarkdownlintMD: {}",
                entry.key
            );
            assert!(
                !entry
                    .fix_suggestion
                    .contains("See markdownlint docs for MD"),
                "markdownlint entry {} contains placeholder fix text",
                entry.code
            );

            for example in &entry.examples {
                let text = example.trim();
                let is_url_only = (text.starts_with("http://") || text.starts_with("https://"))
                    && !text.contains(char::is_whitespace);
                assert!(
                    !is_url_only,
                    "markdownlint entry {} has URL-only example: {}",
                    entry.code, text
                );
            }
        }
    }

    #[test]
    fn smoke_test_merged_catalog_has_unique_keys_and_codes() {
        let catalog = diagnostics_catalog();

        let mut keys = HashSet::new();
        let mut codes = HashSet::new();

        for entry in &catalog.entries {
            assert!(
                keys.insert(entry.key.as_str()),
                "duplicate catalog key in merged catalog: {}",
                entry.key
            );
            assert!(
                codes.insert(entry.code.as_str()),
                "duplicate catalog code in merged catalog: {}",
                entry.code
            );
        }
    }
}
