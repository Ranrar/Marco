use crate::editor::core::MarkdownEditor;
use std::collections::HashMap;

// Public standalone color functions
pub mod color {
    /// Wrapper for ThemeSet::get_theme for clarity
    pub fn get_ui_theme(path: &std::path::Path) -> Result<syntect::highlighting::Theme, syntect::LoadingError> {
        syntect::highlighting::ThemeSet::get_theme(path)
    }
    use gtk4::prelude::*;
    use sourceview5;
    use std::collections::HashMap;
    // Syntect-based highlighting removed

    /// Parse color names and hex codes to RGBA to the editor
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

    // Syntect-based syntax highlighting removed
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
        crate::editor::syntax::color_syntax::color::highlight_colored_text(
            buffer,
            text,
            tag_table,
            color_regex,
            font_color_regex,
        )
    }

    /// Parse color names and hex codes to RGBA (delegated to the color module)
    pub fn parse_color(color: &str) -> Option<gdk4::RGBA> {
        crate::editor::syntax::color_syntax::color::parse_color(color)
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
        crate::editor::syntax::color_syntax::color::ensure_tag_exists(buffer, tag_table, name, configure)
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
