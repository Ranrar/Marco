/// Sets the height of the menu widget (Box or similar)
pub fn set_menu_height(menu_box: &gtk4::Box, height: i32) {
    menu_box.set_height_request(height);
}
// Helper to convert LayoutState to a human-readable string
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use crate::logic::menu_items::layoutstate::{LayoutState, layout_state_label};

use gtk4::{self, prelude::*, Box as GtkBox, Button, Align, WindowHandle};
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
    layout_menu_btn.set_margin_start(0);
    layout_menu_btn.set_margin_end(0);
    layout_menu_btn.set_focusable(false);
    layout_menu_btn.set_can_focus(false);
    layout_menu_btn.set_has_frame(false);
    layout_menu_btn.add_css_class("topright-btn");
    // Use same visual style as window control buttons
    layout_menu_btn.add_css_class("window-control-btn");

    // State management (single shared instance)
    let layout_state = Rc::new(RefCell::new(LayoutState::Split));


    // Use icon font glyph for layout button (IcoMoon '1' = split_scene_left)
    let layout_label = gtk4::Label::new(None);
    layout_label.set_markup(&format!("<span font_family='icomoon'>{}</span>", "\u{31}"));
    layout_label.set_valign(Align::Center);
    layout_label.add_css_class("icon-font");
    layout_menu_btn.add_css_class("window-control-btn");
    layout_menu_btn.set_child(Some(&layout_label));

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
            btn1.add_css_class("layout-btn");
            // IcoMoon '3' = only_editor
            let lbl = gtk4::Label::new(None);
            lbl.set_markup(&format!("<span font_family='icomoon'>{}</span>", "\u{33}"));
            lbl.set_valign(Align::Center);
            lbl.add_css_class("layout-state");
            btn1.set_child(Some(&lbl));
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
            btn2.add_css_class("layout-btn");
            // IcoMoon '2' = only_preview
            let lbl = gtk4::Label::new(None);
            lbl.set_markup(&format!("<span font_family='icomoon'>{}</span>", "\u{32}"));
            lbl.set_valign(Align::Center);
            lbl.add_css_class("layout-state");
            btn2.set_child(Some(&lbl));
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
            btn3.add_css_class("layout-btn");
            // IcoMoon '8' = detatch
            let lbl = gtk4::Label::new(None);
            lbl.set_markup(&format!("<span font_family='icomoon'>{}</span>", "\u{38}"));
            lbl.set_valign(Align::Center);
            lbl.add_css_class("layout-state");
            btn3.set_child(Some(&lbl));
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
            btn4.add_css_class("layout-btn");
            // IcoMoon '7' = editor_preview
            let lbl = gtk4::Label::new(None);
            lbl.set_markup(&format!("<span font_family='icomoon'>{}</span>", "\u{37}"));
            lbl.set_valign(Align::Center);
            lbl.add_css_class("layout-state");
            btn4.set_child(Some(&lbl));
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


    use gtk4::Label;

    // Helper to create a button with icon font
    fn icon_button(label_text: &str, tooltip: &str) -> Button {
    let markup = format!("<span font_family='icomoon'>{}</span>", label_text);
    let label = Label::new(None);
    label.set_markup(&markup);
    label.set_valign(Align::Center);
    label.add_css_class("icon-font");
    let btn = Button::new();
    btn.set_child(Some(&label));
    btn.set_tooltip_text(Some(tooltip));
    btn.set_valign(Align::Center);
    btn.set_margin_start(1);
    btn.set_margin_end(1);
    btn.set_focusable(false);
    btn.set_can_focus(false);
    btn.set_has_frame(false);
    btn.add_css_class("topright-btn");
    btn.add_css_class("window-control-btn");
    btn
    }

    // IcoMoon Unicode glyphs for window controls
    // | Unicode | Icon Name             | Description   |
    // |---------|-----------------------|--------------|
    // | \u{34}  | marco-minimize        | Minimize      |
    // | \u{36}  | marco-fullscreen      | Maximize      |
    // | \u{35}  | marco-fullscreen_exit | Exit maximize |
    // | \u{39}  | marco-close           | Close         |

    let btn_min = icon_button("\u{34}", "Minimize");
    let btn_close = icon_button("\u{39}", "Close");

    // Create a single toggle button for maximize/restore and keep its label so we can update it
    let max_label = gtk4::Label::new(None);
    let initial_glyph = if window.is_maximized() { "\u{35}" } else { "\u{36}" };
    max_label.set_markup(&format!("<span font_family='icomoon'>{}</span>", initial_glyph));
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

    // Append controls
    titlebar.append(&btn_min);
    titlebar.append(&btn_max_toggle);
    titlebar.append(&btn_close);

    // Minimize and close actions
    let win_clone = window.clone();
    btn_min.connect_clicked(move |_| { win_clone.minimize(); });
    let win_clone = window.clone();
    btn_close.connect_clicked(move |_| { win_clone.close(); });

    // Click toggles window state and updates glyph immediately
    let label_for_toggle = max_label.clone();
    let window_for_toggle = window.clone();
    btn_max_toggle.connect_clicked(move |_| {
        if window_for_toggle.is_maximized() {
            window_for_toggle.unmaximize();
            label_for_toggle.set_markup(&format!("<span font_family='icomoon'>{}</span>", "\u{36}"));
        } else {
            window_for_toggle.maximize();
            label_for_toggle.set_markup(&format!("<span font_family='icomoon'>{}</span>", "\u{35}"));
        }
    });

    // Keep glyph in sync if window is maximized/unmaximized externally
    let label_for_notify = max_label.clone();
    window.connect_notify_local(Some("is-maximized"), move |w, _| {
        if w.is_maximized() {
            label_for_notify.set_markup(&format!("<span font_family='icomoon'>{}</span>", "\u{35}"));
        } else {
            label_for_notify.set_markup(&format!("<span font_family='icomoon'>{}</span>", "\u{36}"));
        }
    });

    // Add the titlebar to the WindowHandle
    handle.set_child(Some(&titlebar));
    handle
}