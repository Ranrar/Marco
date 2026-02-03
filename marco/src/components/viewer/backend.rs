//! Cross-platform preview backend helpers.
//!
//! This module provides a thin abstraction over the platform-specific preview
//! implementations so higher-level code can avoid calling `webkit6`/`wry`
//! modules directly.

use std::path::Path;

/// Platform preview webview type.
///
/// - Linux: `webkit6::WebView`
/// - Windows: `wry_platform_webview::PlatformWebView`
#[cfg(target_os = "linux")]
pub type PreviewWebView = webkit6::WebView;

#[cfg(target_os = "windows")]
pub type PreviewWebView = crate::components::viewer::wry_platform_webview::PlatformWebView;

pub fn wrap_html_document(
    body: &str,
    css: &str,
    theme_mode: &str,
    background_color: Option<&str>,
) -> String {
    core::render::wrap_preview_html_document(body, css, theme_mode, background_color)
}

#[cfg(target_os = "linux")]
pub fn generate_base_uri_from_path<P: AsRef<Path>>(document_path: P) -> Option<String> {
    crate::components::viewer::webkit6::generate_base_uri_from_path(document_path)
}

#[cfg(target_os = "windows")]
pub fn generate_base_uri_from_path<P: AsRef<Path>>(document_path: P) -> Option<String> {
    crate::components::viewer::wry::generate_base_uri_from_path(document_path)
}

#[cfg(target_os = "linux")]
pub fn load_html_when_ready(webview: &PreviewWebView, html: String, base_uri: Option<String>) {
    crate::components::viewer::webkit6::load_html_when_ready(webview, html, base_uri)
}

#[cfg(target_os = "windows")]
pub fn load_html_when_ready(webview: &PreviewWebView, html: String, base_uri: Option<String>) {
    webview.load_html_with_base(&html, base_uri.as_deref());
}

#[cfg(target_os = "linux")]
pub fn update_html_content_smooth(webview: &PreviewWebView, content: &str) {
    crate::components::viewer::webkit6::update_html_content_smooth(webview, content)
}

#[cfg(target_os = "windows")]
pub fn update_html_content_smooth(webview: &PreviewWebView, content: &str) {
    // WebView2 doesn't have the same in-page update JS yet; re-load for now.
    // Preserve relative resource resolution by reusing the last base URI.
    let base_uri = crate::components::viewer::wry::get_latest_preview_base_uri();
    webview.load_html_with_base(content, base_uri.as_deref());
}
