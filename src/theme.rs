use gtk4::prelude::*;
use gtk4::CssProvider;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Theme {
    /// Convert theme to string representation for settings/serialization
    ///
    /// Used for converting theme enum values to strings for storage in
    /// configuration files and settings persistence.
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::System => "system",
        }
    }

    /// Convert string to Theme enum for loading from settings
    ///
    /// Used for parsing theme strings from configuration files back
    /// into Theme enum values. Defaults to System theme for unknown values.
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Theme {
        match s {
            "dark" => Theme::Dark,
            "light" => Theme::Light,
            _ => Theme::System,
        }
    }
}

pub struct ThemeManager {
    current_theme: Rc<RefCell<Theme>>,
    callbacks: Rc<RefCell<Vec<Box<dyn Fn(Theme)>>>>,
    current_css_theme: Rc<RefCell<String>>,
    gtk_css_provider: Rc<RefCell<Option<CssProvider>>>,
}

impl ThemeManager {
    pub fn new() -> Self {
        let detected_theme = Self::detect_system_theme();
        Self {
            current_theme: Rc::new(RefCell::new(detected_theme)),
            callbacks: Rc::new(RefCell::new(Vec::new())),
            current_css_theme: Rc::new(RefCell::new("standard".to_string())),
            gtk_css_provider: Rc::new(RefCell::new(None)),
        }
    }

    /// Detect the system theme preference
    pub fn detect_system_theme() -> Theme {
        // Try to detect system theme using GTK settings
        if let Some(settings) = gtk4::Settings::default() {
            // Use property instead of boolean method
            let prefer_dark: bool = settings.property("gtk-application-prefer-dark-theme");
            if prefer_dark {
                return Theme::Dark;
            }
        }

        // Fallback: try environment variables (works on Linux)
        if let Ok(theme) = std::env::var("GTK_THEME") {
            if theme.to_lowercase().contains("dark") {
                return Theme::Dark;
            }
        }

        // Check for GNOME dark theme preference
        if let Ok(output) = std::process::Command::new("gsettings")
            .args(&["get", "org.gnome.desktop.interface", "gtk-theme"])
            .output()
        {
            if let Ok(theme_name) = String::from_utf8(output.stdout) {
                if theme_name.to_lowercase().contains("dark") {
                    return Theme::Dark;
                }
            }
        }

        // Default to light theme
        Theme::Light
    }

    pub fn get_current_theme(&self) -> Theme {
        *self.current_theme.borrow()
    }

    pub fn set_theme(&self, theme: Theme) {
        let actual_theme = match theme {
            Theme::System => Self::detect_system_theme(),
            other => other,
        };

        *self.current_theme.borrow_mut() = theme;

        // Notify all callbacks
        for callback in self.callbacks.borrow().iter() {
            callback(actual_theme);
        }
    }

    pub fn get_effective_theme(&self) -> Theme {
        match self.get_current_theme() {
            Theme::System => Self::detect_system_theme(),
            other => other,
        }
    }

    /// Register a callback for theme change notifications
    ///
    /// Allows components to register functions that will be called whenever
    /// the theme changes. Useful for updating UI elements that need to
    /// respond to theme switches.
    ///
    /// # Arguments
    /// * `callback` - Function to call when theme changes, receives the new effective theme
    #[allow(dead_code)]
    pub fn add_theme_change_callback<F>(&self, callback: F)
    where
        F: Fn(Theme) + 'static,
    {
        self.callbacks.borrow_mut().push(Box::new(callback));
    }

    /// Get CSS class name for the current effective theme
    ///
    /// Returns appropriate CSS class names that can be applied to UI elements
    /// for consistent theming. Useful for components that need to apply
    /// theme-specific styling beyond the automatic theme detection.
    ///
    /// # Returns
    /// * "dark-theme" for dark mode
    /// * "light-theme" for light mode or system fallback
    #[allow(dead_code)]
    pub fn get_css_class(&self) -> &'static str {
        match self.get_effective_theme() {
            Theme::Dark => "dark-theme",
            Theme::Light => "light-theme",
            Theme::System => "light-theme", // fallback
        }
    }

    /// Generate CSS that forces the theme regardless of system preference
    pub fn get_theme_override_css(&self) -> String {
        match self.get_effective_theme() {
            Theme::Dark => {
                // Force dark mode by overriding CSS custom properties
                r#"
/* Force dark theme override */
:root {
    color-scheme: dark !important;
}

/* Override all CSS custom properties to force dark mode */
:root, 
:root:not([data-theme]),
:root[data-theme="auto"] {
    --bg-color: #1a1a1a !important;
    --text-color: #e0e0e0 !important;
    --heading-color: #f0f0f0 !important;
    --quote-color: #a0a0a0 !important;
    --quote-border: #666 !important;
    --code-bg: #2d2d2d !important;
    --pre-bg: #222 !important;
    --pre-border: #444 !important;
    --table-border: #555 !important;
    --table-header-bg: #333 !important;
    --hr-color: #666 !important;
    --link-color: #66b3ff !important;
    --link-hover: #99ccff !important;
    --admonition-bg: #2a2a2a !important;
    --admonition-border: #666 !important;
    
    /* Academic theme specific overrides */
    --text-secondary: #8b949e !important;
    --text-muted: #6e7681 !important;
    --bg-secondary: #161b22 !important;
    --bg-code: #161b22 !important;
    --bg-pre: #161b22 !important;
    --border-color: #30363d !important;
    --border-light: #21262d !important;
    --border-strong: #6e7681 !important;
    --blockquote-border: #30363d !important;
    --blockquote-text: #8b949e !important;
    --blockquote-bg: rgba(110, 118, 129, 0.1) !important;
    --table-stripe-bg: rgba(110, 118, 129, 0.1) !important;
    --mark-bg: #ffd33d !important;
    --mark-color: #24292f !important;
    --strong-color: #f0f6fc !important;
    --heading-accent: #555 !important;
    --h2-border: #555 !important;
    --h6-color: #aaa !important;
    --img-border: #555 !important;
    --caption-color: #aaa !important;
    --hr-bg: #1a1a1a !important;
    --footnote-color: #aaa !important;
    --footnote-border: #555 !important;
    --note-bg: #1a2a3a !important;
    --tip-bg: #1a3a2a !important;
    --important-bg: #3a2a1a !important;
    --warning-bg: #3a3a1a !important;
    --caution-bg: #3a1a1a !important;
}

/* Ensure all elements use dark colors */
body { 
    background-color: var(--bg-color) !important; 
    color: var(--text-color) !important; 
}
h1, h2, h3, h4, h5, h6 { 
    color: var(--heading-color) !important; 
}
"#
                .to_string()
            }
            Theme::Light => {
                // Force light mode by overriding CSS custom properties
                r#"
/* Force light theme override */
:root {
    color-scheme: light !important;
}

/* Override all CSS custom properties to force light mode */
:root,
:root:not([data-theme]),
:root[data-theme="auto"] {
    --bg-color: #ffffff !important;
    --text-color: #333333 !important;
    --heading-color: #222222 !important;
    --quote-color: #666666 !important;
    --quote-border: #cccccc !important;
    --code-bg: #f5f5f5 !important;
    --pre-bg: #f8f8f8 !important;
    --pre-border: #e5e5e5 !important;
    --table-border: #dddddd !important;
    --table-header-bg: #f9f9f9 !important;
    --hr-color: #cccccc !important;
    --link-color: #0066cc !important;
    --link-hover: #0550ae !important;
    --admonition-bg: #f9f9f9 !important;
    --admonition-border: #cccccc !important;
    
    /* Academic theme specific overrides */
    --text-secondary: #666 !important;
    --text-muted: #999 !important;
    --bg-secondary: #f6f8fa !important;
    --bg-code: #f6f8fa !important;
    --bg-pre: #f6f8fa !important;
    --border-color: #d0d7de !important;
    --border-light: #d1d9e0 !important;
    --border-strong: #8c959f !important;
    --blockquote-border: #d0d7de !important;
    --blockquote-text: #656d76 !important;
    --blockquote-bg: rgba(13, 17, 23, 0.05) !important;
    --table-stripe-bg: rgba(175, 184, 193, 0.2) !important;
    --mark-bg: #fff8c5 !important;
    --mark-color: #24292f !important;
    --strong-color: #1f2328 !important;
    --heading-accent: #34495e !important;
    --h2-border: #bdc3c7 !important;
    --h6-color: #7f8c8d !important;
    --img-border: #bdc3c7 !important;
    --caption-color: #7f8c8d !important;
    --hr-bg: #ffffff !important;
    --footnote-color: #7f8c8d !important;
    --footnote-border: #bdc3c7 !important;
    --note-bg: #ebf3fd !important;
    --tip-bg: #eafaf1 !important;
    --important-bg: #fef5e7 !important;
    --warning-bg: #fef9e7 !important;
    --caution-bg: #fdedec !important;
}

/* Ensure all elements use light colors */
body { 
    background-color: var(--bg-color) !important; 
    color: var(--text-color) !important; 
}
h1, h2, h3, h4, h5, h6 { 
    color: var(--heading-color) !important; 
}
"#
                .to_string()
            }
            Theme::System => {
                // No override, let system preference decide
                String::new()
            }
        }
    }

    /// Set the CSS theme for the preview
    pub fn set_css_theme(&self, theme_name: &str) -> Result<String, String> {
        *self.current_css_theme.borrow_mut() = theme_name.to_string();

        // Load CSS file from the themes/ directory
        let css_path = format!("themes/{}.css", theme_name);
        match std::fs::read_to_string(&css_path) {
            Ok(css_content) => {
                // Load GTK-specific CSS into the provider
                self.load_theme_css_into_gtk(&css_content);
                Ok(css_content)
            },
            Err(e) => {
                eprintln!("Failed to load CSS theme '{}': {}", theme_name, e);
                // Fallback to standard theme
                if theme_name != "standard" {
                    return self.set_css_theme("standard");
                }
                Err(format!("Failed to load CSS theme '{}': {}", theme_name, e))
            }
        }
    }

    /// Get the current CSS theme name
    pub fn get_current_css_theme(&self) -> String {
        self.current_css_theme.borrow().clone()
    }

    /// Get available CSS themes by scanning the themes/ directory
    pub fn get_available_css_themes() -> Vec<(String, String, String)> {
        let mut themes = Vec::new();

        if let Ok(entries) = std::fs::read_dir("themes") {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if let Some(extension) = path.extension() {
                        if extension == "css" {
                            if let Some(filename) = path.file_stem() {
                                let theme_id = filename.to_string_lossy().to_string();
                                let display_name = theme_id
                                    .replace('_', " ")
                                    .split(' ')
                                    .map(|word| {
                                        let mut chars = word.chars();
                                        match chars.next() {
                                            None => String::new(),
                                            Some(first) => {
                                                first.to_uppercase().collect::<String>()
                                                    + chars.as_str()
                                            }
                                        }
                                    })
                                    .collect::<Vec<String>>()
                                    .join(" ");

                                // Create a properly sanitized name for action IDs (no spaces, special chars)
                                let sanitized_name =
                                    theme_id.replace(|c: char| !c.is_alphanumeric(), "_");

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

    /// Initialize the GTK CSS provider for editor styling
    fn initialize_gtk_css_provider(&self) -> Result<(), String> {
        if let Some(display) = gtk4::gdk::Display::default() {
            let provider = CssProvider::new();
            *self.gtk_css_provider.borrow_mut() = Some(provider.clone());
            
            // Add the provider to the display with application priority
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
    fn load_theme_css_into_gtk(&self, css_content: &str) {
        if let Some(ref provider) = *self.gtk_css_provider.borrow() {
            // Generate GTK-specific CSS instead of using web CSS
            let gtk_css = self.generate_gtk_css_from_theme(css_content);
            provider.load_from_data(&gtk_css);
            eprintln!("DEBUG: Loaded GTK-specific CSS into provider");
        } else {
            eprintln!("WARNING: GTK CSS provider not initialized");
        }
    }

    /// Generate GTK-compatible CSS from theme variables
    fn generate_gtk_css_from_theme(&self, css_content: &str) -> String {
        let effective_theme = self.get_effective_theme();
        
        // Extract colors based on current theme
        let (bg_color, text_color) = self.extract_theme_colors(css_content, effective_theme);
        
        // Generate minimal GTK CSS that works with SourceView
        format!(
            r#"
/* GTK CSS for Marco Editor - Generated from {theme} theme */

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
            theme = self.get_current_css_theme(),
            bg_color = bg_color,
            text_color = text_color
        )
    }

    /// Extract background and text colors from CSS content
    fn extract_theme_colors(&self, css_content: &str, theme: Theme) -> (String, String) {
        let (theme_class, default_bg, default_text) = match theme {
            Theme::Light => (".theme-light", "#ffffff", "#000000"),
            Theme::Dark => (".theme-dark", "#1e1e1e", "#ffffff"),
            Theme::System => {
                match Self::detect_system_theme() {
                    Theme::Light => (".theme-light", "#ffffff", "#000000"),
                    Theme::Dark => (".theme-dark", "#1e1e1e", "#ffffff"),
                    Theme::System => (".theme-light", "#ffffff", "#000000"), // fallback
                }
            }
        };

        // Look for CSS variables in the theme section
        let bg_color = self.extract_css_variable(css_content, theme_class, "--bg-color")
            .unwrap_or_else(|| default_bg.to_string());
        let text_color = self.extract_css_variable(css_content, theme_class, "--text-color")
            .unwrap_or_else(|| default_text.to_string());

        (bg_color, text_color)
    }

    /// Extract a CSS variable value from a specific theme section
    fn extract_css_variable(&self, css_content: &str, theme_class: &str, var_name: &str) -> Option<String> {
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

    /// Initialize the theme manager with default CSS theme
    /// This should be called at startup to ensure the CSS is loaded
    pub fn initialize(&self) -> Result<(), String> {
        // Initialize the GTK CSS provider
        self.initialize_gtk_css_provider()?;
        
        // Load the default CSS theme
        self.set_css_theme("standard")?;
        Ok(())
    }

    /// Get the syntax theme name based on current theme
    pub fn get_syntax_theme_name(&self) -> String {
        match self.get_effective_theme() {
            Theme::Dark => "dark".to_string(),
            Theme::Light => "light".to_string(),
            Theme::System => {
                match Self::detect_system_theme() {
                    Theme::Dark => "dark".to_string(),
                    Theme::Light => "light".to_string(),
                    Theme::System => "light".to_string(), // fallback
                }
            }
        }
    }

    /// Get the editor background color from CSS content
    pub fn get_editor_background_color(&self, css_content: &str) -> String {
        let effective_theme = self.get_effective_theme();
        let (bg_color, _) = self.extract_theme_colors(css_content, effective_theme);
        bg_color
    }

    /// Create a weak reference to this theme manager
    pub fn downgrade(&self) -> std::rc::Weak<RefCell<ThemeManager>> {
        // This is a placeholder - the actual implementation would depend on how the theme manager is stored
        // For now, we'll use a different approach in the preferences module
        std::rc::Weak::new()
    }
}

impl Clone for ThemeManager {
    fn clone(&self) -> Self {
        Self {
            current_theme: self.current_theme.clone(),
            callbacks: self.callbacks.clone(), // Share the same callbacks vector
            current_css_theme: self.current_css_theme.clone(),
            gtk_css_provider: self.gtk_css_provider.clone(),
        }
    }
}
