/// Sets the height of the menu widget (Box or similar)
pub fn set_menu_height(menu_box: &gtk4::Box, height: i32) {
    menu_box.set_height_request(height);
}
// Helper to convert LayoutState to a human-readable string
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use crate::ui::menu_items::layoutstate::{LayoutState, layout_state_label};

use gtk4::{self, prelude::*, Box as GtkBox, Button, Align, WindowHandle};
use glib::ControlFlow;
use gtk4::gio;
use gtk4::PopoverMenuBar;

pub fn main_menu_structure() -> PopoverMenuBar {
    // ...existing code for menu structure...
    let menu_model = gio::Menu::new();
    let file_menu = gio::Menu::new();
    file_menu.append(Some("New"), Some("app.new"));
    file_menu.append(Some("Open"), Some("app.open"));
    file_menu.append(Some("Save"), Some("app.save"));
    file_menu.append(Some("Save As"), Some("app.save_as"));
    file_menu.append(Some("Settings"), Some("app.settings"));
    file_menu.append(Some("Quit"), Some("app.quit"));
    menu_model.append_submenu(Some("File"), &file_menu);
    let edit_menu = gio::Menu::new();
    edit_menu.append(Some("Undo"), Some("app.undo"));
    edit_menu.append(Some("Redo"), Some("app.redo"));
    edit_menu.append(Some("Cut"), Some("app.cut"));
    edit_menu.append(Some("Copy"), Some("app.copy"));
    edit_menu.append(Some("Paste"), Some("app.paste"));
    menu_model.append_submenu(Some("Edit"), &edit_menu);
    let format_menu = gio::Menu::new();
    format_menu.append(Some("Bold"), Some("app.bold"));
    format_menu.append(Some("Italic"), Some("app.italic"));
    format_menu.append(Some("Code"), Some("app.code"));
    menu_model.append_submenu(Some("Format"), &format_menu);
    let view_menu = gio::Menu::new();
    view_menu.append(Some("HTML Preview"), Some("app.view_html"));
    view_menu.append(Some("Code View"), Some("app.view_code"));
    menu_model.append_submenu(Some("View"), &view_menu);
    let help_menu = gio::Menu::new();
    help_menu.append(Some("About"), Some("app.about"));
    menu_model.append_submenu(Some("Help"), &help_menu);
    let menubar = PopoverMenuBar::from_model(Some(&menu_model));
    menubar.add_css_class("menubar");
    menubar
}

/// Returns a WindowHandle containing the custom menu bar and all controls.
pub fn create_custom_titlebar(window: &gtk4::ApplicationWindow) -> WindowHandle {

    use gtk4::gdk::Display;
    if Display::default().is_some() {
    }
    let handle = WindowHandle::new();
    let titlebar = GtkBox::new(Orientation::Horizontal, 0);
    titlebar.add_css_class("titlebar");
    titlebar.set_spacing(0);
    titlebar.set_margin_top(0);
    titlebar.set_margin_bottom(0);
    titlebar.set_margin_start(0);
    titlebar.set_margin_end(0);
    set_menu_height(&titlebar, 0); // Minimum height, matches footer

    // App icon (left)
    let icon = Image::from_file("src/assets/icons/favicon.png");
    icon.set_pixel_size(16);
    icon.set_halign(Align::Start);
    icon.set_margin_start(5);
    icon.set_margin_end(5);
    icon.set_valign(Align::Center);
    icon.set_tooltip_text(Some("Marco a markdown composer"));
    titlebar.append(&icon);

    // --- Menu bar (next to title) ---
    let menu_bar = main_menu_structure();
    menu_bar.set_valign(Align::Center);
    menu_bar.add_css_class("menubar");
    titlebar.append(&menu_bar);


    // Spacer (expand to push controls to right)
    let spacer = GtkBox::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    titlebar.append(&spacer);

    use gtk4::Image;

     // --- actions layout button ---
    use gtk4::{Popover, Orientation};
    let layout_menu_btn = Button::new();
    layout_menu_btn.set_tooltip_text(Some("Layout options"));
    layout_menu_btn.set_valign(Align::Center);
    layout_menu_btn.set_margin_start(2);
    layout_menu_btn.set_margin_end(2);
    layout_menu_btn.set_focusable(false);
    layout_menu_btn.set_can_focus(false);
    layout_menu_btn.set_has_frame(false);
    layout_menu_btn.add_css_class("topright-btn");

    // State management (single shared instance)
    let layout_state = Rc::new(RefCell::new(LayoutState::Split));


    // Use provided SVG for layout button icon
    let img_menu = gtk4::Image::from_file("src/assets/icons/split_scene_left.svg");
    img_menu.set_pixel_size(16);
    layout_menu_btn.set_child(Some(&img_menu));

    // Helper to (re)build the popover content based on state
    let popover = Popover::new();
    // Attach the popover to the window to ensure it is in a toplevel container
    popover.set_parent(window);
    // Remove unused duplicate clone

    let rebuild_popover: Rc<RefCell<Option<Box<dyn Fn()>>>> = Rc::new(RefCell::new(None));

    let weak_rebuild_popover: Weak<RefCell<Option<Box<dyn Fn()>>>> = Rc::downgrade(&rebuild_popover);
    let layout_state_clone2 = layout_state.clone(); // Used for popover logic
    let popover_clone = popover.clone();
    *rebuild_popover.borrow_mut() = Some(Box::new(move || {
        let state = *layout_state_clone2.borrow();
    let popover_box = GtkBox::new(Orientation::Horizontal, 6);
        popover_box.set_margin_top(8);
        popover_box.set_margin_bottom(8);
        popover_box.set_margin_start(8);
        popover_box.set_margin_end(8);

        // Button 1: Close view (show only editor)
        if matches!(state, LayoutState::Split | LayoutState::ViewOnly | LayoutState::ViewWinOnly) {
            let btn1 = Button::new();
            let img = gtk4::Image::from_file("src/assets/icons/only_editor.svg");
            img.set_pixel_size(16);
            btn1.set_child(Some(&img));
            btn1.set_tooltip_text(Some("Close view (show only editor)"));
            btn1.set_halign(Align::Start);
            let layout_state = layout_state_clone2.clone();
            let weak_rebuild = weak_rebuild_popover.clone();
            btn1.connect_clicked(move |_| {
                let prev = *layout_state.borrow();
                let next = LayoutState::EditorOnly;
                println!("app state {} -> app state {}", layout_state_label(prev), layout_state_label(next));
                *layout_state.borrow_mut() = next;
                if let Some(rc) = weak_rebuild.upgrade() {
                    if let Some(ref rebuild) = *rc.borrow() { rebuild(); }
                }
            });
            popover_box.append(&btn1);
        }

        // Button 2: Close editor (show only view)
        if matches!(state, LayoutState::Split | LayoutState::EditorOnly | LayoutState::EditorAndWin) {
            let btn2 = Button::new();
            let img = gtk4::Image::from_file("src/assets/icons/only_preview.svg");
            img.set_pixel_size(16);
            btn2.set_child(Some(&img));
            btn2.set_tooltip_text(Some("Close editor (show only view)"));
            btn2.set_halign(Align::Start);
            let layout_state = layout_state_clone2.clone();
            let weak_rebuild = weak_rebuild_popover.clone();
            btn2.connect_clicked(move |_| {
                let prev = *layout_state.borrow();
                let next = LayoutState::ViewOnly;
                println!("app state {} -> app state {}", layout_state_label(prev), layout_state_label(next));
                *layout_state.borrow_mut() = next;
                if let Some(rc) = weak_rebuild.upgrade() {
                    if let Some(ref rebuild) = *rc.borrow() { rebuild(); }
                }
            });
            popover_box.append(&btn2);
        }

        // Button 3: Close view (open view in separate window)
        if matches!(state, LayoutState::Split | LayoutState::ViewOnly) {
            let btn3 = Button::new();
            let img = gtk4::Image::from_file("src/assets/icons/detatch.svg");
            img.set_pixel_size(16);
            btn3.set_child(Some(&img));
            btn3.set_tooltip_text(Some("Open view in separate window"));
            btn3.set_halign(Align::Start);
            let layout_state = layout_state_clone2.clone();
            let weak_rebuild = weak_rebuild_popover.clone();
            btn3.connect_clicked(move |_| {
                let prev = *layout_state.borrow();
                let next = LayoutState::EditorAndWin;
                println!("app state {} -> app state {}", layout_state_label(prev), layout_state_label(next));
                *layout_state.borrow_mut() = next;
                if let Some(rc) = weak_rebuild.upgrade() {
                    if let Some(ref rebuild) = *rc.borrow() { rebuild(); }
                }
            });
            popover_box.append(&btn3);
        }

        // Button 4: Restore default split view
        if !matches!(state, LayoutState::Split) {
            let btn4 = Button::new();
            let img = gtk4::Image::from_file("src/assets/icons/editor_preview.svg");
            img.set_pixel_size(16);
            btn4.set_child(Some(&img));
            btn4.set_tooltip_text(Some("Restore default split view"));
            btn4.set_halign(Align::Start);
            let layout_state = layout_state_clone2.clone();
            let weak_rebuild = weak_rebuild_popover.clone();
            btn4.connect_clicked(move |_| {
                let prev = *layout_state.borrow();
                let next = LayoutState::Split;
                println!("app state {} -> app state {}", layout_state_label(prev), layout_state_label(next));
                *layout_state.borrow_mut() = next;
                if let Some(rc) = weak_rebuild.upgrade() {
                    if let Some(ref rebuild) = *rc.borrow() { rebuild(); }
                }
            });
            popover_box.append(&btn4);
        }

    // Set the new child; GTK4 will replace the old one automatically
    popover_clone.set_child(Some(&popover_box));
    popover_clone.set_has_arrow(true);
    popover_clone.set_position(gtk4::PositionType::Bottom);
    popover_clone.set_autohide(true);
    }) as Box<dyn Fn()>);

    // Initial build
    if let Some(ref rebuild) = *rebuild_popover.borrow() { rebuild(); }

    let popover_ref = Rc::new(popover);
    let rebuild_popover_for_btn = rebuild_popover.clone();
    let popover_for_btn = popover_ref.clone();
    layout_menu_btn.connect_clicked(move |btn| {
        if let Some(ref rebuild) = *rebuild_popover_for_btn.borrow() { rebuild(); }
        // Anchor the popover to the button's allocation
        let alloc = btn.allocation();
        popover_for_btn.set_pointing_to(Some(&alloc));
        popover_for_btn.popup();
    });
    titlebar.append(&layout_menu_btn);

    // Spacer (24px) between functional buttons and window controls
    let spacer_between = GtkBox::new(Orientation::Horizontal, 0);
    spacer_between.set_size_request(1, 1);
    titlebar.append(&spacer_between);

    // --- Window controls (rightmost) ---

    // Minimize button using provided minimize SVG asset
    let btn_min = Button::new();
    btn_min.set_tooltip_text(Some("Minimize"));
    btn_min.set_valign(Align::Center);
    btn_min.set_margin_start(1);
    btn_min.set_margin_end(1);
    btn_min.set_focusable(false);
    btn_min.set_can_focus(false);
    btn_min.set_has_frame(false);
    btn_min.add_css_class("topright-btn");
    btn_min.add_css_class("window-control-btn");
    let img_min = gtk4::Image::from_file("src/assets/icons/minimize.svg");
    img_min.set_pixel_size(16);
    btn_min.set_child(Some(&img_min));
    titlebar.append(&btn_min);

    // Maximize/Exit-maximize button with dynamic icon
    let btn_max = Button::new();
    btn_max.set_tooltip_text(Some("Maximize"));
    btn_max.set_valign(Align::Center);
    btn_max.set_margin_start(1);
    btn_max.set_margin_end(1);
    btn_max.set_focusable(false);
    btn_max.set_can_focus(false);
    btn_max.set_has_frame(false);
    btn_max.add_css_class("topright-btn");
    btn_max.add_css_class("window-control-btn");

    // Helper to set icon based on window state (always create a new Image)
    fn update_max_icon(window: &gtk4::ApplicationWindow, btn: &Button) {
        if window.is_maximized() {
            btn.set_tooltip_text(Some("Exit Fullscreen"));
            let img_exit = gtk4::Image::from_file("src/assets/icons/fullscreen_exit.svg");
            img_exit.set_pixel_size(16);
            btn.set_child(Some(&img_exit));
        } else {
            btn.set_tooltip_text(Some("Maximize"));
            let img_full = gtk4::Image::from_file("src/assets/icons/fullscreen.svg");
            img_full.set_pixel_size(16);
            btn.set_child(Some(&img_full));
        }
    }

    // Initial icon
    update_max_icon(window, &btn_max);

    // Toggle maximize/unmaximize and update icon after state change
    let window_clone = window.clone();
    let btn_max_clone2 = btn_max.clone();
    btn_max.connect_clicked(move |_| {
        if window_clone.is_maximized() {
            window_clone.unmaximize();
        } else {
            window_clone.maximize();
        }
        // Defer icon update to allow window state to change
        let win = window_clone.clone();
        let btn = btn_max_clone2.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            update_max_icon(&win, &btn);
            ControlFlow::Break
        });
    });
    titlebar.append(&btn_max);

    // Close button using provided SVG asset
    let btn_close = Button::new();
    btn_close.set_tooltip_text(Some("Close"));
    btn_close.set_valign(Align::Center);
    btn_close.set_margin_start(1);
    btn_close.set_margin_end(10);
    btn_close.set_focusable(false);
    btn_close.set_can_focus(false);
    btn_close.set_has_frame(false);
    btn_close.add_css_class("topright-btn");
    btn_close.add_css_class("window-control-btn");
    let img_close = gtk4::Image::from_file("src/assets/icons/close.svg");
    img_close.set_pixel_size(16);
    btn_close.set_child(Some(&img_close));
    titlebar.append(&btn_close);

    // Connect window control actions
    let win_clone = window.clone();
    btn_min.connect_clicked(move |_| { win_clone.minimize(); });
    let win_clone = window.clone();
    btn_close.connect_clicked(move |_| { win_clone.close(); });

    // Add the titlebar to the WindowHandle
    handle.set_child(Some(&titlebar));
    handle
}