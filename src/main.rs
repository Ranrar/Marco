mod editor;
mod markdown_basic;

use gtk4::prelude::*;
use gtk4::{
    glib, Application, ApplicationWindow, Box, Orientation, Label, Separator,
    PopoverMenuBar, gio, Button,
};
const APP_ID: &str = "com.example.Marco";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run()
}

fn build_ui(app: &Application) {
    // Create the main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Marco - Markdown Composer")
        .default_width(800)
        .default_height(600)
        .build();

    // Create the editor
    let editor = editor::MarkdownEditor::new();

    // Set up header bar (without file buttons)
    let header_bar = editor.create_simple_header_bar();
    window.set_titlebar(Some(&header_bar));

    // Create main vertical box
    let main_box = Box::new(Orientation::Vertical, 0);
    
    // Create and add menu bar
    let menu_bar = create_menu_bar(app, &editor);
    main_box.append(&menu_bar);
    
    // Create and add toolbar with markdown formatting (no file buttons)
    let toolbar = create_markdown_toolbar(&editor);
    main_box.append(&toolbar);
    
    // Add editor to main box (takes most of the space)
    main_box.append(editor.widget());
    editor.widget().set_vexpand(true);
    
    // Add separator
    let separator = Separator::new(Orientation::Horizontal);
    main_box.append(&separator);
    
    // Create footer
    let (footer, footer_labels) = create_footer();
    main_box.append(&footer);
    
    // Connect editor to footer updates
    editor.add_footer_callback({
        let footer_labels = footer_labels.clone();
        move |_text, word_count, char_count, line, column| {
            footer_labels.word_count.set_text(&format!("Words: {}", word_count));
            footer_labels.char_count.set_text(&format!("Characters: {}", char_count));
            footer_labels.cursor_pos.set_text(&format!("Line: {}, Col: {}", line, column));
            footer_labels.status.set_text("Ready");
        }
    });

    // Add main box to window
    window.set_child(Some(&main_box));

    // Set up 50/50 split after window is properly sized
    window.connect_realize({
        let editor = editor.clone();
        move |window| {
            // Get the default window width and set 50/50 split
            let width = window.default_width();
            editor.set_split_ratio(width);
        }
    });

    // Present the window
    window.present();
}

fn create_menu_bar(app: &Application, editor: &editor::MarkdownEditor) -> PopoverMenuBar {
    // Create the menu model
    let menu_model = gio::Menu::new();
    
    // File Menu
    let file_menu = gio::Menu::new();
    file_menu.append(Some("New"), Some("app.new"));
    file_menu.append(Some("Open..."), Some("app.open"));
    file_menu.append(Some("Save"), Some("app.save"));
    file_menu.append(Some("Save As..."), Some("app.save_as"));
    file_menu.append(Some("Quit"), Some("app.quit"));
    
    menu_model.append_submenu(Some("File"), &file_menu);
    
    // Edit Menu
    let edit_menu = gio::Menu::new();
    edit_menu.append(Some("Undo"), Some("app.undo"));
    edit_menu.append(Some("Redo"), Some("app.redo"));
    edit_menu.append(Some("Cut"), Some("app.cut"));
    edit_menu.append(Some("Copy"), Some("app.copy"));
    edit_menu.append(Some("Paste"), Some("app.paste"));
    edit_menu.append(Some("Find..."), Some("app.find"));
    edit_menu.append(Some("Find & Replace..."), Some("app.replace"));
    
    menu_model.append_submenu(Some("Edit"), &edit_menu);
    
    // Insert Menu (Basic Syntax)
    let insert_menu = gio::Menu::new();
    insert_menu.append(Some("Heading 1"), Some("app.heading1"));
    insert_menu.append(Some("Bold"), Some("app.insert_bold"));
    insert_menu.append(Some("Italic"), Some("app.insert_italic"));
    insert_menu.append(Some("Blockquote"), Some("app.insert_blockquote"));
    insert_menu.append(Some("Ordered List"), Some("app.insert_numbered_list"));
    insert_menu.append(Some("Unordered List"), Some("app.insert_bullet_list"));
    insert_menu.append(Some("Inline Code"), Some("app.insert_inline_code"));
    insert_menu.append(Some("Horizontal Rule"), Some("app.insert_hr"));
    insert_menu.append(Some("Link"), Some("app.insert_link"));
    insert_menu.append(Some("Image"), Some("app.insert_image"));
    menu_model.append_submenu(Some("Insert"), &insert_menu);

    // Format Menu (Extended Syntax)
    let format_menu = gio::Menu::new();
    // Add Headings submenu
    let headings_menu = gio::Menu::new();
    headings_menu.append(Some("Heading 1"), Some("app.heading1"));
    headings_menu.append(Some("Heading 2"), Some("app.heading2"));
    headings_menu.append(Some("Heading 3"), Some("app.heading3"));
    headings_menu.append(Some("Heading 4"), Some("app.heading4"));
    headings_menu.append(Some("Heading 5"), Some("app.heading5"));
    headings_menu.append(Some("Heading 6"), Some("app.heading6"));
    format_menu.append_submenu(Some("Headings"), &headings_menu);
    format_menu.append(Some("Strikethrough"), Some("app.strikethrough"));
    format_menu.append(Some("Code Block"), Some("app.code_block"));
    format_menu.append(Some("Table..."), Some("app.insert_table_dialog"));
    menu_model.append_submenu(Some("Format"), &format_menu);
    
    // Help Menu
    let help_menu = gio::Menu::new();
    help_menu.append(Some("Markdown Guide"), Some("app.markdown_guide"));
    help_menu.append(Some("Keyboard Shortcuts"), Some("app.shortcuts"));
    help_menu.append(Some("About"), Some("app.about"));
    
    menu_model.append_submenu(Some("Help"), &help_menu);
    
    // Create actions
    create_menu_actions(app, editor);
    
    // Create the menu bar
    PopoverMenuBar::from_model(Some(&menu_model))
}

fn create_markdown_toolbar(editor: &editor::MarkdownEditor) -> Box {
    // BASIC SYNTAX ONLY - Markdown formatting toolbar
    let markdown_toolbar = Box::new(Orientation::Horizontal, 5);
    markdown_toolbar.set_margin_top(5);
    markdown_toolbar.set_margin_bottom(5);
    markdown_toolbar.set_margin_start(10);
    markdown_toolbar.set_margin_end(10);
    
    // Heading buttons (Basic)
    let h1_button = Button::with_label("H1");
    h1_button.set_tooltip_text(Some("Heading 1 (# text)"));
    h1_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_heading(1);
        }
    });
    markdown_toolbar.append(&h1_button);
    
    let h2_button = Button::with_label("H2");
    h2_button.set_tooltip_text(Some("Heading 2 (## text)"));
    h2_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_heading(2);
        }
    });
    markdown_toolbar.append(&h2_button);
    
    let h3_button = Button::with_label("H3");
    h3_button.set_tooltip_text(Some("Heading 3 (### text)"));
    h3_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_heading(3);
        }
    });
    markdown_toolbar.append(&h3_button);
    
    // Separator
    let sep1 = Separator::new(Orientation::Vertical);
    markdown_toolbar.append(&sep1);
    
    // Text formatting buttons (Basic)
    let bold_button = Button::with_label("𝐁");
    bold_button.set_tooltip_text(Some("Bold (**text**)"));
    bold_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_bold();
        }
    });
    markdown_toolbar.append(&bold_button);
    
    let italic_button = Button::with_label("𝐼");
    italic_button.set_tooltip_text(Some("Italic (*text*)"));
    italic_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_italic();
        }
    });
    markdown_toolbar.append(&italic_button);
    
    let code_button = Button::with_label("`");
    code_button.set_tooltip_text(Some("Inline Code (`code`)"));
    code_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_inline_code();
        }
    });
    markdown_toolbar.append(&code_button);
    
    // Separator
    let sep2 = Separator::new(Orientation::Vertical);
    markdown_toolbar.append(&sep2);
    
    // List buttons (Basic)
    let bullet_list_button = Button::with_label("•");
    bullet_list_button.set_tooltip_text(Some("Unordered List (- item)"));
    bullet_list_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_bullet_list();
        }
    });
    markdown_toolbar.append(&bullet_list_button);
    
    let numbered_list_button = Button::with_label("1.");
    numbered_list_button.set_tooltip_text(Some("Ordered List (1. item)"));
    numbered_list_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_numbered_list();
        }
    });
    markdown_toolbar.append(&numbered_list_button);
    
    let quote_button = Button::with_label("❝");
    quote_button.set_tooltip_text(Some("Blockquote (> text)"));
    quote_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_blockquote();
        }
    });
    markdown_toolbar.append(&quote_button);
    
    // Separator
    let sep3 = Separator::new(Orientation::Vertical);
    markdown_toolbar.append(&sep3);
    
    // Insert buttons (Basic)
    let link_button = Button::with_label("🔗");
    link_button.set_tooltip_text(Some("Link ([text](url))"));
    link_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_link();
        }
    });
    markdown_toolbar.append(&link_button);
    
    let image_button = Button::with_label("🖼");
    image_button.set_tooltip_text(Some("Image (![alt](url))"));
    image_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_image();
        }
    });
    markdown_toolbar.append(&image_button);
    
    let hr_button = Button::with_label("—");
    hr_button.set_tooltip_text(Some("Horizontal Rule (---)"));
    hr_button.connect_clicked({
        let editor = editor.clone();
        move |_| {
            editor.insert_horizontal_rule();
        }
    });
    markdown_toolbar.append(&hr_button);
    
    markdown_toolbar
}

fn create_menu_actions(app: &Application, editor: &editor::MarkdownEditor) {
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
                use gtk4::{Dialog, Grid, Button as GtkButton, ResponseType, EventControllerMotion};
                use std::cell::RefCell;
                use std::rc::Rc;
                
                if let Some(window) = app.active_window() {
                    let dialog = Dialog::with_buttons(
                        Some("Insert Table"),
                        Some(&window),
                        gtk4::DialogFlags::MODAL,
                        &[("Insert", ResponseType::Accept), ("Cancel", ResponseType::Cancel)],
                    );
                    let content_area = dialog.content_area();
                    
                    // Create grid container
                    let grid = Grid::new();
                    grid.set_row_spacing(2);
                    grid.set_column_spacing(2);
                    grid.set_margin_top(12);
                    grid.set_margin_bottom(12);
                    grid.set_margin_start(12);
                    grid.set_margin_end(12);

                    let max_rows = 10;
                    let max_cols = 10;
                    let selected_position = Rc::new(RefCell::new((0, 0)));
                    let buttons = Rc::new(RefCell::new(Vec::<Vec<GtkButton>>::new()));

                    // Create grid of buttons
                    for r in 0..max_rows {
                        let mut row_buttons = vec![];
                        for c in 0..max_cols {
                            let btn = GtkButton::with_label("");
                            btn.set_size_request(24, 24);
                            
                            // Add motion controller for hover effect
                            let motion_controller = EventControllerMotion::new();
                            let selected_position_clone = selected_position.clone();
                            let buttons_clone = buttons.clone();
                            motion_controller.connect_enter(move |_, _, _| {
                                *selected_position_clone.borrow_mut() = (r + 1, c + 1);
                                // Update button styles
                                if let Ok(buttons_vec) = buttons_clone.try_borrow() {
                                    for (ri, row) in buttons_vec.iter().enumerate() {
                                        for (ci, button) in row.iter().enumerate() {
                                            if ri < r + 1 && ci < c + 1 {
                                                button.add_css_class("suggested-action");
                                            } else {
                                                button.remove_css_class("suggested-action");
                                            }
                                        }
                                    }
                                }
                            });
                            btn.add_controller(motion_controller);
                            
                            row_buttons.push(btn.clone());
                            grid.attach(&btn, c as i32, r as i32, 1, 1);
                        }
                        buttons.borrow_mut().push(row_buttons);
                    }

                    content_area.append(&grid);
                    dialog.set_default_response(ResponseType::Accept);
                    dialog.show();

                    let editor_clone = editor.clone();
                    dialog.connect_response(move |dialog, resp| {
                        if resp == ResponseType::Accept {
                            let (rows, cols) = *selected_position.borrow();
                            if rows > 0 && cols > 0 {
                                // Build markdown table string
                                let mut table = String::new();
                                table.push('\n');
                                table.push('|');
                                for c in 0..cols {
                                    table.push_str(&format!(" Header {} |", c + 1));
                                }
                                table.push('\n');
                                table.push('|');
                                for _ in 0..cols {
                                    table.push_str("----------|");
                                }
                                table.push('\n');
                                for r in 0..rows {
                                    table.push('|');
                                    for c in 0..cols {
                                        table.push_str(&format!(" Cell {}-{} |", r + 1, c + 1));
                                    }
                                    table.push('\n');
                                }
                                // For now use the default table - we'll need to make insert_text_at_cursor public later
                                editor_clone.insert_table();
                            }
                        }
                        dialog.close();
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
    ]);
}

#[derive(Clone)]
struct FooterLabels {
    status: Label,
    word_count: Label,
    char_count: Label,
    cursor_pos: Label,
}

fn create_footer() -> (Box, FooterLabels) {
    let footer_box = Box::new(Orientation::Horizontal, 10);
    footer_box.set_margin_top(5);
    footer_box.set_margin_bottom(5);
    footer_box.set_margin_start(10);
    footer_box.set_margin_end(10);
    
    // Status label (left side)
    let status_label = Label::new(Some("Ready"));
    status_label.set_halign(gtk4::Align::Start);
    footer_box.append(&status_label);
    
    // Spacer to push items to the sides
    let spacer = Label::new(None);
    spacer.set_hexpand(true);
    footer_box.append(&spacer);
    
    // Info labels (right side)
    let word_count_label = Label::new(Some("Words: 0"));
    footer_box.append(&word_count_label);
    
    let char_count_label = Label::new(Some("Characters: 0"));
    footer_box.append(&char_count_label);
    
    let cursor_pos_label = Label::new(Some("Line: 1, Col: 1"));
    footer_box.append(&cursor_pos_label);
    
    let labels = FooterLabels {
        status: status_label,
        word_count: word_count_label,
        char_count: char_count_label,
        cursor_pos: cursor_pos_label,
    };
    
    (footer_box, labels)
}
