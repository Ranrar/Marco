//! Toolbar inline-footnote insertion helpers for Markdown (`[^id]` + definition).
//! Uses a compact popover similar to link insertion.

use gtk4::prelude::*;
use std::collections::HashSet;

const FOOTNOTE_POPOVER_WIDTH: i32 = 280;
const FOOTNOTE_POPOVER_HORIZONTAL_SAFE_PADDING: i32 = 8;

pub fn connect_inline_footnote_toolbar_action(
    toolbar: &gtk4::Box,
    editor_buffer: &sourceview5::Buffer,
    editor_view: &sourceview5::View,
    root_popover_state: crate::ui::popover_state::RootPopoverState,
) {
    if let Some(button) = find_button_by_css_class(
        toolbar.upcast_ref::<gtk4::Widget>(),
        "toolbar-btn-inline-footnote",
    ) {
        let editor_buffer = editor_buffer.clone();
        let editor_view = editor_view.clone();
        let root_popover_state = root_popover_state.clone();

        button.connect_clicked(move |_| {
            if root_popover_state.is_root_open() {
                return;
            }
            show_insert_footnote_popover(
                editor_buffer.upcast_ref::<gtk4::TextBuffer>(),
                editor_view.upcast_ref::<gtk4::TextView>(),
            );
        });
    }
}

pub fn connect_block_footnote_toolbar_action(
    toolbar: &gtk4::Box,
    editor_buffer: &sourceview5::Buffer,
    editor_view: &sourceview5::View,
    root_popover_state: crate::ui::popover_state::RootPopoverState,
) {
    if let Some(button) =
        find_button_by_css_class(toolbar.upcast_ref::<gtk4::Widget>(), "toolbar-btn-footnote")
    {
        let editor_buffer = editor_buffer.clone();
        let editor_view = editor_view.clone();
        let root_popover_state = root_popover_state.clone();

        button.connect_clicked(move |_| {
            if root_popover_state.is_root_open() {
                return;
            }
            show_insert_footnote_popover(
                editor_buffer.upcast_ref::<gtk4::TextBuffer>(),
                editor_view.upcast_ref::<gtk4::TextView>(),
            );
        });
    }
}

pub fn show_insert_footnote_popover(text_buffer: &gtk4::TextBuffer, editor_view: &gtk4::TextView) {
    let popover = gtk4::Popover::new();
    popover.set_has_arrow(true);
    popover.set_autohide(true);
    popover.set_position(gtk4::PositionType::Bottom);
    popover.set_can_focus(true);
    popover.add_css_class("marco-link-popover");
    popover.set_parent(editor_view);

    let root = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    root.set_margin_start(8);
    root.set_margin_end(8);
    root.set_margin_top(6);
    root.set_margin_bottom(6);
    root.set_width_request(FOOTNOTE_POPOVER_WIDTH);

    let title = gtk4::Label::new(Some("Footnote"));
    title.set_halign(gtk4::Align::Start);
    title.add_css_class("marco-dialog-section-label");

    let next_number = next_available_footnote_number(text_buffer);

    let id_entry = gtk4::Entry::new();
    id_entry.set_hexpand(true);
    id_entry.set_placeholder_text(Some("Footnote id (required)"));
    id_entry.set_text(&next_number.to_string());
    id_entry.add_css_class("marco-search-entry");

    let note_view = gtk4::TextView::new();
    note_view.set_wrap_mode(gtk4::WrapMode::WordChar);
    note_view.set_monospace(true);
    note_view.set_vexpand(true);

    let note_scroller = gtk4::ScrolledWindow::new();
    note_scroller.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    note_scroller.set_min_content_height(96);
    note_scroller.set_has_frame(false);
    note_scroller.add_css_class("marco-textfield-scroll");
    note_scroller.set_child(Some(&note_view));

    let actions = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    actions.set_halign(gtk4::Align::End);
    actions.set_margin_top(2);

    let cancel_button = gtk4::Button::with_label("Cancel");
    cancel_button.add_css_class("marco-btn");
    cancel_button.add_css_class("marco-btn-yellow");

    let ok_button = gtk4::Button::with_label("Ok");
    ok_button.add_css_class("marco-btn");
    ok_button.add_css_class("marco-btn-blue");

    actions.append(&cancel_button);
    actions.append(&ok_button);

    root.append(&title);
    root.append(&id_entry);
    root.append(&note_scroller);
    root.append(&actions);

    popover.set_child(Some(&root));

    {
        let popover = popover.clone();
        let editor_view = editor_view.clone();
        cancel_button.connect_clicked(move |_| {
            popover.popdown();
            editor_view.grab_focus();
        });
    }

    {
        let text_buffer = text_buffer.clone();
        let editor_view = editor_view.clone();
        let popover = popover.clone();
        let id_entry = id_entry.clone();
        let note_view = note_view.clone();
        ok_button.connect_clicked(move |_| {
            submit_inline_footnote_from_popover_entries(
                &text_buffer,
                &editor_view,
                &popover,
                &id_entry,
                &note_view,
            );
        });
    }

    {
        let text_buffer = text_buffer.clone();
        let editor_view = editor_view.clone();
        let popover = popover.clone();
        let id_entry_for_signal = id_entry.clone();
        let id_entry = id_entry.clone();
        let note_view = note_view.clone();
        id_entry_for_signal.connect_activate(move |_| {
            submit_inline_footnote_from_popover_entries(
                &text_buffer,
                &editor_view,
                &popover,
                &id_entry,
                &note_view,
            );
        });
    }

    {
        let note_buffer = note_view.buffer();
        let note_view = note_view.clone();
        let id_entry = id_entry.clone();
        let ok_button = ok_button.clone();
        note_buffer.connect_changed(move |_| {
            update_footnote_ok_button_sensitivity(&id_entry, &note_view, &ok_button);
        });
    }

    {
        let signal_id_entry = id_entry.clone();
        let id_entry = id_entry.clone();
        let note_view = note_view.clone();
        let ok_button = ok_button.clone();
        signal_id_entry.connect_changed(move |_| {
            update_footnote_ok_button_sensitivity(&id_entry, &note_view, &ok_button);
        });
    }

    let caret_rect = cursor_rect(text_buffer, editor_view);
    let clamped_rect = clamp_rect_to_editor(caret_rect, editor_view);
    popover.set_pointing_to(Some(&clamped_rect));

    if let Some(text_area) = visible_text_area_widget_rect(editor_view) {
        let x_offset = compute_popover_x_offset_for_text_area(
            clamped_rect.x(),
            text_area.x(),
            text_area.x() + text_area.width(),
            FOOTNOTE_POPOVER_WIDTH,
            FOOTNOTE_POPOVER_HORIZONTAL_SAFE_PADDING,
        );
        popover.set_offset(x_offset, 0);
    }

    update_footnote_ok_button_sensitivity(&id_entry, &note_view, &ok_button);

    popover.popup();
    id_entry.grab_focus();
}

fn submit_inline_footnote_from_popover_entries(
    text_buffer: &gtk4::TextBuffer,
    editor_view: &gtk4::TextView,
    popover: &gtk4::Popover,
    id_entry: &gtk4::Entry,
    note_view: &gtk4::TextView,
) {
    let id = id_entry.text().trim().to_string();
    if id.is_empty() {
        id_entry.grab_focus();
        return;
    }

    let note_buffer = note_view.buffer();
    let note_text = note_buffer
        .text(&note_buffer.start_iter(), &note_buffer.end_iter(), false)
        .to_string();
    let trimmed_note = note_text.trim();
    if trimmed_note.is_empty() {
        note_view.grab_focus();
        return;
    }

    insert_inline_footnote_markdown(text_buffer, &id, trimmed_note);

    popover.popdown();
    editor_view.grab_focus();
}

fn update_footnote_ok_button_sensitivity(
    id_entry: &gtk4::Entry,
    note_view: &gtk4::TextView,
    ok_button: &gtk4::Button,
) {
    let has_id = !id_entry.text().trim().is_empty();
    let note_buffer = note_view.buffer();
    let note_text = note_buffer
        .text(&note_buffer.start_iter(), &note_buffer.end_iter(), false)
        .to_string();
    let has_note = !note_text.trim().is_empty();

    ok_button.set_sensitive(has_id && has_note);
}

fn insert_inline_footnote_markdown(text_buffer: &gtk4::TextBuffer, footnote_id: &str, note: &str) {
    let reference = format!("[^{}]", footnote_id.trim());

    let (mut start_iter, mut end_iter) = insertion_bounds(text_buffer);
    let start_offset = start_iter.offset();

    let prev_char = char_before_offset(text_buffer, start_offset);
    let next_char = char_after_offset(text_buffer, end_iter.offset());

    let prefix_space = if should_insert_space_before(prev_char) {
        " "
    } else {
        ""
    };
    let suffix_space = if should_insert_space_after(next_char) {
        " "
    } else {
        ""
    };

    let insertion = format!("{prefix_space}{reference}{suffix_space}");

    text_buffer.begin_user_action();

    text_buffer.delete(&mut start_iter, &mut end_iter);
    text_buffer.insert(&mut start_iter, &insertion);

    let cursor_offset =
        start_offset + prefix_space.chars().count() as i32 + reference.chars().count() as i32;
    let cursor_iter = text_buffer.iter_at_offset(cursor_offset);
    text_buffer.place_cursor(&cursor_iter);

    append_footnote_definition_to_document_end(text_buffer, footnote_id.trim(), note);

    text_buffer.end_user_action();
}

fn append_footnote_definition_to_document_end(
    text_buffer: &gtk4::TextBuffer,
    footnote_id: &str,
    note: &str,
) {
    let definition = build_footnote_definition(footnote_id, note);

    let start = text_buffer.start_iter();
    let end = text_buffer.end_iter();
    let document = text_buffer.text(&start, &end, false).to_string();

    let separator = if document.is_empty() {
        ""
    } else if document.ends_with("\n\n") {
        ""
    } else if document.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };

    let insertion = format!("{separator}{definition}");
    let mut end_iter = text_buffer.end_iter();
    text_buffer.insert(&mut end_iter, &insertion);
}

fn build_footnote_definition(footnote_id: &str, note: &str) -> String {
    let normalized = note.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines = normalized.lines();

    let first_line = lines.next().unwrap_or_default();
    let mut out = format!("[^{}]: {}", footnote_id, first_line);

    for line in lines {
        out.push('\n');
        if line.is_empty() {
            out.push_str("    ");
        } else {
            out.push_str("    ");
            out.push_str(line);
        }
    }

    out
}

fn next_available_footnote_number(text_buffer: &gtk4::TextBuffer) -> u32 {
    let start = text_buffer.start_iter();
    let end = text_buffer.end_iter();
    let document = text_buffer.text(&start, &end, false).to_string();
    next_available_footnote_number_for_document(&document)
}

fn next_available_footnote_number_for_document(document: &str) -> u32 {
    let used = collect_used_numeric_footnote_ids(document);
    let mut next = 1u32;
    while used.contains(&next) {
        next += 1;
    }
    next
}

fn collect_used_numeric_footnote_ids(document: &str) -> HashSet<u32> {
    let mut used = HashSet::new();
    let bytes = document.as_bytes();
    let mut i = 0usize;

    while i + 2 < bytes.len() {
        if bytes[i] == b'[' && bytes[i + 1] == b'^' {
            let mut j = i + 2;
            while j < bytes.len() && bytes[j] != b']' {
                j += 1;
            }

            if j < bytes.len() {
                let label = &document[i + 2..j];
                if !label.is_empty() && label.chars().all(|ch| ch.is_ascii_digit()) {
                    if let Ok(value) = label.parse::<u32>() {
                        if value > 0 {
                            used.insert(value);
                        }
                    }
                }
                i = j + 1;
                continue;
            }

            break;
        }

        i += 1;
    }

    used
}

fn insertion_bounds(text_buffer: &gtk4::TextBuffer) -> (gtk4::TextIter, gtk4::TextIter) {
    if let Some((start, end)) = text_buffer.selection_bounds() {
        if start.offset() != end.offset() {
            return (start, end);
        }
    }

    let cursor = text_buffer.iter_at_offset(text_buffer.cursor_position());
    (cursor, cursor)
}

fn char_before_offset(text_buffer: &gtk4::TextBuffer, offset: i32) -> Option<char> {
    if offset <= 0 {
        return None;
    }

    let prev = text_buffer.iter_at_offset(offset - 1);
    let curr = text_buffer.iter_at_offset(offset);
    text_buffer.text(&prev, &curr, false).chars().next()
}

fn char_after_offset(text_buffer: &gtk4::TextBuffer, offset: i32) -> Option<char> {
    if offset >= text_buffer.char_count() {
        return None;
    }

    let curr = text_buffer.iter_at_offset(offset);
    let next = text_buffer.iter_at_offset(offset + 1);
    text_buffer.text(&curr, &next, false).chars().next()
}

fn should_insert_space_before(prev_char: Option<char>) -> bool {
    prev_char.is_some_and(is_wordish)
}

fn should_insert_space_after(next_char: Option<char>) -> bool {
    next_char.is_some_and(is_wordish)
}

fn is_wordish(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn cursor_rect(
    text_buffer: &gtk4::TextBuffer,
    editor_view: &gtk4::TextView,
) -> gtk4::gdk::Rectangle {
    let iter = text_buffer.iter_at_offset(text_buffer.cursor_position());
    let rect = editor_view.iter_location(&iter);
    let (widget_x, widget_y) =
        editor_view.buffer_to_window_coords(gtk4::TextWindowType::Widget, rect.x(), rect.y());

    gtk4::gdk::Rectangle::new(
        widget_x,
        widget_y,
        rect.width().max(1),
        rect.height().max(1),
    )
}

fn clamp_rect_to_editor(
    rect: gtk4::gdk::Rectangle,
    editor_view: &gtk4::TextView,
) -> gtk4::gdk::Rectangle {
    let view_w = editor_view.allocated_width().max(1);
    let view_h = editor_view.allocated_height().max(1);
    let w = rect.width().max(1);
    let h = rect.height().max(1);

    let max_x = (view_w - w).max(0);
    let max_y = (view_h - h).max(0);
    let x = rect.x().clamp(0, max_x);
    let y = rect.y().clamp(0, max_y);

    gtk4::gdk::Rectangle::new(x, y, w, h)
}

fn visible_text_area_widget_rect(editor_view: &gtk4::TextView) -> Option<gtk4::gdk::Rectangle> {
    let visible = editor_view.visible_rect();
    if visible.width() <= 0 || visible.height() <= 0 {
        return None;
    }

    let (left, top) =
        editor_view.buffer_to_window_coords(gtk4::TextWindowType::Widget, visible.x(), visible.y());
    let (right, bottom) = editor_view.buffer_to_window_coords(
        gtk4::TextWindowType::Widget,
        visible.x() + visible.width(),
        visible.y() + visible.height(),
    );

    let x = left.min(right);
    let y = top.min(bottom);
    let w = (right - left).abs().max(1);
    let h = (bottom - top).abs().max(1);

    Some(gtk4::gdk::Rectangle::new(x, y, w, h))
}

fn compute_popover_x_offset_for_text_area(
    cursor_x: i32,
    text_left: i32,
    text_right: i32,
    popover_width: i32,
    safe_padding: i32,
) -> i32 {
    let half = (popover_width / 2).max(1);
    let desired_left = cursor_x - half;
    let desired_right = cursor_x + half;

    let min_left = text_left + safe_padding;
    let max_right = text_right - safe_padding;

    if desired_left < min_left {
        min_left - desired_left
    } else if desired_right > max_right {
        max_right - desired_right
    } else {
        0
    }
}

fn find_button_by_css_class(root: &gtk4::Widget, css_class: &str) -> Option<gtk4::Button> {
    if let Ok(button) = root.clone().downcast::<gtk4::Button>() {
        if button.has_css_class(css_class) {
            return Some(button);
        }
    }

    let mut child = root.first_child();
    while let Some(widget) = child {
        if let Some(found) = find_button_by_css_class(&widget, css_class) {
            return Some(found);
        }
        child = widget.next_sibling();
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_next_available_footnote_number_fills_gap() {
        let doc = "Text [^1]\n\n[^1]: one\n[^3]: three\nmore [^3]\n";
        assert_eq!(next_available_footnote_number_for_document(doc), 2);
    }

    #[test]
    fn smoke_test_next_available_footnote_number_starts_at_one() {
        assert_eq!(next_available_footnote_number_for_document("No notes"), 1);
    }

    #[test]
    fn smoke_test_build_footnote_definition_single_line() {
        let definition = build_footnote_definition("2", "simple note");
        assert_eq!(definition, "[^2]: simple note");
    }

    #[test]
    fn smoke_test_build_footnote_definition_multiline_markdown() {
        let definition = build_footnote_definition("4", "line1\n- item\n\n```rs\nlet x = 1;\n```");
        assert_eq!(
            definition,
            "[^4]: line1\n    - item\n    \n    ```rs\n    let x = 1;\n    ```"
        );
    }

    #[test]
    fn smoke_test_space_rules_for_inline_reference() {
        assert!(should_insert_space_before(Some('a')));
        assert!(should_insert_space_after(Some('9')));
        assert!(!should_insert_space_before(Some(' ')));
        assert!(!should_insert_space_after(None));
    }
}
