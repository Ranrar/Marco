//! syntect-powered syntax highlighting helpers.
//!
//! The core renderer can optionally emit classed HTML spans for fenced code
//! blocks. CSS for those classes can be generated via `syntect_css_for_theme_mode`.

use crate::render::code_languages::canonical_language_name;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::{css_for_theme_with_class_style, ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();
static CSS_CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn theme_set() -> &'static ThemeSet {
    THEME_SET.get_or_init(ThemeSet::load_defaults)
}

fn select_theme(theme_mode: &str) -> Theme {
    // Normalize theme mode to a small set. We accept strings like "dark", "Dark", "marco-dark".
    let is_dark = theme_mode.to_ascii_lowercase().contains("dark");

    let ts = theme_set();

    // Conservative defaults with fallbacks.
    let preferred = if is_dark { "Monokai" } else { "InspiredGitHub" };
    let fallback_1 = if is_dark {
        "Solarized (dark)"
    } else {
        "Solarized (light)"
    };

    ts.themes
        .get(preferred)
        .or_else(|| ts.themes.get(fallback_1))
        .or_else(|| ts.themes.values().next())
        .cloned()
        .unwrap_or_default()
}

/// Generate syntect CSS for a theme mode.
///
/// The returned CSS is compatible with `ClassStyle::Spaced` output.
pub fn syntect_css_for_theme_mode(theme_mode: &str) -> String {
    let cache = CSS_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    // Use a small normalized cache key.
    let key = if theme_mode.to_ascii_lowercase().contains("dark") {
        "dark"
    } else {
        "light"
    }
    .to_string();

    if let Ok(guard) = cache.lock() {
        if let Some(css) = guard.get(&key) {
            return css.clone();
        }
    }

    let theme = select_theme(theme_mode);
    let css = css_for_theme_with_class_style(&theme, ClassStyle::Spaced)
        .unwrap_or_else(|_| "/* syntect CSS generation failed */".to_string());

    if let Ok(mut guard) = cache.lock() {
        // Only keep a small number of entries (we only expect light/dark).
        if guard.len() < 10 {
            guard.insert(key, css.clone());
        }
    }

    css
}

fn find_syntax_for_language(language: &str) -> &'static SyntaxReference {
    let ss = syntax_set();

    // Prefer the canonical token for common aliases.
    if let Some(canonical) = canonical_language_name(language) {
        if let Some(s) = ss.find_syntax_by_token(canonical) {
            return s;
        }
    }

    // Direct token lookup.
    if let Some(s) = ss.find_syntax_by_token(language) {
        return s;
    }

    // Commonly, fenced info strings look like extensions.
    let ext = language.trim().trim_start_matches('.');
    if !ext.is_empty() {
        if let Some(s) = ss.find_syntax_by_extension(ext) {
            return s;
        }
    }

    ss.find_syntax_plain_text()
}

/// Highlight `code` (multiline) to classed HTML spans.
///
/// Returns `None` when the language cannot be resolved (plain text fallback)
/// or when highlighting fails.
pub fn highlight_code_to_classed_html(code: &str, language: &str) -> Option<String> {
    let language = language.trim();
    if code.is_empty() || language.is_empty() {
        return None;
    }

    let syntax = find_syntax_for_language(language);
    if syntax.name.eq_ignore_ascii_case("plain text") {
        return None;
    }

    let ss = syntax_set();
    let mut generator = ClassedHTMLGenerator::new_with_class_style(syntax, ss, ClassStyle::Spaced);

    for line in LinesWithEndings::from(code) {
        if generator
            .parse_html_for_line_which_includes_newline(line)
            .is_err()
        {
            return None;
        }
    }

    Some(generator.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_css_generation() {
        let css = syntect_css_for_theme_mode("light");
        assert!(css.contains("theme"));
    }

    #[test]
    fn smoke_test_highlight_rust() {
        let html = highlight_code_to_classed_html("let x = 42;\n", "rust");
        assert!(html.is_some());
        let html = html.unwrap();
        assert!(html.contains("span"));
    }
}
