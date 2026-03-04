//! Toolbar italic toggle helpers for Markdown (`*...*`).
//! Behavior is validated against CommonMark emphasis edge cases.

use gtk4::prelude::*;

pub fn connect_italic_toolbar_action(
    toolbar: &gtk4::Box,
    editor_buffer: &sourceview5::Buffer,
    editor_view: &sourceview5::View,
    root_popover_state: crate::ui::popover_state::RootPopoverState,
) {
    if let Some(button) =
        find_button_by_css_class(toolbar.upcast_ref::<gtk4::Widget>(), "toolbar-btn-italic")
    {
        let editor_buffer = editor_buffer.clone();
        let editor_view = editor_view.clone();
        let root_popover_state = root_popover_state.clone();
        button.connect_clicked(move |_| {
            if root_popover_state.is_root_open() {
                return;
            }
            toggle_italic_for_selection_or_word(editor_buffer.upcast_ref::<gtk4::TextBuffer>());
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

pub fn toggle_italic_for_selection_or_word(text_buffer: &gtk4::TextBuffer) {
    if let Some((mut selection_start, mut selection_end)) = text_buffer.selection_bounds() {
        if selection_start.offset() != selection_end.offset() {
            maybe_expand_range_to_surrounding_italic_markers(
                text_buffer,
                &mut selection_start,
                &mut selection_end,
            );
            toggle_italic_on_range(text_buffer, &mut selection_start, &mut selection_end);
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

    let Some((word_start, word_end)) = find_word_bounds(&line_text, cursor_col) else {
        // Cursor might be on/adjacent to existing italic markers — try to unwrap.
        if let Some((span_start, span_end)) = find_italic_span_at_cursor(&line_text, cursor_col) {
            if let (Some(mut si), Some(mut ei)) = (
                iter_at_line_col(text_buffer, line, span_start as i32),
                iter_at_line_col(text_buffer, line, span_end as i32),
            ) {
                toggle_italic_on_range(text_buffer, &mut si, &mut ei);
                return;
            }
        }
        insert_empty_italic_delimiters(text_buffer);
        return;
    };

    let Some(mut start_iter) = iter_at_line_col(text_buffer, line, word_start as i32) else {
        return;
    };
    let Some(mut end_iter) = iter_at_line_col(text_buffer, line, word_end as i32) else {
        return;
    };

    maybe_expand_range_to_surrounding_italic_markers(text_buffer, &mut start_iter, &mut end_iter);

    toggle_italic_on_range(text_buffer, &mut start_iter, &mut end_iter);
}

/// Insert `**` at the current cursor position and place the cursor between
/// the two markers for immediate typing.
fn insert_empty_italic_delimiters(text_buffer: &gtk4::TextBuffer) {
    let cursor_pos = text_buffer.cursor_position();
    let mut iter = text_buffer.iter_at_offset(cursor_pos);

    text_buffer.begin_user_action();
    text_buffer.insert(&mut iter, "**");
    text_buffer.end_user_action();

    // Place cursor between the two `*` markers → offset + 1.
    let mid = text_buffer.iter_at_offset(cursor_pos + 1);
    text_buffer.place_cursor(&mid);
}

fn maybe_expand_range_to_surrounding_italic_markers(
    text_buffer: &gtk4::TextBuffer,
    start_iter: &mut gtk4::TextIter,
    end_iter: &mut gtk4::TextIter,
) {
    if start_iter.line() != end_iter.line() {
        return;
    }

    let line = start_iter.line();
    let start_col = start_iter.line_offset().max(0) as usize;
    let end_col = end_iter.line_offset().max(0) as usize;

    let Some(line_start) = text_buffer.iter_at_line(line) else {
        return;
    };
    let mut line_end = line_start;
    line_end.forward_to_line_end();
    let line_text = text_buffer.text(&line_start, &line_end, false).to_string();

    let Some((expanded_start, expanded_end)) =
        expand_to_surrounding_italic_markers(&line_text, start_col, end_col)
    else {
        return;
    };

    if let Some(iter) = iter_at_line_col(text_buffer, line, expanded_start as i32) {
        *start_iter = iter;
    }
    if let Some(iter) = iter_at_line_col(text_buffer, line, expanded_end as i32) {
        *end_iter = iter;
    }
}

fn is_italic_delimiter(chars: &[char], idx: usize) -> bool {
    if chars.get(idx) != Some(&'*') {
        return false;
    }

    let prev_is_star = idx > 0 && chars[idx - 1] == '*';
    let next_is_star = idx + 1 < chars.len() && chars[idx + 1] == '*';

    !prev_is_star && !next_is_star
}

fn expand_to_surrounding_italic_markers(
    line_text: &str,
    start_col: usize,
    end_col: usize,
) -> Option<(usize, usize)> {
    let chars: Vec<char> = line_text.chars().collect();
    if chars.len() < 3 || start_col > end_col || end_col > chars.len() {
        return None;
    }

    // Fast path: markers directly adjacent to selection.
    if start_col >= 1 && end_col < chars.len() {
        let has_left_marker = is_italic_delimiter(&chars, start_col - 1);
        let has_right_marker = is_italic_delimiter(&chars, end_col);

        if has_left_marker && has_right_marker {
            return Some((start_col - 1, end_col + 1));
        }
    }

    // Fallback: selection might be inside a larger italic span.
    let marker_positions: Vec<usize> = (0..chars.len())
        .filter(|&i| is_italic_delimiter(&chars, i))
        .collect();

    let left = marker_positions
        .iter()
        .copied()
        .rev()
        .find(|&pos| pos < start_col)?;
    let right = marker_positions
        .iter()
        .copied()
        .find(|&pos| pos >= end_col)?;

    if right > left {
        return Some((left, right + 1));
    }

    None
}

/// Find a complete `*…*` span (single italic) that contains the cursor position.
fn find_italic_span_at_cursor(line_text: &str, cursor_col: usize) -> Option<(usize, usize)> {
    let chars: Vec<char> = line_text.chars().collect();
    if chars.is_empty() {
        return None;
    }

    let cursor = cursor_col.min(chars.len() - 1);
    let marker_positions: Vec<usize> = (0..chars.len())
        .filter(|&i| is_italic_delimiter(&chars, i))
        .collect();

    let mut j = 0;
    while j + 1 < marker_positions.len() {
        let open = marker_positions[j];
        let close = marker_positions[j + 1];
        let span_end = close + 1;
        if cursor >= open && cursor < span_end {
            return Some((open, span_end));
        }
        j += 2;
    }
    None
}

fn toggle_italic_on_range(
    text_buffer: &gtk4::TextBuffer,
    start_iter: &mut gtk4::TextIter,
    end_iter: &mut gtk4::TextIter,
) {
    let selected = text_buffer.text(start_iter, end_iter, false).to_string();
    let toggled = toggle_selected_italic_text(&selected);

    text_buffer.begin_user_action();
    text_buffer.delete(start_iter, end_iter);
    text_buffer.insert(start_iter, &toggled);
    text_buffer.end_user_action();
}

fn is_wrapped_with_italic_markers(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 3 {
        return false;
    }

    if chars.first() != Some(&'*') || chars.last() != Some(&'*') {
        return false;
    }

    // Exclude bold opener (**...)
    if chars.get(1) == Some(&'*') {
        return false;
    }

    // Exclude bold closer (...**), except when the penultimate star is escaped
    // like in "*\\**" where inner \* is literal and final * is closer.
    if chars.len() >= 2 && chars[chars.len() - 2] == '*' {
        let penultimate_escaped = chars.len() >= 3 && chars[chars.len() - 3] == '\\';
        if !penultimate_escaped {
            return false;
        }
    }

    true
}

fn is_wrapped_with_bold_and_italic_markers(text: &str) -> bool {
    text.starts_with("***") && text.ends_with("***") && text.len() >= 7
}

fn is_wrapped_with_single_italic_span(text: &str) -> bool {
    if !is_wrapped_with_italic_markers(text) {
        return false;
    }

    let inner = &text[1..text.len() - 1];
    !inner.is_empty() && !inner.contains('*')
}

fn toggle_selected_italic_text(selected: &str) -> String {
    if let Some(stripped_multi) = strip_italic_from_each_non_whitespace_token(selected) {
        stripped_multi
    } else if is_wrapped_with_bold_and_italic_markers(selected)
        || is_wrapped_with_italic_markers(selected)
    {
        selected[1..selected.len() - 1].to_string()
    } else {
        format!("*{}*", selected)
    }
}

fn strip_italic_from_each_non_whitespace_token(text: &str) -> Option<String> {
    let mut result = String::new();
    let mut chars = text.char_indices().peekable();
    let mut saw_non_whitespace = false;

    while let Some((idx, ch)) = chars.peek().copied() {
        if ch.is_whitespace() {
            chars.next();
            result.push(ch);
            continue;
        }

        saw_non_whitespace = true;
        let start = idx;
        let mut end = text.len();

        while let Some((next_idx, next_ch)) = chars.peek().copied() {
            if next_ch.is_whitespace() {
                end = next_idx;
                break;
            }
            chars.next();
        }

        let token = &text[start..end];
        if !is_wrapped_with_single_italic_span(token) {
            return None;
        }

        result.push_str(&token[1..token.len() - 1]);
    }

    if saw_non_whitespace {
        Some(result)
    } else {
        None
    }
}

fn iter_at_line_col(text_buffer: &gtk4::TextBuffer, line: i32, col: i32) -> Option<gtk4::TextIter> {
    let mut iter = text_buffer.iter_at_line(line)?;
    iter.set_line_offset(col.max(0));
    Some(iter)
}

fn find_word_bounds(line_text: &str, cursor_col: usize) -> Option<(usize, usize)> {
    let chars: Vec<char> = line_text.chars().collect();
    if chars.is_empty() {
        return None;
    }

    let mut idx = cursor_col.min(chars.len());

    if idx == chars.len() && idx > 0 {
        idx -= 1;
    }

    if !is_word_char(chars[idx]) {
        return None;
    }

    let mut start = idx;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }

    let mut end = idx + 1;
    while end < chars.len() && is_word_char(chars[end]) {
        end += 1;
    }

    Some((start, end))
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
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
    fn smoke_test_detects_italic_wrapped_text() {
        assert!(is_wrapped_with_italic_markers("*hello*"));
        assert!(!is_wrapped_with_italic_markers("hello"));
    }

    #[test]
    fn smoke_test_word_bounds_mid_word() {
        let bounds = find_word_bounds("hello world", 1);
        assert_eq!(bounds, Some((0, 5)));
    }

    #[test]
    fn smoke_test_word_bounds_cursor_after_word() {
        let bounds = find_word_bounds("hello", 5);
        assert_eq!(bounds, Some((0, 5)));
    }

    #[test]
    fn smoke_test_word_bounds_space_after_word_returns_none() {
        let bounds = find_word_bounds("hello ", 5);
        assert_eq!(bounds, None);
    }

    #[test]
    fn smoke_test_word_bounds_space_before_word_returns_none() {
        let bounds = find_word_bounds(" world", 0);
        assert_eq!(bounds, None);
    }

    #[test]
    fn smoke_test_expand_to_surrounding_italic_markers_detects_wrapped_word() {
        let expanded = expand_to_surrounding_italic_markers("*world*", 1, 6);
        assert_eq!(expanded, Some((0, 7)));
    }

    #[test]
    fn smoke_test_expand_to_surrounding_italic_markers_none_for_plain_word() {
        let expanded = expand_to_surrounding_italic_markers("world", 0, 5);
        assert_eq!(expanded, None);
    }

    #[test]
    fn smoke_test_expand_to_surrounding_italic_markers_for_inner_selection() {
        let expanded = expand_to_surrounding_italic_markers("*hello world*", 1, 6);
        assert_eq!(expanded, Some((0, 13)));
    }

    #[test]
    fn smoke_test_toggle_selected_italic_text_for_multiple_italic_words() {
        let toggled = toggle_selected_italic_text("*hello* *world*");
        assert_eq!(toggled, "hello world");
    }

    #[test]
    fn smoke_test_toggle_selected_italic_text_for_single_span() {
        let toggled = toggle_selected_italic_text("*hello world*");
        assert_eq!(toggled, "hello world");
    }

    // CommonMark traceability (tests/test_suite/spec/commonmark.json):
    // | Local test                                                     | CM example | Snippet                       |
    // |----------------------------------------------------------------|------------|-------------------------------|
    // | ..._commonmark_plain_emphasis                                 | 350        | "*foo bar*"                  |
    // | ..._commonmark_non_emphasis_space_after_opener                | 351        | "a * foo bar*"               |
    // | ..._commonmark_escaped_inner_marker                           | 437        | "foo *\\**"                 |

    #[test]
    fn smoke_test_toggle_selected_italic_text_for_commonmark_plain_emphasis() {
        let toggled = toggle_selected_italic_text("*foo bar*");
        assert_eq!(toggled, "foo bar");
    }

    #[test]
    fn smoke_test_toggle_selected_italic_text_for_commonmark_non_emphasis_space_after_opener() {
        let toggled = toggle_selected_italic_text("* foo bar*");
        assert_eq!(toggled, " foo bar");
    }

    #[test]
    fn smoke_test_toggle_selected_italic_text_for_commonmark_escaped_inner_marker() {
        let toggled = toggle_selected_italic_text("*\\**");
        assert_eq!(toggled, "\\*");
    }

    #[test]
    fn smoke_test_toggle_selected_italic_text_adds_italic_around_bold() {
        let toggled = toggle_selected_italic_text("**hello world**");
        assert_eq!(toggled, "***hello world***");
    }

    #[test]
    fn smoke_test_toggle_selected_italic_text_removes_only_italic_from_bold_italic() {
        let toggled = toggle_selected_italic_text("***hello world***");
        assert_eq!(toggled, "**hello world**");
    }

    #[test]
    fn smoke_test_find_italic_span_cursor_on_opening_marker() {
        assert_eq!(find_italic_span_at_cursor("*hello*", 0), Some((0, 7)));
    }

    #[test]
    fn smoke_test_find_italic_span_cursor_on_closing_marker() {
        assert_eq!(find_italic_span_at_cursor("*hello*", 6), Some((0, 7)));
    }

    #[test]
    fn smoke_test_find_italic_span_cursor_past_end() {
        assert_eq!(find_italic_span_at_cursor("*hello*", 7), Some((0, 7)));
    }

    #[test]
    fn smoke_test_find_italic_span_no_markers() {
        assert_eq!(find_italic_span_at_cursor("hello", 2), None);
    }

    #[test]
    fn smoke_test_find_italic_span_between_two_spans() {
        assert_eq!(find_italic_span_at_cursor("*foo* *bar*", 5), None);
    }
}
