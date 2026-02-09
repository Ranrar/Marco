//! Search Window Creation and Management
//!
//! Handles window creation, dialog management, and window behavior.

#[cfg(target_os = "linux")]
use crate::components::language::SearchTranslations;
use gtk4::prelude::*;
use gtk4::{Label, Window};

#[cfg(target_os = "linux")]
use gtk4::Align;

use super::state::{AsyncSearchManager, ASYNC_MANAGER};

#[cfg(target_os = "linux")]
use super::state::{CACHED_SEARCH_WINDOW, CURRENT_SEARCH_ENTRY};

#[cfg(target_os = "linux")]
use log::trace;

#[cfg(target_os = "linux")]
use std::rc::Rc;

#[cfg(target_os = "linux")]
use super::state::{CURRENT_BUFFER, CURRENT_SOURCE_VIEW, CURRENT_WEBVIEW};

#[cfg(target_os = "linux")]
use sourceview5::{Buffer, View};
#[cfg(target_os = "linux")]
use std::cell::RefCell;
#[cfg(target_os = "linux")]
use webkit6::WebView;

/// Get or create the singleton search window (Linux only)
#[cfg(target_os = "linux")]
pub fn get_or_create_search_window(
    parent: &Window,
    buffer: Rc<Buffer>,
    source_view: Rc<View>,
    webview: Rc<RefCell<WebView>>,
    translations: &SearchTranslations,
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
        let window = Rc::new(create_search_window_impl(parent, translations));

        // Cache the window
        *cached.borrow_mut() = Some(window.clone());

        window
    })
}

/// Create the actual search window implementation
#[cfg(target_os = "linux")]
pub fn create_search_window_impl(parent: &Window, translations: &SearchTranslations) -> Window {
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
    let title_label = Label::new(Some(&translations.title));
    title_label.set_valign(Align::Center);
    title_label.add_css_class("title-label");
    headerbar.set_title_widget(Some(&title_label));

    // Titlebar close button: match Marco's standard window-control look/behavior.
    use crate::ui::css::constants::{DARK_PALETTE, LIGHT_PALETTE};
    use core::logic::loaders::icon_loader::{window_icon_svg, WindowIcon};
    use gio;
    use gtk4::gdk;
    use gtk4::{Button, Picture};
    use rsvg::{CairoRenderer, Loader};

    const ICON_SIZE: f64 = 8.0;

    fn render_svg_icon(icon: WindowIcon, color: &str, icon_size: f64) -> gdk::MemoryTexture {
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

        // Render at 2x the display scale for extra sharpness.
        let render_scale = display_scale * 2.0;
        let render_size = (icon_size * render_scale) as i32;

        let mut surface =
            cairo::ImageSurface::create(cairo::Format::ARgb32, render_size, render_size)
                .expect("create surface");
        {
            let cr = cairo::Context::new(&surface).expect("create context");
            cr.scale(render_scale, render_scale);

            let renderer = CairoRenderer::new(&handle);
            let viewport = cairo::Rectangle::new(0.0, 0.0, icon_size, icon_size);
            renderer
                .render_document(&cr, &viewport)
                .expect("render SVG");
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

    let is_dark = theme_class == "marco-theme-dark";
    let icon_color = if is_dark {
        DARK_PALETTE.control_icon
    } else {
        LIGHT_PALETTE.control_icon
    };
    let hover_color = if is_dark {
        DARK_PALETTE.control_icon_hover
    } else {
        LIGHT_PALETTE.control_icon_hover
    };
    let active_color = if is_dark {
        DARK_PALETTE.control_icon_active
    } else {
        LIGHT_PALETTE.control_icon_active
    };

    let close_pic = Picture::new();
    close_pic.set_paintable(Some(&render_svg_icon(
        WindowIcon::Close,
        icon_color,
        ICON_SIZE,
    )));
    close_pic.set_size_request(ICON_SIZE as i32, ICON_SIZE as i32);
    close_pic.set_can_shrink(false);
    close_pic.set_halign(Align::Center);
    close_pic.set_valign(Align::Center);

    let close_button = Button::new();
    close_button.set_child(Some(&close_pic));
    close_button.set_tooltip_text(Some(&translations.close_tooltip));
    close_button.set_valign(Align::Center);
    close_button.set_margin_start(1);
    close_button.set_margin_end(1);
    close_button.set_focusable(false);
    close_button.set_can_focus(false);
    close_button.set_has_frame(false);
    close_button.set_width_request((ICON_SIZE + 6.0) as i32);
    close_button.set_height_request((ICON_SIZE + 6.0) as i32);
    close_button.add_css_class("topright-btn");
    close_button.add_css_class("window-control-btn");

    {
        let pic_hover = close_pic.clone();
        let motion_controller = gtk4::EventControllerMotion::new();
        motion_controller.connect_enter(move |_ctrl, _x, _y| {
            let texture = render_svg_icon(WindowIcon::Close, hover_color, ICON_SIZE);
            pic_hover.set_paintable(Some(&texture));
        });

        let pic_leave = close_pic.clone();
        motion_controller.connect_leave(move |_ctrl| {
            let texture = render_svg_icon(WindowIcon::Close, icon_color, ICON_SIZE);
            pic_leave.set_paintable(Some(&texture));
        });
        close_button.add_controller(motion_controller);

        let gesture = gtk4::GestureClick::new();
        let pic_pressed = close_pic.clone();
        gesture.connect_pressed(move |_gesture, _n, _x, _y| {
            let texture = render_svg_icon(WindowIcon::Close, active_color, ICON_SIZE);
            pic_pressed.set_paintable(Some(&texture));
        });

        let pic_released = close_pic.clone();
        gesture.connect_released(move |_gesture, _n, _x, _y| {
            let texture = render_svg_icon(WindowIcon::Close, hover_color, ICON_SIZE);
            pic_released.set_paintable(Some(&texture));
        });
        close_button.add_controller(gesture);
    }

    headerbar.pack_end(&close_button);
    let window_weak = window.downgrade();
    close_button.connect_clicked(move |_| {
        if let Some(win) = window_weak.upgrade() {
            win.close();
        }
    });

    window.set_titlebar(Some(&headerbar));

    // Build the main UI
    use super::ui::{
        create_options_panel, create_replace_controls_section, create_search_controls_section,
        create_window_button_panel,
    };
    use gtk4::{Box as GtkBox, Orientation};

    let main_box = GtkBox::new(Orientation::Vertical, 8);
    main_box.set_margin_top(8);
    main_box.set_margin_bottom(8);
    main_box.set_margin_start(8);
    main_box.set_margin_end(8);

    let (search_box, search_entry, match_count_label) =
        create_search_controls_section(translations);
    main_box.append(&search_box);

    let (replace_box, replace_entry) = create_replace_controls_section(translations);
    main_box.append(&replace_box);

    let options_widgets = create_options_panel(translations);
    main_box.append(&options_widgets.0);

    let button_widgets = create_window_button_panel(translations);
    main_box.append(&button_widgets.0);

    // Store the search entry so we can focus it after presenting the window.
    CURRENT_SEARCH_ENTRY.with(|entry_ref| {
        *entry_ref.borrow_mut() = Some(search_entry.clone());
    });

    window.set_child(Some(&main_box));

    // ESC key handler
    let key_controller = gtk4::EventControllerKey::new();
    key_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
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

    // Setup all signal wiring
    setup_window_behavior(
        &window,
        &search_entry,
        &replace_entry,
        &match_count_label,
        &options_widgets,
        &button_widgets,
    );

    // Handle window close (cleanup cache + highlights)
    window.connect_close_request(move |_| {
        use super::engine::clear_enhanced_search_highlighting;

        clear_enhanced_search_highlighting();
        super::state::clear_search_highlighting();

        CURRENT_SEARCH_ENTRY.with(|entry_ref| {
            *entry_ref.borrow_mut() = None;
        });
        CACHED_SEARCH_WINDOW.with(|cached| {
            *cached.borrow_mut() = None;
        });

        glib::Propagation::Proceed
    });

    window
}

/// Focus the search entry when window opens
#[cfg(target_os = "linux")]
pub fn focus_search_entry_in_window(window: &Window) {
    // Ensure the window itself is focused first.
    let _ = window.grab_focus();

    // Then explicitly focus the search entry if we have it.
    CURRENT_SEARCH_ENTRY.with(|entry_ref| {
        if let Some(entry) = entry_ref.borrow().as_ref() {
            entry.grab_focus();
        }
    });
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
    use super::engine::{debounced_search, perform_search};
    use super::navigation::immediate_position_update_with_debounced_navigation;
    use super::replace::{replace_all_matches, replace_next_match};
    use super::state::*;
    use log::debug;

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
            debounced_search(
                &search_entry_clone,
                &match_count_clone,
                &options_clone_for_changed,
            );
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
                perform_search(
                    &search_entry_clone_enter,
                    &match_count_clone_enter,
                    &options_clone_enter,
                );
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
                perform_search(
                    &search_entry_clone_prev,
                    &match_count_clone_prev,
                    &options_clone_prev,
                );
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
                perform_search(
                    &search_entry_clone_next,
                    &match_count_clone_next,
                    &options_clone_next,
                );
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
    button_widgets
        .1
        .replace_all_button
        .connect_clicked(move |_| {
            replace_all_matches(
                &search_entry_clone_replace_all,
                &replace_entry_clone_replace_all,
            );
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
    ]
    .iter()
    {
        let search_entry_clone = search_entry_option.clone();
        let match_count_clone = match_count_option.clone();
        let options_clone = options_for_options.clone();
        checkbox.connect_toggled(move |_cb| {
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
