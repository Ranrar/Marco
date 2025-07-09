use crate::editor::core::MarkdownEditor;
use std::collections::HashMap;

// Public standalone color functions
pub mod color {
    use gtk4::prelude::*;
    use sourceview5;
    use std::collections::HashMap;
    use syntect::easy::HighlightLines;
    use syntect::highlighting::ThemeSet;
    use syntect::parsing::SyntaxSet;

    /// Parse color names and hex codes to RGBA
    pub fn parse_color(color: &str) -> Option<gdk4::RGBA> {
        match color.to_lowercase().as_str() {
            "red" => Some(gdk4::RGBA::new(1.0, 0.0, 0.0, 1.0)),
            "green" => Some(gdk4::RGBA::new(0.0, 1.0, 0.0, 1.0)),
            "blue" => Some(gdk4::RGBA::new(0.0, 0.0, 1.0, 1.0)),
            "yellow" => Some(gdk4::RGBA::new(1.0, 1.0, 0.0, 1.0)),
            "orange" => Some(gdk4::RGBA::new(1.0, 0.5, 0.0, 1.0)),
            "purple" => Some(gdk4::RGBA::new(0.5, 0.0, 0.5, 1.0)),
            "pink" => Some(gdk4::RGBA::new(1.0, 0.0, 0.5, 1.0)),
            "cyan" => Some(gdk4::RGBA::new(0.0, 1.0, 1.0, 1.0)),
            "black" => Some(gdk4::RGBA::new(0.0, 0.0, 0.0, 1.0)),
            "white" => Some(gdk4::RGBA::new(1.0, 1.0, 1.0, 1.0)),
            "gray" | "grey" => Some(gdk4::RGBA::new(0.5, 0.5, 0.5, 1.0)),
            hex if hex.starts_with('#') => {
                // Parse hex color
                if hex.len() == 7 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        u8::from_str_radix(&hex[1..3], 16),
                        u8::from_str_radix(&hex[3..5], 16),
                        u8::from_str_radix(&hex[5..7], 16),
                    ) {
                        return Some(gdk4::RGBA::new(
                            r as f32 / 255.0,
                            g as f32 / 255.0,
                            b as f32 / 255.0,
                            1.0,
                        ));
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Convert syntect Color to gdk4::RGBA
    pub fn syntect_color_to_rgba(color: syntect::highlighting::Color) -> gdk4::RGBA {
        gdk4::RGBA::new(
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        )
    }

    /// Ensure a tag exists in the tag table
    pub fn ensure_tag_exists<F>(
        buffer: &sourceview5::Buffer,
        tag_table: &mut HashMap<String, gtk4::TextTag>,
        name: &str,
        configure: F,
    ) where
        F: FnOnce(&gtk4::TextTag),
    {
        if !tag_table.contains_key(name) {
            let tag = gtk4::TextTag::new(Some(name));
            configure(&tag);

            // Add the tag to the buffer's tag table
            let gtk_tag_table = buffer.tag_table();
            gtk_tag_table.add(&tag);

            tag_table.insert(name.to_string(), tag);
        }
    }

    /// Highlight colored text using style attributes
    pub fn highlight_colored_text(
        buffer: &sourceview5::Buffer,
        text: &str,
        tag_table: &mut HashMap<String, gtk4::TextTag>,
        color_regex: &regex::Regex,
        font_color_regex: &regex::Regex,
    ) {
        // Handle <p style="color:..."> tags
        for captures in color_regex.captures_iter(text) {
            if let Some(mat) = captures.get(0) {
                let color_name = &captures[1];
                let tag_name = format!("color_{}", color_name);

                ensure_tag_exists(buffer, tag_table, &tag_name, |tag| {
                    if let Some(rgba) = parse_color(color_name) {
                        tag.set_foreground_rgba(Some(&rgba));
                    }
                });

                let start_iter = buffer.iter_at_offset(mat.start() as i32);
                let end_iter = buffer.iter_at_offset(mat.end() as i32);
                buffer.apply_tag(&tag_table[&tag_name], &start_iter, &end_iter);
            }
        }

        // Handle <font color="..."> tags (deprecated but still used)
        for captures in font_color_regex.captures_iter(text) {
            if let Some(mat) = captures.get(0) {
                let color_name = &captures[1];
                let tag_name = format!("font_color_{}", color_name);

                ensure_tag_exists(buffer, tag_table, &tag_name, |tag| {
                    if let Some(rgba) = parse_color(color_name) {
                        tag.set_foreground_rgba(Some(&rgba));
                    }
                });

                let start_iter = buffer.iter_at_offset(mat.start() as i32);
                let end_iter = buffer.iter_at_offset(mat.end() as i32);
                buffer.apply_tag(&tag_table[&tag_name], &start_iter, &end_iter);
            }
        }
    }

    /// Apply syntect-based syntax highlighting to markdown text
    pub fn apply_syntax_coloring(
        buffer: &sourceview5::Buffer,
        text: &str,
        tag_table: &mut HashMap<String, gtk4::TextTag>,
        theme_name: &str,
    ) {
        println!(
            "DEBUG: apply_syntax_coloring called with text length: {}, theme: {}",
            text.len(),
            theme_name
        );

        // Load syntax set and theme set (only from project assets)
        let ps = SyntaxSet::load_defaults_newlines();
        let mut ts = ThemeSet::new();
        
        // Load only our custom themes from assets - use exact file names
        if let Ok(entries) = std::fs::read_dir("src/assets/colorize_code_blocks") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "tmTheme" {
                        if let Ok(theme) = ThemeSet::get_theme(&path) {
                            if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                                ts.themes.insert(name.to_string(), theme);
                            }
                        }
                    }
                }
            }
        }

        // Map theme names to our exact tmTheme file names
        let actual_theme_name = match theme_name {
            "MarcoLight" | "light" => "light",
            "MarcoDark" | "dark" => "dark", 
            _ => theme_name, // Use as-is for any other theme names
        };

        // Use only our project themes - no fallbacks to external themes
        let theme = if let Some(theme) = ts.themes.get(actual_theme_name) {
            theme
        } else {
            eprintln!("WARNING: Theme '{}' not found in project assets", actual_theme_name);
            // If we can't find the requested theme, don't apply syntax highlighting
            return;
        };

        println!("DEBUG: Using project theme: {} -> {}", theme_name, actual_theme_name);
        println!("DEBUG: Theme loaded: {:?}", theme.name.as_ref().unwrap_or(&"unknown".to_string()));

        // Find markdown syntax
        let syntax = ps
            .find_syntax_by_extension("md")
            .or_else(|| ps.find_syntax_by_name("Markdown"))
            .unwrap_or_else(|| ps.find_syntax_plain_text());

        println!("DEBUG: Using syntax: {}", syntax.name);

        let mut h = HighlightLines::new(syntax, theme);
        let mut total_tags_applied = 0;

        // Split text into lines and highlight each line
        for (line_idx, line) in text.lines().enumerate() {
            if let Ok(ranges) = h.highlight_line(line, &ps) {
                let mut char_offset = 0;

                // Calculate the starting position of this line in the buffer
                let line_start_offset: usize = text
                    .lines()
                    .take(line_idx)
                    .map(|l| l.len() + 1) // +1 for newline
                    .sum();

                for (style, segment) in ranges {
                    if !segment.is_empty() {
                        let start_pos = line_start_offset + char_offset;
                        let end_pos = start_pos + segment.len();

                        // Create a unique tag name based on the style
                        let tag_name = format!(
                            "syntect_{}_{}_{}_{}",
                            style.foreground.r,
                            style.foreground.g,
                            style.foreground.b,
                            if style
                                .font_style
                                .contains(syntect::highlighting::FontStyle::BOLD)
                            {
                                "bold"
                            } else {
                                "normal"
                            }
                        );

                        ensure_tag_exists(buffer, tag_table, &tag_name, |tag| {
                            let fg_color = syntect_color_to_rgba(style.foreground);
                            tag.set_foreground_rgba(Some(&fg_color));

                            if style
                                .font_style
                                .contains(syntect::highlighting::FontStyle::BOLD)
                            {
                                tag.set_weight(700); // Bold weight
                            }
                            if style
                                .font_style
                                .contains(syntect::highlighting::FontStyle::ITALIC)
                            {
                                tag.set_style(pango::Style::Italic);
                            }
                            if style
                                .font_style
                                .contains(syntect::highlighting::FontStyle::UNDERLINE)
                            {
                                tag.set_underline(pango::Underline::Single);
                            }
                        });

                        // Apply the tag to the buffer
                        let start_iter = buffer.iter_at_offset(start_pos as i32);
                        let end_iter = buffer.iter_at_offset(end_pos as i32);
                        buffer.apply_tag(&tag_table[&tag_name], &start_iter, &end_iter);
                        total_tags_applied += 1;
                    }

                    char_offset += segment.len();
                }
            }
        }

        println!(
            "DEBUG: Applied {} syntect tags to buffer",
            total_tags_applied
        );
    }
}

impl MarkdownEditor {
    // Extra Markdown Syntax Methods

    // --- Markdown Color Highlighting Logic ---

    /// Highlight colored text using style attributes (moved from advanced.rs)
    pub fn highlight_colored_text(
        &self,
        buffer: &sourceview5::Buffer,
        text: &str,
        tag_table: &mut HashMap<String, gtk4::TextTag>,
        color_regex: &regex::Regex,
        font_color_regex: &regex::Regex,
    ) {
        // Delegate to the standalone function in the color module
        crate::editor::syntax::color::highlight_colored_text(
            buffer,
            text,
            tag_table,
            color_regex,
            font_color_regex,
        )
    }

    /// Apply syntect-based syntax highlighting
    pub fn apply_syntect_highlighting(
        &self,
        buffer: &sourceview5::Buffer,
        text: &str,
        tag_table: &mut HashMap<String, gtk4::TextTag>,
        theme_manager: &crate::theme::ThemeManager,
    ) {
        // Get the appropriate theme name from the ThemeManager
        let theme_name = theme_manager.get_syntax_theme_name();
        
        // Apply the syntax coloring with the correct theme
        crate::editor::syntax::color::apply_syntax_coloring(buffer, text, tag_table, &theme_name)
    }

    /// Parse color names and hex codes to RGBA (delegated to the color module)
    pub fn parse_color(color: &str) -> Option<gdk4::RGBA> {
        crate::editor::syntax::color::parse_color(color)
    }

    /// Ensure a tag exists in the tag table (delegated to the color module)
    pub fn ensure_tag_exists<F>(
        buffer: &sourceview5::Buffer,
        tag_table: &mut HashMap<String, gtk4::TextTag>,
        name: &str,
        configure: F,
    ) where
        F: FnOnce(&gtk4::TextTag),
    {
        crate::editor::syntax::color::ensure_tag_exists(buffer, tag_table, name, configure)
    }

    /// Insert underlined text
    pub fn insert_underline(&self, text: &str) {
        crate::markdown::advanced::insert_underline(self, text);
    }

    /// Insert centered text
    pub fn insert_center_text(&self, text: &str) {
        crate::markdown::advanced::insert_center_text(self, text);
    }

    /// Insert colored text
    pub fn insert_colored_text(&self, text: &str, color: &str) {
        crate::markdown::advanced::insert_colored_text(self, text, color);
    }

    /// Insert a markdown comment
    pub fn insert_comment(&self, comment: &str) {
        crate::markdown::advanced::insert_comment(self, comment);
    }

    /// Insert an admonition
    pub fn insert_admonition(&self, emoji: &str, adm_type: &str, text: &str) {
        crate::markdown::advanced::insert_admonition(self, emoji, adm_type, text);
    }

    /// Insert image with size
    pub fn insert_image_with_size(
        &self,
        src: &str,
        alt: &str,
        width: Option<&str>,
        height: Option<&str>,
    ) {
        crate::markdown::advanced::insert_image_with_size(self, src, alt, width, height);
    }

    /// Insert image with caption
    pub fn insert_image_with_caption(&self, src: &str, alt: &str, caption: &str) {
        crate::markdown::advanced::insert_image_with_caption(self, src, alt, caption);
    }

    /// Insert link with target
    pub fn insert_link_with_target(&self, url: &str, text: &str, target: &str) {
        crate::markdown::advanced::insert_link_with_target(self, url, text, target);
    }

    /// Insert HTML entity
    pub fn insert_html_entity(&self, entity: &str) {
        crate::markdown::advanced::insert_html_entity(self, entity);
    }

    /// Insert table of contents
    pub fn insert_table_of_contents(&self) {
        crate::markdown::advanced::insert_table_of_contents(self);
    }

    /// Insert YouTube video embed
    pub fn insert_youtube_video(&self, video_id: &str, alt_text: &str) {
        crate::markdown::advanced::insert_youtube_video(self, video_id, alt_text);
    }

    /// Insert indented text
    pub fn insert_indented_text(&self, text: &str, indent_level: usize) {
        crate::markdown::advanced::insert_indented_text(self, text, indent_level);
    }

    /// Get common HTML entities for UI
    pub fn get_common_html_entities() -> Vec<(&'static str, &'static str, &'static str)> {
        crate::markdown::advanced::get_common_html_entities()
    }

    /// Get common admonition types for UI
    pub fn get_common_admonitions() -> Vec<(&'static str, &'static str, &'static str)> {
        crate::markdown::advanced::get_common_admonitions()
    }
}
