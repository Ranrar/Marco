use crate::editor::core::MarkdownEditor;

impl MarkdownEditor {
    // Extra Markdown Syntax Methods
    
    /// Insert underlined text
    pub fn insert_underline(&self, text: &str) {
        crate::md_advanced::insert_underline(self, text);
    }

    /// Insert centered text
    pub fn insert_center_text(&self, text: &str) {
        crate::md_advanced::insert_center_text(self, text);
    }

    /// Insert colored text
    pub fn insert_colored_text(&self, text: &str, color: &str) {
        crate::md_advanced::insert_colored_text(self, text, color);
    }

    /// Insert a markdown comment
    pub fn insert_comment(&self, comment: &str) {
        crate::md_advanced::insert_comment(self, comment);
    }

    /// Insert an admonition
    pub fn insert_admonition(&self, emoji: &str, adm_type: &str, text: &str) {
        crate::md_advanced::insert_admonition(self, emoji, adm_type, text);
    }

    /// Insert image with size
    pub fn insert_image_with_size(&self, src: &str, alt: &str, width: Option<&str>, height: Option<&str>) {
        crate::md_advanced::insert_image_with_size(self, src, alt, width, height);
    }

    /// Insert image with caption
    pub fn insert_image_with_caption(&self, src: &str, alt: &str, caption: &str) {
        crate::md_advanced::insert_image_with_caption(self, src, alt, caption);
    }

    /// Insert link with target
    pub fn insert_link_with_target(&self, url: &str, text: &str, target: &str) {
        crate::md_advanced::insert_link_with_target(self, url, text, target);
    }

    /// Insert HTML entity
    pub fn insert_html_entity(&self, entity: &str) {
        crate::md_advanced::insert_html_entity(self, entity);
    }

    /// Insert table of contents
    pub fn insert_table_of_contents(&self) {
        crate::md_advanced::insert_table_of_contents(self);
    }

    /// Insert YouTube video embed
    pub fn insert_youtube_video(&self, video_id: &str, alt_text: &str) {
        crate::md_advanced::insert_youtube_video(self, video_id, alt_text);
    }

    /// Insert indented text
    pub fn insert_indented_text(&self, text: &str, indent_level: usize) {
        crate::md_advanced::insert_indented_text(self, text, indent_level);
    }

    /// Get common HTML entities for UI
    pub fn get_common_html_entities() -> Vec<(&'static str, &'static str, &'static str)> {
        crate::md_advanced::get_common_html_entities()
    }

    /// Get common admonition types for UI
    pub fn get_common_admonitions() -> Vec<(&'static str, &'static str, &'static str)> {
        crate::md_advanced::get_common_admonitions()
    }
}
