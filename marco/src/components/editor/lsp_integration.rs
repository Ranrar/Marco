// LSP integration for Marco editor
// Applies syntax highlighting based on parser LSP features

use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Apply LSP highlights to the buffer using line/column positioning
///
/// This function converts parser highlights (line/column coordinates) to GTK TextIter positions
/// and applies appropriate tags for syntax highlighting.
///
/// # Architecture
///
/// The conversion uses a **line-based approach** for robustness:
/// 1. Parser provides line (1-based) and column (1-based byte offset from line start)
/// 2. We convert to GTK coordinates: line (0-based) and char offset (0-based from line start)
/// 3. This avoids cumulative errors from UTF-8 multi-byte characters
///
/// # Arguments
///
/// * `buffer` - The GTK SourceView buffer to apply highlights to
/// * `highlights` - Vec of highlights from the parser's LSP module
#[allow(dead_code)]
pub fn apply_lsp_highlights(buffer: &sourceview5::Buffer, highlights: &[core::lsp::Highlight]) {
    log::debug!("Applying {} LSP highlights", highlights.len());

    // Remove all existing LSP tags first
    remove_all_lsp_tags(buffer);

    // Apply each highlight
    for highlight in highlights {
        if let Some((start_iter, end_iter)) = span_to_text_iter(buffer, &highlight.span) {
            let tag = get_or_create_tag(buffer, &highlight.tag);
            buffer.apply_tag(&tag, &start_iter, &end_iter);
        } else {
            log::warn!(
                "Failed to convert span for highlight {:?} at [{}:{} to {}:{}]",
                highlight.tag,
                highlight.span.start.line,
                highlight.span.start.column,
                highlight.span.end.line,
                highlight.span.end.column
            );
        }
    }

    log::debug!("LSP highlights applied successfully");
}

/// Apply LSP highlights in small chunks scheduled on the main loop.
///
/// Why: For large documents, applying thousands of tags in one go can block the
/// GTK main thread and cause visible stutter. Chunking yields back to the main
/// loop between batches.
///
/// `on_done` is invoked on the main thread once all chunks are applied.
pub fn apply_lsp_highlights_chunked<F>(
    buffer: &sourceview5::Buffer,
    highlights: Vec<core::lsp::Highlight>,
    on_done: F,
) where
    F: FnOnce() + 'static,
{
    const CHUNK_SIZE: usize = 400;

    log::debug!(
        "Applying {} LSP highlights (chunked, chunk_size={})",
        highlights.len(),
        CHUNK_SIZE
    );

    remove_all_lsp_tags(buffer);

    if highlights.is_empty() {
        on_done();
        return;
    }

    let buffer = buffer.clone();
    let highlights = Rc::new(highlights);
    let index = Rc::new(Cell::new(0usize));
    let on_done = Rc::new(RefCell::new(Some(Box::new(on_done) as Box<dyn FnOnce()>)));

    glib::idle_add_local(move || {
        let start = index.get();
        let end = (start + CHUNK_SIZE).min(highlights.len());

        for highlight in &highlights[start..end] {
            if let Some((start_iter, end_iter)) = span_to_text_iter(&buffer, &highlight.span) {
                let tag = get_or_create_tag(&buffer, &highlight.tag);
                buffer.apply_tag(&tag, &start_iter, &end_iter);
            }
        }

        index.set(end);

        if end >= highlights.len() {
            if let Some(done) = on_done.borrow_mut().take() {
                done();
            }
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
}

/// Remove all LSP tags from the buffer
///
/// This iterates through all LSP style names (lsp-*) and removes their tags
/// from the entire buffer range.
fn remove_all_lsp_tags(buffer: &sourceview5::Buffer) {
    let start = buffer.start_iter();
    let end = buffer.end_iter();
    let tag_table = buffer.tag_table();

    // Note: tag names are plain (no `lsp-` prefix). Use the authoritative list
    // from `ui::css::syntax` to keep everything in sync.
    for tag_name in crate::ui::css::syntax::LSP_TAG_NAMES {
        if let Some(tag) = tag_table.lookup(tag_name) {
            buffer.remove_tag(&tag, &start, &end);
        }
    }
}

/// Convert parser Span to GTK TextIter pair using line/column approach
///
/// # Algorithm
///
/// For each position in the span:
/// 1. Convert parser line (1-based) to GTK line (0-based)
/// 2. Get the text of that specific line from the buffer
/// 3. Convert parser column (byte offset) to GTK column (char offset)
/// 4. Create TextIter at (line, char_offset)
///
/// This per-line approach is robust because:
/// - No cumulative errors across the document
/// - Easy to debug (can verify one line at a time)
/// - Handles UTF-8 correctly (counts characters, not bytes)
///
/// # Returns
///
/// `Some((start_iter, end_iter))` if conversion succeeds, `None` if positions are invalid
fn span_to_text_iter(
    buffer: &sourceview5::Buffer,
    span: &core::parser::position::Span,
) -> Option<(gtk4::TextIter, gtk4::TextIter)> {
    let start = position_to_iter(buffer, &span.start)?;
    let end = position_to_iter(buffer, &span.end)?;

    Some((start, end))
}

/// Convert parser Position to GTK TextIter using line/column approach
///
/// # Coordinate Conversion
///
/// **Parser coordinates:**
/// - Line: 1-based (CommonMark convention)
/// - Column: 1-based byte offset from line start
///
/// **GTK coordinates:**
/// - Line: 0-based
/// - Column: 0-based character offset from line start
///
/// # Algorithm
///
/// 1. Convert line: `parser_line (1-based)` → `gtk_line = parser_line - 1 (0-based)`
/// 2. Get line text from buffer using `text(&line_start, &line_end, false)`
/// 3. Count characters up to the byte offset using `char_indices().take_while().count()`
/// 4. Create iterator at (gtk_line, char_offset) using `iter_at_line()` + `set_line_offset()`
///
/// # UTF-8 Handling
///
/// The conversion from byte offset to character offset handles multi-byte UTF-8:
/// - `char_indices()` returns (byte_index, char) pairs
/// - We count how many chars fit before the target byte offset
/// - Example: "Tëst" where 'ë' is 2 bytes:
///   - Byte offset 4 (before 's') → char offset 2
///
/// # Edge Cases
///
/// - Line out of bounds: Returns None
/// - Byte offset past line end: Clamps to line length
/// - Empty lines: Returns position 0 (start of line)
///
/// # Returns
///
/// `Some(TextIter)` if conversion succeeds, `None` if line is out of bounds
fn position_to_iter(
    buffer: &sourceview5::Buffer,
    position: &core::parser::position::Position,
) -> Option<gtk4::TextIter> {
    // Convert parser line (1-based) to GTK line (0-based)
    let gtk_line = position.line.saturating_sub(1);

    // Validate line number
    if gtk_line >= buffer.line_count() as usize {
        log::warn!(
            "Position line {} out of bounds (buffer has {} lines)",
            position.line,
            buffer.line_count()
        );
        return None;
    }

    // Get iterator at start of line
    let mut iter = buffer.iter_at_line(gtk_line as i32)?;

    // Get the text of this line for byte→char conversion.
    // IMPORTANT: Don't include the trailing '\n' in the extracted line text.
    // GTK line offsets are within the line; including the newline can skew
    // clamping and makes end-of-line positions harder to reason about.
    let line_start = buffer.iter_at_line(gtk_line as i32)?;
    let mut line_end = line_start;
    line_end.forward_to_line_end();
    let line_text = buffer.text(&line_start, &line_end, false);

    // Convert parser column (1-based byte offset) to GTK column (0-based char offset)
    let parser_byte_column = position.column.saturating_sub(1); // Convert to 0-based

    // Count characters from line start up to the byte offset.
    // This yields a *character* offset (not byte offset).
    let gtk_char_column = line_text
        .char_indices()
        .take_while(|(byte_idx, _)| *byte_idx < parser_byte_column)
        .count();

    // Clamp to GTK's notion of the line length.
    //
    // IMPORTANT: We do NOT rely solely on `line_text.chars().count()`.
    // In some edge cases (e.g. unusual line endings, buffer normalization,
    // or internal GTK representation), GTK may report a different
    // `chars_in_line` than our extracted string length.
    let gtk_max_chars = iter.chars_in_line().max(0) as usize;
    let gtk_char_column = gtk_char_column.min(gtk_max_chars);

    // Set position within line using character offset.
    // We still clamp, but GTK can be strict here; keep this as the only
    // callsite so invariants are clear.
    iter.set_line_offset(gtk_char_column as i32);

    Some(iter)
}

/// Get or create the `TextTag` for the given highlight kind.
///
/// **Important:** tag colors are applied elsewhere (see `crate::ui::css::syntax`).
/// This function is responsible only for ensuring a tag with the correct name
/// exists in the buffer's tag table.
fn get_or_create_tag(buffer: &sourceview5::Buffer, tag: &core::lsp::HighlightTag) -> gtk4::TextTag {
    use core::lsp::HighlightTag;

    // Map HighlightTag enum to theme style names (no `lsp-` prefix)
    let style_name = match tag {
        HighlightTag::Heading1 => "heading1",
        HighlightTag::Heading2 => "heading2",
        HighlightTag::Heading3 => "heading3",
        HighlightTag::Heading4 => "heading4",
        HighlightTag::Heading5 => "heading5",
        HighlightTag::Heading6 => "heading6",
        HighlightTag::Emphasis => "emphasis",
        HighlightTag::Strong => "strong",
        HighlightTag::Strikethrough => "strikethrough",
        HighlightTag::Mark => "mark",
        HighlightTag::Superscript => "superscript",
        HighlightTag::Subscript => "subscript",
        HighlightTag::Link => "link",
        HighlightTag::Image => "image",
        HighlightTag::CodeSpan => "code-span",
        HighlightTag::CodeBlock => "code-block",
        HighlightTag::InlineHtml => "inline-html",
        HighlightTag::HardBreak => "hard-break",
        HighlightTag::SoftBreak => "soft-break",
        HighlightTag::ThematicBreak => "thematic-break",
        HighlightTag::Blockquote => "blockquote",
        HighlightTag::HtmlBlock => "html-block",
        HighlightTag::List => "list",
        HighlightTag::ListItem => "list-item",
    };

    let tag_table = buffer.tag_table();

    // Check if tag already exists
    if let Some(existing_tag) = tag_table.lookup(style_name) {
        return existing_tag;
    }

    // Create new tag. Colors are assigned via `ui::css::syntax::apply_to_buffer()`.
    let new_tag = gtk4::TextTag::new(Some(style_name));

    // Add to tag table
    tag_table.add(&new_tag);

    log::debug!("Created LSP tag: {}", style_name);

    new_tag
}

#[cfg(test)]
mod tests {
    // Note: GTK tests require GTK to be initialized, which is complex in unit tests.
    // These tests are placeholders for the integration test approach.
    // Real testing should be done via manual testing with the application.

    #[test]
    fn test_position_conversion_logic() {
        // Test the byte→char conversion logic outside of GTK
        let text = "Tëst"; // 'ë' is 2 bytes at positions 1-2

        // Byte positions: T=0, ë=1-2, s=3, t=4
        // Char positions: T=0, ë=1, s=2, t=3

        // Count chars up to byte position 3 (before 's')
        let char_count = text
            .char_indices()
            .take_while(|(byte_idx, _)| *byte_idx < 3)
            .count();

        assert_eq!(char_count, 2); // T and ë = 2 characters
    }

    #[test]
    fn test_emoji_conversion_logic() {
        // Test: "🎨" emoji is 4 bytes but 1 character
        let text = "\u{1F3A8}Test";

        // Count chars up to byte position 4 (before 'T')
        let char_count = text
            .char_indices()
            .take_while(|(byte_idx, _)| *byte_idx < 4)
            .count();

        assert_eq!(char_count, 1); // Just the emoji
    }

    #[test]
    fn test_ascii_conversion_logic() {
        // Test: ASCII where byte offset == char offset
        let text = "Hello World";

        // Count chars up to byte position 5 (before ' ')
        let char_count = text
            .char_indices()
            .take_while(|(byte_idx, _)| *byte_idx < 5)
            .count();

        assert_eq!(char_count, 5); // "Hello"
    }
}
