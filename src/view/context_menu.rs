use webkit6::prelude::WebViewExt;
use crate::editor::MarkdownEditor;
use crate::language;
use gtk4::prelude::*;
use gtk4::{gdk, gio};

/// Context menu functionality for the preview areas (HTML and Code)
pub struct PreviewContextMenu;

impl PreviewContextMenu {
    /// Creates a new context menu for the preview areas
    pub fn new() -> Self {
        Self
    }

    /// Sets up keyboard accelerators for preview context menu actions
    fn setup_accelerators() {
        // Get the default application and cast to GtkApplication
        if let Some(app) = gtk4::gio::Application::default() {
            if let Some(gtk_app) = app.downcast_ref::<gtk4::Application>() {
                // Set accelerators for actions that have keyboard shortcuts
                gtk_app.set_accels_for_action("preview.undo", &["<Ctrl>z"]);
                gtk_app.set_accels_for_action("preview.redo", &["<Ctrl>y"]);
                gtk_app.set_accels_for_action("preview.cut", &["<Ctrl>x"]);
                gtk_app.set_accels_for_action("preview.copy", &["<Ctrl>c"]);
                gtk_app.set_accels_for_action("preview.paste", &["<Ctrl>v"]);
                gtk_app.set_accels_for_action("preview.delete", &["Delete"]);
                gtk_app.set_accels_for_action("preview.select_all", &["<Ctrl>a"]);
                gtk_app.set_accels_for_action("preview.switch_to_html", &["F5"]);
                gtk_app.set_accels_for_action("preview.switch_to_code", &["F6"]);
            }
        }
    }

    /// Adds CSS styling for the preview context menu
    fn add_preview_styling() {
        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_data(
            "
            /* Preview context menu styling */
            popover.menu button.model {
                padding: 8px 16px;
                margin: 1px;
                border-radius: 4px;
                transition: background-color 0.1s ease;
                font-family: -gtk-system-font;
                font-size: 0.9em;
                min-width: 200px;
            }
            
            popover.menu button.model:hover {
                background-color: alpha(@accent_color, 0.1);
                transition: background-color 0.05s ease;
            }
            
            /* Disabled menu items */
            popover.menu button.model:disabled {
                opacity: 0.5;
                color: alpha(@theme_fg_color, 0.5);
            }
            
            /* Section separators */
            popover.menu separator {
                min-height: 1px;
                background-color: alpha(@borders, 0.3);
                margin: 4px 8px;
                border: none;
                padding: 0;
                opacity: 1;
            }
            
            /* Menu item layout for keyboard shortcuts */
            popover.menu button.model {
                padding: 8px 16px;
                margin: 1px;
            }
            
            /* Menu item text */
            popover.menu button.model label {
                color: @theme_fg_color;
            }
            
            /* Keyboard shortcut text styling */
            popover.menu button.model .accelerator {
                color: alpha(@theme_fg_color, 0.7);
                font-size: 0.85em;
                margin-left: 16px;
            }
            ",
        );

        gtk4::style_context_add_provider_for_display(
            &gdk4::Display::default().unwrap(),
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
        );
    }

    /// Creates the menu model for preview context menu
    fn create_menu_model() -> gio::Menu {
        let menu_model = gio::Menu::new();

        // Create edit section (mostly disabled)
        let edit_section = gio::Menu::new();

        // Create menu items without keyboard shortcuts in labels - they'll be shown automatically
        edit_section.append(Some(&language::tr("menu.undo")), Some("preview.undo"));
        edit_section.append(Some(&language::tr("menu.redo")), Some("preview.redo"));

        menu_model.append_section(None, &edit_section);

        // Create clipboard section
        let clipboard_section = gio::Menu::new();

        clipboard_section.append(Some(&language::tr("menu.cut")), Some("preview.cut"));
        clipboard_section.append(Some(&language::tr("menu.copy")), Some("preview.copy"));
        clipboard_section.append(Some(&language::tr("menu.paste")), Some("preview.paste"));
        clipboard_section.append(Some(&language::tr("menu.delete")), Some("preview.delete"));

        menu_model.append_section(None, &clipboard_section);

        // Create selection section
        let selection_section = gio::Menu::new();

        selection_section.append(
            Some(&language::tr("menu.select_all")),
            Some("preview.select_all"),
        );

        menu_model.append_section(None, &selection_section);

        // Create view section
        let view_section = gio::Menu::new();

        // Preview mode submenu
        let preview_modes = gio::Menu::new();

        preview_modes.append(
            Some(&language::tr("menu.preview_html")),
            Some("preview.switch_to_html"),
        );
        preview_modes.append(
            Some(&language::tr("menu.preview_code")),
            Some("preview.switch_to_code"),
        );

        let change_mode_item =
            gio::MenuItem::new(Some(&language::tr("menu.change_preview_mode")), None);
        change_mode_item.set_submenu(Some(&preview_modes));
        view_section.append_item(&change_mode_item);

        menu_model.append_section(None, &view_section);

        menu_model
    }

    /// Creates the action group with all preview context menu actions
    fn create_action_group<W: IsA<gtk4::Widget> + Clone + 'static>(
        editor: &MarkdownEditor,
        widget: W,
    ) -> gio::SimpleActionGroup {
        let action_group = gio::SimpleActionGroup::new();

        // Helper macro to create actions more concisely
        macro_rules! add_action {
            ($name:expr, $enabled:expr, $closure:expr) => {
                let action = gio::SimpleAction::new($name, None);
                action.set_enabled($enabled);
                action.connect_activate($closure);
                action_group.add_action(&action);
            };
        }

        // Edit actions (disabled)
        add_action!("undo", false, {
            move |_action, _param| {}
        });
        add_action!("redo", false, {
            move |_action, _param| {}
        });
        add_action!("cut", false, {
            move |_action, _param| {}
        });

        // Clipboard actions
        {
            let widget = widget.clone();
            add_action!("copy", true, move |_action, _param| {
                // Copy for TextView
                if let Some(text_view) = widget.clone().dynamic_cast::<gtk4::TextView>().ok() {
                    let buffer = text_view.buffer();
                    if let Some(display) = gdk::Display::default() {
                        let clipboard = display.clipboard();
                        buffer.copy_clipboard(&clipboard);
                    }
                }
                // Copy for WebView
                if let Some(webview) = widget.clone().dynamic_cast::<webkit6::WebView>().ok() {
                    webview.execute_editing_command("Copy");
                }
            });
        }

        add_action!("paste", false, {
            move |_action, _param| {}
        });
        add_action!("delete", false, {
            move |_action, _param| {}
        });

        // Selection actions
        {
            let widget = widget.clone();
            add_action!("select_all", true, move |_action, _param| {
                // Select all for TextView
                if let Some(text_view) = widget.clone().dynamic_cast::<gtk4::TextView>().ok() {
                    let buffer = text_view.buffer();
                    let start = buffer.start_iter();
                    let end = buffer.end_iter();
                    buffer.select_range(&start, &end);
                }
                // Select all for WebView
                if let Some(webview) = widget.clone().dynamic_cast::<webkit6::WebView>().ok() {
                    webview.execute_editing_command("SelectAll");
                }
            });
        }

        // View mode switching actions
        add_action!("switch_to_html", true, {
            let editor = editor.clone();
            move |_action, _param| {
                editor.set_view_mode("html");
                let prefs = crate::settings::get_app_preferences();
                prefs.set_view_mode("html");
            }
        });
        add_action!("switch_to_code", true, {
            let editor = editor.clone();
            move |_action, _param| {
                editor.set_view_mode("code");
                let prefs = crate::settings::get_app_preferences();
                prefs.set_view_mode("code");
            }
        });

        action_group
    }

    /// Sets up the right-click gesture on a given widget (HTML or Code preview)
    pub fn setup_gesture_for_widget<W>(&self, widget: &W, editor: &MarkdownEditor)
    where
        W: IsA<gtk4::Widget> + Clone + 'static,
    {
        let menu_model = Self::create_menu_model();
        let popover =
            gtk4::PopoverMenu::from_model_full(&menu_model, gtk4::PopoverMenuFlags::NESTED);
        popover.set_autohide(true);
        popover.set_has_arrow(true);

        let action_group = Self::create_action_group(editor, widget.clone());
        popover.insert_action_group("preview", Some(&action_group));

        Self::add_preview_styling();
        Self::setup_accelerators();

        let gesture = gtk4::GestureClick::new();
        gesture.set_button(3); // Right mouse button
        gesture.set_exclusive(true);
        gesture.set_propagation_phase(gtk4::PropagationPhase::Capture);

        popover.set_parent(widget);

        gesture.connect_pressed({
            let popover = popover.clone();
            move |gesture, _n_press, x, y| {
                gesture.set_state(gtk4::EventSequenceState::Claimed);
                let rect = gdk::Rectangle::new(x as i32, y as i32, 1, 1);
                popover.set_pointing_to(Some(&rect));
                popover.popup();
            }
        });

        widget.add_controller(gesture);

        // Keyboard shortcuts for preview area
        let widget_clone = widget.clone();
        let key_controller = gtk4::EventControllerKey::new();
        key_controller.connect_key_pressed(move |_ctrl, keyval, _keycode, state| {
            let ctrl = state.contains(gdk::ModifierType::CONTROL_MASK);
            use gtk4::glib::Propagation;
            match (ctrl, keyval) {
                (true, gdk::Key::c) => {
                    // Copy
                    if let Some(text_view) = widget_clone.clone().dynamic_cast::<gtk4::TextView>().ok() {
                        let buffer = text_view.buffer();
                        if let Some(display) = gdk::Display::default() {
                            let clipboard = display.clipboard();
                            buffer.copy_clipboard(&clipboard);
                        }
                    }
                    if let Some(webview) = widget_clone.clone().dynamic_cast::<webkit6::WebView>().ok() {
                        webview.execute_editing_command("Copy");
                    }
                    return Propagation::Stop;
                }
                (true, gdk::Key::a) => {
                    // Select All
                    if let Some(text_view) = widget_clone.clone().dynamic_cast::<gtk4::TextView>().ok() {
                        let buffer = text_view.buffer();
                        let start = buffer.start_iter();
                        let end = buffer.end_iter();
                        buffer.select_range(&start, &end);
                    }
                    if let Some(webview) = widget_clone.clone().dynamic_cast::<webkit6::WebView>().ok() {
                        webview.execute_editing_command("SelectAll");
                    }
                    return Propagation::Stop;
                }
                _ => {}
            }
            Propagation::Proceed
        });
        widget.add_controller(key_controller);
    }
}
