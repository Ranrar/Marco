use crate::editor::MarkdownEditor;
use crate::language;
use gtk4::prelude::*;
use gtk4::{gdk, gio};

/// Context menu functionality for the Markdown editor
pub struct ContextMenu {
    popover: gtk4::PopoverMenu,
}

impl ContextMenu {
    /// Creates a new context menu for the given editor
    pub fn new(editor: &MarkdownEditor) -> Self {
        let menu_model = Self::create_menu_model();

        // Create popover with NESTED flag for traditional nested popovers with hover support
        let popover =
            gtk4::PopoverMenu::from_model_full(&menu_model, gtk4::PopoverMenuFlags::NESTED);

        // Configure popover
        popover.set_parent(editor.source_view());
        popover.set_autohide(true);
        popover.set_has_arrow(true);

        // Create action group and actions
        let action_group = Self::create_action_group(editor);

        // Insert the action group into the popover
        popover.insert_action_group("context", Some(&action_group));

        // Add CSS styling to improve hover behavior
        Self::add_hover_styling();

        // Set up accelerators for the context menu actions
        Self::setup_accelerators();

        Self { popover }
    }

    /// Sets up keyboard accelerators for context menu actions
    fn setup_accelerators() {
        // Context menu actions use the same accelerators as the main menu
        // The accelerators are already registered in the main menu setup
        // No need to register them again here to avoid conflicts
    }

    /// Adds CSS styling to improve submenu hover behavior and fix separators
    fn add_hover_styling() {
        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_data(
            "
            /* Context menu styling with enhanced hover behavior */
            popover.menu button.model {
                padding: 8px 16px;
                margin: 1px;
                border-radius: 4px;
                transition: background-color 0.1s ease;
                min-width: 200px;
            }
            
            popover.menu button.model:hover {
                background-color: alpha(@accent_color, 0.1);
                transition: background-color 0.05s ease;
            }
            
            popover.menu button.model:hover arrow {
                opacity: 1.0;
            }
            
            /* Submenu arrow styling */
            popover.menu button.model arrow {
                opacity: 0.7;
                transition: opacity 0.1s ease;
            }
            
            /* Make submenus appear faster on hover */
            popover.menu popover {
                animation-duration: 0.1s;
            }
            
            /* Highlight submenu items on hover */
            popover.menu popover button.model:hover {
                background-color: alpha(@accent_color, 0.15);
            }
            
            /* Section separators should be thin lines */
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

    /// Creates the menu model with all available actions
    fn create_menu_model() -> gio::Menu {
        let menu_model = gio::Menu::new();

        // Create edit section
        let edit_section = gio::Menu::new();
        edit_section.append(Some(&language::tr("menu.undo")), Some("app.undo"));
        edit_section.append(Some(&language::tr("menu.redo")), Some("app.redo"));
        menu_model.append_section(None, &edit_section);

        // Create clipboard section
        let clipboard_section = gio::Menu::new();
        clipboard_section.append(Some(&language::tr("menu.cut")), Some("app.cut"));
        clipboard_section.append(Some(&language::tr("menu.copy")), Some("app.copy"));
        clipboard_section.append(Some(&language::tr("menu.paste")), Some("app.paste"));
        clipboard_section.append(Some(&language::tr("menu.delete")), Some("context.delete"));
        menu_model.append_section(None, &clipboard_section);

        // Create selection section
        let selection_section = gio::Menu::new();
        selection_section.append(
            Some(&language::tr("menu.select_all")),
            Some("app.select_all"),
        );
        menu_model.append_section(None, &selection_section);

        // Create basic formatting section
        let formatting_section = gio::Menu::new();
        formatting_section.append(Some(&language::tr("insert.bold")), Some("app.insert_bold"));
        formatting_section.append(
            Some(&language::tr("insert.italic")),
            Some("app.insert_italic"),
        );
        formatting_section.append(
            Some(&language::tr("insert.strikethrough")),
            Some("context.strikethrough"),
        );
        formatting_section.append(
            Some(&language::tr("insert.inline_code")),
            Some("app.insert_inline_code"),
        );
        menu_model.append_section(None, &formatting_section);

        // Create submenus section
        let submenus_section = gio::Menu::new();

        // Headings submenu
        let headings_menu = gio::Menu::new();
        headings_menu.append(
            Some(&language::tr("insert.heading1")),
            Some("context.heading1"),
        );
        headings_menu.append(
            Some(&language::tr("insert.heading2")),
            Some("context.heading2"),
        );
        headings_menu.append(
            Some(&language::tr("insert.heading3")),
            Some("context.heading3"),
        );
        headings_menu.append(
            Some(&language::tr("insert.heading4")),
            Some("context.heading4"),
        );
        headings_menu.append(
            Some(&language::tr("insert.heading5")),
            Some("context.heading5"),
        );
        headings_menu.append(
            Some(&language::tr("insert.heading6")),
            Some("context.heading6"),
        );
        submenus_section.append_submenu(Some(&language::tr("insert.headings")), &headings_menu);

        // Lists submenu
        let lists_menu = gio::Menu::new();
        lists_menu.append(
            Some(&language::tr("insert.unordered_list")),
            Some("context.bullet_list"),
        );
        lists_menu.append(
            Some(&language::tr("insert.ordered_list")),
            Some("context.numbered_list"),
        );
        lists_menu.append(
            Some(&language::tr("insert.blockquote")),
            Some("context.blockquote"),
        );
        submenus_section.append_submenu(Some(&language::tr("context.lists")), &lists_menu);

        // Advanced formatting submenu
        let advanced_menu = gio::Menu::new();

        // Basic advanced formatting
        advanced_menu.append(
            Some(&language::tr("insert.highlight")),
            Some("context.highlight"),
        );
        advanced_menu.append(
            Some(&language::tr("insert.subscript")),
            Some("context.subscript"),
        );
        advanced_menu.append(
            Some(&language::tr("insert.superscript")),
            Some("context.superscript"),
        );

        // Advanced text styling (from markdown hacks)
        advanced_menu.append(
            Some(&language::tr("advanced.underline")),
            Some("context.underline"),
        );
        advanced_menu.append(
            Some(&language::tr("advanced.center_text")),
            Some("context.center_text"),
        );
        advanced_menu.append(
            Some(&language::tr("advanced.colored_text")),
            Some("context.colored_text"),
        );

        // Code blocks
        advanced_menu.append(
            Some(&language::tr("insert.code_block")),
            Some("context.code_block"),
        );
        advanced_menu.append(
            Some(&language::tr("insert.fenced_code")),
            Some("context.fenced_code"),
        );

        // Comments and admonitions
        advanced_menu.append(
            Some(&language::tr("advanced.comment")),
            Some("context.comment"),
        );
        advanced_menu.append(
            Some(&language::tr("advanced.admonition")),
            Some("context.admonition"),
        );

        submenus_section.append_submenu(Some(&language::tr("context.advanced")), &advanced_menu);

        // Task lists submenu
        let tasks_menu = gio::Menu::new();
        tasks_menu.append(
            Some(&language::tr("insert.task_list_open")),
            Some("context.task_open"),
        );
        tasks_menu.append(
            Some(&language::tr("insert.task_list_closed")),
            Some("context.task_closed"),
        );
        tasks_menu.append(
            Some(&language::tr("insert.task_list_custom")),
            Some("context.task_list"),
        );
        submenus_section.append_submenu(Some(&language::tr("context.tasks")), &tasks_menu);

        // Insert submenu
        let insert_menu = gio::Menu::new();
        insert_menu.append(Some(&language::tr("insert.link")), Some("context.link"));
        insert_menu.append(Some(&language::tr("insert.image")), Some("context.image"));
        insert_menu.append(Some(&language::tr("insert.table")), Some("context.table"));
        insert_menu.append(
            Some(&language::tr("insert.horizontal_rule")),
            Some("context.horizontal_rule"),
        );
        insert_menu.append(
            Some(&language::tr("insert.footnote")),
            Some("context.footnote"),
        );
        insert_menu.append(Some(&language::tr("insert.emoji")), Some("context.emoji"));
        submenus_section.append_submenu(Some(&language::tr("context.insert")), &insert_menu);

        // Definition lists submenu
        let definition_menu = gio::Menu::new();
        definition_menu.append(
            Some(&language::tr("insert.definition_list_single")),
            Some("context.definition_single"),
        );
        definition_menu.append(
            Some(&language::tr("insert.definition_list_custom")),
            Some("context.definition_list"),
        );
        submenus_section
            .append_submenu(Some(&language::tr("context.definitions")), &definition_menu);

        menu_model.append_section(None, &submenus_section);

        menu_model
    }

    /// Creates the action group with all context menu actions
    fn create_action_group(editor: &MarkdownEditor) -> gio::SimpleActionGroup {
        let action_group = gio::SimpleActionGroup::new();

        // Helper macro to create actions more concisely
        macro_rules! add_action {
            ($name:expr, $closure:expr) => {
                let action = gio::ActionEntry::builder($name).activate($closure).build();
                action_group.add_action_entries([action]);
            };
        }

        // Context-specific actions that don't have app equivalents
        add_action!("delete", {
            let source_buffer = editor.source_buffer().clone();
            move |_group, _action, _param| {
                let gtk_buffer = source_buffer.upcast_ref::<gtk4::TextBuffer>();
                if let Some((mut start, mut end)) = gtk_buffer.selection_bounds() {
                    // Delete selected text
                    source_buffer.delete(&mut start, &mut end);
                } else {
                    // Delete character at cursor position (like pressing Delete key)
                    let cursor_iter = gtk_buffer.iter_at_mark(&gtk_buffer.get_insert());
                    let mut end_iter = cursor_iter;
                    if end_iter.forward_char() {
                        let mut start_iter = cursor_iter;
                        source_buffer.delete(&mut start_iter, &mut end_iter);
                    }
                }
            }
        });

        add_action!("strikethrough", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_strikethrough();
            }
        });

        // Heading actions
        add_action!("heading1", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_heading(1);
            }
        });

        add_action!("heading2", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_heading(2);
            }
        });

        add_action!("heading3", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_heading(3);
            }
        });

        add_action!("heading4", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_heading(4);
            }
        });

        add_action!("heading5", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_heading(5);
            }
        });

        add_action!("heading6", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_heading(6);
            }
        });

        // List actions
        add_action!("bullet_list", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_bullet_list();
            }
        });

        add_action!("numbered_list", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_numbered_list();
            }
        });

        add_action!("blockquote", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_blockquote();
            }
        });

        // Advanced formatting actions
        add_action!("highlight", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_highlight();
            }
        });

        add_action!("subscript", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_subscript();
            }
        });

        add_action!("superscript", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_superscript();
            }
        });

        add_action!("code_block", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_code_block();
            }
        });

        add_action!("fenced_code", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_fenced_code_block();
            }
        });

        // Advanced markdown hack actions
        add_action!("underline", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                // Get the window from the editor's widget hierarchy
                if let Some(window) = editor
                    .source_view()
                    .root()
                    .and_then(|w| w.downcast::<gtk4::Window>().ok())
                {
                    crate::menu::show_underline_dialog(&window, &editor);
                }
            }
        });

        add_action!("center_text", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                // Get the window from the editor's widget hierarchy
                if let Some(window) = editor
                    .source_view()
                    .root()
                    .and_then(|w| w.downcast::<gtk4::Window>().ok())
                {
                    crate::menu::show_center_text_dialog(&window, &editor);
                }
            }
        });

        add_action!("colored_text", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                // Get the window from the editor's widget hierarchy
                if let Some(window) = editor
                    .source_view()
                    .root()
                    .and_then(|w| w.downcast::<gtk4::Window>().ok())
                {
                    crate::menu::show_colored_text_dialog(&window, &editor);
                }
            }
        });

        add_action!("comment", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                // Get the window from the editor's widget hierarchy
                if let Some(window) = editor
                    .source_view()
                    .root()
                    .and_then(|w| w.downcast::<gtk4::Window>().ok())
                {
                    crate::menu::show_comment_dialog(&window, &editor);
                }
            }
        });

        add_action!("admonition", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                // Get the window from the editor's widget hierarchy
                if let Some(window) = editor
                    .source_view()
                    .root()
                    .and_then(|w| w.downcast::<gtk4::Window>().ok())
                {
                    crate::menu::show_admonition_dialog(&window, &editor);
                }
            }
        });

        // Insert and table actions
        add_action!("table", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                // Get the window from the editor's widget hierarchy
                if let Some(window) = editor
                    .source_view()
                    .root()
                    .and_then(|w| w.downcast::<gtk4::Window>().ok())
                {
                    crate::menu::create_table_dialog(&window, &editor);
                } else {
                    // Fallback to simple table if no window found
                    editor.insert_table();
                }
            }
        });

        add_action!("link", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_link();
            }
        });

        add_action!("image", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_image();
            }
        });

        add_action!("horizontal_rule", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_horizontal_rule();
            }
        });

        add_action!("footnote", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_footnote();
            }
        });

        add_action!("emoji", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                crate::editor::emoji::show_emoji_picker_dialog(&editor);
            }
        });

        // Task list actions
        add_action!("task_open", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_single_open_task();
            }
        });

        add_action!("task_closed", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_single_closed_task();
            }
        });

        add_action!("task_list", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                // Get the window from the editor's widget hierarchy
                if let Some(window) = editor
                    .source_view()
                    .root()
                    .and_then(|w| w.downcast::<gtk4::Window>().ok())
                {
                    crate::menu::show_task_list_custom_dialog(&window, &editor);
                }
            }
        });

        // Definition list actions
        add_action!("definition_single", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_single_definition();
            }
        });

        add_action!("definition_list", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                // Get the window from the editor's widget hierarchy
                if let Some(window) = editor
                    .source_view()
                    .root()
                    .and_then(|w| w.downcast::<gtk4::Window>().ok())
                {
                    crate::menu::show_definition_list_custom_dialog(&window, &editor);
                }
            }
        });

        action_group
    }

    /// Sets up the right-click gesture on the editor's source view
    pub fn setup_gesture(&self, editor: &MarkdownEditor) {
        let gesture = gtk4::GestureClick::new();
        gesture.set_button(3); // Right mouse button
        gesture.set_exclusive(true);
        gesture.set_propagation_phase(gtk4::PropagationPhase::Capture);

        gesture.connect_pressed({
            let popover = self.popover.clone();
            move |gesture, _n_press, x, y| {
                // Claim the gesture to prevent default context menu
                gesture.set_state(gtk4::EventSequenceState::Claimed);

                let rect = gdk::Rectangle::new(x as i32, y as i32, 1, 1);
                popover.set_pointing_to(Some(&rect));
                popover.popup();
            }
        });

        editor.source_view().add_controller(gesture);

        // Enhance submenu behavior for better hover experience
        self.setup_submenu_hover_behavior();
    }

    /// Configures submenu hover behavior for better user experience
    fn setup_submenu_hover_behavior(&self) {
        // Set up hover controllers for submenu items if needed
        // Note: GTK4's PopoverMenu with NESTED flag should already handle hover-to-open
        // This method can be extended for additional custom hover behaviors

        // GTK4 Note: The gtk-menu-popdown-delay and gtk-menu-popup-delay settings
        // that existed in GTK3 have been removed in GTK4. The PopoverMenu with
        // NESTED flag should provide good hover-to-open behavior by default.

        // Additional custom hover behaviors can be implemented here if needed
    }

    /// Shows the context menu at the current cursor position
    pub fn show_at_cursor(&self, editor: &MarkdownEditor) {
        // Get the cursor position in the text view
        if let Some(_window) = editor
            .source_view()
            .root()
            .and_then(|w| w.downcast::<gtk4::Window>().ok())
        {
            let gtk_buffer = editor.source_buffer().upcast_ref::<gtk4::TextBuffer>();
            let cursor_iter = gtk_buffer.iter_at_mark(&gtk_buffer.get_insert());

            let cursor_rect = editor.source_view().iter_location(&cursor_iter);
            let rect = gdk::Rectangle::new(
                cursor_rect.x(),
                cursor_rect.y() + cursor_rect.height(),
                1,
                1,
            );
            self.popover.set_pointing_to(Some(&rect));
            self.popover.popup();
        }
    }
}
