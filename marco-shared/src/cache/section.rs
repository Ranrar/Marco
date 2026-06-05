//! Document section splitter for section-level caching.
//!
//! Splits a markdown document into [`DocumentSection`]s so that on each
//! keystroke debounce only the section under the cursor needs to be
//! re-rendered. All other sections are served from the HTML cache as-is.
//!
//! Splitting rules (in priority order):
//! 1. Any H1–H3 heading line starts a new section
//! 2. A section covers lines from one heading to the line before the next
//! 3. If no headings exist, split every 50 non-blank lines
//! 4. Minimum section size: 3 lines (avoids degenerate tiny sections by
//!    merging them into the preceding section)

use crate::cache::hash_content;

/// A single section of a markdown document.
#[derive(Debug, Clone)]
pub struct DocumentSection {
    /// 0-based index in the section list.
    pub index: usize,
    /// First line of this section (0-based, inclusive).
    pub start_line: usize,
    /// Last line of this section (0-based, inclusive).
    pub end_line: usize,
    /// The raw markdown text of this section.
    pub content: String,
    /// blake3 hash of `content` (truncated to u64).
    pub content_hash: u64,
}

impl DocumentSection {
    fn new(index: usize, start_line: usize, end_line: usize, content: String) -> Self {
        let content_hash = hash_content(&content);
        Self {
            index,
            start_line,
            end_line,
            content,
            content_hash,
        }
    }
}

/// Returns `true` if `line` starts an H1–H3 heading.
fn is_heading_boundary(line: &str) -> bool {
    let t = line.trim_start();
    matches!(
        t.split_once(' ').map(|(marker, _)| marker),
        Some("#" | "##" | "###")
    )
}

/// Split `text` into sections for section-level caching.
///
/// Pure function: no I/O, no global state.  Safe to call from any thread.
pub fn split_into_sections(text: &str) -> Vec<DocumentSection> {
    let lines: Vec<&str> = text.lines().collect();
    let total = lines.len();

    if total == 0 {
        return vec![DocumentSection::new(0, 0, 0, String::new())];
    }

    // Collect boundary line indices (lines that START a new section).
    let mut boundaries: Vec<usize> = Vec::new();
    boundaries.push(0); // document always starts a section

    // Check whether the document has any H1–H3 headings outside code fences.
    let has_headings = {
        let mut inside = false;
        let mut fc = ' ';
        lines.iter().any(|l| {
            let t = l.trim_start();
            if inside {
                if (fc == '`' && t.starts_with("```")) || (fc == '~' && t.starts_with("~~~")) {
                    inside = false;
                }
                false
            } else if t.starts_with("```") {
                inside = true;
                fc = '`';
                false
            } else if t.starts_with("~~~") {
                inside = true;
                fc = '~';
                false
            } else {
                is_heading_boundary(l)
            }
        })
    };

    if has_headings {
        let mut in_fence = false;
        let mut fence_char = ' '; // '`' or '~'
        for (i, line) in lines.iter().enumerate() {
            let t = line.trim_start();
            // Track fenced code block open/close to avoid treating heading-like
            // lines inside code blocks (e.g. `## comment`) as section boundaries.
            if in_fence {
                if (fence_char == '`' && t.starts_with("```"))
                    || (fence_char == '~' && t.starts_with("~~~"))
                {
                    in_fence = false;
                }
                continue;
            }
            if t.starts_with("```") {
                in_fence = true;
                fence_char = '`';
                continue;
            }
            if t.starts_with("~~~") {
                in_fence = true;
                fence_char = '~';
                continue;
            }
            if i > 0 && is_heading_boundary(line) {
                boundaries.push(i);
            }
        }
    } else {
        // No headings: split every 50 non-blank lines
        let mut non_blank = 0usize;
        for (i, line) in lines.iter().enumerate() {
            if i == 0 {
                continue;
            }
            if !line.trim().is_empty() {
                non_blank += 1;
            }
            if non_blank > 0 && non_blank.is_multiple_of(50) {
                boundaries.push(i);
            }
        }
    }

    // Build raw sections from boundaries
    let raw: Vec<(usize, usize)> = boundaries
        .windows(2)
        .map(|w| (w[0], w[1] - 1))
        .chain(std::iter::once((*boundaries.last().unwrap(), total - 1)))
        .collect();

    // Merge sections that are shorter than 3 lines into the preceding one
    let mut merged: Vec<(usize, usize)> = Vec::with_capacity(raw.len());
    for &(start, end) in &raw {
        let len = end + 1 - start;
        if len < 3 {
            if let Some(last) = merged.last_mut() {
                last.1 = end; // extend previous section
                continue;
            }
        }
        merged.push((start, end));
    }

    merged
        .into_iter()
        .enumerate()
        .map(|(idx, (start, end))| {
            let content = lines[start..=end].join("\n");
            DocumentSection::new(idx, start, end, content)
        })
        .collect()
}

/// Return the index of the section containing `cursor_line` (0-based).
/// Falls back to 0 if the cursor is somehow outside all sections.
pub fn section_for_line(sections: &[DocumentSection], cursor_line: usize) -> usize {
    sections
        .iter()
        .position(|s| cursor_line >= s.start_line && cursor_line <= s.end_line)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Smoke tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc(headings: usize) -> String {
        (0..headings)
            .map(|i| format!("## Section {i}\n\nsome content here\n\n"))
            .collect()
    }

    #[test]
    fn smoke_empty_doc_gives_one_section() {
        let sections = split_into_sections("");
        assert_eq!(sections.len(), 1);
    }

    #[test]
    fn smoke_heading_boundaries_are_respected() {
        let doc = "# A\n\nParagraph.\n\n## B\n\nAnother.\n\n## C\n\nThird.";
        let sections = split_into_sections(doc);
        assert_eq!(sections.len(), 3, "expected 3 sections for 3 headings");
        assert_eq!(sections[0].index, 0);
        assert!(sections[0].content.starts_with("# A"));
        assert!(sections[1].content.starts_with("## B"));
        assert!(sections[2].content.starts_with("## C"));
    }

    #[test]
    fn smoke_section_line_ranges_are_contiguous() {
        let doc = make_doc(5);
        let sections = split_into_sections(&doc);
        for pair in sections.windows(2) {
            assert_eq!(
                pair[0].end_line + 1,
                pair[1].start_line,
                "gap between sections {} and {}",
                pair[0].index,
                pair[1].index
            );
        }
    }

    #[test]
    fn smoke_no_headings_splits_by_paragraph_count() {
        // 120 non-blank lines → should produce at least 2 sections
        let doc: String = (0..120).map(|i| format!("Line {i}\n")).collect();
        let sections = split_into_sections(&doc);
        assert!(
            sections.len() >= 2,
            "expected multiple sections for 120 lines, got {}",
            sections.len()
        );
    }

    #[test]
    fn smoke_section_for_line_finds_correct_section() {
        let doc = "# A\n\nContent A.\n\n## B\n\nContent B.\n\n## C\n\nContent C.";
        let sections = split_into_sections(doc);
        // The heading of section 1 (## B) is line 4 (0-based)
        let sec_b = section_for_line(&sections, 4);
        assert_eq!(sec_b, 1, "cursor on '## B' line should map to section 1");
    }

    #[test]
    fn smoke_content_hash_changes_when_content_changes() {
        let doc = "## Foo\n\nOriginal content.";
        let sections_a = split_into_sections(doc);
        let doc2 = "## Foo\n\nModified content.";
        let sections_b = split_into_sections(doc2);
        assert_ne!(
            sections_a[0].content_hash, sections_b[0].content_hash,
            "content hash must differ when text changes"
        );
    }

    #[test]
    #[ignore = "wall-clock timing; run explicitly with --ignored on capable hardware"]
    fn smoke_stresstest_scale() {
        // Simulate a 1250-heading document and verify splitting is fast
        let doc = make_doc(1250);
        let start = std::time::Instant::now();
        let sections = split_into_sections(&doc);
        let elapsed = start.elapsed();
        assert_eq!(sections.len(), 1250);
        assert!(
            elapsed.as_millis() < 100,
            "splitting 1250 sections took {}ms, expected <100ms",
            elapsed.as_millis()
        );
    }
}
