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

    modules_menu.append_section(None, &modules);
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
}
