// HTML output generator with syntax highlighting for code blocks

use super::code_languages::language_display_label;
use super::plarform_mentions;
use super::syntect_highlighter::highlight_code_to_classed_html;
use super::RenderOptions;
use crate::parser::{AdmonitionKind, AdmonitionStyle, Document, Node, NodeKind};
use anyhow::Result;
use std::collections::HashMap;

const HEADING_ANCHOR_SVG: &str = concat!(
    r#"<svg xmlns=""#,
    "http",
    r#"://www.w3.org/2000/svg"#,
    r#"" width="1em" height="1em" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.0" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-anchor" focusable="false" aria-hidden="true"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M12 9v12m-8 -8a8 8 0 0 0 16 0m1 0h-2m-14 0h-2" /><path d="M9 6a3 3 0 1 0 6 0a3 3 0 1 0 -6 0" /></svg>"#,
);

// Marco sliders UI icons (Tabler).
// These are embedded as inline SVG so they inherit `currentColor`.
const SLIDER_ARROW_LEFT_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-arrow-narrow-left" aria-hidden="true"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M5 12l14 0" /><path d="M5 12l4 4" /><path d="M5 12l4 -4" /></svg>"#;

const SLIDER_ARROW_RIGHT_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-arrow-narrow-right" aria-hidden="true"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M5 12l14 0" /><path d="M15 16l4 -4" /><path d="M15 8l4 4" /></svg>"#;

const SLIDER_PLAY_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-player-play" aria-hidden="true"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M7 4v16l13 -8l-13 -8" /></svg>"#;

const SLIDER_PAUSE_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-player-pause" aria-hidden="true"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M6 6a1 1 0 0 1 1 -1h2a1 1 0 0 1 1 1v12a1 1 0 0 1 -1 1h-2a1 1 0 0 1 -1 -1l0 -12" /><path d="M14 6a1 1 0 0 1 1 -1h2a1 1 0 0 1 1 1v12a1 1 0 0 1 -1 1h-2a1 1 0 0 1 -1 -1l0 -12" /></svg>"#;

const SLIDER_DOT_INACTIVE_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-point" aria-hidden="true"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M8 12a4 4 0 1 0 8 0a4 4 0 1 0 -8 0" /></svg>"#;

const SLIDER_DOT_ACTIVE_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="currentColor" class="icon icon-tabler icons-tabler-filled icon-tabler-point" aria-hidden="true"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M12 7a5 5 0 1 1 -4.995 5.217l-.005 -.217l.005 -.217a5 5 0 0 1 4.995 -4.783z" /></svg>"#;

#[derive(Default)]
struct RenderContext<'a> {
    footnote_defs: HashMap<String, &'a Node>,
    footnote_numbers: HashMap<String, usize>,
    footnote_order: Vec<String>,
    footnote_ref_counts: HashMap<String, usize>,
    tab_group_counter: usize,
    slider_deck_counter: usize,
}

// Render document to HTML
pub fn render_html(document: &Document, options: &RenderOptions) -> Result<String> {
    log::debug!("Rendering {} nodes to HTML", document.len());

    let mut html = String::new();

    let mut ctx = RenderContext::default();
    for node in &document.children {
        collect_footnote_definitions(node, &mut ctx.footnote_defs);
    }

    for node in &document.children {
        render_node(node, &mut html, options, &mut ctx)?;
    }

    if !ctx.footnote_order.is_empty() {
        html.push_str("<section class=\"footnotes\">\n");
        html.push_str("<ol>\n");

        let mut i = 0usize;
        while i < ctx.footnote_order.len() {
            let label = ctx.footnote_order[i].clone();
            let Some(n) = ctx.footnote_numbers.get(&label).copied() else {
                i += 1;
                continue;
            };

            let Some(def_node) = ctx.footnote_defs.get(&label).copied() else {
                i += 1;
                continue;
            };

            html.push_str(&format!("<li id=\"fn{}\">", n));
            for child in &def_node.children {
                render_node(child, &mut html, options, &mut ctx)?;
            }
            html.push_str("</li>\n");

            i += 1;
        }

        html.push_str("</ol>\n");
        html.push_str("</section>\n");
    }

    Ok(html)
}

fn collect_footnote_definitions<'a>(node: &'a Node, defs: &mut HashMap<String, &'a Node>) {
    if let NodeKind::FootnoteDefinition { label } = &node.kind {
        defs.entry(label.clone()).or_insert(node);
    }

    for child in &node.children {
        collect_footnote_definitions(child, defs);
    }
}

// Render individual node
fn render_node(
    node: &Node,
    output: &mut String,
    options: &RenderOptions,
    ctx: &mut RenderContext<'_>,
) -> Result<()> {
    match &node.kind {
        NodeKind::Heading { level, text, id } => {
            log::trace!("Rendering heading level {}", level);
            let escaped_text = escape_html(text);
            output.push_str("<h");
            output.push_str(&level.to_string());
            if let Some(id) = id {
                output.push_str(" id=\"");
                output.push_str(&escape_html(id));
                output.push('"');
            }
            output.push('>');
            output.push_str(&escaped_text);

            // Optional, GitHub-style heading anchor link shown on hover.
            // We keep the SVG stroke as `currentColor` and let CSS inherit
            // the heading color so it matches the theme.
            if let Some(id) = id {
                output.push_str("<a class=\"marco-heading-anchor\" href=\"#");
                output.push_str(&escape_html(id));
                output.push_str("\" aria-label=\"Link to this heading\">");
                output.push_str(HEADING_ANCHOR_SVG);
                output.push_str("</a>");
            }

            output.push_str("</h");
            output.push_str(&level.to_string());
            output.push_str(">\n");
        }
        NodeKind::Paragraph => {
            output.push_str("<p>");
            for child in &node.children {
                render_node(child, output, options, ctx)?;
            }
            output.push_str("</p>\n");
        }
        NodeKind::CodeBlock { language, code } => {
            log::trace!("Rendering code block: {:?}", language);
            let language_raw = language.as_deref().map(str::trim).filter(|s| !s.is_empty());

            output.push_str("<pre");
            if let Some(raw) = language_raw {
                if let Some(label) = language_display_label(raw) {
                    output.push_str(" data-language=\"");
                    output.push_str(&escape_html(label.as_ref()));
                    output.push('"');
                }
            }
            output.push_str("><code");

            // Add language class attribute if language specified
            if let Some(lang) = language_raw {
                output.push_str(&format!(" class=\"language-{}\"", escape_html(lang)));
            }

            output.push('>');

            // Optional syntax highlighting. If syntect can't resolve the language,
            // fall back to plain escaped code.
            if options.syntax_highlighting {
                if let Some(lang) = language_raw {
                    if let Some(highlighted) = highlight_code_to_classed_html(code, lang) {
                        output.push_str(&highlighted);
                        output.push_str("</code></pre>\n");
                        return Ok(());
                    }
                }
            }

            output.push_str(&escape_html(code));
            output.push_str("</code></pre>\n");
        }
        NodeKind::ThematicBreak => {
            output.push_str("<hr />\n");
        }
        NodeKind::HtmlBlock { html } => {
            // HTML blocks are rendered as-is without escaping
            // They already contain the complete HTML including tags
            output.push_str(html);
            if !html.ends_with('\n') {
                output.push('\n');
            }
        }
        NodeKind::Blockquote => {
            output.push_str("<blockquote>\n");
            for child in &node.children {
                render_node(child, output, options, ctx)?;
            }
            output.push_str("</blockquote>\n");
        }
        NodeKind::Admonition {
            kind,
            title,
            icon,
            style,
        } => {
            let (slug, default_title, icon_svg) = admonition_presentation(kind);

            let title_text = title.as_deref().unwrap_or(default_title);

            // Render with both GitHub-compatible classes (`markdown-alert`) and
            // our theme-compatible classes (`admonition`).
            //
            // Quote-style admonitions intentionally omit the `*-<kind>` classes
            // so themes keep neutral/blockquote-like colors.
            output.push_str("<div class=\"");
            output.push_str("markdown-alert");

            if *style == AdmonitionStyle::Alert {
                output.push_str(" markdown-alert-");
                output.push_str(slug);
            }

            output.push_str(" admonition");

            if *style == AdmonitionStyle::Alert {
                output.push_str(" admonition-");
                output.push_str(slug);
            } else {
                output.push_str(" admonition-quote");
            }

            output.push_str("\">\n");

            output.push_str("<p class=\"markdown-alert-title\">");
            output.push_str("<span class=\"markdown-alert-icon\" aria-hidden=\"true\">");
            if let Some(icon_text) = icon {
                // Icon is text (typically emoji). Use an inner span so themes can
                // override line-height without affecting SVG icons.
                output.push_str("<span class=\"markdown-alert-emoji\">");
                output.push_str(&escape_html(icon_text));
                output.push_str("</span>");
            } else {
                output.push_str(icon_svg);
            }
            output.push_str("</span>");
            output.push_str(&escape_html(title_text));
            output.push_str("</p>\n");

            for child in &node.children {
                render_node(child, output, options, ctx)?;
            }

            output.push_str("</div>\n");
        }
        NodeKind::TabGroup => {
            render_tab_group(node, output, options, ctx)?;
        }
        NodeKind::TabItem { .. } => {
            // Tab items should be rendered via `render_tab_group` so we can
            // coordinate radio inputs/labels/panels.
            log::warn!("TabItem rendered outside of TabGroup context");
            for child in &node.children {
                render_node(child, output, options, ctx)?;
            }
        }
        NodeKind::SliderDeck { .. } => {
            render_slider_deck(node, output, options, ctx)?;
        }
        NodeKind::Slide { .. } => {
            // Slides should be rendered via `render_slider_deck` so we can
            // coordinate controls and timers.
            log::warn!("Slide rendered outside of SliderDeck context");
            for child in &node.children {
                render_node(child, output, options, ctx)?;
            }
        }
        NodeKind::Table { .. } => {
            render_table(node, output, options, ctx)?;
        }
        NodeKind::TableRow { .. } => {
            // Tables should be rendered via `render_table` so we can decide
            // whether a row belongs in <thead> or <tbody>.
            log::warn!("TableRow rendered outside of Table context");
            render_table_row(node, output, options, ctx)?;
            output.push('\n');
        }
        NodeKind::TableCell { .. } => {
            // Cells should be rendered via `render_table_row`.
            log::warn!("TableCell rendered outside of TableRow context");
            render_table_cell(node, output, options, ctx)?;
        }
        NodeKind::FootnoteDefinition { .. } => {
            // Footnote definitions are rendered in a dedicated section at the
            // end of the document.
        }
        NodeKind::Text(text) => {
            output.push_str(&escape_html(text));
        }
        NodeKind::CodeSpan(code) => {
            output.push_str("<code>");
            output.push_str(&escape_html(code));
            output.push_str("</code>");
        }
        NodeKind::Emphasis => {
            output.push_str("<em>");
            for child in &node.children {
                render_node(child, output, options, ctx)?;
            }
            output.push_str("</em>");
        }
        NodeKind::Strong => {
            output.push_str("<strong>");
            for child in &node.children {
                render_node(child, output, options, ctx)?;
            }
            output.push_str("</strong>");
        }
        NodeKind::StrongEmphasis => {
            // Triple delimiter: bold + italic.
            output.push_str("<strong><em>");
            for child in &node.children {
                render_node(child, output, options, ctx)?;
            }
            output.push_str("</em></strong>");
        }
        NodeKind::Strikethrough => {
            output.push_str("<del>");
            for child in &node.children {
                render_node(child, output, options, ctx)?;
            }
            output.push_str("</del>");
        }
        NodeKind::Mark => {
            output.push_str("<mark>");
            for child in &node.children {
                render_node(child, output, options, ctx)?;
            }
            output.push_str("</mark>");
        }
        NodeKind::Superscript => {
            output.push_str("<sup>");
            for child in &node.children {
                render_node(child, output, options, ctx)?;
            }
            output.push_str("</sup>");
        }
        NodeKind::Subscript => {
            output.push_str("<sub>");
            for child in &node.children {
                render_node(child, output, options, ctx)?;
            }
            output.push_str("</sub>");
        }
        NodeKind::Link { url, title } => {
            output.push_str("<a href=\"");
            output.push_str(&escape_html(url));
            output.push('"');
            if let Some(t) = title {
                output.push_str(" title=\"");
                output.push_str(&escape_html(t));
                output.push('"');
            }
            output.push('>');
            for child in &node.children {
                render_node(child, output, options, ctx)?;
            }
            output.push_str("</a>");
        }
        NodeKind::PlatformMention {
            username,
            platform,
            display,
        } => {
            let label = display.as_deref().unwrap_or(username);
            let platform_key = platform.trim().to_ascii_lowercase();

            if let Some(url) = plarform_mentions::profile_url(&platform_key, username) {
                output.push_str("<a class=\"marco-mention mention-");
                output.push_str(&escape_html(&platform_key));
                output.push_str("\" href=\"");
                output.push_str(&escape_html(&url));
                output.push_str("\">");
                output.push_str(&escape_html(label));
                output.push_str("</a>");
            } else {
                output.push_str("<span class=\"marco-mention mention-unknown\">");
                output.push_str(&escape_html(label));
                output.push_str("</span>");
            }
        }
        NodeKind::LinkReference { suffix, .. } => {
            // Reference links should normally be resolved during parsing.
            // If a reference is missing, or a caller bypasses the resolver,
            // render the original source-ish form as literal text.
            output.push('[');
            for child in &node.children {
                render_node(child, output, options, ctx)?;
            }
            output.push(']');
            output.push_str(&escape_html(suffix));
        }
        NodeKind::FootnoteReference { label } => {
            if !ctx.footnote_defs.contains_key(label) {
                output.push_str("[^");
                output.push_str(&escape_html(label));
                output.push(']');
                // Missing definition: keep the literal source form.
                return Ok(());
            }

            let n = match ctx.footnote_numbers.get(label) {
                Some(n) => *n,
                None => {
                    let next = ctx.footnote_order.len() + 1;
                    ctx.footnote_order.push(label.clone());
                    ctx.footnote_numbers.insert(label.clone(), next);
                    next
                }
            };

            let count = ctx.footnote_ref_counts.entry(label.clone()).or_insert(0);
            *count += 1;
            let ref_id = if *count == 1 {
                format!("fnref{}", n)
            } else {
                format!("fnref{}-{}", n, *count)
            };

            output.push_str("<sup class=\"footnote-ref\"><a href=\"#fn");
            output.push_str(&n.to_string());
            output.push_str("\" id=\"");
            output.push_str(&escape_html(&ref_id));
            output.push_str("\">");
            output.push_str(&n.to_string());
            output.push_str("</a></sup>");
        }
        NodeKind::Image { url, alt } => {
            output.push_str("<img src=\"");
            output.push_str(&escape_html(url));
            output.push_str("\" alt=\"");
            output.push_str(&escape_html(alt));
            output.push_str("\" />");
        }
        NodeKind::InlineHtml(html) => {
            // Pass through inline HTML directly (no escaping)
            output.push_str(html);
        }
        NodeKind::HardBreak => {
            // Hard line break: <br />
            output.push_str("<br />\n");
        }
        NodeKind::SoftBreak => {
            // Soft line break: rendered as single space (or newline in some contexts)
            output.push('\n');
        }
        NodeKind::List {
            ordered,
            start,
            tight,
        } => {
            // Render list opening tag
            if *ordered {
                output.push_str("<ol");
                if let Some(num) = start {
                    if *num != 1 {
                        output.push_str(&format!(" start=\"{}\"", num));
                    }
                }
                output.push_str(">\n");
            } else {
                output.push_str("<ul>\n");
            }

            // Render list items
            for child in &node.children {
                render_list_item(child, output, *tight, options, ctx)?;
            }

            // Render list closing tag
            if *ordered {
                output.push_str("</ol>\n");
            } else {
                output.push_str("</ul>\n");
            }
        }
        NodeKind::DefinitionList => {
            output.push_str("<dl>\n");
            for child in &node.children {
                render_node(child, output, options, ctx)?;
            }
            output.push_str("</dl>\n");
        }
        NodeKind::DefinitionTerm => {
            output.push_str("<dt>");
            for child in &node.children {
                render_node(child, output, options, ctx)?;
            }
            output.push_str("</dt>\n");
        }
        NodeKind::DefinitionDescription => {
            output.push_str("<dd>\n");
            for child in &node.children {
                render_node(child, output, options, ctx)?;
            }
            output.push_str("</dd>\n");
        }
        NodeKind::ListItem => {
            // This should only be called via render_list_item
            log::warn!("ListItem rendered outside of List context");
            output.push_str("<li>");
            for child in &node.children {
                render_node(child, output, options, ctx)?;
            }
            output.push_str("</li>\n");
        }
        NodeKind::TaskCheckbox { .. } => {
            // This should only be called via render_list_item (as a ListItem child).
            log::warn!("TaskCheckbox rendered outside of ListItem context");
        }
        NodeKind::TaskCheckboxInline { checked } => {
            // Inline checkbox marker (e.g. paragraph starting with `[ ]` / `[x]`).
            render_task_checkbox_icon(output, *checked);
        }
    }

    Ok(())
}

fn admonition_presentation(kind: &AdmonitionKind) -> (&'static str, &'static str, &'static str) {
    // Icons use `stroke="currentColor"` so theme CSS can color them by setting
    // `color` on `.markdown-alert-title`.
    match kind {
        AdmonitionKind::Note => (
            "note",
            "Note",
            concat!(
                r#"<svg xmlns=""#,
                "http",
                r#"://www.w3.org/2000/svg"#,
                r#"" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" focusable="false" aria-hidden="true"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M3 12a9 9 0 1 0 18 0a9 9 0 0 0 -18 0" /><path d="M12 9h.01" /><path d="M11 12h1v4h1" /></svg>"#,
            ),
        ),
        AdmonitionKind::Tip => (
            "tip",
            "Tip",
            concat!(
                r#"<svg xmlns=""#,
                "http",
                r#"://www.w3.org/2000/svg"#,
                r#"" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" focusable="false" aria-hidden="true"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M15.02 19.52c-2.341 .736 -5 .606 -7.32 -.52l-4.7 1l1.3 -3.9c-2.324 -3.437 -1.426 -7.872 2.1 -10.374c3.526 -2.501 8.59 -2.296 11.845 .48c1.649 1.407 2.575 3.253 2.742 5.152" /><path d="M19 22v.01" /><path d="M19 19a2.003 2.003 0 0 0 .914 -3.782a1.98 1.98 0 0 0 -2.414 .483" /></svg>"#,
            ),
        ),
        AdmonitionKind::Important => (
            "important",
            "Important",
            concat!(
                r#"<svg xmlns=""#,
                "http",
                r#"://www.w3.org/2000/svg"#,
                r#"" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" focusable="false" aria-hidden="true"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M8 9h8" /><path d="M8 13h6" /><path d="M15 18l-3 3l-3 -3h-3a3 3 0 0 1 -3 -3v-8a3 3 0 0 1 3 -3h12a3 3 0 0 1 3 3v5.5" /><path d="M19 16v3" /><path d="M19 22v.01" /></svg>"#,
            ),
        ),
        AdmonitionKind::Warning => (
            "warning",
            "Warning",
            concat!(
                r#"<svg xmlns=""#,
                "http",
                r#"://www.w3.org/2000/svg"#,
                r#"" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" focusable="false" aria-hidden="true"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M10.363 3.591l-8.106 13.534a1.914 1.914 0 0 0 1.636 2.871h16.214a1.914 1.914 0 0 0 1.636 -2.87l-8.106 -13.536a1.914 1.914 0 0 0 -3.274 0" /><path d="M12 9h.01" /><path d="M11 12h1v4h1" /></svg>"#,
            ),
        ),
        AdmonitionKind::Caution => (
            "caution",
            "Caution",
            concat!(
                r#"<svg xmlns=""#,
                "http",
                r#"://www.w3.org/2000/svg"#,
                r#"" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" focusable="false" aria-hidden="true"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M19.875 6.27c.7 .398 1.13 1.143 1.125 1.948v7.284c0 .809 -.443 1.555 -1.158 1.948l-6.75 4.27a2.269 2.269 0 0 1 -2.184 0l-6.75 -4.27a2.225 2.225 0 0 1 -1.158 -1.948v-7.285c0 -.809 .443 -1.554 1.158 -1.947l6.75 -3.98a2.33 2.33 0 0 1 2.25 0l6.75 3.98h-.033" /><path d="M12 8v4" /><path d="M12 16h.01" /></svg>"#,
            ),
        ),
    }
}

fn render_tab_group(
    node: &Node,
    output: &mut String,
    options: &RenderOptions,
    ctx: &mut RenderContext<'_>,
) -> Result<()> {
    // Assign a stable sequential id for this render pass.
    let group_id = ctx.tab_group_counter;
    ctx.tab_group_counter = ctx.tab_group_counter.saturating_add(1);

    // Collect tab items in order.
    let mut items: Vec<(&str, &Node)> = Vec::new();
    for child in &node.children {
        if let NodeKind::TabItem { title } = &child.kind {
            items.push((title.as_str(), child));
        } else {
            log::warn!("Unexpected child inside TabGroup: {:?}", child.kind);
        }
    }

    if items.is_empty() {
        return Ok(());
    }

    output.push_str("<div class=\"marco-tabs\">\n");

    // Radio inputs must come before the tablist/panels so generic nth-of-type CSS rules work.
    for (i, (title, _item_node)) in items.iter().enumerate() {
        output.push_str("<input class=\"marco-tabs__radio\" type=\"radio\" name=\"marco-tabs-");
        output.push_str(&group_id.to_string());
        output.push_str("\" id=\"marco-tabs-");
        output.push_str(&group_id.to_string());
        output.push('-');
        output.push_str(&i.to_string());
        output.push_str("\" aria-label=\"");
        output.push_str(&escape_html(title));
        output.push('"');
        if i == 0 {
            output.push_str(" checked");
        }
        output.push_str(" />\n");
    }

    output.push_str("<div class=\"marco-tabs__tablist\">\n");
    for (i, (title, _item_node)) in items.iter().enumerate() {
        output.push_str("<label class=\"marco-tabs__tab\" for=\"marco-tabs-");
        output.push_str(&group_id.to_string());
        output.push('-');
        output.push_str(&i.to_string());
        output.push_str("\">");
        output.push_str(&escape_html(title));
        output.push_str("</label>\n");
    }
    output.push_str("</div>\n");

    output.push_str("<div class=\"marco-tabs__panels\">\n");
    for &(_title, item_node) in items.iter() {
        output.push_str("<div class=\"marco-tabs__panel\">\n");
        for panel_child in &item_node.children {
            render_node(panel_child, output, options, ctx)?;
        }
        output.push_str("</div>\n");
    }
    output.push_str("</div>\n");

    output.push_str("</div>\n");
    Ok(())
}

fn render_slider_deck(
    node: &Node,
    output: &mut String,
    options: &RenderOptions,
    ctx: &mut RenderContext<'_>,
) -> Result<()> {
    let timer_seconds = match &node.kind {
        NodeKind::SliderDeck { timer_seconds } => *timer_seconds,
        other => {
            log::warn!(
                "render_slider_deck called with non SliderDeck node: {:?}",
                other
            );
            return Ok(());
        }
    };

    // Assign a stable sequential id for this render pass.
    let deck_id = ctx.slider_deck_counter;
    ctx.slider_deck_counter = ctx.slider_deck_counter.saturating_add(1);

    // Collect slides in order.
    let mut slides: Vec<(bool, &Node)> = Vec::new();
    for child in &node.children {
        if let NodeKind::Slide { vertical } = &child.kind {
            slides.push((*vertical, child));
        } else {
            log::warn!("Unexpected child inside SliderDeck: {:?}", child.kind);
        }
    }

    if slides.is_empty() {
        return Ok(());
    }

    output.push_str("<div class=\"marco-sliders\" id=\"marco-sliders-");
    output.push_str(&deck_id.to_string());
    output.push('"');
    if let Some(seconds) = timer_seconds {
        output.push_str(" data-timer-seconds=\"");
        output.push_str(&seconds.to_string());
        output.push('"');
    }
    output.push('>');

    output.push_str("<div class=\"marco-sliders__viewport\">");
    for (i, (vertical, slide_node)) in slides.iter().enumerate() {
        output.push_str("<section class=\"marco-sliders__slide");
        if i == 0 {
            output.push_str(" is-active");
        }
        output.push_str("\" data-slide-index=\"");
        output.push_str(&i.to_string());
        output.push('"');
        if *vertical {
            output.push_str(" data-vertical=\"true\"");
        }
        output.push_str(">\n");
        for child in &slide_node.children {
            render_node(child, output, options, ctx)?;
        }
        output.push_str("</section>\n");
    }
    output.push_str("</div>");

    // Controls: prev / play-pause / next
    output.push_str("<div class=\"marco-sliders__controls\" aria-label=\"Slideshow controls\">");

    output.push_str(
        "<button class=\"marco-sliders__btn marco-sliders__btn--prev\" type=\"button\" data-action=\"prev\" aria-label=\"Previous slide\">",
    );
    output.push_str(SLIDER_ARROW_LEFT_SVG);
    output.push_str("</button>");

    output.push_str(
        "<button class=\"marco-sliders__btn marco-sliders__btn--toggle\" type=\"button\" data-action=\"toggle\" aria-label=\"Toggle autoplay\">",
    );
    output.push_str("<span class=\"marco-sliders__icon marco-sliders__icon--play\">");
    output.push_str(SLIDER_PLAY_SVG);
    output.push_str("</span>");
    output.push_str("<span class=\"marco-sliders__icon marco-sliders__icon--pause\">");
    output.push_str(SLIDER_PAUSE_SVG);
    output.push_str("</span>");
    output.push_str("</button>");

    output.push_str(
        "<button class=\"marco-sliders__btn marco-sliders__btn--next\" type=\"button\" data-action=\"next\" aria-label=\"Next slide\">",
    );
    output.push_str(SLIDER_ARROW_RIGHT_SVG);
    output.push_str("</button>");

    output.push_str("</div>");

    // Dots navigation
    output.push_str(
        "<div class=\"marco-sliders__dots\" role=\"tablist\" aria-label=\"Slideshow navigation\">",
    );
    for i in 0..slides.len() {
        output.push_str("<button class=\"marco-sliders__dot");
        if i == 0 {
            output.push_str(" is-active");
        }
        output.push_str("\" type=\"button\" data-action=\"goto\" data-index=\"");
        output.push_str(&i.to_string());
        output.push_str("\" aria-label=\"Go to slide ");
        output.push_str(&(i + 1).to_string());
        output.push('"');
        if i == 0 {
            output.push_str(" aria-selected=\"true\"");
        }
        output.push_str(">\n");
        output
            .push_str("<span class=\"marco-sliders__dot-icon marco-sliders__dot-icon--inactive\">");
        output.push_str(SLIDER_DOT_INACTIVE_SVG);
        output.push_str("</span>");
        output.push_str("<span class=\"marco-sliders__dot-icon marco-sliders__dot-icon--active\">");
        output.push_str(SLIDER_DOT_ACTIVE_SVG);
        output.push_str("</span>");
        output.push_str("</button>");
    }
    output.push_str("</div>");

    output.push_str("</div>\n");
    Ok(())
}

fn render_task_checkbox_icon(output: &mut String, checked: bool) {
    // We keep the SVG strokes as `currentColor` and let CSS decide:
    // - unchecked box: inherited text color
    // - checked box: theme primary
    // - checkmark: theme accent
    if checked {
        output.push_str(
            r#"<span class="task-list-item-checkbox marco-task-checkbox checked" aria-hidden="true">"#,
        );
        output.push_str(
            concat!(
                r#"<svg xmlns=""#,
                "http",
                r#"://www.w3.org/2000/svg"#,
                r#"" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.25" stroke-linecap="round" stroke-linejoin="round" class="marco-task-icon"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path class="marco-task-check" style="stroke: var(--marco-task-accent); stroke-width: 2.0;" d="M9 11l3 3l8 -8" /><path class="marco-task-box" style="stroke: var(--marco-task-primary);" d="M3 5a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v14a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-14" /></svg>"#,
            ),
        );
        output.push_str("</span>");
    } else {
        output.push_str(
            r#"<span class="task-list-item-checkbox marco-task-checkbox unchecked" aria-hidden="true">"#,
        );
        output.push_str(
            concat!(
                r#"<svg xmlns=""#,
                "http",
                r#"://www.w3.org/2000/svg"#,
                r#"" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.25" stroke-linecap="round" stroke-linejoin="round" class="marco-task-icon"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path class="marco-task-box" d="M3 5a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v14a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-14" /></svg>"#,
            ),
        );
        output.push_str("</span>");
    }
}

fn render_table(
    node: &Node,
    output: &mut String,
    options: &RenderOptions,
    ctx: &mut RenderContext<'_>,
) -> Result<()> {
    output.push_str("<table>\n");

    let mut header_rows: Vec<&Node> = Vec::new();
    let mut body_rows: Vec<&Node> = Vec::new();

    for row in &node.children {
        match row.kind {
            NodeKind::TableRow { header: true } => header_rows.push(row),
            NodeKind::TableRow { header: false } => body_rows.push(row),
            _ => {
                log::warn!("Unexpected child inside Table: {:?}", row.kind);
            }
        }
    }

    if !header_rows.is_empty() {
        output.push_str("<thead>\n");
        for row in header_rows {
            render_table_row(row, output, options, ctx)?;
            output.push('\n');
        }
        output.push_str("</thead>\n");
    }

    if !body_rows.is_empty() {
        output.push_str("<tbody>\n");
        for row in body_rows {
            render_table_row(row, output, options, ctx)?;
            output.push('\n');
        }
        output.push_str("</tbody>\n");
    }

    output.push_str("</table>\n");
    Ok(())
}

fn render_table_row(
    node: &Node,
    output: &mut String,
    options: &RenderOptions,
    ctx: &mut RenderContext<'_>,
) -> Result<()> {
    output.push_str("<tr>");
    for cell in &node.children {
        render_table_cell(cell, output, options, ctx)?;
    }
    output.push_str("</tr>");
    Ok(())
}

fn render_table_cell(
    node: &Node,
    output: &mut String,
    options: &RenderOptions,
    ctx: &mut RenderContext<'_>,
) -> Result<()> {
    let (is_header, alignment) = match &node.kind {
        NodeKind::TableCell { header, alignment } => (*header, *alignment),
        _ => {
            log::warn!("Unexpected child inside TableRow: {:?}", node.kind);
            (false, crate::parser::ast::TableAlignment::None)
        }
    };

    let tag = if is_header { "th" } else { "td" };
    output.push('<');
    output.push_str(tag);

    if let Some(style_value) = alignment_to_css(alignment) {
        output.push_str(" style=\"");
        output.push_str(style_value);
        output.push('"');
    }

    output.push('>');
    for child in &node.children {
        render_node(child, output, options, ctx)?;
    }
    output.push_str("</");
    output.push_str(tag);
    output.push('>');
    Ok(())
}

fn alignment_to_css(alignment: crate::parser::ast::TableAlignment) -> Option<&'static str> {
    match alignment {
        crate::parser::ast::TableAlignment::None => None,
        crate::parser::ast::TableAlignment::Left => Some("text-align: left;"),
        crate::parser::ast::TableAlignment::Center => Some("text-align: center;"),
        crate::parser::ast::TableAlignment::Right => Some("text-align: right;"),
    }
}

// Render a list item with proper tight/loose handling
fn render_list_item(
    node: &Node,
    output: &mut String,
    tight: bool,
    options: &RenderOptions,
    ctx: &mut RenderContext<'_>,
) -> Result<()> {
    let task_checked = match node.children.first().map(|n| &n.kind) {
        Some(NodeKind::TaskCheckbox { checked }) => Some(*checked),
        _ => None,
    };

    if let Some(checked) = task_checked {
        if checked {
            output.push_str("<li class=\"task-list-item task-list-item-checked\">");
        } else {
            output.push_str("<li class=\"task-list-item\">");
        }
    } else {
        output.push_str("<li>");
    }

    if tight {
        // Tight list: paragraph content is inlined (no <p> wrapper), so we can
        // safely emit the checkbox icon at the start of the list item.
        if let Some(checked) = task_checked {
            render_task_checkbox_icon(output, checked);
        }

        // Tight list: don't wrap paragraphs in <p> tags
        for child in &node.children {
            if matches!(child.kind, NodeKind::TaskCheckbox { .. }) {
                continue;
            }
            match &child.kind {
                NodeKind::Paragraph => {
                    // Render paragraph children directly without <p> wrapper
                    for grandchild in &child.children {
                        render_node(grandchild, output, options, ctx)?;
                    }
                }
                _ => {
                    // Other block elements render normally
                    render_node(child, output, options, ctx)?;
                }
            }
        }
    } else {
        // Loose list: keep paragraphs wrapped in <p>, but for task list items we
        // want the checkbox icon to sit inline with the first paragraph's text.
        let mut checkbox_emitted = false;

        for child in &node.children {
            if matches!(child.kind, NodeKind::TaskCheckbox { .. }) {
                continue;
            }

            // Emit the checkbox exactly once, either inside the first paragraph
            // or as a standalone prefix if the first block isn't a paragraph.
            if let Some(checked) = task_checked {
                if !checkbox_emitted {
                    match &child.kind {
                        NodeKind::Paragraph => {
                            output.push_str("<p>");
                            render_task_checkbox_icon(output, checked);
                            for grandchild in &child.children {
                                render_node(grandchild, output, options, ctx)?;
                            }
                            output.push_str("</p>");
                            checkbox_emitted = true;
                            continue;
                        }
                        _ => {
                            render_task_checkbox_icon(output, checked);
                            checkbox_emitted = true;
                            // fall through and render this child normally
                        }
                    }
                }
            }

            render_node(child, output, options, ctx)?;
        }
    }

    output.push_str("</li>\n");
    Ok(())
}

// Escape HTML special characters to prevent XSS and ensure proper display
fn escape_html(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#39;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::TableAlignment;
    use crate::parser::{Document, Node, NodeKind};

    #[test]
    fn smoke_test_escape_html_basic() {
        let input = "Hello <world> & \"friends\"";
        let expected = "Hello &lt;world&gt; &amp; &quot;friends&quot;";
        assert_eq!(escape_html(input), expected);
    }

    #[test]
    fn smoke_test_escape_html_script_tag() {
        let input = "<script>alert('XSS')</script>";
        let expected = "&lt;script&gt;alert(&#39;XSS&#39;)&lt;/script&gt;";
        assert_eq!(escape_html(input), expected);
    }

    #[test]
    fn smoke_test_render_heading_h1() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Heading {
                    level: 1,
                    text: "Hello World".to_string(),
                    id: None,
                },
                span: None,
                children: vec![],
            }],
            ..Default::default()
        };
        let options = RenderOptions::default();
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(result, "<h1>Hello World</h1>\n");
    }

    #[test]
    fn smoke_test_render_heading_with_html() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Heading {
                    level: 2,
                    text: "Code <example> & test".to_string(),
                    id: None,
                },
                span: None,
                children: vec![],
            }],
            ..Default::default()
        };
        let options = RenderOptions::default();
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(result, "<h2>Code &lt;example&gt; &amp; test</h2>\n");
    }

    #[test]
    fn smoke_test_render_paragraph_with_text() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![Node {
                    kind: NodeKind::Text("This is a paragraph.".to_string()),
                    span: None,
                    children: vec![],
                }],
            }],
            ..Default::default()
        };
        let options = RenderOptions::default();
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(result, "<p>This is a paragraph.</p>\n");
    }

    #[test]
    fn smoke_test_render_code_block_without_language() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::CodeBlock {
                    language: None,
                    code: "fn main() {\n    println!(\"Hello\");\n}".to_string(),
                },
                span: None,
                children: vec![],
            }],
            ..Default::default()
        };
        let options = RenderOptions::default();
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(
            result,
            "<pre><code>fn main() {\n    println!(&quot;Hello&quot;);\n}</code></pre>\n"
        );
    }

    #[test]
    fn smoke_test_render_code_block_with_language() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::CodeBlock {
                    language: Some("rust".to_string()),
                    code: "let x = 42;".to_string(),
                },
                span: None,
                children: vec![],
            }],
            ..Default::default()
        };
        let options = RenderOptions {
            syntax_highlighting: false,
            ..RenderOptions::default()
        };
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(
            result,
            "<pre data-language=\"Rust\"><code class=\"language-rust\">let x = 42;</code></pre>\n"
        );
    }

    #[test]
    fn smoke_test_render_code_block_escapes_html() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::CodeBlock {
                    language: Some("html".to_string()),
                    code: "<div>Test & verify</div>".to_string(),
                },
                span: None,
                children: vec![],
            }],
            ..Default::default()
        };
        let options = RenderOptions {
            syntax_highlighting: false,
            ..RenderOptions::default()
        };
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(
            result,
            "<pre data-language=\"HTML\"><code class=\"language-html\">&lt;div&gt;Test &amp; verify&lt;/div&gt;</code></pre>\n"
        );
    }

    #[test]
    fn smoke_test_render_code_span() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![
                    Node {
                        kind: NodeKind::Text("Use ".to_string()),
                        span: None,
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::CodeSpan("println!()".to_string()),
                        span: None,
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::Text(" for output.".to_string()),
                        span: None,
                        children: vec![],
                    },
                ],
            }],
            ..Default::default()
        };
        let options = RenderOptions::default();
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(result, "<p>Use <code>println!()</code> for output.</p>\n");
    }

    #[test]
    fn smoke_test_render_mixed_inlines() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Heading {
                        level: 1,
                        text: "Title".to_string(),
                        id: None,
                    },
                    span: None,
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Paragraph,
                    span: None,
                    children: vec![Node {
                        kind: NodeKind::Text("Some text.".to_string()),
                        span: None,
                        children: vec![],
                    }],
                },
                Node {
                    kind: NodeKind::CodeBlock {
                        language: Some("python".to_string()),
                        code: "print('hello')".to_string(),
                    },
                    span: None,
                    children: vec![],
                },
            ],
            ..Default::default()
        };
        let options = RenderOptions {
            syntax_highlighting: false,
            ..RenderOptions::default()
        };
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(
            result,
            "<h1>Title</h1>\n<p>Some text.</p>\n<pre data-language=\"Python\"><code class=\"language-python\">print(&#39;hello&#39;)</code></pre>\n"
        );
    }

    #[test]
    fn smoke_test_render_strong_emphasis() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![Node {
                    kind: NodeKind::StrongEmphasis,
                    span: None,
                    children: vec![Node {
                        kind: NodeKind::Text("bold+italic".to_string()),
                        span: None,
                        children: vec![],
                    }],
                }],
            }],
            ..Default::default()
        };

        let options = RenderOptions::default();
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(result, "<p><strong><em>bold+italic</em></strong></p>\n");
    }

    #[test]
    fn smoke_test_render_strike_mark_sup_sub() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![
                    Node {
                        kind: NodeKind::Strikethrough,
                        span: None,
                        children: vec![Node {
                            kind: NodeKind::Text("del".to_string()),
                            span: None,
                            children: vec![],
                        }],
                    },
                    Node {
                        kind: NodeKind::Text(" ".to_string()),
                        span: None,
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::Mark,
                        span: None,
                        children: vec![Node {
                            kind: NodeKind::Text("mark".to_string()),
                            span: None,
                            children: vec![],
                        }],
                    },
                    Node {
                        kind: NodeKind::Text(" ".to_string()),
                        span: None,
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::Superscript,
                        span: None,
                        children: vec![Node {
                            kind: NodeKind::Text("sup".to_string()),
                            span: None,
                            children: vec![],
                        }],
                    },
                    Node {
                        kind: NodeKind::Text(" ".to_string()),
                        span: None,
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::Subscript,
                        span: None,
                        children: vec![Node {
                            kind: NodeKind::Text("sub".to_string()),
                            span: None,
                            children: vec![],
                        }],
                    },
                ],
            }],
            ..Default::default()
        };

        let options = RenderOptions::default();
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(
            result,
            "<p><del>del</del> <mark>mark</mark> <sup>sup</sup> <sub>sub</sub></p>\n"
        );
    }

    #[test]
    fn smoke_test_render_table_with_alignment() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Table {
                    alignments: vec![TableAlignment::Left, TableAlignment::Center],
                },
                span: None,
                children: vec![
                    Node {
                        kind: NodeKind::TableRow { header: true },
                        span: None,
                        children: vec![
                            Node {
                                kind: NodeKind::TableCell {
                                    header: true,
                                    alignment: TableAlignment::Left,
                                },
                                span: None,
                                children: vec![Node {
                                    kind: NodeKind::Text("h1".to_string()),
                                    span: None,
                                    children: vec![],
                                }],
                            },
                            Node {
                                kind: NodeKind::TableCell {
                                    header: true,
                                    alignment: TableAlignment::Center,
                                },
                                span: None,
                                children: vec![Node {
                                    kind: NodeKind::Text("h2".to_string()),
                                    span: None,
                                    children: vec![],
                                }],
                            },
                        ],
                    },
                    Node {
                        kind: NodeKind::TableRow { header: false },
                        span: None,
                        children: vec![
                            Node {
                                kind: NodeKind::TableCell {
                                    header: false,
                                    alignment: TableAlignment::Left,
                                },
                                span: None,
                                children: vec![Node {
                                    kind: NodeKind::Text("c1".to_string()),
                                    span: None,
                                    children: vec![],
                                }],
                            },
                            Node {
                                kind: NodeKind::TableCell {
                                    header: false,
                                    alignment: TableAlignment::Center,
                                },
                                span: None,
                                children: vec![Node {
                                    kind: NodeKind::Text("c2".to_string()),
                                    span: None,
                                    children: vec![],
                                }],
                            },
                        ],
                    },
                ],
            }],
            ..Default::default()
        };

        let options = RenderOptions::default();
        let result = render_html(&doc, &options).expect("render failed");

        assert!(result.contains("<table>"));
        assert!(result.contains("<thead>"));
        assert!(result.contains("<tbody>"));
        assert!(result.contains("<th style=\"text-align: left;\">h1</th>"));
        assert!(result.contains("<th style=\"text-align: center;\">h2</th>"));
        assert!(result.contains("<td style=\"text-align: left;\">c1</td>"));
        assert!(result.contains("<td style=\"text-align: center;\">c2</td>"));
    }
}
