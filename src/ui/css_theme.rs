use std::collections::HashMap;
use std::cell::RefCell;

thread_local! {
    static CSS_CACHE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    static CURRENT_CSS_THEME: RefCell<String> = RefCell::new("standard".to_string());
}

impl CssTheme {
    /// Set the CSS theme for the preview and cache it
    pub fn set_css_theme(theme_name: &str) -> Result<String, String> {
        CURRENT_CSS_THEME.with(|current| {
            *current.borrow_mut() = theme_name.to_string();
        });

        // Try cache first
        let cached = CSS_CACHE.with(|cache| cache.borrow().get(theme_name).cloned());
        if let Some(css_content) = cached {
            if !css_content.is_empty() {
                return Ok(css_content);
            }
        }

        let css_path = resolve_resource_path("ui/css_theme", &format!("{}.css", theme_name));
        match fs::read_to_string(&css_path) {
            Ok(css_content) => {
                CSS_CACHE.with(|cache| cache.borrow_mut().insert(theme_name.to_string(), css_content.clone()));
                Ok(css_content)
            },
            Err(e) => {
                eprintln!("Failed to load CSS theme '{}': {}", theme_name, e);
                // Fallback to standard theme if not already tried
                if theme_name != "standard" {
                    CssTheme::invalidate_css_cache(Some(theme_name));
                    return CssTheme::set_css_theme("standard");
                }
                Err(format!("Failed to load CSS theme '{}': {}", theme_name, e))
            }
        }
    }

    /// Invalidate the CSS cache for a specific theme (or all)
    pub fn invalidate_css_cache(theme_name: Option<&str>) {
        CSS_CACHE.with(|cache| {
            if let Some(name) = theme_name {
                cache.borrow_mut().remove(name);
            } else {
                cache.borrow_mut().clear();
            }
        });
    }

    /// Get the current CSS theme name
    pub fn get_current_css_theme() -> String {
        CURRENT_CSS_THEME.with(|current| current.borrow().clone())
    }

    /// Get available CSS themes by scanning the ui/css_theme/ directory
    pub fn get_available_css_themes() -> Vec<(String, String, String)> {
        let mut themes = Vec::new();
        let themes_dir = resolve_resource_path("ui/css_theme", "");
        if let Ok(entries) = fs::read_dir(&themes_dir) {
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

    /// Generate CSS that forces the theme regardless of system preference
    pub fn get_theme_override_css(theme: crate::theme::Theme) -> String {
        match theme {
            crate::theme::Theme::Dark => {
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
            crate::theme::Theme::Light => {
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
            crate::theme::Theme::System => {
                // No override, let system preference decide
                String::new()
            }
        }
    }
}

/// HTML/Markdown view theming logic for Marco
/// This module handles all theming for the HTML/markdown preview view.
/// It will NOT handle GTK/editor theming.
use std::fs;
use crate::utils::cross_platform_resource::resolve_resource_path;

pub struct CssTheme {
    pub css_content: String,
    pub theme_name: String,
}

impl CssTheme {
    /// Load a CSS theme for the HTML/markdown view by name
    pub fn load(theme_name: &str) -> Result<Self, String> {
        let css_path = resolve_resource_path("ui/css_theme", &format!("{}.css", theme_name));
        match fs::read_to_string(&css_path) {
            Ok(css_content) => Ok(Self {
                css_content,
                theme_name: theme_name.to_string(),
            }),
            Err(e) => Err(format!("Failed to load CSS theme '{}': {}", theme_name, e)),
        }
    }

    /// Get the raw CSS content for the HTML/markdown view
    pub fn as_str(&self) -> &str {
        &self.css_content
    }
}
