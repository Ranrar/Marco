//! Table of Contents extraction and Markdown generation.
//!
//! # Workflow
//! 1. Parse the document with `core::parse()`
//! 2. Call [`extract_toc`] to get a flat list of [`TocEntry`] items
//! 3. Call [`generate_toc_markdown`] to produce the fenced TOC block
//! 4. Call [`replace_toc_in_text`] to update or insert the block in the source

use crate::parser::{Document, NodeKind};
use std::collections::HashMap;

/// A single entry in the extracted Table of Contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TocEntry {
    pub level: u8,
    pub text: String,
    pub slug: String,
    /// 1-based source line number of the heading (from the AST span), or 0 when unknown.
    pub line: usize,
}

/// Result of attempting to replace an existing TOC block in source text.
#[derive(Debug)]
pub enum TocReplaceResult {
    /// No `<!-- TOC -->` markers found; caller should insert the block at cursor.
    NoMarkers,
    /// Existing block is identical to the new one — no change needed.
    NoChange,
    /// Returns the full source text with the TOC block replaced.
    Updated(String),
}

/// Generate a GitHub-compatible URL slug from heading text.
///
/// Algorithm:
/// 1. Lowercase
/// 2. Each character: alphanumeric → keep; space / hyphen / underscore → `-`; anything else → `-`
/// 3. Collapse consecutive hyphens into one
/// 4. Trim leading and trailing hyphens
pub fn heading_slug(text: &str) -> String {
    let lower = text.to_lowercase();

    let mut slug = String::with_capacity(lower.len());
    let mut prev_hyphen = false;

    for c in lower.chars() {
        let mapped = if c.is_alphanumeric() {
            prev_hyphen = false;
            slug.push(c);
            continue;
        } else {
            '-'
        };

        if !prev_hyphen {
            slug.push(mapped);
        }
        prev_hyphen = true;
    }

    // Trim leading/trailing hyphens
    let trimmed = slug.trim_matches('-');
    trimmed.to_string()
}

/// Extract all top-level headings from a document as a flat list of [`TocEntry`] items.
///
/// - Explicit `{#id}` from the AST `id` field is used as-is; otherwise the slug is
///   derived from the heading text via [`heading_slug`].
/// - Duplicate slugs receive `-1`, `-2`, … suffixes in document order.
///
/// **Slug parity with the renderer**: the renderer increments its own slug counter for
/// *every* heading it encounters during the depth-first tree walk — including headings
/// that live inside blockquotes, admonitions, etc.  This function replicates that walk
/// so that the counters stay in sync.  Nested headings are not listed in the output
/// (callers expect a "structural" TOC), but their slugs are still counted so that the
/// numbers assigned to top-level headings match the `id` attributes in the rendered HTML.
pub fn extract_toc(doc: &Document) -> Vec<TocEntry> {
    let mut entries = Vec::new();
    let mut slug_counts: HashMap<String, usize> = HashMap::new();
    walk_toc_nodes(&doc.children, true, &mut entries, &mut slug_counts);
    entries
}

/// Depth-first walk that mirrors the renderer's heading slug counter.
///
/// `include` is `true` only for top-level document children.  At deeper levels it is
/// `false` so those headings affect the counter but are not added to `entries`.
fn walk_toc_nodes(
    nodes: &[crate::parser::Node],
    include: bool,
    entries: &mut Vec<TocEntry>,
    slug_counts: &mut HashMap<String, usize>,
) {
    for node in nodes {
        if let NodeKind::Heading { level, text, id } = &node.kind {
            let base = id
                .as_deref()
                .map(|s| s.to_string())
                .unwrap_or_else(|| heading_slug(text));

            let count = slug_counts.entry(base.clone()).or_insert(0);
            let slug = if *count == 0 {
                base.clone()
            } else {
                format!("{}-{}", base, count)
            };
            *count += 1;

            if include {
                entries.push(TocEntry {
                    level: *level,
                    text: text.clone(),
                    slug,
                    line: node.span.map(|s| s.start.line).unwrap_or(0),
                });
            }
        }

        // Recurse into block containers (blockquotes, admonitions, list items, …).
        // Their headings are counted for slug deduplication but not shown in the TOC.
        if !node.children.is_empty() {
            walk_toc_nodes(&node.children, false, entries, slug_counts);
        }
    }
}

/// Generate a Markdown TOC block wrapped in `<!-- TOC -->` / `<!-- /TOC -->` markers.
///
/// Returns an empty string when `entries` is empty.
/// Indentation is relative to the minimum heading level present.
pub fn generate_toc_markdown(entries: &[TocEntry]) -> String {
    if entries.is_empty() {
        return String::new();
    }

    let min_level = entries.iter().map(|e| e.level).min().unwrap_or(1);
    let mut lines = vec!["<!-- TOC -->".to_string()];

    for entry in entries {
        let indent = "  ".repeat((entry.level - min_level) as usize);
        lines.push(format!("{}- [{}](#{})", indent, entry.text, entry.slug));
    }

    lines.push(String::new()); // blank line terminates the list before the closing marker
    lines.push("<!-- /TOC -->".to_string());
    lines.join("\n")
}

/// Attempt to find and replace an existing `<!-- TOC -->...<!-- /TOC -->` block in
/// `current_text` with `new_toc`.
///
/// Returns:
/// - [`TocReplaceResult::NoMarkers`] when no markers were found
/// - [`TocReplaceResult::NoChange`] when the existing block equals `new_toc`
/// - [`TocReplaceResult::Updated`] with the new full source text otherwise
pub fn replace_toc_in_text(current_text: &str, new_toc: &str) -> TocReplaceResult {
    const START_MARKER: &str = "<!-- TOC -->";
    const END_MARKER: &str = "<!-- /TOC -->";

    let Some(start_pos) = current_text.find(START_MARKER) else {
        return TocReplaceResult::NoMarkers;
    };
    let Some(end_pos) = current_text.find(END_MARKER) else {
        return TocReplaceResult::NoMarkers;
    };

    if end_pos < start_pos {
        return TocReplaceResult::NoMarkers;
    }

    let end_of_block = end_pos + END_MARKER.len();
    let existing = &current_text[start_pos..end_of_block];

    if existing == new_toc {
        return TocReplaceResult::NoChange;
    }

    let mut result = String::with_capacity(current_text.len());
    result.push_str(&current_text[..start_pos]);
    result.push_str(new_toc);
    result.push_str(&current_text[end_of_block..]);
    TocReplaceResult::Updated(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_heading_slug_basic() {
        assert_eq!(heading_slug("Hello World"), "hello-world");
        assert_eq!(heading_slug("Introduction"), "introduction");
        assert_eq!(
            heading_slug("Getting Started Guide"),
            "getting-started-guide"
        );
    }

    #[test]
    fn smoke_heading_slug_special_chars() {
        assert_eq!(heading_slug("Code <example> & test"), "code-example-test");
        assert_eq!(heading_slug("A/B Testing"), "a-b-testing");
        assert_eq!(heading_slug("Hello---World"), "hello-world");
    }

    #[test]
    fn smoke_heading_slug_empty() {
        assert_eq!(heading_slug(""), "");
        assert_eq!(heading_slug("---"), "");
        assert_eq!(heading_slug("!@#"), "");
    }

    #[test]
    fn smoke_extract_toc_basic() {
        use crate::parser::{Document, Node};

        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Heading {
                        level: 1,
                        text: "Title".to_string(),
                        id: None,
                    },
                    span: None,
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Heading {
                        level: 2,
                        text: "Getting Started".to_string(),
                        id: None,
                    },
                    span: None,
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Heading {
                        level: 2,
                        text: "Installation".to_string(),
                        id: None,
                    },
                    span: None,
                    children: vec![],
                },
            ],
            ..Default::default()
        };

        let entries = extract_toc(&doc);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].slug, "title");
        assert_eq!(entries[1].slug, "getting-started");
        assert_eq!(entries[2].slug, "installation");
    }

    #[test]
    fn smoke_extract_toc_explicit_id_wins() {
        use crate::parser::{Document, Node};

        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Heading {
                    level: 2,
                    text: "My Title".to_string(),
                    id: Some("custom-id".to_string()),
                },
                span: None,
                children: vec![],
            }],
            ..Default::default()
        };

        let entries = extract_toc(&doc);
        assert_eq!(entries[0].slug, "custom-id");
    }

    #[test]
    fn smoke_extract_toc_duplicate_slugs() {
        use crate::parser::{Document, Node};

        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Heading {
                        level: 2,
                        text: "Introduction".to_string(),
                        id: None,
                    },
                    span: None,
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Heading {
                        level: 2,
                        text: "Introduction".to_string(),
                        id: None,
                    },
                    span: None,
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Heading {
                        level: 2,
                        text: "Introduction".to_string(),
                        id: None,
                    },
                    span: None,
                    children: vec![],
                },
            ],
            ..Default::default()
        };

        let entries = extract_toc(&doc);
        assert_eq!(entries[0].slug, "introduction");
        assert_eq!(entries[1].slug, "introduction-1");
        assert_eq!(entries[2].slug, "introduction-2");
    }

    #[test]
    fn smoke_generate_toc_markdown_basic() {
        let entries = vec![
            TocEntry {
                level: 1,
                text: "Title".to_string(),
                slug: "title".to_string(),
                line: 0,
            },
            TocEntry {
                level: 2,
                text: "Getting Started".to_string(),
                slug: "getting-started".to_string(),
                line: 0,
            },
            TocEntry {
                level: 3,
                text: "Installation".to_string(),
                slug: "installation".to_string(),
                line: 0,
            },
        ];

        let md = generate_toc_markdown(&entries);
        assert!(md.starts_with("<!-- TOC -->"));
        assert!(md.ends_with("<!-- /TOC -->"));
        assert!(md.contains("- [Title](#title)"));
        assert!(md.contains("  - [Getting Started](#getting-started)"));
        assert!(md.contains("    - [Installation](#installation)"));
    }

    #[test]
    fn smoke_generate_toc_markdown_empty() {
        assert_eq!(generate_toc_markdown(&[]), "");
    }

    #[test]
    fn smoke_replace_toc_no_markers() {
        let text = "# Hello\n\nSome content.\n";
        let toc = "<!-- TOC -->\n- [Hello](#hello)\n<!-- /TOC -->";
        assert!(matches!(
            replace_toc_in_text(text, toc),
            TocReplaceResult::NoMarkers
        ));
    }

    #[test]
    fn smoke_replace_toc_updates_existing() {
        let text = "# Hello\n\n<!-- TOC -->\n- [Old](#old)\n<!-- /TOC -->\n\nContent.\n";
        let new_toc = "<!-- TOC -->\n- [Hello](#hello)\n<!-- /TOC -->";
        match replace_toc_in_text(text, new_toc) {
            TocReplaceResult::Updated(result) => {
                assert!(result.contains("- [Hello](#hello)"));
                assert!(!result.contains("- [Old](#old)"));
                assert!(result.contains("# Hello"));
                assert!(result.contains("Content."));
            }
            other => panic!("expected Updated, got {:?}", other),
        }
    }

    #[test]
    fn smoke_replace_toc_no_change() {
        let toc = "<!-- TOC -->\n- [Hello](#hello)\n<!-- /TOC -->";
        let text = format!("# Hello\n\n{}\n\nContent.\n", toc);
        assert!(matches!(
            replace_toc_in_text(&text, toc),
            TocReplaceResult::NoChange
        ));
    }

    /// A heading inside a blockquote must consume a slug counter slot so that a
    /// same-text heading at the top level receives the correct suffix — matching
    /// the `id` the renderer would assign to it.
    #[test]
    fn smoke_extract_toc_nested_heading_syncs_slug_counter() {
        use crate::parser::{Document, Node};

        // Document structure:
        //   > ## Introduction   ← inside blockquote, NOT in TOC output
        //   ## Introduction     ← top-level, slug must be "introduction-1"
        let blockquote_heading = Node {
            kind: NodeKind::Heading {
                level: 2,
                text: "Introduction".to_string(),
                id: None,
            },
            span: None,
            children: vec![],
        };
        let blockquote_node = Node {
            kind: NodeKind::Blockquote,
            span: None,
            children: vec![blockquote_heading],
        };
        let top_level_heading = Node {
            kind: NodeKind::Heading {
                level: 2,
                text: "Introduction".to_string(),
                id: None,
            },
            span: None,
            children: vec![],
        };

        let doc = Document {
            children: vec![blockquote_node, top_level_heading],
            ..Default::default()
        };

        let entries = extract_toc(&doc);
        // Only the top-level heading is in the TOC.
        assert_eq!(entries.len(), 1);
        // Its slug must be "introduction-1" (counter was consumed by the blockquote heading).
        assert_eq!(entries[0].slug, "introduction-1");
    }
}

#[cfg(test)]
mod parse_roundtrip {
    #[test]
    fn toc_block_renders_as_invisible_html_comments() {
        // Blank line before closing marker is required so the block parser
        // can recognise <!-- /TOC --> as an HTML comment rather than list inline text.
        let input = "<!-- TOC -->\n- [Title](#title)\n  - [Sub](#sub)\n\n<!-- /TOC -->\n";
        let doc = crate::parser::parse(input).expect("parse failed");
        let kinds: Vec<_> = doc
            .children
            .iter()
            .map(|n| format!("{:?}", n.kind))
            .collect();
        eprintln!("Parsed nodes: {:?}", kinds);
        let html = crate::render::render(&doc, &crate::render::RenderOptions::default())
            .expect("render failed");
        eprintln!("HTML output:\n{}", html);
        // Both markers must be invisible HTML comments, not text
        assert!(
            !html.contains("&lt;!"),
            "markers were escaped as text, not passed through as HTML"
        );
    }
}
