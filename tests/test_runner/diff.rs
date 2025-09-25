//! Diff functionality for comparing expected vs actual HTML output
//!
//! This module provides rich diff visualization using the similar crate
//! with colored output and various diff algorithms.

use colored::*;
use similar::{ChangeTag, TextDiff};

/// Configuration for diff display
#[derive(Debug, Clone)]
pub struct DiffConfig {
    /// Whether to use colored output
    pub use_colors: bool,

    /// Number of context lines to show around changes
    pub context_lines: usize,

    /// Whether to show line numbers
    pub show_line_numbers: bool,
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            use_colors: true,
            context_lines: 3,
            show_line_numbers: true,
        }
    }
}

/// Create a unified diff string from expected and actual HTML
pub fn create_unified_diff(expected: &str, actual: &str, config: &DiffConfig) -> String {
    let diff = TextDiff::from_lines(expected, actual);
    let mut output = Vec::new();

    // Add header
    if config.use_colors {
        output.push(format!("{}", "--- Expected".red().bold()));
        output.push(format!("{}", "+++ Actual".green().bold()));
    } else {
        output.push("--- Expected".to_string());
        output.push("+++ Actual".to_string());
    }
    output.push(String::new()); // Empty line

    for group in diff.grouped_ops(config.context_lines) {
        if !output.is_empty() && !output.last().unwrap().is_empty() {
            output.push(String::new()); // Separator between hunks
        }

        // Calculate hunk header
        let first_op = group.first().unwrap();
        let last_op = group.last().unwrap();

        let old_start = first_op.old_range().start + 1;
        let old_end = last_op.old_range().end;
        let new_start = first_op.new_range().start + 1;
        let new_end = last_op.new_range().end;

        // Prevent underflow
        let old_length = old_end.saturating_sub(old_start.saturating_sub(1));
        let new_length = new_end.saturating_sub(new_start.saturating_sub(1));

        let hunk_header = if config.use_colors {
            format!(
                "@@ -{},{} +{},{} @@",
                old_start, old_length, new_start, new_length
            )
            .cyan()
            .to_string()
        } else {
            format!(
                "@@ -{},{} +{},{} @@",
                old_start, old_length, new_start, new_length
            )
        };
        output.push(hunk_header);

        // Reset line numbers for this hunk
        let mut line_num_old = old_start;
        let mut line_num_new = new_start;

        for op in group {
            for change in diff.iter_changes(&op) {
                let line_content = format!("{}", change);

                match change.tag() {
                    ChangeTag::Delete => {
                        let prefix = if config.show_line_numbers {
                            format!("{:4} -", line_num_old)
                        } else {
                            "-".to_string()
                        };

                        let formatted_line = if config.use_colors {
                            format!("{} {}", prefix.red(), line_content.red())
                        } else {
                            format!("{} {}", prefix, line_content)
                        };
                        output.push(formatted_line);
                        line_num_old += 1;
                    }
                    ChangeTag::Insert => {
                        let prefix = if config.show_line_numbers {
                            format!("{:4} +", line_num_new)
                        } else {
                            "+".to_string()
                        };

                        let formatted_line = if config.use_colors {
                            format!("{} {}", prefix.green(), line_content.green())
                        } else {
                            format!("{} {}", prefix, line_content)
                        };
                        output.push(formatted_line);
                        line_num_new += 1;
                    }
                    ChangeTag::Equal => {
                        let prefix = if config.show_line_numbers {
                            format!("{:4}  ", line_num_old)
                        } else {
                            " ".to_string()
                        };
                        output.push(format!("{} {}", prefix, line_content));
                        line_num_old += 1;
                        line_num_new += 1;
                    }
                }
            }
        }
    }

    output.join("\n")
}

/// Create a side-by-side diff view
pub fn create_side_by_side_diff(expected: &str, actual: &str, config: &DiffConfig) -> String {
    let diff = TextDiff::from_lines(expected, actual);
    let mut output = Vec::new();

    // Header
    let header = if config.use_colors {
        format!(
            "{:40} | {}",
            "Expected".red().bold(),
            "Actual".green().bold()
        )
    } else {
        format!("{:40} | {}", "Expected", "Actual")
    };
    output.push(header);
    output.push("-".repeat(80));

    for change in diff.iter_all_changes() {
        let line = format!("{}", change).trim_end().to_string();

        match change.tag() {
            ChangeTag::Delete => {
                let formatted = if config.use_colors {
                    format!("{:40} | {}", line.red(), "")
                } else {
                    format!("{:40} | - {}", line, "")
                };
                output.push(formatted);
            }
            ChangeTag::Insert => {
                let formatted = if config.use_colors {
                    format!("{:40} | {}", "", line.green())
                } else {
                    format!("{:40} | + {}", "", line)
                };
                output.push(formatted);
            }
            ChangeTag::Equal => {
                output.push(format!("{:40} | {}", line, line));
            }
        }
    }

    output.join("\n")
}

/// Create a compact diff showing only changes
#[allow(dead_code)]
pub fn create_compact_diff(expected: &str, actual: &str, config: &DiffConfig) -> String {
    let diff = TextDiff::from_lines(expected, actual);
    let mut output = Vec::new();

    let mut has_changes = false;

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Delete => {
                has_changes = true;
                let formatted = format!("{}", change);
                let line = formatted.trim_end();
                let formatted = if config.use_colors {
                    format!("- {}", line.red())
                } else {
                    format!("- {}", line)
                };
                output.push(formatted);
            }
            ChangeTag::Insert => {
                has_changes = true;
                let formatted = format!("{}", change);
                let line = formatted.trim_end();
                let formatted = if config.use_colors {
                    format!("+ {}", line.green())
                } else {
                    format!("+ {}", line)
                };
                output.push(formatted);
            }
            ChangeTag::Equal => {
                // Skip unchanged lines in compact mode
            }
        }
    }

    if !has_changes {
        output.push("No differences found".to_string());
    }

    output.join("\n")
}

/// Get similarity ratio between two texts (0.0 = completely different, 1.0 = identical)
pub fn similarity_ratio(expected: &str, actual: &str) -> f64 {
    let diff = TextDiff::from_chars(expected, actual);
    diff.ratio() as f64
}

/// Check if two HTML strings are similar within a given threshold
#[allow(dead_code)]
pub fn is_similar(expected: &str, actual: &str, threshold: f64) -> bool {
    similarity_ratio(expected, actual) >= threshold
}

/// Statistics about a diff
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DiffStats {
    pub total_lines: usize,
    pub added_lines: usize,
    pub removed_lines: usize,
    pub unchanged_lines: usize,
    pub similarity_ratio: f64,
}

/// Calculate diff statistics
pub fn calculate_diff_stats(expected: &str, actual: &str) -> DiffStats {
    let diff = TextDiff::from_lines(expected, actual);
    let mut added = 0;
    let mut removed = 0;
    let mut unchanged = 0;

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Delete => removed += 1,
            ChangeTag::Insert => added += 1,
            ChangeTag::Equal => unchanged += 1,
        }
    }

    DiffStats {
        total_lines: added + removed + unchanged,
        added_lines: added,
        removed_lines: removed,
        unchanged_lines: unchanged,
        similarity_ratio: similarity_ratio(expected, actual),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity_ratio() {
        // Identical strings
        assert_eq!(similarity_ratio("hello", "hello"), 1.0);

        // Completely different strings
        assert!(similarity_ratio("hello", "world") < 1.0);

        // Similar strings
        let ratio = similarity_ratio("hello world", "hello word");
        assert!(ratio > 0.5 && ratio < 1.0);
    }

    #[test]
    fn test_is_similar() {
        assert!(is_similar("hello", "hello", 1.0));
        assert!(is_similar("hello world", "hello word", 0.8));
        assert!(!is_similar("hello", "goodbye", 0.9));
    }

    #[test]
    fn test_diff_stats() {
        let expected = "line1\nline2\nline3";
        let actual = "line1\nmodified line2\nline3\nline4";

        let stats = calculate_diff_stats(expected, actual);

        // Just check that it's calculating something reasonable
        assert!(stats.unchanged_lines > 0); // should have some unchanged lines
        assert!(stats.added_lines > 0); // should have some added lines
        assert!(stats.similarity_ratio > 0.0 && stats.similarity_ratio <= 1.0);
        assert_eq!(
            stats.total_lines,
            stats.unchanged_lines + stats.removed_lines + stats.added_lines
        );
    }

    #[test]
    fn test_create_compact_diff() {
        let expected = "line1\nline2\nline3";
        let actual = "line1\nmodified line2\nline3";

        let config = DiffConfig {
            use_colors: false,
            context_lines: 3,
            show_line_numbers: true,
        };

        let diff = create_compact_diff(expected, actual, &config);

        assert!(diff.contains("- line2"));
        assert!(diff.contains("+ modified line2"));
        assert!(!diff.contains("line1")); // Unchanged lines should not appear
    }

    #[test]
    fn test_create_unified_diff() {
        let expected = "line1\nline2\nline3";
        let actual = "line1\nmodified line2\nline3";

        let config = DiffConfig {
            use_colors: false,
            context_lines: 1,
            show_line_numbers: false,
        };

        let diff = create_unified_diff(expected, actual, &config);

        assert!(diff.contains("--- Expected"));
        assert!(diff.contains("+++ Actual"));
        assert!(diff.contains("- line2"));
        assert!(diff.contains("+ modified line2"));
    }

    #[test]
    fn test_no_diff() {
        let text = "same content";

        let config = DiffConfig {
            use_colors: false,
            context_lines: 3,
            show_line_numbers: true,
        };

        let compact_diff = create_compact_diff(text, text, &config);
        assert!(compact_diff.contains("No differences found"));

        let stats = calculate_diff_stats(text, text);
        assert_eq!(stats.similarity_ratio, 1.0);
        assert_eq!(stats.added_lines, 0);
        assert_eq!(stats.removed_lines, 0);
    }
}
