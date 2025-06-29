use gtk4::prelude::*;
use gtk4::{PopoverMenuBar, Application, gio, Dialog, Grid, Entry, ResponseType, Box, Orientation, Label, SpinButton, Adjustment};
use crate::{editor, localization, emoji, settings};

pub fn create_menu_bar(app: &Application, editor: &editor::MarkdownEditor, theme_manager: &crate::theme::ThemeManager) -> PopoverMenuBar {
    // Create the menu model
    let menu_model = gio::Menu::new();
    
    // File Menu
    let file_menu = gio::Menu::new();
    file_menu.append(Some(&localization::tr("menu.new")), Some("app.new"));
    file_menu.append(Some(&localization::tr("menu.open")), Some("app.open"));
    file_menu.append(Some(&localization::tr("menu.save")), Some("app.save"));
    file_menu.append(Some(&localization::tr("menu.save_as")), Some("app.save_as"));
    file_menu.append(Some(&localization::tr("menu.quit")), Some("app.quit"));
    
    menu_model.append_submenu(Some(&localization::tr("menu.file")), &file_menu);
    
    // Edit Menu
    let edit_menu = gio::Menu::new();
    edit_menu.append(Some(&localization::tr("menu.undo")), Some("app.undo"));
    edit_menu.append(Some(&localization::tr("menu.redo")), Some("app.redo"));
    edit_menu.append(Some(&localization::tr("menu.cut")), Some("app.cut"));
    edit_menu.append(Some(&localization::tr("menu.copy")), Some("app.copy"));
    edit_menu.append(Some(&localization::tr("menu.paste")), Some("app.paste"));
    edit_menu.append(Some(&localization::tr("menu.find")), Some("app.find"));
    edit_menu.append(Some(&localization::tr("menu.replace")), Some("app.replace"));
    
    menu_model.append_submenu(Some(&localization::tr("menu.edit")), &edit_menu);
    
    // Insert Menu (Basic Syntax)
    let insert_menu = gio::Menu::new();
    
    // Add Headings submenu to Insert
    let headings_menu = gio::Menu::new();
    headings_menu.append(Some(&localization::tr("insert.heading1")), Some("app.heading1"));
    headings_menu.append(Some(&localization::tr("insert.heading2")), Some("app.heading2"));
    headings_menu.append(Some(&localization::tr("insert.heading3")), Some("app.heading3"));
    headings_menu.append(Some(&localization::tr("insert.heading4")), Some("app.heading4"));
    headings_menu.append(Some(&localization::tr("insert.heading5")), Some("app.heading5"));
    headings_menu.append(Some(&localization::tr("insert.heading6")), Some("app.heading6"));
    insert_menu.append_submenu(Some(&localization::tr("insert.headings")), &headings_menu);
    
    insert_menu.append(Some(&localization::tr("insert.bold")), Some("app.insert_bold"));
    insert_menu.append(Some(&localization::tr("insert.italic")), Some("app.insert_italic"));
    insert_menu.append(Some(&localization::tr("insert.blockquote")), Some("app.insert_blockquote"));
    insert_menu.append(Some(&localization::tr("insert.ordered_list")), Some("app.insert_numbered_list"));
    insert_menu.append(Some(&localization::tr("insert.unordered_list")), Some("app.insert_bullet_list"));
    insert_menu.append(Some(&localization::tr("insert.inline_code")), Some("app.insert_inline_code"));
    insert_menu.append(Some(&localization::tr("insert.horizontal_rule")), Some("app.insert_hr"));
    insert_menu.append(Some(&localization::tr("insert.link")), Some("app.insert_link"));
    insert_menu.append(Some(&localization::tr("insert.image")), Some("app.insert_image"));
    menu_model.append_submenu(Some(&localization::tr("menu.insert")), &insert_menu);

    // Format Menu (Extended Syntax)
    let format_menu = gio::Menu::new();
    
    // Text formatting
    format_menu.append(Some(&localization::tr("insert.strikethrough")), Some("app.strikethrough"));
    format_menu.append(Some(&localization::tr("insert.highlight")), Some("app.insert_highlight"));
    format_menu.append(Some(&localization::tr("insert.subscript")), Some("app.insert_subscript"));
    format_menu.append(Some(&localization::tr("insert.superscript")), Some("app.insert_superscript"));
    
    // Code
    format_menu.append(Some(&localization::tr("insert.code_block")), Some("app.code_block"));
    
    // Fenced Code submenu with languages
    let fenced_code_menu = gio::Menu::new();
    
    // Create dialog section
    let dialog_section = gio::Menu::new();
    dialog_section.append(Some("⚙️ Fenced Code with Options..."), Some("app.insert_fenced_code"));
    fenced_code_menu.append_section(None, &dialog_section);
    
    // Create programming languages section
    let programming_section = gio::Menu::new();
    programming_section.append(Some("JavaScript"), Some("app.insert_fenced_javascript"));
    programming_section.append(Some("Python"), Some("app.insert_fenced_python"));
    programming_section.append(Some("Java"), Some("app.insert_fenced_java"));
    programming_section.append(Some("TypeScript"), Some("app.insert_fenced_typescript"));
    programming_section.append(Some("C#"), Some("app.insert_fenced_csharp"));
    programming_section.append(Some("C++"), Some("app.insert_fenced_cpp"));
    programming_section.append(Some("C"), Some("app.insert_fenced_c"));
    programming_section.append(Some("PHP"), Some("app.insert_fenced_php"));
    programming_section.append(Some("Go"), Some("app.insert_fenced_go"));
    programming_section.append(Some("Rust"), Some("app.insert_fenced_rust"));
    fenced_code_menu.append_section(None, &programming_section);
    
    // Create markup/data languages section
    let markup_section = gio::Menu::new();
    markup_section.append(Some("HTML"), Some("app.insert_fenced_html"));
    markup_section.append(Some("CSS"), Some("app.insert_fenced_css"));
    markup_section.append(Some("JSON"), Some("app.insert_fenced_json"));
    markup_section.append(Some("XML"), Some("app.insert_fenced_xml"));
    markup_section.append(Some("SQL"), Some("app.insert_fenced_sql"));
    markup_section.append(Some("Bash"), Some("app.insert_fenced_bash"));
    markup_section.append(Some("YAML"), Some("app.insert_fenced_yaml"));
    markup_section.append(Some("Markdown"), Some("app.insert_fenced_markdown"));
    fenced_code_menu.append_section(None, &markup_section);
    
    format_menu.append_submenu(Some(&localization::tr("insert.fenced_code")), &fenced_code_menu);
    
    // Lists and tasks
    let task_list_menu = gio::Menu::new();
    task_list_menu.append(Some("⚙️ Custom Task List..."), Some("app.insert_task_list_custom"));
    task_list_menu.append(Some("☐ Open Task"), Some("app.insert_task_list_open"));
    task_list_menu.append(Some("☑️ Closed Task"), Some("app.insert_task_list_closed"));
    
    format_menu.append_submenu(Some(&localization::tr("insert.task_list")), &task_list_menu);
    
    // Definition list submenu
    let definition_list_menu = gio::Menu::new();
    definition_list_menu.append(Some("⚙️ Custom Definition List..."), Some("app.insert_definition_list_custom"));
    definition_list_menu.append(Some("📖 Single Definition"), Some("app.insert_definition_list_single"));
    
    format_menu.append_submenu(Some(&localization::tr("insert.definition_list")), &definition_list_menu);
    
    // Special elements
    format_menu.append(Some("⚙️ Table..."), Some("app.insert_table_dialog"));
    format_menu.append(Some("📝 Footnote"), Some("app.insert_footnote"));
    format_menu.append(Some("😀 Emoji"), Some("app.insert_emoji"));
    
    menu_model.append_submenu(Some(&localization::tr("menu.format")), &format_menu);
    
    // Advanced Menu (Markdown Hacks from markdownguide.org/hacks/)
    let advanced_menu = gio::Menu::new();
    
    // Text styling hacks
    let text_styling_menu = gio::Menu::new();
    text_styling_menu.append(Some("🔤 Underline"), Some("app.insert_underline"));
    text_styling_menu.append(Some("📐 Center Text"), Some("app.insert_center_text"));
    text_styling_menu.append(Some("🎨 Colored Text"), Some("app.insert_colored_text"));
    text_styling_menu.append(Some("📝 Indent Text"), Some("app.insert_indent_text"));
    advanced_menu.append_submenu(Some(&localization::tr("advanced.text_styling")), &text_styling_menu);
    
    // Comments and admonitions
    let comments_menu = gio::Menu::new();
    comments_menu.append(Some("💬 Comment"), Some("app.insert_comment"));
    comments_menu.append(Some("⚠️ Admonition"), Some("app.insert_admonition"));
    advanced_menu.append_submenu(Some(&localization::tr("advanced.comments_admonitions")), &comments_menu);
    
    // Enhanced images and links
    let media_menu = gio::Menu::new();
    media_menu.append(Some("🖼️ Image with Size"), Some("app.insert_image_with_size"));
    media_menu.append(Some("🖼️ Image with Caption"), Some("app.insert_image_with_caption"));
    media_menu.append(Some("🔗 Link with Target"), Some("app.insert_link_with_target"));
    media_menu.append(Some("📹 YouTube Video"), Some("app.insert_youtube_video"));
    advanced_menu.append_submenu(Some(&localization::tr("advanced.enhanced_media")), &media_menu);
    
    // Special symbols and entities
    let symbols_menu = gio::Menu::new();
    symbols_menu.append(Some("🔣 HTML Entity"), Some("app.insert_html_entity"));
    symbols_menu.append(Some("📑 Table of Contents"), Some("app.insert_table_of_contents"));
    advanced_menu.append_submenu(Some(&localization::tr("advanced.symbols_structure")), &symbols_menu);
    
    menu_model.append_submenu(Some(&localization::tr("menu.advanced")), &advanced_menu);
    
    // View Menu (for language switching, view mode, and CSS themes)
    let view_menu = create_view_menu(editor, theme_manager);
    menu_model.append_submenu(Some(&localization::tr("menu.view")), &view_menu);
    
    // Help Menu
    let help_menu = gio::Menu::new();
    help_menu.append(Some(&localization::tr("help.markdown_guide")), Some("app.markdown_guide"));
    help_menu.append(Some(&localization::tr("help.shortcuts")), Some("app.shortcuts"));
    help_menu.append(Some(&localization::tr("help.about")), Some("app.about"));
    
    menu_model.append_submenu(Some(&localization::tr("menu.help")), &help_menu);
    
    // Create actions
    create_menu_actions(app, editor, theme_manager);
    
    // Set up keyboard accelerators for menu actions
    setup_menu_accelerators(app);
    
    // Create the menu bar
    PopoverMenuBar::from_model(Some(&menu_model))
}

pub fn create_menu_actions(app: &Application, editor: &editor::MarkdownEditor, theme_manager: &crate::theme::ThemeManager) {
    // File actions
    let new_action = gio::ActionEntry::builder("new")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.new_file();
            }
        })
        .build();
    
    let open_action = gio::ActionEntry::builder("open")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                if let Some(window) = app.active_window() {
                    editor.open_file_from_menu(&window);
                }
            }
        })
        .build();
    
    let save_action = gio::ActionEntry::builder("save")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                if let Some(window) = app.active_window() {
                    editor.save_file_from_menu(&window);
                }
            }
        })
        .build();
    
    let save_as_action = gio::ActionEntry::builder("save_as")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                if let Some(window) = app.active_window() {
                    editor.save_as_file_from_menu(&window);
                }
            }
        })
        .build();
    
    let quit_action = gio::ActionEntry::builder("quit")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                println!("Quit clicked");
                
                // Get the main window to use as parent for the dialog
                let window = app.active_window();
                let app_clone = app.clone();
                
                // Check if there are unsaved changes and show confirmation if needed
                let should_quit_immediately = editor.show_unsaved_changes_dialog_and_quit(
                    window.as_ref(), 
                    move || {
                        println!("DEBUG: Confirmed quit, calling app.quit()");
                        app_clone.quit();
                    }
                );
                
                if should_quit_immediately {
                    // No unsaved changes, quit immediately
                    app.quit();
                }
                // Otherwise, the quit will happen in the dialog callback
            }
        })
        .build();
    
    // Edit actions
    let undo_action = gio::ActionEntry::builder("undo")
        .activate(|_app: &Application, _action, _param| {
            println!("Undo clicked");
        })
        .build();
    
    let redo_action = gio::ActionEntry::builder("redo")
        .activate(|_app: &Application, _action, _param| {
            println!("Redo clicked");
        })
        .build();
    
    let cut_action = gio::ActionEntry::builder("cut")
        .activate(|_app: &Application, _action, _param| {
            println!("Cut clicked");
        })
        .build();
    
    let copy_action = gio::ActionEntry::builder("copy")
        .activate(|_app: &Application, _action, _param| {
            println!("Copy clicked");
        })
        .build();
    
    let paste_action = gio::ActionEntry::builder("paste")
        .activate(|_app: &Application, _action, _param| {
            println!("Paste clicked");
        })
        .build();
    
    let find_action = gio::ActionEntry::builder("find")
        .activate(|_app: &Application, _action, _param| {
            println!("Find clicked");
        })
        .build();
    
    let replace_action = gio::ActionEntry::builder("replace")
        .activate(|_app: &Application, _action, _param| {
            println!("Find & Replace clicked");
        })
        .build();
    
    // Insert actions
    let insert_header_action = gio::ActionEntry::builder("insert_header")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_heading(1);
            }
        })
        .build();
    
    let insert_bold_action = gio::ActionEntry::builder("insert_bold")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_bold();
            }
        })
        .build();
    
    let insert_italic_action = gio::ActionEntry::builder("insert_italic")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_italic();
            }
        })
        .build();
    
    let insert_code_action = gio::ActionEntry::builder("insert_code")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_code_block();
            }
        })
        .build();
    
    let insert_link_action = gio::ActionEntry::builder("insert_link")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_link();
            }
        })
        .build();
    
    let insert_image_action = gio::ActionEntry::builder("insert_image")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_image();
            }
        })
        .build();
    
    let insert_table_action = gio::ActionEntry::builder("insert_table")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_table();
            }
        })
        .build();

    let insert_table_dialog_action = gio::ActionEntry::builder("insert_table_dialog")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                use std::rc::Rc;
                
                if let Some(window) = app.active_window() {
                    let dialog = Dialog::with_buttons(
                        Some(&localization::tr("table_dialog.title")),
                        Some(&window),
                        gtk4::DialogFlags::MODAL,
                        &[(&localization::tr("table_dialog.insert"), ResponseType::Accept), 
                          (&localization::tr("table_dialog.cancel"), ResponseType::Cancel)],
                    );
                    let content_area = dialog.content_area();
                    
                    // Create main container
                    let main_container = Box::new(Orientation::Vertical, 12);
                    main_container.set_margin_top(12);
                    main_container.set_margin_bottom(12);
                    main_container.set_margin_start(12);
                    main_container.set_margin_end(12);

                    // Add title label
                    let title_label = Label::new(Some(&localization::tr("table_dialog.description")));
                    title_label.set_halign(gtk4::Align::Start);
                    main_container.append(&title_label);

                    // Create grid for input fields
                    let input_grid = Grid::new();
                    input_grid.set_row_spacing(8);
                    input_grid.set_column_spacing(12);
                    input_grid.set_margin_top(12);

                    // Rows input
                    let rows_label = Label::new(Some(&localization::tr("table_dialog.rows")));
                    rows_label.set_halign(gtk4::Align::End);
                    input_grid.attach(&rows_label, 0, 0, 1, 1);
                    
                    let rows_entry = Entry::new();
                    rows_entry.set_text("3"); // Default value
                    rows_entry.set_width_chars(5);
                    rows_entry.set_max_length(3); // Limit to 3 characters
                    
                    input_grid.attach(&rows_entry, 1, 0, 1, 1);

                    // Columns input
                    let cols_label = Label::new(Some(&localization::tr("table_dialog.columns")));
                    cols_label.set_halign(gtk4::Align::End);
                    input_grid.attach(&cols_label, 0, 1, 1, 1);
                    
                    let cols_entry = Entry::new();
                    cols_entry.set_text("3"); // Default value
                    cols_entry.set_width_chars(5);
                    cols_entry.set_max_length(3); // Limit to 3 characters
                    
                    // Now add the input handlers after both entries are created
                    
                    // Add input filter for rows entry (numbers only)
                    let cols_entry_for_rows = cols_entry.clone();
                    rows_entry.connect_insert_text(move |entry, text, position| {
                        let filtered_text: String = text.chars()
                            .filter(|c| c.is_ascii_digit())
                            .collect();
                        
                        if filtered_text != text {
                            entry.stop_signal_emission_by_name("insert-text");
                            if !filtered_text.is_empty() {
                                entry.insert_text(&filtered_text, position);
                            }
                        }
                        
                        // Clear error state when user types valid input
                        entry.remove_css_class("error");
                        cols_entry_for_rows.remove_css_class("error");
                    });
                    
                    // Add input filter for columns entry (numbers only)
                    let rows_entry_for_cols = rows_entry.clone();
                    cols_entry.connect_insert_text(move |entry, text, position| {
                        let filtered_text: String = text.chars()
                            .filter(|c| c.is_ascii_digit())
                            .collect();
                        
                        if filtered_text != text {
                            entry.stop_signal_emission_by_name("insert-text");
                            if !filtered_text.is_empty() {
                                entry.insert_text(&filtered_text, position);
                            }
                        }
                        
                        // Clear error state when user types valid input
                        entry.remove_css_class("error");
                        rows_entry_for_cols.remove_css_class("error");
                    });
                    
                    input_grid.attach(&cols_entry, 1, 1, 1, 1);

                    main_container.append(&input_grid);
                    content_area.append(&main_container);
                    
                    // Set focus to rows entry
                    rows_entry.grab_focus();
                    
                    dialog.set_default_response(ResponseType::Accept);
                    dialog.show();

                    let editor_clone = editor.clone();
                    let rows_entry_clone = Rc::new(rows_entry);
                    let cols_entry_clone = Rc::new(cols_entry);
                    
                    dialog.connect_response(move |dialog, resp| {
                        if resp == ResponseType::Accept {
                            let rows_text = rows_entry_clone.text();
                            let cols_text = cols_entry_clone.text();
                            
                            // Parse input values and validate
                            let rows_valid = rows_text.parse::<usize>().is_ok() && !rows_text.is_empty();
                            let cols_valid = cols_text.parse::<usize>().is_ok() && !cols_text.is_empty();
                            
                            if rows_valid && cols_valid {
                                if let (Ok(rows), Ok(cols)) = (rows_text.parse::<usize>(), cols_text.parse::<usize>()) {
                                    if rows > 0 && cols > 0 && rows <= 999 && cols <= 999 {
                                        // Valid input - create table and close dialog
                                        editor_clone.insert_custom_table(rows, cols);
                                        dialog.close();
                                        return;
                                    }
                                }
                            }
                            
                            // Invalid input - add red styling and don't close dialog
                            if !rows_valid || rows_text.is_empty() {
                                rows_entry_clone.add_css_class("error");
                            }
                            if !cols_valid || cols_text.is_empty() {
                                cols_entry_clone.add_css_class("error");
                            }
                        } else {
                            // Cancel button - close dialog
                            dialog.close();
                        }
                    });
                }
            }
        })
        .build();
    
    let insert_hr_action = gio::ActionEntry::builder("insert_hr")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_horizontal_rule();
            }
        })
        .build();

    let insert_inline_code_action = gio::ActionEntry::builder("insert_inline_code")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_inline_code();
            }
        })
        .build();

    let insert_bullet_list_action = gio::ActionEntry::builder("insert_bullet_list")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_bullet_list();
            }
        })
        .build();

    let insert_numbered_list_action = gio::ActionEntry::builder("insert_numbered_list")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_numbered_list();
            }
        })
        .build();

    let insert_blockquote_action = gio::ActionEntry::builder("insert_blockquote")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_blockquote();
            }
        })
        .build();

    let strikethrough_action = gio::ActionEntry::builder("strikethrough")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_strikethrough();
            }
        })
        .build();

    let code_block_action = gio::ActionEntry::builder("code_block")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_code_block();
            }
        })
        .build();

    let heading1_action = gio::ActionEntry::builder("heading1")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_heading(1);
            }
        })
        .build();

    let heading2_action = gio::ActionEntry::builder("heading2")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_heading(2);
            }
        })
        .build();

    let heading3_action = gio::ActionEntry::builder("heading3")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_heading(3);
            }
        })
        .build();

    let heading4_action = gio::ActionEntry::builder("heading4")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_heading(4);
            }
        })
        .build();

    let heading5_action = gio::ActionEntry::builder("heading5")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_heading(5);
            }
        })
        .build();

    let heading6_action = gio::ActionEntry::builder("heading6")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_heading(6);
            }
        })
        .build();

    // Help actions
    let markdown_guide_action = gio::ActionEntry::builder("markdown_guide")
        .activate(|_app: &Application, _action, _param| {
            println!("Markdown Guide clicked");
        })
        .build();
    
    let shortcuts_action = gio::ActionEntry::builder("shortcuts")
        .activate({
            move |app: &Application, _action, _param| {
                if let Some(window) = app.active_window() {
                    show_shortcuts_dialog(&window);
                }
            }
        })
        .build();
    
    let about_action = gio::ActionEntry::builder("about")
        .activate(|_app: &Application, _action, _param| {
            println!("About clicked");
        })
        .build();
    
    // Language switching actions
    let set_language_en_action = gio::ActionEntry::builder("set_language_en")
        .activate({
            let editor = editor.clone();
            let theme_mgr = theme_manager.clone();
            move |app: &Application, _action, _param| {
                localization::set_locale("en");
                settings::update_language("en");
                rebuild_menu_bar(app, &editor, &theme_mgr);
                println!("Language changed to English");
            }
        })
        .build();

    // View switching actions
    let view_html_action = gio::ActionEntry::builder("view_html")
        .activate({
            let editor = editor.clone();
            let theme_mgr = theme_manager.clone();
            move |app: &Application, _action, _param| {
                editor.set_view_mode("html");
                settings::update_view_mode("html");
                rebuild_menu_bar(app, &editor, &theme_mgr);
            }
        })
        .build();

    let view_code_action = gio::ActionEntry::builder("view_code")
        .activate({
            let editor = editor.clone();
            let theme_mgr = theme_manager.clone();
            move |app: &Application, _action, _param| {
                editor.set_view_mode("code");
                settings::update_view_mode("code");
                rebuild_menu_bar(app, &editor, &theme_mgr);
            }
        })
        .build();
    
    // Theme switching actions
    let theme_system_action = gio::ActionEntry::builder("theme_system")
        .activate({
            let theme_mgr = theme_manager.clone();
            let editor_clone = editor.clone();
            move |app: &Application, _action, _param| {
                theme_mgr.set_theme(crate::theme::Theme::System);
                editor_clone.refresh_html_view();
                settings::update_ui_theme("system");
                rebuild_menu_bar(app, &editor_clone, &theme_mgr);
                println!("Switched to system theme");
            }
        })
        .build();
        
    let theme_light_action = gio::ActionEntry::builder("theme_light")
        .activate({
            let theme_mgr = theme_manager.clone();
            let editor_clone = editor.clone();
            move |app: &Application, _action, _param| {
                theme_mgr.set_theme(crate::theme::Theme::Light);
                editor_clone.refresh_html_view();
                settings::update_ui_theme("light");
                rebuild_menu_bar(app, &editor_clone, &theme_mgr);
                println!("Switched to light theme");
            }
        })
        .build();
        
    let theme_dark_action = gio::ActionEntry::builder("theme_dark")
        .activate({
            let theme_mgr = theme_manager.clone();
            let editor_clone = editor.clone();
            move |app: &Application, _action, _param| {
                theme_mgr.set_theme(crate::theme::Theme::Dark);
                editor_clone.refresh_html_view();
                settings::update_ui_theme("dark");
                rebuild_menu_bar(app, &editor_clone, &theme_mgr);
                println!("Switched to dark theme");
            }
        })
        .build();

    // CSS Theme switching actions for preview (dynamic from css/ directory)
    let available_themes = editor::MarkdownEditor::get_available_css_themes();
    let mut css_theme_actions = Vec::new();
    
    for (theme_name, _display_name, sanitized_name) in available_themes {
        let action_name = format!("css_theme_{}", sanitized_name);
        let action = gio::ActionEntry::builder(&action_name)
            .activate({
                let editor = editor.clone();
                let theme_mgr = theme_manager.clone();
                let theme = theme_name.clone();
                move |app: &Application, _action, _param| {
                    editor.set_css_theme(&theme);
                    settings::update_css_theme(&theme);
                    rebuild_menu_bar(app, &editor, &theme_mgr);
                    println!("✓ Set CSS theme to {}", theme);
                }
            })
            .build();
        css_theme_actions.push(action);
    }

    // Advanced Markdown Syntax Actions (markdownguide.org/hacks/)
    
    let insert_underline_action = gio::ActionEntry::builder("insert_underline")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                show_underline_dialog(&editor);
            }
        })
        .build();

    let insert_center_text_action = gio::ActionEntry::builder("insert_center_text")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                show_center_text_dialog(&editor);
            }
        })
        .build();

    let insert_colored_text_action = gio::ActionEntry::builder("insert_colored_text")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                show_colored_text_dialog(&editor);
            }
        })
        .build();

    let insert_indent_text_action = gio::ActionEntry::builder("insert_indent_text")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_indented_text("indented text", 1);
            }
        })
        .build();

    let insert_comment_action = gio::ActionEntry::builder("insert_comment")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                show_comment_dialog(&editor);
            }
        })
        .build();

    let insert_admonition_action = gio::ActionEntry::builder("insert_admonition")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                show_admonition_dialog(&editor);
            }
        })
        .build();

    let insert_image_with_size_action = gio::ActionEntry::builder("insert_image_with_size")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                show_image_with_size_dialog(&editor);
            }
        })
        .build();

    let insert_image_with_caption_action = gio::ActionEntry::builder("insert_image_with_caption")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_image_with_caption("image.png", "Alt text", "Image caption");
            }
        })
        .build();

    let insert_link_with_target_action = gio::ActionEntry::builder("insert_link_with_target")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_link_with_target("https://example.com", "Link text", "_blank");
            }
        })
        .build();

    let insert_html_entity_action = gio::ActionEntry::builder("insert_html_entity")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_html_entity("copy");
            }
        })
        .build();

    let insert_table_of_contents_action = gio::ActionEntry::builder("insert_table_of_contents")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_table_of_contents();
            }
        })
        .build();

    let insert_youtube_video_action = gio::ActionEntry::builder("insert_youtube_video")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                show_youtube_video_dialog(&editor);
            }
        })
        .build();

    // Extended Syntax Actions
    let insert_task_list_action = gio::ActionEntry::builder("insert_task_list")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_task_list();
            }
        })
        .build();

    let insert_task_list_custom_action = gio::ActionEntry::builder("insert_task_list_custom")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                show_task_list_custom_dialog(&editor);
            }
        })
        .build();

    let insert_task_list_open_action = gio::ActionEntry::builder("insert_task_list_open")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_single_open_task();
            }
        })
        .build();

    let insert_task_list_closed_action = gio::ActionEntry::builder("insert_task_list_closed")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_single_closed_task();
            }
        })
        .build();

    let insert_footnote_action = gio::ActionEntry::builder("insert_footnote")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_footnote();
            }
        })
        .build();

    let insert_definition_list_action = gio::ActionEntry::builder("insert_definition_list")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_definition_list();
            }
        })
        .build();

    let insert_definition_list_custom_action = gio::ActionEntry::builder("insert_definition_list_custom")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                show_definition_list_custom_dialog(&editor);
            }
        })
        .build();

    let insert_definition_list_single_action = gio::ActionEntry::builder("insert_definition_list_single")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_single_definition();
            }
        })
        .build();

    let insert_highlight_action = gio::ActionEntry::builder("insert_highlight")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_highlight();
            }
        })
        .build();

    let insert_subscript_action = gio::ActionEntry::builder("insert_subscript")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_subscript();
            }
        })
        .build();

    let insert_superscript_action = gio::ActionEntry::builder("insert_superscript")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_superscript();
            }
        })
        .build();

    let insert_emoji_action = gio::ActionEntry::builder("insert_emoji")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                emoji::show_emoji_picker_dialog(&editor);
            }
        })
        .build();

    let insert_fenced_code_action = gio::ActionEntry::builder("insert_fenced_code")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_fenced_code_block();
            }
        })
        .build();

    // Language-specific fenced code actions
    let insert_fenced_javascript_action = gio::ActionEntry::builder("insert_fenced_javascript")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_fenced_code_with_language("javascript");
            }
        })
        .build();

    let insert_fenced_python_action = gio::ActionEntry::builder("insert_fenced_python")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_fenced_code_with_language("python");
            }
        })
        .build();

    let insert_fenced_java_action = gio::ActionEntry::builder("insert_fenced_java")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_fenced_code_with_language("java");
            }
        })
        .build();

    let insert_fenced_typescript_action = gio::ActionEntry::builder("insert_fenced_typescript")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_fenced_code_with_language("typescript");
            }
        })
        .build();

    let insert_fenced_csharp_action = gio::ActionEntry::builder("insert_fenced_csharp")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_fenced_code_with_language("csharp");
            }
        })
        .build();

    let insert_fenced_cpp_action = gio::ActionEntry::builder("insert_fenced_cpp")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_fenced_code_with_language("cpp");
            }
        })
        .build();

    let insert_fenced_c_action = gio::ActionEntry::builder("insert_fenced_c")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_fenced_code_with_language("c");
            }
        })
        .build();

    let insert_fenced_php_action = gio::ActionEntry::builder("insert_fenced_php")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_fenced_code_with_language("php");
            }
        })
        .build();

    let insert_fenced_go_action = gio::ActionEntry::builder("insert_fenced_go")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_fenced_code_with_language("go");
            }
        })
        .build();

    let insert_fenced_rust_action = gio::ActionEntry::builder("insert_fenced_rust")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_fenced_code_with_language("rust");
            }
        })
        .build();

    // Common markup/data language actions
    let insert_fenced_html_action = gio::ActionEntry::builder("insert_fenced_html")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_fenced_code_with_language("html");
            }
        })
        .build();

    let insert_fenced_css_action = gio::ActionEntry::builder("insert_fenced_css")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_fenced_code_with_language("css");
            }
        })
        .build();

    let insert_fenced_json_action = gio::ActionEntry::builder("insert_fenced_json")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_fenced_code_with_language("json");
            }
        })
        .build();

    let insert_fenced_xml_action = gio::ActionEntry::builder("insert_fenced_xml")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_fenced_code_with_language("xml");
            }
        })
        .build();

    let insert_fenced_sql_action = gio::ActionEntry::builder("insert_fenced_sql")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_fenced_code_with_language("sql");
            }
        })
        .build();

    let insert_fenced_bash_action = gio::ActionEntry::builder("insert_fenced_bash")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_fenced_code_with_language("bash");
            }
        })
        .build();

    let insert_fenced_yaml_action = gio::ActionEntry::builder("insert_fenced_yaml")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_fenced_code_with_language("yaml");
            }
        })
        .build();

    let insert_fenced_markdown_action = gio::ActionEntry::builder("insert_fenced_markdown")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_fenced_code_with_language("markdown");
            }
        })
        .build();
    
    let set_language_es_action = gio::ActionEntry::builder("set_language_es")
        .activate({
            let editor = editor.clone();
            let theme_mgr = theme_manager.clone();
            move |app: &Application, _action, _param| {
                localization::set_locale("es");
                settings::update_language("es");
                rebuild_menu_bar(app, &editor, &theme_mgr);
                println!("Language changed to Spanish");
            }
        })
        .build();
    
    let set_language_fr_action = gio::ActionEntry::builder("set_language_fr")
        .activate({
            let editor = editor.clone();
            let theme_mgr = theme_manager.clone();
            move |app: &Application, _action, _param| {
                localization::set_locale("fr");
                settings::update_language("fr");
                rebuild_menu_bar(app, &editor, &theme_mgr);
                println!("Language changed to French");
            }
        })
        .build();
    
    let set_language_de_action = gio::ActionEntry::builder("set_language_de")
        .activate({
            let editor = editor.clone();
            let theme_mgr = theme_manager.clone();
            move |app: &Application, _action, _param| {
                localization::set_locale("de");
                settings::update_language("de");
                rebuild_menu_bar(app, &editor, &theme_mgr);
                println!("Language changed to German");
            }
        })
        .build();
    
    // Add all actions to the application
    app.add_action_entries([
        new_action, open_action, save_action, save_as_action, quit_action,
        undo_action, redo_action, cut_action, copy_action, paste_action, find_action, replace_action,
        insert_header_action, insert_bold_action, insert_italic_action, insert_code_action, insert_inline_code_action,
        insert_bullet_list_action, insert_numbered_list_action, insert_blockquote_action,
        insert_link_action, insert_image_action, insert_hr_action,
        heading1_action, heading2_action, heading3_action, heading4_action, heading5_action, heading6_action,
        strikethrough_action, code_block_action, insert_table_action, insert_table_dialog_action,
        // Extended syntax actions
        insert_task_list_action, insert_task_list_custom_action, insert_task_list_open_action, 
        insert_task_list_closed_action, insert_footnote_action, insert_definition_list_action,
        insert_definition_list_custom_action, insert_definition_list_single_action,
        insert_highlight_action, insert_subscript_action, insert_superscript_action,
        insert_emoji_action, insert_fenced_code_action,
        // Language-specific fenced code actions
        insert_fenced_javascript_action, insert_fenced_python_action, insert_fenced_java_action,
        insert_fenced_typescript_action, insert_fenced_csharp_action, insert_fenced_cpp_action,
        insert_fenced_c_action, insert_fenced_php_action, insert_fenced_go_action, insert_fenced_rust_action,
        insert_fenced_html_action, insert_fenced_css_action, insert_fenced_json_action,
        insert_fenced_xml_action, insert_fenced_sql_action, insert_fenced_bash_action,
        insert_fenced_yaml_action, insert_fenced_markdown_action,
        // Advanced syntax actions
        insert_underline_action, insert_center_text_action, insert_colored_text_action,
        insert_indent_text_action, insert_comment_action, insert_admonition_action,
        insert_image_with_size_action, insert_image_with_caption_action, insert_link_with_target_action,
        insert_html_entity_action, insert_table_of_contents_action, insert_youtube_video_action,
        markdown_guide_action, shortcuts_action, about_action,
        set_language_en_action, set_language_es_action, set_language_fr_action, set_language_de_action,
        view_html_action, view_code_action,
        theme_system_action, theme_light_action, theme_dark_action,
    ]);
    
    // Add dynamic CSS theme actions
    for action in css_theme_actions {
        app.add_action_entries([action]);
    }
}

/// Show the keyboard shortcuts dialog
fn show_shortcuts_dialog(parent: &gtk4::Window) {
    let dialog = Dialog::with_buttons(
        Some(&localization::tr("shortcuts.title")),
        Some(parent),
        gtk4::DialogFlags::MODAL,
        &[("OK", ResponseType::Ok)],
    );
    
    dialog.set_default_size(500, 600);
    
    let content_area = dialog.content_area();
    let main_box = Box::new(Orientation::Vertical, 12);
    main_box.set_margin_top(16);
    main_box.set_margin_bottom(16);
    main_box.set_margin_start(16);
    main_box.set_margin_end(16);
    
    // Scroll window for the content
    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    scroll.set_vexpand(true);
    
    let shortcuts_box = Box::new(Orientation::Vertical, 16);
    
    // Basic Formatting Section
    let basic_section = create_shortcuts_section(
        &localization::tr("shortcuts.basic_formatting"),
        &[
            (&localization::tr("shortcuts.ctrl_b"), &localization::tr("shortcuts.bold_text")),
            (&localization::tr("shortcuts.ctrl_i"), &localization::tr("shortcuts.italic_text")),
            (&localization::tr("shortcuts.ctrl_u"), &localization::tr("shortcuts.strikethrough_text")),
            (&localization::tr("shortcuts.ctrl_k"), &localization::tr("shortcuts.insert_link")),
            (&localization::tr("shortcuts.ctrl_backtick"), &localization::tr("shortcuts.inline_code")),
        ]
    );
    shortcuts_box.append(&basic_section);
    
    // Headings Section
    let headings_section = create_shortcuts_section(
        &localization::tr("shortcuts.headings"),
        &[
            (&localization::tr("shortcuts.ctrl_1"), &localization::tr("shortcuts.heading_1")),
            (&localization::tr("shortcuts.ctrl_2"), &localization::tr("shortcuts.heading_2")),
            (&localization::tr("shortcuts.ctrl_3"), &localization::tr("shortcuts.heading_3")),
            (&localization::tr("shortcuts.ctrl_4"), &localization::tr("shortcuts.heading_4")),
            (&localization::tr("shortcuts.ctrl_5"), &localization::tr("shortcuts.heading_5")),
            (&localization::tr("shortcuts.ctrl_6"), &localization::tr("shortcuts.heading_6")),
        ]
    );
    shortcuts_box.append(&headings_section);
    
    // Lists and Quotes Section
    let lists_section = create_shortcuts_section(
        &localization::tr("shortcuts.lists_and_quotes"),
        &[
            (&localization::tr("shortcuts.ctrl_shift_8"), &localization::tr("shortcuts.bullet_list")),
            (&localization::tr("shortcuts.ctrl_shift_7"), &localization::tr("shortcuts.numbered_list")),
            (&localization::tr("shortcuts.ctrl_shift_period"), &localization::tr("shortcuts.blockquote")),
        ]
    );
    shortcuts_box.append(&lists_section);
    
    scroll.set_child(Some(&shortcuts_box));
    main_box.append(&scroll);
    content_area.append(&main_box);
    
    dialog.show();
    
    dialog.connect_response(|dialog, _response| {
        dialog.close();
    });
}

/// Create a shortcuts section with a title and list of shortcuts
fn create_shortcuts_section(title: &str, shortcuts: &[(&str, &str)]) -> gtk4::Widget {
    let section_box = Box::new(Orientation::Vertical, 8);
    
    // Section title
    let title_label = Label::new(Some(title));
    title_label.set_halign(gtk4::Align::Start);
    title_label.add_css_class("heading");
    title_label.set_markup(&format!("<b>{}</b>", title));
    section_box.append(&title_label);
    
    // Shortcuts grid
    let grid = Grid::new();
    grid.set_row_spacing(6);
    grid.set_column_spacing(20);
    grid.set_margin_start(16);
    
    for (row, (shortcut, description)) in shortcuts.iter().enumerate() {
        // Shortcut key
        let shortcut_label = Label::new(Some(shortcut));
        shortcut_label.set_halign(gtk4::Align::Start);
        shortcut_label.add_css_class("caption");
        shortcut_label.set_markup(&format!("<tt><b>{}</b></tt>", shortcut));
        grid.attach(&shortcut_label, 0, row as i32, 1, 1);
        
        // Description
        let desc_label = Label::new(Some(description));
        desc_label.set_halign(gtk4::Align::Start);
        grid.attach(&desc_label, 1, row as i32, 1, 1);
    }
    
    section_box.append(&grid);
    section_box.upcast()
}

/// Set up keyboard accelerators for menu actions
fn setup_menu_accelerators(app: &Application) {
    // Basic formatting shortcuts
    app.set_accels_for_action("app.insert_bold", &["<Control>b"]);
    app.set_accels_for_action("app.insert_italic", &["<Control>i"]);
    app.set_accels_for_action("app.strikethrough", &["<Control>u"]);
    app.set_accels_for_action("app.insert_link", &["<Control>k"]);
    app.set_accels_for_action("app.insert_inline_code", &["<Control>grave"]);
    
    // Heading shortcuts
    app.set_accels_for_action("app.heading1", &["<Control>1"]);
    app.set_accels_for_action("app.heading2", &["<Control>2"]);
    app.set_accels_for_action("app.heading3", &["<Control>3"]);
    app.set_accels_for_action("app.heading4", &["<Control>4"]);
    app.set_accels_for_action("app.heading5", &["<Control>5"]);
    app.set_accels_for_action("app.heading6", &["<Control>6"]);
    
    // List and quote shortcuts
    app.set_accels_for_action("app.insert_bullet_list", &["<Control><Shift>8"]);
    app.set_accels_for_action("app.insert_numbered_list", &["<Control><Shift>7"]);
    app.set_accels_for_action("app.insert_blockquote", &["<Control><Shift>period"]);
    
    // File shortcuts
    app.set_accels_for_action("app.quit", &["<Control>q"]);
}

/// Show dialog to create a custom task list with specified number of items
pub fn show_task_list_custom_dialog(editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&localization::tr("insert.task_list_custom")),
        None::<&gtk4::Window>,
        gtk4::DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
    );
    
    dialog.set_default_size(350, 200);
    
    // Create the grid layout
    let grid = Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(12);
    grid.set_margin_top(20);
    grid.set_margin_bottom(20);
    grid.set_margin_start(20);
    grid.set_margin_end(20);
    
    // Number of items label and spin button
    let items_label = Label::new(Some("Number of tasks:"));
    items_label.set_halign(gtk4::Align::Start);
    
    // Create spin button with range 1-50, default 3
    let adjustment = Adjustment::new(3.0, 1.0, 50.0, 1.0, 5.0, 0.0);
    let items_spin = SpinButton::new(Some(&adjustment), 1.0, 0);
    items_spin.set_hexpand(true);
    
    // Preview label
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    
    // Preview text with scrolled window
    let preview_text = gtk4::TextView::new();
    preview_text.set_editable(false);
    preview_text.set_cursor_visible(false);
    
    let preview_scroll = gtk4::ScrolledWindow::new();
    preview_scroll.set_child(Some(&preview_text));
    preview_scroll.set_size_request(300, 100);
    preview_scroll.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    
    // Update preview when spin button value changes
    let preview_buffer = preview_text.buffer();
    let update_preview = {
        let items_spin = items_spin.clone();
        let preview_buffer = preview_buffer.clone();
        move || {
            let count = items_spin.value() as usize;
            let mut preview = String::new();
            for i in 0..count.min(10) { // Show max 10 in preview
                preview.push_str(&format!("[ ] Task {}\n", i + 1));
            }
            if count > 10 {
                preview.push_str(&format!("... and {} more tasks", count - 10));
            }
            preview_buffer.set_text(&preview);
        }
    };
    
    // Initial preview update
    update_preview();
    
    // Connect value changed signal
    items_spin.connect_value_changed({
        let update_preview = update_preview.clone();
        move |_| {
            update_preview();
        }
    });
    
    // Add to grid
    grid.attach(&items_label, 0, 0, 1, 1);
    grid.attach(&items_spin, 1, 0, 1, 1);
    grid.attach(&preview_label, 0, 1, 2, 1);
    grid.attach(&preview_scroll, 0, 2, 2, 1);
    
    // Add grid to dialog
    dialog.content_area().append(&grid);
    
    // Focus on spin button
    items_spin.grab_focus();
    
    // Connect response
    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let count = items_spin.value() as usize;
            if count > 0 {
                editor_clone.insert_custom_task_list(count);
            }
        }
        dialog.close();
    });
    
    dialog.present();
}

/// Show dialog to create a custom definition list with specified number of items
pub fn show_definition_list_custom_dialog(editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&localization::tr("insert.definition_list_custom")),
        None::<&gtk4::Window>,
        gtk4::DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
    );
    
    dialog.set_default_size(400, 250);
    
    // Create the grid layout
    let grid = Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(12);
    grid.set_margin_top(20);
    grid.set_margin_bottom(20);
    grid.set_margin_start(20);
    grid.set_margin_end(20);
    
    // Number of items label and spin button
    let items_label = Label::new(Some("Number of definition pairs:"));
    items_label.set_halign(gtk4::Align::Start);
    
    // Create spin button with range 1-20, default 2
    let adjustment = Adjustment::new(2.0, 1.0, 20.0, 1.0, 5.0, 0.0);
    let items_spin = SpinButton::new(Some(&adjustment), 1.0, 0);
    items_spin.set_hexpand(true);
    
    // Preview label
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    
    // Preview text with scrolled window
    let preview_text = gtk4::TextView::new();
    preview_text.set_editable(false);
    preview_text.set_cursor_visible(false);
    
    let preview_scroll = gtk4::ScrolledWindow::new();
    preview_scroll.set_child(Some(&preview_text));
    preview_scroll.set_size_request(350, 120);
    preview_scroll.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    
    // Update preview when spin button value changes
    let preview_buffer = preview_text.buffer();
    let update_preview = {
        let items_spin = items_spin.clone();
        let preview_buffer = preview_buffer.clone();
        move || {
            let count = items_spin.value() as usize;
            let mut preview = String::new();
            for i in 0..count.min(8) { // Show max 8 in preview
                if i > 0 {
                    preview.push('\n');
                }
                preview.push_str(&format!("Term {}\n: Definition of term {}.\n", i + 1, i + 1));
            }
            if count > 8 {
                preview.push_str(&format!("\n... and {} more definition pairs", count - 8));
            }
            preview_buffer.set_text(&preview);
        }
    };
    
    // Initial preview update
    update_preview();
    
    // Connect value changed signal
    items_spin.connect_value_changed({
        let update_preview = update_preview.clone();
        move |_| {
            update_preview();
        }
    });
    
    // Add to grid
    grid.attach(&items_label, 0, 0, 1, 1);
    grid.attach(&items_spin, 1, 0, 1, 1);
    grid.attach(&preview_label, 0, 1, 2, 1);
    grid.attach(&preview_scroll, 0, 2, 2, 1);
    
    // Add grid to dialog
    dialog.content_area().append(&grid);
    
    // Focus on spin button
    items_spin.grab_focus();
    
    // Connect response
    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let count = items_spin.value() as usize;
            if count > 0 {
                editor_clone.insert_custom_definition_list(count);
            }
        }
        dialog.close();
    });
    
    dialog.present();
}

/// Show dialog to insert colored text
pub fn show_colored_text_dialog(editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&localization::tr("advanced.colored_text")),
        None::<&gtk4::Window>,
        gtk4::DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
    );
    
    dialog.set_default_size(400, 200);
    
    let grid = Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(12);
    grid.set_margin_top(20);
    grid.set_margin_bottom(20);
    grid.set_margin_start(20);
    grid.set_margin_end(20);
    
    // Text input
    let text_label = Label::new(Some("Text to color:"));
    text_label.set_halign(gtk4::Align::Start);
    let text_entry = Entry::new();
    text_entry.set_placeholder_text(Some("Enter text here"));
    text_entry.set_hexpand(true);
    
    // Color input
    let color_label = Label::new(Some("Color (hex or name):"));
    color_label.set_halign(gtk4::Align::Start);
    let color_entry = Entry::new();
    color_entry.set_placeholder_text(Some("#ff0000 or red"));
    color_entry.set_text("#ff0000");
    
    // Preview
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    let preview_text = gtk4::TextView::new();
    preview_text.set_editable(false);
    preview_text.set_size_request(350, 60);
    
    // Update preview function
    let update_preview = {
        let text_entry = text_entry.clone();
        let color_entry = color_entry.clone();
        let preview_buffer = preview_text.buffer();
        move || {
            let text = text_entry.text();
            let color = color_entry.text();
            let preview = format!("<span style=\"color: {}\">{}</span>", color, 
                                if text.is_empty() { "Sample text" } else { &text });
            preview_buffer.set_text(&preview);
        }
    };
    
    // Initial preview
    update_preview();
    
    // Connect change signals
    text_entry.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });
    color_entry.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });
    
    grid.attach(&text_label, 0, 0, 1, 1);
    grid.attach(&text_entry, 1, 0, 1, 1);
    grid.attach(&color_label, 0, 1, 1, 1);
    grid.attach(&color_entry, 1, 1, 1, 1);
    grid.attach(&preview_label, 0, 2, 2, 1);
    grid.attach(&preview_text, 0, 3, 2, 1);
    
    dialog.content_area().append(&grid);
    text_entry.grab_focus();
    
    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let text = text_entry.text();
            let color = color_entry.text();
            if !text.is_empty() && !color.is_empty() {
                editor_clone.insert_colored_text(&text, &color);
            }
        }
        dialog.close();
    });
    
    dialog.present();
}

/// Show dialog to insert underlined text
pub fn show_underline_dialog(editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&localization::tr("advanced.underline")),
        None::<&gtk4::Window>,
        gtk4::DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
    );
    
    dialog.set_default_size(350, 150);
    
    let grid = Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(12);
    grid.set_margin_top(20);
    grid.set_margin_bottom(20);
    grid.set_margin_start(20);
    grid.set_margin_end(20);
    
    let text_label = Label::new(Some("Text to underline:"));
    text_label.set_halign(gtk4::Align::Start);
    let text_entry = Entry::new();
    text_entry.set_placeholder_text(Some("Enter text here"));
    text_entry.set_hexpand(true);
    
    // Preview
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    let preview_text = gtk4::TextView::new();
    preview_text.set_editable(false);
    preview_text.set_size_request(300, 50);
    
    let update_preview = {
        let text_entry = text_entry.clone();
        let preview_buffer = preview_text.buffer();
        move || {
            let text = text_entry.text();
            let preview = format!("<u>{}</u>", if text.is_empty() { "Sample text" } else { &text });
            preview_buffer.set_text(&preview);
        }
    };
    
    update_preview();
    text_entry.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });
    
    grid.attach(&text_label, 0, 0, 1, 1);
    grid.attach(&text_entry, 1, 0, 1, 1);
    grid.attach(&preview_label, 0, 1, 2, 1);
    grid.attach(&preview_text, 0, 2, 2, 1);
    
    dialog.content_area().append(&grid);
    text_entry.grab_focus();
    
    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let text = text_entry.text();
            if !text.is_empty() {
                editor_clone.insert_underline(&text);
            }
        }
        dialog.close();
    });
    
    dialog.present();
}

/// Show dialog to center text
pub fn show_center_text_dialog(editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&localization::tr("advanced.center_text")),
        None::<&gtk4::Window>,
        gtk4::DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
    );
    
    dialog.set_default_size(350, 150);
    
    let grid = Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(12);
    grid.set_margin_top(20);
    grid.set_margin_bottom(20);
    grid.set_margin_start(20);
    grid.set_margin_end(20);
    
    let text_label = Label::new(Some("Text to center:"));
    text_label.set_halign(gtk4::Align::Start);
    let text_entry = Entry::new();
    text_entry.set_placeholder_text(Some("Enter text here"));
    text_entry.set_hexpand(true);
    
    // Preview
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    let preview_text = gtk4::TextView::new();
    preview_text.set_editable(false);
    preview_text.set_size_request(300, 50);
    
    let update_preview = {
        let text_entry = text_entry.clone();
        let preview_buffer = preview_text.buffer();
        move || {
            let text = text_entry.text();
            let preview = format!("<div align=\"center\">{}</div>", 
                                if text.is_empty() { "Sample text" } else { &text });
            preview_buffer.set_text(&preview);
        }
    };
    
    update_preview();
    text_entry.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });
    
    grid.attach(&text_label, 0, 0, 1, 1);
    grid.attach(&text_entry, 1, 0, 1, 1);
    grid.attach(&preview_label, 0, 1, 2, 1);
    grid.attach(&preview_text, 0, 2, 2, 1);
    
    dialog.content_area().append(&grid);
    text_entry.grab_focus();
    
    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let text = text_entry.text();
            if !text.is_empty() {
                editor_clone.insert_center_text(&text);
            }
        }
        dialog.close();
    });
    
    dialog.present();
}

/// Show dialog to insert a comment
pub fn show_comment_dialog(editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&localization::tr("advanced.comment")),
        None::<&gtk4::Window>,
        gtk4::DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
    );
    
    dialog.set_default_size(400, 200);
    
    let grid = Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(12);
    grid.set_margin_top(20);
    grid.set_margin_bottom(20);
    grid.set_margin_start(20);
    grid.set_margin_end(20);
    
    let text_label = Label::new(Some("Comment text:"));
    text_label.set_halign(gtk4::Align::Start);
    
    // Use TextView for multi-line comment
    let text_view = gtk4::TextView::new();
    text_view.set_size_request(350, 80);
    let text_buffer = text_view.buffer();
    text_buffer.set_text("Your comment here...");
    
    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_child(Some(&text_view));
    scroll.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    
    // Preview
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    let preview_text = gtk4::TextView::new();
    preview_text.set_editable(false);
    preview_text.set_size_request(350, 50);
    
    let update_preview = {
        let text_buffer = text_buffer.clone();
        let preview_buffer = preview_text.buffer();
        move || {
            let text = text_buffer.text(&text_buffer.start_iter(), &text_buffer.end_iter(), false);
            let preview = format!("<!-- {} -->", text);
            preview_buffer.set_text(&preview);
        }
    };
    
    update_preview();
    text_buffer.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });
    
    grid.attach(&text_label, 0, 0, 2, 1);
    grid.attach(&scroll, 0, 1, 2, 1);
    grid.attach(&preview_label, 0, 2, 2, 1);
    grid.attach(&preview_text, 0, 3, 2, 1);
    
    dialog.content_area().append(&grid);
    text_view.grab_focus();
    
    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let text = text_buffer.text(&text_buffer.start_iter(), &text_buffer.end_iter(), false);
            if !text.trim().is_empty() {
                editor_clone.insert_comment(&text);
            }
        }
        dialog.close();
    });
    
    dialog.present();
}

/// Show dialog to insert an admonition (GitHub-style callout)
pub fn show_admonition_dialog(editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&localization::tr("advanced.admonition")),
        None::<&gtk4::Window>,
        gtk4::DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
    );
    
    dialog.set_default_size(450, 300);
    
    let grid = Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(12);
    grid.set_margin_top(20);
    grid.set_margin_bottom(20);
    grid.set_margin_start(20);
    grid.set_margin_end(20);
    
    // Type selection
    let type_label = Label::new(Some("Admonition type:"));
    type_label.set_halign(gtk4::Align::Start);
    
    let type_combo = gtk4::ComboBoxText::new();
    type_combo.append(Some("NOTE"), "Note");
    type_combo.append(Some("TIP"), "Tip");
    type_combo.append(Some("IMPORTANT"), "Important");
    type_combo.append(Some("WARNING"), "Warning");
    type_combo.append(Some("CAUTION"), "Caution");
    type_combo.set_active_id(Some("NOTE"));
    
    // Content input
    let content_label = Label::new(Some("Content:"));
    content_label.set_halign(gtk4::Align::Start);
    
    let content_view = gtk4::TextView::new();
    content_view.set_size_request(400, 100);
    let content_buffer = content_view.buffer();
    content_buffer.set_text("Add your content here...");
    
    let content_scroll = gtk4::ScrolledWindow::new();
    content_scroll.set_child(Some(&content_view));
    content_scroll.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    
    // Preview
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    let preview_text = gtk4::TextView::new();
    preview_text.set_editable(false);
    preview_text.set_size_request(400, 80);
    
    let preview_scroll2 = gtk4::ScrolledWindow::new();
    preview_scroll2.set_child(Some(&preview_text));
    preview_scroll2.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    
    let update_preview = {
        let type_combo = type_combo.clone();
        let content_buffer = content_buffer.clone();
        let preview_buffer = preview_text.buffer();
        move || {
            let admonition_type = type_combo.active_id().unwrap_or_default();
            let content = content_buffer.text(&content_buffer.start_iter(), &content_buffer.end_iter(), false);
            let preview = format!("> [!{}]\n> {}", admonition_type, 
                                content.lines().collect::<Vec<_>>().join("\n> "));
            preview_buffer.set_text(&preview);
        }
    };
    
    update_preview();
    type_combo.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });
    content_buffer.connect_changed({
        let update_preview = update_preview.clone();
        move |_| update_preview()
    });
    
    grid.attach(&type_label, 0, 0, 1, 1);
    grid.attach(&type_combo, 1, 0, 1, 1);
    grid.attach(&content_label, 0, 1, 2, 1);
    grid.attach(&content_scroll, 0, 2, 2, 1);
    grid.attach(&preview_label, 0, 3, 2, 1);
    grid.attach(&preview_scroll2, 0, 4, 2, 1);
    
    dialog.content_area().append(&grid);
    content_view.grab_focus();
    
    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let admonition_type = type_combo.active_id().unwrap_or_default();
            let content = content_buffer.text(&content_buffer.start_iter(), &content_buffer.end_iter(), false);
            if !content.trim().is_empty() {
                // Use emoji placeholder since the method expects it
                editor_clone.insert_admonition("", &admonition_type, &content);
            }
        }
        dialog.close();
    });
    
    dialog.present();
}

/// Show dialog to insert image with size
pub fn show_image_with_size_dialog(editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&localization::tr("advanced.image_with_size")),
        None::<&gtk4::Window>,
        gtk4::DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
    );
    
    dialog.set_default_size(450, 250);
    
    let grid = Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(12);
    grid.set_margin_top(20);
    grid.set_margin_bottom(20);
    grid.set_margin_start(20);
    grid.set_margin_end(20);
    
    // URL input
    let url_label = Label::new(Some("Image URL:"));
    url_label.set_halign(gtk4::Align::Start);
    let url_entry = Entry::new();
    url_entry.set_placeholder_text(Some("https://example.com/image.jpg"));
    url_entry.set_hexpand(true);
    
    // Alt text input
    let alt_label = Label::new(Some("Alt text:"));
    alt_label.set_halign(gtk4::Align::Start);
    let alt_entry = Entry::new();
    alt_entry.set_placeholder_text(Some("Description of the image"));
    
    // Width input
    let width_label = Label::new(Some("Width (px):"));
    width_label.set_halign(gtk4::Align::Start);
    let width_adjustment = Adjustment::new(300.0, 1.0, 2000.0, 1.0, 10.0, 0.0);
    let width_spin = SpinButton::new(Some(&width_adjustment), 1.0, 0);
    
    // Height input
    let height_label = Label::new(Some("Height (px):"));
    height_label.set_halign(gtk4::Align::Start);
    let height_adjustment = Adjustment::new(200.0, 1.0, 2000.0, 1.0, 10.0, 0.0);
    let height_spin = SpinButton::new(Some(&height_adjustment), 1.0, 0);
    
    // Preview
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    let preview_text = gtk4::TextView::new();
    preview_text.set_editable(false);
    preview_text.set_size_request(400, 60);
    
    let update_preview = {
        let url_entry = url_entry.clone();
        let alt_entry = alt_entry.clone();
        let width_spin = width_spin.clone();
        let height_spin = height_spin.clone();
        let preview_buffer = preview_text.buffer();
        move || {
            let url = url_entry.text();
            let alt = alt_entry.text();
            let width = width_spin.value() as i32;
            let height = height_spin.value() as i32;
            let preview = format!(
                "<img src=\"{}\" alt=\"{}\" width=\"{}\" height=\"{}\">",
                if url.is_empty() { "image-url" } else { &url },
                if alt.is_empty() { "alt-text" } else { &alt },
                width, height
            );
            preview_buffer.set_text(&preview);
        }
    };
    
    // Initial preview and connect signals
    update_preview();
    url_entry.connect_changed({let update_preview = update_preview.clone(); move |_| update_preview()});
    alt_entry.connect_changed({let update_preview = update_preview.clone(); move |_| update_preview()});
    width_spin.connect_value_changed({let update_preview = update_preview.clone(); move |_| update_preview()});
    height_spin.connect_value_changed({let update_preview = update_preview.clone(); move |_| update_preview()});
    
    grid.attach(&url_label, 0, 0, 1, 1);
    grid.attach(&url_entry, 1, 0, 1, 1);
    grid.attach(&alt_label, 0, 1, 1, 1);
    grid.attach(&alt_entry, 1, 1, 1, 1);
    grid.attach(&width_label, 0, 2, 1, 1);
    grid.attach(&width_spin, 1, 2, 1, 1);
    grid.attach(&height_label, 0, 3, 1, 1);
    grid.attach(&height_spin, 1, 3, 1, 1);
    grid.attach(&preview_label, 0, 4, 2, 1);
    grid.attach(&preview_text, 0, 5, 2, 1);
    
    dialog.content_area().append(&grid);
    url_entry.grab_focus();
    
    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let url = url_entry.text();
            let alt = alt_entry.text();
            let width = width_spin.value() as i32;
            let height = height_spin.value() as i32;
            if !url.is_empty() {
                editor_clone.insert_image_with_size(&url, &alt, Some(&width.to_string()), Some(&height.to_string()));
            }
        }
        dialog.close();
    });
    
    dialog.present();
}

/// Show dialog to insert YouTube video
pub fn show_youtube_video_dialog(editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&localization::tr("advanced.youtube_video")),
        None::<&gtk4::Window>,
        gtk4::DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Insert", ResponseType::Accept)],
    );
    
    dialog.set_default_size(450, 200);
    
    let grid = Grid::new();
    grid.set_row_spacing(12);
    grid.set_column_spacing(12);
    grid.set_margin_top(20);
    grid.set_margin_bottom(20);
    grid.set_margin_start(20);
    grid.set_margin_end(20);
    
    // Video ID input
    let id_label = Label::new(Some("YouTube Video ID:"));
    id_label.set_halign(gtk4::Align::Start);
    let id_entry = Entry::new();
    id_entry.set_placeholder_text(Some("dQw4w9WgXcQ (from URL)"));
    id_entry.set_hexpand(true);
    
    // Title input
    let title_label = Label::new(Some("Video title (optional):"));
    title_label.set_halign(gtk4::Align::Start);
    let title_entry = Entry::new();
    title_entry.set_placeholder_text(Some("Video title"));
    
    // Help text
    let help_label = Label::new(Some("Extract the video ID from the YouTube URL (e.g., from\nhttps://www.youtube.com/watch?v=dQw4w9WgXcQ extract 'dQw4w9WgXcQ')"));
    help_label.set_halign(gtk4::Align::Start);
    help_label.add_css_class("caption");
    
    // Preview
    let preview_label = Label::new(Some("Preview:"));
    preview_label.set_halign(gtk4::Align::Start);
    let preview_text = gtk4::TextView::new();
    preview_text.set_editable(false);
    preview_text.set_size_request(400, 60);
    
    let update_preview = {
        let id_entry = id_entry.clone();
        let title_entry = title_entry.clone();
        let preview_buffer = preview_text.buffer();
        move || {
            let id = id_entry.text();
            let title = title_entry.text();
            let preview = if !id.is_empty() {
                let display_title = if title.is_empty() { "YouTube Video" } else { &title };
                format!("[![{}](https://img.youtube.com/vi/{}/0.jpg)](https://www.youtube.com/watch?v={})", 
                       display_title, id, id)
            } else {
                "[![Video Title](https://img.youtube.com/vi/VIDEO_ID/0.jpg)](https://www.youtube.com/watch?v=VIDEO_ID)".to_string()
            };
            preview_buffer.set_text(&preview);
        }
    };
    
    update_preview();
    id_entry.connect_changed({let update_preview = update_preview.clone(); move |_| update_preview()});
    title_entry.connect_changed({let update_preview = update_preview.clone(); move |_| update_preview()});
    
    grid.attach(&id_label, 0, 0, 1, 1);
    grid.attach(&id_entry, 1, 0, 1, 1);
    grid.attach(&title_label, 0, 1, 1, 1);
    grid.attach(&title_entry, 1, 1, 1, 1);
    grid.attach(&help_label, 0, 2, 2, 1);
    grid.attach(&preview_label, 0, 3, 2, 1);
    grid.attach(&preview_text, 0, 4, 2, 1);
    
    dialog.content_area().append(&grid);
    id_entry.grab_focus();
    
    let editor_clone = editor.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let id = id_entry.text();
            let title = title_entry.text();
            if !id.is_empty() {
                let display_title = if title.is_empty() { "YouTube Video" } else { &title };
                editor_clone.insert_youtube_video(&id, display_title);
            }
        }
        dialog.close();
    });
    
    dialog.present();
}

// Function to create just the View menu (using settings for state)
pub fn create_view_menu(_editor: &editor::MarkdownEditor, _theme_manager: &crate::theme::ThemeManager) -> gio::Menu {
    let view_menu = gio::Menu::new();
    
    // Get current settings
    let current_settings = settings::get_current_settings();
    
    // Add view mode submenu
    let view_mode_menu = gio::Menu::new();
    
    // Create view mode menu items with checkmarks based on settings
    let html_label = if current_settings.view_mode == "html" {
        "HTML\t✓"
    } else {
        "HTML"
    };
    view_mode_menu.append(Some(html_label), Some("app.view_html"));
    
    let code_label = if current_settings.view_mode == "code" {
        "HTML Code\t✓"
    } else {
        "HTML Code"
    };
    view_mode_menu.append(Some(code_label), Some("app.view_code"));
    
    view_menu.append_submenu(Some("Preview Mode"), &view_mode_menu);
    
    // Add CSS theme submenu for preview styling (dynamic from css/ directory)
    let css_theme_menu = gio::Menu::new();
    
    // Get available CSS themes dynamically
    let available_themes = editor::MarkdownEditor::get_available_css_themes();
    for (theme_name, display_name, sanitized_name) in available_themes {
        let _icon = match theme_name.as_str() {
            "standard" => "✓ ",
            "github" => "� ",
            "minimal" => "📄 ",
            "academic" => "🎓 ",
            _ => "🎨 ",
        };
        
        // Create menu item with checkmark based on settings
        let display_label = if current_settings.css_theme == theme_name {
            format!("{}\t✓", display_name)
        } else {
            display_name.clone()
        };
        css_theme_menu.append(Some(&display_label), Some(&format!("app.css_theme_{}", sanitized_name)));
    }
    
    view_menu.append_submenu(Some("CSS Style"), &css_theme_menu);
    
    // Add theme submenu (for UI theme)
    let theme_menu = gio::Menu::new();
    
    // Create theme menu items with checkmarks based on settings
    let system_label = if current_settings.ui_theme == "system" {
        "System\t✓"
    } else {
        "System"
    };
    theme_menu.append(Some(system_label), Some("app.theme_system"));
    
    let light_label = if current_settings.ui_theme == "light" {
        "Light\t✓"
    } else {
        "Light"
    };
    theme_menu.append(Some(light_label), Some("app.theme_light"));
    
    let dark_label = if current_settings.ui_theme == "dark" {
        "Dark\t✓"
    } else {
        "Dark"
    };
    theme_menu.append(Some(dark_label), Some("app.theme_dark"));
    
    view_menu.append_submenu(Some("Theme"), &theme_menu);
    
    let language_menu = gio::Menu::new();
    
    for (code, name) in localization::get_available_locales() {
        let lang_label = if current_settings.language == code {
            format!("{}\t✓", name)
        } else {
            name.to_string()
        };
        language_menu.append(Some(&lang_label), Some(&format!("app.set_language_{}", code)));
    }
    view_menu.append_submenu(Some(&localization::tr("menu.language")), &language_menu);
    
    view_menu
}

// Function to rebuild the entire menu bar when settings change
pub fn rebuild_menu_bar(app: &Application, editor: &editor::MarkdownEditor, theme_manager: &crate::theme::ThemeManager) {
    if let Some(window) = app.active_window() {
        // Get the main content area (which should be a Box)
        if let Some(main_box) = window.child().and_then(|c| c.downcast::<gtk4::Box>().ok()) {
            // Find and remove the existing menu bar (should be the first child)
            let mut menu_bar_widget: Option<PopoverMenuBar> = None;
            let mut child = main_box.first_child();
            while let Some(current_child) = child {
                if let Ok(menu_bar) = current_child.clone().downcast::<PopoverMenuBar>() {
                    menu_bar_widget = Some(menu_bar);
                    break;
                }
                child = current_child.next_sibling();
            }
            
            // Remove the old menu bar and create a new one
            if let Some(old_menu_bar) = menu_bar_widget {
                main_box.remove(&old_menu_bar);
                
                // Create a new menu bar with updated checkmarks
                let new_menu_bar = create_menu_bar(app, editor, theme_manager);
                main_box.prepend(&new_menu_bar);
            }
        }
    }
}