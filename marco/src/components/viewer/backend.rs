//! Cross-platform preview backend helpers.
//!
//! Thin veneer over the unified wry-based [`PreviewWebView`] so higher-level
//! code has one stable API for loading, patching, and scripting the preview.

use std::path::Path;

/// Platform preview webview type — the unified wry-based wrapper
/// (Linux: GTK4/WebKit6 via the gtk4-webkit6 wry fork; Windows: WebView2).
pub type PreviewWebView = crate::components::viewer::platform_webview::PlatformWebView;

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

/// Variant of [`wrap_html_document`] that injects paged.js for true CSS Paged Media simulation.
///
/// Uses [`marco_core::render::wrap_preview_html_document_paged`] under the hood, then applies the
/// same `dir="ltr"` fixup so the WebKit viewport scrollbar stays consistent.
///
/// **Important**: Content updates in page view mode require a full HTML reload — do **not**
/// use `update_html_content_smooth` after this.
pub fn wrap_html_document_paged(
    body: &str,
    css: &str,
    theme_mode: &str,
    background_color: Option<&str>,
    page_opts: &marco_core::render::PageViewOptions<'_>,
) -> String {
    let html = marco_core::render::wrap_preview_html_document_paged(
        body,
        css,
        theme_mode,
        background_color,
        page_opts,
    );
    let html = html.replacen("<html ", "<html dir=\"ltr\" ", 1);
    if crate::logic::rtl::is_rtl_global() {
        html.replacen("<body>", "<body dir=\"rtl\">", 1)
    } else {
        html
    }
}

pub fn generate_base_uri_from_path<P: AsRef<Path>>(document_path: P) -> Option<String> {
    crate::components::viewer::preview_helpers::generate_base_uri_from_path(document_path)
}

/// Load a full HTML document into the preview. Creation/loading is deferred
/// internally until the webview's container is mapped and allocated.
pub fn load_html_when_ready(webview: &PreviewWebView, html: String, base_uri: Option<String>) {
    webview.load_html_with_base(&html, base_uri.as_deref());
}

/// Record the most recent full preview document (and its base URI) so the
/// detached preview window can rebuild content without a live webview
/// reference. Call whenever a full preview document is loaded.
pub fn record_latest_preview(html: &str, base_uri: Option<&str>) {
    if let Ok(mut guard) = crate::components::viewer::preview_helpers::LATEST_PREVIEW_HTML
        .get_or_init(|| std::sync::Mutex::new(String::new()))
        .lock()
    {
        *guard = html.to_string();
    }
    crate::components::viewer::preview_helpers::set_latest_preview_base_uri(
        base_uri.map(str::to_string),
    );
    crate::components::viewer::preview_helpers::notify_preview_refreshed();
}

/// Patch the live preview's `mc-content-container` in place via the JS
/// bridge, preserving scroll position (no full reload, no white flash).
pub fn update_html_content_smooth(webview: &PreviewWebView, content: &str) {
    webview.update_html_content_smooth(content);
}

/// Evaluate a JavaScript snippet in the live preview webview.
/// Used to update page-level attributes (e.g. `dir`) without a full reload.
pub fn evaluate_javascript(webview: &PreviewWebView, js: &str) {
    webview.evaluate_script(js);
}
