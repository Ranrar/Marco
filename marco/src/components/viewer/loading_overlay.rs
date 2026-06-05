//! Indeterminate loading-progress overlay shown on top of the preview WebView.
//!
//! Wraps an arbitrary GTK widget in a [`gtk4::Overlay`] and centers a
//! [`gtk4::ProgressBar`] above it.  A small amount of app-scoped CSS
//! (`FRAME_CSS`) is injected so the frame and progress fill match Marco's
//! light/dark theme classes automatically.
//!
//! The overlay is driven from a single thread-local handle: call
//! [`set_global`] once when the preview widget is constructed, then
//! [`show`]/[`hide`] from anywhere on the main thread to start/stop the
//! pulse animation.

use gtk4::{glib, prelude::*, Overlay, ProgressBar};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

/// Pulse update interval for the indeterminate animation.
const PULSE_INTERVAL: Duration = Duration::from_millis(80);

/// CSS for the framed container around the progress bar.
///
/// Scoped under the same `.marco-theme-light` / `.marco-theme-dark` classes
/// that the rest of the app uses for light/dark switching, so the overlay
/// flips when the user changes themes.  The progress bar fill uses Marco's
/// blue accent (`#0066cc` light, `#4f8cff` dark) to match toolbar hover
/// borders.
const FRAME_CSS: &str = "
/* ---- Light theme ---- */
.marco-theme-light .marco-loading-frame {
    background-color: #e8ecef;
    border: 1px solid #ccc;
    border-radius: 8px;
    padding: 14px 18px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}
.marco-theme-light .marco-loading-frame label.marco-loading-label {
    color: #2c3e50;
    font-weight: 600;
    margin-bottom: 6px;
}
.marco-theme-light .marco-loading-frame progressbar > trough {
    background-color: #ffffff;
    border: 1px solid #ccc;
    border-radius: 4px;
    min-height: 10px;
}
.marco-theme-light .marco-loading-frame progressbar > trough > progress {
    background-color: #0066cc;
    border: 1px solid #0066cc;
    border-radius: 4px;
    min-height: 10px;
}

/* ---- Dark theme ---- */
.marco-theme-dark .marco-loading-frame {
    background-color: #23272e;
    border: 1px solid #444;
    border-radius: 8px;
    padding: 14px 18px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
}
.marco-theme-dark .marco-loading-frame label.marco-loading-label {
    color: #e0e0e0;
    font-weight: 600;
    margin-bottom: 6px;
}
.marco-theme-dark .marco-loading-frame progressbar > trough {
    background-color: #1e1e1e;
    border: 1px solid #444;
    border-radius: 4px;
    min-height: 10px;
}
.marco-theme-dark .marco-loading-frame progressbar > trough > progress {
    background-color: #4f8cff;
    border: 1px solid #4f8cff;
    border-radius: 4px;
    min-height: 10px;
}
";

/// Install the frame CSS once per GDK display.
fn ensure_frame_css() {
    thread_local! {
        static INSTALLED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }
    INSTALLED.with(|flag| {
        if flag.get() {
            return;
        }
        if let Some(display) = gtk4::gdk::Display::default() {
            let provider = gtk4::CssProvider::new();
            provider.load_from_data(FRAME_CSS);
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
            flag.set(true);
        }
    });
}

/// A GTK [`Overlay`] holding the preview widget with a centered, hidden
/// indeterminate [`ProgressBar`] floating above it.
pub struct LoadingOverlay {
    overlay: Overlay,
    frame: gtk4::Box,
    progress: ProgressBar,
    pulse_id: RefCell<Option<glib::SourceId>>,
    /// On Windows the wry HWND sits on top of all GTK painting, so the GTK
    /// progress frame is never visible while the WebView is in its normal
    /// position.  This callback is called with `true` when [`show`] runs
    /// (move the HWND off-screen so the GTK frame is visible) and with
    /// `false` when [`hide`] runs (restore the HWND to its normal position).
    #[cfg(target_os = "windows")]
    offscreen_hook: RefCell<Option<Box<dyn Fn(bool)>>>,
}

impl LoadingOverlay {
    /// Wrap `child` in an overlay with a centered indeterminate progress bar.
    pub fn new<W: IsA<gtk4::Widget>>(child: &W) -> Rc<Self> {
        ensure_frame_css();

        let overlay = Overlay::new();
        overlay.set_child(Some(child));

        let progress = ProgressBar::new();
        progress.set_pulse_step(0.12);
        progress.set_hexpand(true);
        progress.set_width_request(240);

        // "Rendering…" label above the bar so the user knows what's happening.
        let label = gtk4::Label::new(Some("Rendering…"));
        label.add_css_class("marco-loading-label");
        label.set_halign(gtk4::Align::Center);

        // Wrap the bar in a themed Box so it has a solid background and a
        // subtle border — keeps it legible over arbitrary preview content
        // (welcome page, dark themes, images, etc.).
        let frame = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        frame.add_css_class("marco-loading-frame");
        frame.set_halign(gtk4::Align::Center);
        frame.set_valign(gtk4::Align::Center);
        frame.append(&label);
        frame.append(&progress);
        frame.set_visible(false);
        // Don't intercept input — let scroll/click pass through to the WebView
        // (the bar is informational only).
        frame.set_can_target(false);
        frame.set_can_focus(false);

        overlay.add_overlay(&frame);

        Rc::new(Self {
            overlay,
            frame,
            progress,
            pulse_id: RefCell::new(None),
            #[cfg(target_os = "windows")]
            offscreen_hook: RefCell::new(None),
        })
    }

    /// Return the wrapping overlay widget for placement in the widget tree.
    pub fn widget(&self) -> &Overlay {
        &self.overlay
    }

    /// Start (or restart) the indeterminate pulse animation.
    pub fn show(self: &Rc<Self>) {
        if self.pulse_id.borrow().is_some() {
            return;
        }
        self.progress.set_fraction(0.0);
        self.frame.set_visible(true);
        #[cfg(target_os = "windows")]
        if let Some(f) = self.offscreen_hook.borrow().as_ref() {
            f(true);
        }

        let pb = self.progress.clone();
        let weak = Rc::downgrade(self);
        let id = glib::timeout_add_local(PULSE_INTERVAL, move || {
            if weak.upgrade().is_none() {
                return glib::ControlFlow::Break;
            }
            pb.pulse();
            glib::ControlFlow::Continue
        });
        *self.pulse_id.borrow_mut() = Some(id);
    }

    /// Stop the pulse animation and hide the bar.
    pub fn hide(&self) {
        if let Some(id) = self.pulse_id.borrow_mut().take() {
            id.remove();
        }
        self.frame.set_visible(false);
        #[cfg(target_os = "windows")]
        if let Some(f) = self.offscreen_hook.borrow().as_ref() {
            f(false);
        }
    }

    /// On Windows: install a callback invoked with `true`/`false` when
    /// [`LoadingOverlay::show`]/[`LoadingOverlay::hide`] runs.  Use this to
    /// move the native wry HWND off-screen so the GTK progress-bar frame
    /// is visible during rendering, then restore it when the page is ready.
    #[cfg(target_os = "windows")]
    pub fn set_offscreen_hook<F: Fn(bool) + 'static>(&self, f: F) {
        *self.offscreen_hook.borrow_mut() = Some(Box::new(f));
    }
}

thread_local! {
    static GLOBAL: RefCell<Option<Rc<LoadingOverlay>>> = const { RefCell::new(None) };
}

/// Register the overlay for global show/hide access from the main thread.
pub fn set_global(overlay: Rc<LoadingOverlay>) {
    GLOBAL.with(|g| *g.borrow_mut() = Some(overlay));
}

/// Show the registered overlay, if any.
pub fn show() {
    GLOBAL.with(|g| {
        if let Some(o) = g.borrow().as_ref() {
            o.show();
        }
    });
}

/// Hide the registered overlay, if any.
pub fn hide() {
    GLOBAL.with(|g| {
        if let Some(o) = g.borrow().as_ref() {
            o.hide();
        }
    });
}
