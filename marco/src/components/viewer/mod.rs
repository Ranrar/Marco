//! Viewer Component Module
//!
//! This module provides the preview rendering system for Marco's markdown editor.
//! It handles HTML rendering, WebView management, and window layout control.
//!
//! # Platform Support
//!
//! - **Linux**: Full support using WebKit6 (GTK4-native WebKit)
//! - **Windows**: Not yet implemented (future: wry/WebView2)
//!
//! # Architecture
//!
//! - **webkit6**: Linux-specific WebView implementation (HTML rendering, JS injection)
//! - **preview**: Markdown-to-HTML rendering coordinator
//! - **previewwindow**: Separate window for split-view mode
//! - **switcher**: WebView reparenting utilities
//! - **controller**: Split pane and WebView location tracking
//! - **syntax_highlighter**: Code block syntax highlighting
//! - **webview_js**: JavaScript utilities for scroll and interactivity
//! - **webview_utils**: CSS utilities for scrollbars and formatting
//!
//! # Future Windows Support
//!
//! When Windows support is added, it will use the wry crate (Chromium-based WebView2)
//! instead of WebKit6. The interface will remain similar but with platform-specific
//! implementations using `#[cfg(target_os = "linux")]` and `#[cfg(windows)]`.

pub mod layout_controller; // Split controller + webview location tracking
#[cfg(target_os = "linux")]
pub mod renderer; // Markdown rendering coordinator (Linux: WebKit6)
#[cfg(target_os = "linux")]
pub mod detached_window; // Separate preview window (Linux: WebKit6)
#[cfg(target_os = "linux")]
pub mod reparenting; // WebView reparenting utilities (Linux: GTK4/WebKit6)

// Windows: wry-based detached preview and helpers
#[cfg(windows)]
pub mod wry; // Windows (wry/WebView2) minimal parity helpers
#[cfg(windows)]
pub mod wry_detached_window; // Detached preview window using wry
#[cfg(windows)]
pub mod wry_platform_webview; // Windows: embedded child WebView

pub mod preview_types; // View mode enum (cross-platform)

/// Open the preview in a new detached window. Implemented per-platform below.
///
/// - On Linux: creates a `PreviewWindow` and re-parents the existing WebView
///   into it (reparenting preserves state).
/// - On Windows: creates a `PreviewWindow` that uses `wry` and attaches the
///   inline `PlatformWebView` as a child if present; otherwise it will load
///   the most recently saved HTML preview content.
use std::option::Option;

// Platform-specific preview window type alias
#[cfg(target_os = "linux")]
pub type PreviewWindowType = crate::components::viewer::detached_window::PreviewWindow;
#[cfg(windows)]
pub type PreviewWindowType = crate::components::viewer::wry_detached_window::PreviewWindow;

pub fn open_preview_in_separate_window(parent_window: &gtk4::ApplicationWindow, webview_opt: Option<&crate::components::viewer::preview_types::PlatformWebView>) -> Option<PreviewWindowType> {
    #[cfg(target_os = "linux")]
    {
        use crate::components::viewer::detached_window::PreviewWindow;
        if let Some(app) = parent_window.application() {
            let pw = PreviewWindow::new(parent_window, &app);
            if let Some(webview) = webview_opt {
                // On Linux the webview is a webkit6::WebView widget
                // We need to attach the actual widget to the preview window
                pw.attach_webview(webview);
            }
            pw.show();
            return Some(pw);
        } else {
            log::warn!("open_preview_in_separate_window: parent window has no Application; cannot create preview window");
            return None;
        }
    }

    #[cfg(windows)]
    {
        use crate::components::viewer::wry_detached_window::PreviewWindow;
        let pw = PreviewWindow::new(parent_window);
        if let Some(webview) = webview_opt {
            // On Windows the PlatformWebView exposes `.widget()` to get gtk4::Widget
            pw.attach_webview(Some(&webview.widget()));
        }
        pw.show();
        return Some(pw);
    }

    #[allow(unreachable_code)]
    None
}

#[cfg(target_os = "linux")]
pub mod webkit6; // WebKit6 WebView implementation (Linux-only)
pub mod javascript; // JavaScript utilities (cross-platform)
pub mod css_utils; // CSS and HTML formatting utilities (cross-platform)
