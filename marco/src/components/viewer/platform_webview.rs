//! Cross-platform `PlatformWebView` built on the gtk4-webkit6 fork of `wry`.
//!
//! One wrapper, two embedding strategies:
//!
//! - **Linux**: the fork's WebKitGTK backend is a native GTK4 widget —
//!   `WebViewBuilderExtUnix::build_gtk` appends it into `self.container`.
//! - **Windows**: WebView2 embedded as a child Win32 window (HWND obtained
//!   from the GDK surface), and a tick callback keeps the HWND aligned with
//!   the GTK container.
//!
//! HTML is served through a custom protocol (`marco-preview://localhost/…`)
//! on **both** platforms, so the IPC `Source` URL is never empty (a
//! wry/WebView2 panic otherwise) and relative asset paths
//! (`<img src="./x.png">`) resolve against the document's own URL via
//! ordinary URL relative-resolution — no `<base>` tag and no base URI is
//! ever passed to a WebKit/WebView2 API. See [`build_document_url`] for the
//! URL shape and the `with_custom_protocol` handler installed in
//! [`PlatformWebView::build_initial_webview`] for how it's served.
//!
//! Everything above construction/loading — IPC routing, scroll-sync, hover,
//! find, state snapshots, smooth content updates — is a single code path.

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
/// [`crate::components::viewer::find_engine`]) reports search results via
/// `marco_find:count=<N>,index=<K>` IPC. The payload is parsed into a
/// [`crate::components::viewer::find_engine::FindReport`] before delivery.
type FindReportCallback = Rc<dyn Fn(crate::components::viewer::find_engine::FindReport)>;
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

/// Document URL queued while the webview's deferred first build is pending
/// (see [`PlatformWebView::load_html_with_base`]).
type PendingLoadCell = Rc<RefCell<Option<String>>>;

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

/// RAII guard owning this instance's display-global background CSS provider.
///
/// Each `PlatformWebView` styles its container through a unique
/// `.marco-preview-bg-<id>` class; when the last clone drops, the provider is
/// removed from the display again so providers do not accumulate as webviews
/// (editor tabs, dialogs, detached windows) come and go.
struct BgCssGuard {
    provider: gtk4::CssProvider,
}

impl Drop for BgCssGuard {
    fn drop(&mut self) {
        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_remove_provider_for_display(&display, &self.provider);
        }
    }
}

/// Custom protocol scheme used to serve preview HTML and its local assets on
/// both platforms. On Windows, wry maps this to `http://marco-preview.localhost/`
/// at build time so the IPC `Source` URL is never empty (which would
/// otherwise cause a panic in wry when using `NavigateToString`). Linux
/// (WebKitGTK) needs no such translation — the scheme is used as-is.
const CUSTOM_SCHEME: &str = "marco-preview";

/// Authority (host) component of every `marco-preview://` URL. Fixed at
/// `"localhost"` rather than a per-webview identifier: each `PlatformWebView`
/// registers its own `with_custom_protocol` closure (capturing its own id
/// directly, see [`PlatformWebView::build_initial_webview`]), so the URL
/// itself doesn't need to carry per-webview routing information — and
/// Windows's `marco-preview://localhost/` → `http://marco-preview.localhost/`
/// translation is only proven to work with this exact authority.
const CUSTOM_AUTHORITY: &str = "localhost";

/// Authority used for `WebView::load_url()` calls on Windows *after* the
/// initial build. wry's `marco-preview://localhost/` → `http://…` URI
/// workaround only runs at build time, not on `load_url()`, so reload
/// navigations must use the already-translated HTTP form directly —
/// otherwise the navigation silently fails.
#[cfg(target_os = "windows")]
const CONTENT_URL_RELOAD_AUTHORITY: &str = "http://marco-preview.localhost";

/// Query-string marker identifying a request as "serve the cached top-level
/// document" rather than a local asset. See [`build_document_url`].
const DOC_MARKER_QUERY: &str = "marco_doc=1";

/// Build the `marco-preview://` URL for the top-level preview document.
///
/// `base_uri`, when present, is the `file:///abs/dir/`-shaped string already
/// produced by `preview_helpers::generate_base_uri_from_path` (or polo's
/// equivalent) — its directory portion becomes the URL path, percent-encoded
/// segment by segment. Relative asset references in the loaded HTML then
/// resolve, via ordinary URL relative-resolution (no `<base>` tag involved),
/// to `marco-preview://localhost/<same-dir>/<asset>` — served by the
/// `with_custom_protocol` closure installed in
/// [`PlatformWebView::build_initial_webview`], which reads that path
/// directly off disk. `version` busts any cache the webview backend keeps
/// for the scheme.
fn build_document_url(base_uri: Option<&str>, version: u64) -> String {
    let dir_path = base_uri
        .and_then(|b| b.strip_prefix("file://"))
        .unwrap_or("");
    let encoded_dir = percent_encode_path(dir_path);
    format!(
        "{}://{}{}?{}&v={}",
        CUSTOM_SCHEME, CUSTOM_AUTHORITY, encoded_dir, DOC_MARKER_QUERY, version
    )
}

/// Percent-encode a filesystem path for use as a URL path, preserving `/` as
/// the segment separator. Encodes every byte outside the URL "unreserved"
/// set (letters, digits, `-_.~`) individually, so multi-byte UTF-8 sequences
/// round-trip correctly through [`percent_decode_path`] regardless of
/// codepoint.
fn percent_encode_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for byte in path.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{:02X}", byte)),
        }
    }
    out
}

/// Percent-decode a URL path back into a filesystem path. Decodes into a
/// byte buffer first, then interprets it as UTF-8 lossily — a malformed
/// sequence degrades gracefully instead of panicking.
fn percent_decode_path(path: &str) -> String {
    let bytes = path.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (
                (bytes[i + 1] as char).to_digit(16),
                (bytes[i + 2] as char).to_digit(16),
            ) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Convert a percent-decoded, always-`/`-prefixed URL path into a native
/// filesystem path. No-op on Linux (Unix paths are already `/`-rooted). On
/// Windows, a URL path built from a `C:/Users/…`-shaped base carries an
/// extra leading `/` (mirroring the `file:///C:/…` convention used by
/// `generate_base_uri_from_path`) that must be stripped before the string is
/// a valid Windows path.
fn url_path_to_fs_path(url_path: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        let bytes = url_path.as_bytes();
        if bytes.len() >= 3
            && bytes[0] == b'/'
            && bytes[1].is_ascii_alphabetic()
            && bytes[2] == b':'
        {
            return url_path[1..].to_string();
        }
    }
    url_path.to_string()
}

/// Best-effort `Content-Type` for a served local asset, by extension. Falls
/// back to `application/octet-stream` for anything outside this deliberately
/// short list — the realistic set of images Markdown embeds.
fn mime_type_for_path(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod protocol_tests {
    use super::*;

    #[test]
    fn smoke_build_document_url_no_base() {
        let url = build_document_url(None, 3);
        assert_eq!(url, "marco-preview://localhost?marco_doc=1&v=3");
    }

    #[test]
    fn smoke_build_document_url_with_base() {
        let url = build_document_url(Some("file:///home/user/docs/"), 1);
        assert_eq!(
            url,
            "marco-preview://localhost/home/user/docs/?marco_doc=1&v=1"
        );
    }

    #[test]
    fn smoke_percent_encode_decode_roundtrip_ascii() {
        let original = "/home/user/my docs/a-b_c.d~e/img.png";
        let encoded = percent_encode_path(original);
        assert_eq!(percent_decode_path(&encoded), original);
    }

    #[test]
    fn smoke_percent_encode_decode_roundtrip_unicode() {
        let original = "/home/user/dossiers/résumé 日本語/img.png";
        let encoded = percent_encode_path(original);
        assert!(!encoded.contains(' '));
        assert_eq!(percent_decode_path(&encoded), original);
    }

    #[test]
    fn smoke_percent_encode_preserves_slash() {
        let encoded = percent_encode_path("/a/b/c");
        assert_eq!(encoded, "/a/b/c");
    }

    #[test]
    fn smoke_mime_type_for_path_known_and_unknown() {
        assert_eq!(mime_type_for_path("/x/img.PNG"), "image/png");
        assert_eq!(mime_type_for_path("/x/img.jpeg"), "image/jpeg");
        assert_eq!(
            mime_type_for_path("/x/data.bin"),
            "application/octet-stream"
        );
        assert_eq!(mime_type_for_path("/x/noext"), "application/octet-stream");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn smoke_url_path_to_fs_path_linux_passthrough() {
        assert_eq!(
            url_path_to_fs_path("/home/user/img.png"),
            "/home/user/img.png"
        );
    }
}

/// Cross-platform PlatformWebView wrapper
#[derive(Clone)]
pub struct PlatformWebView {
    /// Reference-counted RAII guard for this instance's `WEBVIEW_HTML_MAP`
    /// slot. The map entry is removed when the last clone is dropped — see
    /// [`IdGuard::drop`]. Only the Windows custom-protocol path reads the id.
    #[allow(dead_code)]
    id_guard: Rc<IdGuard>,
    /// Monotonically increasing counter appended to the document URL as
    /// `&v=N`. Each increment produces a unique URL so neither backend can
    /// serve a cached response for the custom-protocol scheme.
    load_version: Rc<std::cell::Cell<u64>>,
    /// Document URL waiting for the deferred first build. Once the webview
    /// exists, loads go straight through `WebView::load_url`.
    pending_load: PendingLoadCell,
    pub inner: std::rc::Rc<std::cell::RefCell<Option<wry::WebView>>>,
    pub container: gtk4::Box,
    #[cfg(target_os = "windows")]
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
    /// is at −32000,−32000 during loading. Wrapped in a guard that removes
    /// the provider from the display when the last clone drops.
    bg_css_provider: Rc<BgCssGuard>,
}

impl PlatformWebView {
    /// Construct a new `PlatformWebView` parented to any GTK4 top-level
    /// window (`ApplicationWindow`, plain `Window`, dialog `Window`, ...).
    ///
    /// On Linux the window handle is only kept for API parity — the webview
    /// itself is built into `self.container` as a native GTK child. On
    /// Windows the window's Win32 HWND hosts the WebView2 child window,
    /// which is why this accepts `&impl IsA<gtk4::Window>` (`gtk4::Window`
    /// implements `Native`, needed for `gdk_win32_surface_get_handle`).
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
        let bg_class = format!("marco-preview-bg-{}", id);
        container.add_css_class(&bg_class);
        let bg_css_provider = Rc::new(BgCssGuard {
            provider: gtk4::CssProvider::new(),
        });
        bg_css_provider
            .provider
            .load_from_data(&format!(".{} {{ background-color: #1e1e1e; }}", bg_class));
        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &bg_css_provider.provider,
                gtk4::STYLE_PROVIDER_PRIORITY_USER,
            );
        }

        // Attempt to obtain parent HWND and parent_handle; on failure, keep inner None
        #[cfg(target_os = "windows")]
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
        // Linux needs none of this — the webview is a real GTK child widget
        // and the layout system sizes it.
        #[cfg(target_os = "windows")]
        {
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
        }

        Self {
            id_guard: Rc::new(IdGuard(id)),
            load_version,
            pending_load: Rc::new(RefCell::new(None)),
            inner: webview,
            container,
            #[cfg(target_os = "windows")]
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
    /// parsed into a [`crate::components::viewer::find_engine::FindReport`]
    /// before delivery; malformed payloads are dropped.
    ///
    /// Pair this with [`crate::components::viewer::find_engine::install`] and
    /// the `search` / `next` / `prev` / `clear` helpers.
    pub fn set_find_report_callback<F>(&self, callback: F)
    where
        F: Fn(crate::components::viewer::find_engine::FindReport) + 'static,
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
        // Linux: no-op beyond recording the flag — the loading overlay is a
        // real GTK overlay above a real GTK webview widget, so nothing needs
        // to move. Windows: immediately push the HWND off-screen so we don't
        // wait for the next tick-callback iteration (~16 ms) before the GTK
        // overlay becomes visible.
        #[cfg(target_os = "windows")]
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
            ".marco-preview-bg-{} {{ background-color: rgb({},{},{}); }}",
            self.id_guard.0,
            (color.red() * 255.0) as u32,
            (color.green() * 255.0) as u32,
            (color.blue() * 255.0) as u32,
        );
        self.bg_css_provider.provider.load_from_data(&css_data);
    }

    /// Load `html` into the preview, with an optional base directory
    /// (`base_uri`, `file:///abs/dir/`-shaped) so relative asset references
    /// resolve against it. Both HTML and any local assets are served through
    /// the `marco-preview://` custom protocol — see [`build_document_url`].
    pub fn load_html_with_base(&self, html: &str, base_uri: Option<&str>) {
        let v = self.load_version.get().wrapping_add(1);
        self.load_version.set(v);

        // Store the raw HTML (no `<base>` injection needed — the document's
        // own URL already encodes its directory) so the custom-protocol
        // handler can serve it.
        html_map()
            .lock()
            .unwrap()
            .insert(self.id_guard.0, html.as_bytes().to_vec());

        let doc_url = build_document_url(base_uri, v);

        if let Some(view) = self.inner.borrow().as_ref() {
            self.navigate(view, &doc_url);
            return;
        }

        // Defer first-time creation until the GTK container is mapped *and*
        // has a non-trivial allocation (~4.8 s budget, 60 fps). Only the
        // newest pending URL is kept.
        *self.pending_load.borrow_mut() = Some(doc_url);
        let me = self.clone();
        crate::components::viewer::allocation_wait::run_when_allocated(
            &self.container,
            crate::components::viewer::allocation_wait::DEFAULT_MAX_RETRIES,
            move || {
                // A previous deferred load may have already built the
                // webview — just navigate it instead of building a second one.
                if let Some(view) = me.inner.borrow().as_ref() {
                    if let Some(url) = me.pending_load.borrow_mut().take() {
                        me.navigate(view, &url);
                    }
                    return;
                }
                me.build_initial_webview();
            },
        );
    }

    /// Navigate an already-built webview to `doc_url`. On Windows this uses
    /// the already-HTTP-translated authority (`load_url` doesn't apply
    /// wry's build-time scheme translation); on Linux the `marco-preview://`
    /// URL is used as-is.
    fn navigate(&self, view: &wry::WebView, doc_url: &str) {
        #[cfg(target_os = "windows")]
        let target = doc_url.replacen(
            &format!("{}://{}", CUSTOM_SCHEME, CUSTOM_AUTHORITY),
            CONTENT_URL_RELOAD_AUTHORITY,
            1,
        );
        #[cfg(target_os = "linux")]
        let target = doc_url.to_string();

        if let Err(e) = view.load_url(&target) {
            log::error!("[wry] Failed to load preview document: {}", e);
        }
    }

    /// Build the wry `WebView` for the first time — on Linux as a native GTK
    /// child of `self.container` (`build_gtk`), on Windows as a child Win32
    /// window inside it. Must only be called when:
    ///
    /// 1. `self.inner` is `None` (no existing WebView).
    /// 2. `self.container` is mapped and has `allocated_width/height > 1`.
    ///
    /// Both preconditions are enforced by `load_html_with_base` via
    /// [`allocation_wait::run_when_allocated`].
    fn build_initial_webview(&self) {
        // The document URL this build is for — set by `load_html_with_base`
        // before deferring. Falls back to a bare "no document yet" URL if
        // this is somehow called without a pending load.
        let initial_url = self
            .pending_load
            .borrow()
            .clone()
            .unwrap_or_else(|| build_document_url(None, 0));

        #[cfg(target_os = "windows")]
        let rect = {
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

            rect
        };

        let builder = wry::WebViewBuilder::new()
            .with_background_color(self.bg_color.get())
            .with_custom_protocol(CUSTOM_SCHEME.to_string(), {
                let id = self.id_guard.0;
                move |_webview_id, req| {
                    // The document request carries `?marco_doc=1&v=N` (see
                    // `build_document_url`); everything else is a local asset
                    // reference that resolved, via ordinary URL
                    // relative-resolution, to a path under the document's
                    // own directory (or an absolute path — both cases are
                    // just "read this file").
                    let is_doc = req
                        .uri()
                        .query()
                        .map(|q| q.split('&').any(|kv| kv == "marco_doc=1"))
                        .unwrap_or(false);

                    if is_doc {
                        let html_bytes = html_map()
                            .lock()
                            .unwrap()
                            .get(&id)
                            .cloned()
                            .unwrap_or_default();
                        return wry::http::Response::builder()
                            .header("Content-Type", "text/html; charset=utf-8")
                            .header("Access-Control-Allow-Origin", "*")
                            .body(std::borrow::Cow::Owned(html_bytes))
                            .unwrap();
                    }

                    let fs_path = url_path_to_fs_path(&percent_decode_path(req.uri().path()));
                    match std::fs::read(&fs_path) {
                        Ok(bytes) => wry::http::Response::builder()
                            .header("Content-Type", mime_type_for_path(&fs_path))
                            .header("Access-Control-Allow-Origin", "*")
                            .body(std::borrow::Cow::Owned(bytes))
                            .unwrap(),
                        Err(e) => {
                            log::debug!(
                                "[wry] custom protocol: failed to read local asset '{}': {}",
                                fs_path,
                                e
                            );
                            wry::http::Response::builder()
                                .status(404)
                                .body(std::borrow::Cow::Owned(Vec::new()))
                                .unwrap()
                        }
                    }
                }
            });

        #[cfg(target_os = "windows")]
        let builder = builder.with_bounds(rect).with_url(&initial_url);
        #[cfg(target_os = "linux")]
        let builder = builder.with_url(&initial_url);

        let builder = builder
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
                            crate::components::viewer::find_engine::parse_report(payload)
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
                                if let Err(e) = native_print_dialog(view) {
                                    log::warn!(
                                        "[wry] marco_print: native print dialog failed ({}); falling back to wry view.print()",
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
                        if let Err(e) = crate::components::viewer::preview_helpers::open_external_uri(&uri) {
                            log::warn!("[wry] failed to open external URI '{}': {}", uri, e);
                        }
                        return false;
                    }
                    true
                }
            });

        #[cfg(target_os = "windows")]
        let built = builder.build_as_child(&*self.parent_handle);
        #[cfg(target_os = "linux")]
        let built = {
            use wry::WebViewBuilderExtUnix;
            builder.build_gtk(&self.container)
        };

        match built {
            Ok(view) => {
                #[cfg(target_os = "linux")]
                {
                    use webkit6::prelude::*;
                    use wry::WebViewExtUnix;
                    let wk = view.webview();
                    // Mirror the historical webkit6 backend configuration.
                    // The primary load path no longer uses file:// URLs at
                    // all (assets are served through the custom protocol),
                    // but these flags still matter for explicit `file://`
                    // links written directly in Markdown, which bypass the
                    // document's base entirely and hit WebKit's native
                    // file:// loader.
                    if let Some(settings) = WebViewExt::settings(&wk) {
                        settings.set_allow_file_access_from_file_urls(true);
                        settings.set_allow_universal_access_from_file_urls(true);
                        settings.set_auto_load_images(true);
                    }
                    // Pin the widget direction to LTR so the WebKitGTK overlay
                    // scrollbar stays on the physical right; content direction
                    // is handled via <body dir="rtl"> in the HTML.
                    gtk4::prelude::WidgetExt::set_direction(&wk, gtk4::TextDirection::Ltr);
                }
                // The initial document was already requested via
                // `.with_url(&initial_url)` above — nothing left to flush.
                self.pending_load.borrow_mut().take();
                *self.inner.borrow_mut() = Some(view);
                log::info!("wry WebView successfully created for initial load");
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
    /// If the underlying webview has not been built yet (no preceding
    /// [`Self::load_html_with_base`] call has succeeded), the update is
    /// silently dropped — callers must always do a full load first.
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
            // Prefer the platform-native system print dialog: a WebKit
            // PrintOperation dialog on Linux, and on Windows the WebView2
            // system dialog (top-level Win32 window owned by Marco) rather
            // than wry's `view.print()`, which simply calls `window.print()`
            // JS and renders the print preview *inside* the live preview area.
            match native_print_dialog(view) {
                Ok(()) => return,
                Err(e) => {
                    log::warn!(
                        "native print dialog failed ({}); falling back to wry view.print()",
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
    #[cfg(target_os = "windows")]
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
        crate::components::viewer::print_driver_windows::print_to_pdf(
            view,
            output_path,
            paper,
            orientation,
            margin_mm,
        )
    }
}

/// Open the platform-native print dialog for the current page.
///
/// - **Linux**: the fork's `print()` runs a WebKit `PrintOperation` dialog —
///   the same native GTK dialog the old webkit6 backend used.
/// - **Windows**: `ShowPrintUI(SYSTEM)` via WebView2 COM; call sites fall
///   back to `view.print()` on older runtimes.
fn native_print_dialog(view: &wry::WebView) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    return show_system_print_ui(view);
    #[cfg(target_os = "linux")]
    return view
        .print()
        .map_err(|e| format!("WebKit print dialog failed: {}", e));
}

/// Open the WebView2 *system* print dialog (top-level Win32 window owned by
/// the host) instead of the in-WebView browser print preview that
/// `wry::WebView::print()` triggers via `window.print()`.
///
/// This casts the underlying `ICoreWebView2` to `ICoreWebView2_16` and calls
/// `ShowPrintUI(COREWEBVIEW2_PRINT_DIALOG_KIND_SYSTEM)`.  Returns `Err` on
/// older WebView2 runtimes that don't expose the v16 interface so the caller
/// can fall back to the legacy path.
#[cfg(target_os = "windows")]
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
