use gtk4::prelude::*;
use regex::Regex;
use std::collections::HashMap;

/// Extra markdown syntax features and hacks based on https://www.markdownguide.org/hacks/
/// These are advanced features that extend beyond standard markdown syntax
pub struct ExtraMarkdownSyntax {
    // Regex patterns for various extra syntax elements
    underline_regex: Regex,
    center_regex: Regex,
    color_regex: Regex,
    font_color_regex: Regex,
    comment_regex: Regex,
    admonition_regex: Regex,
    image_size_regex: Regex,
    image_caption_regex: Regex,
    link_target_regex: Regex,
    html_entity_regex: Regex,
    table_linebreak_regex: Regex,
    table_list_regex: Regex,
    video_embed_regex: Regex,
    indent_regex: Regex,
    github_admonition_regex: Regex,
}

impl ExtraMarkdownSyntax {
    pub fn new() -> Self {
        Self {
            // Underline: <ins>text</ins>
            underline_regex: Regex::new(r"<ins>(.*?)</ins>").unwrap(),
            
            // Center: <center>text</center> or <p style="text-align:center">text</p>
            center_regex: Regex::new(r"<center>(.*?)</center>").unwrap(),
            
            // Color: <p style="color:colorname">text</p>
            color_regex: Regex::new(r#"<p\s+style="color:([^"]+)">(.*?)</p>"#).unwrap(),
            
            // Font color (deprecated): <font color="red">text</font>
            font_color_regex: Regex::new(r#"<font\s+color="([^"]+)">(.*?)</font>"#).unwrap(),
            
            // Comments: [comment]: # or [comment]: # (text)
            comment_regex: Regex::new(r"^\s*\[([^\]]+)\]:\s*#\s*(.*)$").unwrap(),
            
            // Admonitions: > :emoji: **Type:** text
            admonition_regex: Regex::new(r">\s*:([^:]+):\s*\*\*([^*]+):\*\*\s*(.*)").unwrap(),
            
            // GitHub-style admonitions: > [!TYPE]
            github_admonition_regex: Regex::new(r"^\s*>\s*\[!([A-Z]+)\]\s*(.*)$").unwrap(),
            
            // Image with size: <img src="..." width="..." height="...">
            image_size_regex: Regex::new(r#"<img\s+src="([^"]+)"(?:\s+width="([^"]+)")?(?:\s+height="([^"]+)")?[^>]*>"#).unwrap(),
            
            // Image caption: <figure><img...><figcaption>...</figcaption></figure>
            image_caption_regex: Regex::new(r#"<figure>\s*<img[^>]+>\s*<figcaption>(.*?)</figcaption>\s*</figure>"#).unwrap(),
            
            // Link target: <a href="..." target="_blank">text</a>
            link_target_regex: Regex::new(r#"<a\s+href="([^"]+)"\s+target="([^"]+)">(.*?)</a>"#).unwrap(),
            
            // HTML entities: &entity;
            html_entity_regex: Regex::new(r"&([a-zA-Z0-9#]+);").unwrap(),
            
            // Table line breaks: <br>
            table_linebreak_regex: Regex::new(r"<br\s*/?>").unwrap(),
            
            // Table lists: <ul><li>...</li></ul>
            table_list_regex: Regex::new(r"<ul>(.*?)</ul>").unwrap(),
            
            // Video embeds: [![alt](thumbnail)](video_url)
            video_embed_regex: Regex::new(r"!\[([^\]]*)\]\(https://img\.youtube\.com/vi/([^/]+)/[^)]+\)\]\(https://www\.youtube\.com/watch\?v=([^)]+)\)").unwrap(),
            
            // Indent with &nbsp;
            indent_regex: Regex::new(r"^(\s*)((?:&nbsp;)+)(.*)$").unwrap(),
        }
    }

    /// Apply extra syntax highlighting to a text buffer
    pub fn apply_extra_syntax_highlighting(
        &self, 
        buffer: &sourceview5::Buffer, 
        text: &str, 
        tag_table: &mut HashMap<String, gtk4::TextTag>
    ) {
        self.highlight_underline(buffer, text, tag_table);
        self.highlight_center_text(buffer, text, tag_table);
        self.highlight_colored_text(buffer, text, tag_table);
        self.highlight_comments(buffer, text, tag_table);
        self.highlight_admonitions(buffer, text, tag_table);
        self.highlight_github_admonitions(buffer, text, tag_table);
        self.highlight_image_extensions(buffer, text, tag_table);
        self.highlight_link_extensions(buffer, text, tag_table);
        self.highlight_html_entities(buffer, text, tag_table);
        self.highlight_table_extensions(buffer, text, tag_table);
        self.highlight_video_embeds(buffer, text, tag_table);
        self.highlight_indentation(buffer, text, tag_table);
    }

    /// Highlight underlined text using <ins> tags
    fn highlight_underline(&self, buffer: &sourceview5::Buffer, text: &str, tag_table: &mut HashMap<String, gtk4::TextTag>) {
        self.ensure_tag_exists(tag_table, "underline", |tag| {
            tag.set_underline(pango::Underline::Single);
            tag.set_foreground_rgba(Some(&gdk4::RGBA::new(0.2, 0.4, 0.8, 1.0))); // Blue underline
        });

        for mat in self.underline_regex.find_iter(text) {
            let start_iter = buffer.iter_at_offset(mat.start() as i32);
            let end_iter = buffer.iter_at_offset(mat.end() as i32);
            buffer.apply_tag(&tag_table["underline"], &start_iter, &end_iter);
        }
    }

    /// Highlight centered text using <center> tags
    fn highlight_center_text(&self, buffer: &sourceview5::Buffer, text: &str, tag_table: &mut HashMap<String, gtk4::TextTag>) {
        self.ensure_tag_exists(tag_table, "center", |tag| {
            tag.set_justification(gtk4::Justification::Center);
            tag.set_foreground_rgba(Some(&gdk4::RGBA::new(0.3, 0.6, 0.3, 1.0))); // Green
        });

        for mat in self.center_regex.find_iter(text) {
            let start_iter = buffer.iter_at_offset(mat.start() as i32);
            let end_iter = buffer.iter_at_offset(mat.end() as i32);
            buffer.apply_tag(&tag_table["center"], &start_iter, &end_iter);
        }
    }

    /// Highlight colored text using style attributes
    fn highlight_colored_text(&self, buffer: &sourceview5::Buffer, text: &str, tag_table: &mut HashMap<String, gtk4::TextTag>) {
        // Handle <p style="color:..."> tags
        for captures in self.color_regex.captures_iter(text) {
            if let Some(mat) = captures.get(0) {
                let color_name = &captures[1];
                let tag_name = format!("color_{}", color_name);
                
                self.ensure_tag_exists(tag_table, &tag_name, |tag| {
                    if let Some(rgba) = self.parse_color(color_name) {
                        tag.set_foreground_rgba(Some(&rgba));
                    }
                });

                let start_iter = buffer.iter_at_offset(mat.start() as i32);
                let end_iter = buffer.iter_at_offset(mat.end() as i32);
                buffer.apply_tag(&tag_table[&tag_name], &start_iter, &end_iter);
            }
        }

        // Handle <font color="..."> tags (deprecated but still used)
        for captures in self.font_color_regex.captures_iter(text) {
            if let Some(mat) = captures.get(0) {
                let color_name = &captures[1];
                let tag_name = format!("font_color_{}", color_name);
                
                self.ensure_tag_exists(tag_table, &tag_name, |tag| {
                    if let Some(rgba) = self.parse_color(color_name) {
                        tag.set_foreground_rgba(Some(&rgba));
                    }
                });

                let start_iter = buffer.iter_at_offset(mat.start() as i32);
                let end_iter = buffer.iter_at_offset(mat.end() as i32);
                buffer.apply_tag(&tag_table[&tag_name], &start_iter, &end_iter);
            }
        }
    }

    /// Highlight markdown comments
    fn highlight_comments(&self, buffer: &sourceview5::Buffer, text: &str, tag_table: &mut HashMap<String, gtk4::TextTag>) {
        self.ensure_tag_exists(tag_table, "comment", |tag| {
            tag.set_foreground_rgba(Some(&gdk4::RGBA::new(0.5, 0.5, 0.5, 0.7))); // Gray, semi-transparent
            tag.set_style(pango::Style::Italic);
        });

        for captures in self.comment_regex.captures_iter(text) {
            if let Some(mat) = captures.get(0) {
                let start_iter = buffer.iter_at_offset(mat.start() as i32);
                let end_iter = buffer.iter_at_offset(mat.end() as i32);
                buffer.apply_tag(&tag_table["comment"], &start_iter, &end_iter);
            }
        }
    }

    /// Highlight admonitions (> :emoji: **Type:** text)
    fn highlight_admonitions(&self, buffer: &sourceview5::Buffer, text: &str, tag_table: &mut HashMap<String, gtk4::TextTag>) {
        self.ensure_tag_exists(tag_table, "admonition", |tag| {
            tag.set_background_rgba(Some(&gdk4::RGBA::new(0.95, 0.95, 0.8, 1.0))); // Light yellow background
            tag.set_left_margin(20);
            tag.set_right_margin(20);
        });

        for captures in self.admonition_regex.captures_iter(text) {
            if let Some(mat) = captures.get(0) {
                let start_iter = buffer.iter_at_offset(mat.start() as i32);
                let end_iter = buffer.iter_at_offset(mat.end() as i32);
                buffer.apply_tag(&tag_table["admonition"], &start_iter, &end_iter);
            }
        }
    }

    /// Highlight GitHub-style admonitions (> [!TYPE])
    fn highlight_github_admonitions(&self, buffer: &sourceview5::Buffer, text: &str, tag_table: &mut HashMap<String, gtk4::TextTag>) {
        // Create different tags for different admonition types
        let admonition_types = [
            ("NOTE", gdk4::RGBA::new(0.0, 0.5, 1.0, 0.1)), // Blue
            ("TIP", gdk4::RGBA::new(0.0, 0.8, 0.0, 0.1)), // Green
            ("IMPORTANT", gdk4::RGBA::new(0.5, 0.0, 1.0, 0.1)), // Purple
            ("WARNING", gdk4::RGBA::new(1.0, 0.6, 0.0, 0.1)), // Orange
            ("CAUTION", gdk4::RGBA::new(1.0, 0.0, 0.0, 0.1)), // Red
        ];

        for (adm_type, color) in &admonition_types {
            let tag_name = format!("github_admonition_{}", adm_type.to_lowercase());
            self.ensure_tag_exists(tag_table, &tag_name, |tag| {
                tag.set_background_rgba(Some(color));
                tag.set_left_margin(10);
                tag.set_right_margin(10);
                tag.set_weight(600); // Semi-bold
            });
        }

        // Default tag for unknown admonition types
        self.ensure_tag_exists(tag_table, "github_admonition_default", |tag| {
            tag.set_background_rgba(Some(&gdk4::RGBA::new(0.9, 0.9, 0.9, 0.3))); // Light gray
            tag.set_left_margin(10);
            tag.set_right_margin(10);
            tag.set_weight(600);
        });

        for captures in self.github_admonition_regex.captures_iter(text) {
            if let Some(mat) = captures.get(0) {
                let adm_type = &captures[1];
                let tag_name = format!("github_admonition_{}", adm_type.to_lowercase());
                
                // Use specific tag if it exists, otherwise use default
                let tag_to_use = if tag_table.contains_key(&tag_name) {
                    &tag_name
                } else {
                    "github_admonition_default"
                };

                let start_iter = buffer.iter_at_offset(mat.start() as i32);
                let end_iter = buffer.iter_at_offset(mat.end() as i32);
                buffer.apply_tag(&tag_table[tag_to_use], &start_iter, &end_iter);
            }
        }
    }

    /// Highlight extended image syntax
    fn highlight_image_extensions(&self, buffer: &sourceview5::Buffer, text: &str, tag_table: &mut HashMap<String, gtk4::TextTag>) {
        self.ensure_tag_exists(tag_table, "image_size", |tag| {
            tag.set_foreground_rgba(Some(&gdk4::RGBA::new(0.8, 0.4, 0.8, 1.0))); // Purple
            tag.set_weight(700); // Bold weight
        });

        // Highlight images with size attributes
        for mat in self.image_size_regex.find_iter(text) {
            let start_iter = buffer.iter_at_offset(mat.start() as i32);
            let end_iter = buffer.iter_at_offset(mat.end() as i32);
            buffer.apply_tag(&tag_table["image_size"], &start_iter, &end_iter);
        }

        // Highlight image captions
        for mat in self.image_caption_regex.find_iter(text) {
            let start_iter = buffer.iter_at_offset(mat.start() as i32);
            let end_iter = buffer.iter_at_offset(mat.end() as i32);
            buffer.apply_tag(&tag_table["image_size"], &start_iter, &end_iter);
        }
    }

    /// Highlight extended link syntax
    fn highlight_link_extensions(&self, buffer: &sourceview5::Buffer, text: &str, tag_table: &mut HashMap<String, gtk4::TextTag>) {
        self.ensure_tag_exists(tag_table, "link_target", |tag| {
            tag.set_foreground_rgba(Some(&gdk4::RGBA::new(0.0, 0.4, 0.8, 1.0))); // Blue
            tag.set_underline(pango::Underline::Single);
        });

        for mat in self.link_target_regex.find_iter(text) {
            let start_iter = buffer.iter_at_offset(mat.start() as i32);
            let end_iter = buffer.iter_at_offset(mat.end() as i32);
            buffer.apply_tag(&tag_table["link_target"], &start_iter, &end_iter);
        }
    }

    /// Highlight HTML entities
    fn highlight_html_entities(&self, buffer: &sourceview5::Buffer, text: &str, tag_table: &mut HashMap<String, gtk4::TextTag>) {
        self.ensure_tag_exists(tag_table, "html_entity", |tag| {
            tag.set_foreground_rgba(Some(&gdk4::RGBA::new(0.6, 0.3, 0.8, 1.0))); // Purple
            tag.set_family(Some("monospace"));
        });

        for mat in self.html_entity_regex.find_iter(text) {
            let start_iter = buffer.iter_at_offset(mat.start() as i32);
            let end_iter = buffer.iter_at_offset(mat.end() as i32);
            buffer.apply_tag(&tag_table["html_entity"], &start_iter, &end_iter);
        }
    }

    /// Highlight table extensions
    fn highlight_table_extensions(&self, buffer: &sourceview5::Buffer, text: &str, tag_table: &mut HashMap<String, gtk4::TextTag>) {
        self.ensure_tag_exists(tag_table, "table_extension", |tag| {
            tag.set_foreground_rgba(Some(&gdk4::RGBA::new(0.8, 0.6, 0.2, 1.0))); // Orange
            tag.set_family(Some("monospace"));
        });

        // Highlight line breaks in tables
        for mat in self.table_linebreak_regex.find_iter(text) {
            let start_iter = buffer.iter_at_offset(mat.start() as i32);
            let end_iter = buffer.iter_at_offset(mat.end() as i32);
            buffer.apply_tag(&tag_table["table_extension"], &start_iter, &end_iter);
        }

        // Highlight lists in tables
        for mat in self.table_list_regex.find_iter(text) {
            let start_iter = buffer.iter_at_offset(mat.start() as i32);
            let end_iter = buffer.iter_at_offset(mat.end() as i32);
            buffer.apply_tag(&tag_table["table_extension"], &start_iter, &end_iter);
        }
    }

    /// Highlight video embeds
    fn highlight_video_embeds(&self, buffer: &sourceview5::Buffer, text: &str, tag_table: &mut HashMap<String, gtk4::TextTag>) {
        self.ensure_tag_exists(tag_table, "video_embed", |tag| {
            tag.set_foreground_rgba(Some(&gdk4::RGBA::new(0.8, 0.2, 0.2, 1.0))); // Red
            tag.set_weight(700); // Bold weight
        });

        for mat in self.video_embed_regex.find_iter(text) {
            let start_iter = buffer.iter_at_offset(mat.start() as i32);
            let end_iter = buffer.iter_at_offset(mat.end() as i32);
            buffer.apply_tag(&tag_table["video_embed"], &start_iter, &end_iter);
        }
    }

    /// Highlight indentation using &nbsp;
    fn highlight_indentation(&self, buffer: &sourceview5::Buffer, text: &str, tag_table: &mut HashMap<String, gtk4::TextTag>) {
        self.ensure_tag_exists(tag_table, "indent", |tag| {
            tag.set_background_rgba(Some(&gdk4::RGBA::new(0.9, 0.9, 1.0, 0.5))); // Light blue background
        });

        for captures in self.indent_regex.captures_iter(text) {
            if let Some(mat) = captures.get(2) { // Just highlight the &nbsp; part
                let start_iter = buffer.iter_at_offset(mat.start() as i32);
                let end_iter = buffer.iter_at_offset(mat.end() as i32);
                buffer.apply_tag(&tag_table["indent"], &start_iter, &end_iter);
            }
        }
    }

    /// Parse color names and hex codes to RGBA
    fn parse_color(&self, color: &str) -> Option<gdk4::RGBA> {
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

    /// Ensure a tag exists in the tag table
    fn ensure_tag_exists<F>(&self, tag_table: &mut HashMap<String, gtk4::TextTag>, name: &str, configure: F)
    where
        F: FnOnce(&gtk4::TextTag),
    {
        if !tag_table.contains_key(name) {
            let tag = gtk4::TextTag::new(Some(name));
            configure(&tag);
            tag_table.insert(name.to_string(), tag);
        }
    }
}

/// Helper functions for inserting extra markdown syntax

/// Insert underlined text
pub fn insert_underline(editor: &crate::editor::MarkdownEditor, text: &str) {
    let underlined = format!("<ins>{}</ins>", text);
    editor.insert_text_at_cursor(&underlined);
}

/// Insert centered text
pub fn insert_center_text(editor: &crate::editor::MarkdownEditor, text: &str) {
    let centered = format!("<center>{}</center>", text);
    editor.insert_text_at_cursor(&centered);
}

/// Insert colored text using CSS style
pub fn insert_colored_text(editor: &crate::editor::MarkdownEditor, text: &str, color: &str) {
    let colored = format!(r#"<p style="color:{}">{}</p>"#, color, text);
    editor.insert_text_at_cursor(&colored);
}

/// Insert a markdown comment
pub fn insert_comment(editor: &crate::editor::MarkdownEditor, comment: &str) {
    let comment_text = format!("[{}]: #\n", comment);
    editor.insert_text_at_cursor(&comment_text);
}

/// Insert an admonition
pub fn insert_admonition(editor: &crate::editor::MarkdownEditor, emoji: &str, adm_type: &str, text: &str) {
    let admonition = format!("> :{}: **{}:** {}\n", emoji, adm_type, text);
    editor.insert_text_at_cursor(&admonition);
}

/// Insert image with size
pub fn insert_image_with_size(editor: &crate::editor::MarkdownEditor, src: &str, alt: &str, width: Option<&str>, height: Option<&str>) {
    let mut img_tag = format!(r#"<img src="{}" alt="{}""#, src, alt);
    
    if let Some(w) = width {
        img_tag.push_str(&format!(r#" width="{}""#, w));
    }
    if let Some(h) = height {
        img_tag.push_str(&format!(r#" height="{}""#, h));
    }
    img_tag.push('>');
    
    editor.insert_text_at_cursor(&img_tag);
}

/// Insert image with caption
pub fn insert_image_with_caption(editor: &crate::editor::MarkdownEditor, src: &str, alt: &str, caption: &str) {
    let img_with_caption = format!(
        "<figure>\n    <img src=\"{}\" alt=\"{}\">\n    <figcaption>{}</figcaption>\n</figure>",
        src, alt, caption
    );
    editor.insert_text_at_cursor(&img_with_caption);
}

/// Insert link with target
pub fn insert_link_with_target(editor: &crate::editor::MarkdownEditor, url: &str, text: &str, target: &str) {
    let link = format!(r#"<a href="{}" target="{}">{}</a>"#, url, target, text);
    editor.insert_text_at_cursor(&link);
}

/// Insert HTML entity
pub fn insert_html_entity(editor: &crate::editor::MarkdownEditor, entity: &str) {
    let entity_text = format!("&{};", entity);
    editor.insert_text_at_cursor(&entity_text);
}

/// Insert table of contents placeholder
pub fn insert_table_of_contents(editor: &crate::editor::MarkdownEditor) {
    let toc = "#### Table of Contents\n\n- [Section 1](#section-1)\n- [Section 2](#section-2)\n- [Section 3](#section-3)\n\n";
    editor.insert_text_at_cursor(toc);
}

/// Insert YouTube video embed
pub fn insert_youtube_video(editor: &crate::editor::MarkdownEditor, video_id: &str, alt_text: &str) {
    let video_embed = format!(
        "[![{}](https://img.youtube.com/vi/{}/0.jpg)](https://www.youtube.com/watch?v={})",
        alt_text, video_id, video_id
    );
    editor.insert_text_at_cursor(&video_embed);
}

/// Insert indented text using &nbsp;
pub fn insert_indented_text(editor: &crate::editor::MarkdownEditor, text: &str, indent_level: usize) {
    let indent = "&nbsp;".repeat(indent_level * 4); // 4 &nbsp; per indent level
    let indented = format!("{}{}", indent, text);
    editor.insert_text_at_cursor(&indented);
}

/// Common HTML entities mapping
pub fn get_common_html_entities() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("copy", "©", "Copyright symbol"),
        ("reg", "®", "Registered trademark"),
        ("trade", "™", "Trademark"),
        ("euro", "€", "Euro symbol"),
        ("larr", "←", "Left arrow"),
        ("uarr", "↑", "Up arrow"),
        ("rarr", "→", "Right arrow"),
        ("darr", "↓", "Down arrow"),
        ("nbsp", " ", "Non-breaking space"),
        ("amp", "&", "Ampersand"),
        ("lt", "<", "Less than"),
        ("gt", ">", "Greater than"),
        ("quot", "\"", "Quotation mark"),
        ("apos", "'", "Apostrophe"),
        ("#176", "°", "Degree symbol"),
        ("#960", "π", "Pi symbol"),
    ]
}

/// Common admonition types with their emojis
pub fn get_common_admonitions() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("warning", "⚠️", "Warning"),
        ("note", "📝", "Note"),
        ("tip", "💡", "Tip"),
        ("info", "ℹ️", "Information"),
        ("danger", "🚨", "Danger"),
        ("success", "✅", "Success"),
        ("error", "❌", "Error"),
        ("bug", "🐛", "Bug"),
        ("question", "❓", "Question"),
        ("important", "❗", "Important"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_parsing() {
        let syntax = ExtraMarkdownSyntax::new();
        
        assert!(syntax.parse_color("red").is_some());
        assert!(syntax.parse_color("#FF0000").is_some());
        assert!(syntax.parse_color("invalid").is_none());
    }

    #[test]
    fn test_regex_patterns() {
        let syntax = ExtraMarkdownSyntax::new();
        
        assert!(syntax.underline_regex.is_match("<ins>underlined</ins>"));
        assert!(syntax.center_regex.is_match("<center>centered</center>"));
        assert!(syntax.comment_regex.is_match("[comment]: #"));
    }
}
