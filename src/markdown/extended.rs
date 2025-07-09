use gtk4::prelude::*;
use regex::Regex;
use std::collections::HashMap;

/// Extended Markdown syntax highlighting based on https://www.markdownguide.org/extended-syntax/
/// Includes tables, task lists, strikethrough, footnotes, definition lists, and more
pub struct ExtendedMarkdownSyntax {
    /// Tables (GitHub Flavored Markdown style)
    table_regex: Regex,
    table_header_separator_regex: Regex,

    /// Task Lists (GitHub style checkboxes)
    task_list_regex: Regex,

    /// Strikethrough (already in basic but included for completeness)
    strikethrough_regex: Regex,

    /// Footnotes
    footnote_reference_regex: Regex,
    footnote_definition_regex: Regex,

    /// Definition Lists
    definition_term_regex: Regex,
    definition_description_regex: Regex,

    /// Fenced Code Blocks with syntax highlighting
    fenced_code_regex: Regex,

    /// Heading IDs
    heading_with_id_regex: Regex,

    /// Highlight (==text==)
    highlight_regex: Regex,

    /// Subscript (~text~)
    subscript_regex: Regex,

    /// Superscript (^text^)
    superscript_regex: Regex,

    /// Automatic URL linking
    auto_url_regex: Regex,

    /// Emoji shortcodes (:emoji:)
    emoji_shortcode_regex: Regex,
}

impl ExtendedMarkdownSyntax {
    pub fn new() -> Self {
        Self {
            // Tables: | Header 1 | Header 2 |
            table_regex: Regex::new(r"^\|.*\|$").unwrap(),

            // Table header separator: |----------|----------|
            table_header_separator_regex: Regex::new(
                r"^\|[\s]*:?-+:?[\s]*(\|[\s]*:?-+:?[\s]*)*\|?$",
            )
            .unwrap(),

            // Task Lists: - [x] Task or - [ ] Task
            task_list_regex: Regex::new(r"^[\s]*[-*+][\s]+\[([ xX])\][\s]+(.*)$").unwrap(),

            // Strikethrough: ~~text~~
            strikethrough_regex: Regex::new(r"~~([^~]+)~~").unwrap(),

            // Footnote reference: [^identifier]
            footnote_reference_regex: Regex::new(r"\[\^([^\]]+)\]").unwrap(),

            // Footnote definition: [^identifier]: text
            footnote_definition_regex: Regex::new(r"^\[\^([^\]]+)\]:[\s]+(.*)$").unwrap(),

            // Definition term (followed by : definition)
            definition_term_regex: Regex::new(r"^([^\n:]+)$").unwrap(),

            // Definition description: : definition text
            definition_description_regex: Regex::new(r"^:[\s]+(.*)$").unwrap(),

            // Fenced code blocks: ```language
            fenced_code_regex: Regex::new(r"^```([a-zA-Z0-9+#\-]*)$").unwrap(),

            // Heading with ID: ## Heading {#custom-id}
            heading_with_id_regex: Regex::new(
                r"^(#{1,6})[\s]+(.+?)[\s]*\{#([a-zA-Z0-9\-_]+)\}[\s]*$",
            )
            .unwrap(),

            // Highlight: ==text==
            highlight_regex: Regex::new(r"==([^=]+)==").unwrap(),

            // Subscript: ~text~
            subscript_regex: Regex::new(r"~([^~\s]+)~").unwrap(),

            // Superscript: ^text^ (avoid conflict with footnotes [^1])
            // Use word boundary or space to ensure we don't match footnotes
            superscript_regex: Regex::new(r"(^|\s)\^([^\^\s]+)\^").unwrap(),

            // Auto URL: http://example.com or https://example.com
            auto_url_regex: Regex::new(r"(?:^|[\s])(https?://[^\s]+)").unwrap(),

            // Emoji shortcodes: :emoji_name:
            emoji_shortcode_regex: Regex::new(r":([a-zA-Z0-9_+\-]+):").unwrap(),
        }
    }

    /// Apply extended syntax highlighting to a buffer
    pub fn apply_syntax_highlighting(
        &self,
        buffer: &sourceview5::Buffer,
        tag_table: &mut HashMap<String, gtk4::TextTag>,
    ) {
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        let text = gtk_buffer.text(&gtk_buffer.start_iter(), &gtk_buffer.end_iter(), false);

        // Clear existing extended syntax tags
        self.clear_extended_tags(buffer, tag_table);

        // Apply extended syntax highlighting
        self.highlight_tables(buffer, &text, tag_table);
        self.highlight_task_lists(buffer, &text, tag_table);
        self.highlight_strikethrough(buffer, &text, tag_table);
        self.highlight_footnotes(buffer, &text, tag_table);
        self.highlight_definition_lists(buffer, &text, tag_table);
        self.highlight_fenced_code_blocks(buffer, &text, tag_table);
        self.highlight_heading_ids(buffer, &text, tag_table);
        self.highlight_special_text(buffer, &text, tag_table);
        self.highlight_auto_urls(buffer, &text, tag_table);
        self.highlight_emoji_shortcodes(buffer, &text, tag_table);
    }

    /// Clear extended syntax tags
    fn clear_extended_tags(
        &self,
        buffer: &sourceview5::Buffer,
        tag_table: &HashMap<String, gtk4::TextTag>,
    ) {
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        let start = gtk_buffer.start_iter();
        let end = gtk_buffer.end_iter();

        let extended_tags = [
            "table",
            "table-header",
            "table-separator",
            "task-list",
            "task-checked",
            "task-unchecked",
            "strikethrough",
            "footnote-ref",
            "footnote-def",
            "definition-term",
            "definition-desc",
            "fenced-code",
            "heading-id",
            "highlight",
            "subscript",
            "superscript",
            "auto-url",
            "emoji",
        ];

        for tag_name in &extended_tags {
            if let Some(tag) = tag_table.get(*tag_name) {
                gtk_buffer.remove_tag(tag, &start, &end);
            }
        }
    }

    /// Highlight tables
    fn highlight_tables(
        &self,
        buffer: &sourceview5::Buffer,
        text: &str,
        tag_table: &mut HashMap<String, gtk4::TextTag>,
    ) {
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();

        // Create table tags if they don't exist
        self.ensure_tag_exists(tag_table, "table", |tag| {
            tag.set_family(Some("monospace"));
            tag.set_foreground(Some("#2563eb")); // Blue
        });

        self.ensure_tag_exists(tag_table, "table-separator", |tag| {
            tag.set_foreground(Some("#6b7280")); // Gray
            tag.set_family(Some("monospace"));
        });

        let lines: Vec<&str> = text.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            if self.table_regex.is_match(line) {
                let line_start = gtk_buffer.iter_at_line(line_num as i32).unwrap();
                let mut line_end = line_start;
                line_end.forward_to_line_end();

                let tag_name = if self.table_header_separator_regex.is_match(line) {
                    "table-separator"
                } else {
                    "table"
                };

                if let Some(tag) = tag_table.get(tag_name) {
                    gtk_buffer.apply_tag(tag, &line_start, &line_end);
                }
            }
        }
    }

    /// Highlight task lists
    fn highlight_task_lists(
        &self,
        buffer: &sourceview5::Buffer,
        text: &str,
        tag_table: &mut HashMap<String, gtk4::TextTag>,
    ) {
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();

        // Create task list tags
        self.ensure_tag_exists(tag_table, "task-checked", |tag| {
            tag.set_foreground(Some("#16a34a")); // Green
            tag.set_weight(700); // Bold weight as integer
        });

        self.ensure_tag_exists(tag_table, "task-unchecked", |tag| {
            tag.set_foreground(Some("#6b7280")); // Gray
        });

        let lines: Vec<&str> = text.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            if let Some(captures) = self.task_list_regex.captures(line) {
                let checkbox_state = captures.get(1).unwrap().as_str();
                let is_checked = checkbox_state.to_lowercase() == "x";

                let line_start = gtk_buffer.iter_at_line(line_num as i32).unwrap();
                let mut line_end = line_start;
                line_end.forward_to_line_end();

                let tag_name = if is_checked {
                    "task-checked"
                } else {
                    "task-unchecked"
                };
                if let Some(tag) = tag_table.get(tag_name) {
                    gtk_buffer.apply_tag(tag, &line_start, &line_end);
                }
            }
        }
    }

    /// Highlight strikethrough text
    fn highlight_strikethrough(
        &self,
        buffer: &sourceview5::Buffer,
        text: &str,
        tag_table: &mut HashMap<String, gtk4::TextTag>,
    ) {
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();

        self.ensure_tag_exists(tag_table, "strikethrough", |tag| {
            tag.set_strikethrough(true);
            tag.set_foreground(Some("#6b7280")); // Gray
        });

        if let Some(tag) = tag_table.get("strikethrough") {
            for mat in self.strikethrough_regex.find_iter(text) {
                let start_iter = gtk_buffer.iter_at_offset(mat.start() as i32);
                let end_iter = gtk_buffer.iter_at_offset(mat.end() as i32);
                gtk_buffer.apply_tag(tag, &start_iter, &end_iter);
            }
        }
    }

    /// Highlight footnotes
    fn highlight_footnotes(
        &self,
        buffer: &sourceview5::Buffer,
        text: &str,
        tag_table: &mut HashMap<String, gtk4::TextTag>,
    ) {
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();

        self.ensure_tag_exists(tag_table, "footnote-ref", |tag| {
            tag.set_foreground(Some("#7c3aed")); // Purple
            tag.set_underline(gtk4::pango::Underline::Single);
        });

        self.ensure_tag_exists(tag_table, "footnote-def", |tag| {
            tag.set_foreground(Some("#7c3aed")); // Purple
            tag.set_weight(700); // Bold weight as integer
        });

        // Highlight footnote references [^1]
        if let Some(ref_tag) = tag_table.get("footnote-ref") {
            for mat in self.footnote_reference_regex.find_iter(text) {
                let start_iter = gtk_buffer.iter_at_offset(mat.start() as i32);
                let end_iter = gtk_buffer.iter_at_offset(mat.end() as i32);
                gtk_buffer.apply_tag(ref_tag, &start_iter, &end_iter);
            }
        }

        // Highlight footnote definitions
        if let Some(def_tag) = tag_table.get("footnote-def") {
            let lines: Vec<&str> = text.lines().collect();
            for (line_num, line) in lines.iter().enumerate() {
                if self.footnote_definition_regex.is_match(line) {
                    let line_start = gtk_buffer.iter_at_line(line_num as i32).unwrap();
                    let mut line_end = line_start;
                    line_end.forward_to_line_end();
                    gtk_buffer.apply_tag(def_tag, &line_start, &line_end);
                }
            }
        }
    }

    /// Highlight definition lists
    fn highlight_definition_lists(
        &self,
        buffer: &sourceview5::Buffer,
        text: &str,
        tag_table: &mut HashMap<String, gtk4::TextTag>,
    ) {
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();

        self.ensure_tag_exists(tag_table, "definition-term", |tag| {
            tag.set_weight(700); // Bold weight as integer
            tag.set_foreground(Some("#1f2937")); // Dark gray
        });

        self.ensure_tag_exists(tag_table, "definition-desc", |tag| {
            tag.set_style(gtk4::pango::Style::Italic);
            tag.set_foreground(Some("#4b5563")); // Medium gray
        });

        let lines: Vec<&str> = text.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            // Check if next line is a definition description
            if line_num + 1 < lines.len()
                && self
                    .definition_description_regex
                    .is_match(lines[line_num + 1])
            {
                // This line is a definition term
                if let Some(term_tag) = tag_table.get("definition-term") {
                    let line_start = gtk_buffer.iter_at_line(line_num as i32).unwrap();
                    let mut line_end = line_start;
                    line_end.forward_to_line_end();
                    gtk_buffer.apply_tag(term_tag, &line_start, &line_end);
                }
            } else if self.definition_description_regex.is_match(line) {
                // This line is a definition description
                if let Some(desc_tag) = tag_table.get("definition-desc") {
                    let line_start = gtk_buffer.iter_at_line(line_num as i32).unwrap();
                    let mut line_end = line_start;
                    line_end.forward_to_line_end();
                    gtk_buffer.apply_tag(desc_tag, &line_start, &line_end);
                }
            }
        }
    }

    /// Highlight fenced code blocks
    fn highlight_fenced_code_blocks(
        &self,
        buffer: &sourceview5::Buffer,
        text: &str,
        tag_table: &mut HashMap<String, gtk4::TextTag>,
    ) {
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();

        self.ensure_tag_exists(tag_table, "fenced-code", |tag| {
            tag.set_family(Some("monospace"));
            tag.set_background(Some("#f3f4f6")); // Light gray background
            tag.set_foreground(Some("#1f2937")); // Dark text
        });

        if let Some(tag) = tag_table.get("fenced-code") {
            let lines: Vec<&str> = text.lines().collect();
            let mut in_code_block = false;
            let mut code_block_start: Option<i32> = None;

            for (line_num, line) in lines.iter().enumerate() {
                if line.starts_with("```") {
                    if in_code_block {
                        // End of code block
                        if let Some(start_line) = code_block_start {
                            let start_iter = gtk_buffer.iter_at_line(start_line).unwrap();
                            let mut end_iter = gtk_buffer.iter_at_line(line_num as i32).unwrap();
                            end_iter.forward_to_line_end();
                            gtk_buffer.apply_tag(tag, &start_iter, &end_iter);
                        }
                        in_code_block = false;
                        code_block_start = None;
                    } else {
                        // Start of code block
                        in_code_block = true;
                        code_block_start = Some(line_num as i32);
                    }
                }
            }
        }
    }

    /// Highlight headings with custom IDs
    fn highlight_heading_ids(
        &self,
        buffer: &sourceview5::Buffer,
        text: &str,
        tag_table: &mut HashMap<String, gtk4::TextTag>,
    ) {
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();

        self.ensure_tag_exists(tag_table, "heading-id", |tag| {
            tag.set_foreground(Some("#7c3aed")); // Purple
            tag.set_style(gtk4::pango::Style::Italic);
        });

        if let Some(tag) = tag_table.get("heading-id") {
            for captures in self.heading_with_id_regex.captures_iter(text) {
                if let Some(id_match) = captures.get(0) {
                    let start_iter = gtk_buffer.iter_at_offset(id_match.start() as i32);
                    let end_iter = gtk_buffer.iter_at_offset(id_match.end() as i32);
                    gtk_buffer.apply_tag(tag, &start_iter, &end_iter);
                }
            }
        }
    }

    /// Highlight special text (highlight, subscript, superscript)
    fn highlight_special_text(
        &self,
        buffer: &sourceview5::Buffer,
        text: &str,
        tag_table: &mut HashMap<String, gtk4::TextTag>,
    ) {
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();

        // Highlight ==text==
        self.ensure_tag_exists(tag_table, "highlight", |tag| {
            tag.set_background(Some("#fef08a")); // Yellow highlight
            tag.set_foreground(Some("#1f2937"));
        });

        // Subscript ~text~ (simplified without actual subscript positioning)
        self.ensure_tag_exists(tag_table, "subscript", |tag| {
            tag.set_foreground(Some("#6b7280"));
            tag.set_scale(0.8); // Smaller text
        });

        // Superscript ^text^ (simplified without actual superscript positioning)
        self.ensure_tag_exists(tag_table, "superscript", |tag| {
            tag.set_foreground(Some("#6b7280"));
            tag.set_scale(0.8); // Smaller text
        });

        // Apply highlight
        if let Some(tag) = tag_table.get("highlight") {
            for mat in self.highlight_regex.find_iter(text) {
                let start_iter = gtk_buffer.iter_at_offset(mat.start() as i32);
                let end_iter = gtk_buffer.iter_at_offset(mat.end() as i32);
                gtk_buffer.apply_tag(tag, &start_iter, &end_iter);
            }
        }

        // Apply subscript
        if let Some(tag) = tag_table.get("subscript") {
            for mat in self.subscript_regex.find_iter(text) {
                let start_iter = gtk_buffer.iter_at_offset(mat.start() as i32);
                let end_iter = gtk_buffer.iter_at_offset(mat.end() as i32);
                gtk_buffer.apply_tag(tag, &start_iter, &end_iter);
            }
        }

        // Apply superscript
        if let Some(tag) = tag_table.get("superscript") {
            for mat in self.superscript_regex.find_iter(text) {
                let start_iter = gtk_buffer.iter_at_offset(mat.start() as i32);
                let end_iter = gtk_buffer.iter_at_offset(mat.end() as i32);
                gtk_buffer.apply_tag(tag, &start_iter, &end_iter);
            }
        }
    }

    /// Highlight automatic URLs
    fn highlight_auto_urls(
        &self,
        buffer: &sourceview5::Buffer,
        text: &str,
        tag_table: &mut HashMap<String, gtk4::TextTag>,
    ) {
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();

        self.ensure_tag_exists(tag_table, "auto-url", |tag| {
            tag.set_foreground(Some("#2563eb")); // Blue
            tag.set_underline(gtk4::pango::Underline::Single);
        });

        if let Some(tag) = tag_table.get("auto-url") {
            for captures in self.auto_url_regex.captures_iter(text) {
                if let Some(url_match) = captures.get(1) {
                    let start_iter = gtk_buffer.iter_at_offset(url_match.start() as i32);
                    let end_iter = gtk_buffer.iter_at_offset(url_match.end() as i32);
                    gtk_buffer.apply_tag(tag, &start_iter, &end_iter);
                }
            }
        }
    }

    /// Highlight emoji shortcodes
    fn highlight_emoji_shortcodes(
        &self,
        buffer: &sourceview5::Buffer,
        text: &str,
        tag_table: &mut HashMap<String, gtk4::TextTag>,
    ) {
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();

        self.ensure_tag_exists(tag_table, "emoji", |tag| {
            tag.set_foreground(Some("#f59e0b")); // Orange
            tag.set_weight(700); // Bold weight as integer
        });

        if let Some(tag) = tag_table.get("emoji") {
            for mat in self.emoji_shortcode_regex.find_iter(text) {
                let start_iter = gtk_buffer.iter_at_offset(mat.start() as i32);
                let end_iter = gtk_buffer.iter_at_offset(mat.end() as i32);
                gtk_buffer.apply_tag(tag, &start_iter, &end_iter);
            }
        }
    }

    /// Helper method to ensure a tag exists in the tag table
    fn ensure_tag_exists<F>(
        &self,
        tag_table: &mut HashMap<String, gtk4::TextTag>,
        name: &str,
        configure: F,
    ) where
        F: FnOnce(&gtk4::TextTag),
    {
        if !tag_table.contains_key(name) {
            let tag = gtk4::TextTag::new(Some(name));
            configure(&tag);
            tag_table.insert(name.to_string(), tag);
        }
    }
}

impl Default for ExtendedMarkdownSyntax {
    fn default() -> Self {
        Self::new()
    }
}
