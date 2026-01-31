//! Detached preview window implementation that uses `wry` on Windows.
#![cfg(target_os = "windows")]

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, ScrolledWindow, Box as GtkBox, Orientation, Label};

use crate::components::viewer::wry;
use crate::components::viewer::wry_platform_webview::PlatformWebView;

/// Type alias for a shared, mutable callback function
type CloseCallback = Rc<RefCell<Option<Box<dyn Fn()>>>>;

/// PreviewWindow implementation for Windows that uses an in-app GTK window
/// with an embedded `PlatformWebView`. This provides a rendered preview that
/// behaves similarly to the Linux detached preview but keeps all windows
/// within the GTK application lifecycle (avoids tao event-loop threading issues).
pub struct PreviewWindow {
    window: ApplicationWindow,
    container: ScrolledWindow,
    platform_webview: Rc<RefCell<Option<PlatformWebView>>>,
    is_visible: Rc<RefCell<bool>>,
    on_close_callback: CloseCallback,
    callback_invoked: Rc<Cell<bool>>,
}

impl PreviewWindow {
    pub fn new(parent_window: &ApplicationWindow) -> Self {
        // Build a transient GTK ApplicationWindow
        let window = ApplicationWindow::builder()
            .default_width(900)
            .default_height(700)
            .title("Marco Preview")
            .build();
        log::info!("Created Windows PreviewWindow (GTK in-app)");

        // Match Linux preview behavior: hide on close and destroy with parent
        // so the window can be reused and cleaned up with the main window.
        window.set_destroy_with_parent(true);
        window.set_hide_on_close(true);

        // Apply theme class for consistent styling
        if parent_window.style_context().has_class("marco-theme-dark") {
            window.add_css_class("marco-theme-dark");
        } else {
            window.add_css_class("marco-theme-light");
        }

        // Setup a minimal titlebar / window controls for parity
        Self::setup_custom_titlebar(&window);

        // Create scrolled container for the platform webview widget
        let container = ScrolledWindow::new();
        container.set_hexpand(true);
        container.set_vexpand(true);
        container.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
        window.set_child(Some(&container));

        let platform_webview: Rc<RefCell<Option<PlatformWebView>>> = Rc::new(RefCell::new(None));
        let is_visible = Rc::new(RefCell::new(false));
        let on_close_callback: CloseCallback = Rc::new(RefCell::new(None));
        let callback_invoked = Rc::new(Cell::new(false));

        // Close-request handling: ensure the embedded webview is dropped before
        // the window is destroyed so any underlying Win32/WebView resources
        // are cleaned up in a predictable order.
        let is_visible_clone = Rc::clone(&is_visible);
        let pw_clone = Rc::clone(&platform_webview);
        let callback_clone = Rc::clone(&on_close_callback);
        let callback_invoked_clone = Rc::clone(&callback_invoked);
        window.connect_close_request(move |_| {
            *is_visible_clone.borrow_mut() = false;
            log::info!("Preview window closed by user");

            // Explicitly drop the wry WebView child if present
            if let Some(pv) = pw_clone.borrow_mut().take() {
                log::info!("Dropping embedded PlatformWebView during preview window close");
                // Drop its inner webview (if any) to ensure WebView2 child windows are destroyed
                pv.inner.borrow_mut().take();
            }

            // Call the on_close callback if not yet invoked
            if !callback_invoked_clone.get() {
                callback_invoked_clone.set(true);
                if let Some(ref cb) = *callback_clone.borrow() {
                    log::info!("Calling on_close callback from close_request");
                    cb();
                }
            }

            gtk4::glib::Propagation::Proceed
        });

        Self {
            window,
            container,
            platform_webview,
            is_visible,
            on_close_callback,
            callback_invoked,
        }
    }

    /// Setup a simple custom titlebar similar to Linux preview window
    fn setup_custom_titlebar(window: &ApplicationWindow) {
        use gtk4::prelude::*;
        use gtk4::{Align, Button, HeaderBar, Label, WindowHandle};

        // Create WindowHandle wrapper for proper window dragging
        let handle = WindowHandle::new();

        // Use GTK4 HeaderBar for proper title centering
        let headerbar = HeaderBar::new();
        headerbar.add_css_class("titlebar");
        headerbar.set_show_title_buttons(false); // We'll add custom window controls

        // Centered title label
        let title_label = Label::new(Some("Marco Preview"));
        title_label.set_valign(Align::Center);
        title_label.add_css_class("title-label");
        headerbar.set_title_widget(Some(&title_label));

        // Helper: render a window control SVG into a GDK texture
        use crate::ui::css::constants::{DARK_PALETTE, LIGHT_PALETTE};
        use core::logic::loaders::icon_loader::{window_icon_svg, WindowIcon};
        use rsvg::{Loader, CairoRenderer};
        use gtk4::gdk;
        use gio;

        fn render_window_svg(icon: WindowIcon, color: &str, icon_size: f64) -> gdk::MemoryTexture {
            let svg = window_icon_svg(icon).replace("currentColor", color);
            let bytes = glib::Bytes::from_owned(svg.into_bytes());
            let stream = gio::MemoryInputStream::from_bytes(&bytes);
            let handle = Loader::new()
                .read_stream(&stream, None::<&gio::File>, gio::Cancellable::NONE)
                .expect("load SVG handle");

            let display_scale = gdk::Display::default()
                .and_then(|d| d.monitors().item(0))
                .and_then(|m| m.downcast::<gdk::Monitor>().ok())
                .map(|m| m.scale_factor() as f64)
                .unwrap_or(1.0);
            let render_scale = display_scale * 2.0;
            let render_size = (icon_size * render_scale) as i32;

            let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, render_size, render_size)
                .expect("create surface");
            {
                let cr = cairo::Context::new(&surface).expect("create context");
                cr.scale(render_scale, render_scale);
                let renderer = CairoRenderer::new(&handle);
                let viewport = cairo::Rectangle::new(0.0, 0.0, icon_size, icon_size);
                renderer.render_document(&cr, &viewport).expect("render SVG");
            }

            let data = surface.data().expect("get surface data").to_vec();
            let bytes = glib::Bytes::from_owned(data);
            gdk::MemoryTexture::new(
                render_size,
                render_size,
                gdk::MemoryFormat::B8g8r8a8Premultiplied,
                &bytes,
                (render_size * 4) as usize,
            )
        }

        // Helper to create a SVG-backed icon button with hover/press interactions
        let svg_icon_button = |icon: WindowIcon, tooltip: &str| {
            use gtk4::Picture;
            let pic = Picture::new();
            let is_dark = window.has_css_class("marco-theme-dark");
            let color = if is_dark { DARK_PALETTE.control_icon } else { LIGHT_PALETTE.control_icon };
            let texture = render_window_svg(icon, color, 8.0);
            pic.set_paintable(Some(&texture));
            pic.set_size_request(8, 8);
            pic.set_valign(Align::Center);
            pic.set_halign(Align::Center);

            let btn = Button::new();
            btn.set_child(Some(&pic));
            btn.set_tooltip_text(Some(tooltip));
            btn.set_valign(Align::Center);
            btn.set_margin_start(1);
            btn.set_margin_end(1);
            btn.set_focusable(true);
            btn.set_can_focus(true);
            btn.set_has_frame(false);
            btn.add_css_class("topright-btn");
            btn.add_css_class("window-control-btn");

            // Hover and press state handling
            {
                let pic_hover = pic.clone();
                let normal_color = color.to_string();
                let hover_color = if is_dark {
                    DARK_PALETTE.control_icon_hover.to_string()
                } else {
                    LIGHT_PALETTE.control_icon_hover.to_string()
                };
                let active_color = if is_dark {
                    DARK_PALETTE.control_icon_active.to_string()
                } else {
                    LIGHT_PALETTE.control_icon_active.to_string()
                };

                let motion = gtk4::EventControllerMotion::new();
                let icon_for_enter = icon;
                let hover_for_enter = hover_color.clone();
                motion.connect_enter(move |_ctrl, _x, _y| {
                    let tex = render_window_svg(icon_for_enter, &hover_for_enter, 8.0);
                    pic_hover.set_paintable(Some(&tex));
                });

                let pic_leave = pic.clone();
                let icon_for_leave = icon;
                let normal_for_leave = normal_color.clone();
                motion.connect_leave(move |_ctrl| {
                    let tex = render_window_svg(icon_for_leave, &normal_for_leave, 8.0);
                    pic_leave.set_paintable(Some(&tex));
                });
                btn.add_controller(motion);

                let gesture = gtk4::GestureClick::new();
                let pic_pressed = pic.clone();
                let icon_for_pressed = icon;
                let active_color_pressed = active_color.clone();
                gesture.connect_pressed(move |_g, _n, _x, _y| {
                    let tex = render_window_svg(icon_for_pressed, &active_color_pressed, 8.0);
                    pic_pressed.set_paintable(Some(&tex));
                });

                let pic_released = pic.clone();
                let hover_for_release = hover_color.clone();
                let icon_for_released = icon;
                gesture.connect_released(move |_g, _n, _x, _y| {
                    let tex = render_window_svg(icon_for_released, &hover_for_release, 8.0);
                    pic_released.set_paintable(Some(&tex));
                });
                btn.add_controller(gesture);
            }

            btn
        };

        // Window control icons (SVG)
        let btn_min = svg_icon_button(WindowIcon::Minimize, "Minimize");
        let btn_close = svg_icon_button(WindowIcon::Close, "Close");

        // Create a single toggle button for maximize/restore using SVG
        let max_pic = gtk4::Picture::new();
        max_pic.set_size_request(8, 8);
        max_pic.set_valign(Align::Center);
        max_pic.set_halign(Align::Center);

        let update_max_icon = {
            let is_dark = window.has_css_class("marco-theme-dark");
            let color = if is_dark { DARK_PALETTE.control_icon } else { LIGHT_PALETTE.control_icon };
            move |is_maximized: bool, pic: &gtk4::Picture| {
                let icon = if is_maximized { WindowIcon::Restore } else { WindowIcon::Maximize };
                let tex = render_window_svg(icon, color, 8.0);
                pic.set_paintable(Some(&tex));
            }
        };

        update_max_icon(window.is_maximized(), &max_pic);

        let btn_max_toggle = Button::new();
        btn_max_toggle.set_child(Some(&max_pic));
        btn_max_toggle.set_tooltip_text(Some("Maximize / Restore"));
        btn_max_toggle.set_valign(Align::Center);
        btn_max_toggle.set_margin_start(1);
        btn_max_toggle.set_margin_end(1);
        btn_max_toggle.set_focusable(false);
        btn_max_toggle.set_width_request(14);
        btn_max_toggle.set_height_request(14);
        btn_max_toggle.set_can_focus(false);
        btn_max_toggle.set_has_frame(false);
        btn_max_toggle.add_css_class("topright-btn");
        btn_max_toggle.add_css_class("window-control-btn");

        // Add controls to headerbar from right to left (pack_end order)
        headerbar.pack_end(&btn_close); // Rightmost
        headerbar.pack_end(&btn_max_toggle); // Middle
        headerbar.pack_end(&btn_min); // Left of window controls

        // Minimize action
        let win_clone = window.clone();
        btn_min.connect_clicked(move |_| {
            log::info!("Preview window minimize button clicked - handler called");
            win_clone.minimize();
            log::info!("Preview window minimize() called");
        });

        // Close action - just close the window (callback will be triggered via close_request)
        let win_for_close = window.clone();
        btn_close.connect_clicked(move |_| {
            win_for_close.close();
            log::debug!("Preview window close clicked");
        });

        // Maximize/restore toggle (update SVG picture)
        let pic_for_toggle = max_pic.clone();
        let update_for_toggle = update_max_icon.clone();
        let window_for_toggle = window.clone();
        btn_max_toggle.connect_clicked(move |_| {
            log::info!("Preview window maximize/restore button clicked - handler called");
            if window_for_toggle.is_maximized() {
                window_for_toggle.unmaximize();
                update_for_toggle(false, &pic_for_toggle);
                log::info!("Preview window unmaximized");
            } else {
                window_for_toggle.maximize();
                update_for_toggle(true, &pic_for_toggle);
                log::info!("Preview window maximized");
            }
        });

        // Keep maximize icon in sync if window is maximized/unmaximized externally
        let pic_for_notify = max_pic.clone();
        let update_for_notify = update_max_icon.clone();
        window.connect_notify_local(Some("is-maximized"), move |w, _| {
            update_for_notify(w.is_maximized(), &pic_for_notify);
        });

        // Set the headerbar in the WindowHandle for dragging
        handle.set_child(Some(&headerbar));

        // Set the WindowHandle as the titlebar
        window.set_titlebar(Some(&handle));
    }

    /// Attach a preview - creates (or re-uses) an embedded `PlatformWebView` and
    /// loads the current saved HTML content. This keeps the editor preview and
    /// detached preview independent so both can render simultaneously.
    pub fn attach_webview(&self, _webview: Option<&gtk4::Widget>) {
        // If we already have a platform webview, refresh content
        if let Some(ref pv) = *self.platform_webview.borrow() {
            if let Ok(guard) = wry::LATEST_PREVIEW_HTML.get_or_init(|| std::sync::Mutex::new(String::new())).lock() {
                pv.load_html_with_base(&guard.clone(), None);
            }
            return;
        }

        // Create new embedded PlatformWebView inside this preview window
        let pv = PlatformWebView::new(&self.window);
        self.container.set_child(Some(&pv.widget()));

        if let Ok(guard) = wry::LATEST_PREVIEW_HTML.get_or_init(|| std::sync::Mutex::new(String::new())).lock() {
            pv.load_html_with_base(&guard.clone(), None);
        }

        *self.platform_webview.borrow_mut() = Some(pv);
        log::info!("Created embedded PlatformWebView in preview window (attempted load)");

        // If the embedded WebView failed to create (no WebView2 runtime), fall back
        // to opening the persisted HTML in the system browser and notify the user.
        if let Some(ref pv_ref) = *self.platform_webview.borrow() {
            if pv_ref.inner.borrow().is_none() {
                log::warn!("Embedded wry WebView not available; falling back to system browser");
                // Persist the HTML to a temp file and open it
                if let Ok(guard) = wry::LATEST_PREVIEW_HTML.get_or_init(|| std::sync::Mutex::new(String::new())).lock() {
                    // Show the HTML source inside the preview window as a fallback, plus a button
                    // to open it in the system browser. This keeps the user inside the app
                    // while still providing a usable rendered experience via the browser.
                    let vbox = gtk4::Box::new(Orientation::Vertical, 8);
                    vbox.set_margin_top(8);
                    vbox.set_margin_bottom(8);
                    vbox.set_margin_start(8);
                    vbox.set_margin_end(8);

                    let label = Label::new(Some("Embedded preview is not available (missing WebView2 runtime)."));
                    label.set_wrap(true);
                    vbox.append(&label);

                    // Text view with the generated HTML source for inspection
                    let sw = ScrolledWindow::new();
                    sw.set_hexpand(true);
                    sw.set_vexpand(true);
                    let tv = gtk4::TextView::new();
                    tv.set_editable(false);
                    tv.set_monospace(true);
                    tv.buffer().set_text(&*guard);
                    sw.set_child(Some(&tv));
                    vbox.append(&sw);

                    // Add a button to open in system browser
                    let ts = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_millis();
                    let file_name = format!("marco_preview_{}.html", ts);
                    let mut file_path = std::env::temp_dir();
                    file_path.push(&file_name);
                    let file_url = if std::fs::write(&file_path, &*guard).is_ok() {
                        let s = format!("file://{}", file_path.to_string_lossy());
                        log::info!("Persisted fallback preview HTML to: {}", file_path.display());
                        s
                    } else {
                        String::new()
                    };

                    let open_btn = gtk4::Button::with_label("Open in system browser");
                    if !file_url.is_empty() {
                        let file_url_clone = file_url.clone();
                        open_btn.connect_clicked(move |_| {
                            if let Err(e) = gio::AppInfo::launch_default_for_uri(&file_url_clone, None::<&gio::AppLaunchContext>) {
                                log::error!("Failed to open fallback preview in system browser: {}", e);
                            }
                        });
                    } else {
                        open_btn.set_sensitive(false);
                    }
                    vbox.append(&open_btn);

                    self.container.set_child(Some(&vbox));
                }
            } else {
                log::info!("Embedded wry WebView available in preview window (rendering inline)");
            }
        }
    }

    /// Detach returns None on Windows - we duplicate the preview instead of moving it
    pub fn detach_webview(&self) -> Option<gtk4::Widget> {
        None
    }

    /// Show the preview window. If no embedded webview exists, create it and
    /// load the latest HTML content.
    pub fn show(&self) {
        if self.platform_webview.borrow().is_none() {
            // Create and load content
            self.attach_webview(None);
        }
        log::info!("Showing PreviewWindow (present)");
        // Reset the callback guard so the callback will fire for each open/close cycle
        self.callback_invoked.set(false);
        self.window.present();
        *self.is_visible.borrow_mut() = true;
    }

    pub fn hide(&self) {
        self.window.hide();
        *self.is_visible.borrow_mut() = false;
        log::info!("Preview window hidden via hide() method");

        // Manually trigger the on_close callback (since hide() doesn't fire close_request)
        // But only if it hasn't been called yet (prevents double-call)
        if !self.callback_invoked.get() {
            self.callback_invoked.set(true);
            if let Some(ref cb) = *self.on_close_callback.borrow() {
                log::info!("Manually calling on_close callback from hide()");
                cb();
            }
        } else {
            log::debug!("Callback already invoked, skipping");
        }
    }

    pub fn set_on_close_callback<F: Fn() + 'static>(&self, callback: F) {
        log::info!("Preview window: on_close callback registered");
        *self.on_close_callback.borrow_mut() = Some(Box::new(callback));
    }

    pub fn is_visible(&self) -> bool {
        *self.is_visible.borrow()
    }
}
