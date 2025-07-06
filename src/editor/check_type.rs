use gtk4::prelude::*;
use gtk4::TextTag;
use pulldown_cmark::{Event, Parser, Tag};
use regex::Regex;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

/// Represents a Markdown syntax warning with location and type
#[derive(Debug, Clone, PartialEq)]
pub struct MarkdownWarning {
    pub line: usize,
    pub column: usize,
    pub start_offset: usize,
    pub end_offset: usize,
    pub warning_type: WarningType,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Types of Markdown syntax warnings
#[derive(Debug, Clone, PartialEq)]
pub enum WarningType {
    UncloseTag,
    ImproperNesting,
    BrokenLink,
    BrokenImage,
    ImproperHeading,
    RawHtml,
    UnclosedCodeBlock,
    MalformedTable,
    EmptyLink,
    InvalidReference,
    InconsistentListMarkers,
    MissingAltText,
    UnclosedEmphasis,
    InvalidTaskList,
    MalformedFootnote,
    UnclosedBlockquote,
    InvalidEscapeSequence,
}

impl std::fmt::Display for WarningType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WarningType::UncloseTag => write!(f, "Unclosed tag"),
            WarningType::ImproperNesting => write!(f, "Improper nesting"),
            WarningType::BrokenLink => write!(f, "Broken link"),
            WarningType::BrokenImage => write!(f, "Broken image"),
            WarningType::ImproperHeading => write!(f, "Improper heading"),
            WarningType::RawHtml => write!(f, "Raw HTML"),
            WarningType::UnclosedCodeBlock => write!(f, "Unclosed code block"),
            WarningType::MalformedTable => write!(f, "Malformed table"),
            WarningType::EmptyLink => write!(f, "Empty link"),
            WarningType::InvalidReference => write!(f, "Invalid reference"),
            WarningType::InconsistentListMarkers => write!(f, "Inconsistent list markers"),
            WarningType::MissingAltText => write!(f, "Missing alt text"),
            WarningType::UnclosedEmphasis => write!(f, "Unclosed emphasis"),
            WarningType::InvalidTaskList => write!(f, "Invalid task list"),
            WarningType::MalformedFootnote => write!(f, "Malformed footnote"),
            WarningType::UnclosedBlockquote => write!(f, "Unclosed blockquote"),
            WarningType::InvalidEscapeSequence => write!(f, "Invalid escape sequence"),
        }
    }
}

impl std::fmt::Display for MarkdownWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Warning: {} at line {}, column {}: {}",
            self.warning_type, self.line, self.column, self.message
        )?;
        if let Some(suggestion) = &self.suggestion {
            write!(f, " (Suggestion: {})", suggestion)?;
        }
        Ok(())
    }
}

/// Configuration for Markdown syntax checking
#[derive(Debug, Clone)]
pub struct MarkdownLintConfig {
    pub check_unclosed_tags: bool,
    pub check_improper_nesting: bool,
    pub check_broken_links: bool,
    pub check_broken_images: bool,
    pub check_improper_headings: bool,
    pub check_raw_html: bool,
    pub check_unclosed_code_blocks: bool,
    pub check_malformed_tables: bool,
    pub check_empty_links: bool,
    pub check_invalid_references: bool,
    pub check_inconsistent_list_markers: bool,
    pub check_missing_alt_text: bool,
    pub check_unclosed_emphasis: bool,
    pub check_invalid_task_lists: bool,
    pub check_malformed_footnotes: bool,
    pub check_unclosed_blockquotes: bool,
    pub check_invalid_escape_sequences: bool,
}

impl Default for MarkdownLintConfig {
    fn default() -> Self {
        Self {
            check_unclosed_tags: true,
            check_improper_nesting: true,
            check_broken_links: true,
            check_broken_images: true,
            check_improper_headings: true,
            check_raw_html: true,
            check_unclosed_code_blocks: true,
            check_malformed_tables: true,
            check_empty_links: true,
            check_invalid_references: true,
            check_inconsistent_list_markers: true,
            check_missing_alt_text: true,
            check_unclosed_emphasis: true,
            check_invalid_task_lists: true,
            check_malformed_footnotes: true,
            check_unclosed_blockquotes: true,
            check_invalid_escape_sequences: false, // Optional by default
        }
    }
}

/// Main Markdown syntax checker with GTK4 integration
pub struct MarkdownSyntaxChecker {
    config: MarkdownLintConfig,
    warnings: Vec<MarkdownWarning>,
    warning_tags: HashMap<String, TextTag>,
    buffer: Option<gtk4::TextBuffer>,
    
    // Regex patterns for advanced syntax checking
    heading_regex: Regex,
    emphasis_regex: Regex,
    link_regex: Regex,
    image_regex: Regex,
    code_block_regex: Regex,
    table_regex: Regex,
    task_list_regex: Regex,
    footnote_regex: Regex,
    html_tag_regex: Regex,
    reference_regex: Regex,
    escape_regex: Regex,
}

impl MarkdownSyntaxChecker {
    pub fn new(config: MarkdownLintConfig) -> Self {
        let warning_tags = Self::create_warning_tags();
        
        Self {
            config,
            warnings: Vec::new(),
            warning_tags,
            buffer: None,
            
            // Initialize regex patterns
            heading_regex: Regex::new(r"^(#{1,6})([^ #]|$)").unwrap(),
            emphasis_regex: Regex::new(r"(\*{1,2}|_{1,2})([^*_\n]*?)(\*{1,2}|_{1,2})").unwrap(),
            link_regex: Regex::new(r"\[([^\]]*)\]\(([^)]*)\)").unwrap(),
            image_regex: Regex::new(r"!\[([^\]]*)\]\(([^)]*)\)").unwrap(),
            code_block_regex: Regex::new(r"```(\w+)?\n(.*?)```").unwrap(),
            table_regex: Regex::new(r"\|.*\|").unwrap(),
            task_list_regex: Regex::new(r"^(\s*)-\s+\[([ xX])\]\s+(.*)$").unwrap(),
            footnote_regex: Regex::new(r"\[\^([^\]]+)\]").unwrap(),
            html_tag_regex: Regex::new(r"<[^>]+>").unwrap(),
            reference_regex: Regex::new(r"\[([^\]]+)\]:\s*(.+)").unwrap(),
            escape_regex: Regex::new(r"\\[^\\]").unwrap(),
        }
    }

    pub fn new_with_defaults() -> Self {
        Self::new(MarkdownLintConfig::default())
    }

    /// Create GTK TextTags for different warning types
    fn create_warning_tags() -> HashMap<String, TextTag> {
        let mut tags = HashMap::new();
        // Create tag for syntax warnings with wavy red underline
        let warning_tag = TextTag::new(Some("markdown-warning"));
        warning_tag.set_property("underline", gtk4::pango::Underline::Error);
        warning_tag.set_property("underline-rgba", &gdk4::RGBA::new(1.0, 0.0, 0.0, 1.0));
        tags.insert("warning".to_string(), warning_tag);

        // Create tag for broken links/images
        let broken_link_tag = TextTag::new(Some("broken-link"));
        broken_link_tag.set_property("underline", gtk4::pango::Underline::Error);
        broken_link_tag.set_property("underline-rgba", &gdk4::RGBA::new(1.0, 0.5, 0.0, 1.0));
        tags.insert("broken-link".to_string(), broken_link_tag);

        // Create tag for HTML warnings
        let html_warning_tag = TextTag::new(Some("html-warning"));
        html_warning_tag.set_property("underline", gtk4::pango::Underline::Single);
        html_warning_tag.set_property("underline-rgba", &gdk4::RGBA::new(0.8, 0.8, 0.0, 1.0));
        tags.insert("html-warning".to_string(), html_warning_tag);

        // Create tag for structure issues (blue underline)
        let structure_tag = TextTag::new(Some("structure-warning"));
        structure_tag.set_property("underline", gtk4::pango::Underline::Single);
        structure_tag.set_property("underline-rgba", &gdk4::RGBA::new(0.2, 0.4, 1.0, 1.0));
        tags.insert("structure-warning".to_string(), structure_tag);

        tags
    }

    /// Create GTK TextTags for a specific buffer to avoid conflicts
    fn create_warning_tags_for_buffer(buffer: &gtk4::TextBuffer) -> HashMap<String, TextTag> {
        let mut tags = HashMap::new();
        let tag_table = buffer.tag_table();
        
        // Create or reuse tag for syntax warnings with wavy red underline
        let warning_tag = if let Some(existing_tag) = tag_table.lookup("markdown-warning") {
            existing_tag
        } else {
            let tag = TextTag::new(Some("markdown-warning"));
            tag.set_property("underline", gtk4::pango::Underline::Error);
            tag.set_property("underline-rgba", &gdk4::RGBA::new(1.0, 0.0, 0.0, 1.0));
            tag_table.add(&tag);
            tag
        };
        tags.insert("warning".to_string(), warning_tag);

        // Create or reuse tag for broken links/images
        let broken_link_tag = if let Some(existing_tag) = tag_table.lookup("broken-link") {
            existing_tag
        } else {
            let tag = TextTag::new(Some("broken-link"));
            tag.set_property("underline", gtk4::pango::Underline::Error);
            tag.set_property("underline-rgba", &gdk4::RGBA::new(1.0, 0.5, 0.0, 1.0));
            tag_table.add(&tag);
            tag
        };
        tags.insert("broken-link".to_string(), broken_link_tag);

        // Create or reuse tag for HTML warnings
        let html_warning_tag = if let Some(existing_tag) = tag_table.lookup("html-warning") {
            existing_tag
        } else {
            let tag = TextTag::new(Some("html-warning"));
            tag.set_property("underline", gtk4::pango::Underline::Single);
            tag.set_property("underline-rgba", &gdk4::RGBA::new(0.8, 0.8, 0.0, 1.0));
            tag_table.add(&tag);
            tag
        };
        tags.insert("html-warning".to_string(), html_warning_tag);

        // Create or reuse tag for structure issues (blue underline)
        let structure_tag = if let Some(existing_tag) = tag_table.lookup("structure-warning") {
            existing_tag
        } else {
            let tag = TextTag::new(Some("structure-warning"));
            tag.set_property("underline", gtk4::pango::Underline::Single);
            tag.set_property("underline-rgba", &gdk4::RGBA::new(0.2, 0.4, 1.0, 1.0));
            tag_table.add(&tag);
            tag
        };
        tags.insert("structure-warning".to_string(), structure_tag);

        tags
    }

    /// Set the text buffer for applying visual warnings
    pub fn set_buffer(&mut self, buffer: &gtk4::TextBuffer) {
        self.buffer = Some(buffer.clone());
        
        // Create fresh tags for this buffer to avoid conflicts
        self.warning_tags = Self::create_warning_tags_for_buffer(buffer);
    }

    /// Lint markdown content and return warnings
    pub fn lint(&mut self, content: &str) -> Vec<MarkdownWarning> {
        self.warnings.clear();
        
        // First pass: use pulldown-cmark for AST-based checking
        self.ast_based_checks(content);
        
        // Second pass: regex-based checks for syntax not covered by AST
        self.regex_based_checks(content);
        
        // Third pass: multi-line structural checks
        self.structural_checks(content);
        
        // Apply visual warnings to the buffer if available
        if let Some(buffer) = &self.buffer {
            self.apply_visual_warnings(buffer, content);
        }
        
        self.warnings.clone()
    }

    /// AST-based checks using pulldown-cmark
    fn ast_based_checks(&mut self, content: &str) {
        let parser = Parser::new(content);
        let mut event_stack = Vec::new();
        let mut current_offset = 0;
        let line_offsets = self.build_line_offsets(content);
        
        for event in parser {
            match event {
                Event::Start(tag) => {
                    event_stack.push((tag, current_offset));
                }
                Event::End(tag) => {
                    if let Some((start_tag, start_offset)) = event_stack.pop() {
                        if std::mem::discriminant(&start_tag) != std::mem::discriminant(&tag) {
                            // Mismatched tags
                            let (line, col) = self.offset_to_line_col(start_offset, &line_offsets);
                            self.warnings.push(MarkdownWarning {
                                line,
                                column: col,
                                start_offset,
                                end_offset: current_offset,
                                warning_type: WarningType::ImproperNesting,
                                message: format!("Mismatched tag: expected {:?}, found {:?}", start_tag, tag),
                                suggestion: Some("Check tag nesting".to_string()),
                            });
                        }
                    }
                }
                Event::Text(text) => {
                    current_offset += text.len();
                }
                Event::Code(code) => {
                    current_offset += code.len();
                }
                Event::Html(html) => {
                    if self.config.check_raw_html {
                        let (line, col) = self.offset_to_line_col(current_offset, &line_offsets);
                        self.warnings.push(MarkdownWarning {
                            line,
                            column: col,
                            start_offset: current_offset,
                            end_offset: current_offset + html.len(),
                            warning_type: WarningType::RawHtml,
                            message: format!("Raw HTML found: {}", html.trim()),
                            suggestion: Some("Consider using Markdown syntax instead".to_string()),
                        });
                    }
                    current_offset += html.len();
                }
                _ => {}
            }
        }
        
        // Check for unclosed tags
        if !event_stack.is_empty() {
            for (tag, start_offset) in event_stack {
                let (line, col) = self.offset_to_line_col(start_offset, &line_offsets);
                self.warnings.push(MarkdownWarning {
                    line,
                    column: col,
                    start_offset,
                    end_offset: start_offset,
                    warning_type: WarningType::UncloseTag,
                    message: format!("Unclosed tag: {:?}", tag),
                    suggestion: Some("Close the tag properly".to_string()),
                });
            }
        }
    }

    /// Regex-based checks for syntax patterns
    fn regex_based_checks(&mut self, content: &str) {
        let lines: Vec<&str> = content.lines().collect();
        let line_offsets = self.build_line_offsets(content);
        
        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num + 1;
            let line_start_offset = if line_num == 1 { 0 } else { line_offsets[line_num - 2] };
            
            // Check headings
            if self.config.check_improper_headings {
                self.check_heading_syntax(line, line_num, line_start_offset);
            }
            
            // Check emphasis
            if self.config.check_unclosed_emphasis {
                self.check_emphasis_syntax(line, line_num, line_start_offset);
            }
            
            // Check links
            if self.config.check_broken_links || self.config.check_empty_links {
                self.check_link_syntax(line, line_num, line_start_offset);
            }
            
            // Check images
            if self.config.check_broken_images || self.config.check_missing_alt_text {
                self.check_image_syntax(line, line_num, line_start_offset);
            }
            
            // Check task lists
            if self.config.check_invalid_task_lists {
                self.check_task_list_syntax(line, line_num, line_start_offset);
            }
            
            // Check footnotes
            if self.config.check_malformed_footnotes {
                self.check_footnote_syntax(line, line_num, line_start_offset);
            }
            
            // Check HTML tags
            if self.config.check_raw_html {
                self.check_html_syntax(line, line_num, line_start_offset);
            }
        }
    }

    /// Structural checks for multi-line patterns
    fn structural_checks(&mut self, content: &str) {
        let lines: Vec<&str> = content.lines().collect();
        
        if self.config.check_unclosed_code_blocks {
            self.check_code_blocks(&lines);
        }
        
        if self.config.check_malformed_tables {
            self.check_table_structure(&lines);
        }
        
        if self.config.check_invalid_references {
            self.check_reference_links(&lines);
        }
        
        if self.config.check_inconsistent_list_markers {
            self.check_list_consistency(&lines);
        }
    }

    /// Check heading syntax
    fn check_heading_syntax(&mut self, line: &str, line_num: usize, line_start_offset: usize) {
        if let Some(caps) = self.heading_regex.captures(line) {
            let hashes = caps.get(1).unwrap().as_str();
            let column = caps.get(1).unwrap().end();
            let start_offset = line_start_offset + caps.get(1).unwrap().start();
            let end_offset = line_start_offset + caps.get(1).unwrap().end();
            
            self.warnings.push(MarkdownWarning {
                line: line_num,
                column,
                start_offset,
                end_offset,
                warning_type: WarningType::ImproperHeading,
                message: format!("Heading should have a space after `{}`", hashes),
                suggestion: Some(format!("Change to `{} `", hashes)),
            });
        }
    }

    /// Check emphasis syntax (bold/italic)
    fn check_emphasis_syntax(&mut self, line: &str, line_num: usize, line_start_offset: usize) {
        // Check for unmatched emphasis markers
        let mut bold_count = 0;
        let mut italic_count = 0;
        let mut i = 0;
        let chars: Vec<char> = line.chars().collect();
        
        while i < chars.len() {
            if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
                bold_count += 1;
                i += 2;
            } else if chars[i] == '*' || chars[i] == '_' {
                italic_count += 1;
                i += 1;
            } else {
                i += 1;
            }
        }
        
        if bold_count % 2 != 0 {
            self.warnings.push(MarkdownWarning {
                line: line_num,
                column: 1,
                start_offset: line_start_offset,
                end_offset: line_start_offset + line.len(),
                warning_type: WarningType::UnclosedEmphasis,
                message: "Unclosed bold marker (**)".to_string(),
                suggestion: Some("Ensure bold markers are properly paired".to_string()),
            });
        }
        
        if italic_count % 2 != 0 {
            self.warnings.push(MarkdownWarning {
                line: line_num,
                column: 1,
                start_offset: line_start_offset,
                end_offset: line_start_offset + line.len(),
                warning_type: WarningType::UnclosedEmphasis,
                message: "Unclosed italic marker (* or _)".to_string(),
                suggestion: Some("Ensure italic markers are properly paired".to_string()),
            });
        }
    }

    /// Check link syntax
    fn check_link_syntax(&mut self, line: &str, line_num: usize, line_start_offset: usize) {
        for caps in self.link_regex.captures_iter(line) {
            let full_match = caps.get(0).unwrap();
            let _text = caps.get(1).unwrap().as_str();
            let url = caps.get(2).unwrap().as_str();
            
            let start_offset = line_start_offset + full_match.start();
            let end_offset = line_start_offset + full_match.end();
            
            // Check for empty links
            if self.config.check_empty_links && url.trim().is_empty() {
                self.warnings.push(MarkdownWarning {
                    line: line_num,
                    column: full_match.start() + 1,
                    start_offset,
                    end_offset,
                    warning_type: WarningType::EmptyLink,
                    message: "Link with empty URL".to_string(),
                    suggestion: Some("Provide a valid URL or remove the link".to_string()),
                });
            }
        }
        
        // Check for broken link syntax
        let broken_link_regex = Regex::new(r"\[([^\]]*)\]\([^)]*$|\[([^\]]*)\]$|\[([^\]]*)$").unwrap();
        for mat in broken_link_regex.find_iter(line) {
            let start_offset = line_start_offset + mat.start();
            let end_offset = line_start_offset + mat.end();
            
            self.warnings.push(MarkdownWarning {
                line: line_num,
                column: mat.start() + 1,
                start_offset,
                end_offset,
                warning_type: WarningType::BrokenLink,
                message: "Incomplete link syntax".to_string(),
                suggestion: Some("Ensure link has format [text](url)".to_string()),
            });
        }
    }

    /// Check image syntax
    fn check_image_syntax(&mut self, line: &str, line_num: usize, line_start_offset: usize) {
        for caps in self.image_regex.captures_iter(line) {
            let full_match = caps.get(0).unwrap();
            let alt_text = caps.get(1).unwrap().as_str();
            let _src = caps.get(2).unwrap().as_str();
            
            let start_offset = line_start_offset + full_match.start();
            let end_offset = line_start_offset + full_match.end();
            
            // Check for missing alt text
            if self.config.check_missing_alt_text && alt_text.trim().is_empty() {
                self.warnings.push(MarkdownWarning {
                    line: line_num,
                    column: full_match.start() + 1,
                    start_offset,
                    end_offset,
                    warning_type: WarningType::MissingAltText,
                    message: "Image missing alt text".to_string(),
                    suggestion: Some("Add descriptive alt text for accessibility".to_string()),
                });
            }
        }
    }

    /// Check task list syntax
    fn check_task_list_syntax(&mut self, line: &str, line_num: usize, line_start_offset: usize) {
        // Check for malformed task lists
        let malformed_task_regex = Regex::new(r"^(\s*)-\s+\[([^\]xX ])\]").unwrap();
        if let Some(caps) = malformed_task_regex.captures(line) {
            let checkbox = caps.get(2).unwrap().as_str();
            let start_offset = line_start_offset + caps.get(0).unwrap().start();
            let end_offset = line_start_offset + caps.get(0).unwrap().end();
            
            self.warnings.push(MarkdownWarning {
                line: line_num,
                column: caps.get(0).unwrap().start() + 1,
                start_offset,
                end_offset,
                warning_type: WarningType::InvalidTaskList,
                message: format!("Invalid task list checkbox: [{}]", checkbox),
                suggestion: Some("Use [ ] for unchecked or [x] for checked".to_string()),
            });
        }
    }

    /// Check footnote syntax
    fn check_footnote_syntax(&mut self, line: &str, line_num: usize, line_start_offset: usize) {
        // Check for malformed footnotes - find footnote references that are not definitions
        let footnote_regex = Regex::new(r"\[\^([^\]]*)\]").unwrap();
        
        for caps in footnote_regex.captures_iter(line) {
            let full_match = caps.get(0).unwrap();
            let footnote_id = caps.get(1).unwrap().as_str();
            let start_offset = line_start_offset + full_match.start();
            let end_offset = line_start_offset + full_match.end();
            
            // Check if this is actually a footnote definition (contains colon after)
            let remainder = &line[full_match.end()..];
            let is_definition = remainder.trim_start().starts_with(':');
            
            // Only warn about empty IDs in footnote references, not definitions
            if !is_definition && footnote_id.trim().is_empty() {
                self.warnings.push(MarkdownWarning {
                    line: line_num,
                    column: caps.get(0).unwrap().start() + 1,
                    start_offset,
                    end_offset,
                    warning_type: WarningType::MalformedFootnote,
                    message: "Footnote reference with empty ID".to_string(),
                    suggestion: Some("Provide a valid footnote ID".to_string()),
                });
            }
        }
    }

    /// Check HTML syntax
    fn check_html_syntax(&mut self, line: &str, line_num: usize, line_start_offset: usize) {
        for mat in self.html_tag_regex.find_iter(line) {
            let start_offset = line_start_offset + mat.start();
            let end_offset = line_start_offset + mat.end();
            
            self.warnings.push(MarkdownWarning {
                line: line_num,
                column: mat.start() + 1,
                start_offset,
                end_offset,
                warning_type: WarningType::RawHtml,
                message: format!("Raw HTML tag found: {}", mat.as_str()),
                suggestion: Some("Consider using Markdown syntax instead".to_string()),
            });
        }
    }

    /// Check code blocks
    fn check_code_blocks(&mut self, lines: &[&str]) {
        let mut in_code_block = false;
        let mut code_block_start = 0;
        let mut code_block_start_offset = 0;
        let mut current_offset = 0;
        
        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num + 1;
            
            if line.starts_with("```") {
                if in_code_block {
                    in_code_block = false;
                } else {
                    in_code_block = true;
                    code_block_start = line_num;
                    code_block_start_offset = current_offset;
                }
            }
            current_offset += line.len() + 1; // +1 for newline
        }
        
        if in_code_block {
            self.warnings.push(MarkdownWarning {
                line: code_block_start,
                column: 1,
                start_offset: code_block_start_offset,
                end_offset: code_block_start_offset + 3, // Length of "```"
                warning_type: WarningType::UnclosedCodeBlock,
                message: "Unclosed code block".to_string(),
                suggestion: Some("Add closing ``` to end the code block".to_string()),
            });
        }
    }

    /// Check table structure
    fn check_table_structure(&mut self, lines: &[&str]) {
        let mut in_table = false;
        let mut expected_columns = 0;
        let mut current_offset = 0;
        
        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num + 1;
            
            if line.contains('|') {
                let current_columns = line.matches('|').count();
                
                if !in_table {
                    in_table = true;
                    expected_columns = current_columns;
                } else if current_columns != expected_columns {
                    self.warnings.push(MarkdownWarning {
                        line: line_num,
                        column: 1,
                        start_offset: current_offset,
                        end_offset: current_offset + line.len(),
                        warning_type: WarningType::MalformedTable,
                        message: format!("Table row has {} columns, expected {}", current_columns, expected_columns),
                        suggestion: Some("Ensure all table rows have the same number of columns".to_string()),
                    });
                }
            } else if in_table && line.trim().is_empty() {
                in_table = false;
            }
            current_offset += line.len() + 1; // +1 for newline
        }
    }

    /// Check reference links
    fn check_reference_links(&mut self, lines: &[&str]) {
        let mut references = HashMap::new();
        let mut current_offset = 0;
        
        // Collect reference definitions
        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num + 1;
            
            if let Some(caps) = self.reference_regex.captures(line) {
                let ref_name = caps.get(1).unwrap().as_str().to_lowercase();
                references.insert(ref_name, line_num);
            }
            current_offset += line.len() + 1; // +1 for newline
        }
        
        // Reset offset for second pass
        current_offset = 0;
        
        // Find reference uses
        let reference_use_regex = Regex::new(r"\[([^\]]+)\]\[([^\]]*)\]").unwrap();
        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num + 1;
            
            for caps in reference_use_regex.captures_iter(line) {
                let ref_name = if caps.get(2).unwrap().as_str().is_empty() {
                    caps.get(1).unwrap().as_str().to_lowercase()
                } else {
                    caps.get(2).unwrap().as_str().to_lowercase()
                };
                
                if !references.contains_key(&ref_name) {
                    let match_start = caps.get(0).unwrap().start();
                    let match_end = caps.get(0).unwrap().end();
                    
                    self.warnings.push(MarkdownWarning {
                        line: line_num,
                        column: match_start + 1,
                        start_offset: current_offset + match_start,
                        end_offset: current_offset + match_end,
                        warning_type: WarningType::InvalidReference,
                        message: format!("Reference '{}' not found", ref_name),
                        suggestion: Some("Define the reference or use a direct link".to_string()),
                    });
                }
            }
            current_offset += line.len() + 1; // +1 for newline
        }
    }

    /// Check list consistency
    fn check_list_consistency(&mut self, lines: &[&str]) {
        let mut list_markers = Vec::new();
        let mut in_list = false;
        let mut current_offset = 0;
        
        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num + 1;
            
            let list_marker_regex = Regex::new(r"^(\s*)([-*+])\s").unwrap();
            if let Some(caps) = list_marker_regex.captures(line) {
                let indent = caps.get(1).unwrap().as_str();
                let marker = caps.get(2).unwrap().as_str();
                
                if !in_list {
                    in_list = true;
                    list_markers.clear();
                    list_markers.push((indent.len(), marker.chars().next().unwrap()));
                } else {
                    let current_indent = indent.len();
                    let current_marker = marker.chars().next().unwrap();
                    
                    if let Some((_, expected_marker)) = list_markers.iter().find(|(i, _)| *i == current_indent) {
                        if *expected_marker != current_marker {
                            let marker_start = caps.get(2).unwrap().start();
                            let marker_end = caps.get(2).unwrap().end();
                            
                            self.warnings.push(MarkdownWarning {
                                line: line_num,
                                column: marker_start + 1,
                                start_offset: current_offset + marker_start,
                                end_offset: current_offset + marker_end,
                                warning_type: WarningType::InconsistentListMarkers,
                                message: format!("Inconsistent list marker '{}', expected '{}'", current_marker, expected_marker),
                                suggestion: Some("Use consistent list markers throughout the list".to_string()),
                            });
                        }
                    } else {
                        list_markers.push((current_indent, current_marker));
                    }
                }
            } else if in_list && line.trim().is_empty() {
                // Continue - empty lines are okay within lists
            } else if in_list && !line.starts_with(' ') {
                in_list = false;
            }
            current_offset += line.len() + 1; // +1 for newline
        }
    }

    /// Apply visual warnings to the text buffer
    fn apply_visual_warnings(&self, buffer: &gtk4::TextBuffer, _content: &str) {
        // Clear existing warning tags - only remove tags that belong to this buffer
        let start_iter = buffer.start_iter();
        let end_iter = buffer.end_iter();
        let tag_table = buffer.tag_table();
        
        // Only remove tags that actually exist in this buffer's tag table
        if let Some(tag) = tag_table.lookup("markdown-warning") {
            buffer.remove_tag(&tag, &start_iter, &end_iter);
        }
        if let Some(tag) = tag_table.lookup("broken-link") {
            buffer.remove_tag(&tag, &start_iter, &end_iter);
        }
        if let Some(tag) = tag_table.lookup("html-warning") {
            buffer.remove_tag(&tag, &start_iter, &end_iter);
        }
        if let Some(tag) = tag_table.lookup("structure-warning") {
            buffer.remove_tag(&tag, &start_iter, &end_iter);
        }

        // Apply new warning tags
        for warning in &self.warnings {
            let start_iter = buffer.iter_at_offset(warning.start_offset as i32);
            let end_iter = buffer.iter_at_offset(warning.end_offset as i32);

            let tag_name = match warning.warning_type {
                WarningType::BrokenLink | WarningType::BrokenImage | WarningType::EmptyLink => "broken-link",
                WarningType::RawHtml => "html-warning",
                // Structure issues get blue underline
                WarningType::UncloseTag
                | WarningType::ImproperNesting
                | WarningType::UnclosedCodeBlock
                | WarningType::MalformedTable
                | WarningType::InvalidReference
                | WarningType::InconsistentListMarkers
                | WarningType::MalformedFootnote
                | WarningType::UnclosedBlockquote => "structure-warning",
                _ => "warning",
            };

            if let Some(tag) = self.warning_tags.get(tag_name) {
                buffer.apply_tag(tag, &start_iter, &end_iter);
            }
        }
    }

    /// Build line offset mapping
    fn build_line_offsets(&self, content: &str) -> Vec<usize> {
        let mut offsets = Vec::new();
        let mut current_offset = 0;
        
        for line in content.lines() {
            current_offset += line.len() + 1; // +1 for newline
            offsets.push(current_offset);
        }
        
        offsets
    }

    /// Convert byte offset to line and column
    fn offset_to_line_col(&self, offset: usize, line_offsets: &[usize]) -> (usize, usize) {
        for (line_num, &line_offset) in line_offsets.iter().enumerate() {
            if offset < line_offset {
                let line_start = if line_num == 0 { 0 } else { line_offsets[line_num - 1] };
                let column = offset - line_start + 1;
                return (line_num + 1, column);
            }
        }
        
        (line_offsets.len() + 1, 1)
    }

    /// Get all warnings
    pub fn get_warnings(&self) -> &[MarkdownWarning] {
        &self.warnings
    }

    /// Clear all warnings and visual indicators
    pub fn clear_warnings(&mut self) {
        self.warnings.clear();
        
        if let Some(buffer) = &self.buffer {
            let start_iter = buffer.start_iter();
            let end_iter = buffer.end_iter();
            let tag_table = buffer.tag_table();
            
            // Only remove tags that actually exist in this buffer's tag table
            if let Some(tag) = tag_table.lookup("markdown-warning") {
                buffer.remove_tag(&tag, &start_iter, &end_iter);
            }
            if let Some(tag) = tag_table.lookup("broken-link") {
                buffer.remove_tag(&tag, &start_iter, &end_iter);
            }
            if let Some(tag) = tag_table.lookup("html-warning") {
                buffer.remove_tag(&tag, &start_iter, &end_iter);
            }
            if let Some(tag) = tag_table.lookup("structure-warning") {
                buffer.remove_tag(&tag, &start_iter, &end_iter);
            }
        }
    }

    /// Format warnings as text
    pub fn format_warnings(&self) -> String {
        self.warnings
            .iter()
            .map(|w| w.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Convenience function to create a syntax checker with default config
pub fn create_syntax_checker() -> MarkdownSyntaxChecker {
    MarkdownSyntaxChecker::new_with_defaults()
}

/// Convenience function to create a syntax checker with custom config
pub fn create_syntax_checker_with_config(config: MarkdownLintConfig) -> MarkdownSyntaxChecker {
    MarkdownSyntaxChecker::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_improper_heading() {
        let mut checker = MarkdownSyntaxChecker::new_with_defaults();
        let warnings = checker.lint("##No space after hash");
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].warning_type, WarningType::ImproperHeading);
    }

    #[test]
    fn test_unclosed_emphasis() {
        let mut checker = MarkdownSyntaxChecker::new_with_defaults();
        let warnings = checker.lint("This is **bold without closing");
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].warning_type, WarningType::UnclosedEmphasis);
    }

    #[test]
    fn test_empty_link() {
        let mut checker = MarkdownSyntaxChecker::new_with_defaults();
        let warnings = checker.lint("This is an [empty link]()");
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].warning_type, WarningType::EmptyLink);
    }

    #[test]
    fn test_missing_alt_text() {
        let mut checker = MarkdownSyntaxChecker::new_with_defaults();
        let warnings = checker.lint("This is an image ![](image.png)");
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].warning_type, WarningType::MissingAltText);
    }

    #[test]
    fn test_unclosed_code_block() {
        let mut checker = MarkdownSyntaxChecker::new_with_defaults();
        let warnings = checker.lint("```rust\nfn main() {}\n// Missing closing ```");
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].warning_type, WarningType::UnclosedCodeBlock);
    }

    #[test]
    fn test_valid_markdown() {
        let mut checker = MarkdownSyntaxChecker::new_with_defaults();
        let warnings = checker.lint("# Heading\n\nThis is **bold** and *italic* text.\n\n[Link](https://example.com)");
        assert_eq!(warnings.len(), 0);
    }
}
