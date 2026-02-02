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

use gtk4::prelude::*;
use gtk4::{Label, Window};
use sourceview5::{Buffer, View};
use std::cell::RefCell;
use std::rc::Rc;
use core::logic::cache::SimpleFileCache;

// Re-export public API from the search component
pub use crate::components::search::{
    apply_enhanced_search_highlighting,
    clear_enhanced_search_highlighting,
    SearchOptions,
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
    use crate::components::search::state::{CACHED_SEARCH_WINDOW, CURRENT_BUFFER, CURRENT_SOURCE_VIEW};
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
    use gtk4::{Box as GtkBox, Orientation, Align};
    
    // Get current theme mode from parent window
    let parent_widget = parent.upcast_ref::<gtk4::Widget>();
    let theme_class = if parent_widget.has_css_class("marco-theme-dark") {
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
    let headerbar = gtk4::HeaderBar::new();
    headerbar.add_css_class("titlebar");
    headerbar.add_css_class("marco-titlebar");
    headerbar.set_show_title_buttons(false);
    
    // Set title in headerbar
    let title_label = Label::new(Some("Search & Replace"));
    title_label.set_valign(Align::Center);
    title_label.add_css_class("title-label");
    headerbar.set_title_widget(Some(&title_label));
    
    // Create close button with SVG icon
    let close_button = create_close_button(&theme_class);
    headerbar.pack_end(&close_button);
    
    let window_weak = window.downgrade();
    close_button.connect_clicked(move |_| {
        if let Some(win) = window_weak.upgrade() {
            win.close();
        }
    });
    
    window.set_titlebar(Some(&headerbar));
    
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
            engine::clear_enhanced_search_highlighting,
            state::CACHED_SEARCH_WINDOW,
        };
        
        clear_enhanced_search_highlighting();
        
        CACHED_SEARCH_WINDOW.with(|cached| {
            *cached.borrow_mut() = None;
        });
        
        glib::Propagation::Proceed
    });
    
    window
}

/// Create close button with SVG icon for Windows search window
#[cfg(target_os = "windows")]
fn create_close_button(theme_class: &str) -> gtk4::Button {
    use core::logic::loaders::icon_loader::{window_icon_svg, WindowIcon};
    use gtk4::gdk;
    use rsvg::{CairoRenderer, Loader};
    use gio;
    
    let close_button = gtk4::Button::new();
    close_button.add_css_class("titlebar-button");
    close_button.set_tooltip_text(Some("Close"));
    
    // Determine icon color based on theme
    let icon_color = if theme_class == "marco-theme-dark" {
        "#FFFFFF"
    } else {
        "#000000"
    };
    
    // Load and render SVG icon
    let svg = window_icon_svg(WindowIcon::Close).replace("currentColor", icon_color);
    let bytes = glib::Bytes::from_owned(svg.into_bytes());
    let stream = gio::MemoryInputStream::from_bytes(&bytes);
    
    if let Ok(handle) = Loader::new().read_stream(&stream, None::<&gio::File>, gio::Cancellable::NONE) {
        let icon_size = 20.0;
        let display_scale = gdk::Display::default()
            .and_then(|d| d.monitors().item(0))
            .and_then(|m| m.downcast::<gdk::Monitor>().ok())
            .map(|m| m.scale_factor() as f64)
            .unwrap_or(1.0);
        
        let render_scale = display_scale * 2.0;
        let render_size = (icon_size * render_scale) as i32;
        
        if let Ok(mut surface) = cairo::ImageSurface::create(cairo::Format::ARgb32, render_size, render_size) {
            if let Ok(cr) = cairo::Context::new(&surface) {
                cr.scale(render_scale, render_scale);
                
                let renderer = CairoRenderer::new(&handle);
                let viewport = cairo::Rectangle::new(0.0, 0.0, icon_size, icon_size);
                if renderer.render_document(&cr, &viewport).is_ok() {
                    if let Ok(data) = surface.data() {
                        let bytes = glib::Bytes::from_owned(data.to_vec());
                        let texture = gdk::MemoryTexture::new(
                            render_size,
                            render_size,
                            gdk::MemoryFormat::B8g8r8a8Premultiplied,
                            &bytes,
                            (render_size * 4) as usize,
                        );
                        let image = gtk4::Image::from_paintable(Some(&texture));
                        close_button.set_child(Some(&image));
                    }
                }
            }
        }
    }
    
    close_button
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
