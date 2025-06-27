/// A simple markdown to HTML converter implementing basic markdown syntax
/// Based on the Markdown Guide Basic Syntax specification
pub struct MarkdownParser {
    // Configuration flags
    pub allow_html: bool,
}

impl MarkdownParser {
    pub fn new() -> Self {
        Self {
            allow_html: true,
        }
    }

    /// Convert markdown text to HTML
    pub fn to_html(&self, markdown: &str) -> String {
        let lines: Vec<&str> = markdown.lines().collect();
        let mut html_lines = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim_end();
            
            // Handle empty lines
            if line.trim().is_empty() {
                html_lines.push(String::new());
                i += 1;
                continue;
            }

            // Check for different block elements
            if let Some(heading) = self.parse_heading(line) {
                html_lines.push(heading);
            } else if line.starts_with('>') {
                let (blockquote, consumed) = self.parse_blockquote(&lines[i..]);
                html_lines.push(blockquote);
                i += consumed - 1; // -1 because we'll increment at the end
            } else if self.is_unordered_list_item(line) {
                let (list, consumed) = self.parse_unordered_list(&lines[i..]);
                html_lines.push(list);
                i += consumed - 1;
            } else if self.is_ordered_list_item(line) {
                let (list, consumed) = self.parse_ordered_list(&lines[i..]);
                html_lines.push(list);
                i += consumed - 1;
            } else if self.is_code_block(line) {
                let (code_block, consumed) = self.parse_code_block(&lines[i..]);
                html_lines.push(code_block);
                i += consumed - 1;
            } else if self.is_horizontal_rule(line) {
                html_lines.push("<hr>".to_string());
            } else {
                // Regular paragraph
                let paragraph = self.parse_inline(line);
                html_lines.push(format!("<p>{}</p>", paragraph));
            }
            
            i += 1;
        }

        html_lines.join("\n")
    }

    /// Parse headings (# H1, ## H2, etc.)
    fn parse_heading(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        if !trimmed.starts_with('#') {
            return None;
        }

        let mut level = 0;
        let mut chars = trimmed.chars();
        
        // Count the number of # symbols
        while let Some(ch) = chars.next() {
            if ch == '#' && level < 6 {
                level += 1;
            } else if ch == ' ' && level > 0 {
                break;
            } else if level == 0 {
                return None; // No space after #, not a valid heading
            } else {
                return None; // Invalid heading format
            }
        }

        if level == 0 || level > 6 {
            return None;
        }

        let text = trimmed[level..].trim();
        if text.is_empty() {
            return None;
        }

        let processed_text = self.parse_inline(text);
        Some(format!("<h{}>{}</h{}>", level, processed_text, level))
    }

    /// Parse blockquotes (> text)
    fn parse_blockquote(&self, lines: &[&str]) -> (String, usize) {
        let mut blockquote_lines = Vec::new();
        let mut consumed = 0;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with('>') {
                let content = trimmed[1..].trim();
                if !content.is_empty() {
                    blockquote_lines.push(self.parse_inline(content));
                } else {
                    blockquote_lines.push(String::new());
                }
                consumed += 1;
            } else if trimmed.is_empty() && !blockquote_lines.is_empty() {
                blockquote_lines.push(String::new());
                consumed += 1;
            } else {
                break;
            }
        }

        let content = blockquote_lines.join("<br>\n");
        (format!("<blockquote>\n{}\n</blockquote>", content), consumed)
    }

    /// Check if line is an unordered list item
    fn is_unordered_list_item(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ")
    }

    /// Parse unordered list
    fn parse_unordered_list(&self, lines: &[&str]) -> (String, usize) {
        let mut list_items = Vec::new();
        let mut consumed = 0;

        for line in lines {
            if self.is_unordered_list_item(line) {
                let trimmed = line.trim();
                let content = trimmed[2..].trim(); // Skip "- ", "* ", or "+ "
                list_items.push(format!("  <li>{}</li>", self.parse_inline(content)));
                consumed += 1;
            } else if line.trim().is_empty() && !list_items.is_empty() {
                consumed += 1;
                // Check if next line is also a list item to continue
                if consumed < lines.len() && self.is_unordered_list_item(lines[consumed]) {
                    continue;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let content = list_items.join("\n");
        (format!("<ul>\n{}\n</ul>", content), consumed)
    }

    /// Check if line is an ordered list item
    fn is_ordered_list_item(&self, line: &str) -> bool {
        let trimmed = line.trim();
        if let Some(pos) = trimmed.find(". ") {
            let number_part = &trimmed[..pos];
            number_part.chars().all(|c| c.is_ascii_digit()) && !number_part.is_empty()
        } else {
            false
        }
    }

    /// Parse ordered list
    fn parse_ordered_list(&self, lines: &[&str]) -> (String, usize) {
        let mut list_items = Vec::new();
        let mut consumed = 0;

        for line in lines {
            if self.is_ordered_list_item(line) {
                let trimmed = line.trim();
                if let Some(pos) = trimmed.find(". ") {
                    let content = trimmed[pos + 2..].trim();
                    list_items.push(format!("  <li>{}</li>", self.parse_inline(content)));
                }
                consumed += 1;
            } else if line.trim().is_empty() && !list_items.is_empty() {
                consumed += 1;
                // Check if next line is also a list item to continue
                if consumed < lines.len() && self.is_ordered_list_item(lines[consumed]) {
                    continue;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let content = list_items.join("\n");
        (format!("<ol>\n{}\n</ol>", content), consumed)
    }

    /// Check if line starts a code block (indented with 4 spaces or 1 tab)
    fn is_code_block(&self, line: &str) -> bool {
        line.starts_with("    ") || line.starts_with("\t")
    }

    /// Parse code block
    fn parse_code_block(&self, lines: &[&str]) -> (String, usize) {
        let mut code_lines = Vec::new();
        let mut consumed = 0;

        for line in lines {
            if self.is_code_block(line) {
                // Remove the indentation (4 spaces or 1 tab)
                let content = if line.starts_with("    ") {
                    &line[4..]
                } else if line.starts_with("\t") {
                    &line[1..]
                } else {
                    line
                };
                code_lines.push(self.escape_html(content));
                consumed += 1;
            } else if line.trim().is_empty() {
                code_lines.push(String::new());
                consumed += 1;
            } else {
                break;
            }
        }

        let content = code_lines.join("\n");
        (format!("<pre><code>{}</code></pre>", content), consumed)
    }

    /// Check if line is a horizontal rule
    fn is_horizontal_rule(&self, line: &str) -> bool {
        let trimmed = line.trim();
        (trimmed.len() >= 3 && trimmed.chars().all(|c| c == '-')) ||
        (trimmed.len() >= 3 && trimmed.chars().all(|c| c == '*')) ||
        (trimmed.len() >= 3 && trimmed.chars().all(|c| c == '_'))
    }

    /// Parse inline elements (bold, italic, code, links, images)
    fn parse_inline(&self, text: &str) -> String {
        let mut result = text.to_string();
        
        // Parse in order: images, links, code, bold, italic
        result = self.parse_images(&result);
        result = self.parse_links(&result);
        result = self.parse_inline_code(&result);
        result = self.parse_bold(&result);
        result = self.parse_italic(&result);
        result = self.parse_strikethrough(&result);
        
        result
    }

    /// Parse images ![alt](url "title")
    fn parse_images(&self, text: &str) -> String {
        let mut result = String::new();
        let mut chars = text.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '!' && chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                
                // Parse alt text
                let mut alt = String::new();
                let mut bracket_count = 1;
                while let Some(ch) = chars.next() {
                    if ch == '[' {
                        bracket_count += 1;
                        alt.push(ch);
                    } else if ch == ']' {
                        bracket_count -= 1;
                        if bracket_count == 0 {
                            break;
                        }
                        alt.push(ch);
                    } else {
                        alt.push(ch);
                    }
                }
                
                // Check for (url)
                if chars.peek() == Some(&'(') {
                    chars.next(); // consume '('
                    let mut url = String::new();
                    let mut title = String::new();
                    let mut in_title = false;
                    let mut title_char = '"';
                    
                    while let Some(ch) = chars.next() {
                        if ch == ')' && !in_title {
                            break;
                        } else if (ch == '"' || ch == '\'') && !in_title {
                            in_title = true;
                            title_char = ch;
                        } else if ch == title_char && in_title {
                            in_title = false;
                        } else if in_title {
                            title.push(ch);
                        } else if ch != ' ' || !url.is_empty() {
                            url.push(ch);
                        }
                    }
                    
                    url = url.trim().to_string();
                    if !title.is_empty() {
                        result.push_str(&format!("<img src=\"{}\" alt=\"{}\" title=\"{}\">", 
                                               self.escape_html(&url), 
                                               self.escape_html(&alt), 
                                               self.escape_html(&title)));
                    } else {
                        result.push_str(&format!("<img src=\"{}\" alt=\"{}\">", 
                                               self.escape_html(&url), 
                                               self.escape_html(&alt)));
                    }
                } else {
                    result.push('!');
                    result.push('[');
                    result.push_str(&alt);
                    result.push(']');
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    }

    /// Parse links [text](url "title")
    fn parse_links(&self, text: &str) -> String {
        let mut result = String::new();
        let mut chars = text.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '[' {
                // Parse link text
                let mut link_text = String::new();
                let mut bracket_count = 1;
                while let Some(ch) = chars.next() {
                    if ch == '[' {
                        bracket_count += 1;
                        link_text.push(ch);
                    } else if ch == ']' {
                        bracket_count -= 1;
                        if bracket_count == 0 {
                            break;
                        }
                        link_text.push(ch);
                    } else {
                        link_text.push(ch);
                    }
                }
                
                // Check for (url)
                if chars.peek() == Some(&'(') {
                    chars.next(); // consume '('
                    let mut url = String::new();
                    let mut title = String::new();
                    let mut in_title = false;
                    let mut title_char = '"';
                    
                    while let Some(ch) = chars.next() {
                        if ch == ')' && !in_title {
                            break;
                        } else if (ch == '"' || ch == '\'') && !in_title {
                            in_title = true;
                            title_char = ch;
                        } else if ch == title_char && in_title {
                            in_title = false;
                        } else if in_title {
                            title.push(ch);
                        } else if ch != ' ' || !url.is_empty() {
                            url.push(ch);
                        }
                    }
                    
                    url = url.trim().to_string();
                    if !title.is_empty() {
                        result.push_str(&format!("<a href=\"{}\" title=\"{}\">{}</a>", 
                                               self.escape_html(&url), 
                                               self.escape_html(&title), 
                                               link_text));
                    } else {
                        result.push_str(&format!("<a href=\"{}\">{}</a>", 
                                               self.escape_html(&url), 
                                               link_text));
                    }
                } else {
                    result.push('[');
                    result.push_str(&link_text);
                    result.push(']');
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    }

    /// Parse inline code `code`
    fn parse_inline_code(&self, text: &str) -> String {
        let mut result = String::new();
        let mut chars = text.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '`' {
                let mut code = String::new();
                let mut found_closing = false;
                
                while let Some(ch) = chars.next() {
                    if ch == '`' {
                        found_closing = true;
                        break;
                    } else {
                        code.push(ch);
                    }
                }
                
                if found_closing {
                    result.push_str(&format!("<code>{}</code>", self.escape_html(&code)));
                } else {
                    result.push('`');
                    result.push_str(&code);
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    }

    /// Parse bold **text** or __text__
    fn parse_bold(&self, text: &str) -> String {
        let mut result = text.to_string();
        
        // Parse **text**
        while let Some(start) = result.find("**") {
            if let Some(end) = result[start + 2..].find("**") {
                let end = start + 2 + end;
                let bold_text = &result[start + 2..end];
                let replacement = format!("<strong>{}</strong>", bold_text);
                result.replace_range(start..end + 2, &replacement);
            } else {
                break;
            }
        }
        
        // Parse __text__
        while let Some(start) = result.find("__") {
            if let Some(end) = result[start + 2..].find("__") {
                let end = start + 2 + end;
                let bold_text = &result[start + 2..end];
                let replacement = format!("<strong>{}</strong>", bold_text);
                result.replace_range(start..end + 2, &replacement);
            } else {
                break;
            }
        }
        
        result
    }

    /// Parse italic *text* or _text_
    fn parse_italic(&self, text: &str) -> String {
        let mut result = text.to_string();
        
        // Parse *text*
        let mut i = 0;
        let chars: Vec<char> = result.chars().collect();
        let mut new_result = String::new();
        
        while i < chars.len() {
            if chars[i] == '*' && i + 1 < chars.len() && chars[i + 1] != '*' {
                // Find closing *
                let mut j = i + 1;
                while j < chars.len() && chars[j] != '*' {
                    j += 1;
                }
                
                if j < chars.len() && j > i + 1 {
                    // Found closing *, extract italic text
                    let italic_text: String = chars[i + 1..j].iter().collect();
                    new_result.push_str(&format!("<em>{}</em>", italic_text));
                    i = j + 1;
                } else {
                    new_result.push(chars[i]);
                    i += 1;
                }
            } else {
                new_result.push(chars[i]);
                i += 1;
            }
        }
        
        result = new_result;
        
        // Parse _text_
        i = 0;
        let chars: Vec<char> = result.chars().collect();
        new_result = String::new();
        
        while i < chars.len() {
            if chars[i] == '_' && i + 1 < chars.len() && chars[i + 1] != '_' {
                // Find closing _
                let mut j = i + 1;
                while j < chars.len() && chars[j] != '_' {
                    j += 1;
                }
                
                if j < chars.len() && j > i + 1 {
                    // Found closing _, extract italic text
                    let italic_text: String = chars[i + 1..j].iter().collect();
                    new_result.push_str(&format!("<em>{}</em>", italic_text));
                    i = j + 1;
                } else {
                    new_result.push(chars[i]);
                    i += 1;
                }
            } else {
                new_result.push(chars[i]);
                i += 1;
            }
        }
        
        new_result
    }

    /// Parse strikethrough ~~text~~
    fn parse_strikethrough(&self, text: &str) -> String {
        let mut result = text.to_string();
        while let Some(start) = result.find("~~") {
            if let Some(end) = result[start + 2..].find("~~") {
                let end = start + 2 + end;
                let strike_text = &result[start + 2..end];
                let replacement = format!("<del>{}</del>", strike_text);
                result.replace_range(start..end + 2, &replacement);
            } else {
                break;
            }
        }
        result
    }

    /// Escape HTML characters
    fn escape_html(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
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
    fn test_headings() {
        let parser = MarkdownParser::new();
        
        // Test all heading levels
        assert_eq!(parser.parse_heading("# Heading level 1"), Some("<h1>Heading level 1</h1>".to_string()));
        assert_eq!(parser.parse_heading("## Heading level 2"), Some("<h2>Heading level 2</h2>".to_string()));
        assert_eq!(parser.parse_heading("### Heading level 3"), Some("<h3>Heading level 3</h3>".to_string()));
        assert_eq!(parser.parse_heading("#### Heading level 4"), Some("<h4>Heading level 4</h4>".to_string()));
        assert_eq!(parser.parse_heading("##### Heading level 5"), Some("<h5>Heading level 5</h5>".to_string()));
        assert_eq!(parser.parse_heading("###### Heading level 6"), Some("<h6>Heading level 6</h6>".to_string()));
    }

    #[test]
    fn test_bold_and_italic() {
        let parser = MarkdownParser::new();
        
        assert_eq!(parser.parse_bold("I love **bold text**."), "I love <strong>bold text</strong>.");
        assert_eq!(parser.parse_italic("This is *italic text*."), "This is <em>italic text</em>.");
    }

    #[test]
    fn test_code() {
        let parser = MarkdownParser::new();
        
        assert_eq!(parser.parse_inline_code("Type `nano` command."), "Type <code>nano</code> command.");
    }

    #[test]
    fn test_links() {
        let parser = MarkdownParser::new();
        
        let result = parser.parse_links("Visit [Google](https://google.com) for search.");
        assert_eq!(result, "Visit <a href=\"https://google.com\">Google</a> for search.");
    }

    #[test]
    fn test_images() {
        let parser = MarkdownParser::new();
        
        let result = parser.parse_images("![Alt text](image.jpg \"Title\")");
        assert_eq!(result, "<img src=\"image.jpg\" alt=\"Alt text\" title=\"Title\">");
    }

    #[test]
    fn test_full_conversion() {
        let parser = MarkdownParser::new();
        
        let markdown = r#"# Main Title
This is a **bold** paragraph with *italic* text and `code`.

## Subtitle
- First item
- Second item

> This is a blockquote"#;

        let html = parser.to_html(markdown);
        assert!(html.contains("<h1>Main Title</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
        assert!(html.contains("<code>code</code>"));
        assert!(html.contains("<ul>"));
        assert!(html.contains("<blockquote>"));
    }
}
