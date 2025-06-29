use gtk4::prelude::*;
use gtk4::{gio, gdk};
use crate::editor::MarkdownEditor;
use crate::localization;

/// Context menu functionality for the Markdown editor
pub struct ContextMenu {
    popover: gtk4::PopoverMenu,
}

impl ContextMenu {
    /// Creates a new context menu for the given editor
    pub fn new(editor: &MarkdownEditor) -> Self {
        let menu_model = Self::create_menu_model();
        
        // Create popover with NESTED flag for traditional nested popovers with hover support
        let popover = gtk4::PopoverMenu::from_model_full(&menu_model, gtk4::PopoverMenuFlags::NESTED);
        
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
        
        Self {
            popover,
        }
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
            "
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
        
        // Create clipboard section
        let clipboard_section = gio::Menu::new();
        clipboard_section.append(Some(&localization::tr("menu.copy")), Some("context.copy"));
        clipboard_section.append(Some(&localization::tr("menu.cut")), Some("context.cut"));
        clipboard_section.append(Some(&localization::tr("menu.paste")), Some("context.paste"));
        menu_model.append_section(None, &clipboard_section);
        
        // Create basic formatting section  
        let formatting_section = gio::Menu::new();
        formatting_section.append(Some(&localization::tr("insert.bold")), Some("context.bold"));
        formatting_section.append(Some(&localization::tr("insert.italic")), Some("context.italic"));
        formatting_section.append(Some(&localization::tr("insert.strikethrough")), Some("context.strikethrough"));
        formatting_section.append(Some(&localization::tr("insert.inline_code")), Some("context.inline_code"));
        menu_model.append_section(None, &formatting_section);
        
        // Create submenus section
        let submenus_section = gio::Menu::new();
        
        // Headings submenu
        let headings_menu = gio::Menu::new();
        headings_menu.append(Some(&localization::tr("insert.heading1")), Some("context.heading1"));
        headings_menu.append(Some(&localization::tr("insert.heading2")), Some("context.heading2"));
        headings_menu.append(Some(&localization::tr("insert.heading3")), Some("context.heading3"));
        headings_menu.append(Some(&localization::tr("insert.heading4")), Some("context.heading4"));
        headings_menu.append(Some(&localization::tr("insert.heading5")), Some("context.heading5"));
        headings_menu.append(Some(&localization::tr("insert.heading6")), Some("context.heading6"));
        submenus_section.append_submenu(Some(&localization::tr("insert.headings")), &headings_menu);
        
        // Lists submenu
        let lists_menu = gio::Menu::new();
        lists_menu.append(Some(&localization::tr("insert.unordered_list")), Some("context.bullet_list"));
        lists_menu.append(Some(&localization::tr("insert.ordered_list")), Some("context.numbered_list"));
        lists_menu.append(Some(&localization::tr("insert.blockquote")), Some("context.blockquote"));
        submenus_section.append_submenu(Some(&localization::tr("context.lists")), &lists_menu);
        
        // Advanced formatting submenu
        let advanced_menu = gio::Menu::new();
        
        // Basic advanced formatting
        advanced_menu.append(Some(&localization::tr("insert.highlight")), Some("context.highlight"));
        advanced_menu.append(Some(&localization::tr("insert.subscript")), Some("context.subscript"));
        advanced_menu.append(Some(&localization::tr("insert.superscript")), Some("context.superscript"));
        
        // Advanced text styling (from markdown hacks)
        advanced_menu.append(Some(&localization::tr("advanced.underline")), Some("context.underline"));
        advanced_menu.append(Some(&localization::tr("advanced.center_text")), Some("context.center_text"));
        advanced_menu.append(Some(&localization::tr("advanced.colored_text")), Some("context.colored_text"));
        
        // Code blocks
        advanced_menu.append(Some(&localization::tr("insert.code_block")), Some("context.code_block"));
        advanced_menu.append(Some(&localization::tr("insert.fenced_code")), Some("context.fenced_code"));
        
        // Comments and admonitions
        advanced_menu.append(Some(&localization::tr("advanced.comment")), Some("context.comment"));
        advanced_menu.append(Some(&localization::tr("advanced.admonition")), Some("context.admonition"));
        
        submenus_section.append_submenu(Some(&localization::tr("context.advanced")), &advanced_menu);
        
        // Task lists submenu
        let tasks_menu = gio::Menu::new();
        tasks_menu.append(Some(&localization::tr("insert.task_list_open")), Some("context.task_open"));
        tasks_menu.append(Some(&localization::tr("insert.task_list_closed")), Some("context.task_closed"));
        tasks_menu.append(Some(&localization::tr("insert.task_list_custom")), Some("context.task_list"));
        submenus_section.append_submenu(Some(&localization::tr("context.tasks")), &tasks_menu);
        
        // Insert submenu
        let insert_menu = gio::Menu::new();
        insert_menu.append(Some(&localization::tr("insert.link")), Some("context.link"));
        insert_menu.append(Some(&localization::tr("insert.image")), Some("context.image"));
        insert_menu.append(Some(&localization::tr("insert.table")), Some("context.table"));
        insert_menu.append(Some(&localization::tr("insert.horizontal_rule")), Some("context.horizontal_rule"));
        insert_menu.append(Some(&localization::tr("insert.footnote")), Some("context.footnote"));
        insert_menu.append(Some(&localization::tr("insert.emoji")), Some("context.emoji"));
        submenus_section.append_submenu(Some(&localization::tr("context.insert")), &insert_menu);
        
        // Definition lists submenu
        let definition_menu = gio::Menu::new();
        definition_menu.append(Some(&localization::tr("insert.definition_list_single")), Some("context.definition_single"));
        definition_menu.append(Some(&localization::tr("insert.definition_list_custom")), Some("context.definition_list"));
        submenus_section.append_submenu(Some(&localization::tr("context.definitions")), &definition_menu);
        
        menu_model.append_section(None, &submenus_section);
        
        menu_model
    }
    
    /// Creates the action group with all context menu actions
    fn create_action_group(editor: &MarkdownEditor) -> gio::SimpleActionGroup {
        let action_group = gio::SimpleActionGroup::new();
        
        // Helper macro to create actions more concisely
        macro_rules! add_action {
            ($name:expr, $closure:expr) => {
                let action = gio::ActionEntry::builder($name)
                    .activate($closure)
                    .build();
                action_group.add_action_entries([action]);
            };
        }
        
        // Clipboard actions
        add_action!("copy", {
            let source_buffer = editor.source_buffer().clone();
            move |_group, _action, _param| {
                let gtk_buffer = source_buffer.upcast_ref::<gtk4::TextBuffer>();
                if let Some((start, end)) = gtk_buffer.selection_bounds() {
                    let selected_text = gtk_buffer.text(&start, &end, false);
                    if let Some(display) = gdk::Display::default() {
                        let clipboard = display.clipboard();
                        clipboard.set_text(&selected_text);
                    }
                }
            }
        });
        
        add_action!("cut", {
            let source_buffer = editor.source_buffer().clone();
            move |_group, _action, _param| {
                let gtk_buffer = source_buffer.upcast_ref::<gtk4::TextBuffer>();
                if let Some((mut start, mut end)) = gtk_buffer.selection_bounds() {
                    let selected_text = gtk_buffer.text(&start, &end, false);
                    if let Some(display) = gdk::Display::default() {
                        let clipboard = display.clipboard();
                        clipboard.set_text(&selected_text);
                    }
                    source_buffer.delete(&mut start, &mut end);
                }
            }
        });
        
        add_action!("paste", {
            let source_buffer = editor.source_buffer().clone();
            move |_group, _action, _param| {
                if let Some(display) = gdk::Display::default() {
                    let clipboard = display.clipboard();
                    clipboard.read_text_async(None::<&gio::Cancellable>, {
                        let source_buffer = source_buffer.clone();
                        move |result| {
                            if let Ok(text) = result {
                                if let Some(text_content) = text {
                                    let gtk_buffer = source_buffer.upcast_ref::<gtk4::TextBuffer>();
                                    let mut cursor_iter = gtk_buffer.iter_at_mark(&gtk_buffer.get_insert());
                                    source_buffer.insert(&mut cursor_iter, &text_content);
                                }
                            }
                        }
                    });
                }
            }
        });
        
        // Basic formatting actions
        add_action!("bold", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_bold();
            }
        });
        
        add_action!("italic", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_italic();
            }
        });
        
        add_action!("strikethrough", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_strikethrough();
            }
        });
        
        add_action!("inline_code", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                editor.insert_inline_code();
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
                crate::menu::show_underline_dialog(&editor);
            }
        });
        
        add_action!("center_text", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                crate::menu::show_center_text_dialog(&editor);
            }
        });
        
        add_action!("colored_text", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                crate::menu::show_colored_text_dialog(&editor);
            }
        });
        
        add_action!("comment", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                crate::menu::show_comment_dialog(&editor);
            }
        });
        
        add_action!("admonition", {
            let editor = editor.clone();
            move |_group, _action, _param| {
                crate::menu::show_admonition_dialog(&editor);
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
        if let Some(_window) = editor.source_view().root().and_then(|w| w.downcast::<gtk4::Window>().ok()) {
            let gtk_buffer = editor.source_buffer().upcast_ref::<gtk4::TextBuffer>();
            let cursor_iter = gtk_buffer.iter_at_mark(&gtk_buffer.get_insert());
            
            let cursor_rect = editor.source_view().iter_location(&cursor_iter);
            let rect = gdk::Rectangle::new(
                cursor_rect.x(),
                cursor_rect.y() + cursor_rect.height(),
                1,
                1
            );
            self.popover.set_pointing_to(Some(&rect));
            self.popover.popup();
        }
    }
}
