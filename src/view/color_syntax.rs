use syntect::parsing::SyntaxSet;
use syntect::html::highlighted_html_for_string;
/// Highlight code as HTML using Syntect and the current theme
pub fn highlight_code_html(code: &str, language: &str) -> Option<String> {
    let theme_set = load_custom_themes();
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_name = if theme_set.themes.contains_key(language) {
        language
    } else {
        // fallback to dark or light
        if theme_set.themes.contains_key("dark") {
            "dark"
        } else if theme_set.themes.contains_key("light") {
            "light"
        } else {
            return None;
        }
    };
    let theme = theme_set.themes.get(theme_name)?;
    let syntax = syntax_set.find_syntax_by_token(language).unwrap_or_else(|| syntax_set.find_syntax_plain_text());
    highlighted_html_for_string(code, &syntax_set, syntax, theme).ok()
}

use syntect::highlighting::{ThemeSet,};
use syntect::html::{css_for_theme_with_class_style, ClassStyle};

/// Load custom themes from the ui/ui_theme directory, matching the highlighter logic
fn load_custom_themes() -> ThemeSet {
    use syntect::highlighting::ThemeSet;
    use crate::utils::cross_platform_resource::resolve_resource_path;
    let mut theme_set = ThemeSet::load_defaults();
    let themes_dir = resolve_resource_path("ui/ui_theme", "");
    if let Ok(entries) = std::fs::read_dir(&themes_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "tmTheme" {
                    if let Ok(theme) = ThemeSet::get_theme(&path) {
                        if let Some(file_stem) = path.file_stem().and_then(|n| n.to_str()) {
                            theme_set.themes.insert(file_stem.to_string(), theme.clone());
                        }
                    }
                }
            }
        }
    }
    theme_set
}

/// Generate Syntect CSS for a given theme name (e.g., "MarcoDark", "MarcoLight", or any in your theme set)
pub fn generate_syntect_css(theme_name: &str) -> Option<String> {
    let theme_set = load_custom_themes();
    let theme = theme_set.themes.get(theme_name)?;
    css_for_theme_with_class_style(theme, ClassStyle::Spaced).ok()
}
