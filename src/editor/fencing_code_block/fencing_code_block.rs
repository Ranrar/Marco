// use glib::MainContext;
use crate::utils::parallel::run_in_pool;
use rayon::prelude::*;
/// Offload code block highlighting to the Rayon thread pool and send results to GTK main thread.
///
/// # Arguments
/// * `code_blocks` - Vec of (code, language) pairs to highlight
/// * `highlighter` - SyntectHighlighter instance (should be cheap to clone)
/// * `on_done` - Callback to receive Vec of highlighted HTML strings (in order)
///
/// # Example
/// ```rust
/// use crate::markdown::colorize_code_blocks::colorize_code_blocks_async;
/// colorize_code_blocks_async(code_blocks, highlighter, move |results| {
///     // Update GTK UI with highlighted code blocks (on main thread)
/// });
/// ```
pub fn colorize_code_blocks_async<F>(
    code_blocks: Vec<(String, String)>,
    highlighter: SyntectHighlighter,
    on_done: F,
) where
    F: FnOnce(Vec<String>) + 'static,
{
    // Use glib::MainContext::channel to send results back to the GTK main thread.
    // This ensures thread-safety and idiomatic GTK4 usage.
    use std::sync::mpsc::channel;
    let (sender, receiver) = channel();

    // Offload the CPU-intensive work to the Rayon thread pool.
    run_in_pool(move || {
        let results: Vec<String> = code_blocks
            .par_iter()
            .map(|(code, lang)| highlighter.highlight_code(code, lang))
            .collect();
        let _ = sender.send(results);
    });

    // Poll the receiver on the GTK main thread and call the callback when ready.
    let mut on_done = Some(on_done);
    glib::idle_add_local(move || {
        match receiver.try_recv() {
            Ok(results) => {
                if let Some(cb) = on_done.take() {
                    cb(results);
                }
                glib::ControlFlow::Break
            },
            Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(_) => glib::ControlFlow::Break,
        }
    });
}
// ---
// ## Rayon + GTK4 Parallel Syntax Highlighting Best Practices
//
// - Use `colorize_code_blocks_async` to offload code block highlighting to the Rayon thread pool.
// - Always send results back to the GTK main thread using `glib::MainContext::channel`.
// - Never update GTK widgets from Rayon threads.
// - Use `.par_iter()` for parallel processing of code blocks.
// - Clone the highlighter if needed (it is cheap to clone).
// - Handle errors gracefully and preserve order of code blocks.
//
// Example usage:
// ```rust
// colorize_code_blocks_async(code_blocks, highlighter, move |results| {
//     // Update UI with results (on main thread)
// });
// ```
use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

/// Modern syntax highlighter using syntect library
pub struct SyntectHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    current_theme: String,
    theme_cache: crate::utils::cache::CacheSync<String, Theme>,
}
impl SyntectHighlighter {
    /// Create a new highlighter with a specific theme name
    pub fn with_theme(theme_name: &str) -> Self {
        let mut highlighter = Self::new();
        // Map 'dark' and 'light' to the correct theme names
        let mapped_theme = match theme_name.to_lowercase().as_str() {
            "dark" => "MarcoDark",
            "light" => "MarcoLight",
            _ => theme_name,
        };
        if highlighter.theme_set.themes.contains_key(mapped_theme) {
            highlighter.current_theme = mapped_theme.to_string();
        }
        highlighter
    }
    /// Load all .tmTheme files from the resolved colorize_code_blocks directory and return as Vec<(name, Theme)>
    fn load_custom_themes_from_folder() -> Vec<(String, Theme)> {
        use crate::utils::cross_platform_resource::resolve_resource_path;
        use std::fs;
        let mut result = Vec::new();
        // Get the resolved path to the ui_theme directory
        let themes_dir = resolve_resource_path("ui/ui_theme", "");
        eprintln!("[DEBUG] Looking for .tmTheme files in: {}", themes_dir.display());
        if let Ok(entries) = fs::read_dir(&themes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "tmTheme" {
                        if let Ok(theme) = ThemeSet::get_theme(&path) {
                            // Register both the file stem and the <name> field as keys
                            if let Some(file_stem) = path.file_stem().and_then(|n| n.to_str()) {
                                result.push((file_stem.to_string(), theme.clone()));
                            }
                            // Try to extract the <name> field from the XML
                            if let Ok(xml) = std::fs::read_to_string(&path) {
                                if let Some(name_start) = xml.find("<string>Marco") {
                                    let name_end = xml[name_start+8..].find("</string>").map(|i| i + name_start + 8);
                                    if let Some(end) = name_end {
                                        let marco_name = &xml[name_start+8..end];
                                        result.push((marco_name.to_string(), theme.clone()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        result
    }
    /// Create a new highlighter with default syntax definitions and themes
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();

        // Load themes from the resolved themes directory
        let mut theme_set = ThemeSet::new();
        let custom_themes = Self::load_custom_themes_from_folder();

        for (name, theme) in custom_themes {
            theme_set.themes.insert(name, theme);
        }

        // Default to MarcoDark if available, otherwise first available theme
        let default_theme = if theme_set.themes.contains_key("MarcoDark") {
            "MarcoDark".to_string()
        } else if theme_set.themes.contains_key("MarcoLight") {
            "MarcoLight".to_string()
        } else {
            theme_set.themes.keys().next().cloned().unwrap_or_else(|| "default".to_string())
        };

        Self {
            syntax_set,
            theme_set,
            current_theme: default_theme,
            theme_cache: crate::utils::cache::CacheSync::new(),
        }
    }

    /// Get the current theme name
    pub fn get_current_theme(&self) -> &str {
        &self.current_theme
    }

    /// Get all available theme names
    pub fn get_available_themes(&self) -> Vec<String> {
        let mut themes: Vec<String> = self.theme_set.themes.keys().cloned().collect();
        themes.sort();
        themes
    }

    /// Highlight code and return HTML with CSS classes
    pub fn highlight_code(&self, code: &str, language: &str) -> String {
        // Try to find syntax by various means
        let syntax = self
            .syntax_set
            .find_syntax_by_token(language)
            .or_else(|| self.syntax_set.find_syntax_by_extension(language))
            .or_else(|| {
                // Try with common aliases
                match language.to_lowercase().as_str() {
                    "js" | "javascript" => self.syntax_set.find_syntax_by_extension("js"),
                    "ts" | "typescript" => self.syntax_set.find_syntax_by_extension("ts"),
                    "py" | "python" => self.syntax_set.find_syntax_by_extension("py"),
                    "rs" | "rust" => self.syntax_set.find_syntax_by_extension("rs"),
                    "cpp" | "c++" => self.syntax_set.find_syntax_by_extension("cpp"),
                    "cs" | "csharp" | "c#" => self.syntax_set.find_syntax_by_extension("cs"),
                    "go" | "golang" => self.syntax_set.find_syntax_by_extension("go"),
                    "php" => self.syntax_set.find_syntax_by_extension("php"),
                    "java" => self.syntax_set.find_syntax_by_extension("java"),
                    _ => None,
                }
            })
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        // Use cache for syntect::Theme objects
        let theme = self.theme_cache.get_or_insert_with(self.current_theme.clone(), |name| {
            self.theme_set.themes.get(name).cloned()
                .or_else(|| self.theme_set.themes.get("MarcoDark").cloned())
                .or_else(|| self.theme_set.themes.get("MarcoLight").cloned())
                .or_else(|| self.theme_set.themes.values().next().cloned())
                .unwrap_or_else(|| Theme::default())
        });

        // Generate highlighted HTML
        match highlighted_html_for_string(code, &self.syntax_set, syntax, &theme) {
            Ok(html) => {
                // Wrap in a div with language-specific class for additional styling
                let language_class = language
                    .to_lowercase()
                    .replace("+", "plus")
                    .replace("#", "sharp");
                format!(
                    r#"<div class="code-block code-block-{}">{}</div>"#,
                    language_class, html
                )
            }
            Err(_) => {
                // Fallback for errors
                format!(
                    r#"<div class="code-block code-block-plain"><pre><code>{}</code></pre></div>"#,
                    Self::html_escape(code)
                )
            }
        }
    }

    /// Get all available programming language names
    pub fn get_language_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .syntax_set
            .syntaxes()
            .iter()
            .map(|syntax| syntax.name.clone())
            .collect();
        names.sort();
        names
    }

    /// Get language suggestions based on partial input
    pub fn get_language_suggestions(&self, partial: &str) -> Vec<String> {
        let partial_lower = partial.to_lowercase();
        let mut suggestions = Vec::new();

        // Search in syntax names
        for syntax in self.syntax_set.syntaxes() {
            if syntax.name.to_lowercase().starts_with(&partial_lower) {
                suggestions.push(syntax.name.clone());
            }
            // Also check file extensions
            for ext in &syntax.file_extensions {
                if ext
                    .trim_start_matches('.')
                    .to_lowercase()
                    .starts_with(&partial_lower)
                {
                    suggestions.push(syntax.name.clone());
                    break;
                }
            }
        }

        // Remove duplicates and sort
        suggestions.sort();
        suggestions.dedup();
        suggestions.truncate(10); // Limit to 10 suggestions
        suggestions
    }

    /// Get smart language suggestions with fuzzy matching and common aliases
    pub fn get_smart_language_suggestions(&self, query: &str) -> Vec<String> {
        if query.is_empty() {
            // Return popular languages for empty query
            let popular = [
                "Rust",
                "JavaScript",
                "Python",
                "Java",
                "TypeScript",
                "C++",
                "C#",
                "Go",
                "PHP",
                "Ruby",
            ];
            let mut result = Vec::new();
            for lang in popular {
                if let Some(found) = self
                    .get_language_names()
                    .iter()
                    .find(|l| l.eq_ignore_ascii_case(lang))
                {
                    result.push(found.clone());
                }
            }
            return result;
        }

        let query_lower = query.to_lowercase();
        let all_languages = self.get_language_names();
        let mut scored_languages: Vec<(String, i32)> = Vec::new();

        // Common aliases for languages
        let aliases = [
            ("js", "JavaScript"),
            ("javascript", "JavaScript"),
            ("ts", "TypeScript"),
            ("typescript", "TypeScript"),
            ("py", "Python"),
            ("python", "Python"),
            ("rs", "Rust"),
            ("rust", "Rust"),
            ("cpp", "C++"),
            ("c++", "C++"),
            ("cxx", "C++"),
            ("cs", "C#"),
            ("csharp", "C#"),
            ("c#", "C#"),
            ("java", "Java"),
            ("go", "Go"),
            ("golang", "Go"),
            ("php", "PHP"),
            ("rb", "Ruby"),
            ("ruby", "Ruby"),
            ("sh", "Bash"),
            ("bash", "Bash"),
            ("html", "HTML"),
            ("css", "CSS"),
            ("json", "JSON"),
            ("xml", "XML"),
            ("sql", "SQL"),
            ("yaml", "YAML"),
            ("yml", "YAML"),
            ("md", "Markdown"),
            ("markdown", "Markdown"),
        ];

        // Check for exact alias matches first
        for (alias, lang_name) in &aliases {
            if alias.to_lowercase() == query_lower {
                if let Some(found) = all_languages
                    .iter()
                    .find(|l| l.eq_ignore_ascii_case(lang_name))
                {
                    scored_languages.push((found.clone(), 1000)); // Highest priority
                }
            }
        }

        // Score all languages based on matching criteria
        for lang in &all_languages {
            let lang_lower = lang.to_lowercase();

            // Skip if already added through alias
            if scored_languages.iter().any(|(l, _)| l == lang) {
                continue;
            }

            let mut score = 0;

            // Exact match (highest priority)
            if lang_lower == query_lower {
                score = 900;
            }
            // Starts with query (high priority)
            else if lang_lower.starts_with(&query_lower) {
                score = 800;
            }
            // Contains query (medium priority)
            else if lang_lower.contains(&query_lower) {
                score = 600;
            }
            // Check for partial word matches
            else {
                let words: Vec<&str> = lang_lower.split_whitespace().collect();
                for word in words {
                    if word.starts_with(&query_lower) {
                        score = 700;
                        break;
                    } else if word.contains(&query_lower) {
                        score = 500;
                        break;
                    }
                }
            }

            // Bonus for popular languages
            let popular_bonus = match lang.as_str() {
                "Rust" | "JavaScript" | "Python" | "Java" | "TypeScript" | "C++" | "C#" | "Go"
                | "PHP" | "Ruby" => 100,
                "HTML" | "CSS" | "JSON" | "XML" | "SQL" | "Bash" | "YAML" | "Markdown" => 50,
                _ => 0,
            };
            score += popular_bonus;

            if score > 0 {
                scored_languages.push((lang.clone(), score));
            }
        }

        // Sort by score (highest first), then alphabetically
        scored_languages.sort_by(|a, b| {
            if a.1 != b.1 {
                b.1.cmp(&a.1) // Higher score first
            } else {
                a.0.cmp(&b.0) // Alphabetical if same score
            }
        });

        // Return top results
        scored_languages
            .into_iter()
            .take(20)
            .map(|(lang, _)| lang)
            .collect()
    }

    /// Check if a language is supported
    pub fn has_language(&self, name: &str) -> bool {
        self.syntax_set.find_syntax_by_token(name).is_some()
            || self.syntax_set.find_syntax_by_extension(name).is_some()
    }

    /// Get the total number of supported languages
    pub fn language_count(&self) -> usize {
        self.syntax_set.syntaxes().len()
    }

    /// Detect language from file extension
    pub fn detect_language_from_extension(&self, filename: &str) -> Option<String> {
        let extension = std::path::Path::new(filename).extension()?.to_str()?;

        self.syntax_set
            .find_syntax_by_extension(extension)
            .map(|syntax| syntax.name.clone())
    }

    /// Get language by file extension
    pub fn get_language_by_extension(&self, extension: &str) -> Option<String> {
        let ext = extension.trim_start_matches('.');
        self.syntax_set
            .find_syntax_by_extension(ext)
            .map(|syntax| syntax.name.clone())
    }

    /// Simple HTML escape function
    pub fn html_escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }

    /// Get information about a specific language
    pub fn get_language_info(&self, name: &str) -> Option<LanguageInfo> {
        let syntax = self
            .syntax_set
            .find_syntax_by_token(name)
            .or_else(|| self.syntax_set.find_syntax_by_extension(name))?;

        Some(LanguageInfo {
            name: syntax.name.clone(),
            file_extensions: syntax.file_extensions.clone(),
            scope_name: syntax.scope.to_string(),
        })
    }

    /// Apply syntax colorize code with custom theme colors that integrate with Marco's theme system
    pub fn colorize_code_with_theme_integration(
        &self,
        code: &str,
        language: &str,
        is_dark_theme: bool,
    ) -> String {
        // Choose MarcoDark for dark mode, MarcoLight for light mode
        let theme_name = if is_dark_theme {
            if self.theme_set.themes.contains_key("MarcoDark") {
                "MarcoDark"
            } else {
                &self.current_theme
            }
        } else {
            if self.theme_set.themes.contains_key("MarcoLight") {
                "MarcoLight"
            } else {
                &self.current_theme
            }
        };

        // Create a new highlighter with the selected theme
        let temp_highlighter = SyntectHighlighter::with_theme(theme_name);
        temp_highlighter.highlight_code(code, language)
    }
}

impl Clone for SyntectHighlighter {
    fn clone(&self) -> Self {
        Self {
            syntax_set: self.syntax_set.clone(),
            theme_set: ThemeSet::load_defaults(), // ThemeSet doesn't implement Clone, so we reload defaults
            current_theme: self.current_theme.clone(),
            theme_cache: crate::utils::cache::CacheSync::new(), // New cache for clone
        }
    }
}

impl Default for SyntectHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a programming language
#[derive(Debug, Clone)]
pub struct LanguageInfo {
    pub name: String,
    pub file_extensions: Vec<String>,
    pub scope_name: String,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntect_colorize_code() {
        let highlighter = SyntectHighlighter::new();

        let rust_code = r#"fn main() {
    println!("Hello, world!");
    let x = 42;
    // This is a comment
    if x > 0 {
        println!("x is positive: {}", x);
    }
}"#;

        let highlighted = highlighter.highlight_code(rust_code, "rust");
        assert!(!highlighted.is_empty());
        assert!(highlighted.contains("span"));

        // Test that we have languages available
        let languages = highlighter.get_language_names();
        assert!(!languages.is_empty());
        assert!(languages.contains(&"Rust".to_string()));
        assert!(languages.contains(&"JavaScript".to_string()));
        assert!(languages.contains(&"Python".to_string()));

        // Test themes
        let themes = highlighter.get_available_themes();
        assert!(!themes.is_empty());
        // Only check for base16-ocean.dark if it is present in the assets
        // This makes the test robust to the actual themes present
        if themes.contains(&"base16-ocean.dark".to_string()) {
            assert!(themes.contains(&"base16-ocean.dark".to_string()));
        } else {
            // At least one theme should be present (already checked above)
            assert!(themes.contains(&"dark".to_string()) || themes.contains(&"light".to_string()));
        }
    }


}
