use gtk4::prelude::*;
use gtk4::{PopoverMenuBar, Application, gio, Dialog, Grid, Entry, ResponseType, Box, Orientation, Label};
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
    format_menu.append(Some(&localization::tr("insert.strikethrough")), Some("app.strikethrough"));
    format_menu.append(Some(&localization::tr("insert.code_block")), Some("app.code_block"));
    format_menu.append(Some(&localization::tr("insert.table")), Some("app.insert_table_dialog"));
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
        .activate(|_app: &Application, _action, _param| {
            println!("Keyboard Shortcuts clicked");
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
        markdown_guide_action, shortcuts_action, about_action,
        set_language_en_action, set_language_es_action, set_language_fr_action, set_language_de_action,
    ]);
}
