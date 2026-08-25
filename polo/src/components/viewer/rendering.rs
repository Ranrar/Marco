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
use crate::components::viewer::platform_webview::PlatformWebView;
use gtk4::gio;
use gtk4::glib;
use marco_core::RenderOptions;
use marco_shared::cache::{cached, parse_to_html_cached};
use marco_shared::logic::swanson::SettingsManager;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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

/// Compute a `file://` base URI for relative resource resolution.
///
/// Returns `file:///path/to/directory/` (trailing slash) so WebKit can
/// resolve relative image and link paths in the rendered document.
fn compute_base_uri(file_path: &str) -> String {
    if let Ok(absolute_path) = Path::new(file_path).canonicalize() {
        if let Some(parent_dir) = absolute_path.parent() {
            return path_to_file_uri_dir(parent_dir);
        }
        return path_to_file_uri_dir(&absolute_path);
    }
    std::env::current_dir()
        .ok()
        .map(|d| path_to_file_uri_dir(&d))
        .unwrap_or_else(|| {
            log::warn!(
                "Cannot determine base URI for file: {}, using file:/// root",
                file_path
            );
            "file:///".to_string()
        })
}

/// Convert an absolute directory path into a `file://` base URI, with a
/// trailing slash.
///
/// `Path::canonicalize()` on Windows returns verbatim (`\\?\C:\...`) paths
/// with backslash separators; embedding that raw string after `file://`
/// (as this used to do) produces a URI WebView2 rejects outright — no `/`
/// separates the authority from the path, and the literal `\\?\` sequence
/// percent-encodes to `%5C%5C%3F%5C` instead of a real path start. Strip the
/// verbatim prefix and switch to forward slashes so the result is a URI a
/// webview backend will actually accept.
fn path_to_file_uri_dir(dir: &Path) -> String {
    let raw = dir.to_string_lossy();
    #[cfg(target_os = "windows")]
    let raw = raw
        .strip_prefix(r"\\?\UNC\")
        .map(|rest| std::borrow::Cow::Owned(format!(r"\\{rest}")))
        .unwrap_or_else(|| {
            raw.strip_prefix(r"\\?\")
                .map(|rest| std::borrow::Cow::Owned(rest.to_string()))
                .unwrap_or(raw)
        });

    let mut normalized = raw.replace('\\', "/");
    if !normalized.ends_with('/') {
        normalized.push('/');
    }

    if normalized.starts_with('/') {
        format!("file://{normalized}")
    } else {
        format!("file:///{normalized}")
    }
}

/// Build a complete HTML document from markdown content.
///
/// This function is designed to run on a **background thread** — it takes
/// only owned, `Send`-compatible values and performs no GTK calls.
fn build_html_from_content(
    content: String,
    theme: String,
    theme_mode: String,
    asset_root: PathBuf,
) -> String {
    let render_options = RenderOptions {
        syntax_highlighting: true,
        line_numbers: false,
        theme: theme_mode.clone(),
    };

    match parse_to_html_cached(&content, render_options) {
        Ok(html) => {
            let theme_css = load_theme_css_from_path(&theme, &asset_root);
            let syntax_css = generate_syntax_highlighting_css(&theme_mode);
            let scrollbar_css = generate_webkit_scrollbar_css(&theme_mode);

            let mut combined_css = if !syntax_css.is_empty() {
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
            // Shared "drop file to open" overlay CSS — see
            // `viewer::drop_overlay` for why a loaded document carries the
            // exact same markup/CSS the empty state does, instead of a
            // separate native GTK panel.
            combined_css.push_str("\n\n/* Drag-and-drop overlay */\n");
            combined_css.push_str(crate::components::viewer::drop_overlay::CSS);
            // In-page link-hover tooltip CSS — see `viewer::link_hover` for
            // why this replaced a native GTK Popover.
            combined_css.push_str("\n\n/* Link-hover tooltip */\n");
            combined_css.push_str(crate::components::viewer::link_hover::CSS);
            // In-page zoom toolbar CSS — see `viewer::zoom_bar`.
            combined_css.push_str("\n\n/* Zoom toolbar */\n");
            combined_css.push_str(crate::components::viewer::zoom_bar::CSS);

            // Append the link-hover tooltip markup (see `viewer::link_hover`)
            // and the shared drag-and-drop overlay markup (see
            // `viewer::drop_overlay`) *before* the hover-tooltip driver JS
            // (`viewer::javascript::HOVER_REPORT_JS`) — that script looks up
            // the tooltip's elements by ID synchronously on load, so they
            // must already exist in the DOM by the time it runs, the same
            // reason it already runs after `html` so every link in the
            // document exists first.
            let html_with_js = format!(
                "{}{}{}{}{}",
                html,
                crate::components::viewer::link_hover::html(),
                crate::components::viewer::drop_overlay::html(),
                crate::components::viewer::javascript::HOVER_REPORT_JS,
                crate::components::viewer::zoom_bar::html()
            );

            let theme_class = format!("theme-{}", theme_mode);
            marco_core::render::wrap_preview_html_document(
                &html_with_js,
                &combined_css,
                &theme_class,
                None,
            )
        }
        Err(e) => format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: system-ui, sans-serif; padding: 2rem; background: #1e1e1e; color: #ccc; }}
        .error {{ background: #5a1d1d; border-left: 4px solid #f48771; padding: 1rem; border-radius: 4px; }}
        .error h2 {{ margin-top: 0; color: #f48771; }}
        pre {{ background: #2d2d2d; padding: 1rem; border-radius: 4px; overflow-x: auto; }}
    </style>
</head>
<body><div class="error"><h2>Markdown Parse Error</h2><pre>{}</pre></div></body>
</html>"#,
            html_escape(&e.to_string())
        ),
    }
}

/// Load a markdown file and render it to HTML in the WebView.
///
/// File reading (cached) happens on the **main thread** (fast – I/O is
/// already cached by `marco_shared::cache`).  The expensive parse + render
/// step is dispatched to a Tokio/GLib background thread via
/// [`gio::spawn_blocking`] so the GTK event loop stays responsive while
/// large documents are being processed.
pub fn load_and_render_markdown(
    webview: &PlatformWebView,
    file_path: &str,
    theme: &str,
    settings_manager: &Arc<SettingsManager>,
    asset_root: &Path,
) {
    // Read file content on the main thread (cached, UTF-8 sanitised — fast).
    let content = match cached::read_to_string(Path::new(file_path)) {
        Ok(c) => c,
        Err(e) => {
            let error_html = format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: system-ui, sans-serif; padding: 2rem; background: #1e1e1e; color: #ccc; }}
        .error {{ background: #5a1d1d; border-left: 4px solid #f48771; padding: 1rem; border-radius: 4px; }}
        .error h2 {{ margin-top: 0; color: #f48771; }}
        code {{ background: #2d2d2d; padding: 0.2rem 0.4rem; border-radius: 3px; font-family: monospace; }}
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
            webview.load_html_with_base(&error_html, None);
            return;
        }
    };

    // Compute the base URI while we still have the file path (main thread, trivial).
    let base_uri = compute_base_uri(file_path);
    log::debug!("Loading HTML with base URI: {}", base_uri);

    // Pre-compute theme_mode on the main thread so we don't need to send
    // `Arc<SettingsManager>` (avoids any Send-bound concerns).
    let theme_mode = get_theme_mode(settings_manager);

    // Owned copies for the background task.
    let theme = theme.to_string();
    let asset_root = asset_root.to_path_buf();
    let webview = webview.clone();

    // Show the indeterminate progress bar over the WebView while we parse and
    // render.  It hides itself when the rendered HTML is loaded (or on error).
    crate::components::viewer::loading_overlay::show();

    // Dispatch the expensive parse + render to a background thread so the GTK
    // event loop stays responsive for large files (e.g. stresstest.md).
    // GTK / WebView calls stay on the main thread — only plain data crosses.
    glib::spawn_future_local(async move {
        let rendered = gio::spawn_blocking(move || {
            build_html_from_content(content, theme, theme_mode, asset_root)
        })
        .await;

        match rendered {
            Ok(html) => webview.load_html_with_base(&html, Some(&base_uri)),
            Err(e) => {
                log::error!("[polo] Background render task panicked: {:?}", e);
                // Render failed — no `load-finished` will fire, hide manually.
                crate::components::viewer::loading_overlay::hide();
            }
        }
        // Success path: the WebView's `load-finished` signal (wired in
        // `main.rs`) hides the overlay once the new HTML is actually painted.
    });
}

/// Parse markdown content to HTML with theme styling.
///
/// Prefer [`load_and_render_markdown`] for file-based rendering — it offloads
/// the work to a background thread automatically.  This function is kept for
/// cases where the caller already holds the markdown string and needs the HTML
/// synchronously (e.g. export pipelines or tests).
#[allow(dead_code)]
pub fn parse_markdown_to_html(
    content: &str,
    theme: &str,
    settings_manager: &Arc<SettingsManager>,
    asset_root: &Path,
) -> String {
    let theme_mode = get_theme_mode(settings_manager);
    log::debug!("Using theme_mode for syntax highlighting: {}", theme_mode);
    build_html_from_content(
        content.to_string(),
        theme.to_string(),
        theme_mode,
        asset_root.to_path_buf(),
    )
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
