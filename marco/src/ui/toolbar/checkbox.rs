//! Toolbar inline-checkbox toggle helpers for Markdown (`[ ]` / `[x]`).
//! Behavior mirrors inline toggles like math: detect/toggle at cursor, or insert when absent.

use gtk4::prelude::*;

pub fn connect_inline_checkbox_toolbar_action(
    toolbar: &gtk4::Box,
    editor_buffer: &sourceview5::Buffer,
    editor_view: &sourceview5::View,
    root_popover_state: crate::ui::popover_state::RootPopoverState,
) {
    if let Some(button) = find_button_by_css_class(
        toolbar.upcast_ref::<gtk4::Widget>(),
        "toolbar-btn-inline-checkbox",
    ) {
        let editor_buffer = editor_buffer.clone();
        let editor_view = editor_view.clone();
        let root_popover_state = root_popover_state.clone();
        button.connect_clicked(move |_| {
            if root_popover_state.is_root_open() {
                return;
            }
            toggle_inline_checkbox(editor_buffer.upcast_ref::<gtk4::TextBuffer>());
            refocus_current_line(&editor_buffer, &editor_view);
        });
    }
}

fn refocus_current_line(editor_buffer: &sourceview5::Buffer, editor_view: &sourceview5::View) {
    let mut iter = editor_buffer.iter_at_offset(editor_buffer.cursor_position());
    editor_buffer.place_cursor(&iter);
    editor_view.scroll_to_iter(&mut iter, 0.15, true, 0.0, 0.35);
    editor_view.grab_focus();
}

pub fn toggle_inline_checkbox(text_buffer: &gtk4::TextBuffer) {
    if let Some((mut start, mut end)) = text_buffer.selection_bounds() {
        if start.offset() != end.offset() {
            let selected = text_buffer.text(&start, &end, false).to_string();
            let replaced = toggle_or_prefix_checkbox_for_text(&selected);

            text_buffer.begin_user_action();
            text_buffer.delete(&mut start, &mut end);
            text_buffer.insert(&mut start, &replaced);
            text_buffer.end_user_action();
            return;
        }
    }

    let cursor_iter = text_buffer.iter_at_offset(text_buffer.cursor_position());
    let line = cursor_iter.line().max(0);
    let cursor_col = cursor_iter.line_offset().max(0) as usize;

    let Some(line_start) = text_buffer.iter_at_line(line) else {
        return;
    };
    let mut line_end = line_start;
    line_end.forward_to_line_end();
    let line_text = text_buffer.text(&line_start, &line_end, false).to_string();

    if let Some((marker_start, marker_end, checked)) =
        find_checkbox_marker_at_cursor(&line_text, cursor_col)
    {
        let Some(mut marker_start_iter) = iter_at_line_col(text_buffer, line, marker_start as i32)
        else {
            return;
        };
        let Some(mut marker_end_iter) = iter_at_line_col(text_buffer, line, marker_end as i32)
        else {
            return;
        };

        let replacement = if checked { "[ ]" } else { "[x]" };

        text_buffer.begin_user_action();
        text_buffer.delete(&mut marker_start_iter, &mut marker_end_iter);
        text_buffer.insert(&mut marker_start_iter, replacement);
        text_buffer.end_user_action();
        return;
    }

    insert_empty_checkbox_marker(text_buffer);
}

fn toggle_or_prefix_checkbox_for_text(text: &str) -> String {
    if let Some((checked, consumed)) = parse_leading_checkbox_marker(text) {
        let rest = &text[consumed..];
        let replacement = if checked { "[ ]" } else { "[x]" };
        format!("{}{}", replacement, rest)
    } else {
        format!("[ ] {}", text)
    }
}

fn insert_empty_checkbox_marker(text_buffer: &gtk4::TextBuffer) {
    let cursor_pos = text_buffer.cursor_position();
    let mut iter = text_buffer.iter_at_offset(cursor_pos);

    text_buffer.begin_user_action();
    text_buffer.insert(&mut iter, "[ ] ");
    text_buffer.end_user_action();

    let after_marker = text_buffer.iter_at_offset(cursor_pos + 4);
    text_buffer.place_cursor(&after_marker);
}

fn parse_leading_checkbox_marker(text: &str) -> Option<(bool, usize)> {
    if text.starts_with("[ ]") {
        Some((false, 3))
    } else if text.starts_with("[x]") || text.starts_with("[X]") {
        Some((true, 3))
    } else {
        None
    }
}

fn find_checkbox_marker_at_cursor(
    line_text: &str,
    cursor_col: usize,
) -> Option<(usize, usize, bool)> {
    let chars: Vec<char> = line_text.chars().collect();
    if chars.len() < 3 {
        return None;
    }

    let cursor = cursor_col.min(chars.len().saturating_sub(1));

    for i in 0..=chars.len().saturating_sub(3) {
        let marker = if chars[i] == '[' && chars[i + 1] == ' ' && chars[i + 2] == ']' {
            Some(false)
        } else if chars[i] == '['
            && (chars[i + 1] == 'x' || chars[i + 1] == 'X')
            && chars[i + 2] == ']'
        {
            Some(true)
        } else {
            None
        };

        let Some(checked) = marker else {
            continue;
        };

        if !is_start_boundary_valid(&chars, i) || !is_end_boundary_valid(&chars, i + 3) {
            continue;
        }

        let span_start = i;
        let span_end = i + 3;
        if cursor >= span_start && cursor <= span_end {
            return Some((span_start, span_end, checked));
        }
    }

    None
}

fn is_start_boundary_valid(chars: &[char], marker_start: usize) -> bool {
    if marker_start == 0 {
        return true;
    }

    let prev = chars[marker_start - 1];
    prev.is_whitespace() || matches!(prev, '(' | '[' | '{' | ',' | ';' | ':' | '!' | '?')
}

fn is_end_boundary_valid(chars: &[char], marker_end: usize) -> bool {
    if marker_end >= chars.len() {
        return true;
    }

    let next = chars[marker_end];
    next.is_whitespace() || matches!(next, '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '}')
}

fn iter_at_line_col(text_buffer: &gtk4::TextBuffer, line: i32, col: i32) -> Option<gtk4::TextIter> {
    let mut iter = text_buffer.iter_at_line(line)?;
    iter.set_line_offset(col.max(0));
    Some(iter)
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
    fn smoke_test_toggle_or_prefix_checkbox_for_text_prefixes_when_missing() {
        assert_eq!(toggle_or_prefix_checkbox_for_text("todo"), "[ ] todo");
    }

    #[test]
    fn smoke_test_toggle_or_prefix_checkbox_for_text_toggles_leading_marker() {
        assert_eq!(toggle_or_prefix_checkbox_for_text("[ ] todo"), "[x] todo");
        assert_eq!(toggle_or_prefix_checkbox_for_text("[x] todo"), "[ ] todo");
    }

    #[test]
    fn smoke_test_find_checkbox_marker_at_cursor_detects_checked() {
        let found = find_checkbox_marker_at_cursor("Do this [x] today", 9);
        assert_eq!(found, Some((8, 11, true)));
    }

    #[test]
    fn smoke_test_find_checkbox_marker_at_cursor_rejects_link_like_continuation() {
        let found = find_checkbox_marker_at_cursor("[x](link)", 1);
        assert_eq!(found, None);
    }
}
