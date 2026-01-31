//! wry-based preview helpers for Windows
//!
//! This module provides minimal, safe Windows implementations that mirror the
//! `webkit6` API surface so the rest of the codebase can call the same functions.
//!
#![cfg(target_os = "windows")]

use gtk4::prelude::*;
use gtk4::{Label, ScrolledWindow, TextView, Box as GtkBox, Orientation};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Mutex, OnceLock};

// Thread-safe global to store the latest preview HTML so detached preview windows
// can read it when they start.
pub(crate) static LATEST_PREVIEW_HTML: OnceLock<Mutex<String>> = OnceLock::new();

fn latest_html_mutex() -> &'static Mutex<String> {
    LATEST_PREVIEW_HTML.get_or_init(|| Mutex::new(String::new()))
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
        if let Ok(absolute_parent) = parent_dir.canonicalize() {
            let path_str = absolute_parent.to_string_lossy();
            return Some(format!("file://{}/", path_str));
        } else {
            let path_str = parent_dir.to_string_lossy();
            return Some(format!("file://{}/", path_str));
        }
    }
    None
}

/// Create a simple placeholder widget for in-editor preview when running on Windows.
/// The returned widget is a `ScrolledWindow` with a label and short message.
pub fn create_html_viewer_with_base(
    _html: &str,
    _base_uri: Option<&str>,
    _background_color: Option<&str>,
) -> gtk4::Widget {
    let sw = ScrolledWindow::new();
    sw.set_hexpand(true);
    sw.set_vexpand(true);

    let vbox = GtkBox::new(Orientation::Vertical, 8);
    vbox.set_margin_top(8);
    vbox.set_margin_bottom(8);
    vbox.set_margin_start(8);
    vbox.set_margin_end(8);

    let label = Label::new(Some(
        "Preview not available inline on Windows. Click \"Open Preview Window\" in the titlebar to open a detached preview.",
    ));
    label.set_wrap(true);

    vbox.append(&label);
    sw.set_child(Some(&vbox));

    // Store the provided HTML into the global preview store so detached windows
    // can access the latest content.
    if !_html.is_empty() {
        if let Ok(mut guard) = latest_html_mutex().lock() {
            *guard = _html.to_string();
        }
    }

    sw.upcast::<gtk4::Widget>()
}

/// Generate test HTML content when the editor is empty
pub(crate) fn generate_test_html(wheel_js: &str) -> String {
    let welcome_html = r#"<div id=\"welcome-message\" style=\"
  text-align:center; 
  margin-top:20%; 
  opacity:0.7; 
  font-family:sans-serif;\">
  <h1>Welcome to Marco</h1>
  <p>Start typing or open a file to begin your writing journey ✍️</p>
</div>"#;
    let mut html_with_js = welcome_html.to_string();
    html_with_js.push_str(wheel_js);
    html_with_js
}

/// Load HTML into the placeholder widget. We store the HTML for detached preview
/// windows and update the label to show a small preview message.
pub fn load_html_when_ready(widget: &gtk4::Widget, html: String, _base_uri: Option<String>) {
    if let Ok(mut guard) = latest_html_mutex().lock() {
        *guard = html.clone();
    }

    // Update the in-editor placeholder if possible
    if let Some(scrolled) = widget.clone().downcast::<ScrolledWindow>().ok() {
        if let Some(child) = scrolled.child() {
            if let Ok(vbox) = child.downcast::<GtkBox>() {
                if let Some(label) = vbox.first_child() {
                    if let Ok(label) = label.downcast::<Label>() {
                        let preview_text = format!("Preview saved ({} bytes). Use detached preview to view.", html.len());
                        label.set_text(&preview_text);
                    }
                }
            }
        }
    }
}

/// Update the placeholder and saved HTML for smooth content updates.
pub fn update_html_content_smooth(_widget: &gtk4::Widget, content: &str) {
    if let Ok(mut guard) = latest_html_mutex().lock() {
        *guard = content.to_string();
    }
}

/// Create a simple HTML source viewer using a `TextView` inside a `ScrolledWindow`.
pub fn create_html_source_viewer_webview(
    html_source: &str,
    _theme_mode: &str,
    _base_uri: Option<&str>,
    _editor_bg: Option<&str>,
    _editor_fg: Option<&str>,
    _scrollbar_thumb: Option<&str>,
    _scrollbar_track: Option<&str>,
) -> Result<gtk4::Widget, String> {
    let sw = ScrolledWindow::new();
    sw.set_hexpand(true);
    sw.set_vexpand(true);

    let tv = TextView::new();
    tv.set_editable(false);
    tv.set_monospace(true);
    tv.buffer().set_text(html_source);

    sw.set_child(Some(&tv));
    Ok(sw.upcast::<gtk4::Widget>())
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
    if let Some(scrolled) = widget.clone().downcast::<ScrolledWindow>().ok() {
        if let Some(child) = scrolled.child() {
            if let Ok(tv) = child.downcast::<TextView>() {
                tv.buffer().set_text(html_source);
                return Ok(());
            }
        }
    }
    Err("Failed to find TextView to update code view".to_string())
}

/// No-op slider controls on Windows (parity only)
pub fn sliders_play_all(_widget: &gtk4::Widget) {}
pub fn sliders_pause_all(_widget: &gtk4::Widget) {}

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
