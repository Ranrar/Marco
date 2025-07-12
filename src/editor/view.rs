use crate::editor::core::MarkdownEditor;
use sourceview5::prelude::*;

impl MarkdownEditor {
    /// Switch between HTML and code views
    pub fn set_view_mode(&self, mode: &str) {
        self.view_stack.set_visible_child_name(mode);
        if mode == "code" {
            // Get the current markdown text from the source buffer
            let gtk_buffer = self.source_buffer.upcast_ref::<gtk4::TextBuffer>();
            let start = gtk_buffer.start_iter();
            let end = gtk_buffer.end_iter();
            let text = gtk_buffer.text(&start, &end, false).to_string();
            self.code_view.update_content(&text);
        }
    }

    /// Get the current view mode
    #[allow(dead_code)]
    pub fn get_view_mode(&self) -> String {
        self.view_stack
            .visible_child_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "html".to_string())
    }

    /// Set the theme manager for both views and source editor
    pub fn set_theme_manager(&self, theme_manager: crate::theme::ThemeManager) {
        // Store the theme manager
        *self.theme_manager.borrow_mut() = Some(theme_manager.clone());

        // Apply to HTML view
        self.html_view.set_theme_manager(theme_manager.clone());

        // Apply to Code view
        self.code_view.set_theme_manager(theme_manager.clone());

        // Apply to source editor
        self.update_source_editor_theme(&theme_manager);

        // Register callback to update editor when theme changes
        let editor_weak = self.html_view.clone();
        let code_view_weak = self.code_view.clone();
        let theme_manager_weak = theme_manager.clone();

        theme_manager.add_theme_change_callback(move |_new_theme| {
            println!("DEBUG: Theme change callback triggered in editor");

            // Update HTML view
            editor_weak.set_theme_manager(theme_manager_weak.clone());

            // Update Code view
            code_view_weak.set_theme_manager(theme_manager_weak.clone());

            // Update source editor theme using the new approach
            // Note: The actual styling will be handled by the main update_source_editor_theme method
            // when the editor view is refreshed

            println!("DEBUG: Editor theme change callback completed");
        });
    }

    /// Update the source editor theme based on the theme manager
    fn update_source_editor_theme(&self, theme_manager: &crate::theme::ThemeManager) {
        // Get CSS content for background color extraction
        let css_content = match theme_manager.set_css_theme(&theme_manager.get_current_css_theme()) {
            Ok(css) => css,
            Err(_) => {
                eprintln!("Failed to load CSS theme for editor background");
                String::new()
            }
        };

        // Extract background color from current theme (for future use)
        let _bg_color = theme_manager.get_editor_background_color(&css_content);
        
        // Apply background color to the source view
        let style_context = self.source_view.style_context();
        
        // Remove any existing theme classes
        style_context.remove_class("theme-light");
        style_context.remove_class("theme-dark");
        
        // Add appropriate theme class
        match theme_manager.get_effective_theme() {
            crate::theme::Theme::Light => style_context.add_class("theme-light"),
            crate::theme::Theme::Dark => style_context.add_class("theme-dark"),
            crate::theme::Theme::System => {
                match crate::theme::ThemeManager::detect_system_theme() {
                    crate::theme::Theme::Dark => style_context.add_class("theme-dark"),
                    _ => style_context.add_class("theme-light"),
                }
            }
        }

        // For syntax highlighting, use only our project's colorize_code_blocks system
        // The actual syntax highlighting will be handled by the colorize_code_blocks module
        // when the editor content contains code, using the appropriate tmTheme file
        
        // Note: We no longer set GTK SourceView style schemes as fallbacks
        // All syntax highlighting is now handled consistently through colorize_code_blocks
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
        println!(
            "DEBUG: Editor set_css_theme called with theme: {}",
            theme_name
        );
        if let Some(ref theme_manager) = *self.theme_manager.borrow() {
            println!("DEBUG: Theme manager is available, setting CSS theme");
            match theme_manager.set_css_theme(theme_name) {
                Ok(css_content) => {
                    println!("DEBUG: CSS theme loaded successfully, applying to HTML view");
                    self.html_view.set_custom_css(&css_content);
                    self.refresh_html_view();
                    println!("DEBUG: CSS theme applied and view refreshed");
                }
                Err(e) => {
                    eprintln!("Failed to set CSS theme: {}", e);
                }
            }
        } else {
            eprintln!("Theme manager not initialized");
        }
    }

    /// Get the current CSS theme name
    pub fn get_current_css_theme(&self) -> String {
        if let Some(ref theme_manager) = *self.theme_manager.borrow() {
            theme_manager.get_current_css_theme()
        } else {
            "standard".to_string()
        }
    }

    /// Get available CSS themes by scanning the themes/ directory
    pub fn get_available_css_themes() -> Vec<(String, String, String)> {
        crate::theme::ThemeManager::get_available_css_themes()
    }
}
