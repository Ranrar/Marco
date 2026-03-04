//! Toolbar inline-emoji insertion helpers for Markdown.
//! Uses a compact popover similar to link insertion and reuses emoji shortcode completion.

use gtk4::prelude::*;
use std::sync::Arc;

const EMOJI_POPOVER_WIDTH: i32 = 280;
const EMOJI_POPOVER_HORIZONTAL_SAFE_PADDING: i32 = 8;

pub fn connect_emoji_toolbar_action(
    toolbar: &gtk4::Box,
    editor_buffer: &sourceview5::Buffer,
    editor_view: &sourceview5::View,
    parent_window: &gtk4::Window,
    settings_manager: Arc<core::logic::swanson::SettingsManager>,
    root_popover_state: crate::ui::popover_state::RootPopoverState,
) {
    if let Some(button) =
        find_button_by_css_class(toolbar.upcast_ref::<gtk4::Widget>(), "toolbar-btn-emoji")
    {
        let editor_buffer = editor_buffer.clone();
        let editor_view = editor_view.clone();
        let parent_window = parent_window.clone();
        let settings_manager = settings_manager.clone();
        let root_popover_state = root_popover_state.clone();

        button.connect_clicked(move |_| {
            if root_popover_state.is_root_open() {
                return;
            }
            show_insert_emoji_popover(
                editor_buffer.upcast_ref::<gtk4::TextBuffer>(),
                editor_view.upcast_ref::<gtk4::TextView>(),
                &parent_window,
                settings_manager.clone(),
            );
        });
    }
}

pub fn show_insert_emoji_popover(
    text_buffer: &gtk4::TextBuffer,
    editor_view: &gtk4::TextView,
    parent_window: &gtk4::Window,
    settings_manager: Arc<core::logic::swanson::SettingsManager>,
) {
    let theme_class = if parent_window.has_css_class("marco-theme-dark") {
        "marco-theme-dark"
    } else {
        "marco-theme-light"
    };

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
    root.set_width_request(EMOJI_POPOVER_WIDTH);

    let title = gtk4::Label::new(Some("Emoji"));
    title.set_halign(gtk4::Align::Start);
    title.add_css_class("marco-dialog-section-label");

    let top_usage = settings_manager.get_settings().get_top_emoji_usage(10);
    let history_grid = gtk4::Grid::new();
    history_grid.set_halign(gtk4::Align::Start);
    history_grid.set_row_spacing(4);
    history_grid.set_column_spacing(4);

    let emoji_entry = gtk4::Entry::new();
    emoji_entry.set_hexpand(true);
    emoji_entry.set_placeholder_text(Some("Emoji shortcode (e.g. smile)"));
    emoji_entry.add_css_class("marco-search-entry");
    emoji_entry.add_css_class("marco-textfield-entry");
    emoji_entry.add_css_class(theme_class);
    attach_emoji_completion(&emoji_entry);

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
    if !top_usage.is_empty() {
        root.append(&history_grid);
    }
    root.append(&emoji_entry);
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
        let emoji_entry = emoji_entry.clone();
        let settings_manager = settings_manager.clone();
        ok_button.connect_clicked(move |_| {
            submit_emoji_from_popover_entry(
                &text_buffer,
                &editor_view,
                &popover,
                &emoji_entry,
                &settings_manager,
            );
        });
    }

    {
        let text_buffer = text_buffer.clone();
        let editor_view = editor_view.clone();
        let popover = popover.clone();
        let signal_emoji_entry = emoji_entry.clone();
        let emoji_entry = emoji_entry.clone();
        let settings_manager = settings_manager.clone();

        signal_emoji_entry.connect_activate(move |_| {
            submit_emoji_from_popover_entry(
                &text_buffer,
                &editor_view,
                &popover,
                &emoji_entry,
                &settings_manager,
            );
        });
    }

    if !top_usage.is_empty() {
        for (idx, item) in top_usage.into_iter().take(10).enumerate() {
            let display_emoji = display_emoji_for_history_value(&item.value);
            let quick_button = gtk4::Button::with_label(&display_emoji);
            quick_button.add_css_class("toolbar-functions-popover-btn");
            quick_button.set_tooltip_text(Some("Recently used"));
            quick_button.set_width_request(26);

            let text_buffer = text_buffer.clone();
            let editor_view = editor_view.clone();
            let popover = popover.clone();
            let settings_manager = settings_manager.clone();
            let emoji_value = item.value.clone();

            quick_button.connect_clicked(move |_| {
                insert_emoji_markdown(&text_buffer, &emoji_value);
                persist_emoji_usage(&settings_manager, &emoji_value);
                popover.popdown();
                editor_view.grab_focus();
            });

            let col = (idx % 5) as i32;
            let row = (idx / 5) as i32;
            history_grid.attach(&quick_button, col, row, 1, 1);
        }
    }

    {
        let signal_emoji_entry = emoji_entry.clone();
        let emoji_entry = emoji_entry.clone();
        let ok_button = ok_button.clone();
        signal_emoji_entry.connect_changed(move |_| {
            update_emoji_ok_button_sensitivity(&emoji_entry, &ok_button);
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
            EMOJI_POPOVER_WIDTH,
            EMOJI_POPOVER_HORIZONTAL_SAFE_PADDING,
        );
        popover.set_offset(x_offset, 0);
    }

    update_emoji_ok_button_sensitivity(&emoji_entry, &ok_button);

    popover.popup();
    emoji_entry.grab_focus();
}

fn submit_emoji_from_popover_entry(
    text_buffer: &gtk4::TextBuffer,
    editor_view: &gtk4::TextView,
    popover: &gtk4::Popover,
    emoji_entry: &gtk4::Entry,
    settings_manager: &core::logic::swanson::SettingsManager,
) {
    let raw = emoji_entry.text().to_string();
    let Some(emoji_value) = normalize_emoji_input(&raw) else {
        emoji_entry.grab_focus();
        return;
    };

    insert_emoji_markdown(text_buffer, &emoji_value);
    persist_emoji_usage(settings_manager, &emoji_value);

    popover.popdown();
    editor_view.grab_focus();
}

fn persist_emoji_usage(
    settings_manager: &core::logic::swanson::SettingsManager,
    emoji_value: &str,
) {
    if let Err(e) = settings_manager.update_settings(|settings| {
        settings.record_emoji_usage(emoji_value, 10);
    }) {
        log::warn!("Failed to persist emoji usage history: {}", e);
    }
}

fn display_emoji_for_history_value(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with(':') && trimmed.ends_with(':') && trimmed.len() >= 3 {
        if let Some(item) = core::logic::emoji_completion_items()
            .iter()
            .find(|item| item.shortcode == trimmed)
        {
            return item.emoji.clone();
        }
    }

    trimmed.to_string()
}

fn update_emoji_ok_button_sensitivity(emoji_entry: &gtk4::Entry, ok_button: &gtk4::Button) {
    ok_button.set_sensitive(normalize_emoji_input(&emoji_entry.text()).is_some());
}

fn insert_emoji_markdown(text_buffer: &gtk4::TextBuffer, emoji_value: &str) {
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

    let insertion = format!("{prefix_space}{emoji_value}{suffix_space}");

    text_buffer.begin_user_action();
    text_buffer.delete(&mut start_iter, &mut end_iter);
    text_buffer.insert(&mut start_iter, &insertion);
    text_buffer.end_user_action();

    let cursor_offset =
        start_offset + prefix_space.chars().count() as i32 + emoji_value.chars().count() as i32;
    let cursor_iter = text_buffer.iter_at_offset(cursor_offset);
    text_buffer.place_cursor(&cursor_iter);
}

fn normalize_emoji_input(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(shortcode) = extract_embedded_shortcode(trimmed) {
        return Some(shortcode);
    }

    if let Some(shortcode) = extract_trailing_shortcode_name(trimmed) {
        return Some(shortcode);
    }

    let normalized = core::logic::normalize_completion_query(trimmed);
    if !normalized.is_empty()
        && normalized
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '+'))
    {
        return Some(format!(":{}:", normalized));
    }

    Some(trimmed.to_string())
}

fn extract_trailing_shortcode_name(raw: &str) -> Option<String> {
    let token = raw.split_whitespace().last()?;
    let normalized = core::logic::normalize_completion_query(token);

    if normalized.is_empty()
        || !normalized
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '+'))
    {
        return None;
    }

    let candidate = format!(":{}:", normalized);
    if core::logic::emoji_shortcodes_for_completion().contains(&candidate) {
        return Some(candidate);
    }

    None
}

fn extract_embedded_shortcode(raw: &str) -> Option<String> {
    let is_valid_shortcode_char =
        |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '+');

    for (start_idx, _) in raw.match_indices(':') {
        let rest = &raw[start_idx + 1..];
        for (end_rel_idx, _) in rest.match_indices(':') {
            if end_rel_idx == 0 {
                continue;
            }

            let candidate = &rest[..end_rel_idx];
            if candidate.chars().all(is_valid_shortcode_char) {
                return Some(format!(":{}:", candidate.to_ascii_lowercase()));
            }
        }
    }

    None
}

#[allow(deprecated)]
fn attach_emoji_completion(entry: &gtk4::Entry) {
    let completion = gtk4::EntryCompletion::new();
    completion.set_inline_completion(true);
    completion.set_inline_selection(true);
    completion.set_popup_completion(true);
    completion.set_popup_single_match(false);
    completion.set_minimum_key_length(1);

    let model = gtk4::ListStore::new(&[String::static_type(), String::static_type()]);
    for item in core::logic::emoji_completion_items() {
        let iter = model.append();
        model.set(&iter, &[(0, &item.display), (1, &item.shortcode)]);
    }

    completion.set_model(Some(&model));
    completion.set_text_column(0);

    completion.set_match_func(|completion, key, iter| {
        let Some(model) = completion.model() else {
            return false;
        };
        let candidate: String = model.get(iter, 1);
        core::logic::emoji_shortcode_matches_query(&candidate, key)
    });

    entry.set_completion(Some(&completion));
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
    fn smoke_test_normalize_emoji_input() {
        assert_eq!(normalize_emoji_input("smile"), Some(":smile:".to_string()));
        assert_eq!(
            normalize_emoji_input(":smile:"),
            Some(":smile:".to_string())
        );
        assert_eq!(
            normalize_emoji_input("😄 :smile:"),
            Some(":smile:".to_string())
        );
        assert_eq!(
            normalize_emoji_input("😄 smile"),
            Some(":smile:".to_string())
        );
        assert_eq!(normalize_emoji_input("😀"), Some("😀".to_string()));
    }

    #[test]
    fn smoke_test_space_rules_for_inline_emoji() {
        assert!(should_insert_space_before(Some('a')));
        assert!(should_insert_space_after(Some('9')));
        assert!(!should_insert_space_before(Some(' ')));
        assert!(!should_insert_space_after(None));
    }

    #[test]
    fn smoke_test_display_emoji_for_history_shortcode() {
        let rendered = display_emoji_for_history_value(":smile:");
        assert_eq!(rendered, "😄");
    }
}
