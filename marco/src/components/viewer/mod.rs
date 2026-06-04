// For ApplicationWindow::application()
#[cfg(target_os = "linux")]
use gtk4::prelude::GtkWindowExt;
// Viewer Component Module
//
// This module provides the preview rendering system for Marco's markdown editor.
// It handles HTML rendering, WebView management, and window layout control.
//
// # Platform Support
//
// - **Linux**: Full support using WebKit6 (GTK4-native WebKit)
// - **Windows**: Not yet implemented (future: wry/WebView2)
//
// # Architecture
//
// - **webkit6**: Linux-specific WebView implementation (HTML rendering, JS injection)
// - **preview**: Markdown-to-HTML rendering coordinator
// - **previewwindow**: Separate window for split-view mode
// - **switcher**: WebView reparenting utilities
// - **controller**: Split pane and WebView location tracking
// - **syntax_highlighter**: Code block syntax highlighting
// - **webview_js**: JavaScript utilities for scroll and interactivity
// - **webview_utils**: CSS utilities for scrollbars and formatting
//
// # Future Windows Support
//
// When Windows support is added, it will use the wry crate (Chromium-based WebView2)
// instead of WebKit6. The interface will remain similar but with platform-specific
// implementations using `#[cfg(target_os = "linux")]` and `#[cfg(target_os = "windows")]`.

pub mod allocation_wait; // Cross-platform widget allocation/map polling helper
pub mod backend; // Cross-platform preview backend helpers (Linux: WebKit6, Windows: wry)
pub mod code_view_html; // Cross-platform HTML / JS builders for the code-view preview
pub mod export_pipeline; // Unified Export & Print Pipeline (skeleton — Phase 1, no callers)
pub mod layout_controller; // Split controller + webview location tracking
pub mod loading_overlay; // Centered indeterminate loading bar overlayed on the preview
pub mod pagedjs; // Embedded paged.js polyfill for page view simulation
pub mod preview_state; // Cross-platform preview state snapshot/restore primitive (§14.3)
#[cfg(target_os = "linux")]
pub mod print_driver; // Print dialog and PDF export driver (Linux: WebKit6)
pub mod renderer; // Markdown rendering coordinator (cross-platform via `backend`)
#[cfg(target_os = "linux")]
pub mod reparenting;
#[cfg(target_os = "linux")]
pub mod webkit6_detached_window; // Separate preview window (Linux: WebKit6) // WebView reparenting utilities (Linux: GTK4/WebKit6)

// Windows: wry-based detached preview and helpers
#[cfg(target_os = "windows")]
pub mod print_driver_windows;
#[cfg(target_os = "windows")]
pub mod wry; // Windows (wry/WebView2) minimal parity helpers
#[cfg(target_os = "windows")]
pub mod wry_detached_window; // Detached preview window using wry
#[cfg(target_os = "windows")]
pub mod wry_find; // Windows: JS-based find-in-preview engine (parity for webkit6 FindController)
#[cfg(target_os = "windows")]
pub mod wry_platform_webview; // Windows: embedded child WebView // Windows print driver (wry/WebView2)
#[cfg(target_os = "windows")]
pub mod wry_print_to_pdf; // Native WebView2 PrintToPdf (replaces headless Chromium)

pub mod find_backend; // Cross-platform find-in-preview trait (§14.1, Step 6b)
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
pub type PreviewWindowType = crate::components::viewer::webkit6_detached_window::PreviewWindow;
#[cfg(target_os = "windows")]
pub type PreviewWindowType = crate::components::viewer::wry_detached_window::PreviewWindow;

pub fn open_preview_in_separate_window(
    parent_window: &gtk4::ApplicationWindow,
    webview_opt: Option<&crate::components::viewer::preview_types::PlatformWebView>,
) -> Option<PreviewWindowType> {
    #[cfg(target_os = "linux")]
    {
        use crate::components::viewer::webkit6_detached_window::PreviewWindow;
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

    #[cfg(target_os = "windows")]
    {
        use crate::components::viewer::wry_detached_window::PreviewWindow;
        let pw = PreviewWindow::new(parent_window);
        if let Some(webview) = webview_opt {
            // Snapshot user-visible state from the editor's live WebView
            // before the detached window builds its own (see §14.3 of the
            // parity audit). The reply is auto-stashed in
            // `preview_state::LATEST_PREVIEW_STATE` and the detached
            // window's `set_ready_callback` will restore it post-load.
            webview.request_state_snapshot();
            // On Windows the detached window creates its own PlatformWebView
            // internally; the editor's WebView cannot be reparented (§14.3).
            pw.load_preview_content();
        }
        pw.show();
        return Some(pw);
    }

    #[allow(unreachable_code)]
    None
}

pub mod css_utils;
pub mod javascript; // JavaScript utilities (cross-platform)
#[cfg(target_os = "linux")]
pub mod webkit6; // WebKit6 WebView implementation (Linux-only) // CSS and HTML formatting utilities (cross-platform)
