use marco_core::logic::layoutstate::{layout_state_label, LayoutState};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use gtk4::gio;
use gtk4::{self, prelude::*, Align, Box as GtkBox, Button, Label, WindowHandle, Orientation};
use log::trace;

// Type alias for the complex rebuild callback type
type RebuildCallback = Box<dyn Fn()>;
type RebuildPopover = Rc<RefCell<Option<RebuildCallback>>>;
type WeakRebuildPopover = Weak<RefCell<Option<RebuildCallback>>>;

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

/// Returns a WindowHandle containing the custom menu bar and all controls.
/// Returns a WindowHandle and the central title `Label` so callers can update the
/// document title (and modification marker) dynamically.
pub fn create_custom_titlebar(
    window: &gtk4::ApplicationWindow,
) -> (WindowHandle, Label, gio::Menu) {
    // Get the asset directory for dynamic path resolution
    use marco_core::logic::paths::get_asset_dir_checked;
    let asset_dir = get_asset_dir_checked();

    // Create WindowHandle wrapper for proper window dragging
    let handle = WindowHandle::new();
    
    // Use GTK4 HeaderBar for proper title centering
    let headerbar = gtk4::HeaderBar::new();
    headerbar.add_css_class("titlebar");
    headerbar.set_show_title_buttons(false); // We'll add custom window controls
    
    // App icon (left) - uses dynamic asset directory path
    let icon_path = asset_dir
        .unwrap_or_else(|_| std::path::PathBuf::from("src/assets"))
        .join("icons/favicon.png");
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
    // Set initial tooltip to the human-readable current layout label
    layout_menu_btn.set_tooltip_text(Some(layout_state_label(*layout_state.borrow())));

    // Use icon font glyph for layout button (IcoMoon '1' = split_scene_left)
    let layout_label = gtk4::Label::new(None);
    layout_label.set_markup(&format!("<span font_family='icomoon'>{}</span>", "\u{31}"));
    layout_label.set_valign(Align::Center);
    layout_label.add_css_class("icon-font");
    layout_menu_btn.add_css_class("window-control-btn");
    layout_menu_btn.set_child(Some(&layout_label));

    // Helper to (re)build the popover content based on state
    let popover = Popover::new();
    // Attach the popover to the layout button for proper positioning
    popover.set_parent(&layout_menu_btn);
    // Remove unused duplicate clone

    let rebuild_popover: RebuildPopover = Rc::new(RefCell::new(None));

    let weak_rebuild_popover: WeakRebuildPopover = Rc::downgrade(&rebuild_popover);
    let layout_state_clone2 = layout_state.clone(); // Used for popover logic
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
                let next = LayoutState::EditorOnly;
                *layout_state.borrow_mut() = next;
                if let Some(rc) = weak_rebuild.upgrade() {
                    if let Some(ref rebuild) = *rc.borrow() {
                        rebuild();
                    }
                }
            });
            popover_box.append(&btn1);
        }

        // Button 2: Close editor (show only view)
        if matches!(
            state,
            LayoutState::DualView | LayoutState::EditorOnly | LayoutState::EditorAndViewSeparate
        ) {
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
                let next = LayoutState::ViewOnly;
                *layout_state.borrow_mut() = next;
                if let Some(rc) = weak_rebuild.upgrade() {
                    if let Some(ref rebuild) = *rc.borrow() {
                        rebuild();
                    }
                }
            });
            popover_box.append(&btn2);
        }

        // Button 3: Close view (open view in separate window)
        if matches!(state, LayoutState::DualView | LayoutState::ViewOnly) {
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
                let next = LayoutState::EditorAndViewSeparate;
                *layout_state.borrow_mut() = next;
                if let Some(rc) = weak_rebuild.upgrade() {
                    if let Some(ref rebuild) = *rc.borrow() {
                        rebuild();
                    }
                }
            });
            popover_box.append(&btn3);
        }

        // Button 4: Restore default split view
        if !matches!(state, LayoutState::DualView) {
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
                let next = LayoutState::DualView;
                *layout_state.borrow_mut() = next;
                if let Some(rc) = weak_rebuild.upgrade() {
                    if let Some(ref rebuild) = *rc.borrow() {
                        rebuild();
                    }
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
    let initial_glyph = if window.is_maximized() {
        "\u{35}"
    } else {
        "\u{36}"
    };
    max_label.set_markup(&format!(
        "<span font_family='icomoon'>{}</span>",
        initial_glyph
    ));
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

    // Add controls to headerbar from right to left (pack_end order)
    // Since pack_end adds from right to left, we add in reverse visual order:
    // First add window controls (they'll be rightmost)
    headerbar.pack_end(&btn_close);        // Rightmost
    headerbar.pack_end(&btn_max_toggle);   // Middle
    headerbar.pack_end(&btn_min);          // Left of window controls
    // Then add layout button (it will be to the left of window controls)
    headerbar.pack_end(&layout_menu_btn);  // Left of minimize button

    // Minimize and close actions
    let win_clone = window.clone();
    btn_min.connect_clicked(move |_| {
        win_clone.minimize();
        trace!("audit: window minimize clicked");
    });
    // When close is pressed, activate the application's quit action so
    // the unified quit flow (including FileOperations::quit_async) runs.
    let win_for_close = window.clone();
    btn_close.connect_clicked(move |_| {
        if let Some(app) = win_for_close.application() {
            // Activate the app-level action 'app.quit' which is registered in main.rs
            if let Some(action) = app.lookup_action("quit") {
                action.activate(None);
            } else {
                // Fallback: close the window if action not found
                win_for_close.close();
            }
        } else {
            // Fallback: close the window if no application is associated
            win_for_close.close();
        }
        trace!("audit: window close clicked");
    });

    // Click toggles window state and updates glyph immediately
    let label_for_toggle = max_label.clone();
    let window_for_toggle = window.clone();
    btn_max_toggle.connect_clicked(move |_| {
        if window_for_toggle.is_maximized() {
            window_for_toggle.unmaximize();
            label_for_toggle
                .set_markup(&format!("<span font_family='icomoon'>{}</span>", "\u{36}"));
        } else {
            window_for_toggle.maximize();
            label_for_toggle
                .set_markup(&format!("<span font_family='icomoon'>{}</span>", "\u{35}"));
        }
        trace!("audit: window maximize/restore clicked");
    });

    // Keep glyph in sync if window is maximized/unmaximized externally
    let label_for_notify = max_label.clone();
    window.connect_notify_local(Some("is-maximized"), move |w, _| {
        if w.is_maximized() {
            label_for_notify
                .set_markup(&format!("<span font_family='icomoon'>{}</span>", "\u{35}"));
        } else {
            label_for_notify
                .set_markup(&format!("<span font_family='icomoon'>{}</span>", "\u{36}"));
        }
    });

    // Add the HeaderBar to the WindowHandle
    handle.set_child(Some(&headerbar));
    (handle, title_label, recent_menu)
}
