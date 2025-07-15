use gio::prelude::*;
use gio::Settings;
use crate::utils::debouncer::Debouncer;
use rayon::prelude::*;
/// Spellcheck all words in Markdown-aware text spans, returning precise SpellErrors for misspellings.
fn spellcheck_text_spans(content: &str) -> Vec<SpellError> {
    let spans = extract_text_spans_with_offsets(content);
    let mut errors = Vec::new();
    // Use the static zspell dictionary
    let dict = match &*SPELL_DICTIONARY {
        Some(d) => d,
        None => {
            // If dictionary failed to load, return a single error
            errors.push(SpellError {
                line: 1,
                column: 1,
                start_offset: 0,
                end_offset: 0,
                warning_type: SpellType::InvalidEscapeSequence, // Use a custom type for spelling
                message: "Spellcheck dictionary not loaded. Please provide dictionaries/en_US.aff and dictionaries/en_US.dic".to_string(),
                suggestion: Some("Download Hunspell en_US dictionary files and place them in the dictionaries/ folder.".to_string()),
            });
            return errors;
        }
    };
    // Collect all (word, start, end) tuples
    let mut word_offsets = Vec::new();
    for (span, span_offset) in &spans {
        for mat in word_regex().find_iter(span) {
            let word = mat.as_str();
            let start = span_offset + mat.start();
            let end = span_offset + mat.end();
            word_offsets.push((word.to_string(), start, end));
        }
    }
    // Build line offsets for offset-to-line/col mapping
    let line_offsets = build_line_offsets(content);
    // Spellcheck in parallel
    let misspelled: Vec<_> = word_offsets.par_iter().filter_map(|(word, start, end)| {
        if !dict.check_word(word) {
            Some((word.clone(), *start, *end))
        } else {
            None
        }
    }).collect();
    // Map to SpellError with line/col
    for (word, start, end) in misspelled {
        let (line, column) = offset_to_line_col(start, &line_offsets);
        errors.push(SpellError {
            line,
            column,
            start_offset: start,
            end_offset: end,
            warning_type: SpellType::InvalidEscapeSequence, // Use a custom type for spelling
            message: format!("Misspelled word: {}", word),
            suggestion: None,
        });
    }
    errors
}
// Standalone version for use in spellcheck_text_spans
fn build_line_offsets(content: &str) -> Vec<usize> {
    let mut offsets = Vec::new();
    let mut current_offset = 0;
    for line in content.lines() {
        current_offset += line.len() + 1; // +1 for newline
        offsets.push(current_offset);
    }
    offsets
}

// Standalone version for use in spellcheck_text_spans
fn offset_to_line_col(offset: usize, line_offsets: &[usize]) -> (usize, usize) {
    for (line_num, &line_offset) in line_offsets.iter().enumerate() {
        if offset < line_offset {
            let line_start = if line_num == 0 {
                0
            } else {
                line_offsets[line_num - 1]
            };
            let column = offset - line_start + 1;
            return (line_num + 1, column);
        }
    }
    (line_offsets.len() + 1, 1)
}
use pulldown_cmark::{Parser, Event, Tag};
/// Extracts all text spans (with offsets) from Markdown, skipping code blocks, inline code, and links.
fn extract_text_spans_with_offsets(content: &str) -> Vec<(String, usize)> {
    let parser = Parser::new(content);
    let mut spans = Vec::new();
    let mut in_code_block = false;
    let mut in_link = false;
    let mut last_text_offset = 0;
    let mut skip_next_text = false;
    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(_)) => in_code_block = true,
            Event::End(Tag::CodeBlock(_)) => in_code_block = false,
            Event::Start(Tag::Link(_, _, _)) => in_link = true,
            Event::End(Tag::Link(_, _, _)) => in_link = false,
            Event::Code(_) => skip_next_text = true, // Next Event::Text is inline code
            Event::Text(ref text) if !in_code_block && !in_link && !skip_next_text => {
                // Find the offset of this text in the original content
                if let Some(pos) = content[last_text_offset..].find(text.as_ref()) {
                    let abs_offset = last_text_offset + pos;
                    spans.push((text.to_string(), abs_offset));
                    last_text_offset = abs_offset + text.len();
                }
            }
            Event::Text(_) => {
                if skip_next_text {
                    skip_next_text = false;
                }
            },
            _ => {},
        }
    }
    spans
}
use once_cell::sync::Lazy;
use std::fs;
use zspell::Dictionary;
// Unicode-aware word regex (minimum 2 letters)
use crate::utils::cache::get_regex;

// Unicode-aware word regex (minimum 2 letters)
// Use the global regex cache for consistency
fn word_regex() -> regex::Regex {
    get_regex(r"\p{L}{2,}")
}

/// Loads the Hunspell dictionary from language/dic/en.aff and language/dic/en.dic.
/// You can replace these files with your preferred language.
static SPELL_DICTIONARY: Lazy<Option<Dictionary>> = Lazy::new(|| {
    let aff_path = "language/dic/en.aff";
    let dic_path = "language/dic/en.dic";
    let aff_content = fs::read_to_string(aff_path).ok()?;
    let dic_content = fs::read_to_string(dic_path).ok()?;
    zspell::DictBuilder::new()
        .config_str(&aff_content)
        .dict_str(&dic_content)
        .build()
        .ok()
});
use gtk4::prelude::*;
use gtk4::TextTag;
// Duplicate import removed: use pulldown_cmark::{Event, Parser};
use regex::Regex;
use std::collections::HashMap;

/// Represents a Markdown syntax warning with location and type
#[derive(Debug, Clone, PartialEq)]
pub struct SpellError {
    pub line: usize,
    pub column: usize,
    pub start_offset: usize,
    pub end_offset: usize,
    pub warning_type: SpellType,
    pub message: String,
    pub suggestion: Option<String>,
}


/// Types of Markdown syntax warnings
#[derive(Debug, Clone, PartialEq)]
pub enum SpellType {
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

impl std::fmt::Display for SpellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpellType::UncloseTag => write!(f, "Unclosed tag"),
            SpellType::ImproperNesting => write!(f, "Improper nesting"),
            SpellType::BrokenLink => write!(f, "Broken link"),
            SpellType::BrokenImage => write!(f, "Broken image"),
            SpellType::ImproperHeading => write!(f, "Improper heading"),
            SpellType::RawHtml => write!(f, "Raw HTML"),
            SpellType::UnclosedCodeBlock => write!(f, "Unclosed code block"),
            SpellType::MalformedTable => write!(f, "Malformed table"),
            SpellType::EmptyLink => write!(f, "Empty link"),
            SpellType::InvalidReference => write!(f, "Invalid reference"),
            SpellType::InconsistentListMarkers => write!(f, "Inconsistent list markers"),
            SpellType::MissingAltText => write!(f, "Missing alt text"),
            SpellType::UnclosedEmphasis => write!(f, "Unclosed emphasis"),
            SpellType::InvalidTaskList => write!(f, "Invalid task list"),
            SpellType::MalformedFootnote => write!(f, "Malformed footnote"),
            SpellType::UnclosedBlockquote => write!(f, "Unclosed blockquote"),
            SpellType::InvalidEscapeSequence => write!(f, "Invalid escape sequence"),
        }
    }
}
impl std::fmt::Display for SpellError {
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
pub struct SpellLintConfig {
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
    pub enable_spellcheck: bool,
}

impl Default for SpellLintConfig {
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
            enable_spellcheck: true,
        }
    }
}

/// Main Markdown syntax checker with GTK4 integration
pub struct SpellSyntaxChecker {
    config: SpellLintConfig,
    warnings: Vec<SpellError>,
    warning_tags: HashMap<String, TextTag>,
    buffer: Option<gtk4::TextBuffer>,

    // Debouncer for live spellcheck
    debouncer: Option<Debouncer>,
    settings: Option<Settings>,

    // Regex patterns for advanced syntax checking
    heading_regex: Regex,
    link_regex: Regex,
    image_regex: Regex,
    html_tag_regex: Regex,
    reference_regex: Regex,
}

impl SpellSyntaxChecker {
    /// Clear all visual warning tags from the buffer immediately.
    ///
    /// # Safety
    /// This function must only be called from the main GTK thread.
    /// Panics in debug mode if called from another thread.
    pub fn clear_visual_warnings(&self) {
        debug_assert!(glib::MainContext::default().is_owner(), "clear_visual_warnings must be called from the main thread!");
        if let Some(buffer) = &self.buffer {
            let start_iter = buffer.start_iter();
            let end_iter = buffer.end_iter();
            let tag_table = buffer.tag_table();
            let mut removed_any = false;
            if let Some(tag) = tag_table.lookup("markdown-warning") {
                buffer.remove_tag(&tag, &start_iter, &end_iter);
                removed_any = true;
            }
            if let Some(tag) = tag_table.lookup("broken-link") {
                buffer.remove_tag(&tag, &start_iter, &end_iter);
                removed_any = true;
            }
            if let Some(tag) = tag_table.lookup("html-warning") {
                buffer.remove_tag(&tag, &start_iter, &end_iter);
                removed_any = true;
            }
            if let Some(tag) = tag_table.lookup("structure-warning") {
                buffer.remove_tag(&tag, &start_iter, &end_iter);
                removed_any = true;
            }
            if removed_any {
                eprintln!("[DEBUG] clear_visual_warnings: tags removed (no changed signal emitted)");
                // DO NOT emit the "changed" signal here! Doing so causes infinite recursion if called from a buffer change handler.
                // If a redraw is needed, the view widget should be queued for redraw externally.
            } else {
                eprintln!("[DEBUG] clear_visual_warnings: no tags found to remove");
            }
        }
    }
    pub fn new(config: SpellLintConfig) -> Self {
        let warning_tags = Self::create_error_tags();
        // Load settings
        let settings = Settings::new("org.marco.editor");
        let timeout_ms = settings.int("debounce-timeout-ms").max(50); // Clamp to minimum 50ms
        let debouncer = Debouncer::new(timeout_ms as u32);
        let checker = Self {
            config,
            warnings: Vec::new(),
            warning_tags,
            buffer: None,
            debouncer: Some(debouncer.clone()),
            settings: Some(settings.clone()),
            heading_regex: get_regex(r"^(#{1,6})([^ #]|$)"),
            link_regex: get_regex(r"\[([^\]]*)\]\(([^)]*)\)"),
            image_regex: get_regex(r"!\[([^\]]*)\]\(([^)]*)\)"),
            html_tag_regex: get_regex(r"<[^>]+>"),
            reference_regex: get_regex(r"\[([^\]]+)\]:\s*(.+)"),
        };
        // Listen for changes to debounce-timeout-ms and update debouncer
        if let Some(settings) = &checker.settings {
            let settings = settings.clone();
            let debouncer = checker.debouncer.as_ref().unwrap().clone();
            settings.connect_changed(Some("debounce-timeout-ms"), move |s, _| {
                let ms = s.int("debounce-timeout-ms").max(50) as u32;
                debouncer.set_timeout_ms(ms);
            });
        }
        checker
    }

    /// Debounced spellcheck trigger. Call this on user input events.
    pub fn trigger_spellcheck_debounced(weak_self: std::rc::Weak<std::cell::RefCell<Self>>, content: String) {
        if let Some(strong_self) = weak_self.upgrade() {
            let checker = strong_self.borrow_mut();
            if let Some(debouncer) = &checker.debouncer {
                let weak_self2 = weak_self.clone();
                let content_clone = content.clone();
                debouncer.debounce(move || {
                    if let Some(strong_self_inner) = weak_self2.upgrade() {
                        let mut checker_inner = strong_self_inner.borrow_mut();
                        checker_inner.lint(&content_clone);
                    }
                });
            }
        }
    }
    pub fn new_with_defaults() -> Self {
        Self::new(SpellLintConfig::default())
    }

    /// Create GTK TextTags for different warning types
    fn create_error_tags() -> HashMap<String, TextTag> {
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
    fn create_error_tags_for_buffer(buffer: &gtk4::TextBuffer) -> HashMap<String, TextTag> {
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

    /// Set the text buffer for applying visual warnings.
    ///
    /// # Safety
    /// This function must only be called from the main GTK thread.
    /// Panics in debug mode if called from another thread.
    pub fn set_buffer(&mut self, buffer: &gtk4::TextBuffer) {
        debug_assert!(glib::MainContext::default().is_owner(), "set_buffer must be called from the main thread!");
        self.buffer = Some(buffer.clone());

        // Create fresh tags for this buffer to avoid conflicts
        self.warning_tags = Self::create_error_tags_for_buffer(buffer);
    }
    // ...existing code...

    /// Lint markdown content and return warnings
    ///
    /// # Panics
    /// Panics in debug mode if `set_buffer` has not been called before linting.
    /// Logs a warning in release mode if `set_buffer` has not been called.
    pub fn lint(&mut self, content: &str) -> Vec<SpellError> {
        // Ensure set_buffer has been called before linting
        if self.buffer.is_none() {
            #[cfg(debug_assertions)]
            panic!("SpellSyntaxChecker: set_buffer() must be called before linting to ensure tags are in the buffer's tag table. Underline graphics will not appear otherwise.");
            #[cfg(not(debug_assertions))]
            eprintln!("[WARN] SpellSyntaxChecker: set_buffer() has not been called before linting. Underline graphics will not appear.");
        }

        // Refactored: collect warnings from each pass, then merge
        let mut all_warnings = Vec::new();

        // Markdown-aware spellcheck (parallel, precise offsets)
        if self.config.enable_spellcheck {
            all_warnings.extend(spellcheck_text_spans(content));
        }

        // First pass: AST-based
        all_warnings.extend(self.ast_based_checks_pure(content));

        // Second pass: regex-based (per-line)
        all_warnings.extend(self.regex_based_checks_pure(content));

        // Third pass: structural (multi-line)
        all_warnings.extend(self.structural_checks_pure(content));

        // Update self.warnings for compatibility
        self.warnings = all_warnings.clone();

        // Apply visual warnings to the buffer if available
        if let Some(buffer) = &self.buffer {
            self.apply_visual_errors(buffer, content);
        }

        all_warnings
    }

    /// AST-based checks using pulldown-cmark (pure, returns Vec)
    fn ast_based_checks_pure(&self, content: &str) -> Vec<SpellError> {
        let parser = Parser::new(content);
        let mut event_stack = Vec::new();
        let mut current_offset = 0;
        let line_offsets = self.build_line_offsets(content);
        let mut warnings = Vec::new();

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
                            warnings.push(SpellError {
                                line,
                                column: col,
                                start_offset,
                                end_offset: current_offset,
                                warning_type: SpellType::ImproperNesting,
                                message: format!(
                                    "Mismatched tag: expected {:?}, found {:?}",
                                    start_tag, tag
                                ),
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
                        warnings.push(SpellError {
                            line,
                            column: col,
                            start_offset: current_offset,
                            end_offset: current_offset + html.len(),
                            warning_type: SpellType::RawHtml,
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
                warnings.push(SpellError {
                    line,
                    column: col,
                    start_offset,
                    end_offset: start_offset,
                    warning_type: SpellType::UncloseTag,
                    message: format!("Unclosed tag: {:?}", tag),
                    suggestion: Some("Close the tag properly".to_string()),
                });
            }
        }
        warnings
    }

    /// Regex-based checks for syntax patterns (pure, returns Vec)
    fn regex_based_checks_pure(&self, content: &str) -> Vec<SpellError> {
        use rayon::prelude::*;
        let lines: Vec<&str> = content.lines().collect();
        let line_offsets = self.build_line_offsets(content);
        // Clone only thread-safe config and regexes
        let config = self.config.clone();
        let heading_regex = self.heading_regex.clone();
        let link_regex = self.link_regex.clone();
        let image_regex = self.image_regex.clone();
        let html_tag_regex = self.html_tag_regex.clone();
        // Parallelize per-line checks without capturing &self
        lines.par_iter().enumerate().flat_map(|(idx, line)| {
            let line_num = idx + 1;
            let line_start_offset = if line_num == 1 {
                0
            } else {
                line_offsets[line_num - 2]
            };
            let mut warnings = Vec::new();
            if config.check_improper_headings {
                if let Some(caps) = heading_regex.captures(line) {
                    let hashes = caps.get(1).unwrap().as_str();
                    let column = caps.get(1).unwrap().end();
                    let start_offset = line_start_offset + caps.get(1).unwrap().start();
                    let end_offset = line_start_offset + caps.get(1).unwrap().end();
                    warnings.push(SpellError {
                        line: line_num,
                        column,
                        start_offset,
                        end_offset,
                        warning_type: SpellType::ImproperHeading,
                        message: format!("Heading should have a space after `{}`", hashes),
                        suggestion: Some(format!("Change to `{} `", hashes)),
                    });
                }
            }
            if config.check_unclosed_emphasis {
                // Inline pure logic for emphasis check
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
                    warnings.push(SpellError {
                        line: line_num,
                        column: 1,
                        start_offset: line_start_offset,
                        end_offset: line_start_offset + line.len(),
                        warning_type: SpellType::UnclosedEmphasis,
                        message: "Unclosed bold marker (**)".to_string(),
                        suggestion: Some("Ensure bold markers are properly paired".to_string()),
                    });
                }
                if italic_count % 2 != 0 {
                    warnings.push(SpellError {
                        line: line_num,
                        column: 1,
                        start_offset: line_start_offset,
                        end_offset: line_start_offset + line.len(),
                        warning_type: SpellType::UnclosedEmphasis,
                        message: "Unclosed italic marker (* or _)".to_string(),
                        suggestion: Some("Ensure italic markers are properly paired".to_string()),
                    });
                }
            }
            if config.check_broken_links || config.check_empty_links {
                for caps in link_regex.captures_iter(line) {
                    let full_match = caps.get(0).unwrap();
                    let url = caps.get(2).unwrap().as_str();
                    let start_offset = line_start_offset + full_match.start();
                    let end_offset = line_start_offset + full_match.end();
                    if config.check_empty_links && url.trim().is_empty() {
                        warnings.push(SpellError {
                            line: line_num,
                            column: full_match.start() + 1,
                            start_offset,
                            end_offset,
                            warning_type: SpellType::EmptyLink,
                            message: "Link with empty URL".to_string(),
                            suggestion: Some("Provide a valid URL or remove the link".to_string()),
                        });
                    }
                }
                let broken_link_regex = Regex::new(r"\[([^\]]*)\]\([^)]*$|\[([^\]]*)\]$|\[([^\]]*)$").unwrap();
                for mat in broken_link_regex.find_iter(line) {
                    let start_offset = line_start_offset + mat.start();
                    let end_offset = line_start_offset + mat.end();
                    warnings.push(SpellError {
                        line: line_num,
                        column: mat.start() + 1,
                        start_offset,
                        end_offset,
                        warning_type: SpellType::BrokenLink,
                        message: "Incomplete link syntax".to_string(),
                        suggestion: Some("Ensure link has format [text](url)".to_string()),
                    });
                }
            }
            if config.check_broken_images || config.check_missing_alt_text {
                for caps in image_regex.captures_iter(line) {
                    let full_match = caps.get(0).unwrap();
                    let alt_text = caps.get(1).unwrap().as_str();
                    let start_offset = line_start_offset + full_match.start();
                    let end_offset = line_start_offset + full_match.end();
                    if config.check_missing_alt_text && alt_text.trim().is_empty() {
                        warnings.push(SpellError {
                            line: line_num,
                            column: full_match.start() + 1,
                            start_offset,
                            end_offset,
                            warning_type: SpellType::MissingAltText,
                            message: "Image missing alt text".to_string(),
                            suggestion: Some("Add descriptive alt text for accessibility".to_string()),
                        });
                    }
                }
            }
            if config.check_invalid_task_lists {
                let malformed_task_regex = Regex::new(r"^(\s*)-\s+\[([^\]xX ])\]").unwrap();
                if let Some(caps) = malformed_task_regex.captures(line) {
                    let checkbox = caps.get(2).unwrap().as_str();
                    let start_offset = line_start_offset + caps.get(0).unwrap().start();
                    let end_offset = line_start_offset + caps.get(0).unwrap().end();
                    warnings.push(SpellError {
                        line: line_num,
                        column: caps.get(0).unwrap().start() + 1,
                        start_offset,
                        end_offset,
                        warning_type: SpellType::InvalidTaskList,
                        message: format!("Invalid task list checkbox: [{}]", checkbox),
                        suggestion: Some("Use [ ] for unchecked or [x] for checked".to_string()),
                    });
                }
            }
            if config.check_malformed_footnotes {
                let footnote_regex = Regex::new(r"\[\^([^\]]*)\]").unwrap();
                for caps in footnote_regex.captures_iter(line) {
                    let full_match = caps.get(0).unwrap();
                    let footnote_id = caps.get(1).unwrap().as_str();
                    let start_offset = line_start_offset + full_match.start();
                    let end_offset = line_start_offset + full_match.end();
                    let remainder = &line[full_match.end()..];
                    let is_definition = remainder.trim_start().starts_with(':');
                    if !is_definition && footnote_id.trim().is_empty() {
                        warnings.push(SpellError {
                            line: line_num,
                            column: caps.get(0).unwrap().start() + 1,
                            start_offset,
                            end_offset,
                            warning_type: SpellType::MalformedFootnote,
                            message: "Footnote reference with empty ID".to_string(),
                            suggestion: Some("Provide a valid footnote ID".to_string()),
                        });
                    }
                }
            }
            if config.check_raw_html {
                for mat in html_tag_regex.find_iter(line) {
                    let start_offset = line_start_offset + mat.start();
                    let end_offset = line_start_offset + mat.end();
                    warnings.push(SpellError {
                        line: line_num,
                        column: mat.start() + 1,
                        start_offset,
                        end_offset,
                        warning_type: SpellType::RawHtml,
                        message: format!("Raw HTML tag found: {}", mat.as_str()),
                        suggestion: Some("Consider using Markdown syntax instead".to_string()),
                    });
                }
            }
            warnings
        }).collect()
    }
    /// Structural checks for multi-line patterns (pure, returns Vec)
    fn structural_checks_pure(&self, content: &str) -> Vec<SpellError> {
        let lines: Vec<&str> = content.lines().collect();
        // For multi-line checks, parallelize where possible (e.g., table rows, list blocks)
        let mut warnings = Vec::new();
        // These checks are not always independent, but we can parallelize the ones that are
        if self.config.check_unclosed_code_blocks {
            warnings.extend(self.check_code_blocks_pure(&lines)); // Not parallelizable (needs state)
        }
        if self.config.check_malformed_tables {
            // Table structure can be checked per row, but needs expected_columns state
            warnings.extend(self.check_table_structure_pure(&lines));
        }
        if self.config.check_invalid_references {
            warnings.extend(self.check_reference_links_pure(&lines));
        }
        if self.config.check_inconsistent_list_markers {
            warnings.extend(self.check_list_consistency_pure(&lines));
        }
        warnings
    }
    // ---
    // The following are pure versions of the per-line and per-block check functions.
    // They return Vec<SpellError> instead of mutating self.warnings.
    // ---
    // ---
    // Pure versions of structural/multi-line checks
    fn check_code_blocks_pure(&self, lines: &[&str]) -> Vec<SpellError> {
        let mut warnings = Vec::new();
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
            current_offset += line.len() + 1;
        }
        if in_code_block {
            warnings.push(SpellError {
                line: code_block_start,
                column: 1,
                start_offset: code_block_start_offset,
                end_offset: code_block_start_offset + 3,
                warning_type: SpellType::UnclosedCodeBlock,
                message: "Unclosed code block".to_string(),
                suggestion: Some("Add closing ``` to end the code block".to_string()),
            });
        }
        warnings
    }
    fn check_table_structure_pure(&self, lines: &[&str]) -> Vec<SpellError> {
        use rayon::prelude::*;
        // Find table blocks (consecutive lines with '|'), then check each block in parallel
        let mut table_blocks = Vec::new();
        let mut current_block = Vec::new();
        let mut block_start = 0;
        for (i, line) in lines.iter().enumerate() {
            if line.contains('|') {
                if current_block.is_empty() {
                    block_start = i;
                }
                current_block.push((i, *line));
            } else if !current_block.is_empty() && line.trim().is_empty() {
                table_blocks.push((block_start, current_block.clone()));
                current_block.clear();
            } else if !current_block.is_empty() {
                // End of table block
                table_blocks.push((block_start, current_block.clone()));
                current_block.clear();
            }
        }
        if !current_block.is_empty() {
            table_blocks.push((block_start, current_block));
        }
        // For each block, check column consistency in parallel
        table_blocks.par_iter()
            .map(|(_start, block)| {
                if block.is_empty() { return Vec::new(); }
                let expected_columns = block[0].1.matches('|').count();
                block.iter().filter_map(move |(line_idx, line)| {
                    let current_columns = line.matches('|').count();
                    if current_columns != expected_columns {
                        Some(SpellError {
                            line: line_idx + 1,
                            column: 1,
                            start_offset: 0, // Offset not tracked for now
                            end_offset: 0,
                            warning_type: SpellType::MalformedTable,
                            message: format!(
                                "Table row has {} columns, expected {}",
                                current_columns, expected_columns
                            ),
                            suggestion: Some(
                                "Ensure all table rows have the same number of columns".to_string(),
                            ),
                        })
                    } else {
                        None
                    }
                }).collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }
    fn check_reference_links_pure(&self, lines: &[&str]) -> Vec<SpellError> {
        use rayon::prelude::*;
        use std::collections::HashMap;
        let mut references = HashMap::new();
        for line in lines.iter() {
            if let Some(caps) = self.reference_regex.captures(line) {
                let ref_name = caps.get(1).unwrap().as_str().to_lowercase();
                references.insert(ref_name, true);
            }
        }
        let reference_use_regex = Regex::new(r"\[([^\]]+)\]\[([^\]]*)\]").unwrap();
        lines.par_iter().enumerate().map(|(line_num, line)| {
            let mut warnings = Vec::new();
            for caps in reference_use_regex.captures_iter(line) {
                let ref_name = if caps.get(2).unwrap().as_str().is_empty() {
                    caps.get(1).unwrap().as_str().to_lowercase()
                } else {
                    caps.get(2).unwrap().as_str().to_lowercase()
                };
                if !references.contains_key(&ref_name) {
                    let match_start = caps.get(0).unwrap().start();
                    warnings.push(SpellError {
                        line: line_num + 1,
                        column: match_start + 1,
                        start_offset: 0, // Offset not tracked for now
                        end_offset: 0,
                        warning_type: SpellType::InvalidReference,
                        message: format!("Reference '{}' not found", ref_name),
                        suggestion: Some("Define the reference or use a direct link".to_string()),
                    });
                }
            }
            warnings
        }).flatten().collect()
    }
    fn check_list_consistency_pure(&self, lines: &[&str]) -> Vec<SpellError> {
        use rayon::prelude::*;
        use regex::Regex;
        use std::sync::Arc;
        // Find list blocks (consecutive lines starting with list marker)
        let list_marker_regex = Arc::new(Regex::new(r"^(\s*)([-*+])\s").unwrap());
        let mut blocks = Vec::new();
        let mut current_block = Vec::new();
        let mut block_start = 0;
        for (i, line) in lines.iter().enumerate() {
            if list_marker_regex.is_match(line) {
                if current_block.is_empty() {
                    block_start = i;
                }
                current_block.push((i, *line));
            } else if !current_block.is_empty() && line.trim().is_empty() {
                blocks.push((block_start, current_block.clone()));
                current_block.clear();
            } else if !current_block.is_empty() {
                blocks.push((block_start, current_block.clone()));
                current_block.clear();
            }
        }
        if !current_block.is_empty() {
            blocks.push((block_start, current_block));
        }
        blocks.par_iter()
            .map(|(_start, block)| {
                if block.is_empty() { return Vec::new(); }
                let list_marker_regex = Arc::clone(&list_marker_regex);
                let (first_indent, first_marker) = {
                    let caps = list_marker_regex.captures(block[0].1).unwrap();
                    (caps.get(1).unwrap().as_str().len(), caps.get(2).unwrap().as_str().chars().next().unwrap())
                };
                block.iter().filter_map(move |(line_idx, line)| {
                    let caps = list_marker_regex.captures(line).unwrap();
                    let indent = caps.get(1).unwrap().as_str().len();
                    let marker = caps.get(2).unwrap().as_str().chars().next().unwrap();
                    if indent != first_indent || marker != first_marker {
                        Some(SpellError {
                            line: line_idx + 1,
                            column: 1,
                            start_offset: 0, // Offset not tracked for now
                            end_offset: 0,
                            warning_type: SpellType::InconsistentListMarkers,
                            message: format!("List marker or indentation inconsistent: expected '{}', found '{}'", first_marker, marker),
                            suggestion: Some("Use consistent list markers and indentation".to_string()),
                        })
                    } else {
                        None
                    }
                }).collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }

    /// Apply visual warnings to the text buffer
    ///
    /// # Panics
    /// Panics in debug mode if `set_buffer` has not been called before tag application.
    fn apply_visual_errors(&self, buffer: &gtk4::TextBuffer, _content: &str) {
        debug_assert!(glib::MainContext::default().is_owner(), "apply_visual_errors must be called from the main thread!");
        // Ensure tags are in the buffer's tag table
        #[cfg(debug_assertions)]
        if self.warning_tags.is_empty() {
            panic!("SpellSyntaxChecker: set_buffer() must be called before applying tags. No tags present.");
        }

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
                SpellType::BrokenLink | SpellType::BrokenImage | SpellType::EmptyLink => {
                    "broken-link"
                }
                SpellType::RawHtml => "html-warning",
                // Structure issues get blue underline
                SpellType::UncloseTag
                | SpellType::ImproperNesting
                | SpellType::UnclosedCodeBlock
                | SpellType::MalformedTable
                | SpellType::InvalidReference
                | SpellType::InconsistentListMarkers
                | SpellType::MalformedFootnote
                | SpellType::UnclosedBlockquote => "structure-warning",
                _ => "warning",
            };

            if let Some(tag) = self.warning_tags.get(tag_name) {
                buffer.apply_tag(tag, &start_iter, &end_iter);
            } else {
                #[cfg(debug_assertions)]
                panic!("SpellSyntaxChecker: Tag '{}' not found in warning_tags. set_buffer() may not have been called.", tag_name);
                #[cfg(not(debug_assertions))]
                eprintln!("[WARN] SpellSyntaxChecker: Tag '{}' not found in warning_tags. set_buffer() may not have been called.", tag_name);
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
                let line_start = if line_num == 0 {
                    0
                } else {
                    line_offsets[line_num - 1]
                };
                let column = offset - line_start + 1;
                return (line_num + 1, column);
            }
        }

        (line_offsets.len() + 1, 1)
    }

    /// Clear all warnings and visual indicators
    pub fn clear_warnings(&mut self) {
        self.warnings.clear();

        if let Some(buffer) = &self.buffer {
            let start_iter = buffer.start_iter();
            let end_iter = buffer.end_iter();
            let tag_table = buffer.tag_table();

            // Only remove tags that actually exist in this buffer's tag table
            if let Some(tag) = tag_table.lookup("markdown-error") {
                buffer.remove_tag(&tag, &start_iter, &end_iter);
            }
            if let Some(tag) = tag_table.lookup("broken-link") {
                buffer.remove_tag(&tag, &start_iter, &end_iter);
            }
            if let Some(tag) = tag_table.lookup("html-error") {
                buffer.remove_tag(&tag, &start_iter, &end_iter);
            }
            if let Some(tag) = tag_table.lookup("structure-error") {
                buffer.remove_tag(&tag, &start_iter, &end_iter);
            }
        }
    }

}

/// Convenience function to create a syntax checker with default config
pub fn create_syntax_checker() -> SpellSyntaxChecker {
    SpellSyntaxChecker::new_with_defaults()
}

/// Convenience function to create a syntax checker with custom config
pub fn create_syntax_checker_with_config(config: SpellLintConfig) -> SpellSyntaxChecker {
    SpellSyntaxChecker::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;
    static GTK_INIT: Once = Once::new();
    fn ensure_gtk_init() {
        GTK_INIT.call_once(|| {
            let _ = gtk4::init();
        });
    }
    #[test]
    fn gtk_spell_check_suite() {
        ensure_gtk_init();
        use gtk4::TextBuffer;
        // test_improper_heading
        {
            let mut config = SpellLintConfig::default();
            config.enable_spellcheck = false;
            let mut checker = SpellSyntaxChecker::new(config);
            let buffer = TextBuffer::new(None);
            checker.set_buffer(&buffer);
            let warnings = checker.lint("##No space after hash");
            assert_eq!(warnings.len(), 1);
            assert_eq!(warnings[0].warning_type, SpellType::ImproperHeading);
        }
        // test_unclosed_emphasis
        {
            let mut config = SpellLintConfig::default();
            config.enable_spellcheck = false;
            let mut checker = SpellSyntaxChecker::new(config);
            let buffer = TextBuffer::new(None);
            checker.set_buffer(&buffer);
            let warnings = checker.lint("This is **bold without closing");
            assert_eq!(warnings.len(), 1);
            assert_eq!(warnings[0].warning_type, SpellType::UnclosedEmphasis);
        }
        // test_empty_link
        {
            let mut config = SpellLintConfig::default();
            config.enable_spellcheck = false;
            let mut checker = SpellSyntaxChecker::new(config);
            let buffer = TextBuffer::new(None);
            checker.set_buffer(&buffer);
            let warnings = checker.lint("This is an [empty link]()");
            assert_eq!(warnings.len(), 1);
            assert_eq!(warnings[0].warning_type, SpellType::EmptyLink);
        }
        // test_missing_alt_text
        {
            let mut config = SpellLintConfig::default();
            config.enable_spellcheck = false;
            let mut checker = SpellSyntaxChecker::new(config);
            let buffer = TextBuffer::new(None);
            checker.set_buffer(&buffer);
            let warnings = checker.lint("This is an image ![](image.png)");
            assert_eq!(warnings.len(), 1);
            assert_eq!(warnings[0].warning_type, SpellType::MissingAltText);
        }
        // test_unclosed_code_block
        {
            let mut config = SpellLintConfig::default();
            config.enable_spellcheck = false;
            let mut checker = SpellSyntaxChecker::new(config);
            let buffer = TextBuffer::new(None);
            checker.set_buffer(&buffer);
            let warnings = checker.lint("```rust\nfn main() {}\n// Missing closing ```");
            assert_eq!(warnings.len(), 1);
            assert_eq!(warnings[0].warning_type, SpellType::UnclosedCodeBlock);
        }
        // test_valid_markdown
        {
            let mut config = SpellLintConfig::default();
            config.enable_spellcheck = false;
            // No warnings expected for valid markdown, but still set buffer
            let mut checker = SpellSyntaxChecker::new(config);
            let buffer = TextBuffer::new(None);
            checker.set_buffer(&buffer);
            let warnings = checker.lint("# Valid Heading\n\nSome text here.");
            assert_eq!(warnings.len(), 0);
        }
    }
}