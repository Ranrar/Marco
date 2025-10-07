// Theme CSS loading and management
//
//! # Theme Module
//!
//! Loads CSS themes for markdown preview and generates syntax highlighting.
//!
//! ## Functions
//!
//! ### `load_theme_css`
//!
//! Loads a CSS theme file from the assets directory:
//!
//! ```rust,ignore
//! let css = load_theme_css("github.css");
//! // Returns contents of assets/themes/html_viever/github.css
//! ```
//!
//! **Fallback**: If theme file is not found, returns minimal embedded CSS.
//!
//! ### `generate_syntax_highlighting_css`
//!
//! Generates CSS for syntax-highlighted code blocks using marco_core's
//! global syntax highlighter:
//!
//! ```rust,ignore
//! let css = generate_syntax_highlighting_css("dark");
//! // Returns CSS with .syntect-* classes for code tokens
//! ```
//!
//! **Theme Modes**: Accepts "light" or "dark" to match overall theme.
//!
//! ## Theme Files
//!
//! Themes are stored in `assets/themes/html_viever/`:
//! - `github.css` - GitHub-like styling
//! - `marco.css` - Marco's custom theme
//! - `academic.css` - Academic paper styling
//! - etc.
//!
//! ## Integration
//!
//! Both functions are used by the rendering module to combine:
//! 1. Theme CSS (document styling)
//! 2. Syntax highlighting CSS (code block styling)
//! 3. Generated HTML content

use marco_core::logic::paths::get_asset_dir_checked;
use std::path::PathBuf;

/// Load CSS content for a theme
pub fn load_theme_css(theme: &str) -> String {
    let asset_dir = get_asset_dir_checked().unwrap_or_else(|_| PathBuf::from("assets"));
    let theme_path = asset_dir.join(format!("themes/html_viever/{}", theme));
    
    match std::fs::read_to_string(&theme_path) {
        Ok(css) => css,
        Err(_) => {
            // Fallback to minimal styling if theme not found
            String::from(r#"
                body {
                    font-family: system-ui, -apple-system, sans-serif;
                    line-height: 1.6;
                    max-width: 800px;
                    margin: 2rem auto;
                    padding: 0 1rem;
                    background: #1e1e1e;
                    color: #cccccc;
                }
                h1, h2, h3, h4, h5, h6 {
                    margin-top: 1.5em;
                    margin-bottom: 0.5em;
                }
                code {
                    background: #2d2d2d;
                    padding: 0.2rem 0.4rem;
                    border-radius: 3px;
                    font-family: 'Courier New', monospace;
                }
                pre {
                    background: #2d2d2d;
                    padding: 1rem;
                    border-radius: 4px;
                    overflow-x: auto;
                }
            "#)
        }
    }
}

/// Generate CSS for syntax highlighting based on current theme mode
pub fn generate_syntax_highlighting_css(theme_mode: &str) -> String {
    use marco_core::components::syntax_highlighter::{global_syntax_highlighter, generate_css_with_global};
    
    // Initialize global highlighter if needed
    if let Err(e) = global_syntax_highlighter() {
        log::warn!("Failed to initialize syntax highlighter for CSS generation: {}", e);
        return String::new();
    }
    
    // Generate CSS for the current theme mode
    match generate_css_with_global(theme_mode) {
        Ok(css) => {
            log::debug!("Generated syntax highlighting CSS for theme: {}", theme_mode);
            css
        }
        Err(e) => {
            log::warn!("Failed to generate syntax highlighting CSS: {}", e);
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_load_theme_css_marco() {
        let css = load_theme_css("marco.css");
        assert!(!css.is_empty(), "Marco theme CSS should not be empty");
        // Should contain basic CSS rules
        assert!(
            css.contains("body") || css.contains("html") || css.contains("font-family"),
            "CSS should contain at least one basic HTML element"
        );
    }

    #[test]
    fn smoke_test_load_theme_css_fallback() {
        // Test fallback when theme doesn't exist
        let css = load_theme_css("nonexistent_theme_12345.css");
        assert!(!css.is_empty(), "Should return fallback CSS");
        assert!(css.contains("body"), "Fallback CSS should contain body rules");
        assert!(css.contains("font-family"), "Fallback CSS should contain font-family");
    }

    #[test]
    fn smoke_test_generate_syntax_highlighting_css_light() {
        let css = generate_syntax_highlighting_css("light");
        // Note: This might be empty if highlighter fails to initialize
        // We just verify it doesn't panic and returns a valid string
        let _ = css; // Verify function doesn't panic
    }

    #[test]
    fn smoke_test_generate_syntax_highlighting_css_dark() {
        let css = generate_syntax_highlighting_css("dark");
        // Note: This might be empty if highlighter fails to initialize
        // We just verify it doesn't panic and returns a valid string
        let _ = css; // Verify function doesn't panic
    }
}
