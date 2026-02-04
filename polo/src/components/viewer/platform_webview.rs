//! Platform-specific WebView wrapper for Polo.
//!
//! Linux uses `webkit6` directly (GTK widget).
//! Windows uses `wry` (WebView2) embedded as a child window inside the GTK
//! `ApplicationWindow` using the Win32 HWND retrieved from the GDK surface.

use gtk4::prelude::*;

#[cfg(target_os = "linux")]
use gio;

#[cfg(target_os = "linux")]
use webkit6::prelude::WebViewExt;

/// Unified WebView wrapper exposed to the rest of the codebase.
///
/// - On Linux this wraps `webkit6::WebView` and keeps identical behavior to the
///   previous implementation.
/// - On Windows this embeds a `wry::WebView` as a child window that follows the
///   GTK allocation of a placeholder widget.
#[derive(Clone)]
pub struct PlatformWebView {
    #[cfg(target_os = "linux")]
    inner: webkit6::WebView,

    #[cfg(target_os = "windows")]
    inner: std::rc::Rc<std::cell::RefCell<Option<wry::WebView>>>,
    #[cfg(target_os = "windows")]
    container: gtk4::Box,
    #[cfg(target_os = "windows")]
    parent_handle: std::rc::Rc<ParentWindowHandle>,
    #[cfg(target_os = "windows")]
    bg_color: std::rc::Rc<std::cell::Cell<(u8, u8, u8, u8)>>,
    #[cfg(target_os = "windows")]
    gtk_window: gtk4::ApplicationWindow,
}

#[cfg(target_os = "linux")]
impl PlatformWebView {
    pub fn new(_window: &gtk4::ApplicationWindow) -> Result<Self, String> {
        let webview = webkit6::WebView::new();
        webview.set_vexpand(true);
        webview.set_hexpand(true);

        if let Some(settings) = webkit6::prelude::WebViewExt::settings(&webview) {
            settings.set_allow_file_access_from_file_urls(true);
            settings.set_allow_universal_access_from_file_urls(true);
            settings.set_auto_load_images(true);
            settings.set_enable_developer_extras(false);
            settings.set_javascript_can_access_clipboard(false);
            settings.set_enable_write_console_messages_to_stdout(false);
        }

        Ok(Self { inner: webview })
    }

    pub fn widget(&self) -> gtk4::Widget {
        self.inner.clone().upcast()
    }

    pub fn set_background_color_rgba(&self, color: &gtk4::gdk::RGBA) {
        self.inner.set_background_color(color);
    }

    pub fn load_html_with_base(&self, html: &str, base_uri: Option<&str>) {
        let webview_clone = self.inner.clone();
        let html_string = html.to_string();
        let base = base_uri.map(|b| b.to_string());
        gtk4::glib::idle_add_local_once(move || {
            webview_clone.load_html(&html_string, base.as_deref());
        });
    }

    /// Kept for API consistency with Windows implementation, not currently used on Linux
    #[allow(dead_code)]
    pub fn evaluate_script(&self, script: &str) {
        self.inner
            .evaluate_javascript(script, None, None, None::<&gio::Cancellable>, |result| {
                if let Err(e) = result {
                    log::error!("JavaScript evaluation failed: {}", e);
                }
            });
    }
}

#[cfg(target_os = "windows")]
impl PlatformWebView {
    pub fn new(window: &gtk4::ApplicationWindow) -> Result<Self, String> {
        use gtk4::prelude::WidgetExt;
        use raw_window_handle::{
            RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
        };
        use std::cell::RefCell;
        use std::num::NonZeroIsize;
        use std::rc::Rc;

        // Ensure the GTK window is realized so a surface/handle exists
        WidgetExt::realize(window);

        // Get the GDK surface from the GTK window
        let surface = window
            .surface()
            .ok_or_else(|| "Failed to get GDK surface".to_string())?;

        // Use gdk4-win32 to get the native Win32 HWND
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

        let parent_handle = ParentWindowHandle {
            window: unsafe { raw_window_handle::WindowHandle::borrow_raw(raw_window) },
            display: unsafe { raw_window_handle::DisplayHandle::borrow_raw(raw_display) },
        };
        let parent_handle = std::rc::Rc::new(parent_handle);

        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        container.set_vexpand(true);
        container.set_hexpand(true);

        let webview: Rc<RefCell<Option<wry::WebView>>> = Rc::new(RefCell::new(None));
        let bg_color = std::rc::Rc::new(std::cell::Cell::new((30u8, 30u8, 30u8, 255u8)));

        // Keep WebView bounds in sync with GTK container on every frame
        // In windowed mode, offset by titlebar/border size; maximized mode needs no offset
        let webview_for_tick = webview.clone();
        let container_weak = container.downgrade();
        let window_weak = window.downgrade();
        container.add_tick_callback(move |_, _| {
            if let (Some(container), Some(win), Some(view)) = (
                container_weak.upgrade(),
                window_weak.upgrade(),
                webview_for_tick.borrow().as_ref(),
            ) {
                let alloc = container.allocation();
                let (offset_x, offset_y) = if win.is_maximized() {
                    (0.0, 0.0)
                } else {
                    (14.0, 12.0)
                };
                let rect = wry::Rect {
                    position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(
                        alloc.x() as f64 + offset_x,
                        alloc.y() as f64 + offset_y,
                    )),
                    size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(
                        alloc.width().max(1) as f64,
                        alloc.height().max(1) as f64,
                    )),
                };
                if let Err(e) = view.set_bounds(rect) {
                    log::debug!("wry set_bounds failed: {}", e);
                }
            }
            gtk4::glib::ControlFlow::Continue
        });

        Ok(Self {
            inner: webview,
            container,
            parent_handle,
            bg_color,
            gtk_window: window.clone(),
        })
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
        let (offset_x, offset_y) = if self.gtk_window.is_maximized() {
            (0.0, 0.0)
        } else {
            (16.0, 14.0)
        };
        let rect = wry::Rect {
            position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(
                alloc.x() as f64 + offset_x,
                alloc.y() as f64 + offset_y,
            )),
            size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(
                alloc.width().max(100) as f64,
                alloc.height().max(100) as f64,
            )),
        };

        // Configure WebView2 to use data directory (portable mode friendly)
        // WebView2 respects WEBVIEW2_USER_DATA_FOLDER environment variable
        let data_dir = core::paths::user_data_dir().join("webview");
        if let Err(e) = std::fs::create_dir_all(&data_dir) {
            log::warn!("Failed to create WebView2 data directory: {}", e);
        }
        std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", data_dir);

        match wry::WebViewBuilder::new()
            .with_background_color(self.bg_color.get())
            .with_bounds(rect)
            .with_html(&final_html)
            .build_as_child(&*self.parent_handle)
        {
            Ok(view) => {
                *self.inner.borrow_mut() = Some(view);
            }
            Err(e) => log::error!("Failed to build wry WebView for initial load: {}", e),
        }
    }

    /// Kept for API consistency with Linux implementation, not currently used on Windows
    #[allow(dead_code)]
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
