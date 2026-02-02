//! Search Window Creation and Management
//!
//! Handles window creation, dialog management, and window behavior.

use gtk4::prelude::*;
use gtk4::{Align, Label, Window};
use log::trace;
use std::rc::Rc;

use super::state::{ASYNC_MANAGER, AsyncSearchManager, CACHED_SEARCH_WINDOW, CURRENT_BUFFER, CURRENT_SOURCE_VIEW, CURRENT_WEBVIEW};

#[cfg(target_os = "linux")]
use sourceview5::{Buffer, View};
#[cfg(target_os = "linux")]
use webkit6::WebView;
#[cfg(target_os = "linux")]
use std::cell::RefCell;

/// Get or create the singleton search window (Linux only)
#[cfg(target_os = "linux")]
pub fn get_or_create_search_window(
    parent: &Window,
    buffer: Rc<Buffer>,
    source_view: Rc<View>,
    webview: Rc<RefCell<WebView>>,
) -> Rc<Window> {
    // Store the current buffer, source view, and webview
    CURRENT_BUFFER.with(|buf| {
        *buf.borrow_mut() = Some(buffer);
    });
    CURRENT_SOURCE_VIEW.with(|view| {
        *view.borrow_mut() = Some(source_view);
    });
    CURRENT_WEBVIEW.with(|web| {
        *web.borrow_mut() = Some(webview);
    });

    CACHED_SEARCH_WINDOW.with(|cached| {
        // Check if we have a valid cached window
        if let Some(window) = cached.borrow().as_ref() {
            // Check if the window is still valid
            if window.is_visible() || window.is_active() {
                trace!("audit: reusing cached search window");
                return window.clone();
            } else {
                // Window was destroyed, clear the cache
                trace!("audit: clearing destroyed window from cache");
                *cached.borrow_mut() = None;
            }
        }

        // Create new window if none cached or previous was destroyed
        trace!("audit: creating new search window");
        let window = Rc::new(create_search_window_impl(parent));

        // Cache the window
        *cached.borrow_mut() = Some(window.clone());

        window
    })
}

/// Create the actual search window implementation
pub fn create_search_window_impl(parent: &Window) -> Window {
    // Get current theme mode from parent window
    let parent_widget = parent.upcast_ref::<gtk4::Widget>();
    let theme_class = if parent_widget.has_css_class("marco-theme-dark") {
        "marco-theme-dark"
    } else {
        "marco-theme-light"
    };

    let window = Window::builder()
        .transient_for(parent)
        .modal(false) // Non-modal so we can interact with main app
        .default_width(420)
        .default_height(240)
        .resizable(true) // Allow resizing for better usability
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

    // Create custom close button with SVG icon
    use core::logic::loaders::icon_loader::{window_icon_svg, WindowIcon};
    use gtk4::gdk;
    use rsvg::{CairoRenderer, Loader};
    use gio;

    fn render_svg_icon(icon: WindowIcon, color: &str, icon_size: f64) -> gdk::MemoryTexture {
        let svg = window_icon_svg(icon).replace("currentColor", color);
        let bytes = glib::Bytes::from_owned(svg.into_bytes());
        let stream = gio::MemoryInputStream::from_bytes(&bytes);

        let handle = match Loader::new().read_stream(&stream, None::<&gio::File>, gio::Cancellable::NONE) {
            Ok(h) => h,
            Err(e) => {
                log::error!("load SVG handle: {}", e);
                let bytes = glib::Bytes::from_owned(vec![0u8, 0u8, 0u8, 0u8]);
                return gdk::MemoryTexture::new(1, 1, gdk::MemoryFormat::B8g8r8a8Premultiplied, &bytes, 4);
            }
        };

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

    // TODO: Complete window implementation - add UI widgets, close button, event handlers
    // This will be filled in during the refactoring process

    window.set_titlebar(Some(&headerbar));
    window
}

/// Focus the search entry when window opens
pub fn focus_search_entry_in_window(window: &Window) {
    let _ = window.grab_focus();
}

/// Setup all window behavior and signal connections
pub fn setup_window_behavior(
    _window: &Window,
    search_entry: &gtk4::Entry,
    replace_entry: &gtk4::Entry,
    match_count_label: &Label,
    options_widgets: &(gtk4::Box, super::ui::OptionsWidgets),
    button_widgets: &(gtk4::Box, super::ui::ButtonWidgets),
) {
    use log::debug;
    use super::state::*;
    use super::engine::{debounced_search, perform_search};
    use super::navigation::immediate_position_update_with_debounced_navigation;
    use super::replace::{replace_next_match, replace_all_matches};

    // Search entry live updates (when text is typed in the entry)
    let match_count_clone = match_count_label.clone();
    let options_clone = super::ui::OptionsWidgets {
        match_case_cb: options_widgets.1.match_case_cb.clone(),
        match_whole_word_cb: options_widgets.1.match_whole_word_cb.clone(),
        match_markdown_cb: options_widgets.1.match_markdown_cb.clone(),
        use_regex_cb: options_widgets.1.use_regex_cb.clone(),
    };
    let search_entry_clone = search_entry.clone();

    // Connect to the entry for live updates
    let options_clone_for_changed = options_clone.clone();
    search_entry.connect_changed(move |_entry| {
        use super::engine::clear_enhanced_search_highlighting;
        
        // Clear old highlights immediately when text changes
        clear_enhanced_search_highlighting();
        
        let query = search_entry_clone.text().to_string();
        if !query.is_empty() {
            debounced_search(&search_entry_clone, &match_count_clone, &options_clone_for_changed);
        } else {
            // Clear both state and visual highlighting
            clear_search_highlighting();
            match_count_clone.set_text("");
        }
    });

    // Connect Enter key to perform search and navigate
    let search_entry_clone_enter = search_entry.clone();
    let match_count_clone_enter = match_count_label.clone();
    let options_clone_enter = options_clone.clone();
    search_entry.connect_activate(move |_entry| {
        let query = search_entry_clone_enter.text().to_string();
        if !query.is_empty() {
            let needs_search = CURRENT_SEARCH_STATE.with(|state_ref| state_ref.borrow().is_none());
            if needs_search {
                perform_search(&search_entry_clone_enter, &match_count_clone_enter, &options_clone_enter);
            }

            let needs_position_reset = CURRENT_MATCH_POSITION.with(|pos| pos.borrow().is_none());
            if needs_position_reset {
                let position = super::navigation::find_position_from_cursor().unwrap_or(0);
                CURRENT_MATCH_POSITION.with(|pos| *pos.borrow_mut() = Some(position));
            }

            immediate_position_update_with_debounced_navigation(1, 100);
        }
    });

    // Previous button
    let search_entry_clone_prev = search_entry.clone();
    let match_count_clone_prev = match_count_label.clone();
    let options_clone_prev = options_clone.clone();
    button_widgets.1.prev_button.connect_clicked(move |_| {
        let needs_search = CURRENT_SEARCH_STATE.with(|state_ref| state_ref.borrow().is_none());
        if needs_search {
            let query = search_entry_clone_prev.text().to_string();
            if !query.is_empty() {
                perform_search(&search_entry_clone_prev, &match_count_clone_prev, &options_clone_prev);
            }
        }

        let needs_position_reset = CURRENT_MATCH_POSITION.with(|pos| pos.borrow().is_none());
        if needs_position_reset {
            let position = super::navigation::find_position_before_cursor().unwrap_or(2);
            CURRENT_MATCH_POSITION.with(|pos| *pos.borrow_mut() = Some(position));
        }

        immediate_position_update_with_debounced_navigation(-1, 200);
    });

    // Next button
    let search_entry_clone_next = search_entry.clone();
    let match_count_clone_next = match_count_label.clone();
    let options_clone_next = options_clone.clone();
    button_widgets.1.next_button.connect_clicked(move |_| {
        let needs_search = CURRENT_SEARCH_STATE.with(|state_ref| state_ref.borrow().is_none());
        if needs_search {
            let query = search_entry_clone_next.text().to_string();
            if !query.is_empty() {
                perform_search(&search_entry_clone_next, &match_count_clone_next, &options_clone_next);
            }
        }

        let needs_position_reset = CURRENT_MATCH_POSITION.with(|pos| pos.borrow().is_none());
        if needs_position_reset {
            let position = super::navigation::find_position_from_cursor().unwrap_or(0);
            CURRENT_MATCH_POSITION.with(|pos| *pos.borrow_mut() = Some(position));
        }

        immediate_position_update_with_debounced_navigation(1, 200);
    });

    // Replace button
    let search_entry_clone_replace = search_entry.clone();
    let replace_entry_clone_replace = replace_entry.clone();
    button_widgets.1.replace_button.connect_clicked(move |_| {
        replace_next_match(&search_entry_clone_replace, &replace_entry_clone_replace);
    });

    // Replace All button
    let search_entry_clone_replace_all = search_entry.clone();
    let replace_entry_clone_replace_all = replace_entry.clone();
    button_widgets.1.replace_all_button.connect_clicked(move |_| {
        replace_all_matches(&search_entry_clone_replace_all, &replace_entry_clone_replace_all);
    });

    // Connect option checkboxes to re-run search when changed
    let search_entry_option = search_entry.clone();
    let match_count_option = match_count_label.clone();
    let options_for_options = options_clone.clone();
    for checkbox in [
        &options_widgets.1.match_case_cb,
        &options_widgets.1.match_whole_word_cb,
        &options_widgets.1.match_markdown_cb,
        &options_widgets.1.use_regex_cb,
    ] {
        let search_entry_clone = search_entry_option.clone();
        let match_count_clone = match_count_option.clone();
        let options_clone = options_for_options.clone();
        checkbox.connect_toggled(move |_| {
            let query = search_entry_clone.text().to_string();
            if !query.is_empty() {
                perform_search(&search_entry_clone, &match_count_clone, &options_clone);
            }
        });
    }

    // Enable/disable replace buttons based on replace text
    let replace_button_clone = button_widgets.1.replace_button.clone();
    let replace_all_button_clone = button_widgets.1.replace_all_button.clone();
    replace_entry.connect_changed(move |entry| {
        let has_text = !entry.text().is_empty();
        replace_button_clone.set_sensitive(has_text);
        replace_all_button_clone.set_sensitive(has_text);
    });

    debug!("Window behavior setup completed");
}

/// Initialize async manager
pub fn initialize_async_manager() {
    ASYNC_MANAGER.with(|manager_ref| {
        if manager_ref.borrow().is_none() {
            *manager_ref.borrow_mut() = Some(AsyncSearchManager::new());
        }
    });
}
