// Autocomplete suggestions: Markdown syntax, image paths, link URLs

use crate::parser::Position;

#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionKind,
    pub insert_text: String,
}

#[derive(Debug, Clone)]
pub enum CompletionKind {
    Syntax,
    FilePath,
    LinkUrl,
}

// Provide completion suggestions at position
pub fn get_completions(position: Position, context: &str) -> Vec<CompletionItem> {
    log::debug!("Computing completions at {:?}", position);

    let mut completions = Vec::new();

    // Get the line where cursor is positioned
    let lines: Vec<&str> = if context.is_empty() {
        vec![""]
    } else {
        context.lines().collect()
    };

    if position.line >= lines.len() {
        log::warn!(
            "Position line {} out of range (total lines: {})",
            position.line,
            lines.len()
        );
        return completions;
    }

    let current_line = lines[position.line];
    let cursor_col = position.column;

    // Ensure cursor column is within line bounds
    if cursor_col > current_line.len() {
        log::warn!(
            "Cursor column {} out of range for line length {}",
            cursor_col,
            current_line.len()
        );
        return completions;
    }

    let before_cursor = &current_line[..cursor_col];

    // Check context and provide appropriate completions

    // 1. Line start: suggest headings, code blocks, lists, blockquotes, and thematic breaks
    if before_cursor.trim().is_empty() {
        add_heading_completions(&mut completions);
        add_code_block_completions(&mut completions);
        add_list_completions(&mut completions);
        add_blockquote_completions(&mut completions);
        add_thematic_break_completions(&mut completions);
    }

    // 1a. After list marker indentation: suggest nested list items
    let trimmed = before_cursor.trim_start();
    if trimmed.is_empty() && !before_cursor.is_empty() {
        // We're at indented position, suggest list items
        add_list_completions(&mut completions);
    }

    // 1b. After '>': suggest blockquote continuation
    if before_cursor.trim_start().starts_with('>')
        && before_cursor.trim_end()
            == before_cursor
                .trim_start()
                .trim_start_matches('>')
                .trim_start()
    {
        add_blockquote_continuation_completions(&mut completions);
    }

    // 2. After '#' at line start: suggest more heading levels
    if let Some(stripped) = before_cursor.trim_start().strip_prefix('#') {
        let hash_count = stripped.chars().take_while(|&c| c == '#').count() + 1;
        if hash_count < 6 && !stripped.chars().any(|c| c != '#') {
            add_heading_level_completions(&mut completions, hash_count);
        }
    }

    // 3. After opening bracket: suggest link syntax
    if before_cursor.ends_with('[') && !before_cursor.ends_with("\\[") {
        add_link_completions(&mut completions);
    }

    // 3a. After '![': suggest image syntax
    if before_cursor.ends_with("![") {
        add_image_completions(&mut completions);
    }

    // 3b. After '<': suggest autolink syntax
    if before_cursor.ends_with('<') && !before_cursor.ends_with("\\<") {
        add_autolink_completions(&mut completions);
    }

    // 3c. After '&': suggest entity references
    if before_cursor.ends_with('&') && !before_cursor.ends_with("\\&") {
        add_entity_reference_completions(&mut completions);
    }

    // 4. After backtick: suggest code span
    if before_cursor.ends_with('`') && !before_cursor.ends_with("\\`") {
        let backtick_count = before_cursor
            .chars()
            .rev()
            .take_while(|&c| c == '`')
            .count();
        if backtick_count == 1 || backtick_count == 3 {
            add_code_span_completions(&mut completions, backtick_count);
        }
    }

    // 5. After asterisk or underscore: suggest emphasis
    if before_cursor.ends_with('*') && !before_cursor.ends_with("\\*") {
        let star_count = before_cursor
            .chars()
            .rev()
            .take_while(|&c| c == '*')
            .count();
        if star_count <= 2 {
            add_emphasis_completions(&mut completions, '*', star_count);
        }
    }

    if before_cursor.ends_with('_') && !before_cursor.ends_with("\\_") {
        let underscore_count = before_cursor
            .chars()
            .rev()
            .take_while(|&c| c == '_')
            .count();
        if underscore_count <= 2 {
            add_emphasis_completions(&mut completions, '_', underscore_count);
        }
    }

    // 6. Inside link text (between [ and ]): suggest closing and URL
    if let Some(bracket_pos) = before_cursor.rfind('[') {
        if !before_cursor[bracket_pos..].contains(']') {
            add_link_url_completions(&mut completions);
        }
    }

    // 7. At end of line with text: suggest line break (but not if line ends with backslash)
    if !before_cursor.trim().is_empty()
        && cursor_col == current_line.len()
        && !before_cursor.ends_with('\\')
    {
        add_line_break_completions(&mut completions);
    }

    log::info!("Generated {} completion items", completions.len());
    completions
}

// Helper functions for generating completion items

fn add_heading_completions(completions: &mut Vec<CompletionItem>) {
    for level in 1..=6 {
        let hashes = "#".repeat(level);
        completions.push(CompletionItem {
            label: format!("Heading {}", level),
            kind: CompletionKind::Syntax,
            insert_text: format!("{} ", hashes),
        });
    }
}

fn add_heading_level_completions(completions: &mut Vec<CompletionItem>, current_level: usize) {
    if current_level < 6 {
        completions.push(CompletionItem {
            label: format!("Continue to Heading {}", current_level + 1),
            kind: CompletionKind::Syntax,
            insert_text: "# ".to_string(),
        });
    }
}

fn add_code_block_completions(completions: &mut Vec<CompletionItem>) {
    // Common languages for code blocks
    let languages = vec![
        "rust",
        "python",
        "javascript",
        "typescript",
        "java",
        "c",
        "cpp",
        "go",
        "bash",
        "shell",
        "json",
        "yaml",
        "toml",
        "html",
        "css",
        "sql",
    ];

    for lang in languages {
        completions.push(CompletionItem {
            label: format!("Code Block ({})", lang),
            kind: CompletionKind::Syntax,
            insert_text: format!("```{}\n\n```", lang),
        });
    }

    // Generic code block
    completions.push(CompletionItem {
        label: "Code Block (no language)".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: "```\n\n```".to_string(),
    });
}

fn add_link_completions(completions: &mut Vec<CompletionItem>) {
    completions.push(CompletionItem {
        label: "Link".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: "text](url)".to_string(),
    });

    completions.push(CompletionItem {
        label: "Link with title".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: "text](url \"title\")".to_string(),
    });
}

fn add_image_completions(completions: &mut Vec<CompletionItem>) {
    completions.push(CompletionItem {
        label: "Image".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: "alt text](image.png)".to_string(),
    });

    completions.push(CompletionItem {
        label: "Image with title".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: "alt text](image.png \"title\")".to_string(),
    });
}

fn add_autolink_completions(completions: &mut Vec<CompletionItem>) {
    completions.push(CompletionItem {
        label: "Autolink (URL)".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: "https://example.com>".to_string(),
    });

    completions.push(CompletionItem {
        label: "Autolink (Email)".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: "user@example.com>".to_string(),
    });
}

fn add_code_span_completions(completions: &mut Vec<CompletionItem>, backtick_count: usize) {
    let backticks = "`".repeat(backtick_count);
    completions.push(CompletionItem {
        label: "Code Span".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: format!("code{}", backticks),
    });
}

fn add_emphasis_completions(completions: &mut Vec<CompletionItem>, delimiter: char, count: usize) {
    if count == 1 {
        completions.push(CompletionItem {
            label: "Emphasis (italic)".to_string(),
            kind: CompletionKind::Syntax,
            insert_text: format!("text{}", delimiter),
        });
    } else if count == 2 {
        completions.push(CompletionItem {
            label: "Strong (bold)".to_string(),
            kind: CompletionKind::Syntax,
            insert_text: format!("text{}{}", delimiter, delimiter),
        });
    }
}

fn add_link_url_completions(completions: &mut Vec<CompletionItem>) {
    completions.push(CompletionItem {
        label: "Complete link".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: "](url)".to_string(),
    });

    completions.push(CompletionItem {
        label: "Complete link with title".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: "](url \"title\")".to_string(),
    });
}

fn add_line_break_completions(completions: &mut Vec<CompletionItem>) {
    completions.push(CompletionItem {
        label: "Hard line break (two spaces)".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: "  \n".to_string(),
    });

    completions.push(CompletionItem {
        label: "Hard line break (backslash)".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: "\\\n".to_string(),
    });
}

fn add_list_completions(completions: &mut Vec<CompletionItem>) {
    // Unordered list markers
    completions.push(CompletionItem {
        label: "Unordered list item (-)".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: "- ".to_string(),
    });

    completions.push(CompletionItem {
        label: "Unordered list item (*)".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: "* ".to_string(),
    });

    completions.push(CompletionItem {
        label: "Unordered list item (+)".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: "+ ".to_string(),
    });

    // Ordered list
    completions.push(CompletionItem {
        label: "Ordered list item".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: "1. ".to_string(),
    });
}

fn add_blockquote_completions(completions: &mut Vec<CompletionItem>) {
    completions.push(CompletionItem {
        label: "Block quote".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: "> ".to_string(),
    });

    completions.push(CompletionItem {
        label: "Nested block quote".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: "> > ".to_string(),
    });
}

fn add_blockquote_continuation_completions(completions: &mut Vec<CompletionItem>) {
    completions.push(CompletionItem {
        label: "Continue block quote".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: "\n> ".to_string(),
    });
}

fn add_thematic_break_completions(completions: &mut Vec<CompletionItem>) {
    completions.push(CompletionItem {
        label: "Thematic break (---)".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: "---".to_string(),
    });

    completions.push(CompletionItem {
        label: "Thematic break (***)".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: "***".to_string(),
    });

    completions.push(CompletionItem {
        label: "Thematic break (___)".to_string(),
        kind: CompletionKind::Syntax,
        insert_text: "___".to_string(),
    });
}

fn add_entity_reference_completions(completions: &mut Vec<CompletionItem>) {
    // Common HTML entities
    let entities = vec![
        ("&amp;", "Ampersand (&)"),
        ("&lt;", "Less than (<)"),
        ("&gt;", "Greater than (>)"),
        ("&quot;", "Double quote (\")"),
        ("&apos;", "Apostrophe (')"),
        ("&nbsp;", "Non-breaking space"),
        ("&copy;", "Copyright (©)"),
        ("&reg;", "Registered (®)"),
        ("&trade;", "Trademark (™)"),
        ("&euro;", "Euro (€)"),
        ("&pound;", "Pound (£)"),
        ("&yen;", "Yen (¥)"),
        ("&cent;", "Cent (¢)"),
        ("&sect;", "Section (§)"),
        ("&para;", "Paragraph (¶)"),
        ("&dagger;", "Dagger (†)"),
        ("&Dagger;", "Double dagger (‡)"),
        ("&bull;", "Bullet (•)"),
        ("&hellip;", "Ellipsis (…)"),
        ("&mdash;", "Em dash (—)"),
        ("&ndash;", "En dash (–)"),
    ];

    for (entity, description) in entities {
        completions.push(CompletionItem {
            label: format!("{} - {}", entity, description),
            kind: CompletionKind::Syntax,
            insert_text: entity[1..].to_string(), // Remove leading '&' since it's already typed
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: Tests below construct `Position` values like `Position::new(0, 0, 0)`.
    // These uses are test-only conveniences where the `Position` acts as a
    // zero-based cursor index into an in-memory `Vec<&str>` (see `get_completions`).
    //
    // Important: elsewhere in the parser/LSP pipeline `Position` may follow a
    // different convention (for example 1-based line/column semantics). These
    // tests intentionally use 0-based positions to avoid needing to populate
    // full parser-style `Document` structures. When wiring `get_completions`
    // into real editor/LSP code, ensure you convert between the editor's
    // cursor coordinates and the parser/LSP `Position` convention as needed.

    #[test]
    fn smoke_test_heading_completions() {
        let context = "";
        let position = Position::new(0, 0, 0);

        let completions = get_completions(position, context);

        // Should suggest headings at line start
        assert!(
            completions.len() >= 6,
            "Should suggest at least 6 heading levels"
        );
        assert!(completions.iter().any(|c| c.label.contains("Heading 1")));
        assert!(completions.iter().any(|c| c.label.contains("Heading 6")));
    }

    #[test]
    fn smoke_test_link_completions() {
        let context = "Some text [";
        let position = Position::new(0, 11, 11); // After '['

        let completions = get_completions(position, context);

        // Should suggest link syntax
        assert!(completions.iter().any(|c| c.label.contains("Link")));
        assert!(completions.iter().any(|c| c.insert_text.contains("](url)")));
    }

    #[test]
    fn smoke_test_code_span_completions() {
        let context = "Some text `";
        let position = Position::new(0, 11, 11); // After '`'

        let completions = get_completions(position, context);

        // Should suggest code span
        assert!(completions.iter().any(|c| c.label == "Code Span"));
        assert!(completions.iter().any(|c| c.insert_text.contains("code`")));
    }

    #[test]
    fn smoke_test_emphasis_completions() {
        let context = "Some text *";
        let position = Position::new(0, 11, 11); // After '*'

        let completions = get_completions(position, context);

        // Should suggest emphasis
        assert!(completions.iter().any(|c| c.label.contains("italic")));
    }

    #[test]
    fn smoke_test_strong_completions() {
        let context = "Some text **";
        let position = Position::new(0, 12, 12); // After '**'

        let completions = get_completions(position, context);

        // Should suggest strong
        assert!(completions.iter().any(|c| c.label.contains("bold")));
    }

    #[test]
    fn smoke_test_code_block_completions() {
        let context = "";
        let position = Position::new(0, 0, 0);

        let completions = get_completions(position, context);

        // Should suggest code blocks at line start
        assert!(completions.iter().any(|c| c.label.contains("Code Block")));
        assert!(completions.iter().any(|c| c.label.contains("rust")));
        assert!(completions.iter().any(|c| c.label.contains("python")));
    }

    #[test]
    fn smoke_test_no_completions_mid_word() {
        let context = "Some text here";
        let position = Position::new(0, 7, 7); // Middle of "text"

        let completions = get_completions(position, context);

        // Should not suggest completions in middle of word
        assert_eq!(
            completions.len(),
            0,
            "Should not suggest completions mid-word"
        );
    }

    #[test]
    fn smoke_test_escaped_delimiters() {
        let context = "Some text \\*";
        let position = Position::new(0, 12, 12); // After '\*'

        let completions = get_completions(position, context);

        // Should not suggest emphasis for escaped asterisk (but may suggest other things like line breaks)
        assert!(
            !completions.iter().any(|c| c.label.contains("italic")),
            "Should not suggest emphasis for escaped asterisk"
        );
        assert!(
            !completions.iter().any(|c| c.label.contains("bold")),
            "Should not suggest strong for escaped asterisk"
        );
    }

    #[test]
    fn smoke_test_image_completions() {
        let context = "Some text ![";
        let position = Position::new(0, 12, 12); // After '!['

        let completions = get_completions(position, context);

        // Should suggest image syntax
        assert!(completions.iter().any(|c| c.label.contains("Image")));
        assert!(completions
            .iter()
            .any(|c| c.insert_text.contains("](image")));
    }

    #[test]
    fn smoke_test_autolink_completions() {
        let context = "Some text <";
        let position = Position::new(0, 11, 11); // After '<'

        let completions = get_completions(position, context);

        // Should suggest autolink syntax
        assert!(completions
            .iter()
            .any(|c| c.label.contains("Autolink (URL)")));
        assert!(completions
            .iter()
            .any(|c| c.label.contains("Autolink (Email)")));
        assert!(completions
            .iter()
            .any(|c| c.insert_text.contains("https://")));
        assert!(completions.iter().any(|c| c.insert_text.contains("@")));
    }

    #[test]
    fn smoke_test_line_break_completions() {
        let context = "Some text at end of line";
        let position = Position::new(0, 24, 24); // At end of line

        let completions = get_completions(position, context);

        // Should suggest line break options
        assert!(completions
            .iter()
            .any(|c| c.label.contains("Hard line break")));
        assert!(completions.iter().any(|c| c.insert_text.contains("  \n")));
        assert!(completions.iter().any(|c| c.insert_text.contains("\\\n")));
    }

    #[test]
    fn smoke_test_list_completions() {
        let context = "";
        let position = Position::new(0, 0, 0); // At line start

        let completions = get_completions(position, context);

        // Should suggest list items at line start
        assert!(completions
            .iter()
            .any(|c| c.label.contains("Unordered list item (-)")));
        assert!(completions
            .iter()
            .any(|c| c.label.contains("Unordered list item (*)")));
        assert!(completions
            .iter()
            .any(|c| c.label.contains("Unordered list item (+)")));
        assert!(completions
            .iter()
            .any(|c| c.label.contains("Ordered list item")));
        assert!(completions.iter().any(|c| c.insert_text == "- "));
        assert!(completions.iter().any(|c| c.insert_text == "1. "));
    }

    #[test]
    fn smoke_test_blockquote_completions() {
        let context = "";
        let position = Position::new(0, 0, 0); // At line start

        let completions = get_completions(position, context);

        // Should suggest block quote at line start
        assert!(completions.iter().any(|c| c.label.contains("Block quote")));
        assert!(completions.iter().any(|c| c.insert_text == "> "));
    }

    #[test]
    fn smoke_test_thematic_break_completions() {
        let context = "";
        let position = Position::new(0, 0, 0); // At line start

        let completions = get_completions(position, context);

        // Should suggest thematic breaks at line start
        assert!(completions
            .iter()
            .any(|c| c.label.contains("Thematic break (---)")));
        assert!(completions
            .iter()
            .any(|c| c.label.contains("Thematic break (***)")));
        assert!(completions
            .iter()
            .any(|c| c.label.contains("Thematic break (___)")));
        assert!(completions.iter().any(|c| c.insert_text == "---"));
    }

    #[test]
    fn smoke_test_entity_reference_completions() {
        let context = "Some text &";
        let position = Position::new(0, 11, 11); // After '&'

        let completions = get_completions(position, context);

        // Should suggest HTML entities
        assert!(completions.iter().any(|c| c.label.contains("&amp;")));
        assert!(completions.iter().any(|c| c.label.contains("&lt;")));
        assert!(completions.iter().any(|c| c.label.contains("&gt;")));
        assert!(completions.iter().any(|c| c.label.contains("&quot;")));
        assert!(completions.iter().any(|c| c.label.contains("&copy;")));
        // Insert text should not include the leading & (already typed)
        assert!(completions.iter().any(|c| c.insert_text == "amp;"));
        assert!(completions.iter().any(|c| c.insert_text == "lt;"));
    }
}
