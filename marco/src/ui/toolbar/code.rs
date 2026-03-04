//! Toolbar inline-code toggle helpers for Markdown (`` `...` ``).
//! Wraps or unwraps the selection / word-at-cursor with single backticks.

use core::render::code_languages::KNOWN_CODE_LANGUAGES;
use gtk4::prelude::*;

const CODE_BLOCK_POPOVER_WIDTH: i32 = 320;
const CODE_BLOCK_POPOVER_HORIZONTAL_SAFE_PADDING: i32 = 8;

pub fn connect_code_toolbar_action(
    toolbar: &gtk4::Box,
    editor_buffer: &sourceview5::Buffer,
    editor_view: &sourceview5::View,
    root_popover_state: crate::ui::popover_state::RootPopoverState,
) {
    if let Some(button) =
        find_button_by_css_class(toolbar.upcast_ref::<gtk4::Widget>(), "toolbar-btn-code")
    {
        let editor_buffer = editor_buffer.clone();
        let editor_view = editor_view.clone();
        let root_popover_state = root_popover_state.clone();
        button.connect_clicked(move |_| {
            if root_popover_state.is_root_open() {
                return;
            }
            toggle_code_for_selection_or_word(editor_buffer.upcast_ref::<gtk4::TextBuffer>());
            refocus_current_line(&editor_buffer, &editor_view);
        });
    }

    if let Some(button) = find_button_by_css_class(
        toolbar.upcast_ref::<gtk4::Widget>(),
        "toolbar-btn-fenced-code-block",
    ) {
        let editor_buffer = editor_buffer.clone();
        let editor_view = editor_view.clone();
        let root_popover_state = root_popover_state.clone();

        button.connect_clicked(move |_| {
            if root_popover_state.is_root_open() {
                return;
            }
            show_insert_code_block_popover(
                editor_buffer.upcast_ref::<gtk4::TextBuffer>(),
                editor_view.upcast_ref::<gtk4::TextView>(),
            );
        });
    }
}

fn show_insert_code_block_popover(text_buffer: &gtk4::TextBuffer, editor_view: &gtk4::TextView) {
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
    root.set_width_request(CODE_BLOCK_POPOVER_WIDTH);

    let title = gtk4::Label::new(Some("Code Block"));
    title.set_halign(gtk4::Align::Start);
    title.add_css_class("marco-dialog-section-label");

    let language_entry = gtk4::Entry::new();
    language_entry.set_hexpand(true);
    language_entry.set_placeholder_text(Some("Language (optional, e.g. rust, js, python)"));
    language_entry.add_css_class("marco-search-entry");
    attach_code_language_completion(&language_entry);

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
    root.append(&language_entry);
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
        let language_entry = language_entry.clone();
        ok_button.connect_clicked(move |_| {
            submit_code_block_from_popover_entry(
                &text_buffer,
                &editor_view,
                &popover,
                &language_entry,
            );
        });
    }

    {
        let text_buffer = text_buffer.clone();
        let editor_view = editor_view.clone();
        let popover = popover.clone();
        let language_entry_for_signal = language_entry.clone();
        let language_entry = language_entry.clone();
        language_entry_for_signal.connect_activate(move |_| {
            submit_code_block_from_popover_entry(
                &text_buffer,
                &editor_view,
                &popover,
                &language_entry,
            );
        });
    }

    let caret_rect = code_cursor_rect(text_buffer, editor_view);
    let clamped_rect = code_clamp_rect_to_editor(caret_rect, editor_view);
    popover.set_pointing_to(Some(&clamped_rect));

    if let Some(text_area) = code_visible_text_area_widget_rect(editor_view) {
        let x_offset = code_compute_popover_x_offset_for_text_area(
            clamped_rect.x(),
            text_area.x(),
            text_area.x() + text_area.width(),
            CODE_BLOCK_POPOVER_WIDTH,
            CODE_BLOCK_POPOVER_HORIZONTAL_SAFE_PADDING,
        );
        popover.set_offset(x_offset, 0);
    }

    popover.popup();
    language_entry.grab_focus();
}

fn submit_code_block_from_popover_entry(
    text_buffer: &gtk4::TextBuffer,
    editor_view: &gtk4::TextView,
    popover: &gtk4::Popover,
    language_entry: &gtk4::Entry,
) {
    let language = normalize_code_block_language_input(&language_entry.text());
    insert_fenced_code_block_markdown(text_buffer, language.as_deref());

    popover.popdown();
    editor_view.grab_focus();
}

#[allow(deprecated)]
fn attach_code_language_completion(entry: &gtk4::Entry) {
    let completion = gtk4::EntryCompletion::new();
    completion.set_inline_completion(false);
    completion.set_inline_selection(false);
    completion.set_popup_completion(true);
    completion.set_popup_single_match(false);
    completion.set_minimum_key_length(1);

    let model = gtk4::ListStore::new(&[
        String::static_type(),
        String::static_type(),
        String::static_type(),
    ]);
    for language in KNOWN_CODE_LANGUAGES {
        let aliases = language.aliases.join(", ");
        let display = format!("{} ({})", language.canonical, aliases);
        let iter = model.append();
        model.set(
            &iter,
            &[(0, &display), (1, &language.canonical), (2, &aliases)],
        );
    }

    completion.set_model(Some(&model));
    completion.set_text_column(0);

    completion.set_match_func(|completion, key, iter| {
        let Some(model) = completion.model() else {
            return false;
        };

        let canonical: String = model.get(iter, 1);
        let aliases: String = model.get(iter, 2);
        code_language_matches_query(&canonical, &aliases, key)
    });

    entry.set_completion(Some(&completion));
}

fn code_language_matches_query(canonical: &str, aliases_csv: &str, query: &str) -> bool {
    let query = normalize_language_query(query);
    if query.is_empty() {
        return true;
    }

    let canonical_norm = normalize_language_query(canonical);
    if canonical_norm.contains(&query)
        || canonical_norm.starts_with(&query)
        || is_subsequence(&query, &canonical_norm)
    {
        return true;
    }

    aliases_csv
        .split(',')
        .map(normalize_language_query)
        .any(|alias| {
            alias.contains(&query) || alias.starts_with(&query) || is_subsequence(&query, &alias)
        })
}

fn normalize_language_query(raw: &str) -> String {
    raw.trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '#'))
        .flat_map(|c| c.to_lowercase())
        .collect()
}

fn is_subsequence(needle: &str, haystack: &str) -> bool {
    if needle.is_empty() {
        return true;
    }

    let mut needle_chars = needle.chars();
    let mut current = needle_chars.next();

    for ch in haystack.chars() {
        if current.is_some_and(|wanted| wanted == ch) {
            current = needle_chars.next();
            if current.is_none() {
                return true;
            }
        }
    }

    false
}

fn normalize_code_block_language_input(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    for language in KNOWN_CODE_LANGUAGES {
        if raw.eq_ignore_ascii_case(language.canonical)
            || language
                .aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(raw))
        {
            return language.aliases.first().map(|alias| (*alias).to_string());
        }
    }

    Some(raw.to_string())
}

fn insert_fenced_code_block_markdown(text_buffer: &gtk4::TextBuffer, language: Option<&str>) {
    let cursor = text_buffer.iter_at_offset(text_buffer.cursor_position());
    let at_line_start = cursor.starts_line();
    let prefix = if at_line_start { "" } else { "\n" };
    let fence_open = match language {
        Some(language) if !language.trim().is_empty() => format!("```{}", language.trim()),
        _ => "```".to_string(),
    };

    let snippet = format!("{prefix}{fence_open}\n\n```\n");
    let start_offset = text_buffer.cursor_position();

    text_buffer.begin_user_action();
    let mut insert_pos = text_buffer.iter_at_offset(start_offset);
    text_buffer.insert(&mut insert_pos, &snippet);

    let cursor_offset =
        start_offset + prefix.chars().count() as i32 + fence_open.chars().count() as i32 + 1;
    let cursor_iter = text_buffer.iter_at_offset(cursor_offset);
    text_buffer.place_cursor(&cursor_iter);
    text_buffer.end_user_action();
}

fn code_cursor_rect(
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

fn code_clamp_rect_to_editor(
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

fn code_visible_text_area_widget_rect(
    editor_view: &gtk4::TextView,
) -> Option<gtk4::gdk::Rectangle> {
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

fn code_compute_popover_x_offset_for_text_area(
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

fn refocus_current_line(editor_buffer: &sourceview5::Buffer, editor_view: &sourceview5::View) {
    let mut iter = editor_buffer.iter_at_offset(editor_buffer.cursor_position());
    editor_buffer.place_cursor(&iter);
    editor_view.scroll_to_iter(&mut iter, 0.15, true, 0.0, 0.35);
    editor_view.grab_focus();
}

pub fn toggle_code_for_selection_or_word(text_buffer: &gtk4::TextBuffer) {
    if let Some((mut selection_start, mut selection_end)) = text_buffer.selection_bounds() {
        if selection_start.offset() != selection_end.offset() {
            maybe_expand_range_to_surrounding_code_markers(
                text_buffer,
                &mut selection_start,
                &mut selection_end,
            );
            toggle_code_on_range(text_buffer, &mut selection_start, &mut selection_end);
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
        // Cursor might be on/adjacent to existing code markers — try to unwrap.
        if let Some((span_start, span_end)) = find_code_span_at_cursor(&line_text, cursor_col) {
            if let (Some(mut si), Some(mut ei)) = (
                iter_at_line_col(text_buffer, line, span_start as i32),
                iter_at_line_col(text_buffer, line, span_end as i32),
            ) {
                toggle_code_on_range(text_buffer, &mut si, &mut ei);
                return;
            }
        }
        insert_empty_code_delimiters(text_buffer);
        return;
    };

    let Some(mut start_iter) = iter_at_line_col(text_buffer, line, word_start as i32) else {
        return;
    };
    let Some(mut end_iter) = iter_at_line_col(text_buffer, line, word_end as i32) else {
        return;
    };

    maybe_expand_range_to_surrounding_code_markers(text_buffer, &mut start_iter, &mut end_iter);

    toggle_code_on_range(text_buffer, &mut start_iter, &mut end_iter);
}

/// Insert ``` `` ``` at the current cursor position and place the cursor between
/// the two backticks for immediate typing.
fn insert_empty_code_delimiters(text_buffer: &gtk4::TextBuffer) {
    let cursor_pos = text_buffer.cursor_position();
    let mut iter = text_buffer.iter_at_offset(cursor_pos);

    text_buffer.begin_user_action();
    text_buffer.insert(&mut iter, "``");
    text_buffer.end_user_action();

    // Place cursor between the two `` ` `` markers → offset + 1.
    let mid = text_buffer.iter_at_offset(cursor_pos + 1);
    text_buffer.place_cursor(&mid);
}

fn maybe_expand_range_to_surrounding_code_markers(
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
        expand_to_surrounding_code_markers(&line_text, start_col, end_col)
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

/// Returns `true` when `chars[idx]` is a single backtick that is NOT part of
/// a fenced code-block opener/closer (`` ``` ``).
fn is_code_delimiter(chars: &[char], idx: usize) -> bool {
    if chars.get(idx) != Some(&'`') {
        return false;
    }

    let prev_is_backtick = idx > 0 && chars[idx - 1] == '`';
    let next_is_backtick = idx + 1 < chars.len() && chars[idx + 1] == '`';

    !prev_is_backtick && !next_is_backtick
}

fn expand_to_surrounding_code_markers(
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
        let has_left_marker = is_code_delimiter(&chars, start_col - 1);
        let has_right_marker = is_code_delimiter(&chars, end_col);

        if has_left_marker && has_right_marker {
            return Some((start_col - 1, end_col + 1));
        }
    }

    // Fallback: selection might be inside a larger code span.
    let marker_positions: Vec<usize> = (0..chars.len())
        .filter(|&i| is_code_delimiter(&chars, i))
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

/// Find a complete `` `…` `` span that contains the cursor position.
fn find_code_span_at_cursor(line_text: &str, cursor_col: usize) -> Option<(usize, usize)> {
    let chars: Vec<char> = line_text.chars().collect();
    if chars.is_empty() {
        return None;
    }

    let cursor = cursor_col.min(chars.len() - 1);
    let marker_positions: Vec<usize> = (0..chars.len())
        .filter(|&i| is_code_delimiter(&chars, i))
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

fn toggle_code_on_range(
    text_buffer: &gtk4::TextBuffer,
    start_iter: &mut gtk4::TextIter,
    end_iter: &mut gtk4::TextIter,
) {
    let selected = text_buffer.text(start_iter, end_iter, false).to_string();
    let toggled = toggle_selected_code_text(&selected);

    text_buffer.begin_user_action();
    text_buffer.delete(start_iter, end_iter);
    text_buffer.insert(start_iter, &toggled);
    text_buffer.end_user_action();
}

fn is_wrapped_with_code_markers(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 3 {
        return false;
    }

    if chars.first() != Some(&'`') || chars.last() != Some(&'`') {
        return false;
    }

    // Exclude fenced code-block opener (```)
    if chars.get(1) == Some(&'`') {
        return false;
    }

    // Exclude fenced code-block closer (```), except when escaped
    if chars.len() >= 2 && chars[chars.len() - 2] == '`' {
        let penultimate_escaped = chars.len() >= 3 && chars[chars.len() - 3] == '\\';
        if !penultimate_escaped {
            return false;
        }
    }

    true
}

fn is_wrapped_with_single_code_span(text: &str) -> bool {
    if !is_wrapped_with_code_markers(text) {
        return false;
    }

    let inner = &text[1..text.len() - 1];
    !inner.is_empty() && !inner.contains('`')
}

fn toggle_selected_code_text(selected: &str) -> String {
    if let Some(stripped_multi) = strip_code_from_each_non_whitespace_token(selected) {
        stripped_multi
    } else if is_wrapped_with_code_markers(selected) {
        selected[1..selected.len() - 1].to_string()
    } else {
        format!("`{}`", selected)
    }
}

fn strip_code_from_each_non_whitespace_token(text: &str) -> Option<String> {
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
        if !is_wrapped_with_single_code_span(token) {
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
    fn smoke_test_detects_code_wrapped_text() {
        assert!(is_wrapped_with_code_markers("`hello`"));
        assert!(!is_wrapped_with_code_markers("hello"));
    }

    #[test]
    fn smoke_test_rejects_fenced_code_block_opener() {
        assert!(!is_wrapped_with_code_markers("```rust```"));
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
    fn smoke_test_expand_to_surrounding_code_markers_detects_wrapped_word() {
        let expanded = expand_to_surrounding_code_markers("`world`", 1, 6);
        assert_eq!(expanded, Some((0, 7)));
    }

    #[test]
    fn smoke_test_expand_to_surrounding_code_markers_none_for_plain_word() {
        let expanded = expand_to_surrounding_code_markers("world", 0, 5);
        assert_eq!(expanded, None);
    }

    #[test]
    fn smoke_test_expand_to_surrounding_code_markers_for_inner_selection() {
        let expanded = expand_to_surrounding_code_markers("`hello world`", 1, 6);
        assert_eq!(expanded, Some((0, 13)));
    }

    #[test]
    fn smoke_test_toggle_selected_code_text_for_multiple_code_words() {
        let toggled = toggle_selected_code_text("`hello` `world`");
        assert_eq!(toggled, "hello world");
    }

    #[test]
    fn smoke_test_toggle_selected_code_text_for_single_span() {
        let toggled = toggle_selected_code_text("`hello world`");
        assert_eq!(toggled, "hello world");
    }

    #[test]
    fn smoke_test_toggle_selected_code_text_wraps_plain_text() {
        let toggled = toggle_selected_code_text("hello");
        assert_eq!(toggled, "`hello`");
    }

    #[test]
    fn smoke_test_toggle_selected_code_text_adds_around_other_formatting() {
        let toggled = toggle_selected_code_text("hello **world**");
        assert_eq!(toggled, "`hello **world**`");
    }

    // ── Edge-case tests ──────────────────────────────────────────────

    #[test]
    fn edge_case_single_char_wraps_to_code() {
        let toggled = toggle_selected_code_text("a");
        assert_eq!(toggled, "`a`");
    }

    #[test]
    fn edge_case_single_char_unwraps_from_code() {
        let toggled = toggle_selected_code_text("`a`");
        assert_eq!(toggled, "a");
    }

    #[test]
    fn edge_case_double_toggle_is_idempotent() {
        let original = "hello";
        let once = toggle_selected_code_text(original);
        assert_eq!(once, "`hello`");
        let twice = toggle_selected_code_text(&once);
        assert_eq!(twice, original);
    }

    #[test]
    fn edge_case_unicode_content_wraps_correctly() {
        let toggled = toggle_selected_code_text("café");
        assert_eq!(toggled, "`café`");
    }

    #[test]
    fn edge_case_unicode_content_unwraps_correctly() {
        let toggled = toggle_selected_code_text("`café`");
        assert_eq!(toggled, "café");
    }

    #[test]
    fn edge_case_whitespace_only_wraps() {
        // Whitespace-only text gets wrapped (consistent with bold/italic).
        let toggled = toggle_selected_code_text("   ");
        assert_eq!(toggled, "`   `");
    }

    #[test]
    fn edge_case_double_backtick_not_detected_as_code() {
        assert!(!is_wrapped_with_code_markers("``"));
    }

    #[test]
    fn edge_case_triple_backtick_rejected_as_code() {
        assert!(!is_wrapped_with_code_markers("```"));
    }

    #[test]
    fn edge_case_fenced_code_block_with_lang_rejected() {
        assert!(!is_wrapped_with_code_markers("```rust```"));
    }

    #[test]
    fn edge_case_delimiter_at_start_of_line() {
        let chars: Vec<char> = "`hello`".chars().collect();
        assert!(is_code_delimiter(&chars, 0));
    }

    #[test]
    fn edge_case_delimiter_at_end_of_line() {
        let chars: Vec<char> = "`hello`".chars().collect();
        assert!(is_code_delimiter(&chars, 6));
    }

    #[test]
    fn edge_case_delimiter_rejects_adjacent_backticks() {
        let chars: Vec<char> = "``hello``".chars().collect();
        assert!(!is_code_delimiter(&chars, 0));
        assert!(!is_code_delimiter(&chars, 1));
        assert!(!is_code_delimiter(&chars, 7));
        assert!(!is_code_delimiter(&chars, 8));
    }

    #[test]
    fn edge_case_code_with_inner_backslash_backtick() {
        // `a\`` — code span containing backslash-backtick.
        assert!(is_wrapped_with_code_markers("`a\\``"));
    }

    #[test]
    fn edge_case_nested_bold_inside_code_unwraps() {
        let toggled = toggle_selected_code_text("`**bold**`");
        assert_eq!(toggled, "**bold**");
    }

    #[test]
    fn edge_case_expand_no_markers_returns_none() {
        let expanded = expand_to_surrounding_code_markers("plain text", 0, 5);
        assert_eq!(expanded, None);
    }

    #[test]
    fn edge_case_expand_empty_string() {
        let expanded = expand_to_surrounding_code_markers("", 0, 0);
        assert_eq!(expanded, None);
    }

    #[test]
    fn edge_case_expand_start_equals_end() {
        // Zero-width range inside a code span.
        let expanded = expand_to_surrounding_code_markers("`hello`", 3, 3);
        assert_eq!(expanded, Some((0, 7)));
    }

    #[test]
    fn smoke_test_find_code_span_cursor_on_opening_marker() {
        assert_eq!(find_code_span_at_cursor("`hello`", 0), Some((0, 7)));
    }

    #[test]
    fn smoke_test_find_code_span_cursor_on_closing_marker() {
        assert_eq!(find_code_span_at_cursor("`hello`", 6), Some((0, 7)));
    }

    #[test]
    fn smoke_test_find_code_span_cursor_past_end() {
        assert_eq!(find_code_span_at_cursor("`hello`", 7), Some((0, 7)));
    }

    #[test]
    fn smoke_test_find_code_span_no_markers() {
        assert_eq!(find_code_span_at_cursor("hello", 2), None);
    }

    #[test]
    fn smoke_test_find_code_span_between_two_spans() {
        assert_eq!(find_code_span_at_cursor("`foo` `bar`", 5), None);
    }

    #[test]
    fn smoke_test_code_language_matches_query() {
        assert!(code_language_matches_query("Rust", "rs, rust", "rs"));
        assert!(code_language_matches_query(
            "JavaScript",
            "js, javascript, node",
            "script"
        ));
        assert!(code_language_matches_query("PHP", "php", "ph"));
        assert!(code_language_matches_query(
            "TypeScript",
            "ts, typescript",
            "tps"
        ));
        assert!(!code_language_matches_query("Rust", "rs, rust", "python"));
    }

    #[test]
    fn smoke_test_normalize_language_query() {
        assert_eq!(normalize_language_query(" C++ "), "c++");
        assert_eq!(normalize_language_query("C#"), "c#");
        assert_eq!(normalize_language_query("Type Script"), "typescript");
    }

    #[test]
    fn smoke_test_normalize_code_block_language_input_known_alias() {
        assert_eq!(
            normalize_code_block_language_input("Rust"),
            Some("rs".to_string())
        );
        assert_eq!(
            normalize_code_block_language_input("python3"),
            Some("py".to_string())
        );
    }

    #[test]
    fn smoke_test_normalize_code_block_language_input_unknown_kept() {
        assert_eq!(
            normalize_code_block_language_input("mydsl"),
            Some("mydsl".to_string())
        );
        assert_eq!(normalize_code_block_language_input("   "), None);
    }
}
