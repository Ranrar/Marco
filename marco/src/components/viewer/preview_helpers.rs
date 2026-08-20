//! Shared preview helpers used on both platforms: the latest-rendered-HTML /
//! base-URI cache (read by the detached-preview and Windows export-restore
//! paths), the empty-editor welcome page, external-link opening, and the
//! syntax-highlighted "code view" webview (construction + smooth updates).

use gtk4::glib::object::IsA;
use std::sync::{Mutex, OnceLock};

use super::platform_webview::PlatformWebView;

// Thread-safe global to store the latest preview HTML so detached preview
// windows (and the Windows export restore path) can read it when they start.
// Written by `backend::record_latest_preview` on every full preview load.
pub(crate) static LATEST_PREVIEW_HTML: OnceLock<Mutex<String>> = OnceLock::new();

/// Retrieve the most recent full preview document recorded by
/// [`crate::components::viewer::backend::record_latest_preview`].
#[allow(dead_code)] // Read by Windows-only export/detached-window code.
pub fn get_latest_preview_html() -> String {
    LATEST_PREVIEW_HTML
        .get_or_init(|| Mutex::new(String::new()))
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

#[allow(dead_code)] // Read by Windows-only detached-window code.
pub(crate) fn get_latest_preview_base_uri() -> Option<String> {
    latest_base_uri_mutex().lock().ok().and_then(|g| g.clone())
}

// Listener fired every time `record_latest_preview` updates the cache above.
// Not `Send` (the registered closure captures GTK types), so a plain
// thread-local is used — matches `loading_overlay`'s `GLOBAL` pattern.
//
// This exists for the Windows detached preview window
// (`detached_window_windows::PreviewWindow`), which — unlike Linux's, which
// reparents the *same live* webview and so needs no explicit refresh —
// rebuilds its content from this cache in a *separate* `PlatformWebView`
// instance (see its module doc comment, §14.3). Without this hook, an
// already-open detached window never learns the cache changed (e.g. on a
// theme switch, which forces a full-document rebuild on the main preview
// but has no other way to reach a webview it doesn't own), so its content —
// and, before this, its theme — appeared permanently stuck on whatever was
// rendered when it was first opened.
type RefreshListener = Box<dyn Fn()>;
thread_local! {
    static PREVIEW_REFRESHED_LISTENER: std::cell::RefCell<Option<RefreshListener>> =
        const { std::cell::RefCell::new(None) };
}

/// Register `f` to run every time [`crate::components::viewer::backend::record_latest_preview`]
/// updates the cached preview HTML/base URI. Replacing the listener overwrites the previous one.
#[allow(dead_code)] // Registered by Windows-only detached-window code.
pub fn set_preview_refreshed_listener(f: impl Fn() + 'static) {
    PREVIEW_REFRESHED_LISTENER.with(|cell| *cell.borrow_mut() = Some(Box::new(f)));
}

pub(crate) fn notify_preview_refreshed() {
    PREVIEW_REFRESHED_LISTENER.with(|cell| {
        if let Some(f) = cell.borrow().as_ref() {
            f();
        }
    });
}

/// Generate a file:// base URI from a document path for resolving relative paths.
pub fn generate_base_uri_from_path<P: AsRef<std::path::Path>>(document_path: P) -> Option<String> {
    if let Some(parent_dir) = document_path.as_ref().parent() {
        let absolute_parent = parent_dir
            .canonicalize()
            .unwrap_or_else(|_| parent_dir.to_path_buf());

        // `canonicalize()` on Windows returns a verbatim path
        // (`\\?\C:\...` or `\\?\UNC\server\share\...`); strip that prefix
        // before converting to a URI, or the WebView backend rejects the
        // resulting URL outright (the literal `\?` sequence doesn't survive
        // percent-encoding into anything a URI parser accepts).
        let raw = absolute_parent.to_string_lossy();
        #[cfg(target_os = "windows")]
        let raw = raw
            .strip_prefix(r"\\?\UNC\")
            .map(|rest| std::borrow::Cow::Owned(format!(r"\\{rest}")))
            .unwrap_or_else(|| {
                raw.strip_prefix(r"\\?\")
                    .map(|rest| std::borrow::Cow::Owned(rest.to_string()))
                    .unwrap_or(raw)
            });

        // File URIs use forward slashes and a trailing slash so relative URLs
        // resolve under the directory. Unix absolute paths already start with
        // '/' (file:///home/...); Windows drive paths need one added
        // (file:///C:/...).
        let mut s = raw.replace('\\', "/");
        if !s.ends_with('/') {
            s.push('/');
        }

        return if s.starts_with('/') {
            Some(format!("file://{}", s))
        } else {
            Some(format!("file:///{}", s))
        };
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

/// Editor-derived colors applied to the code-view page (§14.5, Step 5b).
///
/// All fields are optional CSS color strings; `None` falls back to the
/// theme-mode defaults inside [`super::code_view_html`].
#[derive(Debug, Clone, Copy, Default)]
pub struct CodeViewColors<'a> {
    pub editor_bg: Option<&'a str>,
    pub editor_fg: Option<&'a str>,
    pub scrollbar_thumb: Option<&'a str>,
    pub scrollbar_track: Option<&'a str>,
}

/// Create a syntax-highlighted HTML source viewer backed by a
/// `PlatformWebView` (§14.5 of the parity audit, Step 5b).
///
/// The HTML page is built by the shared
/// [`super::code_view_html::build_full_page`] helper (syntect-based
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
    colors: CodeViewColors<'_>,
) -> Result<PlatformWebView, String> {
    log::debug!(
        "[wry] Creating WebView-based code viewer with theme: {} (source: {} bytes)",
        theme_mode,
        html_source.len()
    );

    let complete_page = crate::components::viewer::code_view_html::build_full_page(
        html_source,
        theme_mode,
        colors.editor_bg,
        colors.editor_fg,
        colors.scrollbar_thumb,
        colors.scrollbar_track,
    )?;

    log::debug!(
        "[wry] Generated code-view HTML page: {} bytes",
        complete_page.len()
    );

    let pv = PlatformWebView::new(parent_window);
    pv.load_html_with_base(&complete_page, base_uri);
    Ok(pv)
}

/// Smooth update for the code-view WebView (§14.5, Step 5b).
///
/// Builds the update script via [`super::code_view_html::build_smooth_update_js`]
/// and dispatches it through [`PlatformWebView::evaluate_script`]. No DOM
/// full-reload occurs: only the highlighted code body and theme CSS are
/// swapped.
pub fn update_code_view_smooth(
    pv: &PlatformWebView,
    html_source: &str,
    theme_mode: &str,
    colors: CodeViewColors<'_>,
) -> Result<(), String> {
    let js_code = crate::components::viewer::code_view_html::build_smooth_update_js(
        html_source,
        theme_mode,
        colors.editor_bg,
        colors.editor_fg,
        colors.scrollbar_thumb,
        colors.scrollbar_track,
    )?;

    pv.evaluate_script(&js_code);
    Ok(())
}

/// Start autoplay timers for all `marco_sliders` decks in the current preview (if any).
///
/// Currently has no caller: it was the unified-backend counterpart of the
/// since-deleted `webkit6::sliders_play_all`, and nothing in the UI drives
/// deck autoplay yet. Kept as the entry point for when it does.
#[allow(dead_code)]
pub fn sliders_play_all(webview: &super::platform_webview::PlatformWebView) {
    let js = r#"(function(){try{if(window.MarcoCorePreview&&window.MarcoCorePreview.sliders&&typeof window.MarcoCorePreview.sliders.playAll==='function'){window.MarcoCorePreview.sliders.playAll();}}catch(e){console.error('sliders_play_all error',e);}})();"#;
    webview.evaluate_script(js);
}

/// Stop autoplay timers for all `marco_sliders` decks in the current preview (if any).
///
/// Uncalled for the same reason as [`sliders_play_all`].
#[allow(dead_code)]
pub fn sliders_pause_all(webview: &super::platform_webview::PlatformWebView) {
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
