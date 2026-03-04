//! Toolbar block-format helpers for Markdown paragraph/quote/ATX heading actions.
//! Behavior is validated against representative CommonMark block examples.

use gtk4::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimpleBlockFormat {
    Paragraph,
    Quote,
    Heading(u8),
}

pub fn connect_simple_markdown_toolbar_actions(
    toolbar: &gtk4::Box,
    editor_buffer: &sourceview5::Buffer,
    editor_view: &sourceview5::View,
    root_popover_state: crate::ui::popover_state::RootPopoverState,
) {
    let mappings = [
        ("toolbar-blocktype-paragraph", SimpleBlockFormat::Paragraph),
        ("toolbar-blocktype-quote", SimpleBlockFormat::Quote),
        ("toolbar-blocktype-h1", SimpleBlockFormat::Heading(1)),
        ("toolbar-blocktype-h2", SimpleBlockFormat::Heading(2)),
        ("toolbar-blocktype-h3", SimpleBlockFormat::Heading(3)),
        ("toolbar-blocktype-h4", SimpleBlockFormat::Heading(4)),
        ("toolbar-blocktype-h5", SimpleBlockFormat::Heading(5)),
        ("toolbar-blocktype-h6", SimpleBlockFormat::Heading(6)),
    ];

    for (css_class, format) in mappings {
        if let Some(button) =
            find_button_by_css_class(toolbar.upcast_ref::<gtk4::Widget>(), css_class)
        {
            let editor_buffer = editor_buffer.clone();
            let editor_view = editor_view.clone();
            let root_popover_state = root_popover_state.clone();
            button.connect_clicked(move |_| {
                if root_popover_state.is_root_open() {
                    return;
                }
                apply_block_format_to_selection_or_current_line(
                    editor_buffer.upcast_ref::<gtk4::TextBuffer>(),
                    format,
                );
                refocus_current_line(&editor_buffer, &editor_view);
            });
        }
    }
}

fn refocus_current_line(editor_buffer: &sourceview5::Buffer, editor_view: &sourceview5::View) {
    let mut iter = editor_buffer.iter_at_offset(editor_buffer.cursor_position());
    editor_buffer.place_cursor(&iter);
    editor_view.scroll_to_iter(&mut iter, 0.15, true, 0.0, 0.35);
    editor_view.grab_focus();
}

pub fn apply_block_format_to_selection_or_current_line(
    text_buffer: &gtk4::TextBuffer,
    format: SimpleBlockFormat,
) {
    let (start_line, end_line) = selection_or_cursor_lines(text_buffer);

    text_buffer.begin_user_action();

    for line in (start_line..=end_line).rev() {
        if let Some(mut line_start) = text_buffer.iter_at_line(line) {
            let mut line_end = line_start;
            line_end.forward_to_line_end();

            let current_line = text_buffer.text(&line_start, &line_end, false).to_string();
            let formatted = format_line_for_block(&current_line, format);

            text_buffer.delete(&mut line_start, &mut line_end);
            text_buffer.insert(&mut line_start, &formatted);
        }
    }

    text_buffer.end_user_action();
}

fn selection_or_cursor_lines(text_buffer: &gtk4::TextBuffer) -> (i32, i32) {
    if let Some((selection_start, selection_end)) = text_buffer.selection_bounds() {
        let start_line = selection_start.line().max(0);
        let mut end_line = selection_end.line().max(0);

        // If the selection ends exactly at the start of the next line,
        // keep the transform limited to the actually selected lines.
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

fn format_line_for_block(line: &str, format: SimpleBlockFormat) -> String {
    let (indent, content) = split_indent(line);
    let stripped = strip_existing_block_prefix(content);

    let formatted_content = match format {
        SimpleBlockFormat::Paragraph => stripped.to_string(),
        SimpleBlockFormat::Quote => {
            if stripped.is_empty() {
                ">".to_string()
            } else {
                format!("> {}", stripped)
            }
        }
        SimpleBlockFormat::Heading(level) => {
            let level = level.clamp(1, 6) as usize;
            let marker = "#".repeat(level);
            if stripped.is_empty() {
                format!("{} ", marker)
            } else {
                format!("{} {}", marker, stripped)
            }
        }
    };

    format!("{}{}", indent, formatted_content)
}

fn split_indent(line: &str) -> (&str, &str) {
    let indent_len = line
        .char_indices()
        .find_map(|(idx, ch)| (!matches!(ch, ' ' | '\t')).then_some(idx))
        .unwrap_or(line.len());
    (&line[..indent_len], &line[indent_len..])
}

fn strip_existing_block_prefix(content: &str) -> &str {
    let mut current = content;

    loop {
        let before = current;

        if let Some(stripped_quote) = strip_quote_prefix(current) {
            current = stripped_quote;
        }

        if let Some(stripped_heading) = strip_heading_prefix(current) {
            current = stripped_heading;
        }

        if current == before {
            break;
        }
    }

    current
}

fn strip_quote_prefix(content: &str) -> Option<&str> {
    let tail = content.strip_prefix('>')?;
    Some(tail.strip_prefix(' ').unwrap_or(tail))
}

fn strip_heading_prefix(content: &str) -> Option<&str> {
    let bytes = content.as_bytes();
    let mut idx = 0usize;

    while idx < bytes.len() && bytes[idx] == b'#' {
        idx += 1;
    }

    if idx == 0 || idx > 6 {
        return None;
    }

    let remainder = &content[idx..];
    if remainder.starts_with(' ') || remainder.starts_with('\t') || remainder.is_empty() {
        Some(remainder.trim_start_matches([' ', '\t']))
    } else {
        None
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
    fn smoke_test_heading_replaces_existing_block_prefixes() {
        let result = format_line_for_block("  > ### Title", SimpleBlockFormat::Heading(2));
        assert_eq!(result, "  ## Title");
    }

    #[test]
    fn smoke_test_quote_from_plain_text() {
        let result = format_line_for_block("Hello", SimpleBlockFormat::Quote);
        assert_eq!(result, "> Hello");
    }

    #[test]
    fn smoke_test_paragraph_strips_heading_prefix() {
        let result = format_line_for_block("#### Hello", SimpleBlockFormat::Paragraph);
        assert_eq!(result, "Hello");
    }

    // CommonMark traceability (tests/test_suite/spec/commonmark.json):
    // | Local test                                              | CM example | Snippet                         |
    // |---------------------------------------------------------|------------|---------------------------------|
    // | ..._commonmark_paragraph_from_atx_heading              | 62         | "# foo"                        |
    // | ..._commonmark_paragraph_from_block_quote              | 228        | "> bar"                        |
    // | ..._commonmark_quote_from_empty_line                   | 239        | ">"                            |

    #[test]
    fn smoke_test_paragraph_commonmark_paragraph_from_atx_heading() {
        let result = format_line_for_block("# foo", SimpleBlockFormat::Paragraph);
        assert_eq!(result, "foo");
    }

    #[test]
    fn smoke_test_paragraph_commonmark_paragraph_from_block_quote() {
        let result = format_line_for_block("> bar", SimpleBlockFormat::Paragraph);
        assert_eq!(result, "bar");
    }

    #[test]
    fn smoke_test_paragraph_commonmark_quote_from_empty_line() {
        let result = format_line_for_block("", SimpleBlockFormat::Quote);
        assert_eq!(result, ">");
    }
}
