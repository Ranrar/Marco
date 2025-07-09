use crate::{editor, language};
use gtk4::prelude::*;
use gtk4::{gio, Application};

pub fn add_insert_menu(menu_model: &gio::Menu) {
    // Insert Menu (Basic Syntax)
    let insert_menu = gio::Menu::new();

    // Add Headings submenu to Insert
    let headings_menu = gio::Menu::new();
    headings_menu.append(Some(&language::tr("insert.heading1")), Some("app.heading1"));
    headings_menu.append(Some(&language::tr("insert.heading2")), Some("app.heading2"));
    headings_menu.append(Some(&language::tr("insert.heading3")), Some("app.heading3"));
    headings_menu.append(Some(&language::tr("insert.heading4")), Some("app.heading4"));
    headings_menu.append(Some(&language::tr("insert.heading5")), Some("app.heading5"));
    headings_menu.append(Some(&language::tr("insert.heading6")), Some("app.heading6"));
    insert_menu.append_submenu(Some(&language::tr("insert.headings")), &headings_menu);

    insert_menu.append(Some(&language::tr("insert.bold")), Some("app.insert_bold"));
    insert_menu.append(
        Some(&language::tr("insert.italic")),
        Some("app.insert_italic"),
    );
    insert_menu.append(
        Some(&language::tr("insert.blockquote")),
        Some("app.insert_blockquote"),
    );
    insert_menu.append(
        Some(&language::tr("insert.ordered_list")),
        Some("app.insert_numbered_list"),
    );
    insert_menu.append(
        Some(&language::tr("insert.unordered_list")),
        Some("app.insert_bullet_list"),
    );
    insert_menu.append(
        Some(&language::tr("insert.inline_code")),
        Some("app.insert_inline_code"),
    );
    insert_menu.append(
        Some(&language::tr("insert.horizontal_rule")),
        Some("app.insert_hr"),
    );
    insert_menu.append(Some(&language::tr("insert.link")), Some("app.insert_link"));
    insert_menu.append(
        Some(&language::tr("insert.image")),
        Some("app.insert_image"),
    );

    menu_model.append_submenu(Some(&language::tr("menu.insert")), &insert_menu);
}

pub fn create_insert_actions(app: &Application, editor: &editor::MarkdownEditor) {
    // Heading actions
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

    // Basic formatting actions
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

    let insert_blockquote_action = gio::ActionEntry::builder("insert_blockquote")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_blockquote();
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

    let insert_bullet_list_action = gio::ActionEntry::builder("insert_bullet_list")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_bullet_list();
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

    let insert_hr_action = gio::ActionEntry::builder("insert_hr")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_horizontal_rule();
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

    app.add_action_entries([
        heading1_action,
        heading2_action,
        heading3_action,
        heading4_action,
        heading5_action,
        heading6_action,
        insert_bold_action,
        insert_italic_action,
        insert_blockquote_action,
        insert_numbered_list_action,
        insert_bullet_list_action,
        insert_inline_code_action,
        insert_hr_action,
        insert_link_action,
        insert_image_action,
    ]);
}
