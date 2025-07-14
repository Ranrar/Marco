/// Analyze the text at the cursor and return an HTML-like formatting string with indentation
pub fn get_formatting_at_cursor(text: &str, line: usize) -> String {
    use crate::markdown::basic::MarkdownParser;
    let parser = MarkdownParser::new();
    let lines: Vec<&str> = text.lines().collect();
    if line == 0 || line > lines.len() {
        return "Format:".to_string();
    }
    let line_text = lines[line - 1];
    let mut parts = Vec::new();

    // Horizontal rule detection (---, ***, ___, etc.)
    let is_hr = {
        let t = line_text.trim();
        t == "---" || t == "***" || t == "___" ||
        (t.chars().all(|c| c == '-') && t.len() >= 3) ||
        (t.chars().all(|c| c == '*') && t.len() >= 3) ||
        (t.chars().all(|c| c == '_') && t.len() >= 3)
    };
    if is_hr {
        return "Format: Horizontal Rule".to_string();
    }

    // HTML tag detection (e.g., <code>, <b>, etc.)
    let html_tag_re = regex::Regex::new(r"<[^>]+>").unwrap();
    if html_tag_re.is_match(line_text) {
        return "Format: HTML".to_string();
    }

    // Horizontal rule detection (---, ***, ___, etc.)
    let is_hr = {
        let t = line_text.trim();
        t == "---" || t == "***" || t == "___" ||
        (t.chars().all(|c| c == '-') && t.len() >= 3) ||
        (t.chars().all(|c| c == '*') && t.len() >= 3) ||
        (t.chars().all(|c| c == '_') && t.len() >= 3)
    };
    if is_hr {
        return "Format: Horizontal Rule".to_string();
    }

    // HTML tag detection (e.g., <code>, <b>, etc.)
    let html_tag_re = regex::Regex::new(r"<[^>]+>").unwrap();
    if html_tag_re.is_match(line_text) {
        return "Format: HTML".to_string();
    }

    // Fenced code block detection
    // We'll scan from the start of the document to the current line to determine if we're inside a fenced code block
    let mut in_fence = false;
    let mut fence_lang = String::new();
    for (_, l) in lines.iter().enumerate().take(line) {
        let trimmed = l.trim_start();
        if trimmed.starts_with("```") {
            if !in_fence {
                // Opening fence
                in_fence = true;
                // fence_start removed (unused)
                // Try to extract language name
                let after = trimmed.trim_start_matches("```").trim();
                if !after.is_empty() {
                    fence_lang = after.to_string();
                } else {
                    fence_lang.clear();
                }
            } else {
                // Closing fence
                in_fence = false;
                fence_lang.clear();
            }
        }
    }
    if in_fence {
        let lang = if !fence_lang.is_empty() {
            format!("{} ", fence_lang[..1].to_uppercase() + &fence_lang[1..])
        } else {
            String::new()
        };
        return format!("Format: Fencing {}code", lang);
    }
    // Indentation detection
    let indent_len = line_text.chars().take_while(|c| c.is_whitespace()).count();
    if indent_len > 0 {
        parts.push((0, "Indent".to_string()));
    }

    // Header detection
    if let Some(hashes) = parser.detect_heading(line_text) {
        // Heading hashes are always at the start
        parts.push((0, format!("Header {}", hashes)));
    }

    // List detection (unordered and ordered)
    let trimmed = line_text.trim_start();
    let indent_offset = line_text.len() - trimmed.len();
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        parts.push((indent_offset, "List".to_string()));
    } else if parser.is_ordered_list(trimmed) {
        parts.push((indent_offset, "Ordered List".to_string()));
    }

    // Blockquote detection
    if trimmed.starts_with("> ") {
        parts.push((indent_offset, "Blockquote".to_string()));
    }

    // Inline Markdown detection (all regions, not just at cursor)
    let mut inline_regions = Vec::new();
    // text_bytes removed (unused)

    // Detect Bold and Italic (***x***)
    let mut idx = 0;
    while let Some(start) = line_text[idx..].find("***") {
        let abs_start = idx + start;
        if let Some(end) = line_text[abs_start + 3..].find("***") {
            let abs_end = abs_start + 3 + end + 3 - 1;
            // Check that the region is exactly ***x***
            let region = &line_text[abs_start..=abs_end];
            if region.starts_with("***") && region.ends_with("***") && region.len() > 6 {
                inline_regions.push((abs_start, "Bold and Italic".to_string()));
            }
            idx = abs_end + 1;
        } else {
            break;
        }
    }

    // Detect Bold (**x**), but not ***x***
    let mut idx = 0;
    while let Some(start) = line_text[idx..].find("**") {
        let abs_start = idx + start;
        // Skip if this is part of a ***x*** region
        if line_text[abs_start..].starts_with("***") {
            idx = abs_start + 3;
            continue;
        }
        if let Some(end) = line_text[abs_start + 2..].find("**") {
            let abs_end = abs_start + 2 + end + 2 - 1;
            let region = &line_text[abs_start..=abs_end];
            if region.starts_with("**") && region.ends_with("**") && region.len() > 4 {
                inline_regions.push((abs_start, "Bold".to_string()));
            }
            idx = abs_end + 1;
        } else {
            break;
        }
    }

    // Detect Italic (*x*), but not **x** or ***x***
    let mut idx = 0;
    while let Some(start) = line_text[idx..].find('*') {
        let abs_start = idx + start;
        // Skip if this is part of a ** or *** region
        if line_text[abs_start..].starts_with("***") {
            idx = abs_start + 3;
            continue;
        } else if line_text[abs_start..].starts_with("**") {
            idx = abs_start + 2;
            continue;
        }
        if let Some(end) = line_text[abs_start + 1..].find('*') {
            let abs_end = abs_start + 1 + end + 1 - 1;
            let region = &line_text[abs_start..=abs_end];
            if region.starts_with('*') && region.ends_with('*') && region.len() > 2 {
                inline_regions.push((abs_start, "Italic".to_string()));
            }
            idx = abs_end + 1;
        } else {
            break;
        }
    }

    // Other inline regions (strikethrough, code, link, image)
    let region_checks = [
        ("Strikethrough", parser.find_strikethrough(line_text)),
        ("Inline Code", parser.find_inline_code(line_text)),
        ("Link", parser.find_links(line_text)),
        ("Image", parser.find_images(line_text)),
    ];
    for (name, regions) in region_checks.iter() {
        for (start, _end) in regions {
            inline_regions.push((*start, name.to_string()));
        }
    }

    // Sort all parts (block and inline) by their position in the line
    parts.extend(inline_regions);
    parts.sort_by_key(|(pos, _)| *pos);

    // Remove duplicates but keep order
    let mut seen = std::collections::HashSet::new();
    let mut ordered = Vec::new();
    for (_pos, name) in parts {
        if seen.insert(name.clone()) {
            ordered.push(name);
        }
    }

    if ordered.is_empty() {
        if line_text.trim().is_empty() {
            "Format:".to_string()
        } else {
            // If not markdown, not html, not hr, treat as plain text
            "Format: Text".to_string()
        }
    } else {
        format!("Format: {}", ordered.join(" > "))
    }
}
/// Update the formatting label in the footer with the current formatting info (Markdown-style, indented)
pub fn update_formatting_label(footer_labels: &FooterLabels, formatting_md: &str) {
    footer_labels.formatting.set_text(formatting_md);
}

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation};

#[derive(Clone)]
pub struct FooterLabels {
    pub word_count: Label,
    pub char_count: Label,
    pub cursor_pos: Label,
    pub formatting: Label,
}

pub fn create_footer() -> (Box, FooterLabels) {
    let footer_box = Box::new(Orientation::Horizontal, 10);
    footer_box.set_margin_top(5);
    footer_box.set_margin_bottom(5);
    footer_box.set_margin_start(10);
    footer_box.set_margin_end(10);

    // Formatting label (left side)
    let formatting_label = Label::new(Some("Format:"));
    formatting_label.set_halign(gtk4::Align::Start);
    formatting_label.set_xalign(0.0);
    footer_box.append(&formatting_label);

    // Spacer to push items to the sides
    let spacer = Label::new(None);
    spacer.set_hexpand(true);
    footer_box.append(&spacer);

    // Info labels (right side)
    let word_count_label = Label::new(Some("Words: 0"));
    footer_box.append(&word_count_label);

    let char_count_label = Label::new(Some("Characters: 0"));
    footer_box.append(&char_count_label);

    let cursor_pos_label = Label::new(Some("Line: 1, Col: 1"));
    footer_box.append(&cursor_pos_label);


    let labels = FooterLabels {
        word_count: word_count_label,
        char_count: char_count_label,
        cursor_pos: cursor_pos_label,
        formatting: formatting_label,
    };

    (footer_box, labels)
}
