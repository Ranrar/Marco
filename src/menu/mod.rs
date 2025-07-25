use crate::{editor, language};
use gtk4::prelude::*;
use gtk4::{gio, AboutDialog, Application, PopoverMenuBar};

// Module declarations
pub mod advanced;
pub mod basic;
pub mod dialogs;
pub mod format;
pub mod insert;
pub mod view;

// Re-export main dialog functions for backward compatibility
pub use dialogs::{
    show_admonition_dialog, show_center_text_dialog, show_colored_text_dialog, show_comment_dialog,
    show_definition_list_custom_dialog, show_html_entity_dialog, show_image_with_size_dialog,
    show_link_open_new_dialog, show_shortcuts_dialog, show_task_list_custom_dialog,
    show_underline_dialog, show_youtube_video_dialog,
};
pub use format::create_table_dialog;

pub fn create_menu_bar(
    app: &Application,
    editor: &editor::MarkdownEditor,
    theme_manager: &crate::theme::ThemeManager,
) -> PopoverMenuBar {
    // Create the menu model
    let menu_model = gio::Menu::new();

    // Add menus from different modules
    basic::add_file_menu(&menu_model);
    basic::add_edit_menu(&menu_model);
    insert::add_insert_menu(&menu_model);
    format::add_format_menu(&menu_model);
    advanced::add_advanced_menu(&menu_model);
    view::add_view_menu(&menu_model, editor, theme_manager);

    // Help Menu
    let help_menu = gio::Menu::new();
    help_menu.append(
        Some(&language::tr("help.markdown_guide")),
        Some("app.markdown_guide"),
    );
    help_menu.append(Some(&language::tr("help.shortcuts")), Some("app.shortcuts"));
    help_menu.append(Some(&language::tr("help.about")), Some("app.about"));

    menu_model.append_submenu(Some(&language::tr("menu.help")), &help_menu);

    // Create actions from all modules
    basic::create_file_actions(app, editor);
    basic::create_edit_actions(app, editor);
    insert::create_insert_actions(app, editor);
    format::create_format_actions(app, editor);
    advanced::create_advanced_actions(app, editor);
    view::create_view_actions(app, editor, theme_manager);

    // Create help actions
    create_help_actions(app, editor);

    // Set up keyboard accelerators for menu actions
    setup_menu_accelerators(app);

    // Create the menu bar
    PopoverMenuBar::from_model(Some(&menu_model))
}

pub fn rebuild_menu_bar(
    app: &Application,
    editor: &editor::MarkdownEditor,
    theme_manager: &crate::theme::ThemeManager,
) {
    // Get the main window
    if let Some(window) = app.active_window() {
        // Create a new menu bar
        let new_menu_bar = create_menu_bar(app, editor, theme_manager);

        // Find the main box (first child of window) and update the menu bar
        if let Some(main_box) = window
            .child()
            .and_then(|child| child.downcast::<gtk4::Box>().ok())
        {
            // Find and remove existing menu bar (should be first child of main_box)
            let mut child = main_box.first_child();
            while let Some(widget) = child {
                let next_child = widget.next_sibling();
                if widget.is::<PopoverMenuBar>() {
                    main_box.remove(&widget);
                    break;
                }
                child = next_child;
            }

            // Insert the new menu bar at the beginning (position 0)
            main_box.insert_child_after(&new_menu_bar, None::<&gtk4::Widget>);
        }
    }
}

fn create_help_actions(app: &Application, editor: &editor::MarkdownEditor) {
    let markdown_guide_action = gio::ActionEntry::builder("markdown_guide")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                open_markdown_guide(&editor);
            }
        })
        .build();

    let shortcuts_action = gio::ActionEntry::builder("shortcuts")
        .activate(|app: &Application, _action, _param| {
            if let Some(window) = app.active_window() {
                show_shortcuts_dialog(&window);
            }
        })
        .build();

    let about_action = gio::ActionEntry::builder("about")
        .activate(|app: &Application, _action, _param| {
            show_about_dialog(app);
        })
        .build();

    app.add_action_entries([markdown_guide_action, shortcuts_action, about_action]);
}

fn setup_menu_accelerators(app: &Application) {
    // File menu accelerators
    app.set_accels_for_action("app.new", &["<Ctrl>n"]);
    app.set_accels_for_action("app.open", &["<Ctrl>o"]);
    app.set_accels_for_action("app.save", &["<Ctrl>s"]);
    app.set_accels_for_action("app.save_as", &["<Ctrl><Shift>s"]);
    app.set_accels_for_action("app.quit", &["<Ctrl>q"]);

    // Edit menu accelerators
    app.set_accels_for_action("app.undo", &["<Ctrl>z"]);
    app.set_accels_for_action("app.redo", &["<Ctrl>y"]);
    app.set_accels_for_action("app.cut", &["<Ctrl>x"]);
    app.set_accels_for_action("app.copy", &["<Ctrl>c"]);
    app.set_accels_for_action("app.paste", &["<Ctrl>v"]);
    app.set_accels_for_action("app.find", &["<Ctrl>f"]);
    app.set_accels_for_action("app.replace", &["<Ctrl>h"]);

    // Insert menu accelerators
    app.set_accels_for_action("app.heading1", &["<Ctrl>1"]);
    app.set_accels_for_action("app.heading2", &["<Ctrl>2"]);
    app.set_accels_for_action("app.heading3", &["<Ctrl>3"]);
    app.set_accels_for_action("app.heading4", &["<Ctrl>4"]);
    app.set_accels_for_action("app.heading5", &["<Ctrl>5"]);
    app.set_accels_for_action("app.heading6", &["<Ctrl>6"]);
    app.set_accels_for_action("app.insert_bold", &["<Ctrl>b"]);
    app.set_accels_for_action("app.insert_italic", &["<Ctrl>i"]);
    app.set_accels_for_action("app.insert_link", &["<Ctrl>k"]);

    // Format menu accelerators
    app.set_accels_for_action("app.code_block", &["<Ctrl><Shift>c"]);
    app.set_accels_for_action("app.insert_inline_code", &["<Ctrl>grave"]);

    // Help menu accelerators
    app.set_accels_for_action("app.shortcuts", &["<Ctrl>question"]);

    // View mode global accelerators
    app.set_accels_for_action("app.view_html", &["F5"]);
    app.set_accels_for_action("app.view_code", &["F6"]);
}

fn show_about_dialog(app: &Application) {
    let window = app.active_window().unwrap();

    let license = "MIT License

Copyright (c) 2025 Kim Skov Rasmussen

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the \"Software\"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE."; // load full license
    let authors = vec!["Kim Skov Rasmussen"];

    let dialog = AboutDialog::builder()
        .program_name("Marco")
        .version("0.1.0")
        .comments("A modern Markdown Composer with advanced features")
        .website("https://github.com/Ranrar/Marco")
        .website_label("GitHub Repository")
        .license(license)
        .wrap_license(true)
        .authors(authors)
        .logo_icon_name("text-editor")
        .modal(true)
        .transient_for(&window)
        .build();

    dialog.present();
}

fn open_markdown_guide(editor: &editor::MarkdownEditor) {
    use std::env;
    use std::fs;
    use std::io::Write;


    // Embed the user guide and logo
    const USER_GUIDE_MD: &str = include_str!("../assets/doc/user_guide.md");
    const LOGO_PNG: &[u8] = include_bytes!("../assets/doc/logo.png");

    // Write logo.png to a temp file if it doesn't exist
    let mut temp_logo_path = env::temp_dir();
    temp_logo_path.push("logo.png");
    if !temp_logo_path.exists() {
        if let Ok(mut file) = fs::File::create(&temp_logo_path) {
            let _ = file.write_all(LOGO_PNG);
        }
    }

    // Rewrite the Markdown to point to the temp logo file
    let logo_path_str = temp_logo_path.to_string_lossy();
    let content = USER_GUIDE_MD.replace(
        "src=\"logo.png\"",
        &format!("src=\"{}\"", logo_path_str)
    );
    editor.get_source_buffer().set_text(&content);
    // Set the HTML preview base path to the temp dir (for image resolution)
    editor.html_view.set_base_path(Some(temp_logo_path.parent().unwrap().to_path_buf()));
    println!("Loaded embedded user guide and logo from binary, using temp file: {}", logo_path_str);
}
