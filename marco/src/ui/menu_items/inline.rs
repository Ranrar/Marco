use crate::components::language::Translations;
use gtk4::gio;
use gtk4::prelude::*;
use std::rc::Rc;
use std::sync::Arc;

pub fn populate_inline_menu(inline_menu: &gio::Menu, translations: &Translations) {
    inline_menu.remove_all();

    let formatting = gio::Menu::new();
    formatting.append(
        Some(&translations.menu.bold),
        Some("app.format_inline_bold"),
    );
    formatting.append(
        Some(&translations.menu.italic),
        Some("app.format_inline_italic"),
    );
    formatting.append(
        Some(&translations.menu.strikethrough),
        Some("app.format_inline_strikethrough"),
    );
    formatting.append(
        Some(&translations.menu.highlight),
        Some("app.format_inline_highlight"),
    );
    formatting.append(
        Some(&translations.menu.code),
        Some("app.format_inline_code"),
    );
    formatting.append(
        Some(&translations.menu.superscript),
        Some("app.format_inline_superscript"),
    );
    formatting.append(
        Some(&translations.menu.subscript),
        Some("app.format_inline_subscript"),
    );
    formatting.append(
        Some(&translations.menu.math_inline),
        Some("app.format_inline_math"),
    );
    inline_menu.append_section(None, &formatting);

    let inserts = gio::Menu::new();
    inserts.append(
        Some(&translations.menu.insert_link),
        Some("app.format_inline_link"),
    );
    inserts.append(
        Some(&translations.menu.link_reference),
        Some("app.format_inline_link_reference"),
    );
    inserts.append(
        Some(&translations.menu.insert_image),
        Some("app.format_inline_image"),
    );
    inserts.append(
        Some(&translations.menu.inline_footnote),
        Some("app.format_inline_footnote"),
    );
    inserts.append(
        Some(&translations.menu.emoji),
        Some("app.format_inline_emoji"),
    );
    inserts.append(
        Some(&translations.menu.checkbox),
        Some("app.format_inline_checkbox"),
    );
    inserts.append(
        Some(&translations.menu.mention),
        Some("app.format_insert_mention"),
    );
    inline_menu.append_section(None, &inserts);
}

pub fn setup_inline_actions(
    app: &gtk4::Application,
    editor_buffer: &sourceview5::Buffer,
    editor_view: &sourceview5::View,
    window: &gtk4::ApplicationWindow,
    settings_manager: Arc<core::logic::swanson::SettingsManager>,
    current_file_provider: Rc<dyn Fn() -> Option<std::path::PathBuf>>,
) {
    // ── Toggle: Bold ──────────────────────────────────────────────────────────
    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "format_inline_bold", move || {
            crate::ui::toolbar::toggle_bold_for_selection_or_word(
                buf.upcast_ref::<gtk4::TextBuffer>(),
            );
            super::refocus(&buf, &view);
        });
    }
    app.set_accels_for_action("app.format_inline_bold", &["<Control>b"]);

    // ── Toggle: Italic ────────────────────────────────────────────────────────
    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "format_inline_italic", move || {
            crate::ui::toolbar::toggle_italic_for_selection_or_word(
                buf.upcast_ref::<gtk4::TextBuffer>(),
            );
            super::refocus(&buf, &view);
        });
    }
    app.set_accels_for_action("app.format_inline_italic", &["<Control>i"]);

    // ── Toggle: Strikethrough ─────────────────────────────────────────────────
    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "format_inline_strikethrough", move || {
            crate::ui::toolbar::toggle_strikethrough_for_selection_or_word(
                buf.upcast_ref::<gtk4::TextBuffer>(),
            );
            super::refocus(&buf, &view);
        });
    }
    app.set_accels_for_action("app.format_inline_strikethrough", &["<Control><Shift>s"]);

    // ── Toggle: Highlight (==…==) ─────────────────────────────────────────────
    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "format_inline_highlight", move || {
            crate::ui::toolbar::toggle_highlight_for_selection_or_word(
                buf.upcast_ref::<gtk4::TextBuffer>(),
            );
            super::refocus(&buf, &view);
        });
    }
    app.set_accels_for_action("app.format_inline_highlight", &["<Control><Shift>h"]);

    // ── Toggle: Inline code (`…`) ─────────────────────────────────────────────
    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "format_inline_code", move || {
            crate::ui::toolbar::toggle_code_for_selection_or_word(
                buf.upcast_ref::<gtk4::TextBuffer>(),
            );
            super::refocus(&buf, &view);
        });
    }
    app.set_accels_for_action("app.format_inline_code", &["<Control>e"]);

    // ── Toggle: Superscript (^…^) ─────────────────────────────────────────────
    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "format_inline_superscript", move || {
            crate::ui::toolbar::toggle_superscript_for_selection_or_word(
                buf.upcast_ref::<gtk4::TextBuffer>(),
            );
            super::refocus(&buf, &view);
        });
    }

    // ── Toggle: Subscript (~…~) ───────────────────────────────────────────────
    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "format_inline_subscript", move || {
            crate::ui::toolbar::toggle_subscript_for_selection_or_word(
                buf.upcast_ref::<gtk4::TextBuffer>(),
            );
            super::refocus(&buf, &view);
        });
    }

    // ── Toggle: Inline math ($…$) ─────────────────────────────────────────────
    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "format_inline_math", move || {
            crate::ui::toolbar::toggle_math_for_selection_or_word(
                buf.upcast_ref::<gtk4::TextBuffer>(),
            );
            super::refocus(&buf, &view);
        });
    }

    // ── Toggle: Inline checkbox ([ ] / [x]) ──────────────────────────────────
    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "format_inline_checkbox", move || {
            crate::ui::toolbar::toggle_inline_checkbox(buf.upcast_ref::<gtk4::TextBuffer>());
            super::refocus(&buf, &view);
        });
    }

    // ── Popover: Link ─────────────────────────────────────────────────────────
    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        let win = window.clone();
        let provider = current_file_provider.clone();
        super::add_format_action(app, "format_inline_link", move || {
            crate::ui::toolbar::show_insert_link_popover(
                buf.upcast_ref::<gtk4::TextBuffer>(),
                view.upcast_ref::<gtk4::TextView>(),
                win.upcast_ref::<gtk4::Window>(),
                provider.clone(),
            );
        });
    }
    app.set_accels_for_action("app.format_inline_link", &["<Control>k"]);

    // ── Popover: Reference link ───────────────────────────────────────────────
    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        let win = window.clone();
        let provider = current_file_provider.clone();
        super::add_format_action(app, "format_inline_link_reference", move || {
            crate::ui::toolbar::show_insert_reference_link_popover(
                buf.upcast_ref::<gtk4::TextBuffer>(),
                view.upcast_ref::<gtk4::TextView>(),
                win.upcast_ref::<gtk4::Window>(),
                provider.clone(),
            );
        });
    }

    // ── Popover: Image ────────────────────────────────────────────────────────
    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        let win = window.clone();
        let provider = current_file_provider.clone();
        super::add_format_action(app, "format_inline_image", move || {
            crate::ui::toolbar::show_insert_image_popover(
                buf.upcast_ref::<gtk4::TextBuffer>(),
                view.upcast_ref::<gtk4::TextView>(),
                win.upcast_ref::<gtk4::Window>(),
                provider.clone(),
            );
        });
    }

    // ── Popover: Footnote ─────────────────────────────────────────────────────
    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "format_inline_footnote", move || {
            crate::ui::toolbar::show_insert_footnote_popover(
                buf.upcast_ref::<gtk4::TextBuffer>(),
                view.upcast_ref::<gtk4::TextView>(),
            );
        });
    }

    // ── Popover: Emoji ────────────────────────────────────────────────────────
    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        let win = window.clone();
        let sm = settings_manager.clone();
        super::add_format_action(app, "format_inline_emoji", move || {
            crate::ui::toolbar::show_insert_emoji_popover(
                buf.upcast_ref::<gtk4::TextBuffer>(),
                view.upcast_ref::<gtk4::TextView>(),
                win.upcast_ref::<gtk4::Window>(),
                sm.clone(),
            );
        });
    }

    // ── Dialog: Mention ───────────────────────────────────────────────────────
    {
        let win = window.clone();
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "format_insert_mention", move || {
            crate::ui::dialogs::mention::show_insert_mention_dialog(
                win.upcast_ref::<gtk4::Window>(),
                &buf,
                &view,
            );
        });
    }
    app.set_accels_for_action("app.format_insert_mention", &["<Control><Shift>at"]);
}
