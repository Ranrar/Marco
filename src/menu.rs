/// Sets the height of the menu widget (Box or similar)
pub fn set_menu_height(menu_box: &gtk4::Box, height: i32) {
    menu_box.set_height_request(height);
}
// Helper to convert LayoutState to a human-readable string
fn layout_state_label(state: LayoutState) -> &'static str {
    match state {
        LayoutState::Split => "standard split view",
        LayoutState::EditorOnly => "editor view only",
        LayoutState::ViewOnly => "preview view only",
        LayoutState::EditorAndWin => "editor + view in separate window",
        LayoutState::ViewWinOnly => "view in separate window only",
    }
}
use std::cell::RefCell;
use std::rc::{Rc, Weak};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum LayoutState {
    Split,          // A: Editor + View (default)
    EditorOnly,     // B: Editor only
    ViewOnly,       // C: View only
    EditorAndWin,   // D: Editor + view in separate window
    ViewWinOnly,    // E: View in separate window only
}
// Removed icondata imports
use gtk4::gdk_pixbuf;
use gtk4::gdk::Texture;
use gtk4::prelude::PixbufLoaderExt;

/// Helper to create a GTK4 Image from icondata SVG with a given fill color
fn svg_icon_image(icon: &'static icondata_core::IconData) -> gtk4::Image {
    // Compose SVG string from IconData fields
    let width = icon.width.unwrap_or("24");
    let height = icon.height.unwrap_or("24");
    let view_box = icon.view_box.unwrap_or("0 0 24 24");
    // Replace all fill attributes in the path data with fill="currentColor"
    let re = regex::Regex::new(r#"fill=\"[^\"]*\""#).unwrap();
    let data_colored = re.replace_all(icon.data, "fill=\"currentColor\"").to_string();
    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' width='{w}' height='{h}' viewBox='{vb}' fill='currentColor'>{data}</svg>",
        w=width, h=height, vb=view_box, data=data_colored
    );
    let loader = gdk_pixbuf::PixbufLoader::with_type("svg").unwrap();
    loader.write(svg.as_bytes()).unwrap();
    loader.close().unwrap();
    let pixbuf = loader.pixbuf().unwrap();
    let texture = Texture::for_pixbuf(&pixbuf);
    gtk4::Image::from_paintable(Some(&texture))
}
// use gtk4::prelude::*;


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

/// Returns a WindowHandle containing the custom VS Code–like titlebar, including the menu bar and all controls.
pub fn create_custom_titlebar(window: &gtk4::ApplicationWindow) -> WindowHandle {
    // NOTE: For include_bytes! to work, these files must exist at src/assets/graphies/ relative to the project root.
    // Use the window background color for the icon (fallback to #222 if not available)
    // color is now controlled by CSS, not a hardcoded value
    // Load custom CSS theme from file
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
    let icon = Image::from_file("src/assets/graphies/favicon.png");
    icon.set_pixel_size(20);
    icon.set_halign(Align::Start);
    icon.set_margin_start(6);
    icon.set_margin_end(6);
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

    // Helper to generate IoEyeOutline and IoEyeSharp SVGs programmatically using svg crate
    fn build_eye_svg(outline: bool, color: &str) -> gtk4::Image {
        use svg::{Document, node::element::Path};
        use gtk4::glib::Bytes;
        use gtk4::gio::MemoryInputStream;
        use gtk4::gdk_pixbuf::Pixbuf;
        use gtk4::gdk::Texture;
        use gtk4::Image;
        // IoEyeOutline: outline only (stroke, no fill)
        let outline_path = "M1 12C2.73 7.61 7.11 4.5 12 4.5s9.27 3.11 11 7.5c-1.73 4.39-6.11 7.5-11 7.5S2.73 16.39 1 12zm11 5a5 5 0 1 0 0-10 5 5 0 0 0 0 10zm0-2a3 3 0 1 1 0-6 3 3 0 0 1 0 6z";
        // IoEyeSharp: filled only (fill, no stroke)
        let sharp_path = "M12 4.5C7.11 4.5 2.73 7.61 1 12c1.73 4.39 6.11 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6.11-7.5-11-7.5zm0 13a5 5 0 1 1 0-10 5 5 0 0 1 0 10z";
        let doc = if outline {
            // IoEyeOutline: three paths, all stroke, no fill, extra thin lines
            let outer = Path::new()
                .set("d", "M1 12C2.73 7.61 7.11 4.5 12 4.5s9.27 3.11 11 7.5c-1.73 4.39-6.11 7.5-11 7.5S2.73 16.39 1 12z")
                .set("fill", "none")
                .set("stroke", color)
                .set("stroke-width", 0.8);
            let eye = Path::new()
                .set("d", "M12 17a5 5 0 1 0 0-10 5 5 0 0 0 0 10z")
                .set("fill", "none")
                .set("stroke", color)
                .set("stroke-width", 0.8);
            let pupil = Path::new()
                .set("d", "M12 15a3 3 0 1 1 0-6 3 3 0 0 1 0 6z")
                .set("fill", "none")
                .set("stroke", color)
                .set("stroke-width", 0.8);
            Document::new()
                .set("viewBox", (0, 0, 24, 24))
                .add(outer)
                .add(eye)
                .add(pupil)
        } else {
            // IoEyeSharp: outline only (stroke, no fill), using its own path
            let path = Path::new()
                .set("d", "M12 4.5C7.11 4.5 2.73 7.61 1 12c1.73 4.39 6.11 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6.11-7.5-11-7.5zm0 13a5 5 0 1 1 0-10 5 5 0 0 1 0 10z")
                .set("fill", "none")
                .set("stroke", color)
                .set("stroke-width", 0.8);
            Document::new()
                .set("viewBox", (0, 0, 24, 24))
                .add(path)
        };
        let svg_str = doc.to_string();
        let gbytes = Bytes::from(svg_str.as_bytes());
        let stream = MemoryInputStream::from_bytes(&gbytes);
        let pixbuf = Pixbuf::from_stream(&stream, gio::Cancellable::NONE).unwrap();
        let texture = Texture::for_pixbuf(&pixbuf);
        let img = Image::from_paintable(Some(&texture));
        img.set_pixel_size(20);
        img
    }

    // Shared image for layout button, using svg crate
    let img_menu = Rc::new(RefCell::new(build_eye_svg(true, "white")));
    layout_menu_btn.set_child(Some(img_menu.borrow().upcast_ref::<gtk4::Widget>()));

    // Hover effect for layout button
    let img_menu_hover = img_menu.clone();
    let layout_state_for_hover = layout_state.clone();
    let motion_controller_menu = EventControllerMotion::new();
    motion_controller_menu.connect_enter(move |_, _, _| {
        let state = *layout_state_for_hover.borrow();
        let outline = matches!(state, LayoutState::Split);
        let new_img = build_eye_svg(outline, "#2196f3");
        if let Some(texture) = new_img.paintable() {
            img_menu_hover.borrow().set_paintable(Some(&texture));
        }
    });
    let img_menu_leave = img_menu.clone();
    let layout_state_for_leave = layout_state.clone();
    motion_controller_menu.connect_leave(move |_| {
        // Always use the latest layout state
        let state = *layout_state_for_leave.borrow();
        let outline = matches!(state, LayoutState::Split);
        let new_img = build_eye_svg(outline, "white");
        if let Some(texture) = new_img.paintable() {
            img_menu_leave.borrow().set_paintable(Some(&texture));
        }
    });
    layout_menu_btn.add_controller(motion_controller_menu);

    // Helper to update icon after state change
    let img_menu_state = img_menu.clone();
    let update_layout_icon = move |state: LayoutState| {
        let outline = matches!(state, LayoutState::Split);
        let new_img = build_eye_svg(outline, "white");
        if let Some(texture) = new_img.paintable() {
            img_menu_state.borrow().set_paintable(Some(&texture));
        }
    };

    // Remove duplicate layout_state initialization (already defined above)

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
        let popover_box = GtkBox::new(Orientation::Vertical, 6);
        popover_box.set_margin_top(8);
        popover_box.set_margin_bottom(8);
        popover_box.set_margin_start(8);
        popover_box.set_margin_end(8);

        // Button 1: Close view (show only editor)
        if matches!(state, LayoutState::Split | LayoutState::ViewOnly | LayoutState::ViewWinOnly) {
            let btn1 = Button::new();
            btn1.set_label("Close view (show only editor)");
            btn1.set_halign(Align::Start);
            let layout_state = layout_state_clone2.clone();
            let weak_rebuild = weak_rebuild_popover.clone();
            let update_layout_icon = update_layout_icon.clone();
            btn1.connect_clicked(move |_| {
                let prev = *layout_state.borrow();
                let next = LayoutState::EditorOnly;
                println!("app state {} -> app state {}", layout_state_label(prev), layout_state_label(next));
                *layout_state.borrow_mut() = next;
                update_layout_icon(next);
                if let Some(rc) = weak_rebuild.upgrade() {
                    if let Some(ref rebuild) = *rc.borrow() { rebuild(); }
                }
            });
            popover_box.append(&btn1);
        }

        // Button 2: Close editor (show only view)
        if matches!(state, LayoutState::Split | LayoutState::EditorOnly | LayoutState::EditorAndWin) {
            let btn2 = Button::new();
            btn2.set_label("Close editor (show only view)");
            btn2.set_halign(Align::Start);
            let layout_state = layout_state_clone2.clone();
            let weak_rebuild = weak_rebuild_popover.clone();
            let update_layout_icon = update_layout_icon.clone();
            btn2.connect_clicked(move |_| {
                let prev = *layout_state.borrow();
                let next = LayoutState::ViewOnly;
                println!("app state {} -> app state {}", layout_state_label(prev), layout_state_label(next));
                *layout_state.borrow_mut() = next;
                update_layout_icon(next);
                if let Some(rc) = weak_rebuild.upgrade() {
                    if let Some(ref rebuild) = *rc.borrow() { rebuild(); }
                }
            });
            popover_box.append(&btn2);
        }

        // Button 3: Close view (open view in separate window)
        if matches!(state, LayoutState::Split | LayoutState::ViewOnly) {
            let btn3 = Button::new();
            btn3.set_label("Open view in separate window");
            btn3.set_halign(Align::Start);
            let layout_state = layout_state_clone2.clone();
            let weak_rebuild = weak_rebuild_popover.clone();
            let update_layout_icon = update_layout_icon.clone();
            btn3.connect_clicked(move |_| {
                let prev = *layout_state.borrow();
                let next = LayoutState::EditorAndWin;
                println!("app state {} -> app state {}", layout_state_label(prev), layout_state_label(next));
                *layout_state.borrow_mut() = next;
                update_layout_icon(next);
                if let Some(rc) = weak_rebuild.upgrade() {
                    if let Some(ref rebuild) = *rc.borrow() { rebuild(); }
                }
            });
            popover_box.append(&btn3);
        }

        // Button 4: Restore default split view
        if !matches!(state, LayoutState::Split) {
            let btn4 = Button::new();
            btn4.set_label("Restore default split view");
            btn4.set_halign(Align::Start);
            let layout_state = layout_state_clone2.clone();
            let weak_rebuild = weak_rebuild_popover.clone();
            let update_layout_icon = update_layout_icon.clone();
            btn4.connect_clicked(move |_| {
                let prev = *layout_state.borrow();
                let next = LayoutState::Split;
                println!("app state {} -> app state {}", layout_state_label(prev), layout_state_label(next));
                *layout_state.borrow_mut() = next;
                update_layout_icon(next);
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

        // Workaround: If the popover is visible, hide and re-show it to force resize
        if popover_clone.is_visible() {
            popover_clone.popdown();
            popover_clone.popup();
        }
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
    spacer_between.set_size_request(4, 1);
    titlebar.append(&spacer_between);

    // --- Window controls (rightmost) ---
    // Window controls using new SVGs from src/assets/graphies
    // Minimize button
    let btn_min = Button::new();
    btn_min.set_tooltip_text(Some("Minimize"));
    btn_min.set_valign(Align::Center);
    btn_min.set_margin_start(2);
    btn_min.set_margin_end(2);
    btn_min.set_focusable(false);
    btn_min.set_can_focus(false);
    btn_min.set_has_frame(false);
    btn_min.add_css_class("topright-btn");
    btn_min.add_css_class("window-control-btn");
    // Minimize button using svg crate for runtime fill color manipulation
    use svg::{Document, node::element::Path};
    use gtk4::glib::Bytes;
    use gtk4::gio::MemoryInputStream;
    use gtk4::gdk_pixbuf::Pixbuf;
    use gtk4::gdk::Texture;
    // use gtk4::Image; (already imported above)
    use gtk4::prelude::*;
    // Minimize icon: bold line at bottom, full width
    let min_path_data = "M96 416h320v32H96z";
    let build_min_svg = move |fill: &str| {
        let path = Path::new()
            .set("d", min_path_data)
            .set("fill", fill);
        let doc = Document::new()
            .set("viewBox", (0, 0, 512, 512))
            .add(path);
        let svg_str = doc.to_string();
        let gbytes = Bytes::from(svg_str.as_bytes());
        let stream = MemoryInputStream::from_bytes(&gbytes);
        let pixbuf = Pixbuf::from_stream(&stream, gio::Cancellable::NONE).unwrap();
        let texture = Texture::for_pixbuf(&pixbuf);
        let img = Image::from_paintable(Some(&texture));
        img.set_pixel_size(20);
        img
    };
    let img_min = build_min_svg("white");
    btn_min.set_child(Some(&img_min));
    use gtk4::EventControllerMotion;
    let motion_controller = EventControllerMotion::new();
    let img_min_clone = img_min.clone();
    motion_controller.connect_enter(move |_, _, _| {
        let new_img = build_min_svg("#2196f3");
        if let Some(texture) = new_img.paintable() {
            img_min_clone.set_paintable(Some(&texture));
        }
    });
    let img_min_clone2 = img_min.clone();
    motion_controller.connect_leave(move |_| {
        let new_img = build_min_svg("white");
        if let Some(texture) = new_img.paintable() {
            img_min_clone2.set_paintable(Some(&texture));
        }
    });
    btn_min.add_controller(motion_controller);
    titlebar.append(&btn_min);

    // Maximize button using svg crate for runtime fill color manipulation
    let btn_max = Button::new();
    btn_max.set_tooltip_text(Some("Maximize"));
    btn_max.set_valign(Align::Center);
    btn_max.set_margin_start(2);
    btn_max.set_margin_end(2);
    btn_max.set_focusable(false);
    btn_max.set_can_focus(false);
    btn_max.set_has_frame(false);
    btn_max.add_css_class("topright-btn");
    btn_max.add_css_class("window-control-btn");
    // Maximize icon: single outline rectangle, no fill
    let build_max_svg = move |stroke: &str| {
        let rect = svg::node::element::Rectangle::new()
            .set("x", 96)
            .set("y", 96)
            .set("width", 320)
            .set("height", 320)
            .set("fill", "none")
            .set("stroke", stroke)
            .set("stroke-width", 32);
        let doc = Document::new()
            .set("viewBox", (0, 0, 512, 512))
            .add(rect);
        let svg_str = doc.to_string();
        let gbytes = Bytes::from(svg_str.as_bytes());
        let stream = MemoryInputStream::from_bytes(&gbytes);
        let pixbuf = Pixbuf::from_stream(&stream, gio::Cancellable::NONE).unwrap();
        let texture = Texture::for_pixbuf(&pixbuf);
        let img = Image::from_paintable(Some(&texture));
        img.set_pixel_size(20);
        img
    };
    let img_max = build_max_svg("white");
    btn_max.set_child(Some(&img_max));
    let motion_controller_max = EventControllerMotion::new();
    let img_max_clone = img_max.clone();
    motion_controller_max.connect_enter(move |_, _, _| {
        let new_img = build_max_svg("#2196f3");
        if let Some(texture) = new_img.paintable() {
            img_max_clone.set_paintable(Some(&texture));
        }
    });
    let img_max_clone2 = img_max.clone();
    motion_controller_max.connect_leave(move |_| {
        let new_img = build_max_svg("white");
        if let Some(texture) = new_img.paintable() {
            img_max_clone2.set_paintable(Some(&texture));
        }
    });
    btn_max.add_controller(motion_controller_max);
    titlebar.append(&btn_max);

    // Close button using svg crate for runtime fill color manipulation
    let btn_close = Button::new();
    btn_close.set_tooltip_text(Some("Close"));
    btn_close.set_valign(Align::Center);
    btn_close.set_margin_start(2);
    btn_close.set_margin_end(12);
    btn_close.set_focusable(false);
    btn_close.set_can_focus(false);
    btn_close.set_has_frame(false);
    btn_close.add_css_class("topright-btn");
    btn_close.add_css_class("window-control-btn");
    // Close icon: two bold diagonal lines, full canvas
    let build_close_svg = move |stroke: &str| {
        let line1 = svg::node::element::Line::new()
            .set("x1", 128)
            .set("y1", 128)
            .set("x2", 384)
            .set("y2", 384)
            .set("stroke", stroke)
            .set("stroke-width", 32)
            .set("stroke-linecap", "round");
        let line2 = svg::node::element::Line::new()
            .set("x1", 384)
            .set("y1", 128)
            .set("x2", 128)
            .set("y2", 384)
            .set("stroke", stroke)
            .set("stroke-width", 32)
            .set("stroke-linecap", "round");
        let doc = Document::new()
            .set("viewBox", (0, 0, 512, 512))
            .add(line1)
            .add(line2);
        let svg_str = doc.to_string();
        let gbytes = Bytes::from(svg_str.as_bytes());
        let stream = MemoryInputStream::from_bytes(&gbytes);
        let pixbuf = Pixbuf::from_stream(&stream, gio::Cancellable::NONE).unwrap();
        let texture = Texture::for_pixbuf(&pixbuf);
        let img = Image::from_paintable(Some(&texture));
        img.set_pixel_size(20);
        img
    };
    let img_close = build_close_svg("white");
    btn_close.set_child(Some(&img_close));
    let motion_controller_close = EventControllerMotion::new();
    let img_close_clone = img_close.clone();
    motion_controller_close.connect_enter(move |_, _, _| {
        let new_img = build_close_svg("#f44336");
        if let Some(texture) = new_img.paintable() {
            img_close_clone.set_paintable(Some(&texture));
        }
    });
    let img_close_clone2 = img_close.clone();
    motion_controller_close.connect_leave(move |_| {
        let new_img = build_close_svg("white");
        if let Some(texture) = new_img.paintable() {
            img_close_clone2.set_paintable(Some(&texture));
        }
    });
    btn_close.add_controller(motion_controller_close);
    titlebar.append(&btn_close);

    // Connect window control actions
    let win_clone = window.clone();
    btn_min.connect_clicked(move |_| { win_clone.minimize(); });
    let win_clone = window.clone();
    btn_max.connect_clicked(move |_| {
        if win_clone.is_maximized() { win_clone.unmaximize(); } else { win_clone.maximize(); }
    });
    let win_clone = window.clone();
    btn_close.connect_clicked(move |_| { win_clone.close(); });

    // Add the titlebar to the WindowHandle
    handle.set_child(Some(&titlebar));
    handle
}