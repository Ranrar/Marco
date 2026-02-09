//! wry-based preview helpers for Windows
//!
//! This module provides minimal, safe Windows implementations that mirror the
//! `webkit6` API surface so the rest of the codebase can call the same functions.
//!
// Note: this module is conditionally compiled from `components::viewer::mod`.

use gtk4::prelude::*;
use gtk4::{ScrolledWindow, TextView};
use std::sync::{Mutex, OnceLock};

// Thread-safe global to store the latest preview HTML so detached preview windows
// can read it when they start.
pub(crate) static LATEST_PREVIEW_HTML: OnceLock<Mutex<String>> = OnceLock::new();

// Thread-safe global to store the latest base URI (directory) for resolving
// relative resources in detached preview windows.
pub(crate) static LATEST_PREVIEW_BASE_URI: OnceLock<Mutex<Option<String>>> = OnceLock::new();

fn latest_base_uri_mutex() -> &'static Mutex<Option<String>> {
    LATEST_PREVIEW_BASE_URI.get_or_init(|| Mutex::new(None))
}

pub(crate) fn set_latest_preview_base_uri(base_uri: Option<String>) {
    if let Ok(mut guard) = latest_base_uri_mutex().lock() {
        *guard = base_uri;
    }
}

pub(crate) fn get_latest_preview_base_uri() -> Option<String> {
    latest_base_uri_mutex().lock().ok().and_then(|g| g.clone())
}

/// Wraps HTML body into a full document (delegates to core renderer);
/// kept for API compatibility.
pub fn wrap_html_document(
    body: &str,
    css: &str,
    theme_mode: &str,
    background_color: Option<&str>,
) -> String {
    core::render::wrap_preview_html_document(body, css, theme_mode, background_color)
}

/// Generate a file:// base URI from a document path for resolving relative paths.
pub fn generate_base_uri_from_path<P: AsRef<std::path::Path>>(document_path: P) -> Option<String> {
    if let Some(parent_dir) = document_path.as_ref().parent() {
        let absolute_parent = parent_dir
            .canonicalize()
            .unwrap_or_else(|_| parent_dir.to_path_buf());

        // Windows file URIs must start with file:/// and use forward slashes.
        // Also ensure a trailing slash so relative URLs resolve under the directory.
        let mut s = absolute_parent.to_string_lossy().replace('\\', "/");
        if !s.ends_with('/') {
            s.push('/');
        }

        return Some(format!("file:///{}", s));
    }
    None
}

/// Generate test HTML content when the editor is empty
pub(crate) fn generate_test_html(wheel_js: &str) -> String {
    let welcome_html = r#"<div id=\"welcome-message\" style=\"
  text-align:center; 
  margin-top:20%; 
  opacity:0.7; 
  font-family:sans-serif;\">
        <h1>Welcome to marco</h1>
  <p>Start typing or open a file to begin your writing journey ✍️</p>
</div>"#;
    let mut html_with_js = welcome_html.to_string();
    html_with_js.push_str(wheel_js);
    html_with_js
}

/// Create a simple HTML source viewer widget.
///
/// Note: on Windows we keep the code-preview as a `TextView` (not a WebView).
/// The caller typically inserts this widget into a surrounding `ScrolledWindow`.
pub fn create_html_source_viewer_webview(
    html_source: &str,
    _theme_mode: &str,
    _base_uri: Option<&str>,
    _editor_bg: Option<&str>,
    _editor_fg: Option<&str>,
    _scrollbar_thumb: Option<&str>,
    _scrollbar_track: Option<&str>,
) -> Result<gtk4::Widget, String> {
    let tv = TextView::new();
    tv.set_editable(false);
    tv.set_monospace(true);
    tv.buffer().set_text(html_source);

    Ok(tv.upcast::<gtk4::Widget>())
}

/// Smooth update for the source view - update text in TextView.
pub fn update_code_view_smooth(
    widget: &gtk4::Widget,
    html_source: &str,
    _theme_mode: &str,
    _editor_bg: Option<&str>,
    _editor_fg: Option<&str>,
    _scrollbar_thumb: Option<&str>,
    _scrollbar_track: Option<&str>,
) -> Result<(), String> {
    if let Ok(tv) = widget.clone().downcast::<TextView>() {
        tv.buffer().set_text(html_source);
        return Ok(());
    }

    if let Ok(scrolled) = widget.clone().downcast::<ScrolledWindow>() {
        if let Some(child) = scrolled.child() {
            if let Ok(tv) = child.clone().downcast::<TextView>() {
                tv.buffer().set_text(html_source);
                return Ok(());
            }

            // GTK ScrolledWindow may wrap the child inside a Viewport.
            if let Ok(viewport) = child.downcast::<gtk4::Viewport>() {
                if let Some(inner) = viewport.child() {
                    if let Ok(tv) = inner.downcast::<TextView>() {
                        tv.buffer().set_text(html_source);
                        return Ok(());
                    }
                }
            }
        }
    }
    Err("Failed to find TextView to update code view".to_string())
}

/// Open external URI in system browser
pub fn open_external_uri(uri: &str) -> Result<(), String> {
    let normalized_uri = if uri.to_lowercase().starts_with("www.") {
        format!("{}://{}", "https", uri)
    } else {
        uri.to_string()
    };

    match gio::AppInfo::launch_default_for_uri(&normalized_uri, None::<&gio::AppLaunchContext>) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to open URI '{}': {}", normalized_uri, e)),
    }
}
