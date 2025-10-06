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
//! 1. Uses marco_core's cached parser for performance
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

use crate::components::css::theme::{generate_syntax_highlighting_css, load_theme_css};
use crate::components::utils::get_theme_mode;
use marco_core::components::marco_engine::{global_parser_cache, HtmlOptions};
use marco_core::logic::swanson::SettingsManager;
use std::sync::Arc;
use webkit6::prelude::WebViewExt;

/// Load a markdown file and render it to HTML in the WebView
pub fn load_and_render_markdown(
    webview: &webkit6::WebView,
    file_path: &str,
    theme: &str,
    settings_manager: &Arc<SettingsManager>,
) {
    match std::fs::read_to_string(file_path) {
        Ok(content) => {
            // Parse markdown to HTML using marco_core
            let html = parse_markdown_to_html(&content, theme, settings_manager);
            
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
                        log::warn!("Cannot determine base URI for file: {}, using file:/// root", file_path);
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
            // Show error in WebView
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
        <h2>⚠️ Error Loading File</h2>
        <p>Could not read file: <code>{}</code></p>
        <p>Error: {}</p>
    </div>
</body>
</html>"#,
                file_path, e
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
) -> String {
    // Determine theme_mode (light/dark) from settings
    let theme_mode = get_theme_mode(settings_manager);
    
    log::debug!("Using theme_mode for syntax highlighting: {}", theme_mode);
    
    // Configure HTML rendering options with theme_mode for syntax highlighting
    let html_options = HtmlOptions {
        theme_mode: theme_mode.clone(),
        ..HtmlOptions::default()
    };
    
    // Parse markdown to HTML using global parser cache
    match global_parser_cache().render_with_cache(content, html_options) {
        Ok(html) => {
            // Load theme CSS
            let theme_css = load_theme_css(theme);
            
            // Generate syntax highlighting CSS for code blocks
            let syntax_css = generate_syntax_highlighting_css(&theme_mode);
            
            // Combine theme CSS with syntax highlighting CSS
            let combined_css = if !syntax_css.is_empty() {
                format!(
                    "{}\n\n/* Syntax Highlighting CSS */\n{}",
                    theme_css, syntax_css
                )
            } else {
                theme_css
            };
            
            // Create theme class for HTML element (theme-light or theme-dark)
            let theme_class = format!("theme-{}", theme_mode);
            log::debug!("Generated HTML with theme class: {}", theme_class);
            
            // Wrap in complete HTML document with theme class on <html> element
            // This allows CSS to target .theme-light or .theme-dark for proper theming
            format!(
                r#"<!DOCTYPE html>
<html class="{}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        {}
    </style>
</head>
<body>
    {}
</body>
</html>"#,
                theme_class, combined_css, html
            )
        }
        Err(e) => {
            // Show parse error
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
        <h2>⚠️ Markdown Parse Error</h2>
        <pre>{}</pre>
    </div>
</body>
</html>"#,
                e
            )
        }
    }
}
