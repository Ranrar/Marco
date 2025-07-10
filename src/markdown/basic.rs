use regex::Regex;

/// A comprehensive markdown to HTML converter implementing basic markdown syntax
/// Following best practices from the Markdown Guide
pub struct MarkdownParser {
    // More precise regex patterns following markdown spec
    heading_regex: Regex,
    bold_double_regex: Regex,       // **text**
    bold_underscore_regex: Regex,   // __text__
    italic_single_regex: Regex,     // *text*
    italic_underscore_regex: Regex, // _text_
    code_regex: Regex,
    strikethrough_regex: Regex,
    link_regex: Regex,
    image_regex: Regex,
    ordered_list_regex: Regex,
}

impl MarkdownParser {
    pub fn new() -> Self {
        Self {
            heading_regex: Regex::new(r"^(#{1,6})\s+(.+)$").unwrap(),
            // Simplified patterns without lookbehind/lookahead
            bold_double_regex: Regex::new(r"\*\*([^*]+)\*\*").unwrap(),
            bold_underscore_regex: Regex::new(r"__([^_]+)__").unwrap(),
            italic_single_regex: Regex::new(r"\*([^*]+)\*").unwrap(),
            italic_underscore_regex: Regex::new(r"_([^_]+)_").unwrap(),
            code_regex: Regex::new(r"`([^`]+)`").unwrap(),
            strikethrough_regex: Regex::new(r"~~([^~]+)~~").unwrap(),
            link_regex: Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap(),
            image_regex: Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap(),
            ordered_list_regex: Regex::new(r"^(\d+)\.\s+(.+)$").unwrap(),
        }
    }

    /// Convert markdown text to HTML using Rayon for parallel block processing when appropriate.
    pub fn to_html(&self, markdown: &str) -> String {
        use rayon::prelude::*;
        let lines: Vec<&str> = markdown.lines().collect();
        // For large documents, split into blocks and process in parallel
        if lines.len() > 200 {
            // Naive block split: treat each paragraph (separated by empty line) as a block
            let mut blocks = Vec::new();
            let mut current = Vec::new();
            for line in &lines {
                if line.trim().is_empty() {
                    if !current.is_empty() {
                        blocks.push(current.clone());
                        current.clear();
                    }
                } else {
                    current.push(*line);
                }
            }
            if !current.is_empty() {
                blocks.push(current);
            }
            let html_blocks: Vec<String> = blocks
                .par_iter()
                .map(|block| {
                    let joined = block.join("\n");
                    self.to_html(&joined)
                })
                .collect();
            html_blocks.join("\n")
        } else {
            // ...existing code (sequential processing for small docs)...
            let mut html = String::new();
            let mut i = 0;
            let mut in_unordered_list = false;
            let mut in_ordered_list = false;
            let mut in_blockquote = false;
            let mut in_code_block = false;
            let mut in_fenced_code = false;

            while i < lines.len() {
                let line = lines[i];
                let trimmed = line.trim();

                // ...existing code...
                if trimmed.starts_with("```") {
                    if !in_fenced_code {
                        let lang = trimmed.strip_prefix("```").unwrap_or("");
                        if !lang.is_empty() {
                            html.push_str(&format!("<pre><code class=\"language-{}\">", lang));
                        } else {
                            html.push_str("<pre><code>");
                        }
                        in_fenced_code = true;
                    } else {
                        html.push_str("</code></pre>\n");
                        in_fenced_code = false;
                    }
                    i += 1;
                    continue;
                }
                if in_fenced_code {
                    html.push_str(&self.escape_html(line));
                    html.push('\n');
                    i += 1;
                    continue;
                }
                if line.starts_with("    ") && !line.trim().is_empty() {
                    if !in_code_block {
                        html.push_str("<pre><code>");
                        in_code_block = true;
                    }
                    html.push_str(&self.escape_html(&line[4..]));
                    html.push('\n');
                    i += 1;
                    continue;
                } else if in_code_block {
                    html.push_str("</code></pre>\n");
                    in_code_block = false;
                }
                if trimmed.is_empty() {
                    if in_unordered_list {
                        html.push_str("</ul>\n");
                        in_unordered_list = false;
                    }
                    if in_ordered_list {
                        html.push_str("</ol>\n");
                        in_ordered_list = false;
                    }
                    if in_blockquote {
                        html.push_str("</blockquote>\n");
                        in_blockquote = false;
                    }
                    i += 1;
                    continue;
                }
                if let Some(heading_html) = self.parse_heading(line) {
                    self.close_all_blocks(
                        &mut html,
                        &mut in_unordered_list,
                        &mut in_ordered_list,
                        &mut in_blockquote,
                    );
                    html.push_str(&heading_html);
                    html.push('\n');
                } else if self.is_horizontal_rule(trimmed) {
                    self.close_all_blocks(
                        &mut html,
                        &mut in_unordered_list,
                        &mut in_ordered_list,
                        &mut in_blockquote,
                    );
                    html.push_str("<hr>\n");
                } else if trimmed.starts_with('>') {
                    if in_unordered_list || in_ordered_list {
                        self.close_lists(&mut html, &mut in_unordered_list, &mut in_ordered_list);
                    }
                    if !in_blockquote {
                        html.push_str("<blockquote>\n");
                        in_blockquote = true;
                    }
                    let quote_content = trimmed[1..].trim();
                    html.push_str("<p>");
                    html.push_str(&self.process_inline_formatting(quote_content));
                    html.push_str("</p>\n");
                } else if trimmed.starts_with("- ")
                    || trimmed.starts_with("* ")
                    || trimmed.starts_with("+ ")
                {
                    if in_blockquote {
                        html.push_str("</blockquote>\n");
                        in_blockquote = false;
                    }
                    if in_ordered_list {
                        html.push_str("</ol>\n");
                        in_ordered_list = false;
                    }
                    if !in_unordered_list {
                        html.push_str("<ul>\n");
                        in_unordered_list = true;
                    }
                    let list_content = &trimmed[2..];
                    html.push_str("<li>");
                    html.push_str(&self.process_inline_formatting(list_content));
                    html.push_str("</li>\n");
                } else if let Some(captures) = self.ordered_list_regex.captures(trimmed) {
                    if in_blockquote {
                        html.push_str("</blockquote>\n");
                        in_blockquote = false;
                    }
                    if in_unordered_list {
                        html.push_str("</ul>\n");
                        in_unordered_list = false;
                    }
                    if !in_ordered_list {
                        html.push_str("<ol>\n");
                        in_ordered_list = true;
                    }
                    let list_content = &captures[2];
                    html.push_str("<li>");
                    html.push_str(&self.process_inline_formatting(list_content));
                    html.push_str("</li>\n");
                } else {
                    self.close_all_blocks(
                        &mut html,
                        &mut in_unordered_list,
                        &mut in_ordered_list,
                        &mut in_blockquote,
                    );
                    html.push_str("<p>");
                    html.push_str(&self.process_inline_formatting(line));
                    html.push_str("</p>\n");
                }
                i += 1;
            }
            self.close_all_blocks(
                &mut html,
                &mut in_unordered_list,
                &mut in_ordered_list,
                &mut in_blockquote,
            );
            if in_code_block {
                html.push_str("</code></pre>\n");
            }
            if in_fenced_code {
                html.push_str("</code></pre>\n");
            }
            html
        }
    }

    fn close_all_blocks(
        &self,
        html: &mut String,
        in_ul: &mut bool,
        in_ol: &mut bool,
        in_bq: &mut bool,
    ) {
        self.close_lists(html, in_ul, in_ol);
        if *in_bq {
            html.push_str("</blockquote>\n");
            *in_bq = false;
        }
    }

    fn close_lists(&self, html: &mut String, in_ul: &mut bool, in_ol: &mut bool) {
        if *in_ul {
            html.push_str("</ul>\n");
            *in_ul = false;
        }
        if *in_ol {
            html.push_str("</ol>\n");
            *in_ol = false;
        }
    }

    pub fn parse_heading(&self, line: &str) -> Option<String> {
        if let Some(captures) = self.heading_regex.captures(line) {
            let level = captures[1].len();
            let content = &captures[2];
            return Some(format!(
                "<h{}>{}</h{}>",
                level,
                self.process_inline_formatting(content),
                level
            ));
        }
        None
    }

    fn is_horizontal_rule(&self, line: &str) -> bool {
        let line = line.trim();
        // Must be at least 3 characters and all the same
        if line.len() < 3 {
            return false;
        }

        // Allow spaces between characters
        let clean = line.replace(' ', "");
        if clean.len() < 3 {
            return false;
        }

        (clean.chars().all(|c| c == '-') && clean.len() >= 3)
            || (clean.chars().all(|c| c == '*') && clean.len() >= 3)
            || (clean.chars().all(|c| c == '_') && clean.len() >= 3)
    }

    pub fn process_inline_formatting(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Handle escaping first
        result = result.replace(r"\*", "&#42;");
        result = result.replace(r"\_", "&#95;");
        result = result.replace(r"\`", "&#96;");
        result = result.replace(r"\[", "&#91;");
        result = result.replace(r"\]", "&#93;");

        // Images (must come before links)
        result = self
            .image_regex
            .replace_all(&result, |caps: &regex::Captures| {
                format!("<img src=\"{}\" alt=\"{}\">", &caps[2], &caps[1])
            })
            .to_string();

        // Links
        result = self
            .link_regex
            .replace_all(&result, |caps: &regex::Captures| {
                format!("<a href=\"{}\">{}</a>", &caps[2], &caps[1])
            })
            .to_string();

        // Process bold and italic with proper ordering and conflict resolution
        result = self.process_bold_italic(&result);

        // Inline code
        result = self
            .code_regex
            .replace_all(&result, "<code>$1</code>")
            .to_string();

        // Handle line breaks (two spaces at end of line)
        if result.ends_with("  ") {
            result = result.trim_end().to_string() + "<br>";
        }

        // Restore escaped characters
        result = result.replace("&#42;", "*");
        result = result.replace("&#95;", "_");
        result = result.replace("&#96;", "`");
        result = result.replace("&#91;", "[");
        result = result.replace("&#93;", "]");

        result
    }

    fn process_bold_italic(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Handle ***text*** (bold italic)
        let bold_italic_regex = Regex::new(r"\*\*\*([^*]+)\*\*\*").unwrap();
        result = bold_italic_regex
            .replace_all(&result, "<strong><em>$1</em></strong>")
            .to_string();

        // Handle ___text___ (bold italic with underscores)
        let bold_italic_underscore_regex = Regex::new(r"___([^_]+)___").unwrap();
        result = bold_italic_underscore_regex
            .replace_all(&result, "<strong><em>$1</em></strong>")
            .to_string();

        // Bold with ** (must come before italic *)
        result = self
            .bold_double_regex
            .replace_all(&result, "<strong>$1</strong>")
            .to_string();

        // Bold with __
        result = self
            .bold_underscore_regex
            .replace_all(&result, "<strong>$1</strong>")
            .to_string();

        // Italic with * (only if not part of **)
        result = self
            .italic_single_regex
            .replace_all(&result, "<em>$1</em>")
            .to_string();

        // Italic with _ (only if not part of __)
        result = self
            .italic_underscore_regex
            .replace_all(&result, "<em>$1</em>")
            .to_string();

        // Strikethrough with ~~
        result = self
            .strikethrough_regex
            .replace_all(&result, "<del>$1</del>")
            .to_string();

        result
    }

    fn escape_html(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }

    // Individual parsing methods for testing (keeping them for potential future use)
    #[allow(dead_code)]
    pub fn parse_bold(&self, text: &str) -> String {
        let mut result = self
            .bold_double_regex
            .replace_all(text, "<strong>$1</strong>")
            .to_string();
        result = self
            .bold_underscore_regex
            .replace_all(&result, "<strong>$1</strong>")
            .to_string();
        result
    }

    #[allow(dead_code)]
    pub fn parse_italic(&self, text: &str) -> String {
        let mut result = self
            .italic_single_regex
            .replace_all(text, "<em>$1</em>")
            .to_string();
        result = self
            .italic_underscore_regex
            .replace_all(&result, "<em>$1</em>")
            .to_string();
        result
    }

    #[allow(dead_code)]
    pub fn parse_inline_code(&self, text: &str) -> String {
        self.code_regex
            .replace_all(text, "<code>$1</code>")
            .to_string()
    }

    #[allow(dead_code)]
    pub fn parse_links(&self, text: &str) -> String {
        self.link_regex
            .replace_all(text, |caps: &regex::Captures| {
                format!("<a href=\"{}\">{}</a>", &caps[2], &caps[1])
            })
            .to_string()
    }

    #[allow(dead_code)]
    pub fn parse_images(&self, text: &str) -> String {
        self.image_regex
            .replace_all(text, |caps: &regex::Captures| {
                format!("<img src=\"{}\" alt=\"{}\">", &caps[2], &caps[1])
            })
            .to_string()
    }

    #[allow(dead_code)]
    pub fn parse_strikethrough(&self, text: &str) -> String {
        self.strikethrough_regex
            .replace_all(text, "<del>$1</del>")
            .to_string()
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headings_all_levels() {
        let parser = MarkdownParser::new();

        assert_eq!(
            parser.parse_heading("# Heading level 1"),
            Some("<h1>Heading level 1</h1>".to_string())
        );
        assert_eq!(
            parser.parse_heading("## Heading level 2"),
            Some("<h2>Heading level 2</h2>".to_string())
        );
        assert_eq!(
            parser.parse_heading("### Heading level 3"),
            Some("<h3>Heading level 3</h3>".to_string())
        );
        assert_eq!(
            parser.parse_heading("#### Heading level 4"),
            Some("<h4>Heading level 4</h4>".to_string())
        );
        assert_eq!(
            parser.parse_heading("##### Heading level 5"),
            Some("<h5>Heading level 5</h5>".to_string())
        );
        assert_eq!(
            parser.parse_heading("###### Heading level 6"),
            Some("<h6>Heading level 6</h6>".to_string())
        );
    }

    #[test]
    fn test_bold_both_syntaxes() {
        let parser = MarkdownParser::new();

        assert_eq!(
            parser.parse_bold("I love **bold text**."),
            "I love <strong>bold text</strong>."
        );
        assert_eq!(
            parser.parse_bold("I love __bold text__."),
            "I love <strong>bold text</strong>."
        );
    }

    #[test]
    fn test_italic_both_syntaxes() {
        let parser = MarkdownParser::new();

        assert_eq!(
            parser.parse_italic("This is *italic text*."),
            "This is <em>italic text</em>."
        );
        assert_eq!(
            parser.parse_italic("This is _italic text_."),
            "This is <em>italic text</em>."
        );
    }

    #[test]
    fn test_bold_italic_combination() {
        let parser = MarkdownParser::new();

        let text = "***bold and italic***";
        let result = parser.process_inline_formatting(text);
        // Should handle nested formatting
        assert!(result.contains("<strong>") && result.contains("<em>"));
    }

    #[test]
    fn test_fenced_code_blocks() {
        let parser = MarkdownParser::new();

        let markdown = "```rust\nlet x = 5;\n```";
        let html = parser.to_html(markdown);
        assert!(html.contains("<pre><code class=\"language-rust\">"));
        assert!(html.contains("let x = 5;"));
        assert!(html.contains("</code></pre>"));
    }

    #[test]
    fn test_horizontal_rules_all_types() {
        let parser = MarkdownParser::new();

        assert!(parser.to_html("---").contains("<hr>"));
        assert!(parser.to_html("***").contains("<hr>"));
        assert!(parser.to_html("___").contains("<hr>"));
        assert!(parser.to_html("- - -").contains("<hr>"));
        assert!(parser.to_html("* * *").contains("<hr>"));
    }

    #[test]
    fn test_mixed_lists() {
        let parser = MarkdownParser::new();

        let markdown =
            "1. First ordered\n2. Second ordered\n\n- First unordered\n- Second unordered";
        let html = parser.to_html(markdown);

        assert!(html.contains("<ol>"));
        assert!(html.contains("</ol>"));
        assert!(html.contains("<ul>"));
        assert!(html.contains("</ul>"));
    }

    #[test]
    fn test_escaping() {
        let parser = MarkdownParser::new();

        // Test escaping of italic markers
        let markdown1 = r"This is \*not italic\*.";
        let html1 = parser.process_inline_formatting(markdown1);
        assert!(html1.contains("*not italic*"));

        // Test escaping of bold markers (both asterisks need to be escaped)
        let markdown2 = r"This is \*\*not bold\*\*.";
        let html2 = parser.process_inline_formatting(markdown2);
        assert!(html2.contains("**not bold**"));

        // Test escaping of inline code
        let markdown3 = r"This is \`not code\`.";
        let html3 = parser.process_inline_formatting(markdown3);
        assert!(html3.contains("`not code`"));

        // Test escaping of link brackets
        let markdown4 = r"This is \[not a link\].";
        let html4 = parser.process_inline_formatting(markdown4);
        assert!(html4.contains("[not a link]"));
    }
}
