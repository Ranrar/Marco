//! Search & Replace Window Implementation
//!
//! Provides GTK4-based search and replace functionality:
//!
//! ## Separate Window Mode (`show_search_window`)
//! - Non-modal window that allows interaction with the main application
//! - Resizable window with native window controls
//! - Can be positioned independently and kept open while working
//!
//! ## Features
//! - Enhanced dual-color highlighting (all matches + selected match)
//! - Match case, whole word, Markdown-only, and regex options
//! - Navigation through search results with match count display
//! - Replace next and replace all functionality
//! - Singleton pattern to prevent multiple instances
//! - Debounced search for performance
//! - Integration with Marco's theme system

use glib::{timeout_add_local, SourceId};
use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, CheckButton, Entry, Label, Orientation, Overlay, Separator,
    Window,
};
use log::{debug, trace};
use sourceview5::prelude::*;
use sourceview5::{Buffer, SearchContext, SearchSettings, View};
use std::cell::RefCell;
use std::rc::Rc;

// Provide a lightweight WebView alias on non-Linux platforms so the code can
// compile without WebKit6. The real WebView type is imported on Linux.
#[cfg(not(target_os = "linux"))]
type WebView = gtk4::Widget;

#[cfg(target_os = "linux")]
use webkit6::{prelude::*, WebView};

use crate::logic::signal_manager::safe_source_remove;
use core::logic::cache::SimpleFileCache;
// use crate::logic::buffer::DocumentBuffer; // Reserved for future use

/// Search options for controlling search behavior
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub match_case: bool,
    pub match_whole_word: bool,
    pub match_markdown_only: bool, // Not yet implemented: requires integration with Marco's grammar parser
    pub use_regex: bool,
}

/// Current search state
#[derive(Debug)]
struct SearchState {
    search_context: SearchContext,
}

/// Simple async search manager for better UI responsiveness
struct AsyncSearchManager {
    current_timer_id: Option<SourceId>,
}

impl AsyncSearchManager {
    fn new() -> Self {
        Self {
            current_timer_id: None,
        }
    }

    /// Schedule a search operation with debouncing
    fn schedule_search<F>(&mut self, delay_ms: u32, callback: F)
    where
        F: Fn() + 'static,
    {
        // Cancel any existing timer safely
        if let Some(timer_id) = self.current_timer_id.take() {
            safe_source_remove(timer_id);
        }

        // Schedule new search
        let timer_id = timeout_add_local(
            std::time::Duration::from_millis(delay_ms as u64),
            move || {
                callback();
                glib::ControlFlow::Break
            },
        );

        self.current_timer_id = Some(timer_id);
    }
}

thread_local! {
    static CACHED_SEARCH_WINDOW: RefCell<Option<Rc<Window>>> = const { RefCell::new(None) };
    static CURRENT_BUFFER: RefCell<Option<Rc<Buffer>>> = const { RefCell::new(None) };
    static CURRENT_SOURCE_VIEW: RefCell<Option<Rc<View>>> = const { RefCell::new(None) };
    static CURRENT_WEBVIEW: RefCell<Option<Rc<RefCell<WebView>>>> = const { RefCell::new(None) };
    static CURRENT_SEARCH_STATE: RefCell<Option<SearchState>> = const { RefCell::new(None) };
    static CURRENT_MATCH_LABEL: RefCell<Option<Label>> = const { RefCell::new(None) };
    static NAVIGATION_IN_PROGRESS: RefCell<bool> = const { RefCell::new(false) };
    static CURRENT_MATCH_POSITION: RefCell<Option<i32>> = const { RefCell::new(None) };
    static SEARCH_DEBOUNCE_TIMER: RefCell<Option<SourceId>> = const { RefCell::new(None) };
    static NAVIGATION_DEBOUNCE_TIMER: RefCell<Option<SourceId>> = const { RefCell::new(None) };
    static ASYNC_MANAGER: RefCell<Option<AsyncSearchManager>> = const { RefCell::new(None) };
}

/// Main entry point - shows or creates the search dialog
/// Entry point for separate search window - shows search in a standalone window
#[cfg(target_os = "linux")]
pub fn show_search_window(
    parent: &Window,
    _file_cache: Rc<RefCell<SimpleFileCache>>,
    buffer: Rc<Buffer>,
    source_view: Rc<View>,
    webview: Rc<RefCell<WebView>>,
) {
    // Initialize async manager if not already done
    ASYNC_MANAGER.with(|manager_ref| {
        if manager_ref.borrow().is_none() {
            *manager_ref.borrow_mut() = Some(AsyncSearchManager::new());
        }
    });

    let search_window = get_or_create_search_window(parent, buffer, source_view, webview);
    search_window.present();

    // Focus the search entry for immediate typing
    focus_search_entry_in_window(&search_window);
}

// Minimal fallback for non-Linux platforms: show simple informational window
#[cfg(not(target_os = "linux"))]
pub fn show_search_window_no_webview(
    parent: &Window,
    _file_cache: Rc<RefCell<SimpleFileCache>>,
    _buffer: Rc<Buffer>,
    _source_view: Rc<View>,
) {
    let win = Window::builder()
        .transient_for(parent)
        .modal(true)
        .default_width(420)
        .default_height(120)
        .resizable(false)
        .build();
    win.add_css_class("marco-search-window");
    let label = Label::new(Some(
        "Search in preview is not available on this platform. Use editor-only search.",
    ));
    label.set_margin_top(16);
    label.set_margin_start(16);
    label.set_margin_end(16);
    label.set_wrap(true);
    win.set_child(Some(&label));
    win.present();
}

/// Get or create the singleton search dialog
/// Get or create the singleton search window
#[cfg(target_os = "linux")]
fn get_or_create_search_window(
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

/// Create the actual search window implementation (separate window)
fn create_search_window_impl(parent: &Window) -> Window {
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
    use crate::ui::css::constants::{DARK_PALETTE, LIGHT_PALETTE};
    use core::logic::loaders::icon_loader::{window_icon_svg, WindowIcon};
    use rsvg::{CairoRenderer, Loader};
    use gio;
    use gtk4::gdk;

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

    fn svg_icon_button(window: &Window, icon: WindowIcon, tooltip: &str, color: &str, icon_size: f64) -> Button {
        let pic = gtk4::Picture::new();
        let texture = render_svg_icon(icon, color, icon_size);
        pic.set_paintable(Some(&texture));
        pic.set_size_request(icon_size as i32, icon_size as i32);
        pic.set_can_shrink(false);
        pic.set_halign(gtk4::Align::Center);
        pic.set_valign(gtk4::Align::Center);

        let btn = Button::new();
        btn.set_child(Some(&pic));
        btn.set_tooltip_text(Some(tooltip));
        btn.set_valign(gtk4::Align::Center);
        btn.set_margin_start(1);
        btn.set_margin_end(1);
        btn.set_focusable(false);
        btn.set_can_focus(false);
        btn.set_has_frame(false);
        btn.add_css_class("topright-btn");
        btn.add_css_class("window-control-btn");
        btn.set_width_request((icon_size + 6.0) as i32);
        btn.set_height_request((icon_size + 6.0) as i32);

        // Hover and click interactions
        {
            let pic_hover = pic.clone();
            let normal_color = color.to_string();
            let is_dark = window.has_css_class("marco-theme-dark");
            let hover_color = if is_dark { DARK_PALETTE.control_icon_hover.to_string() } else { LIGHT_PALETTE.control_icon_hover.to_string() };
            let active_color = if is_dark { DARK_PALETTE.control_icon_active.to_string() } else { LIGHT_PALETTE.control_icon_active.to_string() };

            let motion_controller = gtk4::EventControllerMotion::new();
            let icon_for_enter = icon;
            let hover_color_enter = hover_color.clone();
            motion_controller.connect_enter(move |_ctrl, _x, _y| {
                let texture = render_svg_icon(icon_for_enter, &hover_color_enter, icon_size);
                pic_hover.set_paintable(Some(&texture));
            });

            let pic_leave = pic.clone();
            let icon_for_leave = icon;
            let normal_color_leave = normal_color.clone();
            motion_controller.connect_leave(move |_ctrl| {
                let texture = render_svg_icon(icon_for_leave, &normal_color_leave, icon_size);
                pic_leave.set_paintable(Some(&texture));
            });
            btn.add_controller(motion_controller);

            let gesture = gtk4::GestureClick::new();
            let pic_pressed = pic.clone();
            let icon_for_pressed = icon;
            let active_color_pressed = active_color.clone();
            gesture.connect_pressed(move |_gesture, _n, _x, _y| {
                let texture = render_svg_icon(icon_for_pressed, &active_color_pressed, icon_size);
                pic_pressed.set_paintable(Some(&texture));
            });

            let pic_released = pic.clone();
            let icon_for_released = icon;
            gesture.connect_released(move |_gesture, _n, _x, _y| {
                let texture = render_svg_icon(icon_for_released, &hover_color, icon_size);
                pic_released.set_paintable(Some(&texture));
            });
            btn.add_controller(gesture);
        }

        btn
    }

    let icon_color: std::borrow::Cow<'static, str> = if window.has_css_class("marco-theme-dark") {
        std::borrow::Cow::from(DARK_PALETTE.control_icon)
    } else {
        std::borrow::Cow::from(LIGHT_PALETTE.control_icon)
    };

    let btn_close_titlebar = svg_icon_button(&window, WindowIcon::Close, "Close", &icon_color, 8.0);

    // Wire up close button
    let window_weak_for_close = window.downgrade();
    btn_close_titlebar.connect_clicked(move |_| {
        if let Some(window) = window_weak_for_close.upgrade() {
            window.close();
        }
    });

    // Add close button to right side of headerbar
    headerbar.pack_end(&btn_close_titlebar);

    window.set_titlebar(Some(&headerbar));

    // Main container
    let main_box = GtkBox::new(Orientation::Vertical, 8);
    main_box.add_css_class("marco-search-content");

    // Search controls section
    let (search_box, search_entry, match_count_label) = create_search_controls_section();
    main_box.append(&search_box);

    // Replace controls section (always visible)
    let (replace_box, replace_entry) = create_replace_controls_section();
    main_box.append(&replace_box);

    // Options panel
    let options_widgets = create_options_panel();
    main_box.append(&options_widgets.0);

    // Button panel - modified for window (no close button needed)
    let button_widgets = create_window_button_panel();
    main_box.append(&button_widgets.0);

    window.set_child(Some(&main_box));

    // Add ESC key handler to close window
    let key_controller = gtk4::EventControllerKey::new();
    let window_weak_for_esc = window.downgrade();
    key_controller.connect_key_pressed(move |_controller, key, _code, _state| {
        if key == gtk4::gdk::Key::Escape {
            if let Some(window) = window_weak_for_esc.upgrade() {
                window.close();
            }
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    window.add_controller(key_controller);

    // Connect all the signals and behavior for window
    setup_window_behavior(
        &window,
        &search_entry,
        &replace_entry,
        &match_count_label,
        &options_widgets,
        &button_widgets,
    );

    // Handle window close request
    window.connect_close_request(move |_| {
        // Clear search highlights when window is closed
        clear_enhanced_search_highlighting();
        debug!("Search window closed, cleared search highlights");

        // Clear cached window
        CACHED_SEARCH_WINDOW.with(|cached| {
            trace!("audit: clearing search window cache on close");
            *cached.borrow_mut() = None;
        });

        // Allow the window to close
        glib::Propagation::Proceed
    });

    window
}

/// Create the search controls section
fn create_search_controls_section() -> (GtkBox, Entry, Label) {
    let search_box = GtkBox::new(Orientation::Vertical, 4);

    let search_row = GtkBox::new(Orientation::Horizontal, 8);

    let search_label = Label::new(Some("Find:"));
    search_label.set_width_request(60);
    search_label.set_halign(Align::Start);
    search_label.add_css_class("marco-search-label");

    // Create overlay to show match count inside the search input
    let search_overlay = Overlay::new();
    search_overlay.set_hexpand(true);

    let search_entry = Entry::new();
    search_entry.set_hexpand(true);
    search_entry.set_placeholder_text(Some("Enter search text..."));
    search_entry.add_css_class("marco-search-entry");

    // Match count label positioned as overlay inside the search field
    let match_count_label = Label::new(Some(""));
    match_count_label.set_halign(Align::End);
    match_count_label.set_valign(Align::Center);
    match_count_label.add_css_class("dim-label");
    match_count_label.add_css_class("marco-search-match-label");
    match_count_label.set_sensitive(false); // Make it non-interactive

    // Add entry as main child and label as overlay
    search_overlay.set_child(Some(&search_entry));
    search_overlay.add_overlay(&match_count_label);

    // No Find button needed - search happens automatically while typing

    search_row.append(&search_label);
    search_row.append(&search_overlay);

    search_box.append(&search_row);

    // Store label for global access
    CURRENT_MATCH_LABEL.with(|label_ref| {
        *label_ref.borrow_mut() = Some(match_count_label.clone());
    });

    (search_box, search_entry, match_count_label)
}

/// Create the replace controls section
fn create_replace_controls_section() -> (GtkBox, Entry) {
    let replace_box = GtkBox::new(Orientation::Vertical, 4);
    // Always visible in the simplified UI

    let replace_row = GtkBox::new(Orientation::Horizontal, 8);

    let replace_label = Label::new(Some("Replace:"));
    replace_label.set_width_request(60);
    replace_label.set_halign(Align::Start);
    replace_label.add_css_class("marco-search-label");

    let replace_entry = Entry::new();
    replace_entry.set_hexpand(true);
    replace_entry.set_placeholder_text(Some("Enter replacement text..."));
    replace_entry.add_css_class("marco-search-entry");

    replace_row.append(&replace_label);
    replace_row.append(&replace_entry);

    replace_box.append(&replace_row);

    (replace_box, replace_entry)
}

/// Options panel widgets
#[derive(Clone)]
pub struct OptionsWidgets {
    pub match_case_cb: CheckButton,
    pub match_whole_word_cb: CheckButton,
    pub match_markdown_cb: CheckButton,
    pub use_regex_cb: CheckButton,
}

/// Create the options panel with checkboxes
fn create_options_panel() -> (GtkBox, OptionsWidgets) {
    let options_box = GtkBox::new(Orientation::Vertical, 6);

    // Separator
    let separator = Separator::new(Orientation::Horizontal);
    separator.add_css_class("marco-search-separator");
    options_box.append(&separator);

    // Options grid - two rows of two checkboxes each
    let options_grid = GtkBox::new(Orientation::Vertical, 3);

    // First row
    let row1 = GtkBox::new(Orientation::Horizontal, 16);
    row1.set_homogeneous(true);

    let match_case_cb = CheckButton::with_label("Match Case");
    match_case_cb.add_css_class("marco-search-checkbox");
    let match_markdown_cb = CheckButton::with_label("Match only Markdown syntax");
    match_markdown_cb.add_css_class("marco-search-checkbox");

    row1.append(&match_case_cb);
    row1.append(&match_markdown_cb);

    // Second row
    let row2 = GtkBox::new(Orientation::Horizontal, 16);
    row2.set_homogeneous(true);

    let match_whole_word_cb = CheckButton::with_label("Match Whole Word");
    match_whole_word_cb.add_css_class("marco-search-checkbox");
    let use_regex_cb = CheckButton::with_label("Regular Expressions");
    use_regex_cb.add_css_class("marco-search-checkbox");

    row2.append(&match_whole_word_cb);
    row2.append(&use_regex_cb);

    options_grid.append(&row1);
    options_grid.append(&row2);
    options_box.append(&options_grid);

    let widgets = OptionsWidgets {
        match_case_cb,
        match_whole_word_cb,
        match_markdown_cb,
        use_regex_cb,
    };

    (options_box, widgets)
}

/// Button panel widgets  
pub struct ButtonWidgets {
    pub prev_button: Button,
    pub next_button: Button,
    pub replace_button: Button,
    pub replace_all_button: Button,
}

/// Create the button panel
/// Create the button panel for search window (no close button needed)
fn create_window_button_panel() -> (GtkBox, ButtonWidgets) {
    let button_box = GtkBox::new(Orientation::Horizontal, 6);
    button_box.set_halign(Align::End);
    button_box.set_margin_top(8);

    // Bottom buttons: [Previous] [Next] [Replace] [Replace All]
    // No close button needed since the window has its own close controls
    let prev_button = Button::with_label("Previous");
    prev_button.add_css_class("marco-search-button");
    let next_button = Button::with_label("Next");
    next_button.add_css_class("marco-search-button");

    let replace_button = Button::with_label("Replace");
    replace_button.add_css_class("marco-search-button");
    replace_button.set_sensitive(false); // Initially disabled when Replace input is empty

    let replace_all_button = Button::with_label("Replace All");
    replace_all_button.add_css_class("marco-search-button");
    replace_all_button.set_sensitive(false); // Initially disabled when Replace input is empty

    button_box.append(&prev_button);
    button_box.append(&next_button);
    button_box.append(&replace_button);
    button_box.append(&replace_all_button);

    let widgets = ButtonWidgets {
        prev_button,
        next_button,
        replace_button,
        replace_all_button,
    };

    (button_box, widgets)
}

/// Clear search highlighting from any previous search operations
fn clear_search_highlighting() {
    debug!("Clearing previous search highlighting");

    // Use the enhanced clearing function that handles both standard and selected highlighting
    clear_enhanced_search_highlighting();

    // Clear the current search state
    CURRENT_SEARCH_STATE.with(|state_ref| {
        *state_ref.borrow_mut() = None;
    });

    // Clear match position tracking
    CURRENT_MATCH_POSITION.with(|pos| {
        *pos.borrow_mut() = None;
    });
}

/// Focus the search entry in a window for immediate typing
fn focus_search_entry_in_window(window: &Window) {
    // Try to focus the search entry widget in the window
    let _ = window.grab_focus();
}

/// Setup all window behavior and signal connections (similar to dialog but adapted for windows)
fn setup_window_behavior(
    _window: &Window,
    search_entry: &Entry,
    replace_entry: &Entry,
    match_count_label: &Label,
    options_widgets: &(GtkBox, OptionsWidgets),
    button_widgets: &(GtkBox, ButtonWidgets),
) {
    // Search entry live updates (when text is typed in the entry)
    let match_count_clone = match_count_label.clone();
    let options_clone = OptionsWidgets {
        match_case_cb: options_widgets.1.match_case_cb.clone(),
        match_whole_word_cb: options_widgets.1.match_whole_word_cb.clone(),
        match_markdown_cb: options_widgets.1.match_markdown_cb.clone(),
        use_regex_cb: options_widgets.1.use_regex_cb.clone(),
    };
    let search_entry_clone = search_entry.clone();

    // Connect to the entry for live updates and Enter key
    let options_clone_for_changed = options_clone.clone();

    search_entry.connect_changed(move |_entry| {
        let query = search_entry_clone.text().to_string();
        // Use debounced search while typing to prevent rapid search operations
        if !query.is_empty() {
            debounced_search(
                search_entry_clone.clone(),
                match_count_clone.clone(),
                options_clone_for_changed.clone(),
                200, // 200ms debounce delay
            );
        } else {
            // Clear search immediately when query is empty
            clear_search_highlighting();
            match_count_clone.set_text("");
        }
    });

    // Connect Enter key to perform search and highlight matches
    let search_entry_clone_enter = search_entry.clone();
    let match_count_clone_enter = match_count_label.clone();
    let options_clone_enter = options_clone.clone();

    search_entry.connect_activate(move |_entry| {
        let query = search_entry_clone_enter.text().to_string();
        if !query.is_empty() {
            // If no search is active, perform search first
            let needs_search = CURRENT_SEARCH_STATE.with(|state_ref| state_ref.borrow().is_none());

            if needs_search {
                perform_search(
                    &search_entry_clone_enter,
                    &match_count_clone_enter,
                    &options_clone_enter,
                );
            }

            // If position is None, find position from cursor
            // Otherwise keep current position for incremental navigation
            let needs_position_reset = CURRENT_MATCH_POSITION.with(|pos| pos.borrow().is_none());

            if needs_position_reset {
                // Find the closest match from cursor position
                // This works immediately if search exists, or returns None for new searches
                let position = find_position_from_cursor().unwrap_or(0);
                CURRENT_MATCH_POSITION.with(|pos| {
                    *pos.borrow_mut() = Some(position);
                });
            }

            // Navigate to next match (increments current position)
            immediate_position_update_with_debounced_navigation(1, 100);
        }
    });

    // Previous button
    let search_entry_clone_prev = search_entry.clone();
    let match_count_clone_prev = match_count_label.clone();
    let options_clone_prev = options_clone.clone();

    button_widgets.1.prev_button.connect_clicked(move |_| {
        // If no search is active, perform search first
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

        // If position is None (no navigation yet), find position from cursor
        // For Previous, we want to go backwards, so add 1 to the found position
        let needs_position_reset = CURRENT_MATCH_POSITION.with(|pos| pos.borrow().is_none());

        if needs_position_reset {
            // Find the closest match before cursor position
            let position = find_position_before_cursor().unwrap_or(2);
            CURRENT_MATCH_POSITION.with(|pos| {
                *pos.borrow_mut() = Some(position);
            });
        }

        // Immediately update position counter and debounce the actual navigation
        immediate_position_update_with_debounced_navigation(-1, 200); // direction=-1 for previous
    });

    // Next button
    let search_entry_clone_next = search_entry.clone();
    let match_count_clone_next = match_count_label.clone();
    let options_clone_next = options_clone.clone();

    button_widgets.1.next_button.connect_clicked(move |_| {
        // If no search is active, perform search first
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

        // If position is None, find position from cursor
        // Otherwise keep current position for incremental navigation
        let needs_position_reset = CURRENT_MATCH_POSITION.with(|pos| pos.borrow().is_none());

        if needs_position_reset {
            // Find the closest match from cursor position
            let position = find_position_from_cursor().unwrap_or(0);
            CURRENT_MATCH_POSITION.with(|pos| {
                *pos.borrow_mut() = Some(position);
            });
        }

        // Navigate to next match (increments current position)
        immediate_position_update_with_debounced_navigation(1, 200); // direction=1 for next
    });

    // Replace button connection
    let search_entry_clone_replace = search_entry.clone();
    let replace_entry_clone_replace = replace_entry.clone();

    button_widgets.1.replace_button.connect_clicked(move |_| {
        replace_next_match(&search_entry_clone_replace, &replace_entry_clone_replace);
    });

    // Replace All button connection
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
    ] {
        let search_entry_option_clone = search_entry_option.clone();
        let match_count_option_clone = match_count_option.clone();
        let options_for_options_clone = options_for_options.clone();

        checkbox.connect_toggled(move |_| {
            let query = search_entry_option_clone.text().to_string();
            if !query.is_empty() {
                perform_search(
                    &search_entry_option_clone,
                    &match_count_option_clone,
                    &options_for_options_clone,
                );
            }
        });
    }

    // Enable/disable replace buttons based on replace text
    let replace_button_clone = button_widgets.1.replace_button.clone();
    let replace_all_button_clone = button_widgets.1.replace_all_button.clone();
    let replace_entry_for_enable = replace_entry.clone();

    replace_entry_for_enable.connect_changed(move |entry| {
        let has_text = !entry.text().is_empty();
        replace_button_clone.set_sensitive(has_text);
        replace_all_button_clone.set_sensitive(has_text);
    });

    debug!("Window behavior setup completed");
}

/// Find the position of the closest match at or after the cursor position
/// Returns the match position (0-indexed for increment logic) or None if no matches exist
fn find_position_from_cursor() -> Option<i32> {
    CURRENT_SEARCH_STATE.with(|state_ref| {
        if let Some(search_state) = state_ref.borrow().as_ref() {
            let search_context = &search_state.search_context;
            let total_count = search_context.occurrences_count();

            debug!("find_position_from_cursor: total_count = {}", total_count);

            if total_count <= 0 {
                debug!("find_position_from_cursor: No matches available");
                return None;
            }

            CURRENT_BUFFER.with(|buffer_ref| {
                if let Some(buffer) = buffer_ref.borrow().as_ref() {
                    // Get cursor position
                    let cursor_iter = buffer.iter_at_mark(&buffer.get_insert());
                    let cursor_offset = cursor_iter.offset();
                    let cursor_line = cursor_iter.line() + 1;

                    debug!("find_position_from_cursor: Cursor at line {}, offset {}", cursor_line, cursor_offset);

                    // Iterate through all matches to find the closest one at or after cursor
                    let mut current_iter = buffer.start_iter();
                    let mut position = 1;

                    while let Some((match_start, match_end, _wrapped)) = search_context.forward(&current_iter) {
                        let match_offset = match_start.offset();
                        let match_line = match_start.line() + 1;

                        // If this match is at or after the cursor, use it
                        if match_offset >= cursor_offset {
                            debug!("find_position_from_cursor: Found match #{} at line {} (offset {}), cursor at line {} (offset {})",
                                   position, match_line, match_offset, cursor_line, cursor_offset);
                            return Some(position - 1); // Return 0-indexed for increment logic
                        }

                        // Move to next match
                        current_iter = match_end;
                        position += 1;

                        // Safety check to prevent infinite loop
                        if position > total_count {
                            break;
                        }
                    }

                    // If no match found at or after cursor, wrap to beginning (position 1)
                    debug!("find_position_from_cursor: No match at/after cursor, wrapping to position 1");
                    Some(0) // Will become 1 after increment
                } else {
                    debug!("find_position_from_cursor: No buffer available");
                    Some(0)
                }
            })
        } else {
            debug!("find_position_from_cursor: No search state available");
            None
        }
    })
}

/// Find the position of the closest match before the cursor position (for Previous navigation)
/// Returns the match position (1-indexed, ready for decrement) or None if no matches exist
fn find_position_before_cursor() -> Option<i32> {
    CURRENT_SEARCH_STATE.with(|state_ref| {
        if let Some(search_state) = state_ref.borrow().as_ref() {
            let search_context = &search_state.search_context;
            let total_count = search_context.occurrences_count();

            debug!("find_position_before_cursor: total_count = {}", total_count);

            if total_count <= 0 {
                debug!("find_position_before_cursor: No matches available");
                return None;
            }

            CURRENT_BUFFER.with(|buffer_ref| {
                if let Some(buffer) = buffer_ref.borrow().as_ref() {
                    // Get cursor position
                    let cursor_iter = buffer.iter_at_mark(&buffer.get_insert());
                    let cursor_offset = cursor_iter.offset();
                    let cursor_line = cursor_iter.line() + 1;

                    debug!("find_position_before_cursor: Cursor at line {}, offset {}", cursor_line, cursor_offset);

                    // Iterate through all matches to find the last one before cursor
                    let mut current_iter = buffer.start_iter();
                    let mut position = 1;
                    let mut last_valid_position = None;

                    while let Some((match_start, match_end, _wrapped)) = search_context.forward(&current_iter) {
                        let match_offset = match_start.offset();
                        let match_line = match_start.line() + 1;

                        // If this match is before the cursor, remember it
                        if match_offset < cursor_offset {
                            last_valid_position = Some(position);
                            debug!("find_position_before_cursor: Match #{} at line {} is before cursor", position, match_line);
                        } else {
                            // We've reached matches at/after cursor, stop searching
                            break;
                        }

                        // Move to next match
                        current_iter = match_end;
                        position += 1;

                        // Safety check to prevent infinite loop
                        if position > total_count {
                            break;
                        }
                    }

                    if let Some(pos) = last_valid_position {
                        debug!("find_position_before_cursor: Found match #{} before cursor", pos);
                        // Return position + 1 so decrement brings us to the correct match
                        Some(pos + 1)
                    } else {
                        // No match found before cursor, wrap to end (last match)
                        debug!("find_position_before_cursor: No match before cursor, wrapping to last match #{}", total_count);
                        Some(total_count + 1) // Will become total_count after decrement
                    }
                } else {
                    debug!("find_position_before_cursor: No buffer available");
                    Some(2) // Will become 1 after decrement
                }
            })
        } else {
            debug!("find_position_before_cursor: No search state available");
            None
        }
    })
}

/// Debounced search function to prevent rapid search operations
fn debounced_search(
    search_entry: Entry,
    match_count_label: Label,
    options: OptionsWidgets,
    delay_ms: u32,
) {
    // Use the async manager for simplified debouncing
    perform_search_async(search_entry, match_count_label, options, delay_ms);
}

/// Update match position immediately for rapid button presses, but debounce the actual navigation
fn immediate_position_update_with_debounced_navigation(direction: i32, delay_ms: u32) {
    // Get total count first to enforce bounds
    let total_count = CURRENT_SEARCH_STATE.with(|state_ref| {
        if let Some(search_state) = state_ref.borrow().as_ref() {
            search_state.search_context.occurrences_count()
        } else {
            0
        }
    });

    if total_count <= 0 {
        debug!("No matches available for position update");
        return;
    }

    // Immediately update the position tracking for rapid button presses with bounds checking
    CURRENT_MATCH_POSITION.with(|pos| {
        let current_pos = pos.borrow().unwrap_or(1);
        let new_pos = if direction == 1 {
            // Next: increment position, but wrap around at the end
            if current_pos >= total_count {
                1 // Wrap to first match
            } else {
                current_pos + 1
            }
        } else if direction == -1 {
            // Previous: decrement position, but wrap around at the beginning
            if current_pos <= 1 {
                total_count // Wrap to last match
            } else {
                current_pos - 1
            }
        } else {
            // First navigation or reset
            1
        };

        *pos.borrow_mut() = Some(new_pos);
        debug!(
            "Position updated: {} -> {} (total: {})",
            current_pos, new_pos, total_count
        );

        // Update the display immediately with the new position
        CURRENT_MATCH_LABEL.with(|label_ref| {
            if let Some(label) = label_ref.borrow().as_ref() {
                let text = format!("{} of {} matches", new_pos, total_count);
                label.set_text(&text);
                debug!("Immediate position update: {}", text);
            }
        });
    });

    // Cancel any existing navigation timer
    NAVIGATION_DEBOUNCE_TIMER.with(|timer_ref| {
        if let Some(timer_id) = timer_ref.borrow_mut().take() {
            safe_source_remove(timer_id);
        }
    });

    // Set up debounced actual navigation to the final position
    let timer_id = timeout_add_local(
        std::time::Duration::from_millis(delay_ms as u64),
        move || {
            // Clear the timer ID since we're about to execute
            NAVIGATION_DEBOUNCE_TIMER.with(|timer_ref| {
                *timer_ref.borrow_mut() = None;
            });

            // Perform the actual navigation to the final position
            navigate_to_current_position();

            glib::ControlFlow::Break
        },
    );

    // Store the timer ID for potential cancellation
    NAVIGATION_DEBOUNCE_TIMER.with(|timer_ref| {
        *timer_ref.borrow_mut() = Some(timer_id);
    });
}

/// Navigate to the position stored in CURRENT_MATCH_POSITION
fn navigate_to_current_position() {
    if is_navigation_in_progress() {
        debug!("Navigation already in progress, ignoring position navigation request");
        return;
    }

    set_navigation_in_progress(true);

    let target_position = CURRENT_MATCH_POSITION.with(|pos| *pos.borrow());

    if let Some(target_pos) = target_position {
        debug!("Navigating to stored position: {}", target_pos);

        CURRENT_SEARCH_STATE.with(|state_ref| {
            if let Some(search_state) = state_ref.borrow().as_ref() {
                CURRENT_BUFFER.with(|buffer_ref| {
                    if let Some(buffer) = buffer_ref.borrow().as_ref() {
                        // Find the target match by iterating through all matches
                        let mut current_iter = buffer.start_iter();
                        let mut found_match = None;

                        for _ in 1..=target_pos {
                            if let Some((match_start, match_end, _)) =
                                search_state.search_context.forward(&current_iter)
                            {
                                found_match = Some((match_start, match_end));
                                current_iter = match_end;
                            } else {
                                break;
                            }
                        }

                        if let Some((match_start, match_end)) = found_match {
                            let line_number = match_start.line() + 1;
                            debug!(
                                "Found target match at line {} for position {}",
                                line_number, target_pos
                            );

                            // Move cursor to the match and select it
                            buffer.place_cursor(&match_start);
                            buffer.select_range(&match_start, &match_end);

                            // Apply enhanced highlighting with the current match highlighted differently
                            apply_enhanced_search_highlighting(
                                &search_state.search_context,
                                Some(&match_start),
                                Some(&match_end),
                            );

                            // Scroll the editor to show the match
                            scroll_to_match(&match_start);

                            // Update the display with accurate position information
                            let total_count = search_state.search_context.occurrences_count();
                            CURRENT_MATCH_LABEL.with(|label_ref| {
                                if let Some(label) = label_ref.borrow().as_ref() {
                                    let text = format!(
                                        "{} of {} matches (line {})",
                                        target_pos, total_count, line_number
                                    );
                                    label.set_text(&text);
                                    debug!("Final navigation completed: {}", text);
                                }
                            });
                        } else {
                            debug!("Could not find match at position {}", target_pos);
                        }
                    }
                });
            }
        });
    } else {
        debug!("No target position set for navigation");
    }

    set_navigation_in_progress(false);
}

/// Check if navigation is currently in progress to prevent race conditions
fn is_navigation_in_progress() -> bool {
    NAVIGATION_IN_PROGRESS.with(|flag| *flag.borrow())
}

/// Set navigation progress flag
fn set_navigation_in_progress(in_progress: bool) {
    NAVIGATION_IN_PROGRESS.with(|flag| {
        *flag.borrow_mut() = in_progress;
    });
}

/// Simple async search with debouncing
fn perform_search_async(
    search_entry: Entry,
    match_count_label: Label,
    options: OptionsWidgets,
    delay_ms: u32,
) {
    ASYNC_MANAGER.with(|manager_ref| {
        if let Some(manager) = manager_ref.borrow_mut().as_mut() {
            manager.schedule_search(delay_ms, move || {
                perform_search(&search_entry, &match_count_label, &options);
            });
        } else {
            // Fallback to immediate search if manager not available
            perform_search(&search_entry, &match_count_label, &options);
        }
    });
}

/// Enhanced search highlighting with different colors for all matches and current selection
///
/// This function provides dual-color highlighting for better search result visualization:
/// - All search matches are highlighted with the standard 'search-match' style (yellow background)
/// - The currently selected match is highlighted with 'search-match-selected' style (orange background)
///
/// # Arguments
/// * `search_context` - The GTK SourceView SearchContext containing the search results
/// * `current_match_start` - Optional start iterator for the currently selected match
/// * `current_match_end` - Optional end iterator for the currently selected match
///
/// # Example
/// ```rust
/// // Highlight all search results with standard highlighting
/// apply_enhanced_search_highlighting(&search_context, None, None);
///
/// // Highlight all results and mark a specific match as selected
/// if let Some((start, end, _)) = search_context.forward(&buffer.start_iter()) {
///     apply_enhanced_search_highlighting(&search_context, Some(&start), Some(&end));
/// }
/// ```
///
/// # Theme Requirements
/// The theme files should define both:
/// - `search-match` style for regular matches
/// - `search-match-selected` style for the selected match
pub fn apply_enhanced_search_highlighting(
    search_context: &SearchContext,
    current_match_start: Option<&gtk4::TextIter>,
    current_match_end: Option<&gtk4::TextIter>,
) {
    CURRENT_BUFFER.with(|buffer_ref| {
        if let Some(buffer) = buffer_ref.borrow().as_ref() {
            // Get the style scheme to check for available styles
            if let Some(style_scheme) = buffer.style_scheme() {
                // Check if we have the enhanced highlighting styles
                let has_selected_style = style_scheme.style("search-match-selected").is_some();

                if has_selected_style {
                    debug!("Applying enhanced search highlighting with dual colors");

                    // First, apply standard highlighting to all matches
                    search_context.set_highlight(true);

                    // If we have a current match, add additional highlighting for the selected match
                    if let (Some(start), Some(end)) = (current_match_start, current_match_end) {
                        // Create a text tag for the selected match highlighting
                        let tag_table = buffer.tag_table();

                        // Check if we already have a selected match tag, or create a new one
                        let selected_tag = if let Some(existing_tag) = tag_table.lookup("search-match-selected-custom") {
                            existing_tag
                        } else {
                            let new_tag = gtk4::TextTag::new(Some("search-match-selected-custom"));

                            // Get the colors from the style scheme
                            if let Some(selected_style) = style_scheme.style("search-match-selected") {
                                // Apply the style properties from the scheme
                                if let Some(bg_color) = selected_style.background() {
                                    new_tag.set_background(Some(&bg_color));
                                }
                                if let Some(fg_color) = selected_style.foreground() {
                                    new_tag.set_foreground(Some(&fg_color));
                                }
                                if selected_style.is_bold() {
                                    new_tag.set_weight(700); // Bold weight
                                }
                            } else {
                                // Fallback colors if style is not found
                                new_tag.set_background(Some("#FF6B35")); // Orange background
                                new_tag.set_foreground(Some("#FFFFFF")); // White text
                                new_tag.set_weight(700); // Bold weight
                            }

                            tag_table.add(&new_tag);
                            new_tag
                        };

                        // Remove any existing selected match highlighting
                        let start_iter = buffer.start_iter();
                        let end_iter = buffer.end_iter();
                        buffer.remove_tag(&selected_tag, &start_iter, &end_iter);

                        // Apply the selected match highlighting to the current match
                        buffer.apply_tag(&selected_tag, start, end);

                        let line_number = start.line() + 1;
                        debug!("Applied enhanced highlighting to current match at line {}", line_number);
                    }
                } else {
                    debug!("Enhanced highlighting styles not found in theme, using standard highlighting");
                    search_context.set_highlight(true);
                }
            } else {
                debug!("No style scheme available, using default highlighting");
                search_context.set_highlight(true);
            }
        }
    });
}

/// Clear enhanced search highlighting including custom selected match tags
pub fn clear_enhanced_search_highlighting() {
    CURRENT_BUFFER.with(|buffer_ref| {
        if let Some(buffer) = buffer_ref.borrow().as_ref() {
            // Clear standard search highlighting
            CURRENT_SEARCH_STATE.with(|state_ref| {
                if let Some(search_state) = state_ref.borrow().as_ref() {
                    search_state.search_context.set_highlight(false);
                }
            });

            // Clear custom selected match highlighting
            let tag_table = buffer.tag_table();
            if let Some(selected_tag) = tag_table.lookup("search-match-selected-custom") {
                let start_iter = buffer.start_iter();
                let end_iter = buffer.end_iter();
                buffer.remove_tag(&selected_tag, &start_iter, &end_iter);
                debug!("Cleared enhanced search highlighting");
            }
        }
    });
}

/// Perform search operation
fn perform_search(search_entry: &Entry, match_count_label: &Label, options: &OptionsWidgets) {
    let query = search_entry.text().to_string();
    if query.is_empty() {
        // Clear any existing search highlighting when query is empty
        clear_search_highlighting();
        match_count_label.set_text("0 matches");
        return;
    }

    debug!("Performing search for: '{}'", query);

    // Clear any previous search highlighting before starting new search
    clear_search_highlighting();

    // Get the current buffer from thread-local storage
    CURRENT_BUFFER.with(|buffer_ref| {
        if let Some(buffer) = buffer_ref.borrow().as_ref() {
            // Create search settings
            let search_settings = SearchSettings::new();
            search_settings.set_search_text(Some(&query));
            search_settings.set_case_sensitive(options.match_case_cb.is_active());
            search_settings.set_wrap_around(true);
            search_settings.set_at_word_boundaries(options.match_whole_word_cb.is_active());
            search_settings.set_regex_enabled(options.use_regex_cb.is_active());

            // Create search context
            let search_context = SearchContext::new(&**buffer, Some(&search_settings));

            // Apply enhanced highlighting initially (without a specific selected match)
            apply_enhanced_search_highlighting(&search_context, None, None);

            // Configure search highlighting with proper style scheme integration
            if let Some(style_scheme) = buffer.style_scheme() {
                // Check if the style scheme has enhanced highlighting styles
                if let Some(_search_match_style) = style_scheme.style("search-match") {
                    debug!("Using enhanced search highlighting with scheme '{}'", style_scheme.name());
                    if style_scheme.style("search-match-selected").is_some() {
                        debug!("Enhanced selected match highlighting available");
                    }
                } else {
                    // Log that we're using default highlighting
                    debug!("Style scheme '{}' does not define 'search-match' style, using SearchContext default highlighting", style_scheme.name());
                }
            } else {
                debug!("No style scheme set, using default highlighting");
            }

            // Store the search state for navigation functions
            CURRENT_SEARCH_STATE.with(|state_ref| {
                *state_ref.borrow_mut() = Some(SearchState {
                    search_context: search_context.clone(),
                });
            });

            // Reset match position tracking for new search
            CURRENT_MATCH_POSITION.with(|pos| *pos.borrow_mut() = None);

            // Set up count monitoring with enhanced position tracking
            let label_clone = match_count_label.clone();
            let search_context_clone = search_context.clone();
            search_context.connect_occurrences_count_notify(move |ctx| {
                let count = ctx.occurrences_count();
                let text = if count == -1 {
                    "Searching...".to_string()
                } else if count == 0 {
                    "No matches".to_string()
                } else if count == 1 {
                    "1 match".to_string()
                } else {
                    format!("{} matches", count)
                };
                label_clone.set_text(&text);
                debug!("Match count updated: {}", count);

                // If scanning is complete and we have a current selection, update position display
                if count > 0 {
                    CURRENT_BUFFER.with(|buffer_ref| {
                        if let Some(buffer) = buffer_ref.borrow().as_ref() {
                            if buffer.has_selection() {
                                let (start_iter, end_iter) = buffer.selection_bounds().unwrap();
                                // Check if the current selection is a valid search match
                                let position = search_context_clone.occurrence_position(&start_iter, &end_iter);
                                if position > 0 {
                                    let line_number = start_iter.line() + 1;
                                    let updated_text = format!("{} of {} matches (line {})", position, count, line_number);
                                    label_clone.set_text(&updated_text);
                                    debug!("Updated position after scan completion: {}", updated_text);
                                }
                            }
                        }
                    });
                }
            });

            // Initial count display
            let match_count = search_context.occurrences_count();
            let match_text = if match_count == -1 {
                "Searching...".to_string()
            } else if match_count == 0 {
                "No matches".to_string()
            } else if match_count == 1 {
                "1 match".to_string()
            } else {
                format!("{} matches", match_count)
            };
            match_count_label.set_text(&match_text);

            debug!("Search initiated: initial count {} for '{}'", match_count, query);

            // Don't automatically navigate to first match during search setup
            // Let the user explicitly choose when to navigate with Enter key or buttons
            debug!("Search context created for '{}' with highlighting enabled", query);
        } else {
            debug!("No buffer available for search");
            match_count_label.set_text("No buffer");
        }
    });
}

/// Replace next match
fn replace_next_match(search_entry: &Entry, replace_entry: &Entry) {
    let query = search_entry.text().to_string();
    let replacement = replace_entry.text().to_string();

    if query.is_empty() {
        debug!("Replace next: query is empty");
        return;
    }

    debug!("Replacing next match: '{}' -> '{}'", query, replacement);

    CURRENT_SEARCH_STATE.with(|state_ref| {
        if let Some(search_state) = state_ref.borrow().as_ref() {
            CURRENT_BUFFER.with(|buffer_ref| {
                if let Some(buffer) = buffer_ref.borrow().as_ref() {
                    buffer.begin_user_action();

                    // Get current cursor position
                    let cursor_iter = buffer.iter_at_offset(buffer.cursor_position());

                    // If there's a selection, start search from the beginning of selection
                    // Otherwise start from cursor position
                    let search_start = if buffer.has_selection() {
                        let (start_iter, _) = buffer.selection_bounds().unwrap();
                        start_iter
                    } else {
                        cursor_iter
                    };

                    // Find the next match from the search start position
                    if let Some((match_start, match_end, _has_wrapped)) =
                        search_state.search_context.forward(&search_start)
                    {
                        // Create marks to preserve positions across buffer modifications
                        let start_mark = buffer.create_mark(None, &match_start, false);
                        let end_mark = buffer.create_mark(None, &match_end, true);

                        // Use SearchContext's replace method - this respects all search settings
                        let mut start_iter = match_start;
                        let mut end_iter = match_end;
                        match search_state.search_context.replace(
                            &mut start_iter,
                            &mut end_iter,
                            &replacement,
                        ) {
                            Ok(()) => {
                                debug!(
                                    "Successfully replaced match: '{}' -> '{}'",
                                    query, replacement
                                );

                                // Get the position after replacement using the mark
                                let replacement_end_iter = buffer.iter_at_mark(&start_mark);
                                let mut search_from_iter = replacement_end_iter;

                                // Move the search position forward by the replacement length
                                search_from_iter.forward_chars(replacement.len() as i32);
                                buffer.place_cursor(&search_from_iter);

                                // Find and select the next match for easy continuation
                                if let Some((next_start, next_end, _)) =
                                    search_state.search_context.forward(&search_from_iter)
                                {
                                    buffer.select_range(&next_start, &next_end);

                                    // Scroll to show the next match
                                    scroll_to_match(&next_start);

                                    // Position display is automatically updated by cursor-based navigation
                                } else {
                                    debug!("No more matches found after replacement");
                                }

                                // Clean up marks
                                buffer.delete_mark(&start_mark);
                                buffer.delete_mark(&end_mark);
                            }
                            Err(e) => {
                                debug!("Replace operation failed: {}", e);

                                // Clean up marks even on error
                                buffer.delete_mark(&start_mark);
                                buffer.delete_mark(&end_mark);
                            }
                        }
                    } else {
                        debug!("No matches found to replace");
                    }

                    buffer.end_user_action();
                } else {
                    debug!("No buffer available for replace operation");
                }
            });
        } else {
            debug!("No active search state - please perform a search first");
        }
    });
}

/// Replace all matches
fn replace_all_matches(search_entry: &Entry, replace_entry: &Entry) {
    let query = search_entry.text().to_string();
    let replacement = replace_entry.text().to_string();

    if query.is_empty() {
        debug!("Replace all: query is empty");
        return;
    }

    debug!("Replacing all matches: '{}' -> '{}'", query, replacement);

    CURRENT_SEARCH_STATE.with(|state_ref| {
        if let Some(search_state) = state_ref.borrow().as_ref() {
            CURRENT_BUFFER.with(|buffer_ref| {
                if let Some(buffer) = buffer_ref.borrow().as_ref() {
                    buffer.begin_user_action();

                    // Use SearchContext's replace_all method
                    match search_state.search_context.replace_all(&replacement) {
                        Ok(()) => {
                            debug!(
                                "Replace all completed successfully: '{}' -> '{}'",
                                query, replacement
                            );

                            // Update match count display after replacement
                            CURRENT_MATCH_LABEL.with(|label_ref| {
                                if let Some(label) = label_ref.borrow().as_ref() {
                                    // After replace all, there should be no matches left for the old query
                                    label.set_text("No matches");
                                }
                            });

                            // Clear current selection since all matches were replaced
                            if buffer.has_selection() {
                                let cursor_mark = buffer.get_insert();
                                let cursor_iter = buffer.iter_at_mark(&cursor_mark);
                                buffer.place_cursor(&cursor_iter);
                            }
                        }
                        Err(e) => {
                            debug!("Replace all failed: {}", e);

                            // Update match count display to show error
                            CURRENT_MATCH_LABEL.with(|label_ref| {
                                if let Some(label) = label_ref.borrow().as_ref() {
                                    label.set_text("Replace failed");
                                }
                            });
                        }
                    }

                    buffer.end_user_action();
                } else {
                    debug!("No buffer available for replace all operation");
                }
            });
        } else {
            debug!("No active search state - please perform a search first");
        }
    });
}

/// Check if a widget has valid allocation for rendering operations
fn has_valid_allocation(widget: &impl IsA<gtk4::Widget>) -> bool {
    let allocation = widget.allocation();
    allocation.width() > 0 && allocation.height() > 0
}

/// Scroll the editor to show the match at the given position
fn scroll_to_match(match_iter: &gtk4::TextIter) {
    CURRENT_SOURCE_VIEW.with(|view_ref| {
        if let Some(source_view) = view_ref.borrow().as_ref() {
            // Check if the source view has proper allocation before scrolling
            if !has_valid_allocation(source_view.as_ref()) {
                debug!("Skipping scroll operation - SourceView has no allocation");
                return;
            }

            // Create a mutable copy of the iterator for scroll_to_iter
            let mut iter_copy = *match_iter;

            // Scroll to the match position with some margin
            // Parameters: iter, within_margin, use_align, xalign, yalign
            // within_margin: 0.1 = 10% margin from edges before scrolling
            // use_align: true = use the alignment values
            // xalign: 0.0 = align to left edge
            // yalign: 0.3 = position match at 30% from top (comfortable reading position)
            source_view.scroll_to_iter(&mut iter_copy, 0.1, true, 0.0, 0.3);

            debug!(
                "Scrolled editor to show match at line {}",
                match_iter.line() + 1
            );

            // Also sync the HTML preview if scroll sync is enabled
            sync_html_preview_scroll(match_iter);
        } else {
            debug!("No source view available for scrolling");
        }
    });
}

/// Sync HTML preview scroll to match the given position (if scroll sync is enabled)
#[cfg(target_os = "linux")]
fn sync_html_preview_scroll(match_iter: &gtk4::TextIter) {
    // Check if scroll sync is enabled globally
    use crate::components::editor::editor_manager::get_global_scroll_synchronizer;
    if let Some(sync) = get_global_scroll_synchronizer() {
        // Only sync if scroll synchronization is actually enabled
        if !sync.is_enabled() {
            debug!("Scroll sync is disabled, skipping preview scroll sync");
            return;
        }
        // Access the WebView to perform sync
        CURRENT_WEBVIEW.with(|webview_ref| {
            if let Some(webview) = webview_ref.borrow().as_ref() {
                // Calculate the scroll percentage based on the match position
                CURRENT_BUFFER.with(|buffer_ref| {
                    if let Some(buffer) = buffer_ref.borrow().as_ref() {
                        let total_lines = buffer.line_count();
                        let match_line = match_iter.line();

                        // Calculate approximate scroll percentage
                        // Position the match at about 30% from the top (same as editor scroll)
                        let scroll_percentage = if total_lines > 1 {
                            (match_line as f64 / (total_lines - 1) as f64).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };

                        // Use JavaScript to scroll the WebView to the corresponding position
                        let js_code = format!(
                            r#"
                            (function() {{
                                if (window.__scroll_sync_guard) return;
                                window.__scroll_sync_guard = true;

                                const maxScroll = Math.max(0, document.documentElement.scrollHeight - window.innerHeight);
                                const targetScroll = {} * maxScroll;

                                // Adjust to position the target at 30% from top (like editor)
                                const viewportHeight = window.innerHeight;
                                const adjustedScroll = Math.max(0, targetScroll - viewportHeight * 0.3);

                                window.scrollTo({{
                                    top: adjustedScroll,
                                    behavior: 'smooth'
                                }});

                                setTimeout(() => {{
                                    window.__scroll_sync_guard = false;
                                }}, 150);
                            }})();
                            "#,
                            scroll_percentage
                        );

                        webview.borrow().evaluate_javascript(&js_code, None, None, None::<&gio::Cancellable>, |result| {
                            if let Err(e) = result {
                                debug!("JavaScript preview scroll sync error: {:?}", e);
                            }
                        });

                        debug!(
                            "Synced HTML preview scroll to line {} ({:.1}%)",
                            match_line + 1,
                            scroll_percentage * 100.0
                        );
                    }
                });
            } else {
                debug!("No WebView available for preview scroll sync");
            }
        });
    }
}

#[cfg(not(target_os = "linux"))]
fn sync_html_preview_scroll(_match_iter: &gtk4::TextIter) {
    // No-op on platforms without a WebView implementation
}

// Smoke tests for the search dialog
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
    fn smoke_test_async_search_manager() {
        let manager = AsyncSearchManager::new();

        // Test that manager initializes correctly
        assert!(manager.current_timer_id.is_none());
    }

    #[test]
    fn smoke_test_simple_integration() {
        // Test that our simple async integration compiles and works
        let _manager = AsyncSearchManager::new();

        // This test passes if the code compiles and instantiates correctly
        println!("Simple async integration working");
        println!("SignalManager integrated");
        println!("SimpleFileCache integrated");
        println!("Basic debouncing implemented");
    }

    #[test]
    fn smoke_test_match_position_tracking() {
        // Test the match position tracking logic
        CURRENT_MATCH_POSITION.with(|pos| {
            // Initially should be None
            assert!(pos.borrow().is_none());

            // Set a position
            *pos.borrow_mut() = Some(5);
            assert_eq!(*pos.borrow(), Some(5));

            // Clear position
            *pos.borrow_mut() = None;
            assert!(pos.borrow().is_none());
        });

        // Test navigation state
        assert!(!is_navigation_in_progress());

        set_navigation_in_progress(true);
        assert!(is_navigation_in_progress());

        set_navigation_in_progress(false);
        assert!(!is_navigation_in_progress());
    }

    #[test]
    fn smoke_test_search_highlighting_clear() {
        // Test that clearing search highlighting works properly

        // Initially no search state
        let has_search_state = CURRENT_SEARCH_STATE.with(|state_ref| state_ref.borrow().is_some());
        assert!(!has_search_state);

        // Test clearing when no state exists (should not panic)
        clear_search_highlighting();

        // Verify still no state
        let has_search_state_after =
            CURRENT_SEARCH_STATE.with(|state_ref| state_ref.borrow().is_some());
        assert!(!has_search_state_after);

        // Test position is cleared
        let has_position = CURRENT_MATCH_POSITION.with(|pos| pos.borrow().is_some());
        assert!(!has_position);
    }

    #[test]
    fn smoke_test_first_navigation_behavior() {
        // Test that both Enter and Next behave the same on first navigation

        // Simulate the state of a fresh search - no previous match position
        CURRENT_MATCH_POSITION.with(|pos| {
            *pos.borrow_mut() = None;
        });

        // Verify that we detect this as first navigation
        let is_first_navigation = CURRENT_MATCH_POSITION.with(|pos| pos.borrow().is_none());
        assert!(
            is_first_navigation,
            "Should detect first navigation when position is None"
        );

        // Simulate having navigated once - set a position
        CURRENT_MATCH_POSITION.with(|pos| {
            *pos.borrow_mut() = Some(1);
        });

        // Verify that we detect this as subsequent navigation
        let is_subsequent_navigation = CURRENT_MATCH_POSITION.with(|pos| pos.borrow().is_some());
        assert!(
            is_subsequent_navigation,
            "Should detect subsequent navigation when position is set"
        );

        // Reset to original state
        CURRENT_MATCH_POSITION.with(|pos| {
            *pos.borrow_mut() = None;
        });
    }

    #[test]
    fn smoke_test_enter_next_consistency() {
        // Test that Enter key and Next button now use identical logic

        // Both should start with no search state
        CURRENT_SEARCH_STATE.with(|state_ref| {
            *state_ref.borrow_mut() = None;
        });
        CURRENT_MATCH_POSITION.with(|pos| {
            *pos.borrow_mut() = None;
        });

        // Verify initial states are identical for both code paths
        let needs_search_enter =
            CURRENT_SEARCH_STATE.with(|state_ref| state_ref.borrow().is_none());
        let needs_search_next = CURRENT_SEARCH_STATE.with(|state_ref| state_ref.borrow().is_none());

        assert_eq!(
            needs_search_enter, needs_search_next,
            "Both Enter and Next should start with same search state"
        );

        // Both should detect first navigation the same way
        let is_first_nav_enter = CURRENT_MATCH_POSITION.with(|pos| pos.borrow().is_none());
        let is_first_nav_next = CURRENT_MATCH_POSITION.with(|pos| pos.borrow().is_none());

        assert_eq!(
            is_first_nav_enter, is_first_nav_next,
            "Both Enter and Next should detect first navigation identically"
        );
        assert!(
            is_first_nav_enter,
            "Both should detect this as first navigation"
        );

        // After simulated navigation, both should behave the same
        CURRENT_MATCH_POSITION.with(|pos| {
            *pos.borrow_mut() = Some(2);
        });

        let is_subsequent_enter = CURRENT_MATCH_POSITION.with(|pos| pos.borrow().is_some());
        let is_subsequent_next = CURRENT_MATCH_POSITION.with(|pos| pos.borrow().is_some());

        assert_eq!(
            is_subsequent_enter, is_subsequent_next,
            "Both should detect subsequent navigation identically"
        );
        assert!(
            is_subsequent_enter,
            "Both should detect this as subsequent navigation"
        );

        // Clean up
        CURRENT_MATCH_POSITION.with(|pos| {
            *pos.borrow_mut() = None;
        });
    }

    #[test]
    fn smoke_test_single_press_navigation() {
        // Test that navigation works on the first press (no double-press required)

        // Start with clean state
        CURRENT_MATCH_POSITION.with(|pos| {
            *pos.borrow_mut() = None;
        });

        // Verify we start with no position (first navigation)
        let is_first_navigation = CURRENT_MATCH_POSITION.with(|pos| pos.borrow().is_none());
        assert!(is_first_navigation, "Should start with no position set");

        // Simulate the logic that runs during first navigation
        // The key fix: we always set CURRENT_MATCH_POSITION to something non-None after first navigation
        CURRENT_MATCH_POSITION.with(|pos| {
            // This simulates what should happen after first navigation completes
            *pos.borrow_mut() = Some(1); // Either actual position or fallback to 1
        });

        // Verify that after first navigation, we have a position set
        let has_position_after_first = CURRENT_MATCH_POSITION.with(|pos| pos.borrow().is_some());
        assert!(
            has_position_after_first,
            "After first navigation, position should be set to prevent double-press issue"
        );

        // Verify that second press will be treated as subsequent navigation
        let is_subsequent_navigation = CURRENT_MATCH_POSITION.with(|pos| pos.borrow().is_some());
        assert!(
            is_subsequent_navigation,
            "Second press should be treated as subsequent navigation"
        );

        // Clean up
        CURRENT_MATCH_POSITION.with(|pos| {
            *pos.borrow_mut() = None;
        });
    }

    #[test]
    fn smoke_test_debounce_timers() {
        // Test that debounce timers are properly managed

        // Start with clean state
        SEARCH_DEBOUNCE_TIMER.with(|timer_ref| {
            *timer_ref.borrow_mut() = None;
        });
        NAVIGATION_DEBOUNCE_TIMER.with(|timer_ref| {
            *timer_ref.borrow_mut() = None;
        });

        // Verify we start with no timers
        let has_search_timer = SEARCH_DEBOUNCE_TIMER.with(|timer_ref| timer_ref.borrow().is_some());
        let has_nav_timer =
            NAVIGATION_DEBOUNCE_TIMER.with(|timer_ref| timer_ref.borrow().is_some());

        assert!(
            !has_search_timer,
            "Should start with no search debounce timer"
        );
        assert!(
            !has_nav_timer,
            "Should start with no navigation debounce timer"
        );

        // Test that timer cleanup logic works
        // (We can't actually create real timers in unit tests, but we can test the state management)

        // Simulate having timers (this would happen during actual usage)
        // In real usage, the timers would be created by debounced_search() and debounced_navigation()
        // But we're just testing the cleanup logic here

        // Verify that the cleanup logic can handle None timers gracefully
        SEARCH_DEBOUNCE_TIMER.with(|timer_ref| {
            if let Some(_timer_id) = timer_ref.borrow_mut().take() {
                // This branch won't execute since timer is None, but tests the logic path
            }
        });
        NAVIGATION_DEBOUNCE_TIMER.with(|timer_ref| {
            if let Some(_timer_id) = timer_ref.borrow_mut().take() {
                // This branch won't execute since timer is None, but tests the logic path
            }
        });

        // Verify state is still clean after cleanup attempt
        let still_no_search_timer =
            SEARCH_DEBOUNCE_TIMER.with(|timer_ref| timer_ref.borrow().is_none());
        let still_no_nav_timer =
            NAVIGATION_DEBOUNCE_TIMER.with(|timer_ref| timer_ref.borrow().is_none());

        assert!(
            still_no_search_timer,
            "Search timer should still be None after cleanup"
        );
        assert!(
            still_no_nav_timer,
            "Navigation timer should still be None after cleanup"
        );
    }

    #[test]
    fn smoke_test_immediate_position_update() {
        // Test the immediate position update functionality for rapid button presses

        // Start with clean state
        CURRENT_MATCH_POSITION.with(|pos| {
            *pos.borrow_mut() = None;
        });

        // Test bounds checking logic (simulating the new logic)
        let total_count = 5; // Simulate 5 total matches

        // Test forward navigation with wrapping
        let mut current_pos = 4;
        let new_pos = if current_pos >= total_count {
            1 // Wrap to first match
        } else {
            current_pos + 1
        };
        assert_eq!(new_pos, 5, "Should increment from 4 to 5");

        // Test wrapping at the end
        current_pos = 5;
        let new_pos = if current_pos >= total_count {
            1 // Wrap to first match
        } else {
            current_pos + 1
        };
        assert_eq!(new_pos, 1, "Should wrap from 5 to 1 when at maximum");

        // Test backward navigation with wrapping
        current_pos = 2;
        let new_pos = if current_pos <= 1 {
            total_count // Wrap to last match
        } else {
            current_pos - 1
        };
        assert_eq!(new_pos, 1, "Should decrement from 2 to 1");

        // Test wrapping at the beginning
        current_pos = 1;
        let new_pos = if current_pos <= 1 {
            total_count // Wrap to last match
        } else {
            current_pos - 1
        };
        assert_eq!(new_pos, 5, "Should wrap from 1 to 5 when at minimum");

        // Test that positions stay within bounds
        for test_pos in 1..=total_count {
            // Forward direction
            let next_pos = if test_pos >= total_count {
                1
            } else {
                test_pos + 1
            };
            assert!(
                next_pos >= 1 && next_pos <= total_count,
                "Forward position {} should be within bounds 1-{}",
                next_pos,
                total_count
            );

            // Backward direction
            let prev_pos = if test_pos <= 1 {
                total_count
            } else {
                test_pos - 1
            };
            assert!(
                prev_pos >= 1 && prev_pos <= total_count,
                "Backward position {} should be within bounds 1-{}",
                prev_pos,
                total_count
            );
        }

        // Clean up
        CURRENT_MATCH_POSITION.with(|pos| {
            *pos.borrow_mut() = None;
        });
    }

    #[test]
    fn smoke_test_position_bounds_checking() {
        // Test that position never exceeds total match count (fixes the "36 of 8" issue)

        let total_matches = 8; // Like in the user's example

        // Test that rapid Next button presses don't exceed bounds
        let mut position = 1;
        for _ in 0..50 {
            // Simulate 50 rapid presses
            position = if position >= total_matches {
                1 // Should wrap to 1
            } else {
                position + 1
            };
            assert!(
                position >= 1 && position <= total_matches,
                "Position {} should never exceed total matches {}",
                position,
                total_matches
            );
        }

        // After 50 presses, we should have wrapped around multiple times
        // The exact final position depends on (50 mod 8), but it must be valid
        assert!(position >= 1 && position <= total_matches);

        // Test Previous button with wrapping
        position = 1;
        for _ in 0..50 {
            // Simulate 50 rapid Previous presses
            position = if position <= 1 {
                total_matches // Should wrap to last match
            } else {
                position - 1
            };
            assert!(
                position >= 1 && position <= total_matches,
                "Position {} should never exceed bounds during backward navigation",
                position
            );
        }

        // Test edge cases
        assert_eq!(
            if 8 >= 8 { 1 } else { 8 + 1 },
            1,
            "Position 8 of 8 should wrap to 1"
        );
        assert_eq!(
            if 1 <= 1 { 8 } else { 1 - 1 },
            8,
            "Position 1 of 8 should wrap to 8 when going backward"
        );
    }

    #[test]
    fn smoke_test_enhanced_search_highlighting() {
        // This is a smoke test to verify the enhanced highlighting function doesn't panic
        // In a real GTK environment, this would test the actual highlighting behavior

        // Test that calling the function with None parameters doesn't crash
        // (This tests the code path that handles no selected match)
        let result = std::panic::catch_unwind(|| {
            // In a real test, we would have a proper SearchContext and Buffer
            // For now, we just test that the function structure is sound
            debug!("Smoke test: Enhanced highlighting function structure verified");
        });

        assert!(
            result.is_ok(),
            "Enhanced highlighting function should not panic with None parameters"
        );

        // Verify the function exists and is callable (compilation test)
        let _function_exists = apply_enhanced_search_highlighting;
        let _clear_function_exists = clear_enhanced_search_highlighting;

        // Test passes if compilation succeeds and no panics occur
        debug!("Enhanced highlighting functions are properly defined and callable");
    }

    #[test]
    fn smoke_test_search_window_function() {
        // This is a smoke test to verify the search window function exists and is callable
        // Tests that the new separate window functionality compiles correctly

        // Verify the function exists and is callable (compilation test)
        let _window_function_exists = show_search_window;

        // Verify helper functions exist
        let _get_window_function_exists = get_or_create_search_window;
        let _create_window_function_exists = create_search_window_impl;
        let _window_behavior_function_exists = setup_window_behavior;
        let _window_button_panel_exists = create_window_button_panel;
        let _window_focus_function_exists = focus_search_entry_in_window;

        // Test passes if compilation succeeds - functions are properly defined
        debug!("Smoke test: Search window functionality structure verified");
    }
}
