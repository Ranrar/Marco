use gtk4::CssProvider;
use std::cell::RefCell;
use std::rc::Rc;

/// Used for color code syntax in light and dark mode ui_theme in editor and html
pub struct UiThemeProvider {
    gtk_css_provider: Rc<RefCell<Option<CssProvider>>>,
}

impl UiThemeProvider {
    /// Reload the GTK CSS provider with new CSS content and theme
    pub fn reload_gtk_css(&self, css_content: &str, theme: crate::theme::Theme) {
        if let Some(ref provider) = *self.gtk_css_provider.borrow() {
            let ui_theme = super::ui_theme::UiTheme::from_css(css_content, theme);
            let gtk_css = ui_theme.to_gtk_css();
            provider.load_from_data(&gtk_css);
            eprintln!("DEBUG: Reloaded GTK-specific CSS into provider");
        } else {
            eprintln!("WARNING: GTK CSS provider not initialized");
        }
    }
    pub fn new() -> Self {
        Self {
            gtk_css_provider: Rc::new(RefCell::new(None)),
        }
    }

    /// Initialize the GTK CSS provider for editor styling
    pub fn initialize_gtk_css_provider(&self) -> Result<(), String> {
        if let Some(display) = gtk4::gdk::Display::default() {
            let provider = CssProvider::new();
            *self.gtk_css_provider.borrow_mut() = Some(provider.clone());
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
            Ok(())
        } else {
            Err("Could not get default display for CSS provider".to_string())
        }
    }

    /// Generate GTK-specific CSS from theme colors and load into provider
    pub fn load_theme_css_into_gtk(&self, css_content: &str, theme: crate::theme::Theme) {
        if let Some(ref provider) = *self.gtk_css_provider.borrow() {
            let ui_theme = super::ui_theme::UiTheme::from_css(css_content, theme);
            let gtk_css = ui_theme.to_gtk_css();
            provider.load_from_data(&gtk_css);
            eprintln!("DEBUG: Loaded GTK-specific CSS into provider");
        } else {
            eprintln!("WARNING: GTK CSS provider not initialized");
        }
    }
}

/// UI/GTK/editor theming logic for Marco
/// This module will handle all theming for GTK widgets and the code editor (SourceView).
/// It will NOT handle HTML/markdown view theming.
pub struct UiTheme {
    pub bg_color: String,
    pub text_color: String,
    // Add more editor/GTK-specific theme fields as needed
}

impl UiTheme {
    /// Get the editor background color from CSS content and theme
    pub fn get_editor_background_color(css_content: &str, theme: crate::theme::Theme) -> String {
        let (bg_color, _) = Self::extract_theme_colors(css_content, theme);
        bg_color
    }
    /// Generate GTK CSS for the editor/SourceView from the theme colors
    pub fn to_gtk_css(&self) -> String {
        format!(
            r#"
/* GTK CSS for Marco Editor - Generated from UiTheme */

/* Base SourceView styling */
textview {{
    background-color: {bg_color};
    color: {text_color};
}}

textview text {{
    background-color: {bg_color};
    color: {text_color};
}}

/* Theme-specific classes for SourceView */
.theme-light textview {{
    background-color: {bg_color};
    color: {text_color};
}}

.theme-dark textview {{
    background-color: {bg_color};
    color: {text_color};
}}

/* SourceView specific elements */
textview.sourceview {{
    background-color: {bg_color};
    color: {text_color};
}}

/* Line numbers */
textview gutter {{
    background-color: {bg_color};
    color: {text_color};
}}

/* Current line highlighting */
textview text:selected {{
    background-color: alpha({text_color}, 0.1);
}}
"#,
            bg_color = self.bg_color,
            text_color = self.text_color
        )
    }
    /// Construct a UiTheme from CSS content and a Theme
    pub fn from_css(css_content: &str, theme: crate::theme::Theme) -> Self {
        let (bg_color, text_color) = Self::extract_theme_colors(css_content, theme);
        Self {
            bg_color,
            text_color,
        }
    }
    pub fn new(bg_color: &str, text_color: &str) -> Self {
        Self {
            bg_color: bg_color.to_string(),
            text_color: text_color.to_string(),
        }
    }

    /// Extract background and text colors from CSS content for the editor/GTK
    pub fn extract_theme_colors(css_content: &str, theme: crate::theme::Theme) -> (String, String) {
        let (theme_class, default_bg, default_text) = match theme {
            crate::theme::Theme::Light => (".theme-light", "#ffffff", "#000000"),
            crate::theme::Theme::Dark => (".theme-dark", "#1e1e1e", "#ffffff"),
            crate::theme::Theme::System => {
                match crate::theme::ThemeManager::detect_system_theme() {
                    crate::theme::Theme::Light => (".theme-light", "#ffffff", "#000000"),
                    crate::theme::Theme::Dark => (".theme-dark", "#1e1e1e", "#ffffff"),
                    crate::theme::Theme::System => (".theme-light", "#ffffff", "#000000"), // fallback
                }
            }
        };

        // Look for CSS variables in the theme section
        let bg_color = UiTheme::extract_css_variable(css_content, theme_class, "--bg-color")
            .unwrap_or_else(|| default_bg.to_string());
        let text_color = UiTheme::extract_css_variable(css_content, theme_class, "--text-color")
            .unwrap_or_else(|| default_text.to_string());

        (bg_color, text_color)
    }

    /// Extract a CSS variable value from a specific theme section
    pub fn extract_css_variable(css_content: &str, theme_class: &str, var_name: &str) -> Option<String> {
        // Find the theme class section
        let theme_start = css_content.find(&format!("{} {{", theme_class))?;
        let theme_section = &css_content[theme_start..];
        
        // Find the closing brace for this section
        let mut brace_count = 0;
        let mut theme_end = 0;
        for (i, c) in theme_section.char_indices() {
            match c {
                '{' => brace_count += 1,
                '}' => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        theme_end = i;
                        break;
                    }
                }
                _ => {}
            }
        }
        
        let theme_content = &theme_section[..theme_end];
        
        // Look for the variable
        for line in theme_content.lines() {
            let line = line.trim();
            if line.starts_with(var_name) {
                if let Some(colon_pos) = line.find(':') {
                    let value = line[colon_pos + 1..].trim();
                    let value = value.trim_end_matches(';').trim();
                    // Remove any comments
                    if let Some(comment_pos) = value.find("/*") {
                        return Some(value[..comment_pos].trim().to_string());
                    }
                    return Some(value.to_string());
                }
            }
        }
        
        None
    }
}
