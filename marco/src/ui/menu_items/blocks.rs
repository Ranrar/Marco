use crate::components::language::Translations;
use gtk4::gio;
use gtk4::prelude::*;

pub fn populate_blocks_menu(blocks_menu: &gio::Menu, translations: &Translations) {
    blocks_menu.remove_all();

    let block_types = gio::Menu::new();

    // Block type group (kept visible; currently not wired as app actions).
    block_types.append(
        Some(&translations.menu.paragraph),
        Some("app.format_block_paragraph"),
    );
    block_types.append(
        Some(&translations.menu.blockquote),
        Some("app.format_block_blockquote"),
    );
    block_types.append(
        Some(&translations.menu.heading_1),
        Some("app.format_block_h1"),
    );
    block_types.append(
        Some(&translations.menu.heading_2),
        Some("app.format_block_h2"),
    );
    block_types.append(
        Some(&translations.menu.heading_3),
        Some("app.format_block_h3"),
    );
    block_types.append(
        Some(&translations.menu.heading_4),
        Some("app.format_block_h4"),
    );
    block_types.append(
        Some(&translations.menu.heading_5),
        Some("app.format_block_h5"),
    );
    block_types.append(
        Some(&translations.menu.heading_6),
        Some("app.format_block_h6"),
    );
    block_types.append(
        Some(&translations.menu.heading_id),
        Some("app.format_block_heading_id"),
    );
    blocks_menu.append_section(None, &block_types);

    let insertions = gio::Menu::new();

    // Existing wired block insertions.
    insertions.append(
        Some(&translations.menu.insert_code_block),
        Some("app.insert_code_block"),
    );
    insertions.append(
        Some(&translations.menu.insert_math),
        Some("app.insert_math_block"),
    );
    insertions.append(
        Some(&translations.menu.insert_footnote),
        Some("app.insert_footnote"),
    );
    insertions.append(
        Some(&translations.menu.lists),
        Some("app.format_insert_list"),
    );

    // Keep visible but disabled (not currently exposed as app action here).
    insertions.append(
        Some(&translations.menu.insert_horizontal_rule),
        Some("app.format_block_horizontal_rule"),
    );
    blocks_menu.append_section(None, &insertions);
}

pub fn setup_block_actions(
    app: &gtk4::Application,
    editor_buffer: &sourceview5::Buffer,
    editor_view: &sourceview5::View,
    window: &gtk4::ApplicationWindow,
) {
    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "insert_code_block", move || {
            super::insert_block_snippet(buf.upcast_ref::<gtk4::TextBuffer>(), "```\n\n```");
            super::refocus(&buf, &view);
        });
    }

    {
        let win = window.clone();
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "insert_math_block", move || {
            crate::ui::dialogs::math::show_insert_math_dialog(
                win.upcast_ref::<gtk4::Window>(),
                &buf,
                &view,
            );
        });
    }

    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "insert_footnote", move || {
            crate::ui::toolbar::show_insert_footnote_popover(
                buf.upcast_ref::<gtk4::TextBuffer>(),
                view.upcast_ref::<gtk4::TextView>(),
            );
        });
    }

    {
        let win = window.clone();
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "format_insert_list", move || {
            crate::ui::dialogs::lists::show_insert_list_dialog(
                win.upcast_ref::<gtk4::Window>(),
                &buf,
                &view,
            );
        });
    }

    app.set_accels_for_action("app.insert_code_block", &["<Control><Shift>c"]);
    app.set_accels_for_action("app.insert_math_block", &["<Control><Shift>m"]);
    app.set_accels_for_action("app.insert_footnote", &["<Control><Shift>f"]);
    app.set_accels_for_action("app.format_insert_list", &["<Control><Shift>l"]);

    // ── Block type: Paragraph (strip prefix) ──────────────────────────────────
    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "format_block_paragraph", move || {
            crate::ui::toolbar::apply_block_format_to_selection_or_current_line(
                buf.upcast_ref::<gtk4::TextBuffer>(),
                crate::ui::toolbar::SimpleBlockFormat::Paragraph,
            );
            super::refocus(&buf, &view);
        });
    }

    // ── Block type: Blockquote ────────────────────────────────────────────────
    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "format_block_blockquote", move || {
            crate::ui::toolbar::apply_block_format_to_selection_or_current_line(
                buf.upcast_ref::<gtk4::TextBuffer>(),
                crate::ui::toolbar::SimpleBlockFormat::Quote,
            );
            super::refocus(&buf, &view);
        });
    }
    app.set_accels_for_action("app.format_block_blockquote", &["<Control><Shift>q"]);

    // ── Block type: Heading 1-6 ───────────────────────────────────────────────
    for (name, level, accel) in [
        ("format_block_h1", 1u8, "<Control>1"),
        ("format_block_h2", 2u8, "<Control>2"),
        ("format_block_h3", 3u8, "<Control>3"),
        ("format_block_h4", 4u8, "<Control>4"),
        ("format_block_h5", 5u8, "<Control>5"),
        ("format_block_h6", 6u8, "<Control>6"),
    ] {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, name, move || {
            crate::ui::toolbar::apply_block_format_to_selection_or_current_line(
                buf.upcast_ref::<gtk4::TextBuffer>(),
                crate::ui::toolbar::SimpleBlockFormat::Heading(level),
            );
            super::refocus(&buf, &view);
        });
        app.set_accels_for_action(&format!("app.{}", name), &[accel]);
    }

    // ── Block: Heading with explicit ID suffix {#…} ───────────────────────────
    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "format_block_heading_id", move || {
            let text_buffer = buf.upcast_ref::<gtk4::TextBuffer>();
            let cursor_pos = text_buffer.cursor_position();
            let mut iter = text_buffer.iter_at_offset(cursor_pos);
            text_buffer.begin_user_action();
            text_buffer.insert(&mut iter, " {#}");
            text_buffer.end_user_action();
            // Place cursor inside the braces (between # and })
            let inside = text_buffer.iter_at_offset(cursor_pos + 3);
            text_buffer.place_cursor(&inside);
            super::refocus(&buf, &view);
        });
    }

    // ── Block: Horizontal rule (---) ─────────────────────────────────────────
    {
        let buf = editor_buffer.clone();
        let view = editor_view.clone();
        super::add_format_action(app, "format_block_horizontal_rule", move || {
            crate::ui::toolbar::insert_hr_at_cursor_and_refocus(
                buf.upcast_ref::<gtk4::TextBuffer>(),
                &view,
            );
        });
    }
    app.set_accels_for_action("app.format_block_horizontal_rule", &["<Control><Shift>r"]);
}
