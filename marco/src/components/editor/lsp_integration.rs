// LSP integration for Marco editor
// Applies syntax highlighting based on parser LSP features

use gtk4::prelude::*;
use sourceview5::prelude::*;

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
pub fn apply_lsp_highlights(
    buffer: &sourceview5::Buffer,
    highlights: &[core::lsp::Highlight],
) {
    log::debug!("Applying {} LSP highlights", highlights.len());
    
    // Remove all existing LSP tags first
    remove_all_lsp_tags(buffer);
    
    // Apply each highlight
    for highlight in highlights {
        if let Some((start_iter, end_iter)) = span_to_text_iter(buffer, &highlight.span) {
            let tag = get_or_create_tag(buffer, &highlight.tag);
            buffer.apply_tag(&tag, &start_iter, &end_iter);
            
            log::trace!(
                "Applied {:?} tag from [{}:{} to {}:{}]",
                highlight.tag,
                highlight.span.start.line, highlight.span.start.column,
                highlight.span.end.line, highlight.span.end.column
            );
        } else {
            log::warn!(
                "Failed to convert span for highlight {:?} at [{}:{} to {}:{}]",
                highlight.tag,
                highlight.span.start.line, highlight.span.start.column,
                highlight.span.end.line, highlight.span.end.column
            );
        }
    }
    
    log::debug!("LSP highlights applied successfully");
}

/// Remove all LSP tags from the buffer
///
/// This iterates through all LSP style names (lsp-*) and removes their tags
/// from the entire buffer range.
fn remove_all_lsp_tags(buffer: &sourceview5::Buffer) {
    let start = buffer.start_iter();
    let end = buffer.end_iter();
    let tag_table = buffer.tag_table();
    
    // List of all LSP tag names (must match the theme style names)
    let lsp_tags = [
        "lsp-heading1", "lsp-heading2", "lsp-heading3", "lsp-heading4", "lsp-heading5", "lsp-heading6",
        "lsp-emphasis", "lsp-strong", "lsp-link", "lsp-image",
        "lsp-code-span", "lsp-code-block", "lsp-inline-html",
        "lsp-hard-break", "lsp-soft-break", "lsp-thematic-break",
        "lsp-blockquote", "lsp-html-block", "lsp-list", "lsp-list-item",
    ];
    
    for tag_name in &lsp_tags {
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
    
    log::trace!(
        "span_to_text_iter: span [{}:{} to {}:{}]",
        span.start.line, span.start.column,
        span.end.line, span.end.column
    );
    
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
            position.line, buffer.line_count()
        );
        return None;
    }
    
    // Get iterator at start of line
    let mut iter = buffer.iter_at_line(gtk_line as i32)?;
    
    // Get the text of this line for byte→char conversion
    let line_start = buffer.iter_at_line(gtk_line as i32)?;
    let line_end = if gtk_line + 1 < buffer.line_count() as usize {
        buffer.iter_at_line((gtk_line + 1) as i32)?
    } else {
        buffer.end_iter()
    };
    let line_text = buffer.text(&line_start, &line_end, false);
    
    // Convert parser column (1-based byte offset) to GTK column (0-based char offset)
    let parser_byte_column = position.column.saturating_sub(1); // Convert to 0-based
    
    // Count characters from line start up to the byte offset
    let gtk_char_column = line_text
        .char_indices()
        .take_while(|(byte_idx, _)| *byte_idx < parser_byte_column)
        .count();
    
    // Clamp to line length
    let max_chars = line_text.chars().count();
    let gtk_char_column = gtk_char_column.min(max_chars);
    
    // Set position within line using character offset
    iter.set_line_offset(gtk_char_column as i32);
    
    log::trace!(
        "position_to_iter: parser (line={}, byte_col={}) → GTK (line={}, char_col={})",
        position.line, position.column, gtk_line, gtk_char_column
    );
    
    Some(iter)
}

/// Get or create a TextTag for the given LSP highlight tag
///
/// Tags are looked up from the SourceView style scheme using the naming convention:
/// "lsp-{tag_name}" (e.g., "lsp-heading1", "lsp-emphasis", etc.)
///
/// The colors and styles are defined in the theme XML files:
/// - `assets/themes/editor/dark.xml`
/// - `assets/themes/editor/light.xml`
///
/// # Arguments
///
/// * `buffer` - The SourceView buffer (provides access to the style scheme)
/// * `tag` - The LSP highlight tag type
///
/// # Returns
///
/// The GTK TextTag for this highlight type, with styling from the theme
fn get_or_create_tag(
    buffer: &sourceview5::Buffer,
    tag: &core::lsp::HighlightTag,
) -> gtk4::TextTag {
    use core::lsp::HighlightTag;
    
    // Map HighlightTag enum to theme style names
    let style_name = match tag {
        HighlightTag::Heading1 => "lsp-heading1",
        HighlightTag::Heading2 => "lsp-heading2",
        HighlightTag::Heading3 => "lsp-heading3",
        HighlightTag::Heading4 => "lsp-heading4",
        HighlightTag::Heading5 => "lsp-heading5",
        HighlightTag::Heading6 => "lsp-heading6",
        HighlightTag::Emphasis => "lsp-emphasis",
        HighlightTag::Strong => "lsp-strong",
        HighlightTag::Link => "lsp-link",
        HighlightTag::Image => "lsp-image",
        HighlightTag::CodeSpan => "lsp-code-span",
        HighlightTag::CodeBlock => "lsp-code-block",
        HighlightTag::InlineHtml => "lsp-inline-html",
        HighlightTag::HardBreak => "lsp-hard-break",
        HighlightTag::SoftBreak => "lsp-soft-break",
        HighlightTag::ThematicBreak => "lsp-thematic-break",
        HighlightTag::Blockquote => "lsp-blockquote",
        HighlightTag::HtmlBlock => "lsp-html-block",
        HighlightTag::List => "lsp-list",
        HighlightTag::ListItem => "lsp-list-item",
    };
    
    let tag_table = buffer.tag_table();
    
    // Check if tag already exists
    if let Some(existing_tag) = tag_table.lookup(style_name) {
        return existing_tag;
    }
    
    // Create new tag
    let new_tag = gtk4::TextTag::new(Some(style_name));
    
    // Get the style from the SourceView style scheme
    if let Some(scheme) = buffer.style_scheme() {
        if let Some(style) = scheme.style(style_name) {
            // Apply foreground color from theme
            if let Some(fg) = style.foreground() {
                new_tag.set_foreground(Some(&fg));
            }
            
            // Apply background color from theme (if defined)
            if let Some(bg) = style.background() {
                new_tag.set_background(Some(&bg));
            }
            
            // Note: We intentionally do NOT apply bold, italic, underline, or other
            // font properties here. Only colors are used from the theme, as per
            // the theme XML restrictions.
        } else {
            log::warn!("Style '{}' not found in theme, using default", style_name);
        }
    } else {
        log::warn!("No style scheme available, tag '{}' will use defaults", style_name);
    }
    
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
        let text = "🎨Test";
        
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
