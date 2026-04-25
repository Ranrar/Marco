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
use std::sync::{Mutex, OnceLock};
use std::sync::atomic::{AtomicU64, Ordering};

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

/// Monotonic counter for assigning unique IDs to each PlatformWebView instance.
static WEBVIEW_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Per-webview HTML content storage.
/// Keyed by PlatformWebView.id so the custom protocol handler always serves
/// the correct HTML even when multiple webviews coexist (one per editor tab).
static WEBVIEW_HTML_MAP: OnceLock<Mutex<std::collections::HashMap<u64, Vec<u8>>>> =
    OnceLock::new();

fn html_map() -> &'static Mutex<std::collections::HashMap<u64, Vec<u8>>> {
    WEBVIEW_HTML_MAP.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
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
    /// Unique ID for this instance — used as key in `WEBVIEW_HTML_MAP`.
    id: u64,
    /// Monotonically increasing counter appended to the reload URL as `?v=N`.
    /// Each increment produces a unique URL so WebView2 cannot serve a cached response.
    load_version: Rc<std::cell::Cell<u64>>,
    pub inner: std::rc::Rc<std::cell::RefCell<Option<wry::WebView>>>,
    pub container: gtk4::Box,
    parent_handle: std::rc::Rc<ParentWindowHandle>,
    pub bg_color: std::rc::Rc<std::cell::Cell<(u8, u8, u8, u8)>>,
    pub gtk_window: gtk4::ApplicationWindow,
    scroll_report_callback: ScrollReportCallbackCell,
    local_md_link_callback: LocalMdLinkCallbackCell,
}

impl PlatformWebView {
    pub fn new(window: &gtk4::ApplicationWindow) -> Self {
        use gtk4::prelude::WidgetExt;

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
        container.add_tick_callback(move |_, _| {
            if let (Some(container), Some(win), Some(view)) = (container_weak.upgrade(), window_weak.upgrade(), webview_for_tick.borrow().as_ref()) {
                // When the GTK container is not mapped (e.g. the Stack switched to code_preview),
                // move the native Win32 WebView off-screen so it does not overdraw other GTK widgets.
                if !container.is_mapped() {
                    let _ = view.set_bounds(wry::Rect {
                        position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(
                            -32000.0,
                            -32000.0,
                        )),
                        size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(1.0, 1.0)),
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
            id,
            load_version,
            inner: webview,
            container,
            parent_handle: parent_handle_rc,
            bg_color,
            gtk_window: window.clone(),
            scroll_report_callback,
                    local_md_link_callback,
        }
    }

    pub fn set_scroll_report_callback<F: Fn(f64) + 'static>(&self, callback: F) {
        *self.scroll_report_callback.borrow_mut() = Some(Rc::new(callback));
    }

    pub fn set_local_md_link_handler<F: Fn(String, Option<String>) + 'static>(&self, callback: F) {
        *self.local_md_link_callback.borrow_mut() = Some(Rc::new(callback));
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
            .insert(self.id, final_html.into_bytes());

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

        // If the WebView is not ready yet (early call before the first allocation),
        // store the HTML and load it after the widget is realized by forcing an
        // initial creation now with a minimal rect.
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

        let rect = wry::Rect {
            position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(
                origin_in_window.0 + offset_x - 1.0,
                origin_in_window.1 + offset_y,
            )),
            size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(
                alloc.width().max(100) as f64 + 1.0,
                alloc.height().max(100) as f64,
            )),
        };

        log::debug!("[wry] initial_create origin_in_window=({}, {}), alloc=({}, {}), rect_pos=({}, {}), rect_size=({}, {})",
            origin_in_window.0, origin_in_window.1, alloc.x(), alloc.y(),
            origin_in_window.0 + offset_x - 1.0, origin_in_window.1 + offset_y,
            alloc.width().max(100) + 1, alloc.height().max(100)
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
                let id = self.id;
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
                let callback = self.scroll_report_callback.clone();
                move |req: wry::http::Request<String>| {
                    let msg = req.body().as_str();
                    if let Some(scroll_data) = msg.strip_prefix("marco_scroll:") {
                        if let Ok(percentage) = scroll_data.parse::<f64>() {
                            let cb_opt = callback.borrow().clone();
                            if let Some(cb) = cb_opt {
                                let percentage = percentage.clamp(0.0, 1.0);
                                gtk4::glib::MainContext::default()
                                    .invoke_local(move || cb(percentage));
                            }
                        }
                    }
                }
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
    let without_query = without_fragment.split('?').next().unwrap_or(without_fragment);
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
