use crate::components::language::Translations;
use gtk4::gio;
use gtk4::prelude::*;

pub fn populate_modules_menu(modules_menu: &gio::Menu, _translations: &Translations) {
    modules_menu.remove_all();

    let modules = gio::Menu::new();
    modules.append(Some("Table"), Some("app.insert_table"));
    modules.append(Some("Format table"), Some("app.format_table"));
    modules.append(Some("Tab block"), Some("app.insert_tab_block"));
    modules.append(Some("Slider deck"), Some("app.insert_slideshow"));
    modules.append(Some("Mermaid"), Some("app.insert_mermaid"));
    modules.append(Some("Admonition"), Some("app.format_insert_admonition"));

    let navigation = gio::Menu::new();
    navigation.append(Some("Insert / Update TOC"), Some("app.insert_update_toc"));

    modules_menu.append_section(None, &modules);
    modules_menu.append_section(None, &navigation);
}

pub fn setup_modules_actions(
    app: &gtk4::Application,
    editor_buffer: &sourceview5::Buffer,
    editor_view: &sourceview5::View,
    window: &gtk4::ApplicationWindow,
) {
    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "insert_table", move || {
            super::insert_block_snippet(
                buf.upcast_ref::<gtk4::TextBuffer>(),
                "| Column 1 | Column 2 |\n|----------|----------|\n| Cell     | Cell     |",
            );
            super::refocus(&buf, &view);
        });
    }

    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "format_table", move || {
            let text_buffer: gtk4::TextBuffer = buf.clone().upcast();
            if crate::components::editor::table_edit::format_table_at_cursor(&text_buffer) {
                super::refocus(&buf, &view);
            }
        });
    }

    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        let win = window.clone();
        super::add_format_action(app, "insert_tab_block", move || {
            crate::ui::dialogs::tabs::show_insert_tabs_dialog(
                win.upcast_ref::<gtk4::Window>(),
                &buf,
                &view,
            );
        });
    }

    {
        let win = window.clone();
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "insert_slideshow", move || {
            crate::ui::dialogs::sliderdeck::show_insert_slider_deck_dialog(
                win.upcast_ref::<gtk4::Window>(),
                &buf,
                &view,
            );
        });
    }

    {
        let win = window.clone();
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "insert_mermaid", move || {
            crate::ui::dialogs::mermaid::show_insert_mermaid_dialog(
                win.upcast_ref::<gtk4::Window>(),
                &buf,
                &view,
            );
        });
    }

    {
        let win = window.clone();
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "format_insert_admonition", move || {
            crate::ui::dialogs::admonition::show_insert_admonition_dialog(
                win.upcast_ref::<gtk4::Window>(),
                &buf,
                &view,
            );
        });
    }

    app.set_accels_for_action("app.insert_table", &["<Control><Shift>t"]);
    app.set_accels_for_action("app.format_table", &["<Control><Alt>t"]);
    app.set_accels_for_action("app.format_insert_admonition", &["<Control><Shift>a"]);

    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "insert_update_toc", move || {
            use core::intelligence::toc::{extract_toc, generate_toc_markdown, replace_toc_in_text,
                TocReplaceResult};

            let text_buffer: gtk4::TextBuffer = buf.clone().upcast();
            let current_text = text_buffer
                .text(&text_buffer.start_iter(), &text_buffer.end_iter(), false)
                .to_string();

            let doc = match core::parse(&current_text) {
                Ok(d) => d,
                Err(e) => {
                    log::warn!("TOC: parse failed: {}", e);
                    return;
                }
            };

            let entries = extract_toc(&doc);
            if entries.is_empty() {
                log::debug!("TOC: no headings found in document");
                return;
            }

            let toc_md = generate_toc_markdown(&entries);

            match replace_toc_in_text(&current_text, &toc_md) {
                TocReplaceResult::Updated(new_text) => {
                    // Use delete + insert instead of set_text: GtkSourceBuffer's set_text
                    // internally calls begin_irreversible_action which conflicts with the
                    // open user action and emits GTK-WARNING messages.
                    text_buffer.begin_user_action();
                    let mut start = text_buffer.start_iter();
                    let mut end = text_buffer.end_iter();
                    text_buffer.delete(&mut start, &mut end);
                    let mut pos = text_buffer.start_iter();
                    text_buffer.insert(&mut pos, &new_text);
                    text_buffer.end_user_action();
                    super::refocus(&buf, &view);
                }
                TocReplaceResult::NoMarkers => {
                    super::insert_block_snippet(&text_buffer, &toc_md);
                    super::refocus(&buf, &view);
                }
                TocReplaceResult::NoChange => {
                    log::debug!("TOC: already up to date");
                }
            }
        });
    }
}
