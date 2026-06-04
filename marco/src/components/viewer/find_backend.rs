//! Cross-platform find-in-preview abstraction (§14.1 / Step 6b of
//! `webkit6_wry_parity_audit.md`).
//!
//! # Why a trait
//!
//! On Linux, WebKitGTK exposes a native `WebView::find_controller()` that
//! drives highlight, count, and forward/backward navigation entirely inside
//! the WebView. On Windows, WebView2 has no equivalent — the
//! [`super::wry_find`] module provides a JS-based engine that mirrors the
//! same behaviour (CSS Custom Highlight API + `marco_find:` IPC reporting).
//!
//! The [`FindBackend`] trait gives the search subsystem a single, uniform
//! API across both platforms so future search-window wiring can be written
//! once.
//!
//! # Wiring status
//!
//! * **Windows** — [`WryFindBackend`] is implemented and delegates to
//!   [`super::wry_find`] plus
//!   [`super::wry_platform_webview::PlatformWebView::set_find_report_callback`].
//! * **Linux** — [`WebKit6FindBackend`] is a thin marker today; the actual
//!   `webkit6::FindController` wiring is intentionally deferred until the
//!   search-window UI starts driving find-in-preview. The trait surface is
//!   intentionally identical on both platforms so the eventual Linux impl
//!   is a drop-in.
//!
//! # Lifetime and threading
//!
//! All trait methods take `&self` and are designed to be called from the GTK
//! main thread. Callbacks are stored as `Rc<dyn Fn(...)>` internally on the
//! Windows side (matching `PlatformWebView`'s existing callback model); the
//! trait itself uses a boxed `Fn` so future implementations are free to use
//! any interior-mutability strategy that suits their backend.

// This module is a deliberate scaffold: the trait + both backends are
// public so the eventual search-window wiring (§14.1, "Windows search UI"
// follow-up) can plug in without further refactors. Until then, nothing
// constructs the backends, so silence the expected dead-code warnings at
// module scope rather than peppering every item with attributes.
#![allow(dead_code)]

/// User-facing search options shared between backends.
///
/// Mirrors the fields the marco search UI actually exposes today. Regex /
/// Markdown-only filtering happen in the editor buffer (sourceview
/// `SearchContext`), not in the preview pane, and are deliberately out of
/// scope here.
#[derive(Debug, Clone, Copy, Default)]
pub struct FindOptions {
    /// Match case exactly. When `false`, the comparison is case-insensitive
    /// (Windows lower-cases both sides; Linux passes
    /// `webkit6::FindOptions::CASE_INSENSITIVE`).
    pub case_sensitive: bool,
    /// Require word boundaries on both sides of every match.
    pub whole_word: bool,
}

/// Result snapshot delivered to the host after every `search`, `next`, or
/// `prev` call.
///
/// On Windows the report is parsed from a `marco_find:` IPC payload. On
/// Linux the report is assembled from `WebKitFindController`'s
/// `connect_counted_matches` / `connect_found_text` signals.
#[derive(Debug, Clone, Copy, Default)]
pub struct FindReport {
    /// Total number of matches currently highlighted (0 when the document
    /// contains no match or immediately after `clear`).
    pub total: usize,
    /// 1-based index of the "active" (scrolled-into-view) match, or 0 when
    /// there is no active match.
    pub active: usize,
}

/// Boxed callback signature for find-result reports.
pub type FindReportCallback = Box<dyn Fn(FindReport) + 'static>;

/// Cross-platform find-in-preview engine.
///
/// Every method is a no-op when the underlying WebView is not yet ready;
/// implementations are responsible for any required deferral or queueing.
pub trait FindBackend {
    /// One-time bootstrap. On Windows this injects the `MarcoFind` JS
    /// engine into the current document; on Linux it is a no-op
    /// (`webkit6::FindController` is always available on a live WebView).
    ///
    /// Safe to call multiple times — implementations must be idempotent so
    /// the host can re-install after every preview reload.
    fn install(&self);

    /// Begin (or restart) a search for `query` with `opts`. Replaces any
    /// previously highlighted results.
    fn search(&self, query: &str, opts: FindOptions);

    /// Advance to the next match, wrapping at the end of the document.
    fn next(&self);

    /// Step back to the previous match, wrapping at the start of the
    /// document.
    fn prev(&self);

    /// Drop all highlights and reset internal find state.
    fn clear(&self);

    /// Install (or replace) the report callback. The callback is invoked on
    /// the GTK main thread after every state-changing call (`search`,
    /// `next`, `prev`, `clear`). At most one callback is active at a time.
    fn set_report_callback(&self, cb: FindReportCallback);
}

// -------------------------------------------------------------------------
// Windows implementation
// -------------------------------------------------------------------------

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::{FindBackend, FindOptions, FindReport, FindReportCallback};
    use crate::components::viewer::wry_find;
    use crate::components::viewer::wry_platform_webview::PlatformWebView;

    /// `FindBackend` implementation that drives the JS engine in
    /// [`super::wry_find`] through a single shared [`PlatformWebView`].
    pub struct WryFindBackend {
        pv: PlatformWebView,
    }

    impl WryFindBackend {
        /// Wrap an existing `PlatformWebView` (the preview pane). The
        /// caller is responsible for invoking [`FindBackend::install`] once
        /// the WebView has fired `marco_zoom:ready`.
        pub fn new(pv: PlatformWebView) -> Self {
            Self { pv }
        }
    }

    impl FindBackend for WryFindBackend {
        fn install(&self) {
            wry_find::install(&self.pv);
        }

        fn search(&self, query: &str, opts: FindOptions) {
            wry_find::search(
                &self.pv,
                query,
                wry_find::FindOptions {
                    case_sensitive: opts.case_sensitive,
                    whole_word: opts.whole_word,
                },
            );
        }

        fn next(&self) {
            wry_find::next(&self.pv);
        }

        fn prev(&self) {
            wry_find::prev(&self.pv);
        }

        fn clear(&self) {
            wry_find::clear(&self.pv);
        }

        fn set_report_callback(&self, cb: FindReportCallback) {
            // Adapt the trait's report type (defined in this module) to
            // the `wry_find::FindReport` posted by the IPC arm. Both have
            // identical {total, active} shape — keep the conversion
            // explicit so a future field divergence is a compile error.
            self.pv.set_find_report_callback(move |native| {
                cb(FindReport {
                    total: native.total,
                    active: native.active,
                });
            });
        }
    }
}

#[cfg(target_os = "windows")]
#[allow(unused_imports)] // Re-exported for the future search-window wiring.
pub use windows_impl::WryFindBackend;

// -------------------------------------------------------------------------
// Linux implementation (placeholder)
// -------------------------------------------------------------------------

#[cfg(target_os = "linux")]
mod linux_impl {
    use super::{FindBackend, FindOptions, FindReport, FindReportCallback};
    use std::cell::RefCell;
    use std::rc::Rc;
    use webkit6::WebView;

    /// `FindBackend` placeholder for Linux. Holds a clone of the preview
    /// `WebView` and stores the report callback, but does not yet drive
    /// `webkit6::FindController` — that wiring lands when the search-window
    /// UI starts exposing find-in-preview (tracked in §14.1 of the parity
    /// audit). All methods are intentionally no-ops so the trait can be
    /// constructed and stored today without changing user-visible behaviour.
    pub struct WebKit6FindBackend {
        _webview: WebView,
        callback: Rc<RefCell<Option<FindReportCallback>>>,
    }

    impl WebKit6FindBackend {
        pub fn new(webview: WebView) -> Self {
            Self {
                _webview: webview,
                callback: Rc::new(RefCell::new(None)),
            }
        }
    }

    impl FindBackend for WebKit6FindBackend {
        fn install(&self) {
            // No-op: webkit6's FindController is always available on a live
            // WebView; the upcoming UI wiring will obtain it on demand.
        }

        fn search(&self, _query: &str, _opts: FindOptions) {
            // TODO(§14.1): translate `opts` into
            // `webkit6::FindOptions::CASE_INSENSITIVE | AT_WORD_STARTS` and
            // call `find_controller.search(query, opts, max_match_count)`.
            // Emit a zero report so the trait contract is honoured.
            if let Some(cb) = self.callback.borrow().as_ref() {
                cb(FindReport::default());
            }
        }

        fn next(&self) {
            // TODO(§14.1): call `find_controller.search_next()`.
        }

        fn prev(&self) {
            // TODO(§14.1): call `find_controller.search_previous()`.
        }

        fn clear(&self) {
            // TODO(§14.1): call `find_controller.search_finish()`.
            if let Some(cb) = self.callback.borrow().as_ref() {
                cb(FindReport::default());
            }
        }

        fn set_report_callback(&self, cb: FindReportCallback) {
            *self.callback.borrow_mut() = Some(cb);
        }
    }
}

#[cfg(target_os = "linux")]
#[allow(unused_imports)] // Re-exported for the future search-window wiring.
pub use linux_impl::WebKit6FindBackend;

// -------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_find_options_default() {
        let opts = FindOptions::default();
        assert!(!opts.case_sensitive);
        assert!(!opts.whole_word);
    }

    #[test]
    fn smoke_find_report_default() {
        let report = FindReport::default();
        assert_eq!(report.total, 0);
        assert_eq!(report.active, 0);
    }

    /// The trait must be object-safe so the search subsystem can hold a
    /// `Box<dyn FindBackend>` keyed by the active preview.
    #[test]
    fn smoke_find_backend_is_object_safe() {
        fn _assert_object_safe(_: &dyn FindBackend) {}
    }
}
