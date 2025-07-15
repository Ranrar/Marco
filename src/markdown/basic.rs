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
    /// Check if a line is an ordered list item
    pub fn is_ordered_list(&self, line: &str) -> bool {
        self.ordered_list_regex.is_match(line)
    }

    /// Iterate over all regions for a given inline format in a line
    pub fn find_bold_double(&self, line: &str) -> Vec<(usize, usize)> {
        self.bold_double_regex.captures_iter(line)
            .filter_map(|cap| cap.get(0).map(|m| (m.start(), m.end())))
            .collect()
    }
    pub fn find_bold_underscore(&self, line: &str) -> Vec<(usize, usize)> {
        self.bold_underscore_regex.captures_iter(line)
            .filter_map(|cap| cap.get(0).map(|m| (m.start(), m.end())))
            .collect()
    }
    pub fn find_italic_single(&self, line: &str) -> Vec<(usize, usize)> {
        self.italic_single_regex.captures_iter(line)
            .filter_map(|cap| cap.get(0).map(|m| (m.start(), m.end())))
            .collect()
    }
    pub fn find_italic_underscore(&self, line: &str) -> Vec<(usize, usize)> {
        self.italic_underscore_regex.captures_iter(line)
            .filter_map(|cap| cap.get(0).map(|m| (m.start(), m.end())))
            .collect()
    }
    pub fn find_strikethrough(&self, line: &str) -> Vec<(usize, usize)> {
        self.strikethrough_regex.captures_iter(line)
            .filter_map(|cap| cap.get(0).map(|m| (m.start(), m.end())))
            .collect()
    }
    pub fn find_inline_code(&self, line: &str) -> Vec<(usize, usize)> {
        self.code_regex.captures_iter(line)
            .filter_map(|cap| cap.get(0).map(|m| (m.start(), m.end())))
            .collect()
    }
    pub fn find_links(&self, line: &str) -> Vec<(usize, usize)> {
        self.link_regex.captures_iter(line)
            .filter_map(|cap| cap.get(0).map(|m| (m.start(), m.end())))
            .collect()
    }
    pub fn find_images(&self, line: &str) -> Vec<(usize, usize)> {
        self.image_regex.captures_iter(line)
            .filter_map(|cap| cap.get(0).map(|m| (m.start(), m.end())))
            .collect()
    }
    /// Check if a line is a Markdown heading and return its level if so
    pub fn detect_heading(&self, line: &str) -> Option<usize> {
        if let Some(caps) = self.heading_regex.captures(line.trim_start()) {
            Some(caps[1].len())
        } else {
            None
        }
    }

    /// Check if the text contains bold formatting (** or __)
    pub fn detect_bold(&self, text: &str) -> bool {
        self.bold_double_regex.is_match(text) || self.bold_underscore_regex.is_match(text)
    }

    /// Check if the text contains italic formatting (* or _)
    pub fn detect_italic(&self, text: &str) -> bool {
        self.italic_single_regex.is_match(text) || self.italic_underscore_regex.is_match(text)
    }

    /// Check if the text contains inline code formatting (`)
    pub fn detect_inline_code(&self, text: &str) -> bool {
        self.code_regex.is_match(text)
    }
    pub fn new() -> Self {
        Self {
            heading_regex: Regex::new(r"^(#{1,6})\s+(.+)$").unwrap(),
            // Simplified patterns without lookbehind/lookahead
            bold_double_regex: Regex::new(r"\*\*([^*]+)\*\*").unwrap(),
            bold_underscore_regex: Regex::new(r"__([^_]+)__").unwrap(),
            italic_single_regex: Regex::new(r"\*([^*]+)\*").unwrap(),
            italic_underscore_regex: Regex::new(r"_([^_]+)_").unwrap(),
            // Only match `code` where backticks touch the text (no spaces), allow single char
            code_regex: Regex::new(r"`([^\s`]|[^\s`][^`]*[^\s`])`").unwrap(),
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
            // Sequential processing for small docs, with table support
            let mut html = String::new();
            // Footnote collection: map label -> definition
            let mut footnotes: std::collections::HashMap<String, String> = std::collections::HashMap::new();
            let mut footnote_order: Vec<String> = Vec::new();
            // Pre-scan for footnote definitions
            let mut j = 0;
            while j < lines.len() {
                let line = lines[j].trim();
                if let Some(caps) = Regex::new(r"^\[\^([a-zA-Z0-9_-]+)\]:\s*(.+)$").unwrap().captures(line) {
                    let label = caps[1].to_string();
                    let def = caps[2].to_string();
                    footnotes.insert(label.clone(), def);
                    footnote_order.push(label);
                }
                j += 1;
            }
            let mut i = 0;
            let mut in_unordered_list = false;
            let mut in_ordered_list = false;
            let mut in_blockquote = false;
            let mut in_code_block = false;
            let mut in_fenced_code = false;
            let mut in_table = false;
            let mut table_header: Vec<String> = Vec::new();
            let mut table_align: Vec<String> = Vec::new();
            let mut table_rows: Vec<Vec<String>> = Vec::new();

            while i < lines.len() {
                let line = lines[i];
                let trimmed = line.trim();
                // Skip footnote definition lines in main output
                if Regex::new(r"^\[\^([a-zA-Z0-9_-]+)\]:\s*(.+)$").unwrap().is_match(trimmed) {
                    i += 1;
                    continue;
                }

                // Table detection: GitHub-style pipe tables
                if !in_code_block && !in_fenced_code && !in_table && line.contains('|') {
                    // Try to parse as table header
                    let mut parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
                    // Remove leading/trailing empty cells from pipes
                    if !parts.is_empty() && parts[0].is_empty() { parts.remove(0); }
                    if !parts.is_empty() && parts.last().unwrap().is_empty() { parts.pop(); }
                    if parts.len() > 1 && i + 1 < lines.len() {
                        let mut next_parts: Vec<&str> = lines[i + 1].split('|').map(|s| s.trim()).collect();
                        if !next_parts.is_empty() && next_parts[0].is_empty() { next_parts.remove(0); }
                        if !next_parts.is_empty() && next_parts.last().unwrap().is_empty() { next_parts.pop(); }
                        let next = lines[i + 1].trim();
                        // Check for alignment row
                        let is_align = next.chars().all(|c| c == '|' || c == ':' || c == '-' || c.is_whitespace());
                        if is_align && next.contains('-') {
                            // Parse header and alignment
                            table_header = parts.iter().map(|s| s.to_string()).collect();
                            table_align = next_parts.iter().map(|s| s.to_string()).collect();
                            in_table = true;
                            table_rows.clear();
                            i += 2;
                            continue;
                        }
                    }
                }
                if in_table {
                    // End of table if blank line or not a table row
                    if trimmed.is_empty() || !line.contains('|') {
                        // Render table
                        html.push_str("<table>\n<thead><tr>");
                        for h in &table_header {
                            html.push_str(&format!("<th>{}</th>", self.process_inline_formatting(h)));
                        }
                        html.push_str("</tr></thead>\n<tbody>\n");
                        for row in &table_rows {
                            // Remove leading/trailing empty cells
                            let mut cells = row.clone();
                            if !cells.is_empty() && cells[0].is_empty() { cells.remove(0); }
                            if !cells.is_empty() && cells.last().unwrap().is_empty() { cells.pop(); }
                            html.push_str("<tr>");
                            for cell in &cells {
                                html.push_str(&format!("<td>{}</td>", self.process_inline_formatting(cell)));
                            }
                            html.push_str("</tr>\n");
                        }
                        html.push_str("</tbody></table>\n");
                        in_table = false;
                        table_header.clear();
                        table_align.clear();
                        table_rows.clear();
                        // Do not increment i, reprocess this line
                        continue;
                    } else if line.contains('|') {
                        // Parse table row
                        let mut row: Vec<String> = line.split('|').map(|s| s.trim().to_string()).collect();
                        if !row.is_empty() && row[0].is_empty() { row.remove(0); }
                        if !row.is_empty() && row.last().unwrap().is_empty() { row.pop(); }
                        table_rows.push(row);
                        i += 1;
                        continue;
                    } else {
                        // Malformed table row, forcibly close table and continue
                        html.push_str("<table>\n<thead><tr>");
                        for h in &table_header {
                            html.push_str(&format!("<th>{}</th>", self.process_inline_formatting(h)));
                        }
                        html.push_str("</tr></thead>\n<tbody>\n");
                        for row in &table_rows {
                            let mut cells = row.clone();
                            if !cells.is_empty() && cells[0].is_empty() { cells.remove(0); }
                            if !cells.is_empty() && cells.last().unwrap().is_empty() { cells.pop(); }
                            html.push_str("<tr>");
                            for cell in &cells {
                                html.push_str(&format!("<td>{}</td>", self.process_inline_formatting(cell)));
                            }
                            html.push_str("</tr>\n");
                        }
                        html.push_str("</tbody></table>\n");
                        in_table = false;
                        table_header.clear();
                        table_align.clear();
                        table_rows.clear();
                        // Do not increment i, reprocess this line
                        continue;
                    }
                }

                // Footnote reference: [^label]
                let footnote_ref_re = Regex::new(r"\[\^([a-zA-Z0-9_-]+)\]").unwrap();
                if footnote_ref_re.is_match(line) {
                    let replaced = footnote_ref_re.replace_all(line, |caps: &regex::Captures| {
                        let label = &caps[1];
                        if let Some(idx) = footnote_order.iter().position(|l| l == label) {
                            format!("<sup class=\"footnote\"><a href=\"#fn-{}\" id=\"fnref-{}\">{}</a></sup>", label, label, idx+1)
                        } else {
                            format!("<sup class=\"footnote\"><a href=\"#fn-{}\" id=\"fnref-{}\">?</a></sup>", label, label)
                        }
                    });
                    self.close_all_blocks(
                        &mut html,
                        &mut in_unordered_list,
                        &mut in_ordered_list,
                        &mut in_blockquote,
                    );
                    html.push_str("<p>");
                    html.push_str(&self.process_inline_formatting(&replaced));
                    html.push_str("</p>\n");
                    i += 1;
                    continue;
                }
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
                    // Task list support: - [ ] and - [x]
                    let task_open = Regex::new(r"^\[ \]\s?(.*)$").unwrap();
                    let task_closed = Regex::new(r"^\[x\]\s?(.*)$").unwrap();
                    if let Some(cap) = task_open.captures(list_content) {
                        html.push_str("<li><input type=\"checkbox\" disabled> ");
                        html.push_str(&self.process_inline_formatting(&cap[1]));
                        html.push_str("</li>\n");
                    } else if let Some(cap) = task_closed.captures(list_content) {
                        html.push_str("<li><input type=\"checkbox\" checked disabled> ");
                        html.push_str(&self.process_inline_formatting(&cap[1]));
                        html.push_str("</li>\n");
                    } else {
                        html.push_str("<li>");
                        html.push_str(&self.process_inline_formatting(list_content));
                        html.push_str("</li>\n");
                    }
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
            // Render footnotes at the end
            if !footnote_order.is_empty() {
                html.push_str("<section class=\"footnotes\"><ol>\n");
                for label in &footnote_order {
                    if let Some(def) = footnotes.get(label) {
                        html.push_str(&format!("<li id=\"fn-{}\">{} <a href=\"#fnref-{}\" class=\"footnote-backref\">↩</a></li>\n", label, self.process_inline_formatting(def), label));
                    }
                }
                html.push_str("</ol></section>\n");
            }
            // If table was open at EOF, close and render it (even if malformed)
            if in_table {
                html.push_str("<table>\n<thead><tr>");
                for h in &table_header {
                    html.push_str(&format!("<th>{}</th>", self.process_inline_formatting(h)));
                }
                html.push_str("</tr></thead>\n<tbody>\n");
                for row in &table_rows {
                    let mut cells = row.clone();
                    if !cells.is_empty() && cells[0].is_empty() { cells.remove(0); }
                    if !cells.is_empty() && cells.last().unwrap().is_empty() { cells.pop(); }
                    html.push_str("<tr>");
                    for cell in &cells {
                        html.push_str(&format!("<td>{}</td>", self.process_inline_formatting(cell)));
                    }
                    html.push_str("</tr>\n");
                }
                html.push_str("</tbody></table>\n");
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
        // Support heading IDs: ## Heading {#id}
        let heading_id_re = Regex::new(r"^(#{1,6})\s+(.+?)(?:\s*\{#([a-zA-Z0-9\-_]+)\})?$" ).unwrap();
        if let Some(captures) = heading_id_re.captures(line.trim_start()) {
            let level = captures[1].len();
            let content = captures.get(2).map(|m| m.as_str()).unwrap_or("");
            let id = captures.get(3).map(|m| m.as_str());
            let anchor_id = if let Some(id) = id {
                id.to_string()
            } else {
                // Generate anchor from content: lowercase, replace all non-alphanum with '-', collapse dashes, fallback to 'heading' if empty
                let mut s = content.to_lowercase();
                // Replace all non-alphanumeric with '-'
                s = s.chars().map(|c| if c.is_alphanumeric() { c } else { '-' }).collect();
                // Collapse multiple dashes
                let mut collapsed = String::new();
                let mut prev_dash = false;
                for c in s.chars() {
                    if c == '-' {
                        if !prev_dash {
                            collapsed.push('-');
                            prev_dash = true;
                        }
                    } else {
                        collapsed.push(c);
                        prev_dash = false;
                    }
                }
                let s = collapsed.trim_matches('-').to_string();
                if s.is_empty() {
                    "heading".to_string()
                } else {
                    s
                }
            };
            return Some(format!(
                "<h{lvl} id=\"{id}\">{content}</h{lvl}>",
                lvl=level,
                id=anchor_id,
                content=self.process_inline_formatting(content)
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
        // 0. Use regex to replace all escaped markdown characters with unique placeholders before any formatting
        let mut result = text.to_string();
        let escape_patterns = [
            ('*', "ESCAPEDMARKERAST1"),
            ('_', "ESCAPEDMARKERUND1"),
            ('`', "ESCAPEDMARKERBTK1"),
            ('[', "ESCAPEDMARKERLB1"),
            (']', "ESCAPEDMARKERRB1"),
        ];
        for (ch, placeholder) in &escape_patterns {
            let re = Regex::new(&format!(r"\\{}", regex::escape(&ch.to_string()))).unwrap();
            result = re.replace_all(&result, *placeholder).to_string();
        }

        // 1. Extract inline code spans and replace with safe placeholders
        let mut code_spans = Vec::new();
        let code_regex = &self.code_regex;
        let mut placeholder_result = String::new();
        let mut last = 0;
        for cap in code_regex.captures_iter(&result) {
            if let Some(m) = cap.get(0) {
                // Push text before code span
                placeholder_result.push_str(&result[last..m.start()]);
                // Store code content (no formatting, just HTML-escape)
                code_spans.push(cap[1].to_string());
                // Insert safe placeholder
                placeholder_result.push_str(&format!("CODEMARKER{}", code_spans.len() - 1));
                last = m.end();
            }
        }
        placeholder_result.push_str(&result[last..]);
        result = placeholder_result;

        // 2. (Removed: escaping now handled above)

        // 3. Images (must come before links)
        result = self
            .image_regex
            .replace_all(&result, |caps: &regex::Captures| {
                format!("<img src=\"{}\" alt=\"{}\">", &caps[2], &caps[1])
            })
            .to_string();

        // 4. Links
        result = self
            .link_regex
            .replace_all(&result, |caps: &regex::Captures| {
                format!("<a href=\"{}\">{}</a>", &caps[2], &caps[1])
            })
            .to_string();

        // 5. ==highlighted==
        let highlight_re = Regex::new(r"==([^=]+)==").unwrap();
        result = highlight_re.replace_all(&result, "<mark>$1</mark>").to_string();

        // 6. Auto-link URLs (not inside []())
        let url_regex = Regex::new(r"(?i)\b((?:https?://|www\.)[^\s<]+[^\s<\.,;:])").unwrap();
        result = url_regex.replace_all(&result, |caps: &regex::Captures| {
            let url = &caps[1];
            if url.starts_with("www.") {
                format!("<a href=\"http://{0}\">{0}</a>", url)
            } else {
                format!("<a href=\"{0}\">{0}</a>", url)
            }
        }).to_string();

        // 7. Emoji shortcodes :smile:
        let emoji_re = Regex::new(r":([a-zA-Z0-9_+\-]+):").unwrap();
        result = emoji_re.replace_all(&result, |caps: &regex::Captures| {
            let name = &caps[1];
            let emoji = match name {
                "smile" => "😄",
                "rocket" => "🚀",
                "joy" => "😂",
                "sob" => "😭",
                "heart" => "❤️",
                "thumbsup" => "👍",
                "fire" => "🔥",
                _ => caps.get(0).unwrap().as_str(),
            };
            emoji.to_string()
        }).to_string();

        // 8. Handle stacked tildes: ~text~, ~~text~~, ~~~text~~~
        // Only apply subscript for ~text~, strikethrough for ~~text~~, leave ~~~text~~~ as plain text
        let mut formatted = String::new();
        let mut chars = result.chars().peekable();
        while let Some(c) = chars.peek() {
            if *c == '~' {
                // Count consecutive tildes
                let mut tilde_count = 0;
                while let Some('~') = chars.peek() {
                    tilde_count += 1;
                    chars.next();
                }
                // Find the closing tildes
                let mut content = String::new();
                let mut found = false;
                while let Some(&next_c) = chars.peek() {
                    if next_c == '~' {
                        // Check if enough tildes to close
                        let mut close_count = 0;
                        let mut close_peek = chars.clone();
                        while let Some('~') = close_peek.peek() {
                            close_count += 1;
                            close_peek.next();
                        }
                        if close_count == tilde_count {
                            // Found closing
                            for _ in 0..tilde_count { chars.next(); }
                            found = true;
                            break;
                        } else {
                            // Not enough, treat as text
                            content.push(chars.next().unwrap());
                        }
                    } else {
                        content.push(chars.next().unwrap());
                    }
                }
                if found {
                    if tilde_count == 2 {
                        formatted.push_str(&format!("<del>{}</del>", self.process_inline_formatting(&content)));
                    } else if tilde_count == 1 {
                        formatted.push_str(&format!("<sub>{}</sub>", self.process_inline_formatting(&content)));
                    } else {
                        // 3 or more tildes, treat as plain text
                        formatted.push_str(&"~".repeat(tilde_count));
                        formatted.push_str(&content);
                        formatted.push_str(&"~".repeat(tilde_count));
                    }
                } else {
                    // No closing, treat as plain text
                    formatted.push_str(&"~".repeat(tilde_count));
                    formatted.push_str(&content);
                }
            } else {
                formatted.push(chars.next().unwrap());
            }
        }
        result = formatted;

        // 9. Superscript: ^text^
        let sup_re = Regex::new(r"\^([^\s^]+)\^").unwrap();
        result = sup_re.replace_all(&result, "<sup>$1</sup>").to_string();

        // 10. Process bold and italic with proper ordering and conflict resolution
        result = self.process_bold_italic(&result);

        // 11. Handle line breaks (two spaces at end of line)
        if result.ends_with("  ") {
            result = result.trim_end().to_string() + "<br>";
        }

        // 12. Substitute code placeholders with HTML-escaped code content (no formatting)
        for (idx, code) in code_spans.iter().enumerate() {
            let placeholder = format!("CODEMARKER{}", idx);
            let escaped = self.escape_html(code);
            result = result.replace(&placeholder, &format!("<code>{}</code>", escaped));
        }

        // 13. Restore escaped characters at the very end
        result = result.replace("ESCAPEDMARKERAST1", "*");
        result = result.replace("ESCAPEDMARKERUND1", "_");
        result = result.replace("ESCAPEDMARKERBTK1", "`");
        result = result.replace("ESCAPEDMARKERLB1", "[");
        result = result.replace("ESCAPEDMARKERRB1", "]");

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
    #[test]
    fn debug_escaping_output() {
        let parser = MarkdownParser::new();
        let markdown1 = r"This is \*not italic\*.";
        let html1 = parser.process_inline_formatting(markdown1);
        println!("DEBUG OUTPUT: {}", html1);
    }
    use super::*;

    #[test]
    fn test_headings_all_levels() {
        let parser = MarkdownParser::new();

        assert_eq!(
            parser.parse_heading("# Heading level 1"),
            Some("<h1 id=\"heading-level-1\">Heading level 1</h1>".to_string())
        );
        assert_eq!(
            parser.parse_heading("## Heading level 2"),
            Some("<h2 id=\"heading-level-2\">Heading level 2</h2>".to_string())
        );
        assert_eq!(
            parser.parse_heading("### Heading level 3"),
            Some("<h3 id=\"heading-level-3\">Heading level 3</h3>".to_string())
        );
        assert_eq!(
            parser.parse_heading("#### Heading level 4"),
            Some("<h4 id=\"heading-level-4\">Heading level 4</h4>".to_string())
        );
        assert_eq!(
            parser.parse_heading("##### Heading level 5"),
            Some("<h5 id=\"heading-level-5\">Heading level 5</h5>".to_string())
        );
        assert_eq!(
            parser.parse_heading("###### Heading level 6"),
            Some("<h6 id=\"heading-level-6\">Heading level 6</h6>".to_string())
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
