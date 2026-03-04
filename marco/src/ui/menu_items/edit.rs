use crate::components::language::Translations;
use gtk4::{gdk, gio, prelude::*, TextBuffer};

/// Populate the Edit menu in grouped sections:
/// Undo/Redo, clipboard actions, selection, indent, and search.
pub fn populate_edit_menu(edit_menu: &gio::Menu, translations: &Translations) {
    edit_menu.remove_all();

    let history = gio::Menu::new();
    history.append(Some(&translations.menu.undo), Some("app.undo"));
    history.append(Some(&translations.menu.redo), Some("app.redo"));
    edit_menu.append_section(None, &history);

    let clipboard = gio::Menu::new();
    clipboard.append(Some(&translations.menu.cut), Some("app.cut"));
    clipboard.append(Some(&translations.menu.copy), Some("app.copy"));
    clipboard.append(Some(&translations.menu.paste), Some("app.paste"));
    clipboard.append(Some(&translations.menu.delete), Some("app.delete"));
    edit_menu.append_section(None, &clipboard);

    let selection = gio::Menu::new();
    selection.append(Some(&translations.menu.select_all), Some("app.select_all"));
    edit_menu.append_section(None, &selection);

    let indent = gio::Menu::new();
    indent.append(
        Some(&translations.menu.indent_increase),
        Some("app.indent_increase"),
    );
    indent.append(
        Some(&translations.menu.indent_decrease),
        Some("app.indent_decrease"),
    );
    edit_menu.append_section(None, &indent);

    let search = gio::Menu::new();
    search.append(Some(&translations.menu.search_replace), Some("app.search"));
    edit_menu.append_section(None, &search);
}

/// Register Edit menu actions and accelerators.
pub fn setup_edit_actions(
    app: &gtk4::Application,
    editor_buffer: &sourceview5::Buffer,
    editor_source_view: &sourceview5::View,
) {
    setup_undo_redo_actions(app, editor_buffer);
    setup_clipboard_actions(app, editor_buffer, editor_source_view);
    setup_selection_actions(app, editor_buffer);
    setup_indent_actions(app, editor_buffer);

    app.set_accels_for_action("app.undo", &["<Control>z"]);
    app.set_accels_for_action("app.redo", &["<Control>y"]);
    app.set_accels_for_action("app.cut", &["<Control>x"]);
    app.set_accels_for_action("app.copy", &["<Control>c"]);
    app.set_accels_for_action("app.paste", &["<Control>v"]);
    app.set_accels_for_action("app.delete", &["Delete"]);
    app.set_accels_for_action("app.select_all", &["<Control>a"]);
    app.set_accels_for_action("app.indent_increase", &["Tab"]);
    app.set_accels_for_action("app.indent_decrease", &["<Shift>Tab"]);

    // Keep top-level Edit menu actions state-aware, mirroring the context menu.
    {
        let app = app.clone();
        let signal_buffer = editor_buffer.clone();
        let state_buffer = editor_buffer.clone();
        let editor_source_view = editor_source_view.clone();
        signal_buffer.connect_changed(move |_| {
            update_edit_action_sensitivity(&app, &state_buffer, &editor_source_view);
        });
    }

    {
        let app = app.clone();
        let signal_buffer = editor_buffer.clone();
        let state_buffer = editor_buffer.clone();
        let editor_source_view = editor_source_view.clone();
        signal_buffer.connect_mark_set(move |_, _, _| {
            update_edit_action_sensitivity(&app, &state_buffer, &editor_source_view);
        });
    }

    // Initial state sync right after actions are installed.
    update_edit_action_sensitivity(app, editor_buffer, editor_source_view);
}

fn update_edit_action_sensitivity(
    app: &gtk4::Application,
    editor_buffer: &sourceview5::Buffer,
    editor_source_view: &sourceview5::View,
) {
    let has_selection = editor_buffer.selection_bounds().is_some();
    let can_edit = editor_source_view.is_editable();

    set_action_enabled(app, "undo", editor_buffer.can_undo());
    set_action_enabled(app, "redo", editor_buffer.can_redo());

    set_action_enabled(app, "cut", has_selection && can_edit);
    set_action_enabled(app, "copy", has_selection);
    set_action_enabled(app, "paste", can_edit);
    set_action_enabled(
        app,
        "delete",
        can_edit && (has_selection || editor_buffer.char_count() > 0),
    );

    set_action_enabled(app, "select_all", editor_buffer.char_count() > 0);
    set_action_enabled(app, "indent_increase", can_edit);

    let text_buffer: TextBuffer = editor_buffer.clone().upcast();
    set_action_enabled(
        app,
        "indent_decrease",
        can_edit && can_outdent_current_selection(&text_buffer),
    );
}

fn set_action_enabled(app: &gtk4::Application, action_name: &str, enabled: bool) {
    if let Some(action) = app
        .lookup_action(action_name)
        .and_then(|a| a.downcast::<gio::SimpleAction>().ok())
    {
        action.set_enabled(enabled);
    }
}

fn can_outdent_current_selection(text_buffer: &TextBuffer) -> bool {
    let (start_line, end_line) = selection_or_cursor_lines(text_buffer);

    for line in start_line..=end_line {
        if let Some(line_start) = text_buffer.iter_at_line(line) {
            let mut probe = line_start;
            if probe.forward_char() {
                let first = text_buffer.text(&line_start, &probe, false).to_string();
                if first == "\t" || first == " " {
                    return true;
                }
            }
        }
    }

    false
}

fn add_simple_action<F>(app: &gtk4::Application, name: &str, activate: F)
where
    F: Fn() + 'static,
{
    if app.lookup_action(name).is_some() {
        return;
    }

    let action = gio::SimpleAction::new(name, None);
    action.connect_activate(move |_, _| activate());
    app.add_action(&action);
}

fn setup_undo_redo_actions(app: &gtk4::Application, editor_buffer: &sourceview5::Buffer) {
    {
        let editor_buffer = editor_buffer.clone();
        add_simple_action(app, "undo", move || {
            if editor_buffer.can_undo() {
                editor_buffer.undo();
            }
        });
    }

    {
        let editor_buffer = editor_buffer.clone();
        add_simple_action(app, "redo", move || {
            if editor_buffer.can_redo() {
                editor_buffer.redo();
            }
        });
    }
}

fn setup_clipboard_actions(
    app: &gtk4::Application,
    editor_buffer: &sourceview5::Buffer,
    editor_source_view: &sourceview5::View,
) {
    {
        let editor_buffer = editor_buffer.clone();
        let editor_source_view = editor_source_view.clone();
        add_simple_action(app, "cut", move || {
            if let Some(display) = gdk::Display::default() {
                let clipboard = display.clipboard();
                editor_buffer.cut_clipboard(&clipboard, editor_source_view.is_editable());
            }
        });
    }

    {
        let editor_buffer = editor_buffer.clone();
        add_simple_action(app, "copy", move || {
            if let Some(display) = gdk::Display::default() {
                let clipboard = display.clipboard();
                editor_buffer.copy_clipboard(&clipboard);
            }
        });
    }

    {
        let editor_buffer = editor_buffer.clone();
        let editor_source_view = editor_source_view.clone();
        add_simple_action(app, "paste", move || {
            if let Some(display) = gdk::Display::default() {
                let clipboard = display.clipboard();
                editor_buffer.paste_clipboard(&clipboard, None, editor_source_view.is_editable());
            }
        });
    }

    {
        let text_buffer: TextBuffer = editor_buffer.clone().upcast();
        add_simple_action(app, "delete", move || {
            delete_selection_or_next_char(&text_buffer);
        });
    }
}

fn setup_selection_actions(app: &gtk4::Application, editor_buffer: &sourceview5::Buffer) {
    let text_buffer: TextBuffer = editor_buffer.clone().upcast();
    add_simple_action(app, "select_all", move || {
        let start = text_buffer.start_iter();
        let end = text_buffer.end_iter();
        text_buffer.select_range(&start, &end);
    });
}

fn setup_indent_actions(app: &gtk4::Application, editor_buffer: &sourceview5::Buffer) {
    {
        let text_buffer: TextBuffer = editor_buffer.clone().upcast();
        add_simple_action(app, "indent_increase", move || {
            increase_indent(&text_buffer);
        });
    }

    {
        let text_buffer: TextBuffer = editor_buffer.clone().upcast();
        add_simple_action(app, "indent_decrease", move || {
            decrease_indent(&text_buffer);
        });
    }
}

fn delete_selection_or_next_char(text_buffer: &TextBuffer) {
    if let Some((mut start, mut end)) = text_buffer.selection_bounds() {
        text_buffer.begin_user_action();
        text_buffer.delete(&mut start, &mut end);
        text_buffer.end_user_action();
        return;
    }

    let mut start = text_buffer.iter_at_offset(text_buffer.cursor_position());
    let mut end = start;
    if end.forward_char() {
        text_buffer.begin_user_action();
        text_buffer.delete(&mut start, &mut end);
        text_buffer.end_user_action();
    }
}

fn increase_indent(text_buffer: &TextBuffer) {
    let (start_line, end_line) = selection_or_cursor_lines(text_buffer);

    text_buffer.begin_user_action();

    for line in (start_line..=end_line).rev() {
        if let Some(mut iter) = text_buffer.iter_at_line(line) {
            text_buffer.insert(&mut iter, "\t");
        }
    }

    text_buffer.end_user_action();
}

fn decrease_indent(text_buffer: &TextBuffer) {
    let (start_line, end_line) = selection_or_cursor_lines(text_buffer);

    text_buffer.begin_user_action();

    for line in (start_line..=end_line).rev() {
        if let Some(mut line_start) = text_buffer.iter_at_line(line) {
            let mut next = line_start;
            if next.forward_char() {
                let first = text_buffer.text(&line_start, &next, false).to_string();

                if first == "\t" {
                    text_buffer.delete(&mut line_start, &mut next);
                    continue;
                }

                if first == " " {
                    // Remove up to 4 leading spaces (editor-style soft-tab outdent).
                    let mut remove_end = line_start;
                    let mut removed = 0usize;
                    while removed < 4 {
                        let mut probe = remove_end;
                        if !probe.forward_char() {
                            break;
                        }
                        let ch = text_buffer.text(&remove_end, &probe, false).to_string();
                        if ch != " " {
                            break;
                        }
                        remove_end = probe;
                        removed += 1;
                    }
                    if removed > 0 {
                        text_buffer.delete(&mut line_start, &mut remove_end);
                    }
                }
            }
        }
    }

    text_buffer.end_user_action();
}

fn selection_or_cursor_lines(text_buffer: &TextBuffer) -> (i32, i32) {
    if let Some((selection_start, selection_end)) = text_buffer.selection_bounds() {
        let start_line = selection_start.line().max(0);
        let mut end_line = selection_end.line().max(0);

        // If selection ends exactly at start of next line, keep transform bounded.
        if selection_end.starts_line() && end_line > start_line {
            end_line -= 1;
        }

        (start_line, end_line.max(start_line))
    } else {
        let cursor_iter = text_buffer.iter_at_offset(text_buffer.cursor_position());
        let line = cursor_iter.line().max(0);
        (line, line)
    }
}
