use gtk4::prelude::*;
use gtk4::{PopoverMenuBar, Application, gio, Dialog, Grid, Entry, ResponseType, Box, Orientation, Label, SpinButton, Adjustment};
use crate::{editor, localization};

pub fn create_menu_bar(app: &Application, editor: &editor::MarkdownEditor) -> PopoverMenuBar {
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
    insert_menu.append(Some(&localization::tr("insert.heading1")), Some("app.heading1"));
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
    // Add Headings submenu
    let headings_menu = gio::Menu::new();
    headings_menu.append(Some(&localization::tr("insert.heading1")), Some("app.heading1"));
    headings_menu.append(Some(&localization::tr("insert.heading2")), Some("app.heading2"));
    headings_menu.append(Some(&localization::tr("insert.heading3")), Some("app.heading3"));
    headings_menu.append(Some(&localization::tr("insert.heading4")), Some("app.heading4"));
    headings_menu.append(Some(&localization::tr("insert.heading5")), Some("app.heading5"));
    headings_menu.append(Some(&localization::tr("insert.heading6")), Some("app.heading6"));
    format_menu.append_submenu(Some(&localization::tr("insert.headings")), &headings_menu);
    
    // Text formatting
    format_menu.append(Some(&localization::tr("insert.strikethrough")), Some("app.strikethrough"));
    format_menu.append(Some(&localization::tr("insert.highlight")), Some("app.insert_highlight"));
    format_menu.append(Some(&localization::tr("insert.subscript")), Some("app.insert_subscript"));
    format_menu.append(Some(&localization::tr("insert.superscript")), Some("app.insert_superscript"));
    
    // Code
    format_menu.append(Some(&localization::tr("insert.code_block")), Some("app.code_block"));
    
    // Fenced Code submenu with languages
    let fenced_code_menu = gio::Menu::new();
    fenced_code_menu.append(Some(&localization::tr("insert.fenced_code_dialog")), Some("app.insert_fenced_code"));
    fenced_code_menu.append(None, None); // Separator
    
    // Add top 10 programming languages
    fenced_code_menu.append(Some("JavaScript"), Some("app.insert_fenced_javascript"));
    fenced_code_menu.append(Some("Python"), Some("app.insert_fenced_python"));
    fenced_code_menu.append(Some("Java"), Some("app.insert_fenced_java"));
    fenced_code_menu.append(Some("TypeScript"), Some("app.insert_fenced_typescript"));
    fenced_code_menu.append(Some("C#"), Some("app.insert_fenced_csharp"));
    fenced_code_menu.append(Some("C++"), Some("app.insert_fenced_cpp"));
    fenced_code_menu.append(Some("C"), Some("app.insert_fenced_c"));
    fenced_code_menu.append(Some("PHP"), Some("app.insert_fenced_php"));
    fenced_code_menu.append(Some("Go"), Some("app.insert_fenced_go"));
    fenced_code_menu.append(Some("Rust"), Some("app.insert_fenced_rust"));
    fenced_code_menu.append(None, None); // Separator
    
    // Common markup/data languages
    fenced_code_menu.append(Some("HTML"), Some("app.insert_fenced_html"));
    fenced_code_menu.append(Some("CSS"), Some("app.insert_fenced_css"));
    fenced_code_menu.append(Some("JSON"), Some("app.insert_fenced_json"));
    fenced_code_menu.append(Some("XML"), Some("app.insert_fenced_xml"));
    fenced_code_menu.append(Some("SQL"), Some("app.insert_fenced_sql"));
    fenced_code_menu.append(Some("Bash"), Some("app.insert_fenced_bash"));
    fenced_code_menu.append(Some("YAML"), Some("app.insert_fenced_yaml"));
    fenced_code_menu.append(Some("Markdown"), Some("app.insert_fenced_markdown"));
    
    format_menu.append_submenu(Some(&localization::tr("insert.fenced_code")), &fenced_code_menu);
    
    // Lists and tasks
    let task_list_menu = gio::Menu::new();
    task_list_menu.append(Some(&localization::tr("insert.task_list_custom")), Some("app.insert_task_list_custom"));
    task_list_menu.append(Some(&localization::tr("insert.task_list_open")), Some("app.insert_task_list_open"));
    task_list_menu.append(Some(&localization::tr("insert.task_list_closed")), Some("app.insert_task_list_closed"));
    
    format_menu.append_submenu(Some(&localization::tr("insert.task_list")), &task_list_menu);
    format_menu.append(Some(&localization::tr("insert.definition_list")), Some("app.insert_definition_list"));
    
    // Special elements
    format_menu.append(Some(&localization::tr("insert.table")), Some("app.insert_table_dialog"));
    format_menu.append(Some(&localization::tr("insert.footnote")), Some("app.insert_footnote"));
    format_menu.append(Some(&localization::tr("insert.emoji")), Some("app.insert_emoji"));
    
    menu_model.append_submenu(Some(&localization::tr("menu.format")), &format_menu);
    
    // View Menu (for language switching)
    let view_menu = gio::Menu::new();
    let language_menu = gio::Menu::new();
    for (code, name) in localization::get_available_locales() {
        language_menu.append(Some(name), Some(&format!("app.set_language_{}", code)));
    }
    view_menu.append_submenu(Some(&localization::tr("menu.language")), &language_menu);
    menu_model.append_submenu(Some(&localization::tr("menu.view")), &view_menu);
    
    // Help Menu
    let help_menu = gio::Menu::new();
    help_menu.append(Some(&localization::tr("help.markdown_guide")), Some("app.markdown_guide"));
    help_menu.append(Some(&localization::tr("help.shortcuts")), Some("app.shortcuts"));
    help_menu.append(Some(&localization::tr("help.about")), Some("app.about"));
    
    menu_model.append_submenu(Some(&localization::tr("menu.help")), &help_menu);
    
    // Create actions
    create_menu_actions(app, editor);
    
    // Set up keyboard accelerators for menu actions
    setup_menu_accelerators(app);
    
    // Create the menu bar
    PopoverMenuBar::from_model(Some(&menu_model))
}

pub fn create_menu_actions(app: &Application, editor: &editor::MarkdownEditor) {
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
        .activate(|app: &Application, _action, _param| {
            println!("Quit clicked");
            app.quit();
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
        .activate(|_app: &Application, _action, _param| {
            localization::set_locale("en");
            println!("Language changed to English");
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
                editor.insert_emoji();
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
        .activate(|_app: &Application, _action, _param| {
            localization::set_locale("es");
            println!("Language changed to Spanish");
        })
        .build();
    
    let set_language_fr_action = gio::ActionEntry::builder("set_language_fr")
        .activate(|_app: &Application, _action, _param| {
            localization::set_locale("fr");
            println!("Language changed to French");
        })
        .build();
    
    let set_language_de_action = gio::ActionEntry::builder("set_language_de")
        .activate(|_app: &Application, _action, _param| {
            localization::set_locale("de");
            println!("Language changed to German");
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
        insert_highlight_action, insert_subscript_action, insert_superscript_action,
        insert_emoji_action, insert_fenced_code_action,
        // Language-specific fenced code actions
        insert_fenced_javascript_action, insert_fenced_python_action, insert_fenced_java_action,
        insert_fenced_typescript_action, insert_fenced_csharp_action, insert_fenced_cpp_action,
        insert_fenced_c_action, insert_fenced_php_action, insert_fenced_go_action, insert_fenced_rust_action,
        insert_fenced_html_action, insert_fenced_css_action, insert_fenced_json_action,
        insert_fenced_xml_action, insert_fenced_sql_action, insert_fenced_bash_action,
        insert_fenced_yaml_action, insert_fenced_markdown_action,
        markdown_guide_action, shortcuts_action, about_action,
        set_language_en_action, set_language_es_action, set_language_fr_action, set_language_de_action,
    ]);
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
}

/// Show dialog to create a custom task list with specified number of items
fn show_task_list_custom_dialog(editor: &editor::MarkdownEditor) {
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
                preview.push_str(&format!("- [ ] Task {}\n", i + 1));
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
