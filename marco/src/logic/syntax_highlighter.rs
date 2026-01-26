//! Simple Syntax Highlighter for Marco
//!
//! Provides syntax highlighting using syntect with CSS class output.
//! Designed to be simple: one light theme, one dark theme, robust fallbacks.
//! Integrates with Marco's existing async infrastructure using GLib.

// ============================================================================
// THEME CONFIGURATION - Easy place to edit light and dark themes
// ============================================================================
//
// Available syntect-assets themes:
// Light themes: "InspiredGitHub", "Solarized (light)", "Sourcegraph (light)", "base16-ocean.light"
// Dark themes:  "Solarized (dark)", "Monokai", "Visual Studio Dark", "base16-ocean.dark",
//               "base16-eighties.dark", "base16-mocha.dark"
//
// Simply change the theme names below to try different color schemes:

/// Light theme for syntax highlighting (used when Marco is in light mode)
const LIGHT_THEME_NAME: &str = "Solarized (light)";

/// Dark theme for syntax highlighting (used when Marco is in dark mode)  
const DARK_THEME_NAME: &str = "Monokai";

// ============================================================================

use std::collections::HashMap;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::{css_for_theme_with_class_style, ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::{SyntaxReference, SyntaxSet};

/// Simple syntax highlighter with minimal themes and robust fallbacks
pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    light_theme: Theme,
    dark_theme: Theme,
    css_cache: HashMap<String, String>, // theme_mode -> CSS
}

impl SyntaxHighlighter {
    /// Create a new syntax highlighter with configurable themes
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        log::debug!("[syntax_highlighter] Initializing syntax highlighter");

        // Load syntax sets using syntect-assets for better performance
        let syntax_set = SyntaxSet::load_defaults_newlines();

        // Load theme set and select themes based on our configuration constants
        let theme_set = ThemeSet::load_defaults();

        // Load light theme using the configurable constant
        let light_theme = theme_set
            .themes
            .get(LIGHT_THEME_NAME)
            .or_else(|| theme_set.themes.get("InspiredGitHub")) // fallback
            .or_else(|| theme_set.themes.get("Solarized (light)")) // another fallback
            .or_else(|| theme_set.themes.values().next()) // final fallback to any theme
            .ok_or_else(|| -> Box<dyn std::error::Error> {
                format!(
                    "Light theme '{}' not found and no fallbacks available",
                    LIGHT_THEME_NAME
                ).into()
            })?
            .clone();

        // Load dark theme using the configurable constant
        let dark_theme = theme_set
            .themes
            .get(DARK_THEME_NAME)
            .or_else(|| theme_set.themes.get("Solarized (dark)")) // fallback
            .or_else(|| theme_set.themes.get("base16-ocean.dark")) // another fallback
            .or_else(|| theme_set.themes.values().next()) // final fallback to any theme
            .ok_or_else(|| -> Box<dyn std::error::Error> {
                format!(
                    "Dark theme '{}' not found and no fallbacks available",
                    DARK_THEME_NAME
                ).into()
            })?
            .clone();

        log::debug!(
            "[syntax_highlighter] Loaded themes: light='{}' ({}), dark='{}' ({})",
            LIGHT_THEME_NAME,
            light_theme.name.as_ref().unwrap_or(&"unknown".to_string()),
            DARK_THEME_NAME,
            dark_theme.name.as_ref().unwrap_or(&"unknown".to_string())
        );

        Ok(Self {
            syntax_set,
            light_theme,
            dark_theme,
            css_cache: HashMap::new(),
        })
    }

    /// Highlight code synchronously with CSS classes
    /// Returns highlighted HTML with CSS class spans
    pub fn highlight_to_html(
        &self,
        code: &str,
        language: &str,
        _theme_mode: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Find the syntax definition for the language
        let syntax = self.find_syntax_for_language(language);

        // Generate highlighted HTML with CSS classes
        let mut html_generator = ClassedHTMLGenerator::new_with_class_style(
            syntax,
            &self.syntax_set,
            ClassStyle::Spaced,
        );

        // Process the entire code block efficiently
        // Split into lines and process each with newline
        if code.is_empty() {
            return Ok(String::new());
        }

        for line in code.lines() {
            // Use a single allocation for the line with newline
            let mut line_with_newline = String::with_capacity(line.len() + 1);
            line_with_newline.push_str(line);
            line_with_newline.push('\n');
            html_generator.parse_html_for_line_which_includes_newline(&line_with_newline)?;
        }

        Ok(html_generator.finalize())
    }

    /// Generate CSS for syntax highlighting
    /// Returns CSS string with all the highlighting classes
    pub fn generate_css(&mut self, theme_mode: &str) -> String {
        // Check cache first
        if let Some(cached_css) = self.css_cache.get(theme_mode) {
            return cached_css.clone();
        }

        let theme = match theme_mode {
            "dark" => &self.dark_theme,
            _ => &self.light_theme,
        };

        // Generate CSS with spaced class style to match our HTML generator
        let css = css_for_theme_with_class_style(theme, ClassStyle::Spaced)
            .unwrap_or_else(|_| "/* CSS generation failed */".to_string());

        // Cache the result with bounds checking (prevent unlimited growth)
        // We expect only "light" and "dark" theme modes, but limit to 10 just in case
        if self.css_cache.len() < 10 {
            self.css_cache.insert(theme_mode.to_string(), css.clone());
        } else {
            log::warn!(
                "[syntax_highlighter] CSS cache limit reached, not caching theme_mode: {}",
                theme_mode
            );
        }

        log::debug!(
            "[syntax_highlighter] Generated CSS for theme_mode: {}",
            theme_mode
        );
        css
    }

    /// Find syntax definition for a language, with fallbacks
    /// Returns a syntax reference, defaulting to plain text if not found
    fn find_syntax_for_language(&self, language: &str) -> &SyntaxReference {
        // Try direct lookup first
        if let Some(syntax) = self.syntax_set.find_syntax_by_token(language) {
            return syntax;
        }

        // Try shared alias normalization from core.
        if let Some(canonical) = core::render::canonical_language_name(language) {
            if let Some(syntax) = self.syntax_set.find_syntax_by_token(canonical) {
                return syntax;
            }
        }

        // Try by extension
        let extension = language.trim().trim_start_matches('.');
        if let Some(syntax) = self.syntax_set.find_syntax_by_extension(extension) {
            return syntax;
        }

        // Fallback to plain text
        self.syntax_set.find_syntax_plain_text()
    }

    /// Check if a language is supported (has a non-plain-text syntax)
    #[cfg(test)]
    pub fn is_language_supported(&self, language: &str) -> bool {
        let syntax = self.find_syntax_for_language(language);
        !syntax.name.eq_ignore_ascii_case("plain text")
    }
}

thread_local! {
    pub static SYNTAX_HIGHLIGHTER: std::cell::RefCell<Option<SyntaxHighlighter>> =
        const { std::cell::RefCell::new(None) };
}

/// Get the global syntax highlighter instance, initializing if needed
pub fn global_syntax_highlighter() -> Result<(), Box<dyn std::error::Error>> {
    SYNTAX_HIGHLIGHTER.with(|highlighter| {
        let mut h = highlighter.borrow_mut();
        if h.is_none() {
            log::debug!("[syntax_highlighter] Initializing global syntax highlighter");
            *h = Some(SyntaxHighlighter::new()?);
        }
        Ok(())
    })
}

/// Generate CSS using the global syntax highlighter  
pub fn generate_css_with_global(theme_mode: &str) -> Result<String, Box<dyn std::error::Error>> {
    global_syntax_highlighter()?;

    SYNTAX_HIGHLIGHTER.with(|highlighter| {
        let mut h = highlighter.borrow_mut();
        let syntax_highlighter = h
            .as_mut()
            .ok_or_else(|| "Syntax highlighter not initialized")?;

        Ok(syntax_highlighter.generate_css(theme_mode))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn smoke_test_syntax_highlighter_creation() {
        let result = SyntaxHighlighter::new();
        assert!(
            result.is_ok(),
            "Should be able to create syntax highlighter"
        );

        let highlighter = result.unwrap();
        assert!(!highlighter.syntax_set.syntaxes().is_empty());
        assert!(highlighter.light_theme.name.is_some());
        assert!(highlighter.dark_theme.name.is_some());
    }

    #[test]
    fn smoke_test_language_detection() {
        let highlighter = SyntaxHighlighter::new().expect("Failed to create highlighter");

        // Test common languages
        assert!(highlighter.is_language_supported("rust"));
        assert!(highlighter.is_language_supported("javascript"));
        assert!(highlighter.is_language_supported("python"));

        // Test aliases
        assert!(highlighter.is_language_supported("js"));
        assert!(highlighter.is_language_supported("py"));
        assert!(highlighter.is_language_supported("rs"));

        // Test unknown language falls back gracefully
        assert!(!highlighter.is_language_supported("unknownlanguage123"));
    }

    #[test]
    fn smoke_test_highlighting() {
        let highlighter = SyntaxHighlighter::new().expect("Failed to create highlighter");

        let code = r#"fn main() {
    println!("Hello, world!");
}"#;

        let result = highlighter.highlight_to_html(code, "rust", "light");
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(html.contains("class=")); // Should contain CSS classes
        assert!(html.contains("main")); // Should contain the code content
    }

    #[test]
    fn smoke_test_css_generation() {
        let mut highlighter = SyntaxHighlighter::new().expect("Failed to create highlighter");

        let light_css = highlighter.generate_css("light");
        let dark_css = highlighter.generate_css("dark");

        assert!(!light_css.is_empty());
        assert!(!dark_css.is_empty());
        assert_ne!(light_css, dark_css); // Should be different

        // Test caching
        let cached_light_css = highlighter.generate_css("light");
        assert_eq!(light_css, cached_light_css);
    }

    #[test]
    fn smoke_test_fallback_behavior() {
        let highlighter = SyntaxHighlighter::new().expect("Failed to create highlighter");

        // Test unknown language
        let result = highlighter.highlight_to_html("some code", "unknownlang", "light");
        assert!(result.is_ok()); // Should not fail

        let html = result.unwrap();
        assert!(html.contains("some code")); // Should contain the original code
    }

    #[test]
    #[serial(syntax_highlighter)]
    fn smoke_test_global_highlighter() {
        let result = global_syntax_highlighter();
        assert!(result.is_ok());

        // Test CSS generation
        let css_result = generate_css_with_global("light");
        assert!(css_result.is_ok());
        assert!(!css_result.unwrap().is_empty());
    }
}
