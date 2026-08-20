// Viewer Component Module
//
// This module provides the preview rendering system for Marco's markdown editor.
// It handles HTML rendering, WebView management, and window layout control.
//
// # Architecture
//
// The preview runs on a single wry-based webview on all platforms (the
// gtk4-webkit6 fork of wry: GTK4/WebKit6 on Linux, WebView2 on Windows).
//
// - **platform_webview**: the cross-platform embedded WebView wrapper
// - **backend**: thin API veneer used by the renderer and dialogs
// - **renderer**: Markdown-to-HTML rendering coordinator
// - **find_engine**: JS-based find-in-preview engine (MarcoFind)
// - **layout_controller**: split pane and WebView location tracking
// - **javascript / css_utils**: JS + CSS utilities for the preview page
//
// The remaining `#[cfg(target_os = ...)]` splits below select the detached
// preview *window strategy* (Linux reparents the live webview; Windows
// rebuilds one from the recorded preview HTML) and Windows print/PDF COM
// plumbing — not a webview backend.

// For ApplicationWindow::application()
#[cfg(target_os = "linux")]
use gtk4::prelude::GtkWindowExt;

pub mod allocation_wait; // Cross-platform widget allocation/map polling helper
pub mod backend; // Cross-platform preview backend helpers (unified wry wrapper)
pub mod code_view_html; // Cross-platform HTML / JS builders for the code-view preview
#[cfg(target_os = "linux")]
pub mod detached_window_linux;
pub mod export_pipeline; // Unified Export & Print Pipeline (driven from main.rs export actions)
pub mod layout_controller; // Split controller + webview location tracking
pub mod loading_overlay; // Centered indeterminate loading bar overlayed on the preview
pub mod pagedjs; // Embedded paged.js polyfill for page view simulation
pub mod preview_state; // Cross-platform preview state snapshot/restore primitive (§14.3)
#[cfg(target_os = "linux")]
pub mod print_driver_linux; // Print dialog + PDF export driver (WebKit PrintOperation via escape hatch)
pub mod renderer; // Markdown rendering coordinator (cross-platform via `backend`)
#[cfg(target_os = "linux")]
pub mod reparenting; // WebView reparenting utilities (detached-window flow) // Detached preview window (reparents the live webview)

// Unified wry-based preview webview (Linux: GTK4/WebKit6 via the gtk4-webkit6
// fork; Windows: WebView2 child window) and helpers.
pub mod find_engine; // JS-based find-in-preview engine (MarcoFind)
pub mod platform_webview;
pub mod preview_helpers; // Shared preview helpers (latest-HTML cache, external URI opener, code viewer) // Cross-platform embedded WebView wrapper

// Windows-only: wry/WebView2 specifics
#[cfg(target_os = "windows")]
pub mod detached_window_windows;
#[cfg(target_os = "windows")]
pub mod print_driver_windows; // Print dialog + native PrintToPdf export (WebView2 COM) // Detached preview window using wry

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
pub type PreviewWindowType = crate::components::viewer::detached_window_linux::PreviewWindow;
#[cfg(target_os = "windows")]
pub type PreviewWindowType = crate::components::viewer::detached_window_windows::PreviewWindow;

pub fn open_preview_in_separate_window(
    parent_window: &gtk4::ApplicationWindow,
    webview_opt: Option<&crate::components::viewer::preview_types::PlatformWebView>,
) -> Option<PreviewWindowType> {
    #[cfg(target_os = "linux")]
    {
        use crate::components::viewer::detached_window_linux::PreviewWindow;
        if let Some(app) = parent_window.application() {
            let pw = PreviewWindow::new(parent_window, &app);
            if let Some(webview) = webview_opt {
                // Reparent the unified wrapper's widget into the new window.
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
        use crate::components::viewer::detached_window_windows::PreviewWindow;
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

pub mod css_utils; // CSS and HTML formatting utilities (cross-platform)
pub mod javascript; // JavaScript utilities (cross-platform)
