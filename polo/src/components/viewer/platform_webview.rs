//! Cross-platform WebView wrapper for Polo, built on the gtk4-webkit6 wry fork.
//!
//! One wrapper, two embedding strategies:
//!
//! - **Linux**: the fork's WebKitGTK backend is a native GTK4 widget —
//!   `WebViewBuilderExtUnix::build_gtk` appends it into `self.container` at
//!   construction time.
//! - **Windows**: WebView2 embedded as a child Win32 window (HWND from the GDK
//!   surface), built lazily on the first load, and a tick callback keeps the
//!   HWND aligned with the GTK container.
//!
//! HTML is served through a custom protocol (`polo-preview://localhost/…`) on
//! **both** platforms — this bypasses WebView2's ~2 MB `NavigateToString`
//! content limit (which would otherwise silently drop large markdown files),
//! and relative asset paths (`<img src="./x.png">`) resolve against the
//! document's own URL via ordinary URL relative-resolution — no `<base>` tag
//! and no base URI is ever passed to a WebKit/WebView2 API. See
//! [`build_document_url`] for the URL shape.
//!
//! Navigation policy (external links → system browser, local `.md` links →
//! callback) and page-load notification are a single code path on both
//! platforms via wry's `with_navigation_handler` / `with_on_page_load_handler`.

use gtk4::prelude::*;

type NavigationHandlerCell =
    std::rc::Rc<std::cell::RefCell<Option<Box<dyn Fn(String, Option<String>)>>>>;
type LoadFinishedHandlerCell = std::rc::Rc<std::cell::RefCell<Option<Box<dyn Fn()>>>>;

/// Unified WebView wrapper exposed to the rest of the codebase.
#[derive(Clone)]
pub struct PlatformWebView {
    inner: std::rc::Rc<std::cell::RefCell<Option<wry::WebView>>>,
    container: gtk4::Box,
    #[cfg(target_os = "windows")]
    parent_handle: std::rc::Rc<ParentWindowHandle>,
    bg_color: std::rc::Rc<std::cell::Cell<(u8, u8, u8, u8)>>,
    #[cfg(target_os = "windows")]
    gtk_window: gtk4::ApplicationWindow,
    navigation_handler: NavigationHandlerCell,
    load_finished_handler: LoadFinishedHandlerCell,
    /// When `true` the tick callback keeps the wry HWND at −32000,−32000 so
    /// the GTK loading-overlay frame is visible above it (Windows-only effect;
    /// on Linux the overlay stacks above the webview widget natively).
    #[allow(dead_code)] // Only read by the Windows tick callback.
    is_offscreen_for_loading: std::rc::Rc<std::cell::Cell<bool>>,
    /// Monotonically increasing counter appended to the document URL as
    /// `&v=N` so neither backend serves a cached response for the custom
    /// protocol.
    load_version: std::rc::Rc<std::cell::Cell<u64>>,
    /// GTK CSS provider used to paint the container background to match the
    /// WebView background colour, preventing a white flash during loading.
    bg_css_provider: std::rc::Rc<gtk4::CssProvider>,
}

impl PlatformWebView {
    pub fn new(window: &gtk4::ApplicationWindow) -> Result<Self, String> {
        use gtk4::prelude::WidgetExt;

        // Ensure the GTK window is realized so a surface/handle exists.
        WidgetExt::realize(window);

        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        container.set_vexpand(true);
        container.set_hexpand(true);

        let webview: std::rc::Rc<std::cell::RefCell<Option<wry::WebView>>> =
            std::rc::Rc::new(std::cell::RefCell::new(None));
        let bg_color = std::rc::Rc::new(std::cell::Cell::new((30u8, 30u8, 30u8, 255u8)));
        let navigation_handler: NavigationHandlerCell =
            std::rc::Rc::new(std::cell::RefCell::new(None));
        let load_finished_handler: LoadFinishedHandlerCell =
            std::rc::Rc::new(std::cell::RefCell::new(None));
        let is_offscreen_for_loading = std::rc::Rc::new(std::cell::Cell::new(false));

        // Paint the container background to match the theme so there is no
        // white flash while the page is loading.
        container.add_css_class("polo-preview-bg");
        let bg_css_provider = std::rc::Rc::new(gtk4::CssProvider::new());
        bg_css_provider.load_from_data(".polo-preview-bg { background-color: #1e1e1e; }");
        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &*bg_css_provider,
                gtk4::STYLE_PROVIDER_PRIORITY_USER,
            );
        }

        // Windows: obtain the Win32 parent HWND for the lazy build in
        // `load_html_with_base`.
        #[cfg(target_os = "windows")]
        let parent_handle = {
            use raw_window_handle::{
                RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
            };
            use std::num::NonZeroIsize;

            let surface = window
                .surface()
                .ok_or_else(|| "Failed to get GDK surface".to_string())?;

            use gdk4_win32::Win32Surface;
            let win32_surface: &Win32Surface = surface
                .downcast_ref()
                .ok_or_else(|| "Failed to downcast to Win32Surface".to_string())?;

            let hwnd_ptr = unsafe {
                gdk4_win32::ffi::gdk_win32_surface_get_handle(win32_surface.as_ptr() as *mut _)
            };
            let hwnd =
                NonZeroIsize::new(hwnd_ptr as isize).ok_or_else(|| "HWND is null".to_string())?;

            let win_handle = Win32WindowHandle::new(hwnd);
            let raw_window = RawWindowHandle::Win32(win_handle);
            let raw_display = RawDisplayHandle::Windows(WindowsDisplayHandle::new());

            std::rc::Rc::new(ParentWindowHandle {
                window: unsafe { raw_window_handle::WindowHandle::borrow_raw(raw_window) },
                display: unsafe { raw_window_handle::DisplayHandle::borrow_raw(raw_display) },
            })
        };

        // Windows: keep WebView bounds in sync with the GTK container on every
        // frame. Linux needs none of this — the webview is a real GTK child
        // widget and the layout system sizes it.
        #[cfg(target_os = "windows")]
        {
            let webview_for_tick = webview.clone();
            let container_weak = container.downgrade();
            let window_weak = window.downgrade();
            let is_offscreen_tick = is_offscreen_for_loading.clone();
            container.add_tick_callback(move |_, _| {
                if let (Some(container), Some(win), Some(view)) = (
                    container_weak.upgrade(),
                    window_weak.upgrade(),
                    webview_for_tick.borrow().as_ref(),
                ) {
                    // When the GTK container is not mapped, or a loading
                    // operation is in progress, move the native Win32 WebView
                    // off-screen so the GTK loading-overlay frame is visible.
                    if !container.is_mapped() || is_offscreen_tick.get() {
                        let (w, h) = if is_offscreen_tick.get() {
                            let alloc = container.allocation();
                            (
                                alloc.width().max(100) as f64,
                                alloc.height().max(100) as f64,
                            )
                        } else {
                            (1.0, 1.0)
                        };
                        let _ = view.set_bounds(wry::Rect {
                            position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(
                                -32000.0, -32000.0,
                            )),
                            size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(w, h)),
                        });
                        return gtk4::glib::ControlFlow::Continue;
                    }

                    let alloc = container.allocation();
                    let (offset_x, offset_y) = if win.is_maximized() {
                        (0.0, 0.0)
                    } else {
                        (14.0, 12.0)
                    };

                    let origin_in_window = match container.translate_coordinates(&win, 0.0, 0.0) {
                        Some((x, y)) => (x, y),
                        None => (alloc.x() as f64, alloc.y() as f64),
                    };

                    let rect = wry::Rect {
                        position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(
                            origin_in_window.0 + offset_x - 1.0,
                            origin_in_window.1 + offset_y,
                        )),
                        size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(
                            alloc.width().max(1) as f64 + 1.0,
                            alloc.height().max(1) as f64,
                        )),
                    };
                    if let Err(e) = view.set_bounds(rect) {
                        log::debug!("wry set_bounds failed: {}", e);
                    }
                }
                gtk4::glib::ControlFlow::Continue
            });
        }

        let this = Self {
            inner: webview,
            container,
            #[cfg(target_os = "windows")]
            parent_handle,
            bg_color,
            #[cfg(target_os = "windows")]
            gtk_window: window.clone(),
            navigation_handler,
            load_finished_handler,
            is_offscreen_for_loading,
            load_version: std::rc::Rc::new(std::cell::Cell::new(0u64)),
            bg_css_provider,
        };

        // Linux: the webview is a plain GTK widget — build it right away so
        // handlers and loads have a live target. (Windows builds lazily on the
        // first load because the HWND child needs an allocated container.)
        #[cfg(target_os = "linux")]
        this.build_webview_gtk()?;

        Ok(this)
    }

    /// Build the wry WebView as a native GTK child of `self.container` (Linux).
    ///
    /// Built blank (no real content — `polo_html()` is still empty at this
    /// point); the first [`Self::load_html_with_base`] call navigates it to
    /// the actual document.
    #[cfg(target_os = "linux")]
    fn build_webview_gtk(&self) -> Result<(), String> {
        use wry::{WebViewBuilderExtUnix, WebViewExtUnix};

        let builder = wry::WebViewBuilder::new()
            .with_background_color(self.bg_color.get())
            .with_url(build_document_url(None, 0))
            .with_custom_protocol(POLO_SCHEME.to_string(), polo_protocol_handler)
            .with_navigation_handler({
                let nav_handler = self.navigation_handler.clone();
                move |uri: String| wry_navigation_handler(&uri, &nav_handler)
            })
            .with_on_page_load_handler({
                let load_handler = self.load_finished_handler.clone();
                move |event, _url: String| {
                    if matches!(event, wry::PageLoadEvent::Finished) {
                        if let Some(f) = load_handler.borrow().as_ref() {
                            f();
                        }
                    }
                }
            });

        let view = builder
            .build_gtk(&self.container)
            .map_err(|e| format!("Failed to build wry WebView (gtk): {}", e))?;

        {
            use webkit6::prelude::*;
            let wk = view.webview();
            // Mirror the historical webkit6 configuration. The primary load
            // path no longer uses file:// URLs at all (assets are served
            // through the custom protocol), but these flags still matter for
            // explicit `file://` links written directly in Markdown, which
            // bypass the document's base entirely and hit WebKit's native
            // file:// loader.
            if let Some(settings) = WebViewExt::settings(&wk) {
                settings.set_allow_file_access_from_file_urls(true);
                settings.set_allow_universal_access_from_file_urls(true);
                settings.set_auto_load_images(true);
                settings.set_enable_developer_extras(false);
                settings.set_javascript_can_access_clipboard(false);
                settings.set_enable_write_console_messages_to_stdout(false);
            }
            // Pin the widget direction to LTR so the WebKitGTK overlay
            // scrollbar stays on the physical right; content direction is
            // handled via <body dir="rtl"> in the HTML.
            gtk4::prelude::WidgetExt::set_direction(&wk, gtk4::TextDirection::Ltr);
        }

        *self.inner.borrow_mut() = Some(view);
        Ok(())
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
        // Keep the GTK container background in sync so there is no white
        // flash while the page is loading.
        let css_data = format!(
            ".polo-preview-bg {{ background-color: rgb({},{},{}); }}",
            (color.red() * 255.0) as u32,
            (color.green() * 255.0) as u32,
            (color.blue() * 255.0) as u32,
        );
        self.bg_css_provider.load_from_data(&css_data);
    }

    /// Load `html` into the preview, with an optional base directory
    /// (`base_uri`, `file:///abs/dir/`-shaped) so relative asset references
    /// resolve against it. Both HTML and any local assets are served through
    /// the `polo-preview://` custom protocol — see [`build_document_url`].
    pub fn load_html_with_base(&self, html: &str, base_uri: Option<&str>) {
        // Store the raw HTML (no `<base>` injection needed — the document's
        // own URL already encodes its directory) so the custom-protocol
        // handler can serve it. Using a custom protocol instead of
        // `load_html` / `NavigateToString` also bypasses WebView2's ~2 MB
        // content limit that would otherwise silently drop large markdown
        // files.
        if let Ok(mut guard) = polo_html().lock() {
            *guard = html.as_bytes().to_vec();
        } else {
            log::error!("[polo] polo_html mutex poisoned — cannot update HTML");
            return;
        }

        let v = self.load_version.get().wrapping_add(1);
        self.load_version.set(v);
        let doc_url = build_document_url(base_uri, v);

        #[cfg(target_os = "linux")]
        {
            // The webview is always already built (created eagerly in
            // `new()`). Defer to the next event-loop idle so the loading
            // overlay gets one frame to paint first (parity with the
            // historical webkit6 behaviour).
            let inner = self.inner.clone();
            gtk4::glib::idle_add_local_once(move || {
                if let Some(view) = inner.borrow().as_ref() {
                    navigate(view, &doc_url);
                } else {
                    log::error!("[polo] load_html_with_base: WebView not built");
                }
            });
        }

        #[cfg(target_os = "windows")]
        {
            if self.inner.borrow().is_some() {
                // Defer to the next event-loop idle: this gives the loading
                // overlay one frame to paint before WebView2 starts replacing
                // the current page.
                let inner = self.inner.clone();
                gtk4::glib::idle_add_local_once(move || {
                    if let Some(view) = inner.borrow().as_ref() {
                        navigate(view, &doc_url);
                    }
                });
                return;
            }

            // First load: build the WebView now.
            // Build the WebView off-screen if a loading operation is in progress
            // so the GTK loading-overlay frame stays visible until the page loads.
            let rect = if self.is_offscreen_for_loading.get() {
                let alloc = self.container.allocation();
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
                let alloc = self.container.allocation();
                let (offset_x, offset_y) = if self.gtk_window.is_maximized() {
                    (0.0, 0.0)
                } else {
                    (14.0, 12.0)
                };
                let origin_in_window =
                    match self
                        .container
                        .translate_coordinates(&self.gtk_window, 0.0, 0.0)
                    {
                        Some((x, y)) => (x, y),
                        None => (alloc.x() as f64, alloc.y() as f64),
                    };
                wry::Rect {
                    position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(
                        origin_in_window.0 + offset_x - 1.0,
                        origin_in_window.1 + offset_y,
                    )),
                    size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(
                        alloc.width().max(100) as f64 + 1.0,
                        alloc.height().max(100) as f64,
                    )),
                }
            };

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
                .with_url(&doc_url)
                .with_custom_protocol(POLO_SCHEME.to_string(), polo_protocol_handler)
                .with_navigation_handler({
                    let nav_handler = self.navigation_handler.clone();
                    move |uri: String| wry_navigation_handler(&uri, &nav_handler)
                })
                .with_on_page_load_handler({
                    let load_handler = self.load_finished_handler.clone();
                    move |event, _url: String| {
                        if matches!(event, wry::PageLoadEvent::Finished) {
                            if let Some(f) = load_handler.borrow().as_ref() {
                                f();
                            }
                        }
                    }
                })
                .build_as_child(&*self.parent_handle)
            {
                Ok(view) => {
                    *self.inner.borrow_mut() = Some(view);
                }
                Err(e) => log::error!("Failed to build wry WebView for initial load: {}", e),
            }
        }
    }

    /// Setup link interception policy for the WebView.
    ///
    /// - External links (http/https/mailto/www) are opened in the system browser.
    /// - Local `.md` / `.markdown` file links call `on_local_md(path, fragment)`.
    ///
    /// All other navigation (in-page anchors, resources) is passed through.
    pub fn setup_link_policy(&self, on_local_md: impl Fn(String, Option<String>) + 'static) {
        *self.navigation_handler.borrow_mut() = Some(Box::new(on_local_md));
    }

    /// Run `f` once each time the WebView finishes loading a page.
    ///
    /// Used to hide the loading overlay when the new HTML is actually on
    /// screen, rather than when the load was merely *queued*.
    pub fn connect_load_finished<F: Fn() + 'static>(&self, f: F) {
        *self.load_finished_handler.borrow_mut() = Some(Box::new(f));
    }

    /// Move the wry HWND off-screen (`offscreen = true`) so the GTK loading-
    /// overlay frame is visible during rendering, or restore it to its normal
    /// position (`offscreen = false`) once the page has loaded.
    ///
    /// On Linux this only records the flag — the loading overlay is a real
    /// GTK overlay above a real GTK webview widget, so nothing needs to move.
    #[allow(dead_code)] // Wired up by the Windows-only offscreen hook.
    pub fn set_offscreen_for_loading(&self, offscreen: bool) {
        self.is_offscreen_for_loading.set(offscreen);
        // Immediately push the HWND off-screen so we don't wait for the next
        // tick-callback iteration (~16 ms) before the GTK overlay becomes visible.
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

    #[allow(dead_code)]
    pub fn evaluate_script(&self, script: &str) {
        if let Some(view) = self.inner.borrow().as_ref() {
            if let Err(e) = view.evaluate_script(script) {
                log::error!("JavaScript evaluation failed: {}", e);
            }
        }
    }

    /// Open the native print dialog for the current page.
    ///
    /// - **Linux**: injects `@media print` CSS, runs a WebKit `PrintOperation`
    ///   dialog through the escape hatch, then removes the injected CSS.
    /// - **Windows**: prefers the WebView2 system print dialog
    ///   (`ICoreWebView2_16::ShowPrintUI(SYSTEM)`), falls back to
    ///   `view.print()`, then `window.print()` JS.
    #[cfg(target_os = "linux")]
    pub fn print(&self, parent: Option<&gtk4::Window>) {
        use marco_shared::logic::print_css::make_print_export_css;
        // No fixed paper size — the user configures paper in the print dialog.
        let css = make_print_export_css("", "", false);
        let css_escaped = css
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n");
        let inject_js = format!(
            r#"(function(){{var el=document.getElementById('polo-print-css');if(!el){{el=document.createElement('style');el.id='polo-print-css';document.head.appendChild(el);}}el.textContent="{}";}})();"#,
            css_escaped
        );
        self.evaluate_script(&inject_js);

        if let Some(view) = self.inner.borrow().as_ref() {
            use wry::WebViewExtUnix;
            let print_op = webkit6::PrintOperation::new(&view.webview());
            let _ = print_op.run_dialog(parent);
        } else {
            log::warn!("[polo] print: WebView not ready; ignoring");
        }

        self.evaluate_script(
            "(function(){var el=document.getElementById('polo-print-css');if(el)el.remove();})()",
        );
    }

    /// Trigger the browser print UI for the current page content (Windows).
    #[cfg(target_os = "windows")]
    pub fn print(&self, _parent: Option<&gtk4::Window>) {
        if let Some(view) = self.inner.borrow().as_ref() {
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
                    "[polo] wry print() failed ({}); falling back to window.print()",
                    e
                );
                self.evaluate_script("window.print();");
            }
            return;
        }

        log::warn!("[polo] print: WebView not ready; ignoring");
    }
}

/// Custom protocol scheme used to serve preview HTML and its local assets on
/// both platforms. On Windows, wry maps this to `http://polo-preview.localhost/`
/// at build time — this also bypasses WebView2's ~2 MB `NavigateToString`
/// content limit, which would otherwise silently drop large markdown files.
/// Linux (WebKitGTK) needs no such translation — the scheme is used as-is.
const POLO_SCHEME: &str = "polo-preview";

/// Authority (host) component of every `polo-preview://` URL. Fixed at
/// `"localhost"` — polo has exactly one `PlatformWebView` per process (single
/// global HTML store, see [`polo_html`]), so the URL doesn't need to carry
/// per-instance routing information, and Windows's
/// `polo-preview://localhost/` → `http://polo-preview.localhost/` translation
/// is only proven to work with this exact authority.
const POLO_AUTHORITY: &str = "localhost";

/// Authority used for `WebView::load_url()` calls on Windows *after* the
/// initial build. wry's `polo-preview://localhost/` → `http://…` URI
/// workaround only runs at build time, not on `load_url()`, so reload
/// navigations must use the already-translated HTTP form directly —
/// otherwise the navigation silently fails.
#[cfg(target_os = "windows")]
const POLO_CONTENT_URL_RELOAD_AUTHORITY: &str = "http://polo-preview.localhost";

/// Query-string marker identifying a request as "serve the cached top-level
/// document" rather than a local asset. See [`build_document_url`].
const DOC_MARKER_QUERY: &str = "marco_doc=1";

/// Build the `polo-preview://` URL for the top-level preview document.
///
/// `base_uri`, when present, is the `file:///abs/dir/`-shaped string already
/// produced by `rendering::compute_base_uri` — its directory portion becomes
/// the URL path, percent-encoded segment by segment. Relative asset
/// references in the loaded HTML then resolve, via ordinary URL
/// relative-resolution (no `<base>` tag involved), to
/// `polo-preview://localhost/<same-dir>/<asset>` — served by
/// [`polo_protocol_handler`], which reads that path directly off disk.
/// `version` busts any cache the webview backend keeps for the scheme.
fn build_document_url(base_uri: Option<&str>, version: u64) -> String {
    let dir_path = base_uri
        .and_then(|b| b.strip_prefix("file://"))
        .unwrap_or("");
    let encoded_dir = percent_encode_path(dir_path);
    format!(
        "{}://{}{}?{}&v={}",
        POLO_SCHEME, POLO_AUTHORITY, encoded_dir, DOC_MARKER_QUERY, version
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
/// `rendering::compute_base_uri`) that must be stripped before the string is
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

/// `with_custom_protocol` handler shared by both platforms' build sites.
/// Bare `fn` rather than a closure — polo's single-viewer design needs no
/// per-instance capture, since the HTML store is one process-wide global
/// ([`polo_html`]), not per-instance.
fn polo_protocol_handler(
    _webview_id: wry::WebViewId,
    req: wry::http::Request<Vec<u8>>,
) -> wry::http::Response<std::borrow::Cow<'static, [u8]>> {
    let is_doc = req
        .uri()
        .query()
        .map(|q| q.split('&').any(|kv| kv == "marco_doc=1"))
        .unwrap_or(false);

    if is_doc {
        let body = match polo_html().lock() {
            Ok(guard) => guard.clone(),
            Err(_) => b"<html><body>Content unavailable</body></html>".to_vec(),
        };
        return wry::http::Response::builder()
            .header("Content-Type", "text/html; charset=utf-8")
            .header("Access-Control-Allow-Origin", "*")
            .body(std::borrow::Cow::Owned(body))
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
                "[polo] custom protocol: failed to read local asset '{}': {}",
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

/// Navigate an already-built webview to `doc_url`. On Windows this uses the
/// already-HTTP-translated authority (`load_url` doesn't apply wry's
/// build-time scheme translation); on Linux the `polo-preview://` URL is
/// used as-is.
fn navigate(view: &wry::WebView, doc_url: &str) {
    #[cfg(target_os = "windows")]
    let target = doc_url.replacen(
        &format!("{}://{}", POLO_SCHEME, POLO_AUTHORITY),
        POLO_CONTENT_URL_RELOAD_AUTHORITY,
        1,
    );
    #[cfg(target_os = "linux")]
    let target = doc_url.to_string();

    if let Err(e) = view.load_url(&target) {
        log::error!("[polo] Failed to load preview document: {}", e);
    }
}

/// In-memory HTML store for polo's single custom-protocol WebView.
/// Updated before every `load_url` call so the protocol handler always
/// returns the latest rendered HTML.
static POLO_HTML: std::sync::OnceLock<std::sync::Mutex<Vec<u8>>> = std::sync::OnceLock::new();

fn polo_html() -> &'static std::sync::Mutex<Vec<u8>> {
    POLO_HTML.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}

#[cfg(test)]
mod protocol_tests {
    use super::*;

    #[test]
    fn smoke_build_document_url_no_base() {
        let url = build_document_url(None, 3);
        assert_eq!(url, "polo-preview://localhost?marco_doc=1&v=3");
    }

    #[test]
    fn smoke_build_document_url_with_base() {
        let url = build_document_url(Some("file:///home/user/docs/"), 1);
        assert_eq!(
            url,
            "polo-preview://localhost/home/user/docs/?marco_doc=1&v=1"
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
    fn smoke_mime_type_for_path_known_and_unknown() {
        assert_eq!(mime_type_for_path("/x/img.PNG"), "image/png");
        assert_eq!(
            mime_type_for_path("/x/data.bin"),
            "application/octet-stream"
        );
    }
}

/// Open the WebView2 *system* print dialog.
///
/// Casts the underlying `ICoreWebView2` to `ICoreWebView2_16` and calls
/// `ShowPrintUI(COREWEBVIEW2_PRINT_DIALOG_KIND_SYSTEM)`.  Returns `Err` on
/// older WebView2 runtimes so the caller can fall back to the legacy path.
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

/// Decide whether a wry navigation should be allowed, and handle side effects.
///
/// - External links (http/https/www/mailto) → open in system browser, deny navigation.
/// - Local `.md` / `.markdown` file links → invoke the stored callback, deny navigation.
/// - Everything else (anchors, file resources, data URIs, about:blank) → allow.
///
/// Returns `true` to allow the navigation, `false` to deny it.
fn wry_navigation_handler(uri: &str, on_local_md: &NavigationHandlerCell) -> bool {
    let uri_lower = uri.to_lowercase();

    // Allow polo custom-protocol navigation (internal content reloads on
    // Windows). These are `http://polo-preview.localhost/?v=N` URLs produced
    // by `load_html_with_base` and must NOT be treated as external links.
    if uri_lower.starts_with("polo-preview:")
        || uri_lower.starts_with("http://polo-preview.")
        || uri_lower.starts_with("https://polo-preview.")
    {
        return true;
    }

    // External link → open in system browser, deny navigation.
    if uri_lower.starts_with("http:")
        || uri_lower.starts_with("https:")
        || uri_lower.starts_with("www.")
        || uri_lower.starts_with("mailto:")
    {
        let normalized = if uri_lower.starts_with("www.") {
            format!("https://{}", uri)
        } else {
            uri.to_string()
        };
        open_external_url(&normalized);
        return false;
    }

    // Local .md file link → invoke callback, deny navigation.
    if uri_lower.starts_with("file://") {
        let path_part = uri_lower.split('#').next().unwrap_or("");
        if path_part.ends_with(".md") || path_part.ends_with(".markdown") {
            let without_scheme = uri.trim_start_matches("file://");
            let (raw_path, fragment) = match without_scheme.split_once('#') {
                Some((p, f)) => (
                    p,
                    if f.is_empty() {
                        None
                    } else {
                        Some(f.to_string())
                    },
                ),
                None => (without_scheme, None),
            };
            let path = raw_path.replace("%20", " ");
            if let Some(handler) = on_local_md.borrow().as_ref() {
                handler(path, fragment);
            }
            return false;
        }
    }

    // Allow all other navigation (in-page anchors, resources, etc.).
    true
}

/// Open a URL in the default system browser.
#[cfg(target_os = "linux")]
fn open_external_url(url: &str) {
    if let Err(e) = gio::AppInfo::launch_default_for_uri(url, None::<&gio::AppLaunchContext>) {
        log::warn!("[polo] Failed to open external link '{}': {}", url, e);
    }
}

/// Open a URL in the default system browser on Windows.
#[cfg(target_os = "windows")]
fn open_external_url(url: &str) {
    // Use `rundll32 url.dll,FileProtocolHandler <url>` instead of
    // `cmd /c start` to avoid shell-metacharacter injection.  rundll32
    // passes the URL as a discrete argument without involving cmd.exe,
    // so characters like `&`, `|`, or `>` in a crafted URL cannot be
    // interpreted as shell commands.
    if let Err(e) = std::process::Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", url])
        .spawn()
    {
        log::warn!("[polo] Failed to open external link '{}': {}", url, e);
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
