//! wry-based preview helpers for Windows
//!
//! This module provides minimal, safe Windows implementations that mirror the
//! `webkit6` API surface so the rest of the codebase can call the same functions.
//!
// Note: this module is conditionally compiled from `components::viewer::mod`.

use gtk4::glib::object::IsA;
use std::sync::{Mutex, OnceLock};

use super::wry_platform_webview::PlatformWebView;

// Thread-safe global to store the latest preview HTML so detached preview windows
// can read it when they start.
pub(crate) static LATEST_PREVIEW_HTML: OnceLock<Mutex<String>> = OnceLock::new();

// Thread-safe global to store the latest clean live-preview HTML (no scroll-sync JS)
// for "Save as HTML" export.
pub(crate) static LATEST_LIVE_HTML: OnceLock<Mutex<String>> = OnceLock::new();

fn latest_live_html_mutex() -> &'static Mutex<String> {
    LATEST_LIVE_HTML.get_or_init(|| Mutex::new(String::new()))
}

/// Store the clean live-preview HTML (no wheel_js / scroll-report JS) for export.
pub(crate) fn set_latest_live_html(html: &str) {
    if let Ok(mut guard) = latest_live_html_mutex().lock() {
        *guard = html.to_string();
    }
}

/// Retrieve the clean live-preview HTML for "Save as HTML" export.
pub fn get_latest_live_html() -> String {
    latest_live_html_mutex()
        .lock()
        .ok()
        .map(|g| g.clone())
        .unwrap_or_default()
}

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
    let html =
        marco_core::render::wrap_preview_html_document(body, css, theme_mode, background_color);
    // Always keep <html dir="ltr"> so the WebKit viewport scrollbar stays on the right,
    // consistent with the editor/TOC scrollbar behaviour.  For RTL documents, inject
    // dir="rtl" on <body> instead — content flows RTL while the scrollbar stays right.
    let html = html.replacen("<html ", "<html dir=\"ltr\" ", 1);
    if crate::logic::rtl::is_rtl_global() {
        html.replacen("<body>", "<body dir=\"rtl\">", 1)
    } else {
        html
    }
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

/// Create a syntax-highlighted HTML source viewer backed by a wry / WebView2
/// `PlatformWebView` (§14.5 of `webkit6_wry_parity_audit.md`, Step 5b).
///
/// Mirrors [`super::webkit6::create_html_source_viewer_webview`] so both
/// platforms render bit-identical output. The HTML page is built by the
/// shared [`super::code_view_html::build_full_page`] helper (syntect-based
/// highlighting + theme / scrollbar CSS + body shell), then loaded into a
/// fresh `PlatformWebView` attached to `parent_window`.
///
/// The caller is responsible for inserting `pv.widget()` into the visual tree
/// (typically a surrounding `ScrolledWindow`) and for retaining the
/// `PlatformWebView` so subsequent [`update_code_view_smooth`] calls have a
/// live `evaluate_script` target.
pub fn create_html_source_viewer_webview(
    parent_window: &impl IsA<gtk4::Window>,
    html_source: &str,
    theme_mode: &str,
    base_uri: Option<&str>,
    editor_bg: Option<&str>,
    editor_fg: Option<&str>,
    scrollbar_thumb: Option<&str>,
    scrollbar_track: Option<&str>,
) -> Result<PlatformWebView, String> {
    log::debug!(
        "[wry] Creating WebView-based code viewer with theme: {} (source: {} bytes)",
        theme_mode,
        html_source.len()
    );

    // Same builder the Linux branch uses — keeps the highlighted HTML output
    // identical across backends.
    let complete_page = crate::components::viewer::code_view_html::build_full_page(
        html_source,
        theme_mode,
        editor_bg,
        editor_fg,
        scrollbar_thumb,
        scrollbar_track,
    )?;

    log::debug!(
        "[wry] Generated code-view HTML page: {} bytes",
        complete_page.len()
    );

    let pv = PlatformWebView::new(parent_window);
    pv.load_html_with_base(&complete_page, base_uri);
    Ok(pv)
}

/// Smooth update for the wry code-view WebView — mirrors
/// [`super::webkit6::update_code_view_smooth`] (§14.5, Step 5b).
///
/// Builds the update script via [`super::code_view_html::build_smooth_update_js`]
/// (shared with webkit6 so the JS payload is identical) and dispatches it
/// through [`PlatformWebView::evaluate_script`]. No DOM full-reload occurs:
/// only the highlighted code body and theme CSS are swapped.
pub fn update_code_view_smooth(
    pv: &PlatformWebView,
    html_source: &str,
    theme_mode: &str,
    editor_bg: Option<&str>,
    editor_fg: Option<&str>,
    scrollbar_thumb: Option<&str>,
    scrollbar_track: Option<&str>,
) -> Result<(), String> {
    let js_code = crate::components::viewer::code_view_html::build_smooth_update_js(
        html_source,
        theme_mode,
        editor_bg,
        editor_fg,
        scrollbar_thumb,
        scrollbar_track,
    )?;

    pv.evaluate_script(&js_code);
    Ok(())
}

/// Start autoplay timers for all `marco_sliders` decks in the current preview (if any).
///
/// Mirrors `webkit6::sliders_play_all` so `MarcoCorePreview.sliders.playAll()`
/// can be invoked uniformly across both backends (Gap #9 / §14.9).
#[allow(dead_code)]
pub fn sliders_play_all(webview: &super::wry_platform_webview::PlatformWebView) {
    let js = r#"(function(){try{if(window.MarcoCorePreview&&window.MarcoCorePreview.sliders&&typeof window.MarcoCorePreview.sliders.playAll==='function'){window.MarcoCorePreview.sliders.playAll();}}catch(e){console.error('sliders_play_all error',e);}})();"#;
    webview.evaluate_script(js);
}

/// Stop autoplay timers for all `marco_sliders` decks in the current preview (if any).
///
/// Mirrors `webkit6::sliders_pause_all` (Gap #9 / §14.9).
#[allow(dead_code)]
pub fn sliders_pause_all(webview: &super::wry_platform_webview::PlatformWebView) {
    let js = r#"(function(){try{if(window.MarcoCorePreview&&window.MarcoCorePreview.sliders&&typeof window.MarcoCorePreview.sliders.pauseAll==='function'){window.MarcoCorePreview.sliders.pauseAll();}}catch(e){console.error('sliders_pause_all error',e);}})();"#;
    webview.evaluate_script(js);
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
