use core::logic::layoutstate::{layout_state_label, LayoutState};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use gtk4::gio;
use gtk4::{
    self, prelude::*, Align, Box as GtkBox, Button, Label, Orientation, Paned, WindowHandle,
    Picture,
};
use rsvg::{CairoRenderer, Loader};
use gtk4::gdk;
use log::trace;

// Type alias for the complex rebuild callback type
type RebuildCallback = Box<dyn Fn()>;
type RebuildPopover = Rc<RefCell<Option<RebuildCallback>>>;
type WeakRebuildPopover = Weak<RefCell<Option<RebuildCallback>>>;

/// Helper function to reparent WebView back to main window from preview window
///
/// This encapsulates the common reparenting logic used by all layout button handlers.
/// Returns `true` if reparenting was performed or WebView was already in main window.
fn reparent_webview_to_main_window(
    webview_rc_opt: &Option<Rc<RefCell<webkit6::WebView>>>,
    split_opt: &Option<Paned>,
    preview_window_opt: &Option<
        Rc<RefCell<Option<crate::components::viewer::detached_window::PreviewWindow>>>,
    >,
    tracker_opt: &Option<crate::components::viewer::layout_controller::WebViewLocationTracker>,
    guard_opt: &Option<crate::components::viewer::reparenting::ReparentGuard>,
    layout_mode: &str, // For logging purposes
) -> bool {
    use crate::components::viewer::layout_controller::WebViewLocation;
    use crate::components::viewer::reparenting::move_webview_to_main_window;

    if let (Some(webview_rc), Some(split), Some(preview_window_opt), Some(tracker), Some(guard)) = (
        webview_rc_opt,
        split_opt,
        preview_window_opt,
        tracker_opt,
        guard_opt,
    ) {
        log::debug!(
            "{}: Current WebView location: {:?}",
            layout_mode,
            tracker.current()
        );

        // If WebView is in preview window, move it back
        if tracker.current() == WebViewLocation::PreviewWindow {
            log::info!("{}: WebView is in preview window, moving back", layout_mode);
            if guard.try_begin() {
                let webview_borrow = webview_rc.borrow();
                let preview_window_borrow = preview_window_opt.borrow();

                if let Some(ref preview_window) = *preview_window_borrow {
                    match move_webview_to_main_window(&webview_borrow, split, preview_window, true)
                    {
                        Ok(_) => {
                            tracker.set(WebViewLocation::MainWindow);
                            preview_window.hide();

                            // Ensure Stack shows html_preview after reparenting
                            if let Some(stack_widget) = split.end_child() {
                                if let Some(stack) = stack_widget.downcast_ref::<gtk4::Stack>() {
                                    stack.set_visible_child_name("html_preview");
                                    log::debug!("{}: Stack set to show html_preview", layout_mode);
                                }
                            }

                            log::info!("{}: WebView moved back to main window", layout_mode);
                        }
                        Err(e) => {
                            log::error!("{}: Failed to move WebView back: {}", layout_mode, e);
                            guard.end();
                            return false;
                        }
                    }
                } else {
                    log::warn!("{}: Preview window is None", layout_mode);
                    guard.end();
                    return false;
                }

                guard.end();
                return true;
            } else {
                log::warn!("{}: Failed to acquire reparent guard", layout_mode);
                return false;
            }
        } else {
            log::info!(
                "{}: WebView already in main window, no reparenting needed",
                layout_mode
            );

            // Even if already in main window, ensure Stack shows html_preview
            if let Some(stack_widget) = split.end_child() {
                if let Some(stack) = stack_widget.downcast_ref::<gtk4::Stack>() {
                    stack.set_visible_child_name("html_preview");
                    log::debug!(
                        "{}: Stack set to show html_preview (no reparenting)",
                        layout_mode
                    );
                }
            }
            return true;
        }
    }

    log::debug!("{}: Reparenting state not available", layout_mode);
    false
}

/// Helper function to create a menu button with a popover
fn create_menu_button(label: &str, menu: &gio::Menu) -> Button {
    let button = Button::with_label(label);
    button.add_css_class("menu-button");
    button.set_has_frame(false);

    // Create popover with the menu model
    let popover = gtk4::PopoverMenu::from_model(Some(menu));
    popover.set_parent(&button);

    // Connect button click to show popover
    let popover_clone = popover.clone();
    button.connect_clicked(move |_| {
        popover_clone.popup();
    });

    button
}

pub fn main_menu_structure() -> (GtkBox, gio::Menu) {
    // File menu with document operations and application settings
    let file_menu = gio::Menu::new();
    file_menu.append(Some("New"), Some("app.new"));
    file_menu.append(Some("Open"), Some("app.open"));

    // Recent Files submenu: the application can populate this at runtime.
    // Create the submenu model that will be mutated at runtime.
    let recent_menu = gio::Menu::new();
    // Placeholder disabled entry shown when there are no recent files
    let placeholder = gio::MenuItem::new(Some("(No recent files)"), None::<&str>);
    recent_menu.append_item(&placeholder);
    // Create a MenuItem that references the application action "app.recent"
    // so enabling/disabling that action will also affect the top-level menu item.
    let recent_menu_item = gio::MenuItem::new(Some("Recent Files"), Some("app.recent"));
    // Attach the submenu to the menu item
    recent_menu_item.set_submenu(Some(&recent_menu));
    // Append the menu item to the File menu
    file_menu.append_item(&recent_menu_item);
    file_menu.append(Some("Save"), Some("app.save"));
    file_menu.append(Some("Save As"), Some("app.save_as"));
    file_menu.append(Some("Export"), Some("app.export"));
    file_menu.append(Some("Settings"), Some("app.settings"));
    file_menu.append(Some("Quit"), Some("app.quit"));

    // Edit menu with text editing and search operations
    let edit_menu = gio::Menu::new();
    edit_menu.append(Some("Undo"), Some("app.undo"));
    edit_menu.append(Some("Redo"), Some("app.redo"));
    edit_menu.append(Some("Cut"), Some("app.cut"));
    edit_menu.append(Some("Copy"), Some("app.copy"));
    edit_menu.append(Some("Paste"), Some("app.paste"));
    edit_menu.append(Some("Search & Replace"), Some("app.search"));

    // Document menu with builder and splitter tools
    let document_menu = gio::Menu::new();
    document_menu.append(Some("Document Builder"), Some("app.document_builder"));
    document_menu.append(Some("Document Splitter"), Some("app.document_splitter"));

    // Bookmarks menu (empty for now)
    let bookmarks_menu = gio::Menu::new();
    let placeholder_bookmark = gio::MenuItem::new(Some("(No bookmarks)"), None::<&str>);
    bookmarks_menu.append_item(&placeholder_bookmark);

    // Format menu with text styling options
    let format_menu = gio::Menu::new();
    format_menu.append(Some("Bold"), Some("app.bold"));
    format_menu.append(Some("Italic"), Some("app.italic"));
    format_menu.append(Some("Code"), Some("app.code"));

    // View menu with display and layout options
    let view_menu = gio::Menu::new();
    view_menu.append(Some("HTML Preview"), Some("app.view_html"));
    view_menu.append(Some("Code View"), Some("app.view_code"));

    // Help menu with application information
    let help_menu = gio::Menu::new();
    help_menu.append(Some("About"), Some("app.about"));

    // Create horizontal box for menu buttons
    let menu_box = GtkBox::new(Orientation::Horizontal, 0);
    menu_box.add_css_class("menubar");

    // Create menu buttons
    let file_btn = create_menu_button("File", &file_menu);
    let edit_btn = create_menu_button("Edit", &edit_menu);
    let document_btn = create_menu_button("Document", &document_menu);
    let bookmarks_btn = create_menu_button("Bookmarks", &bookmarks_menu);
    let format_btn = create_menu_button("Format", &format_menu);
    let view_btn = create_menu_button("View", &view_menu);
    let help_btn = create_menu_button("Help", &help_menu);

    // Add buttons to the box
    menu_box.append(&file_btn);
    menu_box.append(&edit_btn);
    menu_box.append(&document_btn);
    menu_box.append(&bookmarks_btn);
    menu_box.append(&format_btn);
    menu_box.append(&view_btn);
    menu_box.append(&help_btn);

    (menu_box, recent_menu)
}

use crate::components::viewer::layout_controller::{SplitController, WebViewLocationTracker};
use crate::components::viewer::detached_window::PreviewWindow;
use crate::components::viewer::reparenting::ReparentGuard;

/// Configuration for creating the custom titlebar
pub struct TitlebarConfig<'a> {
    pub window: &'a gtk4::ApplicationWindow,
    pub webview_rc: Option<Rc<RefCell<webkit6::WebView>>>,
    pub split: Option<Paned>,
    pub preview_window_opt: Option<Rc<RefCell<Option<PreviewWindow>>>>,
    pub webview_location_tracker: Option<WebViewLocationTracker>,
    pub reparent_guard: Option<ReparentGuard>,
    pub split_controller: Option<SplitController>,
    pub asset_root: &'a std::path::Path,
}

/// Returns a WindowHandle containing the custom menu bar and all controls.
/// Returns a WindowHandle and the central title `Label` so callers can update the
/// document title (and modification marker) dynamically.
pub fn create_custom_titlebar(config: TitlebarConfig) -> (WindowHandle, Label, gio::Menu) {
    // Destructure config for easier access
    let TitlebarConfig {
        window,
        webview_rc,
        split,
        preview_window_opt,
        webview_location_tracker,
        reparent_guard,
        split_controller,
        asset_root,
    } = config;

    // Create WindowHandle wrapper for proper window dragging
    let handle = WindowHandle::new();

    // Use GTK4 HeaderBar for proper title centering
    let headerbar = gtk4::HeaderBar::new();
    headerbar.add_css_class("titlebar");
    headerbar.set_show_title_buttons(false); // We'll add custom window controls

    // App icon (left) - uses dynamic asset directory path
    let icon_path = asset_root.join("icons/icon_64x64_marco.png");
    let icon = Image::from_file(&icon_path);
    icon.set_pixel_size(16);
    icon.set_halign(Align::Start);
    icon.set_margin_start(5);
    icon.set_margin_end(5);
    icon.set_valign(Align::Center);
    icon.set_tooltip_text(Some("Marco a markdown composer"));
    headerbar.pack_start(&icon);

    // --- Menu bar (next to icon) ---
    let (menu_bar, recent_menu) = main_menu_structure();
    menu_bar.set_valign(Align::Center);
    menu_bar.add_css_class("menubar");
    headerbar.pack_start(&menu_bar);

    // Centered document title label as custom title widget
    let title_label = Label::new(None);
    title_label.set_valign(Align::Center);
    title_label.add_css_class("title-label");
    // Start with placeholder
    title_label.set_text("Untitled.md");
    // Set as title widget - HeaderBar will automatically center it
    headerbar.set_title_widget(Some(&title_label));

    use gtk4::Image;

    // --- actions layout button ---
    use gtk4::{Orientation, Popover};
    let layout_menu_btn = Button::new();
    // Tooltip will be set after state is created (below)
    layout_menu_btn.set_valign(Align::Center);
    layout_menu_btn.set_margin_start(0);
    layout_menu_btn.set_margin_end(0);
    layout_menu_btn.set_focusable(false);
    layout_menu_btn.set_can_focus(false);
    layout_menu_btn.set_has_frame(false);
    layout_menu_btn.add_css_class("topright-btn");
    // Use same visual style as window control buttons
    layout_menu_btn.add_css_class("window-control-btn");

    // State management (single shared instance)
    let layout_state = Rc::new(RefCell::new(LayoutState::DualView));
    // Track the previous layout state before switching to EditorAndViewSeparate
    // This allows us to return to the exact state when closing the preview window
    let previous_layout_state = Rc::new(RefCell::new(LayoutState::DualView));
    // Track the split position when in DualView mode
    let previous_split_position = Rc::new(RefCell::new(0i32));
    // Set initial tooltip to the human-readable current layout label
    layout_menu_btn.set_tooltip_text(Some(layout_state_label(*layout_state.borrow())));

    // Use SVG layout switcher icon instead of IcoMoon
    let layout_icon_color: std::borrow::Cow<'static, str> = if window.style_context().has_class("marco-theme-dark") {
        std::borrow::Cow::from(DARK_PALETTE.control_icon)
    } else {
        std::borrow::Cow::from(LIGHT_PALETTE.control_icon)
    };
    let layout_pic = Picture::new();
    let layout_texture = {
        let svg = layout_icon_svg(LayoutIcon::LayoutSwitcherButton).replace("currentColor", &layout_icon_color);
        let bytes = glib::Bytes::from_owned(svg.into_bytes());
        let stream = gio::MemoryInputStream::from_bytes(&bytes);
        let handle = Loader::new()
            .read_stream(&stream, None::<&gio::File>, gio::Cancellable::NONE)
            .expect("load SVG handle");
        let display_scale = gtk4::gdk::Display::default()
            .and_then(|d| d.monitors().item(0))
            .and_then(|m| m.downcast::<gtk4::gdk::Monitor>().ok())
            .map(|m| m.scale_factor() as f64)
            .unwrap_or(1.0);
        let render_scale = display_scale * 2.0;
        let render_size = (LAYOUT_ICON_SIZE_F * render_scale) as i32;
        let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, render_size, render_size)
            .expect("create surface");
        {
            let cr = cairo::Context::new(&surface).expect("create context");
            cr.scale(render_scale, render_scale);
            let renderer = CairoRenderer::new(&handle);
            let viewport = cairo::Rectangle::new(0.0, 0.0, LAYOUT_ICON_SIZE_F, LAYOUT_ICON_SIZE_F);
            renderer.render_document(&cr, &viewport).expect("render SVG");
        }
        let data = surface.data().expect("get surface data").to_vec();
        let bytes = glib::Bytes::from_owned(data);
        gtk4::gdk::MemoryTexture::new(
            render_size,
            render_size,
            gtk4::gdk::MemoryFormat::B8g8r8a8Premultiplied,
            &bytes,
            (render_size * 4) as usize,
        )
    };
    layout_pic.set_paintable(Some(&layout_texture));
    layout_pic.set_size_request(LAYOUT_ICON_SIZE_F as i32, LAYOUT_ICON_SIZE_F as i32);
    layout_menu_btn.add_css_class("window-control-btn");
    layout_menu_btn.set_child(Some(&layout_pic));

    // Add hover/active interaction to layout switcher to match window controls
    {
        let pic_hover = layout_pic.clone();
        let is_dark = window.style_context().has_class("marco-theme-dark");
        let hover_color = if is_dark { DARK_PALETTE.control_icon_hover.to_string() } else { LIGHT_PALETTE.control_icon_hover.to_string() };
        let active_color = if is_dark { DARK_PALETTE.control_icon_active.to_string() } else { LIGHT_PALETTE.control_icon_active.to_string() };
        let normal_color = layout_icon_color.clone().to_string();
        let icon = LayoutIcon::LayoutSwitcherButton;

        let motion_controller = gtk4::EventControllerMotion::new();
        let hover_color_enter = hover_color.clone();
        motion_controller.connect_enter(move |_ctrl, _x, _y| {
            let texture = render_layout_svg_icon(icon, &hover_color_enter, LAYOUT_ICON_SIZE_F);
            pic_hover.set_paintable(Some(&texture));
        });

        let pic_leave = layout_pic.clone();
        let normal_color_leave = normal_color.clone();
        let icon_for_leave = icon;
        motion_controller.connect_leave(move |_ctrl| {
            let texture = render_layout_svg_icon(icon_for_leave, &normal_color_leave, LAYOUT_ICON_SIZE_F);
            pic_leave.set_paintable(Some(&texture));
        });
        layout_menu_btn.add_controller(motion_controller);

        let gesture = gtk4::GestureClick::new();
        let pic_pressed = layout_pic.clone();
        let active_color_pressed = active_color.clone();
        let icon_for_pressed = icon;
        gesture.connect_pressed(move |_gesture, _n, _x, _y| {
            let texture = render_layout_svg_icon(icon_for_pressed, &active_color_pressed, LAYOUT_ICON_SIZE_F);
            pic_pressed.set_paintable(Some(&texture));
        });

        let pic_released = layout_pic.clone();
        let hover_color_released = hover_color.clone();
        let icon_for_released = icon;
        gesture.connect_released(move |_gesture, _n, _x, _y| {
            let texture = render_layout_svg_icon(icon_for_released, &hover_color_released, LAYOUT_ICON_SIZE_F);
            pic_released.set_paintable(Some(&texture));
        });
        layout_menu_btn.add_controller(gesture);
    }

    // Helper to (re)build the popover content based on state
    let popover = Popover::new();
    // Attach the popover to the layout button for proper positioning
    popover.set_parent(&layout_menu_btn);
    // Remove unused duplicate clone

    // Create window weak reference for reparenting logic (before rebuild closure)
    // Clone reparenting parameters for capture in rebuild closure
    let window_weak_for_reparent = window.downgrade();
    let webview_rc_for_rebuild = webview_rc.clone();
    let split_for_rebuild = split.clone();
    let preview_window_opt_for_rebuild = preview_window_opt.clone();
    let webview_location_tracker_for_rebuild = webview_location_tracker.clone();
    let reparent_guard_for_rebuild = reparent_guard.clone();
    let split_controller_for_rebuild = split_controller.clone();

    let rebuild_popover: RebuildPopover = Rc::new(RefCell::new(None));

    let weak_rebuild_popover: WeakRebuildPopover = Rc::downgrade(&rebuild_popover);

    // Pre-create layout popover buttons to avoid capturing non-'static `window` inside the rebuild closure
    const LAYOUT_ICON_SIZE_F: f64 = 14.0;
    let base_icon_color: &'static str = if window.style_context().has_class("marco-theme-dark") {
        DARK_PALETTE.control_icon
    } else {
        LIGHT_PALETTE.control_icon
    };

    // Button 1: Close view (show only editor)
    let btn1 = svg_layout_button(window, LayoutIcon::EditorOnly, "Close view (show only editor)", base_icon_color, LAYOUT_ICON_SIZE_F);
    btn1.add_css_class("layout-btn");
    btn1.set_halign(Align::Start);
    {
        let layout_state = layout_state.clone();
        let weak_rebuild_local = weak_rebuild_popover.clone();
        let webview_rc_opt = webview_rc_for_rebuild.clone();
        let split_opt = split_for_rebuild.clone();
        let preview_window_opt_clone = preview_window_opt_for_rebuild.clone();
        let webview_location_tracker_opt = webview_location_tracker_for_rebuild.clone();
        let reparent_guard_opt = reparent_guard_for_rebuild.clone();
        let split_controller_opt = split_controller_for_rebuild.clone();
        btn1.connect_clicked(move |_| {
            let next = LayoutState::EditorOnly;
            *layout_state.borrow_mut() = next;
            reparent_webview_to_main_window(
                &webview_rc_opt,
                &split_opt,
                &preview_window_opt_clone,
                &webview_location_tracker_opt,
                &reparent_guard_opt,
                "EditorOnly",
            );
            if let Some(controller) = &split_controller_opt {
                controller.set_mode(next);
            }
            if let Some(rc) = weak_rebuild_local.upgrade() {
                if let Some(ref rebuild) = *rc.borrow() {
                    rebuild();
                }
            }
        });
    }

    // Button 2: Close editor (show only view)
    let btn2 = svg_layout_button(window, LayoutIcon::ViewOnly, "Close editor (show only view)", base_icon_color, LAYOUT_ICON_SIZE_F);
    btn2.add_css_class("layout-btn");
    btn2.set_halign(Align::Start);
    {
        let layout_state = layout_state.clone();
        let weak_rebuild_local = weak_rebuild_popover.clone();
        let webview_rc_opt = webview_rc_for_rebuild.clone();
        let split_opt = split_for_rebuild.clone();
        let preview_window_opt_clone = preview_window_opt_for_rebuild.clone();
        let webview_location_tracker_opt = webview_location_tracker_for_rebuild.clone();
        let reparent_guard_opt = reparent_guard_for_rebuild.clone();
        let split_controller_opt = split_controller_for_rebuild.clone();
        btn2.connect_clicked(move |_| {
            let next = LayoutState::ViewOnly;
            *layout_state.borrow_mut() = next;
            reparent_webview_to_main_window(
                &webview_rc_opt,
                &split_opt,
                &preview_window_opt_clone,
                &webview_location_tracker_opt,
                &reparent_guard_opt,
                "ViewOnly",
            );
            if let Some(controller) = &split_controller_opt {
                controller.set_mode(next);
            }
            if let Some(rc) = weak_rebuild_local.upgrade() {
                if let Some(ref rebuild) = *rc.borrow() {
                    rebuild();
                }
            }
        });
    }

    // Button 3: Open view in separate window
    let btn3 = svg_layout_button(window, LayoutIcon::EditorAndViewSeparate, "Open view in separate window", base_icon_color, LAYOUT_ICON_SIZE_F);
    btn3.add_css_class("layout-btn");
    btn3.set_tooltip_text(Some("Open view in separate window"));
    btn3.set_halign(Align::Start);
    {
        let layout_state = layout_state.clone();
        let weak_rebuild_local = weak_rebuild_popover.clone();
        let webview_rc_opt = webview_rc_for_rebuild.clone();
        let split_opt = split_for_rebuild.clone();
        let preview_window_opt_clone = preview_window_opt_for_rebuild.clone();
        let webview_location_tracker_opt = webview_location_tracker_for_rebuild.clone();
        let reparent_guard_opt = reparent_guard_for_rebuild.clone();
        let window_weak = window_weak_for_reparent.clone();
        let split_controller_opt = split_controller_for_rebuild.clone();
        let previous_layout_state_for_btn3 = previous_layout_state.clone();
        let previous_split_position_for_btn3 = previous_split_position.clone();
        btn3.connect_clicked(move |_| {
            // Store the current layout state before switching to EditorAndViewSeparate
            let current_state = *layout_state.borrow();
            *previous_layout_state_for_btn3.borrow_mut() = current_state;
            if current_state == LayoutState::DualView {
                if let Some(ref split) = split_opt {
                    let current_position = split.position();
                    *previous_split_position_for_btn3.borrow_mut() = current_position;
                    log::info!("Storing previous DualView split position: {}", current_position);
                }
            }
            log::info!("Storing previous layout state: {:?} before switching to EditorAndViewSeparate", current_state);
            let next = LayoutState::EditorAndViewSeparate;
            *layout_state.borrow_mut() = next;
            if let Some(controller) = &split_controller_opt {
                controller.set_mode(next);
            }

            if let (Some(webview_rc), Some(split), Some(preview_window_opt), Some(tracker), Some(guard)) = (&webview_rc_opt, &split_opt, &preview_window_opt_clone, &webview_location_tracker_opt, &reparent_guard_opt) {
                use crate::components::viewer::detached_window::PreviewWindow;
                use crate::components::viewer::reparenting::move_webview_to_preview_window;
                use crate::components::viewer::layout_controller::WebViewLocation;

                if tracker.current() == WebViewLocation::MainWindow {
                    if guard.try_begin() {
                        let should_reparent = {
                            let mut opt_borrow = preview_window_opt.borrow_mut();
                            if opt_borrow.is_none() {
                                if let Some(window) = window_weak.upgrade() {
                                    if let Some(app) = window.application() {
                                        let new_preview_window = PreviewWindow::new(&window, &app);
                                        let layout_state_for_callback = layout_state.clone();
                                        let previous_layout_state_for_callback = previous_layout_state_for_btn3.clone();
                                        let previous_split_position_for_callback = previous_split_position_for_btn3.clone();
                                        let webview_rc_for_callback = webview_rc.clone();
                                        let split_for_callback = split.clone();
                                        let tracker_for_callback = tracker.clone();
                                        let guard_for_callback = guard.clone();
                                        let preview_window_opt_weak = Rc::downgrade(preview_window_opt);
                                        let weak_rebuild_for_callback = weak_rebuild_local.clone();
                                        let split_controller_for_callback = split_controller_opt.clone();

                                        new_preview_window.set_on_close_callback(move || {
                                            use crate::components::viewer::reparenting::move_webview_to_main_window;
                                            use crate::components::viewer::layout_controller::WebViewLocation;

                                            log::info!("Preview window close callback triggered");

                                            let preview_window_opt = match preview_window_opt_weak.upgrade() {
                                                Some(p) => p,
                                                None => {
                                                    log::warn!("preview_window_opt dropped, aborting callback");
                                                    return;
                                                }
                                            };

                                            // Restore to the previous layout state (the state before EditorAndViewSeparate)
                                            let previous_state = *previous_layout_state_for_callback.borrow();
                                            *layout_state_for_callback.borrow_mut() = previous_state;
                                            log::info!("Restoring to previous layout state: {:?}", previous_state);

                                            if let Some(ref controller) = split_controller_for_callback {
                                                controller.set_mode(previous_state);
                                                log::info!("Split controller set to {:?} mode", previous_state);
                                            }

                                            if previous_state == LayoutState::DualView {
                                                let saved_position = *previous_split_position_for_callback.borrow();
                                                if saved_position > 0 {
                                                    let split_for_position = split_for_callback.clone();
                                                    glib::idle_add_local_once(move || {
                                                        split_for_position.set_position(saved_position);
                                                        log::info!("Restored DualView split position to: {}", saved_position);
                                                    });
                                                }
                                            }

                                            if tracker_for_callback.current() == WebViewLocation::PreviewWindow && guard_for_callback.try_begin() {
                                                let webview_borrow = webview_rc_for_callback.borrow();
                                                let preview_window_borrow = preview_window_opt.borrow();

                                                if let Some(ref preview_window) = *preview_window_borrow {
                                                    match move_webview_to_main_window(&webview_borrow, &split_for_callback, preview_window, true) {
                                                        Ok(_) => {
                                                            tracker_for_callback.set(WebViewLocation::MainWindow);
                                                            if let Some(stack_widget) = split_for_callback.end_child() {
                                                                if let Some(stack) = stack_widget.downcast_ref::<gtk4::Stack>() {
                                                                    stack.set_visible_child_name("html_preview");
                                                                    log::info!("Stack set to show html_preview after window close");
                                                                }
                                                            }
                                                            log::info!("WebView reparented back to main window after preview window close");
                                                        }
                                                        Err(e) => {
                                                            log::error!("Failed to reparent WebView after window close: {}", e);
                                                        }
                                                    }
                                                }
                                                guard_for_callback.end();
                                            }

                                            if let Some(rc) = weak_rebuild_for_callback.upgrade() {
                                                if let Some(ref rebuild) = *rc.borrow() {
                                                    rebuild();
                                                }
                                            }
                                        });

                                        *opt_borrow = Some(new_preview_window);
                                        log::info!("Created new preview window for EditorAndViewSeparate mode with close callback");
                                    }
                                }
                            }
                            opt_borrow.is_some()
                        };

                        if should_reparent {
                            let webview_borrow = webview_rc.borrow();
                            let preview_window_borrow = preview_window_opt.borrow();
                            if let Some(ref preview_window) = *preview_window_borrow {
                                match move_webview_to_preview_window(&webview_borrow, split, preview_window) {
                                    Ok(_) => {
                                        tracker.set(WebViewLocation::PreviewWindow);
                                        log::info!("Successfully moved WebView to preview window");
                                        preview_window.show();
                                    }
                                    Err(e) => {
                                        log::error!("Failed to move WebView to preview window: {}", e);
                                    }
                                }
                            }
                        }

                        guard.end();
                    } else {
                        log::warn!("Cannot reparent WebView: reparenting already in progress");
                    }
                }
            }

            if let Some(rc) = weak_rebuild_local.upgrade() {
                if let Some(ref rebuild) = *rc.borrow() {
                    rebuild();
                }
            }
        });
    }

    // Button 4: Restore default split view (pre-created)
    let btn4 = svg_layout_button(window, LayoutIcon::DualView, "Restore default split view", base_icon_color, LAYOUT_ICON_SIZE_F);
    btn4.add_css_class("layout-btn");
    btn4.set_halign(Align::Start);
    {
        let layout_state = layout_state.clone();
        let weak_rebuild_local = weak_rebuild_popover.clone();
        let webview_rc_opt = webview_rc_for_rebuild.clone();
        let split_opt = split_for_rebuild.clone();
        let preview_window_opt_clone = preview_window_opt_for_rebuild.clone();
        let webview_location_tracker_opt = webview_location_tracker_for_rebuild.clone();
        let reparent_guard_opt = reparent_guard_for_rebuild.clone();
        let split_controller_opt = split_controller_for_rebuild.clone();
        btn4.connect_clicked(move |_| {
            let next = LayoutState::DualView;
            *layout_state.borrow_mut() = next;

            // Handle reparenting if needed (from EditorAndViewSeparate back to DualView)
            reparent_webview_to_main_window(
                &webview_rc_opt,
                &split_opt,
                &preview_window_opt_clone,
                &webview_location_tracker_opt,
                &reparent_guard_opt,
                "DualView",
            );

            // Set split controller to DualView mode (unlocks split, 50% position)
            if let Some(controller) = &split_controller_opt {
                controller.set_mode(next);
            }

            if let Some(rc) = weak_rebuild_local.upgrade() {
                if let Some(ref rebuild) = *rc.borrow() {
                    rebuild();
                }
            }
        });
    }

    let layout_state_clone2 = layout_state.clone(); // Used for popover logic
    let previous_layout_state_clone = previous_layout_state.clone(); // Used for tracking state before EditorAndViewSeparate
    let previous_split_position_clone = previous_split_position.clone(); // Used for tracking split position
    let popover_clone = popover.clone();
    // Clone the layout menu button so the rebuild closure can update its tooltip
    let layout_menu_btn_for_rebuild = layout_menu_btn.clone();
    *rebuild_popover.borrow_mut() = Some(Box::new(move || {
        let state = *layout_state_clone2.borrow();
        // Update the layout button tooltip to reflect the current state
        layout_menu_btn_for_rebuild.set_tooltip_text(Some(layout_state_label(state)));
        let popover_box = GtkBox::new(Orientation::Horizontal, 6);
        popover_box.set_margin_top(8);
        popover_box.set_margin_bottom(8);
        popover_box.set_margin_start(8);
        popover_box.set_margin_end(8);

        // Button 1: Close view (show only editor)
        if matches!(
            state,
            LayoutState::DualView | LayoutState::ViewOnly | LayoutState::EditorAndViewSeparate
        ) {
            // Use pre-created button
            if let Some(_) = btn1.parent() { btn1.unparent(); }
            popover_box.append(&btn1);
        }

        // Button 2: Close editor (show only view)
        if matches!(
            state,
            LayoutState::DualView | LayoutState::EditorOnly | LayoutState::EditorAndViewSeparate
        ) {
            // Use pre-created button
            if let Some(_) = btn2.parent() { btn2.unparent(); }
            popover_box.append(&btn2);
        }

        // Button 3: Close view (open view in separate window)
        if matches!(state, LayoutState::DualView | LayoutState::ViewOnly) {
            if let Some(_) = btn3.parent() { btn3.unparent(); }
            popover_box.append(&btn3);
        }

        // Button 4: Restore default split view
        if !matches!(state, LayoutState::DualView) {
            // Use pre-created button
            if let Some(_) = btn4.parent() { btn4.unparent(); }
            popover_box.append(&btn4);
        }

        // Set the new child; GTK4 will replace the old one automatically
        popover_clone.set_child(Some(&popover_box));
        popover_clone.set_has_arrow(true);
        popover_clone.set_position(gtk4::PositionType::Bottom);
        popover_clone.set_autohide(true);
    }) as Box<dyn Fn()>);

    // Initial build
    if let Some(ref rebuild) = *rebuild_popover.borrow() {
        rebuild();
    }

    let popover_ref = Rc::new(popover);
    let rebuild_popover_for_btn = rebuild_popover.clone();
    let popover_for_btn = popover_ref.clone();
    layout_menu_btn.connect_clicked(move |_btn| {
        if let Some(ref rebuild) = *rebuild_popover_for_btn.borrow() {
            rebuild();
        }
        // Popover is already parented to the button, so just popup
        popover_for_btn.popup();
        trace!("audit: layout menu opened");
    });

    use gtk4::Label;

    use crate::ui::css::constants::{DARK_PALETTE, LIGHT_PALETTE};
use core::logic::loaders::icon_loader::{layout_icon_svg, LayoutIcon, window_icon_svg, WindowIcon};
    // Helper: render an SVG icon into a GDK memory texture at high DPI for crisp icons
    fn render_svg_icon(icon: WindowIcon, color: &str, icon_size: f64) -> gdk::MemoryTexture {
        let svg = window_icon_svg(icon).replace("currentColor", color);
        let bytes = glib::Bytes::from_owned(svg.into_bytes());
        let stream = gio::MemoryInputStream::from_bytes(&bytes);

        // Use librsvg for native SVG rendering
        let handle = Loader::new()
            .read_stream(&stream, None::<&gio::File>, gio::Cancellable::NONE)
            .expect("load SVG handle");

        // Get scale factor for HiDPI displays
        let display_scale = gdk::Display::default()
            .and_then(|d| d.monitors().item(0))
            .and_then(|m| m.downcast::<gdk::Monitor>().ok())
            .map(|m| m.scale_factor() as f64)
            .unwrap_or(1.0);

        // Render at 2x the display scale for extra sharpness (prevents pixelation)
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

        // Convert cairo surface to GDK texture
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

    // Helper: render layout SVG icons (uses LayoutIcon) - same approach as render_svg_icon
    fn render_layout_svg_icon(icon: LayoutIcon, color: &str, icon_size: f64) -> gdk::MemoryTexture {
        let svg = layout_icon_svg(icon).replace("currentColor", color);
        let bytes = glib::Bytes::from_owned(svg.as_bytes().to_vec());
        let stream = gio::MemoryInputStream::from_bytes(&bytes);

        let handle = match Loader::new().read_stream(&stream, None::<&gio::File>, gio::Cancellable::NONE) {
            Ok(h) => h,
            Err(e) => {
                log::error!("load layout SVG handle: {}", e);
                log::error!("SVG content was: {}", svg);
                // Fallback tiny transparent texture so UI can continue
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

    // Helper to create a button with layout SVG icon and hover/active color changes
    fn svg_layout_button(window: &gtk4::ApplicationWindow, icon: LayoutIcon, tooltip: &str, color: &str, icon_size: f64) -> Button {
        let pic = Picture::new();
        let texture = render_layout_svg_icon(icon, color, icon_size);
        pic.set_paintable(Some(&texture));
        pic.set_size_request(icon_size as i32, icon_size as i32);
        pic.set_can_shrink(false);
        pic.set_halign(Align::Center);
        pic.set_valign(Align::Center);

        let btn = Button::new();
        btn.set_child(Some(&pic));
        btn.set_tooltip_text(Some(tooltip));
        btn.set_valign(Align::Center);
        btn.set_margin_start(1);
        btn.set_margin_end(1);
        btn.set_focusable(false);
        btn.set_can_focus(false);
        btn.set_has_frame(false);
        // Auto-calculate button size: icon + padding for comfortable click target
        btn.set_width_request((icon_size + 6.0) as i32);
        btn.set_height_request((icon_size + 6.0) as i32);
        btn.add_css_class("topright-btn");
        btn.add_css_class("window-control-btn");
        btn.add_css_class("layout-btn");

        // Add hover state handling - regenerate icon with hover color
        {
            let pic_hover = pic.clone();
            let normal_color = color.to_string();
            let is_dark = window.style_context().has_class("marco-theme-dark");
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

            let motion_controller = gtk4::EventControllerMotion::new();
            let icon_for_enter = icon;
            let hover_color_enter = hover_color.clone();
            motion_controller.connect_enter(move |_ctrl, _x, _y| {
                let texture = render_layout_svg_icon(icon_for_enter, &hover_color_enter, icon_size);
                pic_hover.set_paintable(Some(&texture));
            });

            let pic_leave = pic.clone();
            let icon_for_leave = icon;
            let normal_color_leave = normal_color.clone();
            motion_controller.connect_leave(move |_ctrl| {
                let texture = render_layout_svg_icon(icon_for_leave, &normal_color_leave, icon_size);
                pic_leave.set_paintable(Some(&texture));
            });
            btn.add_controller(motion_controller);

            // Add click state handling
            let gesture = gtk4::GestureClick::new();
            let pic_pressed = pic.clone();
            let icon_for_pressed = icon;
            let active_color_pressed = active_color.clone();
            gesture.connect_pressed(move |_gesture, _n, _x, _y| {
                let texture = render_layout_svg_icon(icon_for_pressed, &active_color_pressed, icon_size);
                pic_pressed.set_paintable(Some(&texture));
            });

            let pic_released = pic.clone();
            let icon_for_released = icon;
            gesture.connect_released(move |_gesture, _n, _x, _y| {
                let texture = render_layout_svg_icon(icon_for_released, &hover_color, icon_size);
                pic_released.set_paintable(Some(&texture));
            });
            btn.add_controller(gesture);
        }

        btn
    }

    // Helper to create a button with SVG icon and hover/active color changes
    fn svg_icon_button(window: &gtk4::ApplicationWindow, icon: WindowIcon, tooltip: &str, color: &str, icon_size: f64) -> Button {
        let pic = Picture::new();
        let texture = render_svg_icon(icon, color, icon_size);
        pic.set_paintable(Some(&texture));
        pic.set_size_request(icon_size as i32, icon_size as i32);
        pic.set_can_shrink(false);
        pic.set_halign(Align::Center);
        pic.set_valign(Align::Center);

        let btn = Button::new();
        btn.set_child(Some(&pic));
        btn.set_tooltip_text(Some(tooltip));
        btn.set_valign(Align::Center);
        btn.set_margin_start(1);
        btn.set_margin_end(1);
        btn.set_focusable(false);
        btn.set_can_focus(false);
        btn.set_has_frame(false);
        // Auto-calculate button size: icon + padding for comfortable click target
        btn.set_width_request((icon_size + 6.0) as i32);
        btn.set_height_request((icon_size + 6.0) as i32);
        btn.add_css_class("topright-btn");
        btn.add_css_class("window-control-btn");

        // Add hover state handling - regenerate icon with hover color
        {
            let pic_hover = pic.clone();
            let normal_color = color.to_string();
            let is_dark = window.style_context().has_class("marco-theme-dark");
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

            // Add click state handling
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

    // Create window control buttons (minimize, maximize/restore, close)
    fn create_window_controls(window: &gtk4::ApplicationWindow) -> (Button, Button, Button) {
        const ICON_SIZE: f64 = 8.0;

        // Use palette colors for window control icons (not hardcoded)
        // Use Polo-aligned palette control colors for the icon itself
        let icon_color: std::borrow::Cow<'static, str> = if window.style_context().has_class("marco-theme-dark") {
            std::borrow::Cow::from(DARK_PALETTE.control_icon)
        } else {
            std::borrow::Cow::from(LIGHT_PALETTE.control_icon)
        };

        let btn_min = svg_icon_button(window, WindowIcon::Minimize, "Minimize", &icon_color, ICON_SIZE);
        let btn_close = svg_icon_button(window, WindowIcon::Close, "Close", &icon_color, ICON_SIZE);

        // Create maximize/restore toggle button with its own picture for dynamic icon switching
        let max_pic = Picture::new();
        max_pic.set_size_request(ICON_SIZE as i32, ICON_SIZE as i32);
        max_pic.set_can_shrink(false);
        max_pic.set_halign(Align::Center);
        max_pic.set_valign(Align::Center);

        // Helper closure to update maximize button icon based on window state
        let update_max_icon = {
            let color = icon_color.clone();
            move |is_maximized: bool, pic: &Picture| {
                let icon = if is_maximized { WindowIcon::Restore } else { WindowIcon::Maximize };
                let texture = render_svg_icon(icon, &color, ICON_SIZE);
                pic.set_paintable(Some(&texture));
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
        btn_max_toggle.set_width_request((ICON_SIZE + 6.0) as i32);
        btn_max_toggle.set_height_request((ICON_SIZE + 6.0) as i32);
        btn_max_toggle.set_can_focus(false);
        btn_max_toggle.set_has_frame(false);
        btn_max_toggle.add_css_class("topright-btn");
        btn_max_toggle.add_css_class("window-control-btn");

        // Add hover/active color changes for maximize button
        {
            let is_dark = window.style_context().has_class("marco-theme-dark");
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
            let normal_color = icon_color.to_string();

            let motion_controller = gtk4::EventControllerMotion::new();
            let pic_hover = max_pic.clone();
            let hover_color_enter = hover_color.clone();
            let window_hover_enter = window.clone();
            motion_controller.connect_enter(move |_ctrl, _x, _y| {
                let icon = if window_hover_enter.is_maximized() { WindowIcon::Restore } else { WindowIcon::Maximize };
                let texture = render_svg_icon(icon, &hover_color_enter, ICON_SIZE);
                pic_hover.set_paintable(Some(&texture));
            });

            let pic_leave = max_pic.clone();
            let normal_color_leave = normal_color.clone();
            let window_hover_leave = window.clone();
            motion_controller.connect_leave(move |_ctrl| {
                let icon = if window_hover_leave.is_maximized() { WindowIcon::Restore } else { WindowIcon::Maximize };
                let texture = render_svg_icon(icon, &normal_color_leave, ICON_SIZE);
                pic_leave.set_paintable(Some(&texture));
            });
            btn_max_toggle.add_controller(motion_controller);

            let gesture = gtk4::GestureClick::new();
            let pic_pressed = max_pic.clone();
            let active_color_pressed = active_color.clone();
            let window_pressed = window.clone();
            gesture.connect_pressed(move |_gesture, _n, _x, _y| {
                let icon = if window_pressed.is_maximized() { WindowIcon::Restore } else { WindowIcon::Maximize };
                let texture = render_svg_icon(icon, &active_color_pressed, ICON_SIZE);
                pic_pressed.set_paintable(Some(&texture));
            });

            let pic_released = max_pic.clone();
            let hover_color_released = hover_color.clone();
            let window_released = window.clone();
            gesture.connect_released(move |_gesture, _n, _x, _y| {
                let icon = if window_released.is_maximized() { WindowIcon::Restore } else { WindowIcon::Maximize };
                let texture = render_svg_icon(icon, &hover_color_released, ICON_SIZE);
                pic_released.set_paintable(Some(&texture));
            });
            btn_max_toggle.add_controller(gesture);
        }

        // Wire up window controls
        let window_for_min = window.clone();
        btn_min.connect_clicked(move |_| {
            window_for_min.minimize();
            trace!("audit: window minimize clicked");
        });

        // Click toggles window state and updates icon immediately
        let pic_for_toggle = max_pic.clone();
        let window_for_toggle = window.clone();
        let update_for_toggle = update_max_icon.clone();
        btn_max_toggle.connect_clicked(move |_| {
            if window_for_toggle.is_maximized() {
                window_for_toggle.unmaximize();
                update_for_toggle(false, &pic_for_toggle);
            } else {
                window_for_toggle.maximize();
                update_for_toggle(true, &pic_for_toggle);
            }
            trace!("audit: window maximize/restore clicked");
        });

        // Keep icon in sync if window is maximized/unmaximized externally
        let pic_for_notify = max_pic.clone();
        let update_for_notify = update_max_icon.clone();
        window.connect_notify_local(Some("is-maximized"), move |w, _| {
            update_for_notify(w.is_maximized(), &pic_for_notify);
        });

        let window_for_close = window.clone();
        btn_close.connect_clicked(move |_| {
            if let Some(app) = window_for_close.application() {
                // Activate the app-level action 'app.quit' which is registered in main.rs
                if let Some(action) = app.lookup_action("quit") {
                    action.activate(None);
                } else {
                    // Fallback: close the window if action not found
                    window_for_close.close();
                }
            } else {
                // Fallback: close the window if no application is associated
                window_for_close.close();
            }
            trace!("audit: window close clicked");
        });

        (btn_min, btn_max_toggle, btn_close)
    }

    // Create window controls (SVG-based) and add them to the headerbar
    let (btn_min, btn_max_toggle, btn_close) = create_window_controls(window);

    // Add controls to headerbar from right to left (pack_end order)
    headerbar.pack_end(&btn_close); // Rightmost
    headerbar.pack_end(&btn_max_toggle); // Middle
    headerbar.pack_end(&btn_min); // Left of window controls
    // Then add layout button (it will be to the left of window controls)
    headerbar.pack_end(&layout_menu_btn); // Left of minimize button

    // Add the HeaderBar to the WindowHandle
    handle.set_child(Some(&headerbar));
    (handle, title_label, recent_menu)
}
