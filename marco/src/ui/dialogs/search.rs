//! Search & Replace Dialog - Thin Wrapper
//!
//! This module provides platform-specific entry points for the search functionality.
//! All core logic has been moved to `crate::components::search`.
//!
//! ## Entry Points
//!
//! - **Linux**: `show_search_window` - Full-featured search window with WebView integration
//! - **Windows**: `show_search_window_no_webview` - Basic informational message
//!
//! ## Architecture
//!
//! The refactored search component is organized into focused modules:
//! - `state` - State management and thread-local storage
//! - `window` - Window creation and behavior setup
//! - `ui` - UI widget builders
//! - `engine` - Search logic and highlighting
//! - `navigation` - Match navigation and scrolling
//! - `replace` - Replace operations

use core::logic::cache::SimpleFileCache;
use gtk4::prelude::*;
use gtk4::Window;
use sourceview5::{Buffer, View};
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(target_os = "windows")]
use gtk4::Label;

// Re-export public API from the search component
pub use crate::components::search::{
    apply_enhanced_search_highlighting, clear_enhanced_search_highlighting, SearchOptions,
};

#[cfg(target_os = "linux")]
use webkit6::WebView;

/// Entry point for separate search window - shows search in a standalone window (Linux only)
///
/// Creates or reuses a singleton search window with full WebView integration for preview
/// synchronization. The window is non-modal and allows interaction with the main application.
#[cfg(target_os = "linux")]
pub fn show_search_window(
    parent: &Window,
    _file_cache: Rc<RefCell<SimpleFileCache>>,
    buffer: Rc<Buffer>,
    source_view: Rc<View>,
    webview: Rc<RefCell<WebView>>,
) {
    // Initialize async manager for debouncing
    crate::components::search::window::initialize_async_manager();

    // Get or create the search window (singleton pattern)
    let search_window = crate::components::search::window::get_or_create_search_window(
        parent,
        buffer,
        source_view,
        webview,
    );

    // Present the window and focus the search entry
    search_window.present();
    crate::components::search::window::focus_search_entry_in_window(&search_window);
}

/// Windows search window - provides full search functionality without WebView preview sync
#[cfg(target_os = "windows")]
pub fn show_search_window_no_webview(
    parent: &Window,
    _file_cache: Rc<RefCell<SimpleFileCache>>,
    buffer: Rc<Buffer>,
    source_view: Rc<View>,
) {
    use crate::components::search::state::{
        CACHED_SEARCH_WINDOW, CURRENT_BUFFER, CURRENT_SOURCE_VIEW,
    };
    use crate::components::search::window::initialize_async_manager;

    // Initialize async manager
    initialize_async_manager();

    // Store buffer and source view in thread-local storage
    CURRENT_BUFFER.with(|buf| {
        *buf.borrow_mut() = Some(buffer);
    });
    CURRENT_SOURCE_VIEW.with(|view| {
        *view.borrow_mut() = Some(source_view);
    });

    // Check for cached window
    let window = CACHED_SEARCH_WINDOW.with(|cached| {
        if let Some(window) = cached.borrow().as_ref() {
            if window.is_visible() || window.is_active() {
                return window.clone();
            }
        }

        // Create new window
        let win = create_windows_search_window(parent);
        let win_rc = Rc::new(win);
        *cached.borrow_mut() = Some(win_rc.clone());
        win_rc
    });

    window.present();
}

/// Create search window for Windows (without WebView)
#[cfg(target_os = "windows")]
fn create_windows_search_window(parent: &Window) -> Window {
    use crate::components::search::{ui::*, window::setup_window_behavior};
    use gtk4::{Align, Box as GtkBox, Orientation, WindowHandle};

    // Get current theme mode from parent window
    let parent_widget = parent.upcast_ref::<gtk4::Widget>();
    let is_dark = parent_widget.has_css_class("marco-theme-dark");
    let theme_class = if is_dark {
        "marco-theme-dark"
    } else {
        "marco-theme-light"
    };

    let window = Window::builder()
        .transient_for(parent)
        .modal(false)
        .default_width(420)
        .default_height(240)
        .resizable(true)
        .build();

    // Apply CSS classes for theming
    window.add_css_class("marco-search-window");
    window.add_css_class(theme_class);

    // Create custom titlebar matching marco's style
    let handle = WindowHandle::new();
    let headerbar = gtk4::HeaderBar::new();
    headerbar.add_css_class("titlebar");
    headerbar.add_css_class("marco-titlebar");
    headerbar.set_show_title_buttons(false);

    // Set title in headerbar
    let title_label = Label::new(Some("Search & Replace"));
    title_label.set_valign(Align::Center);
    title_label.add_css_class("title-label");
    headerbar.set_title_widget(Some(&title_label));

    // Create close button with the same styling as the main app window controls
    let (close_button, close_pic) = create_close_button(&window);
    headerbar.pack_end(&close_button);

    let window_weak = window.downgrade();
    close_button.connect_clicked(move |_| {
        if let Some(win) = window_weak.upgrade() {
            win.close();
        }
    });

    // Set the headerbar in the WindowHandle for proper dragging
    handle.set_child(Some(&headerbar));
    window.set_titlebar(Some(&handle));

    // Keep search window theme in sync with parent theme (Windows).
    {
        let window_weak = window.downgrade();
        let close_pic = close_pic.clone();
        parent_widget.connect_notify_local(Some("css-classes"), move |parent_widget, _| {
            let Some(window) = window_weak.upgrade() else {
                return;
            };

            let parent_is_dark = parent_widget.has_css_class("marco-theme-dark");
            window.remove_css_class("marco-theme-dark");
            window.remove_css_class("marco-theme-light");
            window.add_css_class(if parent_is_dark {
                "marco-theme-dark"
            } else {
                "marco-theme-light"
            });

            // Refresh the close icon in its normal state for the new theme
            set_window_control_icon(
                &close_pic,
                core::logic::loaders::icon_loader::WindowIcon::Close,
                parent_is_dark,
                WindowControlState::Normal,
            );
        });
    }

    // Main container
    let main_box = GtkBox::new(Orientation::Vertical, 8);
    main_box.set_margin_top(8);
    main_box.set_margin_bottom(8);
    main_box.set_margin_start(8);
    main_box.set_margin_end(8);

    // Search controls
    let (search_box, search_entry, match_count_label) = create_search_controls_section();
    main_box.append(&search_box);

    // Replace controls
    let (replace_box, replace_entry) = create_replace_controls_section();
    main_box.append(&replace_box);

    // Options panel
    let options_widgets = create_options_panel();
    main_box.append(&options_widgets.0);

    // Button panel
    let button_widgets = create_window_button_panel();
    main_box.append(&button_widgets.0);

    window.set_child(Some(&main_box));

    // ESC key handler
    let key_controller = gtk4::EventControllerKey::new();
    let window_weak = window.downgrade();
    key_controller.connect_key_pressed(move |_controller, key, _code, _state| {
        if key == gtk4::gdk::Key::Escape {
            if let Some(win) = window_weak.upgrade() {
                win.close();
            }
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    window.add_controller(key_controller);

    // Setup window behavior
    setup_window_behavior(
        &window,
        &search_entry,
        &replace_entry,
        &match_count_label,
        &options_widgets,
        &button_widgets,
    );

    // Handle window close
    window.connect_close_request(move |_| {
        use crate::components::search::{
            engine::clear_enhanced_search_highlighting, state::CACHED_SEARCH_WINDOW,
        };

        clear_enhanced_search_highlighting();

        CACHED_SEARCH_WINDOW.with(|cached| {
            *cached.borrow_mut() = None;
        });

        glib::Propagation::Proceed
    });

    window
}

/// Window-control icon states (normal/hover/active)
#[cfg(target_os = "windows")]
#[derive(Clone, Copy, Debug)]
enum WindowControlState {
    Normal,
    Hover,
    Active,
}

/// Render and apply a window-control SVG icon into a Picture, using the same palette
/// colors as the main application window controls.
#[cfg(target_os = "windows")]
fn set_window_control_icon(
    pic: &gtk4::Picture,
    icon: core::logic::loaders::icon_loader::WindowIcon,
    is_dark: bool,
    state: WindowControlState,
) {
    use crate::ui::css::constants::{DARK_PALETTE, LIGHT_PALETTE};
    use core::logic::loaders::icon_loader::window_icon_svg;
    use gio;
    use gtk4::gdk;
    use rsvg::{CairoRenderer, Loader};

    let color = match (is_dark, state) {
        (true, WindowControlState::Normal) => DARK_PALETTE.control_icon,
        (true, WindowControlState::Hover) => DARK_PALETTE.control_icon_hover,
        (true, WindowControlState::Active) => DARK_PALETTE.control_icon_active,
        (false, WindowControlState::Normal) => LIGHT_PALETTE.control_icon,
        (false, WindowControlState::Hover) => LIGHT_PALETTE.control_icon_hover,
        (false, WindowControlState::Active) => LIGHT_PALETTE.control_icon_active,
    };

    let icon_size = 8.0;
    let svg = window_icon_svg(icon).replace("currentColor", color);
    let bytes = glib::Bytes::from_owned(svg.into_bytes());
    let stream = gio::MemoryInputStream::from_bytes(&bytes);
    let handle =
        match Loader::new().read_stream(&stream, None::<&gio::File>, gio::Cancellable::NONE) {
            Ok(h) => h,
            Err(e) => {
                log::error!("load SVG handle: {}", e);
                return;
            }
        };

    let display_scale = gdk::Display::default()
        .and_then(|d| d.monitors().item(0))
        .and_then(|m| m.downcast::<gdk::Monitor>().ok())
        .map(|m| m.scale_factor() as f64)
        .unwrap_or(1.0);
    let render_scale = display_scale * 2.0;
    let render_size = (icon_size * render_scale) as i32;

    let mut surface =
        match cairo::ImageSurface::create(cairo::Format::ARgb32, render_size, render_size) {
            Ok(s) => s,
            Err(e) => {
                log::error!("create surface: {}", e);
                return;
            }
        };
    {
        let cr = match cairo::Context::new(&surface) {
            Ok(c) => c,
            Err(e) => {
                log::error!("create cairo context: {}", e);
                return;
            }
        };
        cr.scale(render_scale, render_scale);
        let renderer = CairoRenderer::new(&handle);
        let viewport = cairo::Rectangle::new(0.0, 0.0, icon_size, icon_size);
        if let Err(e) = renderer.render_document(&cr, &viewport) {
            log::error!("render SVG: {}", e);
            return;
        }
    }

    let data = match surface.data() {
        Ok(d) => d.to_vec(),
        Err(e) => {
            log::error!("get surface data: {}", e);
            return;
        }
    };
    let bytes = glib::Bytes::from_owned(data);
    let texture = gdk::MemoryTexture::new(
        render_size,
        render_size,
        gdk::MemoryFormat::B8g8r8a8Premultiplied,
        &bytes,
        (render_size * 4) as usize,
    );
    pic.set_paintable(Some(&texture));
    pic.set_size_request(icon_size as i32, icon_size as i32);
    pic.set_can_shrink(false);
    pic.set_halign(gtk4::Align::Center);
    pic.set_valign(gtk4::Align::Center);
}

/// Create a close button that matches the main app's window control styling.
#[cfg(target_os = "windows")]
fn create_close_button(window: &gtk4::Window) -> (gtk4::Button, gtk4::Picture) {
    use core::logic::loaders::icon_loader::WindowIcon;
    use gtk4::prelude::*;
    use gtk4::{Button, Picture};

    let pic = Picture::new();
    let is_dark = window.has_css_class("marco-theme-dark");
    set_window_control_icon(&pic, WindowIcon::Close, is_dark, WindowControlState::Normal);

    let btn = Button::new();
    btn.set_child(Some(&pic));
    btn.set_tooltip_text(Some("Close"));
    btn.set_valign(gtk4::Align::Center);
    btn.set_margin_start(1);
    btn.set_margin_end(1);
    btn.set_focusable(false);
    btn.set_can_focus(false);
    btn.set_has_frame(false);
    btn.add_css_class("topright-btn");
    btn.add_css_class("window-control-btn");
    btn.set_width_request((8.0 + 6.0) as i32);
    btn.set_height_request((8.0 + 6.0) as i32);

    // Hover/active behavior reads the current theme at event time.
    {
        let pic_hover = pic.clone();
        let win_weak = window.downgrade();
        let motion_controller = gtk4::EventControllerMotion::new();
        motion_controller.connect_enter(move |_ctrl, _x, _y| {
            let Some(win) = win_weak.upgrade() else {
                return;
            };
            let is_dark = win.has_css_class("marco-theme-dark");
            set_window_control_icon(
                &pic_hover,
                WindowIcon::Close,
                is_dark,
                WindowControlState::Hover,
            );
        });

        let pic_leave = pic.clone();
        let win_weak = window.downgrade();
        motion_controller.connect_leave(move |_ctrl| {
            let Some(win) = win_weak.upgrade() else {
                return;
            };
            let is_dark = win.has_css_class("marco-theme-dark");
            set_window_control_icon(
                &pic_leave,
                WindowIcon::Close,
                is_dark,
                WindowControlState::Normal,
            );
        });
        btn.add_controller(motion_controller);

        let gesture = gtk4::GestureClick::new();
        let pic_pressed = pic.clone();
        let win_weak = window.downgrade();
        gesture.connect_pressed(move |_gesture, _n, _x, _y| {
            let Some(win) = win_weak.upgrade() else {
                return;
            };
            let is_dark = win.has_css_class("marco-theme-dark");
            set_window_control_icon(
                &pic_pressed,
                WindowIcon::Close,
                is_dark,
                WindowControlState::Active,
            );
        });

        let pic_released = pic.clone();
        let win_weak = window.downgrade();
        gesture.connect_released(move |_gesture, _n, _x, _y| {
            let Some(win) = win_weak.upgrade() else {
                return;
            };
            let is_dark = win.has_css_class("marco-theme-dark");
            set_window_control_icon(
                &pic_released,
                WindowIcon::Close,
                is_dark,
                WindowControlState::Hover,
            );
        });
        btn.add_controller(gesture);
    }

    (btn, pic)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_search_options() {
        let options = SearchOptions {
            match_case: true,
            match_whole_word: false,
            match_markdown_only: true,
            use_regex: false,
        };

        assert!(options.match_case);
        assert!(!options.match_whole_word);
        assert!(options.match_markdown_only);
        assert!(!options.use_regex);
    }

    #[test]
    fn smoke_test_search_options_default() {
        let options = SearchOptions::default();

        assert!(!options.match_case);
        assert!(!options.match_whole_word);
        assert!(!options.match_markdown_only);
        assert!(!options.use_regex);
    }

    #[test]
    fn smoke_test_api_reexports() {
        // Verify that the public API functions are accessible
        let _highlight_fn = apply_enhanced_search_highlighting;
        let _clear_fn = clear_enhanced_search_highlighting;

        // Test passes if this compiles - functions are properly re-exported
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn smoke_test_linux_entry_point() {
        // Verify the Linux entry point exists and is callable
        let _entry_point = show_search_window;
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn smoke_test_windows_entry_point() {
        // Verify the Windows entry point exists and is callable
        let _entry_point = show_search_window_no_webview;
    }
}
