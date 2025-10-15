//! Shared rendering utilities for HTML generation
//!
//! Contains helper functions used by both block and inline renderers.

/// Helper function to determine if a URL should open in a new tab
/// Returns true for URLs that should have target="_blank"
/// This includes: http/https URLs, www URLs, and mailto links
pub fn is_external_url(url: &str) -> bool {
    let url_lower = url.to_lowercase();
    url_lower.starts_with("http://")
        || url_lower.starts_with("https://")
        || url_lower.starts_with("www.")
        || url_lower.starts_with("mailto:")
}

/// Format a language identifier into a human-readable display name
/// Examples: "rust" -> "Rust", "javascript" -> "JavaScript", "cpp" -> "C++"
pub fn format_language_name(lang: &str) -> String {
    match lang.to_lowercase().as_str() {
        "rust" => "Rust".to_string(),
        "python" => "Python".to_string(),
        "javascript" | "js" => "JavaScript".to_string(),
        "typescript" | "ts" => "TypeScript".to_string(),
        "html" => "HTML".to_string(),
        "css" => "CSS".to_string(),
        "json" => "JSON".to_string(),
        "xml" => "XML".to_string(),
        "yaml" | "yml" => "YAML".to_string(),
        "toml" => "TOML".to_string(),
        "bash" | "sh" => "Bash".to_string(),
        "c" => "C".to_string(),
        "cpp" | "c++" | "cxx" => "C++".to_string(),
        "java" => "Java".to_string(),
        "go" => "Go".to_string(),
        "php" => "PHP".to_string(),
        "ruby" => "Ruby".to_string(),
        "sql" => "SQL".to_string(),
        "markdown" | "md" => "Markdown".to_string(),
        "shell" => "Shell".to_string(),
        "powershell" | "ps1" => "PowerShell".to_string(),
        "dockerfile" => "Dockerfile".to_string(),
        "makefile" => "Makefile".to_string(),
        "cmake" => "CMake".to_string(),
        // For unknown languages, capitalize first letter
        _ => {
            let mut chars = lang.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }
    }
}

/// Escape HTML special characters
pub fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Check if HTML content is safe (basic sanitization)
/// Returns true if the content doesn't contain potentially unsafe patterns
pub fn is_safe_html(content: &str) -> bool {
    // Basic check for dangerous patterns
    let dangerous_patterns = [
        "<script", "</script>", "javascript:", "onerror=", "onload=", "onclick=",
        "eval(", "expression(",
    ];

    let content_lower = content.to_lowercase();
    !dangerous_patterns
        .iter()
        .any(|pattern| content_lower.contains(pattern))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_is_external_url() {
        // External URLs
        assert!(is_external_url("http://example.com"));
        assert!(is_external_url("https://example.com"));
        assert!(is_external_url("www.example.com"));
        assert!(is_external_url("mailto:user@example.com"));

        // Internal/relative URLs
        assert!(!is_external_url("/path/to/page"));
        assert!(!is_external_url("./relative/path"));
        assert!(!is_external_url("../parent/path"));
        assert!(!is_external_url("#anchor"));
        assert!(!is_external_url("page.html"));
    }

    #[test]
    fn smoke_test_format_language_name() {
        assert_eq!(format_language_name("rust"), "Rust");
        assert_eq!(format_language_name("javascript"), "JavaScript");
        assert_eq!(format_language_name("cpp"), "C++");
        assert_eq!(format_language_name("unknown"), "Unknown");
        assert_eq!(format_language_name(""), "");
    }

    #[test]
    fn smoke_test_escape_html() {
        assert_eq!(escape_html("<div>"), "&lt;div&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("\"quote\""), "&quot;quote&quot;");
    }

    #[test]
    fn smoke_test_is_safe_html() {
        // Safe content
        assert!(is_safe_html("<div>Hello</div>"));
        assert!(is_safe_html("<p class='test'>Text</p>"));

        // Unsafe content
        assert!(!is_safe_html("<script>alert('xss')</script>"));
        assert!(!is_safe_html("<img onerror='alert(1)'>"));
        assert!(!is_safe_html("javascript:alert(1)"));
    }
}
