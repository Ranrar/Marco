use sourceview5::prelude::*;
use sourceview5::StyleSchemeManager;
use crate::editor::core::MarkdownEditor;

impl MarkdownEditor {
    /// Switch between HTML and code views
    pub fn set_view_mode(&self, mode: &str) {
        self.view_stack.set_visible_child_name(mode);
    }
    
    /// Get the current view mode
    #[allow(dead_code)]
    pub fn get_view_mode(&self) -> String {
        self.view_stack.visible_child_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "html".to_string())
    }

    /// Set the theme manager for both the HTML view and source editor
    pub fn set_theme_manager(&self, theme_manager: crate::theme::ThemeManager) {
        // Store the theme manager
        *self.theme_manager.borrow_mut() = Some(theme_manager.clone());
        
        // Apply to HTML view
        self.html_view.set_theme_manager(theme_manager.clone());
        
        // Apply to source editor
        self.update_source_editor_theme(&theme_manager);
    }

    /// Update the source editor theme based on the theme manager
    fn update_source_editor_theme(&self, theme_manager: &crate::theme::ThemeManager) {
        let style_manager = StyleSchemeManager::default();
        
        // Choose appropriate style scheme based on theme
        let preferred_schemes = match theme_manager.get_effective_theme() {
            crate::theme::Theme::Light => vec!["Adwaita", "classic", "tango", "kate", "solarized-light"],
            crate::theme::Theme::Dark => vec!["Adwaita-dark", "oblivion", "cobalt", "monokai", "solarized-dark"],
            crate::theme::Theme::System => {
                // For system theme, detect and choose appropriate schemes
                match crate::theme::ThemeManager::detect_system_theme() {
                    crate::theme::Theme::Dark => vec!["Adwaita-dark", "oblivion", "cobalt", "monokai", "solarized-dark"],
                    _ => vec!["Adwaita", "classic", "tango", "kate", "solarized-light"],
                }
            }
        };
        
        // Try to find the first available scheme from the preferred list
        let mut applied_scheme = false;
        for scheme_name in preferred_schemes {
            if let Some(scheme) = style_manager.scheme(scheme_name) {
                self.source_buffer.set_style_scheme(Some(&scheme));
                applied_scheme = true;
                break;
            }
        }
        
        // Ultimate fallback - use default scheme
        if !applied_scheme {
            if let Some(scheme) = style_manager.scheme("Adwaita") {
                self.source_buffer.set_style_scheme(Some(&scheme));
            }
        }
    }

    /// Refresh both the HTML view and source editor (useful after theme changes)
    pub fn refresh_html_view(&self) {
        self.html_view.refresh_with_current_content();
        
        // Also refresh source editor theme if we have a theme manager
        if let Some(ref theme_manager) = *self.theme_manager.borrow() {
            self.update_source_editor_theme(theme_manager);
        }
    }

    /// Set the CSS theme for the preview
    pub fn set_css_theme(&self, theme_name: &str) {
        *self.current_css_theme.borrow_mut() = theme_name.to_string();
        
        // Load CSS file from the css/ directory
        let css_path = format!("css/{}.css", theme_name);
        match std::fs::read_to_string(&css_path) {
            Ok(css_content) => {
                self.html_view.set_custom_css(&css_content);
                self.refresh_html_view();
            }
            Err(e) => {
                eprintln!("Failed to load CSS theme '{}': {}", theme_name, e);
                // Fallback to standard theme
                if theme_name != "standard" {
                    self.set_css_theme("standard");
                }
            }
        }
    }
    
    /// Get the current CSS theme name
    pub fn get_current_css_theme(&self) -> String {
        self.current_css_theme.borrow().clone()
    }

    /// Get available CSS themes by scanning the css/ directory
    pub fn get_available_css_themes() -> Vec<(String, String, String)> {
        let mut themes = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir("css") {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if let Some(extension) = path.extension() {
                        if extension == "css" {
                            if let Some(filename) = path.file_stem() {
                                let theme_id = filename.to_string_lossy().to_string();
                                let display_name = theme_id.replace('_', " ")
                                    .split(' ')
                                    .map(|word| {
                                        let mut chars = word.chars();
                                        match chars.next() {
                                            None => String::new(),
                                            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                                        }
                                    })
                                    .collect::<Vec<String>>()
                                    .join(" ");
                                
                                // Create a properly sanitized name for action IDs (no spaces, special chars)
                                let sanitized_name = theme_id.replace(|c: char| !c.is_alphanumeric(), "_");
                                
                                themes.push((theme_id, display_name, sanitized_name));
                            }
                        }
                    }
                }
            }
        }
        
        // Sort themes alphabetically by display name
        themes.sort_by(|a, b| a.1.cmp(&b.1));
        themes
    }
}
