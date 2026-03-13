//! Intelligence integration for Marco editor
//!
//! This module provides syntax highlighting based on the parser's markdown
//! intelligence features. It applies highlight tags to the GTK SourceView
//! buffer using chunked processing to avoid blocking the main thread.

use gtk4::pango;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Apply intelligence highlights in small chunks scheduled on the main loop.
pub fn apply_intelligence_highlights_chunked<F>(
    buffer: &sourceview5::Buffer,
    highlights: Vec<core::intelligence::Highlight>,
    on_done: F,
) where
    F: FnOnce() + 'static,
{
    const CHUNK_SIZE: usize = 400;

    log::debug!(
        "Applying {} intelligence highlights (chunked, chunk_size={})",
        highlights.len(),
        CHUNK_SIZE
    );

    remove_all_intelligence_tags(buffer);

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

/// Apply diagnostics markers in small chunks scheduled on the main loop.
pub fn apply_diagnostics_markers_chunked<F>(
    buffer: &sourceview5::Buffer,
    diagnostics: Vec<core::intelligence::Diagnostic>,
    on_done: F,
) where
    F: FnOnce() + 'static,
{
    const CHUNK_SIZE: usize = 400;

    // When multiple diagnostics share the same span, only show the most severe
    // one visually (mirrors VS Code behaviour, avoids conflicting tag colours).
    let diagnostics = deduplicate_diagnostics_by_span(diagnostics);

    remove_all_diagnostic_tags(buffer);

    if diagnostics.is_empty() {
        on_done();
        return;
    }

    let buffer = buffer.clone();
    let diagnostics = Rc::new(diagnostics);
    let index = Rc::new(Cell::new(0usize));
    let on_done = Rc::new(RefCell::new(Some(Box::new(on_done) as Box<dyn FnOnce()>)));

    glib::idle_add_local(move || {
        let start = index.get();
        let end = (start + CHUNK_SIZE).min(diagnostics.len());

        for diagnostic in &diagnostics[start..end] {
            if let Some((start_iter, end_iter)) = span_to_text_iter(&buffer, &diagnostic.span) {
                let tag = get_or_create_diagnostic_tag(&buffer, &diagnostic.severity);
                buffer.apply_tag(&tag, &start_iter, &end_iter);
            } else {
                log::warn!(
                    "Diagnostic {:?} span=({},{})→({},{}) resolved to None — skipped",
                    diagnostic.code,
                    diagnostic.span.start.line,
                    diagnostic.span.start.column,
                    diagnostic.span.end.line,
                    diagnostic.span.end.column,
                );
            }
        }

        index.set(end);

        if end >= diagnostics.len() {
            if let Some(done) = on_done.borrow_mut().take() {
                done();
            }
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
}

fn remove_all_intelligence_tags(buffer: &sourceview5::Buffer) {
    let start = buffer.start_iter();
    let end = buffer.end_iter();
    let tag_table = buffer.tag_table();

    for tag_name in crate::ui::css::syntax::INTELLIGENCE_TAG_NAMES {
        if let Some(tag) = tag_table.lookup(tag_name) {
            buffer.remove_tag(&tag, &start, &end);
        }
    }
}

fn remove_all_diagnostic_tags(buffer: &sourceview5::Buffer) {
    let start = buffer.start_iter();
    let end = buffer.end_iter();
    let tag_table = buffer.tag_table();

    for tag_name in [
        "diagnostic-error",
        "diagnostic-warning",
        "diagnostic-info",
        "diagnostic-hint",
    ] {
        if let Some(tag) = tag_table.lookup(tag_name) {
            buffer.remove_tag(&tag, &start, &end);
        }
    }
}

fn span_to_text_iter(
    buffer: &sourceview5::Buffer,
    span: &core::parser::position::Span,
) -> Option<(gtk4::TextIter, gtk4::TextIter)> {
    let start = position_to_iter(buffer, &span.start)?;
    let end = position_to_iter(buffer, &span.end)?;

    Some((start, end))
}

fn position_to_iter(
    buffer: &sourceview5::Buffer,
    position: &core::parser::position::Position,
) -> Option<gtk4::TextIter> {
    let gtk_line = position.line.saturating_sub(1);

    if gtk_line >= buffer.line_count() as usize {
        log::warn!(
            "Position line {} out of bounds (buffer has {} lines)",
            position.line,
            buffer.line_count()
        );
        return None;
    }

    let mut iter = buffer.iter_at_line(gtk_line as i32)?;

    let line_start = buffer.iter_at_line(gtk_line as i32)?;
    let mut line_end = line_start;
    line_end.forward_to_line_end();
    let line_text = buffer.text(&line_start, &line_end, false);

    let parser_byte_column = position.column.saturating_sub(1);

    let gtk_char_column = line_text
        .char_indices()
        .take_while(|(byte_idx, _)| *byte_idx < parser_byte_column)
        .count();

    let gtk_max_chars = iter.chars_in_line().max(0) as usize;
    let gtk_char_column = gtk_char_column.min(gtk_max_chars);

    iter.set_line_offset(gtk_char_column as i32);

    Some(iter)
}

fn get_or_create_tag(
    buffer: &sourceview5::Buffer,
    tag: &core::intelligence::HighlightTag,
) -> gtk4::TextTag {
    use core::intelligence::HighlightTag;

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
        HighlightTag::Admonition => "blockquote",
        HighlightTag::HtmlBlock => "html-block",
        HighlightTag::List => "list",
        HighlightTag::ListItem => "list-item",
        HighlightTag::TaskCheckboxChecked => "task-checkbox-checked",
        HighlightTag::TaskCheckboxUnchecked => "task-checkbox-unchecked",
        HighlightTag::Table => "table",
        HighlightTag::TableRow => "table-row",
        HighlightTag::TableRowHeader => "table-row-header",
        HighlightTag::TableCell => "table-cell",
        HighlightTag::TableCellHeader => "table-cell-header",
        HighlightTag::LinkReference => "link-reference",
        HighlightTag::DefinitionList => "definition-list",
        HighlightTag::DefinitionTerm => "definition-term",
        HighlightTag::DefinitionDescription => "definition-description",
        HighlightTag::TabBlockContainer => "tab-block-container",
        HighlightTag::TabBlockHeader => "tab-block-header",
        HighlightTag::SliderDeckMarker => "slider-deck-marker",
        HighlightTag::SliderSeparatorHorizontal => "slider-separator-horizontal",
        HighlightTag::SliderSeparatorVertical => "slider-separator-vertical",
    };

    let tag_table = buffer.tag_table();

    if let Some(existing_tag) = tag_table.lookup(style_name) {
        return existing_tag;
    }

    let new_tag = gtk4::TextTag::new(Some(style_name));
    tag_table.add(&new_tag);

    log::debug!("Created intelligence tag: {}", style_name);

    new_tag
}

fn get_or_create_diagnostic_tag(
    buffer: &sourceview5::Buffer,
    severity: &core::intelligence::DiagnosticSeverity,
) -> gtk4::TextTag {
    use core::intelligence::DiagnosticSeverity;

    let style_name = match severity {
        DiagnosticSeverity::Error => "diagnostic-error",
        DiagnosticSeverity::Warning => "diagnostic-warning",
        DiagnosticSeverity::Info => "diagnostic-info",
        DiagnosticSeverity::Hint => "diagnostic-hint",
    };

    let tag_table = buffer.tag_table();

    if let Some(existing_tag) = tag_table.lookup(style_name) {
        return existing_tag;
    }

    let new_tag = gtk4::TextTag::new(Some(style_name));

    match severity {
        DiagnosticSeverity::Error => {
            new_tag.set_underline(pango::Underline::Double);
            // red: #e05252
            new_tag.set_underline_rgba(Some(&gtk4::gdk::RGBA::new(0.878, 0.322, 0.322, 1.0)));
        }
        DiagnosticSeverity::Warning => {
            new_tag.set_underline(pango::Underline::Single);
            // yellow: #f0c040
            new_tag.set_underline_rgba(Some(&gtk4::gdk::RGBA::new(0.941, 0.753, 0.251, 1.0)));
        }
        DiagnosticSeverity::Info | DiagnosticSeverity::Hint => {
            new_tag.set_underline(pango::Underline::Single);
            // blue: #4f8cff
            new_tag.set_underline_rgba(Some(&gtk4::gdk::RGBA::new(0.310, 0.549, 1.0, 1.0)));
        }
    }

    tag_table.add(&new_tag);
    new_tag
}

fn severity_rank(severity: &core::intelligence::DiagnosticSeverity) -> u8 {
    use core::intelligence::DiagnosticSeverity;
    match severity {
        DiagnosticSeverity::Error => 3,
        DiagnosticSeverity::Warning => 2,
        DiagnosticSeverity::Info => 1,
        DiagnosticSeverity::Hint => 0,
    }
}

/// Deduplicate diagnostics by span, keeping the highest-severity entry per
/// unique (start_offset, end_offset) pair.
fn deduplicate_diagnostics_by_span(
    diagnostics: Vec<core::intelligence::Diagnostic>,
) -> Vec<core::intelligence::Diagnostic> {
    let mut map: std::collections::HashMap<(usize, usize), core::intelligence::Diagnostic> =
        std::collections::HashMap::new();
    for d in diagnostics {
        let key = (d.span.start.offset, d.span.end.offset);
        match map.entry(key) {
            std::collections::hash_map::Entry::Vacant(v) => {
                v.insert(d);
            }
            std::collections::hash_map::Entry::Occupied(mut o) => {
                if severity_rank(&d.severity) > severity_rank(&o.get().severity) {
                    *o.get_mut() = d;
                }
            }
        }
    }
    map.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_conversion_logic() {
        let text = "Tëst";

        let char_count = text
            .char_indices()
            .take_while(|(byte_idx, _)| *byte_idx < 3)
            .count();

        assert_eq!(char_count, 2);
    }

    #[test]
    fn test_emoji_conversion_logic() {
        let text = "\u{1F3A8}Test";

        let char_count = text
            .char_indices()
            .take_while(|(byte_idx, _)| *byte_idx < 4)
            .count();

        assert_eq!(char_count, 1);
    }

    #[test]
    fn smoke_test_diagnostic_tags_have_underline_rgba() {
        if let Err(err) = gtk4::init() {
            eprintln!(
                "Skipping smoke_test_diagnostic_tags_have_underline_rgba: GTK init failed: {}",
                err
            );
            return;
        }

        let buffer = sourceview5::Buffer::new(None::<&gtk4::TextTagTable>);

        let severities = [
            core::intelligence::DiagnosticSeverity::Error,
            core::intelligence::DiagnosticSeverity::Warning,
            core::intelligence::DiagnosticSeverity::Info,
            core::intelligence::DiagnosticSeverity::Hint,
        ];

        for severity in severities {
            let tag = get_or_create_diagnostic_tag(&buffer, &severity);
            assert!(
                tag.underline_rgba().is_some(),
                "expected underline color for severity {:?}",
                severity
            );
        }
    }

    #[test]
    fn test_ascii_conversion_logic() {
        let text = "Hello World";

        let char_count = text
            .char_indices()
            .take_while(|(byte_idx, _)| *byte_idx < 5)
            .count();

        assert_eq!(char_count, 5);
    }
}
