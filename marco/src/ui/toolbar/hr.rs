//! Toolbar horizontal-rule helpers for Markdown thematic breaks (`---`).

use gtk4::prelude::*;

pub fn connect_hr_toolbar_action(
    toolbar: &gtk4::Box,
    editor_buffer: &sourceview5::Buffer,
    editor_view: &sourceview5::View,
    root_popover_state: crate::ui::popover_state::RootPopoverState,
) {
    if let Some(button) =
        find_button_by_css_class(toolbar.upcast_ref::<gtk4::Widget>(), "toolbar-btn-hr")
    {
        let editor_buffer = editor_buffer.clone();
        let editor_view = editor_view.clone();
        let root_popover_state = root_popover_state.clone();
        button.connect_clicked(move |_| {
            if root_popover_state.is_root_open() {
                return;
            }
            insert_hr_at_cursor_and_refocus(
                editor_buffer.upcast_ref::<gtk4::TextBuffer>(),
                &editor_view,
            );
        });
    }
}

pub fn insert_hr_at_cursor_and_refocus(
    text_buffer: &gtk4::TextBuffer,
    editor_view: &sourceview5::View,
) {
    let cursor_offset = text_buffer.cursor_position();
    let mut insert_iter = text_buffer.iter_at_offset(cursor_offset);

    let char_before = char_before_iter(text_buffer, &insert_iter);
    let char_after = char_after_iter(text_buffer, &insert_iter);
    let hr_text = thematic_break_insert_text(char_before, char_after);

    text_buffer.begin_user_action();
    text_buffer.insert(&mut insert_iter, &hr_text);
    text_buffer.place_cursor(&insert_iter);
    text_buffer.end_user_action();

    let mut scroll_iter = insert_iter;
    editor_view.scroll_to_iter(&mut scroll_iter, 0.15, true, 0.0, 0.35);
    editor_view.grab_focus();
}

fn thematic_break_insert_text(char_before: Option<char>, char_after: Option<char>) -> String {
    let mut out = String::new();

    if let Some(ch) = char_before {
        if ch != '\n' {
            out.push('\n');
        }
    }

    out.push_str("---");

    if char_after != Some('\n') {
        out.push('\n');
    }

    out
}

fn char_before_iter(text_buffer: &gtk4::TextBuffer, iter: &gtk4::TextIter) -> Option<char> {
    let mut start = *iter;
    if !start.backward_char() {
        return None;
    }

    let text = text_buffer.text(&start, iter, false);
    text.chars().next()
}

fn char_after_iter(text_buffer: &gtk4::TextBuffer, iter: &gtk4::TextIter) -> Option<char> {
    let start = *iter;
    let mut end = *iter;
    if !end.forward_char() {
        return None;
    }

    let text = text_buffer.text(&start, &end, false);
    text.chars().next()
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
    fn smoke_test_insert_text_middle_of_line_gets_surrounding_newlines() {
        let text = thematic_break_insert_text(Some('a'), Some('b'));
        assert_eq!(text, "\n---\n");
    }

    #[test]
    fn smoke_test_insert_text_at_line_start_no_prefix_newline() {
        let text = thematic_break_insert_text(Some('\n'), Some('x'));
        assert_eq!(text, "---\n");
    }

    #[test]
    fn smoke_test_insert_text_before_line_break_no_extra_suffix() {
        let text = thematic_break_insert_text(Some('x'), Some('\n'));
        assert_eq!(text, "\n---");
    }
}
