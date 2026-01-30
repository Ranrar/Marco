//! Windows-specific PlatformWebView using `wry` (WebView2) embedded as a child
//! window inside the GTK `ApplicationWindow`.
//!
//! This mirrors the approach used in `polo` so Marco's preview can embed a
//! `wry::WebView` on Windows (using Win32 HWND obtained from GDK surface) and
//! avoid spawning a separate tao EventLoop thread.

#![cfg(windows)]

use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(windows)]
use raw_window_handle::{RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle};

#[cfg(windows)]
use std::num::NonZeroIsize;

/// Windows PlatformWebView wrapper
#[derive(Clone)]
pub struct PlatformWebView {
    pub inner: std::rc::Rc<std::cell::RefCell<Option<wry::WebView>>>,
    pub container: gtk4::Box,
    pub parent_handle: std::rc::Rc<ParentWindowHandle>,
    pub bg_color: std::rc::Rc<std::cell::Cell<(u8, u8, u8, u8)>>,
    pub gtk_window: gtk4::ApplicationWindow,
}

impl PlatformWebView {
    pub fn new(window: &gtk4::ApplicationWindow) -> Self {
        use gtk4::prelude::WidgetExt;

        // Ensure the GTK window is realized so a surface/handle exists
        WidgetExt::realize(window);

        // Default fallback container & state
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        container.set_vexpand(true);
        container.set_hexpand(true);
        let webview: Rc<RefCell<Option<wry::WebView>>> = Rc::new(RefCell::new(None));
        let bg_color = std::rc::Rc::new(std::cell::Cell::new((30u8, 30u8, 30u8, 255u8)));

        // Attempt to obtain parent HWND and parent_handle; on failure, keep inner None
        let parent_handle_rc = match (|| {
            // Get the GDK surface from the GTK window
            let surface = window.surface()?;

            // Use gdk4-win32 to get the native Win32 HWND
            use gdk4_win32::Win32Surface;
            let win32_surface: &Win32Surface = surface.downcast_ref()?;

            let hwnd_ptr = unsafe { gdk4_win32::ffi::gdk_win32_surface_get_handle(win32_surface.as_ptr() as *mut _) };
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
            },
            None => {
                log::warn!("wry PlatformWebView: failed to get Win32 parent handle - falling back to placeholder container");
                // Add a placeholder label into the container so UI is usable
                let label = gtk4::Label::new(Some("Preview not available inline on Windows (missing Win32 handle)"));
                label.set_wrap(true);
                container.append(&label);
                // Provide a dummy ParentWindowHandle so types work later if needed
                let parent_handle = ParentWindowHandle {
                    window: unsafe { raw_window_handle::WindowHandle::borrow_raw(RawWindowHandle::Win32(Win32WindowHandle::new(NonZeroIsize::new(1).unwrap()))) },
                    display: unsafe { raw_window_handle::DisplayHandle::borrow_raw(RawDisplayHandle::Windows(WindowsDisplayHandle::new())) },
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

        Self { inner: webview, container, parent_handle: parent_handle_rc, bg_color, gtk_window: window.clone() }
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
        let final_html = if let Some(base) = base_uri { inject_base_href(html, base) } else { html.to_string() };

        if let Some(view) = self.inner.borrow().as_ref() {
            if let Err(e) = view.load_html(&final_html) {
                log::error!("Failed to load HTML into wry WebView: {}", e);
            }
            return;
        }

        // If the WebView is not ready yet (early call before the first allocation),
        // store the HTML and load it after the widget is realized by forcing an
        // initial creation now with a minimal rect.
        let alloc = self.container.allocation();
        let (offset_x, offset_y) = if self.gtk_window.is_maximized() { (0.0, 0.0) } else { (16.0, 14.0) };

        // Translate container origin into window coordinate space so the initial
        // creation uses correct coordinates on Windows.
        let origin_in_window = match self.container.translate_coordinates(&self.gtk_window, 0.0, 0.0) {
            Some((x, y)) => (x, y),
            None => (alloc.x() as f64, alloc.y() as f64),
        };

        let rect = wry::Rect {
            position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(
                origin_in_window.0 + offset_x - 1.0,
                origin_in_window.1 + offset_y
            )),
            size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(alloc.width().max(100) as f64 + 1.0, alloc.height().max(100) as f64)),
        };

        log::debug!("[wry] initial_create origin_in_window=({}, {}), alloc=({}, {}), rect_pos=({}, {}), rect_size=({}, {})",
            origin_in_window.0, origin_in_window.1, alloc.x(), alloc.y(),
            origin_in_window.0 + offset_x - 1.0, origin_in_window.1 + offset_y,
            alloc.width().max(100) + 1, alloc.height().max(100)
        );
        match wry::WebViewBuilder::new()
            .with_background_color(self.bg_color.get())
            .with_bounds(rect)
            .with_html(&final_html)
            .build_as_child(&*self.parent_handle)
        {
            Ok(view) => {
                *self.inner.borrow_mut() = Some(view);
                log::info!("wry WebView successfully created as child for initial load");
            }
            Err(e) => log::error!("Failed to build wry WebView for initial load: {}", e),
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
}

#[cfg(windows)]
#[derive(Clone)]
struct ParentWindowHandle {
    window: raw_window_handle::WindowHandle<'static>,
    display: raw_window_handle::DisplayHandle<'static>,
}

#[cfg(windows)]
impl raw_window_handle::HasWindowHandle for ParentWindowHandle {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        Ok(self.window)
    }
}

#[cfg(windows)]
impl raw_window_handle::HasDisplayHandle for ParentWindowHandle {
    fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        Ok(self.display)
    }
}

#[cfg(windows)]
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
