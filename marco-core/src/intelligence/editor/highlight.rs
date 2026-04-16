// Syntax highlighting: map AST nodes to SourceView5 text tags

use crate::parser::{Document, Node, NodeKind, Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct Highlight {
    pub span: Span,
    pub tag: HighlightTag,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HighlightTag {
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
    Emphasis,
    Strong,
    Strikethrough,
    Mark,
    Superscript,
    Subscript,
    Link,
    Image,
    CodeSpan,
    CodeBlock,
    InlineHtml,
    HardBreak,
    SoftBreak,
    ThematicBreak,
    Blockquote,
    Admonition,
    HtmlBlock,
    List,
    ListItem,
    TaskCheckboxChecked,
    TaskCheckboxUnchecked,
    Table,
    TableRow,
    TableRowHeader,
    TableCell,
    TableCellHeader,
    LinkReference,
    DefinitionList,
    DefinitionTerm,
    DefinitionDescription,
    TabBlockContainer,
    TabBlockHeader,
    SliderDeckMarker,
    SliderSeparatorHorizontal,
    SliderSeparatorVertical,
}

pub fn compute_highlights(document: &Document) -> Vec<Highlight> {
    let mut highlights = Vec::new();

    for node in &document.children {
        collect_highlights(node, &mut highlights);
    }

    finalize_highlights(highlights)
}

pub fn compute_highlights_with_source(document: &Document, source: &str) -> Vec<Highlight> {
    let mut highlights = compute_highlights(document);
    highlights.extend(compute_tab_block_marker_highlights(source));
    highlights.extend(compute_slider_marker_highlights(source));
    finalize_highlights(highlights)
}

fn finalize_highlights(mut highlights: Vec<Highlight>) -> Vec<Highlight> {
    highlights.retain(|h| is_non_empty_span(&h.span));
    highlights.sort_by(|a, b| {
        let start_cmp =
            (a.span.start.line, a.span.start.column).cmp(&(b.span.start.line, b.span.start.column));
        if start_cmp != std::cmp::Ordering::Equal {
            return start_cmp;
        }

        let end_cmp =
            (b.span.end.line, b.span.end.column).cmp(&(a.span.end.line, a.span.end.column));
        if end_cmp != std::cmp::Ordering::Equal {
            return end_cmp;
        }

        tag_rank(&a.tag).cmp(&tag_rank(&b.tag))
    });
    highlights.dedup_by(|a, b| a.tag == b.tag && a.span == b.span);

    highlights
}

fn is_non_empty_span(span: &Span) -> bool {
    let start = (span.start.line, span.start.column);
    let end = (span.end.line, span.end.column);
    start < end
}

fn tag_rank(tag: &HighlightTag) -> u8 {
    match tag {
        HighlightTag::Heading1 => 1,
        HighlightTag::Heading2 => 2,
        HighlightTag::Heading3 => 3,
        HighlightTag::Heading4 => 4,
        HighlightTag::Heading5 => 5,
        HighlightTag::Heading6 => 6,
        HighlightTag::Emphasis => 10,
        HighlightTag::Strong => 11,
        HighlightTag::Strikethrough => 12,
        HighlightTag::Mark => 13,
        HighlightTag::Superscript => 14,
        HighlightTag::Subscript => 15,
        HighlightTag::Link => 16,
        HighlightTag::Image => 17,
        HighlightTag::CodeSpan => 20,
        HighlightTag::CodeBlock => 21,
        HighlightTag::InlineHtml => 30,
        HighlightTag::HardBreak => 40,
        HighlightTag::SoftBreak => 41,
        HighlightTag::ThematicBreak => 42,
        HighlightTag::Blockquote => 50,
        HighlightTag::Admonition => 51,
        HighlightTag::HtmlBlock => 52,
        HighlightTag::List => 60,
        HighlightTag::ListItem => 61,
        HighlightTag::TaskCheckboxUnchecked => 62,
        HighlightTag::TaskCheckboxChecked => 63,
        HighlightTag::Table => 70,
        HighlightTag::TableRowHeader => 71,
        HighlightTag::TableRow => 72,
        HighlightTag::TableCellHeader => 73,
        HighlightTag::TableCell => 74,
        HighlightTag::LinkReference => 80,
        HighlightTag::DefinitionList => 90,
        HighlightTag::DefinitionTerm => 91,
        HighlightTag::DefinitionDescription => 92,
        HighlightTag::TabBlockContainer => 100,
        HighlightTag::TabBlockHeader => 101,
        HighlightTag::SliderDeckMarker => 110,
        HighlightTag::SliderSeparatorHorizontal => 111,
        HighlightTag::SliderSeparatorVertical => 112,
    }
}

fn compute_tab_block_marker_highlights(source: &str) -> Vec<Highlight> {
    fn trim_upto_3_spaces(s: &str) -> (&str, usize) {
        let bytes = s.as_bytes();
        let mut i = 0usize;
        for _ in 0..3 {
            if bytes.get(i) == Some(&b' ') {
                i += 1;
            } else {
                break;
            }
        }
        (&s[i..], i)
    }

    fn fence_prefix(rest: &str) -> Option<(char, usize, &str)> {
        let mut chars = rest.chars();
        let ch = chars.next()?;
        if ch != '`' && ch != '~' {
            return None;
        }
        let mut count = 1usize;
        for c in chars.clone() {
            if c == ch {
                count += 1;
            } else {
                break;
            }
        }
        if count >= 3 {
            Some((ch, count, &rest[count..]))
        } else {
            None
        }
    }

    let mut highlights: Vec<Highlight> = Vec::new();
    let mut in_tab_block = false;
    let mut in_fence: Option<(char, usize)> = None;
    let mut line_start_offset: usize = 0;
    let mut line_no: usize = 1;

    for seg in source.split_inclusive('\n') {
        let seg_len = seg.len();

        let line = seg
            .strip_suffix('\n')
            .unwrap_or(seg)
            .strip_suffix('\r')
            .unwrap_or(seg.strip_suffix('\n').unwrap_or(seg));

        let (rest, _indent_len) = trim_upto_3_spaces(line);

        if let Some((fch, fcount, after_fence)) = fence_prefix(rest) {
            match in_fence {
                None => in_fence = Some((fch, fcount)),
                Some((open_ch, open_count)) => {
                    if fch == open_ch && fcount >= open_count && after_fence.trim().is_empty() {
                        in_fence = None;
                    }
                }
            }
        }

        if in_fence.is_none() {
            if !in_tab_block {
                if let Some(after) = rest.strip_prefix(":::tab") {
                    if after.is_empty()
                        || after
                            .chars()
                            .next()
                            .is_some_and(|ch| ch == ' ' || ch == '\t')
                    {
                        highlights.push(line_highlight(
                            line_no,
                            line_start_offset,
                            line.len(),
                            HighlightTag::TabBlockContainer,
                        ));
                        in_tab_block = true;
                    }
                }
            } else {
                if let Some(after) = rest.strip_prefix("@tab") {
                    let after = after.strip_prefix(' ').or_else(|| after.strip_prefix('\t'));
                    if let Some(after_ws) = after {
                        if !after_ws.trim().is_empty() {
                            highlights.push(line_highlight(
                                line_no,
                                line_start_offset,
                                line.len(),
                                HighlightTag::TabBlockHeader,
                            ));
                        }
                    }
                }

                if let Some(after) = rest.strip_prefix(":::") {
                    if after.trim().is_empty() {
                        highlights.push(line_highlight(
                            line_no,
                            line_start_offset,
                            line.len(),
                            HighlightTag::TabBlockContainer,
                        ));
                        in_tab_block = false;
                    }
                }
            }
        }

        line_start_offset = line_start_offset.saturating_add(seg_len);
        line_no = line_no.saturating_add(1);
    }

    highlights
}

fn compute_slider_marker_highlights(source: &str) -> Vec<Highlight> {
    fn trim_upto_3_spaces(s: &str) -> (&str, usize) {
        let bytes = s.as_bytes();
        let mut i = 0usize;
        for _ in 0..3 {
            if bytes.get(i) == Some(&b' ') {
                i += 1;
            } else {
                break;
            }
        }
        (&s[i..], i)
    }

    fn fence_prefix(rest: &str) -> Option<(char, usize, &str)> {
        let mut chars = rest.chars();
        let ch = chars.next()?;
        if ch != '`' && ch != '~' {
            return None;
        }
        let mut count = 1usize;
        for c in chars.clone() {
            if c == ch {
                count += 1;
            } else {
                break;
            }
        }
        if count >= 3 {
            Some((ch, count, &rest[count..]))
        } else {
            None
        }
    }

    let mut highlights: Vec<Highlight> = Vec::new();
    let mut in_slider_deck = false;
    let mut in_fence: Option<(char, usize)> = None;
    let mut line_start_offset: usize = 0;
    let mut line_no: usize = 1;

    for seg in source.split_inclusive('\n') {
        let seg_len = seg.len();

        let line = seg
            .strip_suffix('\n')
            .unwrap_or(seg)
            .strip_suffix('\r')
            .unwrap_or(seg.strip_suffix('\n').unwrap_or(seg));

        let (rest, _indent_len) = trim_upto_3_spaces(line);

        if let Some((fch, fcount, after_fence)) = fence_prefix(rest) {
            match in_fence {
                None => in_fence = Some((fch, fcount)),
                Some((open_ch, open_count)) => {
                    if fch == open_ch && fcount >= open_count && after_fence.trim().is_empty() {
                        in_fence = None;
                    }
                }
            }
        }

        if in_fence.is_none() {
            if !in_slider_deck {
                if let Some(after) = rest.strip_prefix("@slidestart") {
                    let ok = after.is_empty()
                        || after
                            .chars()
                            .next()
                            .is_some_and(|ch| ch == ' ' || ch == '\t' || ch == ':');
                    if ok {
                        highlights.push(line_highlight(
                            line_no,
                            line_start_offset,
                            line.len(),
                            HighlightTag::SliderDeckMarker,
                        ));
                        in_slider_deck = true;
                    }
                }
            } else {
                if let Some(after) = rest.strip_prefix("@slideend") {
                    if after.is_empty()
                        || after
                            .chars()
                            .next()
                            .is_some_and(|ch| ch == ' ' || ch == '\t')
                    {
                        highlights.push(line_highlight(
                            line_no,
                            line_start_offset,
                            line.len(),
                            HighlightTag::SliderDeckMarker,
                        ));
                        in_slider_deck = false;
                    }
                }

                if rest.trim() == "---" {
                    highlights.push(line_highlight(
                        line_no,
                        line_start_offset,
                        line.len(),
                        HighlightTag::SliderSeparatorHorizontal,
                    ));
                } else if rest.trim() == "--" {
                    highlights.push(line_highlight(
                        line_no,
                        line_start_offset,
                        line.len(),
                        HighlightTag::SliderSeparatorVertical,
                    ));
                }
            }
        }

        line_start_offset = line_start_offset.saturating_add(seg_len);
        line_no = line_no.saturating_add(1);
    }

    highlights
}

fn line_highlight(
    line: usize,
    line_start_offset: usize,
    line_len_bytes: usize,
    tag: HighlightTag,
) -> Highlight {
    let start = Position::new(line, 1, line_start_offset);
    let end = Position::new(line, line_len_bytes + 1, line_start_offset + line_len_bytes);
    Highlight {
        span: Span::new(start, end),
        tag,
    }
}

fn collect_highlights(node: &Node, highlights: &mut Vec<Highlight>) {
    if let Some(span) = &node.span {
        match &node.kind {
            NodeKind::Heading { level, .. } => {
                let tag = match level {
                    1 => HighlightTag::Heading1,
                    2 => HighlightTag::Heading2,
                    3 => HighlightTag::Heading3,
                    4 => HighlightTag::Heading4,
                    5 => HighlightTag::Heading5,
                    6 => HighlightTag::Heading6,
                    _ => HighlightTag::Heading1,
                };

                let full_line_span = Span::new(
                    Position::new(span.start.line, 1, span.start_line_offset()),
                    span.end,
                );

                highlights.push(Highlight {
                    span: full_line_span,
                    tag,
                });
            }
            NodeKind::Emphasis => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::Emphasis,
            }),
            NodeKind::Strong | NodeKind::StrongEmphasis => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::Strong,
            }),
            NodeKind::Strikethrough => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::Strikethrough,
            }),
            NodeKind::Mark => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::Mark,
            }),
            NodeKind::Superscript => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::Superscript,
            }),
            NodeKind::Subscript => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::Subscript,
            }),
            NodeKind::Link { .. }
            | NodeKind::PlatformMention { .. }
            | NodeKind::FootnoteReference { .. } => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::Link,
            }),
            NodeKind::Image { .. } => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::Image,
            }),
            NodeKind::CodeSpan(_) | NodeKind::InlineMath { .. } => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::CodeSpan,
            }),
            NodeKind::CodeBlock { .. }
            | NodeKind::DisplayMath { .. }
            | NodeKind::MermaidDiagram { .. } => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::CodeBlock,
            }),
            NodeKind::InlineHtml(_) => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::InlineHtml,
            }),
            NodeKind::ThematicBreak => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::ThematicBreak,
            }),
            NodeKind::HtmlBlock { .. } => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::HtmlBlock,
            }),
            NodeKind::Blockquote => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::Blockquote,
            }),
            NodeKind::Admonition { .. } => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::Admonition,
            }),
            NodeKind::List { .. } => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::List,
            }),
            NodeKind::ListItem => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::ListItem,
            }),
            NodeKind::TaskCheckbox { checked } | NodeKind::TaskCheckboxInline { checked } => {
                highlights.push(Highlight {
                    span: *span,
                    tag: if *checked {
                        HighlightTag::TaskCheckboxChecked
                    } else {
                        HighlightTag::TaskCheckboxUnchecked
                    },
                })
            }
            NodeKind::Table { .. } => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::Table,
            }),
            NodeKind::TableRow { header } => highlights.push(Highlight {
                span: *span,
                tag: if *header {
                    HighlightTag::TableRowHeader
                } else {
                    HighlightTag::TableRow
                },
            }),
            NodeKind::TableCell { header, .. } => highlights.push(Highlight {
                span: *span,
                tag: if *header {
                    HighlightTag::TableCellHeader
                } else {
                    HighlightTag::TableCell
                },
            }),
            NodeKind::LinkReference { .. } => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::LinkReference,
            }),
            NodeKind::DefinitionList => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::DefinitionList,
            }),
            NodeKind::DefinitionTerm => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::DefinitionTerm,
            }),
            NodeKind::DefinitionDescription => highlights.push(Highlight {
                span: *span,
                tag: HighlightTag::DefinitionDescription,
            }),
            NodeKind::Paragraph
            | NodeKind::Text(_)
            | NodeKind::HardBreak
            | NodeKind::SoftBreak
            | NodeKind::TabGroup
            | NodeKind::TabItem { .. }
            | NodeKind::SliderDeck { .. }
            | NodeKind::Slide { .. }
            | NodeKind::FootnoteDefinition { .. } => {}
        }
    }

    for child in &node.children {
        collect_highlights(child, highlights);
    }
}
