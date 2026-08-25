//! Preview zoom state for Polo's viewer.
//!
//! Deliberately session-only: the zoom level lives in a thread-local for the
//! lifetime of the running process and is never written to `settings.ron`.
//! Polo opens a different document on every launch (unlike Marco's long-lived
//! editing session), so a zoom level that survived a restart would carry over
//! from whatever was last read into an unrelated document. If that turns out
//! to be wanted after all, persistence can be added later as its own step —
//! see the shared `preview_zoom` field already on
//! `marco_shared::logic::swanson::LayoutSettings`, which Marco uses.
//!
//! Mirrors `marco/src/components/editor/editor_manager.rs`'s zoom state, but
//! without a "registered primary webview" thread-local registry — Polo has
//! exactly one `PlatformWebView` per process and every call site already has
//! a reference to it, so it's passed in directly instead.

use super::platform_webview::PlatformWebView;
use std::cell::Cell;

/// Zoom step increment/decrement per action (button click, keyboard shortcut,
/// or Ctrl+wheel notch).
pub const ZOOM_STEP: f64 = 0.1;
/// Minimum allowed zoom level.
pub const ZOOM_MIN: f64 = 0.5;
/// Maximum allowed zoom level.
pub const ZOOM_MAX: f64 = 3.0;
/// Default zoom level.
pub const ZOOM_DEFAULT: f64 = 1.0;

thread_local! {
    static PREVIEW_ZOOM: Cell<f64> = const { Cell::new(ZOOM_DEFAULT) };
}

/// Get the current in-session preview zoom level.
pub fn get_preview_zoom() -> f64 {
    PREVIEW_ZOOM.with(|c| c.get())
}

/// Set the current preview zoom level (clamped to `ZOOM_MIN..=ZOOM_MAX`) and
/// apply it to `webview` immediately.
pub fn set_preview_zoom(zoom: f64, webview: &PlatformWebView) {
    let clamped = zoom.clamp(ZOOM_MIN, ZOOM_MAX);
    PREVIEW_ZOOM.with(|c| c.set(clamped));

    // `__poloApplyZoom` (defined in `zoom_bar::html`) scales `<body>`,
    // relocates the zoom bar so it isn't scaled along with the content, and
    // updates the percent label. Fall back to setting `style.zoom` directly
    // in case the bar's script hasn't initialized yet.
    let pct = (clamped * 100.0).round() as i32;
    let js = format!(
        "if (typeof window.__poloApplyZoom === 'function') {{ \
             window.__poloApplyZoom({zoom}); \
         }} else {{ \
             document.body.style.zoom = '{zoom}'; \
             if (typeof window.__poloSetZoomLabel === 'function') {{ \
                 window.__poloSetZoomLabel({pct}); \
             }} \
         }}",
        zoom = clamped,
        pct = pct,
    );
    webview.evaluate_script(&js);
}

/// Re-apply the current in-session zoom level to `webview`.
///
/// Call this after every document load: `document.body.style.zoom` lives in
/// the DOM of the currently-loaded page, so it resets on every navigation.
/// Unlike Marco (where most preview refreshes are incremental DOM patches
/// that never touch `style.zoom`), every render in Polo is a full navigation
/// — opening a file, F5 reload, a theme switch, or following a local link —
/// so this needs to run unconditionally on every `load-finished` signal.
pub fn reapply(webview: &PlatformWebView) {
    set_preview_zoom(get_preview_zoom(), webview);
}

/// Apply one zoom step: `"in"` (+`ZOOM_STEP`), `"out"` (-`ZOOM_STEP`), or
/// `"reset"` (back to `ZOOM_DEFAULT`). Unknown actions are ignored.
///
/// Shared by the `polo_zoom:` IPC handler (zoom-bar buttons, Ctrl+wheel) and
/// the `win.polo-zoom-*` keyboard-shortcut GActions, so the delta math lives
/// in exactly one place.
pub fn step(action: &str, webview: &PlatformWebView) {
    let current = get_preview_zoom();
    let new_zoom = match action {
        "in" => current + ZOOM_STEP,
        "out" => current - ZOOM_STEP,
        "reset" => ZOOM_DEFAULT,
        _ => return,
    };
    set_preview_zoom(new_zoom, webview);
}
