use icondata::{BiDockLeftSolid, BiDockRightSolid, MdiMicrosoftXboxControllerView, BsLayoutSplit};
use icondata::{VsChromeMaximize, VsChromeMinimize, VsClose};
use gtk4::{gdk_pixbuf, EventControllerMotion};
use gtk4::gdk::Texture;
use gtk4::prelude::PixbufLoaderExt;

/// Helper to create a GTK4 Image from icondata SVG with a given fill color
fn svg_icon_image(icon: &'static icondata_core::IconData, fill: &str) -> gtk4::Image {
    // Compose SVG string from IconData fields
    let width = icon.width.unwrap_or("24");
    let height = icon.height.unwrap_or("24");
    let view_box = icon.view_box.unwrap_or("0 0 24 24");
    // Replace all fill attributes in the path data with the desired color
    let re = regex::Regex::new(r#"fill=\"[^\"]*\""#).unwrap();
    let data_colored = re.replace_all(icon.data, &format!("fill=\"{}\"", fill)).to_string();
    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' width='{w}' height='{h}' viewBox='{vb}' fill='{fill}'>{data}</svg>",
        w=width, h=height, vb=view_box, fill=fill, data=data_colored
    );
    let loader = gdk_pixbuf::PixbufLoader::with_type("svg").unwrap();
    loader.write(svg.as_bytes()).unwrap();
    loader.close().unwrap();
    let pixbuf = loader.pixbuf().unwrap();
    let texture = Texture::for_pixbuf(&pixbuf);
    gtk4::Image::from_paintable(Some(&texture))
}
// use gtk4::prelude::*;


use gtk4::{self, prelude::*, Box as GtkBox, Orientation, Button, Align, WindowHandle};
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
    PopoverMenuBar::from_model(Some(&menu_model))
}

/// Returns a WindowHandle containing the custom VS Code–like titlebar, including the menu bar and all controls.
pub fn create_custom_titlebar(window: &gtk4::ApplicationWindow) -> WindowHandle {
    // Use the window background color for the icon (fallback to #222 if not available)
    let normal_color = "#222";
    let hover_color = "#fff";
    // Load custom CSS theme from file
    use gtk4::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};
    use gtk4::gdk::Display;
    let provider = CssProvider::new();
    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(&display, &provider, STYLE_PROVIDER_PRIORITY_APPLICATION);
    }
    let handle = WindowHandle::new();
    let titlebar = GtkBox::new(Orientation::Horizontal, 0);
    titlebar.set_spacing(0);
    titlebar.set_margin_top(0);
    titlebar.set_margin_bottom(0);
    titlebar.set_margin_start(0);
    titlebar.set_margin_end(0);
    titlebar.set_height_request(32);

    // App icon (left)
    let icon = Image::from_file("src/assets/graphies/favicon.png");
    icon.set_pixel_size(24);
    icon.set_halign(Align::Start);
    icon.set_margin_start(12);
    icon.set_margin_end(12);
    icon.set_valign(Align::Center);
    icon.set_tooltip_text(Some("Marco a markdown composer"));
    titlebar.append(&icon);

    // --- Menu bar (next to title) ---
    let menu_bar = main_menu_structure();
    menu_bar.set_valign(Align::Center);
    titlebar.append(&menu_bar);


    // Spacer (expand to push controls to right)
    let spacer = GtkBox::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    titlebar.append(&spacer);

    // --- Functional buttons (split/editor/preview/detach) ---
    use gtk4::Image;
    // Split view
    let img_split = svg_icon_image(&BsLayoutSplit, normal_color);
    let btn_split = Button::new();
    btn_split.set_child(Some(&img_split));
    btn_split.set_tooltip_text(Some("Split view"));

    // Edit only
    let img_editor = svg_icon_image(&BiDockLeftSolid, normal_color);
    let btn_editor = Button::new();
    btn_editor.set_child(Some(&img_editor));
    btn_editor.set_tooltip_text(Some("Show only editor (hide preview)"));

    // View only
    let img_preview = svg_icon_image(&BiDockRightSolid, normal_color);
    let btn_preview = Button::new();
    btn_preview.set_child(Some(&img_preview));
    btn_preview.set_tooltip_text(Some("Show only preview (hide editor)"));

    // Detach view
    let img_detach = svg_icon_image(&MdiMicrosoftXboxControllerView, normal_color);
    let btn_detach = Button::new();
    btn_detach.set_child(Some(&img_detach));
    btn_detach.set_tooltip_text(Some("Open view in a new window"));

    for btn in [&btn_split, &btn_editor, &btn_preview, &btn_detach] {
        btn.set_valign(Align::Center);
        btn.set_margin_start(2);
        btn.set_margin_end(2);
        btn.set_focusable(false);
        btn.add_css_class("icon-btn");
        titlebar.append(btn);
    }

    // Spacer (24px) between functional buttons and window controls
    let spacer_between = GtkBox::new(Orientation::Horizontal, 0);
    spacer_between.set_size_request(24, 1);
    titlebar.append(&spacer_between);

    // --- Window controls (rightmost) ---
    // Window controls using requested icondata SVGs
    let min_icon = VsChromeMinimize;
    let max_icon = VsChromeMaximize;
    let close_icon = VsClose;

    // Minimize button
    let btn_min = Button::new();
    let img_min = svg_icon_image(min_icon, normal_color);
    btn_min.set_child(Some(&img_min));
    btn_min.set_tooltip_text(Some("Minimize"));
    btn_min.set_valign(Align::Center);
    btn_min.set_margin_start(2);
    btn_min.set_margin_end(2);
    btn_min.set_focusable(false);
    btn_min.add_css_class("icon-btn");
    let img_min_clone = img_min.clone();
    let motion_min = EventControllerMotion::new();
    {
        let img_min = img_min_clone.clone();
        let icon = min_icon;
        motion_min.connect_enter(move |_, _, _| {
            let hover_img = svg_icon_image(icon, hover_color);
            if let Some(paintable) = hover_img.paintable() {
                img_min.set_paintable(Some(&paintable));
            }
        });
    }
    {
        let img_min = img_min_clone.clone();
        let icon = min_icon;
        motion_min.connect_leave(move |_| {
            let normal_img = svg_icon_image(icon, normal_color);
            if let Some(paintable) = normal_img.paintable() {
                img_min.set_paintable(Some(&paintable));
            }
        });
    }
    btn_min.add_controller(motion_min);
    titlebar.append(&btn_min);

    // Maximize button
    let btn_max = Button::new();
    let img_max = svg_icon_image(max_icon, normal_color);
    btn_max.set_child(Some(&img_max));
    btn_max.set_tooltip_text(Some("Maximize"));
    btn_max.set_valign(Align::Center);
    btn_max.set_margin_start(2);
    btn_max.set_margin_end(2);
    btn_max.set_focusable(false);
    btn_max.add_css_class("icon-btn");
    let img_max_clone = img_max.clone();
    let motion_max = EventControllerMotion::new();
    {
        let img_max = img_max_clone.clone();
        let icon = max_icon;
        motion_max.connect_enter(move |_, _, _| {
            let hover_img = svg_icon_image(icon, hover_color);
            if let Some(paintable) = hover_img.paintable() {
                img_max.set_paintable(Some(&paintable));
            }
        });
    }
    {
        let img_max = img_max_clone.clone();
        let icon = max_icon;
        motion_max.connect_leave(move |_| {
            let normal_img = svg_icon_image(icon, normal_color);
            if let Some(paintable) = normal_img.paintable() {
                img_max.set_paintable(Some(&paintable));
            }
        });
    }
    btn_max.add_controller(motion_max);
    titlebar.append(&btn_max);

    // Close button
    let btn_close = Button::new();
    let img_close = svg_icon_image(close_icon, normal_color);
    btn_close.set_child(Some(&img_close));
    btn_close.set_tooltip_text(Some("Close"));
    btn_close.set_valign(Align::Center);
    btn_close.set_margin_start(2);
    btn_close.set_margin_end(12); // Match favicon's left margin
    btn_close.set_focusable(false);
    btn_close.add_css_class("icon-btn");
    let img_close_clone = img_close.clone();
    let motion_close = EventControllerMotion::new();
    {
        let img_close = img_close_clone.clone();
        let icon = close_icon;
        motion_close.connect_enter(move |_, _, _| {
            let hover_img = svg_icon_image(icon, hover_color);
            if let Some(paintable) = hover_img.paintable() {
                img_close.set_paintable(Some(&paintable));
            }
        });
    }
    {
        let img_close = img_close_clone.clone();
        let icon = close_icon;
        motion_close.connect_leave(move |_| {
            let normal_img = svg_icon_image(icon, normal_color);
            if let Some(paintable) = normal_img.paintable() {
                img_close.set_paintable(Some(&paintable));
            }
        });
    }
    btn_close.add_controller(motion_close);
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