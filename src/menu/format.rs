use crate::editor::emoji;
use crate::{editor, language};
use gtk4::prelude::*;
use gtk4::{gio, Application, Box, Dialog, Entry, Grid, Label, Orientation, ResponseType};
use std::rc::Rc;

pub fn add_format_menu(menu_model: &gio::Menu) {
    // Format Menu (Extended Syntax)
    let format_menu = gio::Menu::new();

    // Text formatting
    format_menu.append(
        Some(&language::tr("insert.strikethrough")),
        Some("app.strikethrough"),
    );
    format_menu.append(
        Some(&language::tr("insert.highlight")),
        Some("app.insert_highlight"),
    );
    format_menu.append(
        Some(&language::tr("insert.subscript")),
        Some("app.insert_subscript"),
    );
    format_menu.append(
        Some(&language::tr("insert.superscript")),
        Some("app.insert_superscript"),
    );

    // Code
    format_menu.append(
        Some(&language::tr("insert.code_block")),
        Some("app.code_block"),
    );
    format_menu.append(Some("Fenced code block..."), Some("app.insert_fenced_code"));

    // Lists and tasks
    let task_list_menu = gio::Menu::new();
    task_list_menu.append(Some("📝 Default Task List"), Some("app.insert_task_list"));
    task_list_menu.append(
        Some("⚙️ Custom Task List..."),
        Some("app.insert_task_list_custom"),
    );
    task_list_menu.append(Some("☐ Open Task"), Some("app.insert_task_list_open"));
    task_list_menu.append(Some("☑️ Closed Task"), Some("app.insert_task_list_closed"));

    format_menu.append_submenu(Some(&language::tr("insert.task_list")), &task_list_menu);

    // Definition list submenu
    let definition_list_menu = gio::Menu::new();
    definition_list_menu.append(
        Some("📚 Default Definition List"),
        Some("app.insert_definition_list"),
    );
    definition_list_menu.append(
        Some("⚙️ Custom Definition List..."),
        Some("app.insert_definition_list_custom"),
    );
    definition_list_menu.append(
        Some("📖 Single Definition"),
        Some("app.insert_definition_list_single"),
    );

    format_menu.append_submenu(
        Some(&language::tr("insert.definition_list")),
        &definition_list_menu,
    );

    // Special elements
    format_menu.append(Some("⚙️ Table..."), Some("app.insert_table_dialog"));
    format_menu.append(Some("📝 Footnote"), Some("app.insert_footnote"));
    format_menu.append(Some("😀 Emoji"), Some("app.insert_emoji"));
    format_menu.append(
        Some("&amp; HTML Entities..."),
        Some("app.insert_html_entity_dialog"),
    );

    menu_model.append_submenu(Some(&language::tr("menu.format")), &format_menu);
}

pub fn create_format_actions(app: &Application, editor: &editor::MarkdownEditor) {
    // Text formatting actions
    let strikethrough_action = gio::ActionEntry::builder("strikethrough")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_strikethrough();
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

    // Code block actions
    let code_block_action = gio::ActionEntry::builder("code_block")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_code_block();
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

    // Task list actions
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
            move |app: &Application, _action, _param| {
                if let Some(window) = app.active_window() {
                    super::show_task_list_custom_dialog(&window, &editor);
                }
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

    // Definition list actions
    let insert_definition_list_action = gio::ActionEntry::builder("insert_definition_list")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_definition_list();
            }
        })
        .build();

    let insert_definition_list_custom_action =
        gio::ActionEntry::builder("insert_definition_list_custom")
            .activate({
                let editor = editor.clone();
                move |app: &Application, _action, _param| {
                    if let Some(window) = app.active_window() {
                        super::show_definition_list_custom_dialog(&window, &editor);
                    }
                }
            })
            .build();

    let insert_definition_list_single_action =
        gio::ActionEntry::builder("insert_definition_list_single")
            .activate({
                let editor = editor.clone();
                move |_app: &Application, _action, _param| {
                    editor.insert_single_definition();
                }
            })
            .build();

    // Table action
    let insert_table_dialog_action = gio::ActionEntry::builder("insert_table_dialog")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                if let Some(window) = app.active_window() {
                    create_table_dialog(&window, &editor);
                }
            }
        })
        .build();

    // Other special element actions
    let insert_footnote_action = gio::ActionEntry::builder("insert_footnote")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_footnote();
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

    let insert_html_entity_dialog_action = gio::ActionEntry::builder("insert_html_entity_dialog")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                if let Some(window) = app.active_window() {
                    super::show_html_entity_dialog(&window, &editor);
                }
            }
        })
        .build();

    // Add all actions to the application
    let all_actions = vec![
        strikethrough_action,
        insert_highlight_action,
        insert_subscript_action,
        insert_superscript_action,
        code_block_action,
        insert_fenced_code_action,
        insert_task_list_action,
        insert_task_list_custom_action,
        insert_task_list_open_action,
        insert_task_list_closed_action,
        insert_definition_list_action,
        insert_definition_list_custom_action,
        insert_definition_list_single_action,
        insert_table_dialog_action,
        insert_footnote_action,
        insert_emoji_action,
        insert_html_entity_dialog_action,
    ];

    app.add_action_entries(all_actions);
}

pub fn create_table_dialog(window: &gtk4::Window, editor: &editor::MarkdownEditor) {
    let dialog = Dialog::with_buttons(
        Some(&language::tr("table_dialog.title")),
        Some(window),
        gtk4::DialogFlags::MODAL,
        &[
            (&language::tr("table_dialog.insert"), ResponseType::Accept),
            (&language::tr("table_dialog.cancel"), ResponseType::Cancel),
        ],
    );
    let content_area = dialog.content_area();

    // Create main container
    let main_container = Box::new(Orientation::Vertical, 12);
    main_container.set_margin_top(12);
    main_container.set_margin_bottom(12);
    main_container.set_margin_start(12);
    main_container.set_margin_end(12);

    // Add title label
    let title_label = Label::new(Some(&language::tr("table_dialog.description")));
    title_label.set_halign(gtk4::Align::Start);
    main_container.append(&title_label);

    // Create grid for input fields
    let input_grid = Grid::new();
    input_grid.set_row_spacing(8);
    input_grid.set_column_spacing(12);
    input_grid.set_margin_top(12);

    // Rows input
    let rows_label = Label::new(Some(&language::tr("table_dialog.rows")));
    rows_label.set_halign(gtk4::Align::End);
    input_grid.attach(&rows_label, 0, 0, 1, 1);

    let rows_entry = Entry::new();
    rows_entry.set_text("3"); // Default value
    rows_entry.set_width_chars(5);
    rows_entry.set_max_length(3); // Limit to 3 characters

    input_grid.attach(&rows_entry, 1, 0, 1, 1);

    // Columns input
    let cols_label = Label::new(Some(&language::tr("table_dialog.columns")));
    cols_label.set_halign(gtk4::Align::End);
    input_grid.attach(&cols_label, 0, 1, 1, 1);

    let cols_entry = Entry::new();
    cols_entry.set_text("3"); // Default value
    cols_entry.set_width_chars(5);
    cols_entry.set_max_length(3); // Limit to 3 characters

    // Add input filters for numbers only
    let cols_entry_for_rows = cols_entry.clone();
    rows_entry.connect_insert_text(move |entry, text, position| {
        let filtered_text: String = text.chars().filter(|c| c.is_ascii_digit()).collect();

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

    let rows_entry_for_cols = rows_entry.clone();
    cols_entry.connect_insert_text(move |entry, text, position| {
        let filtered_text: String = text.chars().filter(|c| c.is_ascii_digit()).collect();

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
                if let (Ok(rows), Ok(cols)) =
                    (rows_text.parse::<usize>(), cols_text.parse::<usize>())
                {
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
