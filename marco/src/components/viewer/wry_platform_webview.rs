//! Windows-specific PlatformWebView using `wry` (WebView2) embedded as a child
//! window inside the GTK `ApplicationWindow`.
//!
//! This mirrors the approach used in `polo` so Marco's preview can embed a
//! `wry::WebView` on Windows (using Win32 HWND obtained from GDK surface) and
//! avoid spawning a separate tao EventLoop thread.

// Note: this module is conditionally compiled from `components::viewer::mod`.

use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

#[cfg(target_os = "windows")]
use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
};

#[cfg(target_os = "windows")]
use std::num::NonZeroIsize;

type ScrollReportCallback = Rc<dyn Fn(f64)>;
type ScrollReportCallbackCell = Rc<RefCell<Option<ScrollReportCallback>>>;

type LocalMdLinkCallback = Rc<dyn Fn(String, Option<String>)>;
type LocalMdLinkCallbackCell = Rc<RefCell<Option<LocalMdLinkCallback>>>;

/// Callback invoked when the export WebView posts a `marco_export:*` IPC message.
/// Receives the full raw message string (e.g. `"marco_export:layout_done"`).
type ExportEventCallback = Rc<dyn Fn(String)>;
type ExportEventCallbackCell = Rc<RefCell<Option<ExportEventCallback>>>;

/// Callback invoked when the preview JS reports a hovered link via
/// `marco_hover:<url>` (or `marco_hover:` to clear). `None` clears.
type HoverLinkCallback = Rc<dyn Fn(Option<String>)>;
type HoverLinkCallbackCell = Rc<RefCell<Option<HoverLinkCallback>>>;

/// Callback invoked when the in-page JS find engine (`MarcoFind`, see
/// [`crate::components::viewer::wry_find`]) reports search results via
/// `marco_find:count=<N>,index=<K>` IPC. The payload is parsed into a
/// [`crate::components::viewer::wry_find::FindReport`] before delivery.
type FindReportCallback = Rc<dyn Fn(crate::components::viewer::wry_find::FindReport)>;
type FindReportCallbackCell = Rc<RefCell<Option<FindReportCallback>>>;

/// Callback invoked when the preview JS replies to a state-snapshot request
/// with `marco_state:{...json...}` IPC. The payload is parsed into a
/// [`crate::components::viewer::preview_state::PreviewState`] before
/// delivery; malformed payloads are dropped.
type StateSnapshotCallback = Rc<dyn Fn(crate::components::viewer::preview_state::PreviewState)>;
type StateSnapshotCallbackCell = Rc<RefCell<Option<StateSnapshotCallback>>>;

/// Callback invoked when the in-page bootstrap posts `marco_zoom:ready`,
/// which is the earliest reliable "document painted" signal available
/// through wry/WebView2 IPC. Used by detached-preview restoration to wait
/// for the new WebView to finish loading before dispatching
/// `MarcoCorePreview.restoreState` (§14.3 of the parity audit).
type ReadyCallback = Rc<dyn Fn()>;
type ReadyCallbackCell = Rc<RefCell<Option<ReadyCallback>>>;

/// Monotonic counter for assigning unique IDs to each PlatformWebView instance.
static WEBVIEW_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Per-webview HTML content storage.
/// Keyed by PlatformWebView.id so the custom protocol handler always serves
/// the correct HTML even when multiple webviews coexist (one per editor tab).
static WEBVIEW_HTML_MAP: OnceLock<Mutex<std::collections::HashMap<u64, Vec<u8>>>> = OnceLock::new();

fn html_map() -> &'static Mutex<std::collections::HashMap<u64, Vec<u8>>> {
    WEBVIEW_HTML_MAP.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

/// RAII guard that owns a `WEBVIEW_HTML_MAP` slot. When the last `Rc<IdGuard>`
/// for a given `PlatformWebView` instance is dropped, the corresponding HTML
/// entry is removed so the map cannot grow unboundedly as editor tabs are
/// opened and closed.
///
/// `PlatformWebView` derives `Clone`, so the guard must be reference-counted
/// — only the *last* surviving clone should evict the entry. The custom
/// protocol handler captures the same `u64` id, so as long as a clone is
/// alive the lookup keeps working.
struct IdGuard(u64);

impl Drop for IdGuard {
    fn drop(&mut self) {
        if let Ok(mut map) = html_map().lock() {
            map.remove(&self.0);
        }
        log::trace!("[wry] WEBVIEW_HTML_MAP entry evicted for id={}", self.0);
    }
}

/// Custom protocol scheme name used for serving HTML content.
/// On Windows, wry maps this to `http://marco-preview.localhost/` so the
/// IPC `Source` URL is never empty (which would otherwise cause a panic in
/// wry 0.55 when using `NavigateToString`).
const CUSTOM_SCHEME: &str = "marco-preview";

/// URL used only for the initial WebViewBuilder `.with_url()` call.
/// wry applies the custom-protocol workaround during `build()`, translating
/// `marco-preview://localhost/` → `http://marco-preview.localhost/`.
const CONTENT_URL_BUILDER: &str = "marco-preview://localhost/";

/// URL used for subsequent `WebView::load_url()` calls.
/// wry's URI workaround is NOT applied by `load_url` — it only runs at build time —
/// so we must use the already-transformed HTTP form directly, otherwise
/// `Navigate("marco-preview://localhost/")` silently fails and the page never refreshes.
const CONTENT_URL_RELOAD_BASE: &str = "http://marco-preview.localhost/";

/// Windows PlatformWebView wrapper
#[derive(Clone)]
pub struct PlatformWebView {
    /// Reference-counted RAII guard for this instance's `WEBVIEW_HTML_MAP`
    /// slot. The map entry is removed when the last clone is dropped — see
    /// [`IdGuard::drop`]. Access the id directly via `self.id()`.
    id_guard: Rc<IdGuard>,
    /// Monotonically increasing counter appended to the reload URL as `?v=N`.
    /// Each increment produces a unique URL so WebView2 cannot serve a cached response.
    load_version: Rc<std::cell::Cell<u64>>,
    pub inner: std::rc::Rc<std::cell::RefCell<Option<wry::WebView>>>,
    pub container: gtk4::Box,
    parent_handle: std::rc::Rc<ParentWindowHandle>,
    pub bg_color: std::rc::Rc<std::cell::Cell<(u8, u8, u8, u8)>>,
    /// Top-level GTK window hosting this preview. Stored as `gtk4::Window`
    /// (rather than `ApplicationWindow`) so the preview can also be embedded
    /// inside transient sub-windows such as dialogs without requiring a
    /// downcast to `ApplicationWindow`.
    pub gtk_window: gtk4::Window,
    scroll_report_callback: ScrollReportCallbackCell,
    local_md_link_callback: LocalMdLinkCallbackCell,
    /// One-shot listener for `marco_export:*` lifecycle events posted from the
    /// export WebView's JS bridge. Installed by [`Self::set_export_event_listener`]
    /// and cleared by [`Self::clear_export_event_listener`].
    export_event_callback: ExportEventCallbackCell,
    /// Listener for `marco_hover:<url>` IPC messages emitted by the preview's
    /// hover-report JS. Used to drive the footer hovered-link label on Windows.
    hover_link_callback: HoverLinkCallbackCell,
    /// Listener for `marco_find:count=N,index=K` IPC messages emitted by the
    /// `MarcoFind` JS engine. Drives the search-window "K of N" indicator.
    find_report_callback: FindReportCallbackCell,
    /// Listener for `marco_state:{...}` IPC messages produced in reply to
    /// [`Self::request_state_snapshot`]. Drives the detached-preview state
    /// transfer flow (§14.3 of the parity audit).
    state_snapshot_callback: StateSnapshotCallbackCell,
    /// Listener for `marco_zoom:ready` IPC messages emitted by the in-page
    /// bootstrap on every full document load. Drives detached-preview state
    /// *restoration*: the detached window installs a callback that, on
    /// ready, dispatches `restore_script(&take_latest_state())`.
    ready_callback: ReadyCallbackCell,
    /// When `true` the tick callback keeps the wry HWND at −32000,−32000 so
    /// the GTK loading-overlay frame is visible above it.
    is_offscreen_for_loading: Rc<std::cell::Cell<bool>>,
    /// GTK CSS provider used to paint the container background to match the
    /// WebView2 background colour, preventing a white flash while the HWND
    /// is at −32000,−32000 during loading.
    bg_css_provider: Rc<gtk4::CssProvider>,
}

impl PlatformWebView {
    /// Construct a new Windows `PlatformWebView` parented to any GTK4 top-level
    /// window (`ApplicationWindow`, plain `Window`, dialog `Window`, ...).
    ///
    /// Previously this required `&gtk4::ApplicationWindow`, which forced dialog
    /// hosts to downcast and fall back to a `Label` when the parent was a plain
    /// `gtk4::Window`. Accepting `&impl IsA<gtk4::Window>` removes that
    /// degraded path because `gtk4::Window` already implements `Native`, which
    /// is what we actually need for `gdk_win32_surface_get_handle`.
    pub fn new(window: &impl IsA<gtk4::Window>) -> Self {
        use gtk4::prelude::WidgetExt;

        // Normalise to a concrete `gtk4::Window` once so the rest of the body
        // doesn't need to be generic.
        let window: gtk4::Window = window.clone().upcast();
        let window = &window;

        // Ensure the GTK window is realized so a surface/handle exists
        WidgetExt::realize(window);

        // Assign a unique ID to this instance
        let id = WEBVIEW_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        let load_version: Rc<std::cell::Cell<u64>> = Rc::new(std::cell::Cell::new(0));

        // Default fallback container & state
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        container.set_vexpand(true);
        container.set_hexpand(true);
        let webview: Rc<RefCell<Option<wry::WebView>>> = Rc::new(RefCell::new(None));
        let bg_color = std::rc::Rc::new(std::cell::Cell::new((30u8, 30u8, 30u8, 255u8)));
        let scroll_report_callback: ScrollReportCallbackCell = Rc::new(RefCell::new(None));
        let local_md_link_callback: LocalMdLinkCallbackCell = Rc::new(RefCell::new(None));
        let export_event_callback: ExportEventCallbackCell = Rc::new(RefCell::new(None));
        let hover_link_callback: HoverLinkCallbackCell = Rc::new(RefCell::new(None));
        let find_report_callback: FindReportCallbackCell = Rc::new(RefCell::new(None));
        let state_snapshot_callback: StateSnapshotCallbackCell = Rc::new(RefCell::new(None));
        let ready_callback: ReadyCallbackCell = Rc::new(RefCell::new(None));
        let is_offscreen_for_loading = Rc::new(std::cell::Cell::new(false));

        // Set up a GTK CSS provider so the container widget is painted with the
        // theme background colour while the WebView2 HWND is offscreen during
        // loading — preventing the white-GTK-widget-behind-invisible-HWND flash.
        container.add_css_class("marco-preview-bg");
        let bg_css_provider = Rc::new(gtk4::CssProvider::new());
        bg_css_provider.load_from_data(".marco-preview-bg { background-color: #1e1e1e; }");
        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &*bg_css_provider,
                gtk4::STYLE_PROVIDER_PRIORITY_USER,
            );
        }

        // Attempt to obtain parent HWND and parent_handle; on failure, keep inner None
        let parent_handle_rc = match (|| {
            // Get the GDK surface from the GTK window
            let surface = window.surface()?;

            // Use gdk4-win32 to get the native Win32 HWND
            use gdk4_win32::Win32Surface;
            let win32_surface: &Win32Surface = surface.downcast_ref()?;

            let hwnd_ptr = unsafe {
                gdk4_win32::ffi::gdk_win32_surface_get_handle(win32_surface.as_ptr() as *mut _)
            };
            let hwnd = NonZeroIsize::new(hwnd_ptr as isize)?;

            let win_handle = Win32WindowHandle::new(hwnd);
            let raw_window = RawWindowHandle::Win32(win_handle);
            let raw_display = RawDisplayHandle::Windows(WindowsDisplayHandle::new());

            let parent_handle = ParentWindowHandle {
                window: unsafe { raw_window_handle::WindowHandle::borrow_raw(raw_window) },
                display: unsafe { raw_window_handle::DisplayHandle::borrow_raw(raw_display) },
            };
            Some(std::rc::Rc::new(parent_handle))
        })() {
            Some(ph) => {
                log::info!("wry PlatformWebView: obtained Win32 parent handle");
                ph
            }
            None => {
                log::warn!("wry PlatformWebView: failed to get Win32 parent handle - falling back to placeholder container");
                // Add a placeholder label into the container so UI is usable
                let label = gtk4::Label::new(Some(
                    "Preview not available inline on Windows (missing Win32 handle)",
                ));
                label.set_wrap(true);
                container.append(&label);
                // Provide a dummy ParentWindowHandle so types work later if needed
                let parent_handle = ParentWindowHandle {
                    window: unsafe {
                        raw_window_handle::WindowHandle::borrow_raw(RawWindowHandle::Win32(
                            Win32WindowHandle::new(NonZeroIsize::new(1).unwrap()),
                        ))
                    },
                    display: unsafe {
                        raw_window_handle::DisplayHandle::borrow_raw(RawDisplayHandle::Windows(
                            WindowsDisplayHandle::new(),
                        ))
                    },
                };
                std::rc::Rc::new(parent_handle)
            }
        };

        // Keep WebView bounds in sync with GTK container on every frame.
        // Use `compute_point` to translate container origin into the window's
        // coordinate system so positioning matches Win32 expectations.
        let webview_for_tick = webview.clone();
        let container_weak = container.downgrade();
        let window_weak = window.downgrade();
        let is_offscreen_tick = is_offscreen_for_loading.clone();
        container.add_tick_callback(move |_, _| {
            if let (Some(container), Some(win), Some(view)) = (container_weak.upgrade(), window_weak.upgrade(), webview_for_tick.borrow().as_ref()) {
                // When the GTK container is not mapped (e.g. the Stack switched to code_preview),
                // or a loading operation is in progress, move the native Win32 WebView
                // off-screen so the GTK loading-overlay frame is visible.
                if !container.is_mapped() || is_offscreen_tick.get() {
                    // When offscreen for loading, use the actual container size so
                    // WebView2 renders the page at the correct viewport dimensions
                    // and no reflow/white-flash occurs when the HWND is restored.
                    let (w, h) = if is_offscreen_tick.get() {
                        let alloc = container.allocation();
                        (alloc.width().max(100) as f64, alloc.height().max(100) as f64)
                    } else {
                        // Container not mapped (e.g. Stack switched to code view): use minimal size.
                        (1.0, 1.0)
                    };
                    let _ = view.set_bounds(wry::Rect {
                        position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(
                            -32000.0,
                            -32000.0,
                        )),
                        size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(w, h)),
                    });
                    return gtk4::glib::ControlFlow::Continue;
                }

                let alloc = container.allocation();
                let (offset_x, offset_y) = if win.is_maximized() { (0.0, 0.0) } else { (14.0, 12.0) };

                // Compute the top-left of the container in window coordinates
                let origin_in_window = match container.translate_coordinates(&win, 0.0, 0.0) {
                    Some((x, y)) => (x, y),
                    None => (alloc.x() as f64, alloc.y() as f64),
                };

                let rect = wry::Rect {
                    position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(
                        origin_in_window.0 + offset_x - 1.0,
                        origin_in_window.1 + offset_y,
                    )),
                    size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(alloc.width().max(1) as f64 + 1.0, alloc.height().max(1) as f64)),
                };

                log::debug!("[wry] container origin_in_window=({}, {}), alloc=({}, {}), rect_pos=({}, {}), rect_size=({}, {})",
                    origin_in_window.0, origin_in_window.1, alloc.x(), alloc.y(),
                    origin_in_window.0 + offset_x - 1.0, origin_in_window.1 + offset_y,
                    alloc.width().max(1) + 1, alloc.height().max(1)
                );

                if let Err(e) = view.set_bounds(rect) {
                    log::debug!("wry set_bounds failed: {}", e);
                }
            }
            gtk4::glib::ControlFlow::Continue
        });

        Self {
            id_guard: Rc::new(IdGuard(id)),
            load_version,
            inner: webview,
            container,
            parent_handle: parent_handle_rc,
            bg_color,
            gtk_window: window.clone(),
            scroll_report_callback,
            local_md_link_callback,
            export_event_callback,
            hover_link_callback,
            find_report_callback,
            state_snapshot_callback,
            ready_callback,
            is_offscreen_for_loading,
            bg_css_provider,
        }
    }

    pub fn set_scroll_report_callback<F: Fn(f64) + 'static>(&self, callback: F) {
        *self.scroll_report_callback.borrow_mut() = Some(Rc::new(callback));
    }

    pub fn set_local_md_link_handler<F: Fn(String, Option<String>) + 'static>(&self, callback: F) {
        *self.local_md_link_callback.borrow_mut() = Some(Rc::new(callback));
    }

    /// Install a callback invoked when the preview reports a hovered link via
    /// `marco_hover:<url>` IPC. Pass `None` (empty payload) to clear.
    pub fn set_hover_link_callback<F: Fn(Option<String>) + 'static>(&self, callback: F) {
        *self.hover_link_callback.borrow_mut() = Some(Rc::new(callback));
    }

    /// Install a callback invoked when the in-page `MarcoFind` JS engine
    /// posts a `marco_find:count=<N>,index=<K>` IPC message. The payload is
    /// parsed into a [`crate::components::viewer::wry_find::FindReport`]
    /// before delivery; malformed payloads are dropped.
    ///
    /// Pair this with [`crate::components::viewer::wry_find::install`] and
    /// the `search` / `next` / `prev` / `clear` helpers.
    pub fn set_find_report_callback<F>(&self, callback: F)
    where
        F: Fn(crate::components::viewer::wry_find::FindReport) + 'static,
    {
        *self.find_report_callback.borrow_mut() = Some(Rc::new(callback));
    }

    /// Install a callback invoked when the preview JS replies to a
    /// state-snapshot request (see [`Self::request_state_snapshot`]).
    ///
    /// Used by the detach flow to capture scroll position and open
    /// `<details>` panels before destroying the in-editor WebView so the
    /// detached WebView can restore them via
    /// [`crate::components::viewer::preview_state::restore_script`].
    pub fn set_state_snapshot_callback<F>(&self, callback: F)
    where
        F: Fn(crate::components::viewer::preview_state::PreviewState) + 'static,
    {
        *self.state_snapshot_callback.borrow_mut() = Some(Rc::new(callback));
    }

    /// Request a state snapshot from the live document.
    ///
    /// Evaluates [`crate::components::viewer::preview_state::snapshot_script`]
    /// in the WebView. The page replies asynchronously via the
    /// `marco_state:` IPC arm, which delivers the parsed
    /// [`PreviewState`](crate::components::viewer::preview_state::PreviewState)
    /// to the callback installed by [`Self::set_state_snapshot_callback`].
    /// No-op if the WebView has not been built yet.
    pub fn request_state_snapshot(&self) {
        self.evaluate_script(crate::components::viewer::preview_state::snapshot_script());
    }

    /// Install a callback invoked every time the in-page bootstrap posts
    /// `marco_zoom:ready` — the earliest reliable "document painted"
    /// signal available through wry/WebView2 IPC.
    ///
    /// Used by the detached preview window to fire
    /// [`crate::components::viewer::preview_state::restore_script`] once
    /// the freshly-built WebView has rendered the new HTML. Replacing the
    /// callback simply overwrites the previous one (use `Cell`-style logic
    /// inside the closure if you want one-shot semantics).
    pub fn set_ready_callback<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        *self.ready_callback.borrow_mut() = Some(Rc::new(callback));
    }

    /// Move the wry HWND off-screen (`offscreen = true`) so the GTK loading-
    /// overlay frame is visible during rendering, or restore it to its normal
    /// position (`offscreen = false`) once the page has loaded.
    ///
    /// Called by the [`LoadingOverlay`](super::loading_overlay::LoadingOverlay)
    /// offscreen hook that is wired up in `editor/ui.rs`.
    pub fn set_offscreen_for_loading(&self, offscreen: bool) {
        self.is_offscreen_for_loading.set(offscreen);
        // Immediately push the HWND off-screen so we don't wait for the next
        // tick-callback iteration (~16 ms) before the GTK overlay becomes visible.
        if offscreen {
            if let Some(view) = self.inner.borrow().as_ref() {
                let alloc = self.container.allocation();
                let _ = view.set_bounds(wry::Rect {
                    position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(
                        -32000.0, -32000.0,
                    )),
                    size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(
                        alloc.width().max(100) as f64,
                        alloc.height().max(100) as f64,
                    )),
                });
            }
        }
    }

    pub fn widget(&self) -> gtk4::Widget {
        self.container.clone().upcast()
    }

    pub fn set_background_color_rgba(&self, color: &gtk4::gdk::RGBA) {
        let rgba = (
            (color.red() * 255.0) as u8,
            (color.green() * 255.0) as u8,
            (color.blue() * 255.0) as u8,
            (color.alpha() * 255.0) as u8,
        );
        self.bg_color.set(rgba);
        if let Some(view) = self.inner.borrow().as_ref() {
            if let Err(e) = view.set_background_color(rgba) {
                log::warn!("Failed to update wry background color: {}", e);
            }
        }
        // Keep the GTK container background in sync so there is no white flash
        // while the WebView2 HWND is offscreen during loading.
        let css_data = format!(
            ".marco-preview-bg {{ background-color: rgb({},{},{}); }}",
            (color.red() * 255.0) as u32,
            (color.green() * 255.0) as u32,
            (color.blue() * 255.0) as u32,
        );
        self.bg_css_provider.load_from_data(&css_data);
    }

    pub fn load_html_with_base(&self, html: &str, base_uri: Option<&str>) {
        let final_html = if let Some(base) = base_uri {
            inject_base_href(html, base)
        } else {
            html.to_string()
        };

        // Store HTML so the custom protocol handler can serve it.
        // Using NavigateToString (wry's load_html) leaves WebView2's Source URL
        // empty, which causes a panic in wry's IPC handler when any JS IPC
        // message fires.  By storing the HTML here and navigating to a custom
        // protocol URL instead, the Source URL is always a non-empty valid URI.
        html_map()
            .lock()
            .unwrap()
            .insert(self.id_guard.0, final_html.into_bytes());

        // Increment load version so each reload URL is unique (busts WebView2 cache).
        let v = self.load_version.get().wrapping_add(1);
        self.load_version.set(v);
        let reload_url = format!("{}?v={}", CONTENT_URL_RELOAD_BASE, v);

        if let Some(view) = self.inner.borrow().as_ref() {
            if let Err(e) = view.load_url(&reload_url) {
                log::error!("Failed to reload wry WebView via custom protocol: {}", e);
            }
            return;
        }

        // Defer first-time creation until the GTK container is mapped *and*
        // has a non-trivial allocation. This is the wry counterpart of the
        // Linux `load_html_when_ready` polling loop (~4.8 s budget, 60 fps).
        // Without this, WebView2 is built against an `allocated_width <= 1`
        // container and renders at a wrong size until the next tick callback.
        let me = self.clone();
        crate::components::viewer::allocation_wait::run_when_allocated(
            &self.container,
            crate::components::viewer::allocation_wait::DEFAULT_MAX_RETRIES,
            move || {
                // A previous deferred load may have already built the WebView —
                // in that case just navigate it instead of building a second one.
                if let Some(view) = me.inner.borrow().as_ref() {
                    let url = format!("{}?v={}", CONTENT_URL_RELOAD_BASE, me.load_version.get());
                    if let Err(e) = view.load_url(&url) {
                        log::error!("Failed to reload wry WebView via custom protocol: {}", e);
                    }
                    return;
                }
                me.build_initial_webview();
            },
        );
    }

    /// Build the wry `WebView` for the first time and install it as a child
    /// Win32 window inside `self.container`. Must only be called when:
    ///
    /// 1. `self.inner` is `None` (no existing WebView).
    /// 2. `self.container` is mapped and has `allocated_width/height > 1`.
    ///
    /// Both preconditions are enforced by `load_html_with_base` via
    /// [`allocation_wait::run_when_allocated`].
    fn build_initial_webview(&self) {
        let alloc = self.container.allocation();
        let (offset_x, offset_y) = if self.gtk_window.is_maximized() {
            (0.0, 0.0)
        } else {
            (16.0, 14.0)
        };

        // Translate container origin into window coordinate space so the initial
        // creation uses correct coordinates on Windows.
        let origin_in_window =
            match self
                .container
                .translate_coordinates(&self.gtk_window, 0.0, 0.0)
            {
                Some((x, y)) => (x, y),
                None => (alloc.x() as f64, alloc.y() as f64),
            };

        // Build the WebView off-screen if a loading operation is in progress
        // so the GTK loading-overlay frame stays visible until the page loads.
        // Use the actual container allocation size so WebView2 renders the HTML
        // at the correct viewport dimensions — avoids a reflow/white-flash when
        // the HWND is restored by the tick callback after page load completes.
        let rect = if self.is_offscreen_for_loading.get() {
            wry::Rect {
                position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(
                    -32000.0, -32000.0,
                )),
                size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(
                    alloc.width().max(100) as f64,
                    alloc.height().max(100) as f64,
                )),
            }
        } else {
            // Allocation is guaranteed > 1 here by `run_when_allocated`.
            wry::Rect {
                position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(
                    origin_in_window.0 + offset_x - 1.0,
                    origin_in_window.1 + offset_y,
                )),
                size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(
                    alloc.width() as f64 + 1.0,
                    alloc.height() as f64,
                )),
            }
        };

        log::debug!("[wry] initial_create origin_in_window=({}, {}), alloc=({}, {}), rect_pos=({}, {}), rect_size=({}, {})",
            origin_in_window.0, origin_in_window.1, alloc.x(), alloc.y(),
            origin_in_window.0 + offset_x - 1.0, origin_in_window.1 + offset_y,
            alloc.width() + 1, alloc.height()
        );

        // Configure WebView2 to use data directory (portable mode friendly)
        // WebView2 respects WEBVIEW2_USER_DATA_FOLDER environment variable
        let data_dir = marco_shared::paths::user_data_dir().join("webview");
        if let Err(e) = std::fs::create_dir_all(&data_dir) {
            log::warn!("Failed to create WebView2 data directory: {}", e);
        }
        std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", data_dir);

        match wry::WebViewBuilder::new()
            .with_background_color(self.bg_color.get())
            .with_bounds(rect)
            .with_url(CONTENT_URL_BUILDER)
            .with_custom_protocol(CUSTOM_SCHEME.to_string(), {
                let id = self.id_guard.0;
                move |_webview_id, _req| {
                    let html_bytes = html_map()
                        .lock()
                        .unwrap()
                        .get(&id)
                        .cloned()
                        .unwrap_or_default();
                    wry::http::Response::builder()
                        .header("Content-Type", "text/html; charset=utf-8")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(std::borrow::Cow::Owned(html_bytes))
                        .unwrap()
                }
            })
            .with_ipc_handler({
                let scroll_cb = self.scroll_report_callback.clone();
                let export_cb = self.export_event_callback.clone();
                let hover_cb = self.hover_link_callback.clone();
                let find_cb = self.find_report_callback.clone();
                let state_cb = self.state_snapshot_callback.clone();
                let ready_cb = self.ready_callback.clone();
                // For `marco_print:` IPC — reach back into the WebView to invoke
                // the system print UI (`ICoreWebView2_7::ShowPrintUI(SYSTEM)`)
                // instead of letting in-page JS fall through to WebView2's
                // Chromium-style in-page print overlay. See §14.7 of
                // `documentation/webkit6_wry_parity_audit.md`.
                let inner_for_print = self.inner.clone();
                move |req: wry::http::Request<String>| {
                    let msg = req.body().as_str();
                    // ── scroll sync ───────────────────────────────────────
                    if let Some(scroll_data) = msg.strip_prefix("marco_scroll:") {
                        if let Ok(percentage) = scroll_data.parse::<f64>() {
                            let cb_opt = scroll_cb.borrow().clone();
                            if let Some(cb) = cb_opt {
                                let percentage = percentage.clamp(0.0, 1.0);
                                gtk4::glib::MainContext::default()
                                    .invoke_local(move || cb(percentage));
                            }
                        }
                        return;
                    }
                    // ── hovered link reports ──────────────────────────────
                    if let Some(payload) = msg.strip_prefix("marco_hover:") {
                        let url = if payload.is_empty() {
                            None
                        } else {
                            Some(payload.to_string())
                        };
                        let cb_opt = hover_cb.borrow().clone();
                        if let Some(cb) = cb_opt {
                            gtk4::glib::MainContext::default().invoke_local(move || cb(url));
                        }
                        return;
                    }
                    // ── find-in-preview reports ───────────────────────────
                    if let Some(payload) = msg.strip_prefix("marco_find:") {
                        if let Some(report) =
                            crate::components::viewer::wry_find::parse_report(payload)
                        {
                            let cb_opt = find_cb.borrow().clone();
                            if let Some(cb) = cb_opt {
                                gtk4::glib::MainContext::default()
                                    .invoke_local(move || cb(report));
                            }
                        } else {
                            log::debug!(
                                "[wry] malformed marco_find payload: {:?}",
                                payload
                            );
                        }
                        return;
                    }
                    // ── preview state snapshot reply ───────────────────────
                    if let Some(payload) = msg.strip_prefix("marco_state:") {
                        if let Some(state) =
                            crate::components::viewer::preview_state::parse_snapshot_payload(
                                payload,
                            )
                        {
                            // Always persist the latest snapshot so a detach
                            // can read it without needing an explicit
                            // callback handshake.
                            crate::components::viewer::preview_state::set_latest_state(
                                Some(state.clone()),
                            );
                            let cb_opt = state_cb.borrow().clone();
                            if let Some(cb) = cb_opt {
                                gtk4::glib::MainContext::default()
                                    .invoke_local(move || cb(state));
                            }
                        } else {
                            log::debug!(
                                "[wry] malformed marco_state payload: {:?}",
                                payload
                            );
                        }
                        return;
                    }
                    // ── in-page zoom toolbar ──────────────────────────────
                    if let Some(action) = msg.strip_prefix("marco_zoom:") {
                        use crate::components::editor::editor_manager as em;
                        let action = action.to_string();
                        let ready_cb_owned = ready_cb.clone();
                        gtk4::glib::MainContext::default().invoke_local(move || {
                            let new_zoom = match action.as_str() {
                                "in" => em::get_preview_zoom() + em::ZOOM_STEP,
                                "out" => em::get_preview_zoom() - em::ZOOM_STEP,
                                "reset" => em::ZOOM_DEFAULT,
                                // The page just (re)loaded — re-apply the
                                // currently persisted zoom because `style.zoom`
                                // is reset on every document replacement.
                                "ready" => em::get_preview_zoom(),
                                _ => return,
                            };
                            em::set_preview_zoom(new_zoom);
                            // `ready` is the earliest reliable "page is painted"
                            // signal available through wry/WebView2 IPC.  Hide the
                            // loading overlay here instead of eagerly after queuing
                            // the navigation, which would dismiss it before the new
                            // content is actually visible.
                            if action == "ready" {
                                crate::components::viewer::loading_overlay::hide();
                                // Notify any per-WebView listener (e.g. the
                                // detached preview window's state-restore hook).
                                let cb_opt = ready_cb_owned.borrow().clone();
                                if let Some(cb) = cb_opt {
                                    cb();
                                }
                            }
                        });
                        return;
                    }
                    // ── export lifecycle events ───────────────────────────
                    if msg.starts_with("marco_export:") {
                        let cb_opt = export_cb.borrow().clone();
                        if let Some(cb) = cb_opt {
                            let owned = msg.to_string();
                            gtk4::glib::MainContext::default().invoke_local(move || cb(owned));
                        }
                    }                    // ── print request from in-page JS ────────────────────
                    // Any `marco_print:<subcommand>` payload (currently only
                    // `dialog` is meaningful) routes through the host so the
                    // system print UI opens instead of WebView2's Chromium
                    // in-page print preview. This lets future in-page
                    // toolbars (e.g. a print button inside `WIN_ZOOM_BAR_HTML`)
                    // request a print without falling back to `window.print()`.
                    if msg.starts_with("marco_print:") {
                        let inner = inner_for_print.clone();
                        gtk4::glib::MainContext::default().invoke_local(move || {
                            if let Some(view) = inner.borrow().as_ref() {
                                if let Err(e) = show_system_print_ui(view) {
                                    log::warn!(
                                        "[wry] marco_print: ShowPrintUI(SYSTEM) failed ({}); falling back to wry view.print()",
                                        e
                                    );
                                    if let Err(e2) = view.print() {
                                        log::warn!(
                                            "[wry] marco_print: wry view.print() also failed: {}",
                                            e2
                                        );
                                    }
                                }
                            } else {
                                log::debug!(
                                    "[wry] marco_print: ignored \u{2014} WebView not yet initialized"
                                );
                            }
                        });
                    }                }
            })
            .with_navigation_handler({
                let md_callback = self.local_md_link_callback.clone();
                move |uri: String| {
                    // Intercept local .md file links — open in editor instead of navigating
                    if is_local_md_uri(&uri) {
                        log::info!("[wry] Local .md link intercepted: {}", uri);
                        let (path, fragment) = extract_path_and_fragment_from_file_uri(&uri);
                        let cb_opt = md_callback.borrow().clone();
                        if let Some(cb) = cb_opt {
                            gtk4::glib::MainContext::default()
                                .invoke_local(move || cb(path, fragment));
                        }
                        return false;
                    }
                    if should_open_externally(&uri) {
                        log::debug!("[wry] intercept navigation to external URI: {}", uri);
                        if let Err(e) = crate::components::viewer::wry::open_external_uri(&uri) {
                            log::warn!("[wry] failed to open external URI '{}': {}", uri, e);
                        }
                        return false;
                    }
                    true
                }
            })
            .build_as_child(&*self.parent_handle)
        {
            Ok(view) => {
                *self.inner.borrow_mut() = Some(view);
                log::info!("wry WebView successfully created as child for initial load");
            }
            Err(e) => log::error!("Failed to build wry WebView for initial load: {}", e),
        }
    }

    pub fn evaluate_script(&self, script: &str) {
        if let Some(view) = self.inner.borrow().as_ref() {
            if let Err(e) = view.evaluate_script(script) {
                log::error!("JavaScript evaluation failed: {}", e);
            }
        }
    }

    /// Patch the live preview's `mc-content-container` in place via JavaScript,
    /// without reloading the page.
    ///
    /// Mirrors `webkit6::update_html_content_smooth`:
    /// 1. Prefer `window.MarcoCorePreview.updateContent(html)` when the page's
    ///    preview bootstrap script has registered it (this preserves the
    ///    Mermaid/KaTeX/scroll caches).
    /// 2. Otherwise fall back to direct `innerHTML` replacement, preserving
    ///    `documentElement.scrollTop` across the update.
    ///
    /// `content` is JSON-encoded with `serde_json` so any HTML body — including
    /// quotes, backslashes, embedded `</script>` sequences and non-ASCII —
    /// becomes a safe JavaScript string literal without manual escaping.
    ///
    /// If the underlying WebView2 has not been built yet (no preceding
    /// [`Self::load_html_with_base`] call has succeeded), the update is
    /// silently dropped — callers must always do a full load first, matching
    /// the Linux contract documented on `webkit6::update_html_content_smooth`.
    pub fn update_html_content_smooth(&self, content: &str) {
        // Without a live WebView2 there is nothing to patch. The caller is
        // expected to do a `load_html_with_base` first; smooth updates are a
        // post-load optimization, not an initial-load path.
        if self.inner.borrow().is_none() {
            log::debug!(
                "[wry] update_html_content_smooth called before initial load_html_with_base; skipping"
            );
            return;
        }

        // JSON-encode the HTML body so it embeds safely as a JS string literal.
        let json_content = match serde_json::to_string(content) {
            Ok(s) => s,
            Err(e) => {
                log::warn!(
                    "[wry] update_html_content_smooth: failed to JSON-encode content: {}",
                    e
                );
                return;
            }
        };

        let js = format!(
            r#"(function() {{
    try {{
        var html = {json};
        if (window.MarcoCorePreview && typeof window.MarcoCorePreview.updateContent === 'function') {{
            window.MarcoCorePreview.updateContent(html);
            return;
        }}
        var container = document.getElementById('mc-content-container');
        if (container) {{
            var scrollTop = document.documentElement.scrollTop || document.body.scrollTop;
            container.innerHTML = html;
            setTimeout(function() {{
                document.documentElement.scrollTop = scrollTop;
                document.body.scrollTop = scrollTop;
            }}, 10);
        }} else {{
            var body = document.body || document.getElementsByTagName('body')[0];
            if (body) {{
                body.innerHTML = '<div id="mc-content-container">' + html + '</div>';
            }}
        }}
    }} catch (e) {{
        console.error('Error in content update:', e);
    }}
}})();"#,
            json = json_content
        );

        self.evaluate_script(&js);
    }

    /// Install a one-shot listener for `marco_export:*` IPC events from the
    /// export WebView's lifecycle bridge.
    ///
    /// Only one listener is active at a time; installing a new one replaces
    /// the previous one. Call [`Self::clear_export_event_listener`] when the
    /// export run finishes to avoid stale callbacks from future page loads.
    pub fn set_export_event_listener<F: Fn(String) + 'static>(&self, cb: F) {
        *self.export_event_callback.borrow_mut() = Some(Rc::new(cb));
    }

    /// Remove the currently registered export-event listener.
    pub fn clear_export_event_listener(&self) {
        *self.export_event_callback.borrow_mut() = None;
    }

    /// Trigger the browser print UI for the current page content.
    pub fn trigger_print_dialog(&self) {
        if let Some(view) = self.inner.borrow().as_ref() {
            // Prefer the WebView2 native system print dialog (top-level Win32
            // window owned by Marco) over wry's `view.print()`, which simply
            // calls `window.print()` JS and renders the print preview *inside*
            // the live preview area.
            match show_system_print_ui(view) {
                Ok(()) => return,
                Err(e) => {
                    log::warn!(
                        "ShowPrintUI(SYSTEM) failed ({}); falling back to wry view.print()",
                        e
                    );
                }
            }

            if let Err(e) = view.print() {
                log::warn!(
                    "Failed to open native WebView print UI: {}. Falling back to window.print()",
                    e
                );
                self.evaluate_script("window.print();");
            }
            return;
        }

        // Fallback when the WebView is not initialized yet — nothing to do.
        log::warn!(
            "[wry] trigger_print_dialog: print dialog requested before WebView is ready; ignoring"
        );
    }

    /// Export the current page contents to a PDF using WebView2's native
    /// `ICoreWebView2_7::PrintToPdf` (no Chromium subprocess).
    ///
    /// Blocks the main thread (pumping Win32 messages) until the COM async
    /// operation completes, so the GTK main loop and any modal "Exporting…"
    /// dialog stay responsive.
    ///
    /// Returns `Err` if the WebView is not initialized yet, the COM cast
    /// fails, or the PDF write does not succeed.
    pub fn print_to_pdf(
        &self,
        output_path: &std::path::Path,
        paper: &str,
        orientation: &str,
        margin_mm: u8,
    ) -> Result<(), String> {
        let inner_borrow = self.inner.borrow();
        let view = inner_borrow
            .as_ref()
            .ok_or_else(|| "WebView is not initialized yet".to_string())?;
        crate::components::viewer::wry_print_to_pdf::print_to_pdf(
            view,
            output_path,
            paper,
            orientation,
            margin_mm,
        )
    }
}

/// Open the WebView2 *system* print dialog (top-level Win32 window owned by
/// the host) instead of the in-WebView browser print preview that
/// `wry::WebView::print()` triggers via `window.print()`.
///
/// This casts the underlying `ICoreWebView2` to `ICoreWebView2_16` and calls
/// `ShowPrintUI(COREWEBVIEW2_PRINT_DIALOG_KIND_SYSTEM)`.  Returns `Err` on
/// older WebView2 runtimes that don't expose the v16 interface so the caller
/// can fall back to the legacy path.
fn show_system_print_ui(view: &wry::WebView) -> Result<(), String> {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        ICoreWebView2_16, COREWEBVIEW2_PRINT_DIALOG_KIND_SYSTEM,
    };
    use windows::core::Interface;
    use wry::WebViewExtWindows;

    let core = view.webview();
    let core16: ICoreWebView2_16 = core
        .cast()
        .map_err(|e| format!("WebView2 missing ICoreWebView2_16 (ShowPrintUI): {}", e))?;
    unsafe {
        core16
            .ShowPrintUI(COREWEBVIEW2_PRINT_DIALOG_KIND_SYSTEM)
            .map_err(|e| format!("ShowPrintUI failed: {}", e))?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
#[derive(Clone)]
struct ParentWindowHandle {
    window: raw_window_handle::WindowHandle<'static>,
    display: raw_window_handle::DisplayHandle<'static>,
}

#[cfg(target_os = "windows")]
impl raw_window_handle::HasWindowHandle for ParentWindowHandle {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        Ok(self.window)
    }
}

#[cfg(target_os = "windows")]
impl raw_window_handle::HasDisplayHandle for ParentWindowHandle {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        Ok(self.display)
    }
}

#[cfg(target_os = "windows")]
fn inject_base_href(html: &str, base: &str) -> String {
    if html.contains("<base") {
        return html.to_string();
    }

    if let Some(idx) = html.find("<head>") {
        let mut result = String::with_capacity(html.len() + base.len() + 32);
        result.push_str(&html[..idx + 6]);
        result.push_str(&format!("<base href=\"{}\">", base));
        result.push_str(&html[idx + 6..]);
        return result;
    }

    // Fallback: prepend base tag
    format!("<base href=\"{}\">{}", base, html)
}

/// Returns true if a URI points to a local `.md` file (file:// scheme + .md extension).
fn is_local_md_uri(uri: &str) -> bool {
    let lower = uri.to_ascii_lowercase();
    if !lower.starts_with("file://") {
        return false;
    }
    // Strip fragment before checking extension
    let without_fragment = lower.split('#').next().unwrap_or(&lower);
    // Strip query before checking extension
    let without_query = without_fragment
        .split('?')
        .next()
        .unwrap_or(without_fragment);
    without_query.ends_with(".md")
}

/// Extract (path_string, optional_fragment) from a `file://` URI.
/// Returns the decoded filesystem path and the URL fragment if present.
fn extract_path_and_fragment_from_file_uri(uri: &str) -> (String, Option<String>) {
    // Split off the fragment
    let (uri_no_frag, fragment) = if let Some(pos) = uri.find('#') {
        let frag = uri[pos + 1..].to_string();
        (&uri[..pos], if frag.is_empty() { None } else { Some(frag) })
    } else {
        (uri, None)
    };

    // Strip "file://" prefix
    let path_raw = uri_no_frag.strip_prefix("file://").unwrap_or(uri_no_frag);
    // Strip query string
    let path_raw = path_raw.split('?').next().unwrap_or(path_raw);

    // URL-decode percent-encoding using stdlib (no extra dependency needed)
    let path_decoded = percent_decode(path_raw);

    (path_decoded, fragment)
}

/// Simple percent-decoder for file URIs (handles %20, %23, etc.)
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (
                char::from(bytes[i + 1]).to_digit(16),
                char::from(bytes[i + 2]).to_digit(16),
            ) {
                out.push(char::from((h * 16 + l) as u8));
                i += 3;
                continue;
            }
        }
        out.push(char::from(bytes[i]));
        i += 1;
    }
    out
}

fn should_open_externally(uri: &str) -> bool {
    let u = uri.trim();
    if u.is_empty() {
        return false;
    }

    let lower = u.to_ascii_lowercase();

    // Allow in-document and local navigation.
    if lower.starts_with('#')
        || lower.starts_with("about:")
        || lower.starts_with("data:")
        || lower.starts_with("file:")
        || lower.starts_with("marco-preview:")
        || lower.starts_with("http://marco-preview.")
        || lower.starts_with("https://marco-preview.")
    {
        return false;
    }

    // Treat typical external schemes as external.
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with("tel:")
        || lower.starts_with("ftp://")
        || lower.starts_with("www.")
}
