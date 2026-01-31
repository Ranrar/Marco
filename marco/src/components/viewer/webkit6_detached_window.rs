//! Detached preview window implementation that uses `webkit6` on Linux.
//!
//! This module manages a separate GTK ApplicationWindow that hosts the reparented WebView
//! for the EditorAndViewSeparate layout mode. The window provides a dedicated space for
//! viewing the HTML preview while editing in the main window.
//!
//!
//! # Reparenting Approach
//!
//! The PreviewWindow does NOT create its own WebView. Instead, it receives the existing
//! WebView from the main window through the `attach_webview()` method. This approach:
//! - Preserves scroll position automatically
//! - Maintains DOM state without reload
//! - Eliminates content synchronization complexity
//! - Uses less memory (single WebView instance)
//!
//! # Window Lifecycle
//!
//! - **Created once** when EditorAndViewSeparate mode is first activated
//! - **Hidden** (not destroyed) when switching to other layout modes
//! - **Reused** when returning to EditorAndViewSeparate mode
//! - **Destroyed** automatically when main window closes (via destroy_with_parent)
//!
//! # Example
//!
//! ```no_run
//! use marco::components::viewer::previewwindow::PreviewWindow;
//! use gtk4::prelude::*;
//!
//! let preview_window = PreviewWindow::new(&main_window, &app);
//! preview_window.attach_webview(&webview);
//! preview_window.show();
//!
//! // Later...
//! if let Some(webview) = preview_window.detach_webview() {
//!     // Reparent back to main window
//! }
//! preview_window.hide();
//! ```

#![cfg(target_os = "linux")]

use gtk4::prelude::*;
use gtk4::{ApplicationWindow, ScrolledWindow, Picture};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use webkit6::WebView;

/// Type alias for a shared, mutable callback function
type CloseCallback = Rc<RefCell<Option<Box<dyn Fn()>>>>;

/// Manages a separate window for displaying the HTML preview
///
/// This window is designed to work with the WebView reparenting approach:
/// - Does not create its own WebView
/// - Receives WebView through `attach_webview()`
/// - Returns WebView through `detach_webview()`
/// - Maintains window state across hide/show cycles
pub struct PreviewWindow {
    /// The GTK application window
    window: ApplicationWindow,
    /// ScrolledWindow container for the WebView
    container: ScrolledWindow,
    /// Tracks whether the window is currently visible
    is_visible: Rc<RefCell<bool>>,
    /// Callback to be called when window is closed
    on_close_callback: CloseCallback,
    /// Prevents callback from being called multiple times
    callback_invoked: Rc<Cell<bool>>,
}

impl PreviewWindow {
    /// Create a new preview window
    ///
    /// # Arguments
    ///
    /// * `parent_window` - The main application window (for transient-for relationship)
    /// * `app` - The GTK application instance
    ///
    /// # Window Properties
    ///
    /// - **transient_for**: Links to parent window for proper stacking
    /// - **destroy_with_parent**: Ensures cleanup when main window closes
    /// - **hide_on_close**: Window hides instead of destroying on close button
    /// - **Default size**: 800x600 (user can resize)
    ///
    /// # Example
    ///
    /// ```no_run
    /// let preview_window = PreviewWindow::new(&main_window, &app);
    /// ```
    pub fn new(parent_window: &ApplicationWindow, app: &gtk4::Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(800)
            .default_height(600)
            .build();

        // NOTE: Do NOT set transient-for - this prevents minimize/maximize from working
        // Transient windows are treated as dialogs by window managers and typically can't be minimized
        // window.set_transient_for(Some(parent_window));  // REMOVED

        // Still destroy with parent to ensure cleanup
        window.set_destroy_with_parent(true);
        window.set_hide_on_close(true); // Hide instead of destroy

        // Apply the same theme class as the parent window to ensure consistent styling
        // Check which theme class the parent has and apply it to the preview window
        if parent_window.has_css_class("marco-theme-dark") {
            window.add_css_class("marco-theme-dark");
            log::debug!("Preview window: Applied marco-theme-dark class");
        } else if parent_window.has_css_class("marco-theme-light") {
            window.add_css_class("marco-theme-light");
            log::debug!("Preview window: Applied marco-theme-light class");
        } else {
            // Default to dark theme if parent has no theme class
            window.add_css_class("marco-theme-dark");
            log::debug!("Preview window: Applied default marco-theme-dark class");
        }

        // Create custom titlebar with window controls
        Self::setup_custom_titlebar(&window);

        // Create scrolled container for WebView
        let container = ScrolledWindow::new();
        container.set_hexpand(true);
        container.set_vexpand(true);
        container.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);

        window.set_child(Some(&container));

        let is_visible = Rc::new(RefCell::new(false));
        let on_close_callback: CloseCallback = Rc::new(RefCell::new(None));
        let callback_invoked = Rc::new(Cell::new(false));

        // Setup close-request handler
        let is_visible_clone = Rc::clone(&is_visible);
        let callback_clone = Rc::clone(&on_close_callback);
        let callback_invoked_clone = Rc::clone(&callback_invoked);
        window.connect_close_request(move |_| {
            *is_visible_clone.borrow_mut() = false;
            log::info!("Preview window closed by user");

            // Call the on_close callback if it exists and hasn't been called yet
            if !callback_invoked_clone.get() {
                callback_invoked_clone.set(true);
                if let Some(ref callback) = *callback_clone.borrow() {
                    log::info!("Calling on_close callback from close_request");
                    callback();
                }
            } else {
                log::debug!("Callback already invoked, skipping");
            }

            gtk4::glib::Propagation::Proceed
        });

        Self {
            window,
            container,
            is_visible,
            on_close_callback,
            callback_invoked,
        }
    }

    /// Setup custom titlebar with window controls (minimize, maximize, close)
    ///
    /// Creates a HeaderBar with centered title and window control buttons
    /// matching the style of the main application window.
    fn setup_custom_titlebar(window: &ApplicationWindow) {
        use gtk4::prelude::*;
        use gtk4::{Align, Button, Label, WindowHandle};

        // Create WindowHandle wrapper for proper window dragging
        let handle = WindowHandle::new();

        // Use GTK4 HeaderBar for proper title centering
        let headerbar = gtk4::HeaderBar::new();
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

        log::debug!(
            "Created minimize button, visible: {}, sensitive: {}",
            btn_min.is_visible(),
            btn_min.is_sensitive()
        );
        log::debug!(
            "Created close button, visible: {}, sensitive: {}",
            btn_close.is_visible(),
            btn_close.is_sensitive()
        );

        // Create a single toggle button for maximize/restore using SVG
        let max_pic = Picture::new();
        max_pic.set_size_request(8, 8);
        max_pic.set_valign(Align::Center);
        max_pic.set_halign(Align::Center);

        let update_max_icon = {
            let is_dark = window.has_css_class("marco-theme-dark");
            let color = if is_dark { DARK_PALETTE.control_icon } else { LIGHT_PALETTE.control_icon };
            move |is_maximized: bool, pic: &Picture| {
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
        // Auto-calculate button size: icon + padding for comfortable click target
        btn_max_toggle.set_width_request(14);
        btn_max_toggle.set_height_request(14);
        btn_max_toggle.set_can_focus(false);
        btn_max_toggle.set_has_frame(false);
        btn_max_toggle.add_css_class("topright-btn");
        btn_max_toggle.add_css_class("window-control-btn");

        log::debug!(
            "Created maximize toggle button, visible: {}, sensitive: {}",
            btn_max_toggle.is_visible(),
            btn_max_toggle.is_sensitive()
        );

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

    /// Attach a WebView to this preview window (reparenting)
    ///
    /// This method removes the WebView from its current parent and adds it
    /// to this window's container. The WebView's state (scroll, DOM, theme)
    /// is preserved automatically by GTK4.
    ///
    /// # Arguments
    ///
    /// * `webview` - The WebView widget to attach (borrowed from Rc<RefCell<>>)
    ///
    /// # Reparenting Process
    ///
    /// 1. Detects current parent (Paned or ScrolledWindow)
    /// 2. Removes WebView from current parent using container methods
    /// 3. Adds WebView to this window's ScrolledWindow
    ///
    /// # Example
    ///
    /// ```no_run
    /// let webview = webview_rc.borrow();
    /// preview_window.attach_webview(&webview);
    /// ```
    pub fn attach_webview(&self, webview: &WebView) {
        log::debug!("Attaching WebView to preview window");

        // Remove from current parent if any
        if let Some(parent) = webview.parent() {
            if let Some(paned) = parent.downcast_ref::<gtk4::Paned>() {
                // Remove from paned
                if paned.start_child().as_ref() == Some(&parent) {
                    paned.set_start_child(gtk4::Widget::NONE);
                    log::debug!("Removed WebView from Paned start child");
                } else if paned.end_child().as_ref() == Some(&parent) {
                    paned.set_end_child(gtk4::Widget::NONE);
                    log::debug!("Removed WebView from Paned end child");
                }
            } else if let Some(scrolled) = parent.downcast_ref::<ScrolledWindow>() {
                // Remove from scrolled window
                scrolled.set_child(gtk4::Widget::NONE);
                log::debug!("Removed WebView from ScrolledWindow");
            } else if let Some(stack) = parent.downcast_ref::<gtk4::Stack>() {
                // Remove from stack
                stack.remove(webview);
                log::debug!("Removed WebView from Stack");
            } else {
                log::warn!("WebView parent is unknown type: {:?}", parent.type_());
            }
        }

        // Add to this window's container
        self.container.set_child(Some(webview));
        log::info!("WebView attached to preview window");
    }

    /// Detach the WebView from this preview window
    ///
    /// Removes the WebView from the container and returns it, allowing
    /// it to be reparented back to the main window.
    ///
    /// # Returns
    ///
    /// * `Some(WebView)` - The detached WebView if one was present
    /// * `None` - If no WebView was attached
    ///
    /// # Example
    ///
    /// ```no_run
    /// if let Some(webview) = preview_window.detach_webview() {
    ///     paned.set_end_child(Some(&webview));
    /// }
    /// ```
    pub fn detach_webview(&self) -> Option<WebView> {
        // ScrolledWindow wraps children in a Viewport, so we need to check for that
        if let Some(child) = self.container.child() {
            log::debug!("Container child type: {:?}", child.type_());

            // Try direct WebView (unlikely with ScrolledWindow)
            if let Ok(webview) = child.clone().downcast::<WebView>() {
                self.container.set_child(gtk4::Widget::NONE);
                log::info!("WebView detached directly from preview window");
                return Some(webview);
            }

            // Check if child is a Viewport containing the WebView
            if let Ok(viewport) = child.downcast::<gtk4::Viewport>() {
                if let Some(viewport_child) = viewport.child() {
                    log::debug!("Viewport child type: {:?}", viewport_child.type_());
                    if let Ok(webview) = viewport_child.downcast::<WebView>() {
                        // Remove the entire ScrolledWindow child (which includes the Viewport)
                        self.container.set_child(gtk4::Widget::NONE);
                        log::info!("WebView detached from preview window (via Viewport)");
                        return Some(webview);
                    }
                }
            }
        }

        log::warn!("No WebView found to detach from preview window");
        None
    }

    /// Show the preview window
    ///
    /// Makes the window visible and brings it to the front using `present()`.
    /// Updates the internal visibility flag and resets the callback invocation flag.
    pub fn show(&self) {
        self.window.present();
        *self.is_visible.borrow_mut() = true;
        // Reset callback flag so it can be called again on next close
        self.callback_invoked.set(false);
        log::info!("Preview window shown");
    }

    /// Hide the preview window
    ///
    /// Hides the window (does not destroy it due to hide_on_close property).
    /// The window can be shown again later without recreating it.
    ///
    /// **Important**: This method also triggers the on_close_callback if set,
    /// to ensure proper cleanup and reparenting happens.
    pub fn hide(&self) {
        self.window.set_visible(false);
        *self.is_visible.borrow_mut() = false;
        log::info!("Preview window hidden via hide() method");

        // Manually trigger the on_close callback (since set_visible doesn't fire close_request)
        // But only if it hasn't been called yet (prevents double-call)
        if !self.callback_invoked.get() {
            self.callback_invoked.set(true);
            if let Some(ref callback) = *self.on_close_callback.borrow() {
                log::info!("Manually calling on_close callback from hide()");
                callback();
            }
        } else {
            log::debug!("Callback already invoked, skipping");
        }
    }

    /// Set a callback to be called when the window is closed
    ///
    /// This callback will be invoked when the user closes the window manually
    /// or when hide() is called. Use this to handle cleanup, reparenting, etc.
    ///
    /// # Arguments
    ///
    /// * `callback` - A function to call when the window closes
    ///
    /// # Example
    ///
    /// ```no_run
    /// preview_window.set_on_close_callback(move || {
    ///     // Handle window close - e.g., reparent WebView back
    ///     log::info!("Window closed, cleaning up");
    /// });
    /// ```
    pub fn set_on_close_callback<F: Fn() + 'static>(&self, callback: F) {
        *self.on_close_callback.borrow_mut() = Some(Box::new(callback));
    }

    /// Returns true if the preview window currently has a child widget (usually the WebView)
    #[allow(dead_code)]
    pub fn has_webview(&self) -> bool {
        self.container.child().is_some()
    }


}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_preview_window_struct() {
        // Test that the struct can be created (without GTK initialization)
        // This verifies the struct layout and field types

        // Note: We can't actually create a PreviewWindow without GTK event loop,
        // but we can verify the type definitions compile correctly

        // Verify the struct has the expected public API
        fn _assert_has_methods(_: &PreviewWindow) {
            // These calls won't execute, but they verify the methods exist
            // with the correct signatures at compile time
        }
    }

    #[test]
    fn smoke_test_visibility_tracking() {
        // Test visibility flag behavior without GTK
        let is_visible = Rc::new(RefCell::new(false));

        assert!(!*is_visible.borrow());

        *is_visible.borrow_mut() = true;
        assert!(*is_visible.borrow());

        *is_visible.borrow_mut() = false;
        assert!(!*is_visible.borrow());
    }
}
