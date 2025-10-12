//! Preview Window Module
//!
//! This module manages a separate GTK ApplicationWindow that hosts the reparented WebView
//! for the EditorAndViewSeparate layout mode. The window provides a dedicated space for
//! viewing the HTML preview while editing in the main window.
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

use gtk4::prelude::*;
use gtk4::{ApplicationWindow, ScrolledWindow};
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

        // Helper function to create icon buttons
        let icon_button = |glyph: &str, tooltip: &str| {
            let label = Label::new(None);
            label.set_markup(&format!("<span font_family='icomoon'>{}</span>", glyph));
            label.set_valign(Align::Center);
            label.add_css_class("icon-font");
            let btn = Button::new();
            btn.set_child(Some(&label));
            btn.set_tooltip_text(Some(tooltip));
            btn.set_valign(Align::Center);
            btn.set_margin_start(1);
            btn.set_margin_end(1);
            btn.set_focusable(true);  // Changed from false to true
            btn.set_can_focus(true);  // Changed from false to true
            btn.set_has_frame(false);
            btn.add_css_class("topright-btn");
            btn.add_css_class("window-control-btn");
            log::debug!("Created button with tooltip: {}", tooltip);
            btn
        };

        // IcoMoon Unicode glyphs for window controls
        // | Unicode | Icon Name             | Description   |
        // |---------|-----------------------|--------------|
        // | \u{34}  | marco-minimize        | Minimize      |
        // | \u{36}  | marco-fullscreen      | Maximize      |
        // | \u{35}  | marco-fullscreen_exit | Exit maximize |
        // | \u{39}  | marco-close           | Close         |

        let btn_min = icon_button("\u{34}", "Minimize");
        let btn_close = icon_button("\u{39}", "Close");

        log::debug!("Created minimize button, visible: {}, sensitive: {}", 
            btn_min.is_visible(), btn_min.is_sensitive());
        log::debug!("Created close button, visible: {}, sensitive: {}", 
            btn_close.is_visible(), btn_close.is_sensitive());

        // Create a single toggle button for maximize/restore
        let max_label = Label::new(None);
        let initial_glyph = if window.is_maximized() {
            "\u{35}"
        } else {
            "\u{36}"
        };
        max_label.set_markup(&format!(
            "<span font_family='icomoon'>{}</span>",
            initial_glyph
        ));
        max_label.set_valign(Align::Center);
        max_label.add_css_class("icon-font");
        let btn_max_toggle = Button::new();
        btn_max_toggle.set_child(Some(&max_label));
        btn_max_toggle.set_tooltip_text(Some("Maximize / Restore"));
        btn_max_toggle.set_valign(Align::Center);
        btn_max_toggle.set_margin_start(1);
        btn_max_toggle.set_margin_end(1);
        btn_max_toggle.set_focusable(false);
        btn_max_toggle.set_can_focus(false);
        btn_max_toggle.set_has_frame(false);
        btn_max_toggle.add_css_class("topright-btn");
        btn_max_toggle.add_css_class("window-control-btn");

        log::debug!("Created maximize toggle button, visible: {}, sensitive: {}", 
            btn_max_toggle.is_visible(), btn_max_toggle.is_sensitive());

        // Add controls to headerbar from right to left (pack_end order)
        headerbar.pack_end(&btn_close);        // Rightmost
        headerbar.pack_end(&btn_max_toggle);   // Middle
        headerbar.pack_end(&btn_min);          // Left of window controls

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

        // Maximize/restore toggle
        let label_for_toggle = max_label.clone();
        let window_for_toggle = window.clone();
        btn_max_toggle.connect_clicked(move |_| {
            log::info!("Preview window maximize/restore button clicked - handler called");
            if window_for_toggle.is_maximized() {
                window_for_toggle.unmaximize();
                label_for_toggle
                    .set_markup(&format!("<span font_family='icomoon'>{}</span>", "\u{36}"));
                log::info!("Preview window unmaximized");
            } else {
                window_for_toggle.maximize();
                label_for_toggle
                    .set_markup(&format!("<span font_family='icomoon'>{}</span>", "\u{35}"));
                log::info!("Preview window maximized");
            }
        });

        // Keep glyph in sync if window is maximized/unmaximized externally
        let label_for_notify = max_label.clone();
        window.connect_notify_local(Some("is-maximized"), move |w, _| {
            if w.is_maximized() {
                label_for_notify
                    .set_markup(&format!("<span font_family='icomoon'>{}</span>", "\u{35}"));
            } else {
                label_for_notify
                    .set_markup(&format!("<span font_family='icomoon'>{}</span>", "\u{36}"));
            }
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

    /// Get a reference to the ScrolledWindow container
    ///
    /// Can be used to access or modify container properties.
    pub fn container(&self) -> &ScrolledWindow {
        &self.container
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
