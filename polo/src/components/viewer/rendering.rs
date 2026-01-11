// Markdown rendering to HTML
//
//! # Rendering Module
//!
//! Core markdown-to-HTML conversion and WebView loading logic.
//!
//! ## Functions
//!
//! ### `load_and_render_markdown`
//!
//! Main entry point for rendering a markdown file:
//! 1. Reads file content from disk
//! 2. Parses markdown to HTML using `parse_markdown_to_html`
//! 3. Generates base URI for relative resource resolution
//! 4. Loads HTML into WebView with base URI
//!
//! **Error Handling**: File read errors show themed error page in WebView.
//!
//! ### `parse_markdown_to_html`
//!
//! Internal function that:
//! 1. Uses core's cached parser for performance
//! 2. Loads selected CSS theme
//! 3. Generates syntax highlighting CSS based on light/dark mode
//! 4. Wraps rendered HTML in complete document with theme class
//!
//! **Error Handling**: Parse errors show themed error page with details.
//!
//! ## Base URI Resolution
//!
//! The base URI is critical for loading images and links relative to the markdown file:
//!
//! ```text
//! File: /home/user/docs/README.md
//! Image: ![logo](./images/logo.png)
//!
//! Base URI: file:///home/user/docs/
//! Resolved:  file:///home/user/docs/images/logo.png
//! ```
//!
//! ## Theme Integration
//!
//! HTML output includes:
//! - Selected CSS theme (github.css, marco.css, etc.)
//! - Generated syntax highlighting CSS (theme-aware)
//! - Theme class on `<html>` element (`.theme-light` or `.theme-dark`)

use crate::components::css::theme::{generate_syntax_highlighting_css, load_theme_css_from_path};
use crate::components::utils::get_theme_mode;
use core::logic::swanson::SettingsManager;
use core::{parse_to_html_cached, RenderOptions};
use std::path::Path;
use std::sync::Arc;
use webkit6::prelude::WebViewExt;

/// Light theme scrollbar colors (from assets/themes/editor/light.xml)
const LIGHT_SCROLLBAR_THUMB: &str = "#D0D4D8";
const LIGHT_SCROLLBAR_TRACK: &str = "#F0F0F0";

/// Dark theme scrollbar colors (from assets/themes/editor/dark.xml)
const DARK_SCROLLBAR_THUMB: &str = "#3A3F44";
const DARK_SCROLLBAR_TRACK: &str = "#252526";

/// Generate WebKit scrollbar CSS for HTML preview
/// Matches the GTK scrollbar styling in the main application
fn generate_webkit_scrollbar_css(theme_mode: &str) -> String {
    let (thumb, track) = if theme_mode == "dark" {
        (DARK_SCROLLBAR_THUMB, DARK_SCROLLBAR_TRACK)
    } else {
        (LIGHT_SCROLLBAR_THUMB, LIGHT_SCROLLBAR_TRACK)
    };

    format!(
        r#"
        /* Match editor scrollbar styling for WebView */
        ::-webkit-scrollbar {{ width: 12px; height: 12px; background: {}; }}
        ::-webkit-scrollbar-track {{ background: {}; }}
        ::-webkit-scrollbar-thumb {{ background: {}; border-radius: 0px; }}
        ::-webkit-scrollbar-thumb:hover {{ background: {}; opacity: 0.9; }}
        "#,
        track, track, thumb, thumb
    )
}

/// Escape HTML special characters to prevent XSS attacks
/// Converts &, <, >, ", and ' to their HTML entity equivalents
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Load a markdown file and render it to HTML in the WebView
pub fn load_and_render_markdown(
    webview: &webkit6::WebView,
    file_path: &str,
    theme: &str,
    settings_manager: &Arc<SettingsManager>,
    asset_root: &Path,
) {
    // Use the same cached file loader as Marco (includes UTF-8 sanitization)
    match core::logic::cache::cached::read_to_string(file_path) {
        Ok(content) => {
            // Parse markdown to HTML using core
            let html = parse_markdown_to_html(&content, theme, settings_manager, asset_root);

            // Generate base URI for relative resource resolution (images, links, etc.)
            // Format: file:///absolute/path/to/directory/ (with trailing slash)
            let base_uri = if let Ok(absolute_path) = std::path::Path::new(file_path).canonicalize()
            {
                if let Some(parent_dir) = absolute_path.parent() {
                    format!("file://{}/", parent_dir.display())
                } else {
                    format!("file://{}/", absolute_path.display())
                }
            } else {
                // Fallback: try to use current directory or file:/// root
                std::env::current_dir()
                    .ok()
                    .map(|d| format!("file://{}/", d.display()))
                    .unwrap_or_else(|| {
                        log::warn!(
                            "Cannot determine base URI for file: {}, using file:/// root",
                            file_path
                        );
                        "file:///".to_string()
                    })
            };

            log::debug!("Loading HTML with base URI: {}", base_uri);

            // Load HTML into WebView with base URI
            // Use idle_add_local to avoid GTK allocation warnings
            let webview_clone = webview.clone();
            let html_clone = html.clone();
            let base_uri_clone = base_uri.clone();
            gtk4::glib::idle_add_local_once(move || {
                webview_clone.load_html(&html_clone, Some(&base_uri_clone));
            });
        }
        Err(e) => {
            // Show error in WebView with properly escaped content to prevent XSS
            let error_html = format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{
            font-family: system-ui, -apple-system, sans-serif;
            padding: 2rem;
            background: #1e1e1e;
            color: #cccccc;
        }}
        .error {{
            background: #5a1d1d;
            border-left: 4px solid #f48771;
            padding: 1rem;
            border-radius: 4px;
        }}
        .error h2 {{
            margin-top: 0;
            color: #f48771;
        }}
        code {{
            background: #2d2d2d;
            padding: 0.2rem 0.4rem;
            border-radius: 3px;
            font-family: 'Courier New', monospace;
        }}
    </style>
</head>
<body>
    <div class="error">
        <h2>Error Loading File</h2>
        <p>Could not read file: <code>{}</code></p>
        <p>Error: {}</p>
    </div>
</body>
</html>"#,
                html_escape(file_path),
                html_escape(&e.to_string())
            );
            let webview_clone = webview.clone();
            gtk4::glib::idle_add_local_once(move || {
                webview_clone.load_html(&error_html, None);
            });
        }
    }
}

/// Parse markdown content to HTML with theme styling
pub fn parse_markdown_to_html(
    content: &str,
    theme: &str,
    settings_manager: &Arc<SettingsManager>,
    asset_root: &Path,
) -> String {
    // Determine theme_mode (light/dark) from settings
    let theme_mode = get_theme_mode(settings_manager);

    log::debug!("Using theme_mode for syntax highlighting: {}", theme_mode);

    // Configure HTML rendering options with syntax highlighting enabled
    let render_options = RenderOptions {
        syntax_highlighting: true, // Enable syntax highlighting for code blocks
        line_numbers: false,       // Polo viewer doesn't need line numbers
        theme: theme_mode.clone(), // Use theme_mode (light/dark) for highlighting
    };

    // Parse markdown to HTML using global parser cache with convenience function
    match parse_to_html_cached(content, render_options) {
        Ok(html) => {
            // Load theme CSS
            let theme_css = load_theme_css_from_path(theme, asset_root);

            // Generate syntax highlighting CSS for code blocks
            let syntax_css = generate_syntax_highlighting_css(&theme_mode);

            // Generate WebKit scrollbar CSS to match editor
            let scrollbar_css = generate_webkit_scrollbar_css(&theme_mode);

            // Combine theme CSS with syntax highlighting CSS and scrollbar CSS
            let combined_css = if !syntax_css.is_empty() {
                format!(
                    "{}\n\n/* Syntax Highlighting CSS */\n{}\n\n/* Scrollbar Styling */\n{}",
                    theme_css, syntax_css, scrollbar_css
                )
            } else {
                format!(
                    "{}\n\n/* Scrollbar Styling */\n{}",
                    theme_css, scrollbar_css
                )
            };

            // Create theme class for HTML element (theme-light or theme-dark)
            let theme_class = format!("theme-{}", theme_mode);
            log::debug!("Generated HTML with theme class: {}", theme_class);

            // Wrap in the shared preview document so both Marco and Polo get
            // identical in-page JS behavior (including table resizing).
            core::render::wrap_preview_html_document(&html, &combined_css, &theme_class, None)
        }
        Err(e) => {
            // Show parse error with properly escaped content to prevent XSS
            format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{
            font-family: system-ui, -apple-system, sans-serif;
            padding: 2rem;
            background: #1e1e1e;
            color: #cccccc;
        }}
        .error {{
            background: #5a1d1d;
            border-left: 4px solid #f48771;
            padding: 1rem;
            border-radius: 4px;
        }}
        .error h2 {{
            margin-top: 0;
            color: #f48771;
        }}
        pre {{
            background: #2d2d2d;
            padding: 1rem;
            border-radius: 4px;
            overflow-x: auto;
        }}
    </style>
</head>
<body>
    <div class="error">
        <h2>Markdown Parse Error</h2>
        <pre>{}</pre>
    </div>
</body>
</html>"#,
                html_escape(&e.to_string())
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_html_escape_basic() {
        assert_eq!(html_escape("hello"), "hello");
        assert_eq!(html_escape("hello & world"), "hello &amp; world");
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(html_escape("'apostrophe'"), "&#39;apostrophe&#39;");
    }

    #[test]
    fn smoke_test_html_escape_xss_prevention() {
        let malicious = "<script>alert('XSS')</script>";
        let escaped = html_escape(malicious);
        assert!(!escaped.contains("<script>"));
        assert!(escaped.contains("&lt;script&gt;"));
        assert!(escaped.contains("&#39;XSS&#39;"));
    }

    #[test]
    fn smoke_test_html_escape_multiple_chars() {
        let input = "<div class=\"test\" data-value='123'>A & B</div>";
        let escaped = html_escape(input);
        assert_eq!(
            escaped,
            "&lt;div class=&quot;test&quot; data-value=&#39;123&#39;&gt;A &amp; B&lt;/div&gt;"
        );
    }
}
