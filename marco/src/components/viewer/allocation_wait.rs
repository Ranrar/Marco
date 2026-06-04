//! Cross-platform GTK widget allocation/map waiting helpers.
//!
//! Both the WebKit6 (Linux) and wry (Windows) preview WebViews must be loaded
//! against a widget that has been:
//!
//! 1. **Mapped** — visible in the widget tree (so it has a backing GDK surface).
//! 2. **Allocated** — given a non-trivial size (so the WebView contents render
//!    against the correct viewport).
//!
//! Calling `webkit6::WebView::load_html` or `wry::WebView::load_url` too early
//! produces GTK warnings ("Trying to snapshot ... without a current allocation")
//! on Linux, and renders against a zero-sized child Win32 window on Windows.
//!
//! Both backends previously implemented this polling loop independently — Linux
//! had a 16 ms × 300 retry loop, Windows had a single fallback to `max(100)`.
//! This module unifies both into a single GLib-based helper that is generic
//! over any `gtk4::Widget`.
//!
//! # Implementation notes
//!
//! * Uses `glib::timeout_add_local` (16 ms ≈ 60 fps) rather than an idle loop;
//!   idle sources can iterate thousands of times per second on Windows which
//!   makes the retry counter effectively meaningless.
//! * If the widget is **not mapped**, defers via `connect_map` rather than
//!   wasting the retry budget while the widget is legitimately hidden (e.g.
//!   in a `gtk4::Stack` that is currently showing a different page).
//! * If the widget is dropped while waiting, the timeout is automatically
//!   cancelled via a weak reference, so closures never run against a stale
//!   widget.

use gtk4::glib;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

/// Default polling cadence — matches GTK4's 60 fps tick callback rate.
pub const POLL_INTERVAL: Duration = Duration::from_millis(16);

/// Default maximum retries — 300 × 16 ms ≈ 4.8 seconds.
pub const DEFAULT_MAX_RETRIES: u32 = 300;

type OnceFn = Box<dyn FnOnce()>;

/// Run `f` once the widget is mapped (visible in the widget tree).
///
/// If the widget is already mapped, `f` runs synchronously before this
/// function returns. Otherwise, a one-shot `map` signal handler is connected
/// that runs `f` on the next map and immediately disconnects itself.
///
/// # Use cases
///
/// - Loading HTML into a WebView that is currently hidden in a `gtk4::Stack`.
/// - Updating content in tabs that are not currently visible.
///
/// # Panic safety
///
/// The closure is stored in a `RefCell<Option<_>>` and taken on first signal
/// fire, so re-entrant map signals (rare but possible during GTK shutdown)
/// will not double-invoke `f`.
pub fn run_once_when_mapped<W>(widget: &W, f: impl FnOnce() + 'static)
where
    W: IsA<gtk4::Widget> + Clone + 'static,
{
    if widget.is_mapped() {
        f();
        return;
    }

    let handler_id: Rc<RefCell<Option<glib::SignalHandlerId>>> = Rc::new(RefCell::new(None));
    let f_cell: Rc<RefCell<Option<OnceFn>>> = Rc::new(RefCell::new(Some(Box::new(f))));

    let widget_for_disconnect = widget.clone();
    let handler_id_cb = handler_id.clone();
    let f_cell_cb = f_cell.clone();

    let id = widget.connect_map(move |_| {
        if let Some(id) = handler_id_cb.borrow_mut().take() {
            widget_for_disconnect.disconnect(id);
        }
        if let Some(callback) = f_cell_cb.borrow_mut().take() {
            callback();
        }
    });

    *handler_id.borrow_mut() = Some(id);
}

/// Run `f` once the widget is both mapped and allocated a non-trivial size.
///
/// The closure is invoked on the GTK main loop as soon as all of the
/// following hold:
///
/// 1. The widget has been mapped (deferred via [`run_once_when_mapped`] if not).
/// 2. The widget is realized (has a backing GDK surface).
/// 3. `allocated_width()` and `allocated_height()` are both `> 1`.
///
/// If `max_retries` polls elapse before the allocation is ready, the closure
/// is **not** called and a debug log entry is emitted. Use [`DEFAULT_MAX_RETRIES`]
/// for the standard ~4.8 second budget.
///
/// If the widget is dropped while waiting (e.g. the user closes the tab),
/// the timeout is automatically cancelled via the weak reference and the
/// closure is silently dropped.
pub fn run_when_allocated<W>(widget: &W, max_retries: u32, f: impl FnOnce() + 'static)
where
    W: IsA<gtk4::Widget> + Clone + 'static,
{
    // If not mapped, defer until map then re-enter.
    if !widget.is_mapped() {
        let widget_clone = widget.clone();
        run_once_when_mapped(widget, move || {
            run_when_allocated(&widget_clone, max_retries, f);
        });
        return;
    }

    // Fast path: already allocated. Run synchronously to avoid a 16 ms delay
    // on every refresh in the common case.
    if widget.is_realized() && widget.allocated_width() > 1 && widget.allocated_height() > 1 {
        f();
        return;
    }

    // Slow path: poll on a timer.
    let widget_weak = widget.downgrade();
    let tries = Cell::new(0u32);
    let f_cell: Rc<RefCell<Option<OnceFn>>> = Rc::new(RefCell::new(Some(Box::new(f))));

    glib::timeout_add_local(POLL_INTERVAL, move || {
        let Some(widget) = widget_weak.upgrade() else {
            // Widget dropped while we were waiting — silently abort.
            return glib::ControlFlow::Break;
        };

        let t = tries.get();
        if t >= max_retries {
            log::debug!(
                "[allocation_wait] giving up after {} retries ({:?} total)",
                t,
                POLL_INTERVAL * t
            );
            return glib::ControlFlow::Break;
        }
        tries.set(t + 1);

        if !widget.is_realized() {
            return glib::ControlFlow::Continue;
        }
        if widget.allocated_width() <= 1 || widget.allocated_height() <= 1 {
            return glib::ControlFlow::Continue;
        }

        if let Some(callback) = f_cell.borrow_mut().take() {
            callback();
        }
        glib::ControlFlow::Break
    });
}
