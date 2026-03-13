#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarIcon {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    Paragraph,
    ChevronDown,
    Bold,
    Italic,
    Code,
    Strikethrough,
    Highlight,
    BulletList,
    NumberedList,
    Checkbox,
    CreateTable,
    Link,
    LinkReference,
    Blockquote,
    CodeBlock,
    TaskList,
    Image,
    Table,
    ThematicBreak,
    Undo,
    Redo,
    Functions,
    HeadingId,
    Admonition,
    Footnote,
    InlineFootnote,
    Superscript,
    Subscript,
    Emoji,
    Mention,
    TabBlock,
    Slideshow,
    Math,
    Mermaid,
    Issue,
    Terminal,
    Toc,
    GutterOn,
    GutterOff,
}

/// Inner SVG path data extracted from 24×24 icons, for building composite buttons.
///
/// These contain only the visible `<path>` elements (without the SVG wrapper and
/// the `M0 0h24v24H0z` clearing rect).
pub mod composite_paths {
    pub const PARAGRAPH: &str = "\
        <path d='M13 4v16'/>\
        <path d='M17 4v16'/>\
        <path d='M19 4h-9a4 4 0 0 0 0 8h7'/>";

    pub const BLOCKQUOTE: &str = "\
        <path d='M6 15h15'/>\
        <path d='M21 19h-15'/>\
        <path d='M15 11h6'/>\
        <path d='M21 7h-6'/>\
        <path d='M9 9h1a1 1 0 1 1 -1 1v-2.5a2 2 0 0 1 2 -2'/>\
        <path d='M3 9h1a1 1 0 1 1 -1 1v-2.5a2 2 0 0 1 2 -2'/>";

    pub const ADMONITION: &str = "\
        <path d='M7 20l10 0'/>\
        <path d='M6 6l6 -1l6 1'/>\
        <path d='M12 3l0 17'/>\
        <path d='M9 12l-3 -6l-3 6a3 3 0 0 0 6 0'/>\
        <path d='M21 12l-3 -6l-3 6a3 3 0 0 0 6 0'/>";

    pub const MENTION: &str = "\
        <path d='M16 12a4 4 0 1 0 -4 4'/>\
        <path d='M16 12v1a2 2 0 1 0 4 0v-1a8 8 0 1 0 -4 6.93'/>";

    pub const TABLE: &str = "\
        <path d='M3 5a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v14a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-14'/>\
        <path d='M3 10h18'/>\
        <path d='M10 3v18'/>";

    pub const TEXT_INLINE: &str = "\
        <path d='M19 10h-14'/>\
        <path d='M5 6h14'/>\
        <path d='M14 14h-9'/>\
        <path d='M5 18h6'/>\
        <path d='M18 15v6'/>\
        <path d='M15 18h6'/>";

    pub const LISTS: &str = "\
        <path d='M9 6l11 0'/>\
        <path d='M9 12l11 0'/>\
        <path d='M9 18l11 0'/>\
        <path d='M5 6l0 .01'/>\
        <path d='M5 12l0 .01'/>\
        <path d='M5 18l0 .01'/>";

    pub const INLINE_ITEMS: &str = "\
        <path d='M6 19a2 2 0 0 1 -2 -2v-4l-1 -1l1 -1v-4a2 2 0 0 1 2 -2'/>\
        <path d='M12 11.875l3 -1.687'/>\
        <path d='M12 11.875v3.375'/>\
        <path d='M12 11.875l-3 -1.687'/>\
        <path d='M12 11.875l3 1.688'/>\
        <path d='M12 8.5v3.375'/>\
        <path d='M12 11.875l-3 1.688'/>\
        <path d='M18 19a2 2 0 0 0 2 -2v-4l1 -1l-1 -1v-4a2 2 0 0 0 -2 -2'/>";

    pub const BLOCK_ITEMS: &str = "\
        <path d='M4 5a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v4a1 1 0 0 1 -1 1h-4a1 1 0 0 1 -1 -1l0 -4'/>\
        <path d='M4 15a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v4a1 1 0 0 1 -1 1h-4a1 1 0 0 1 -1 -1l0 -4'/>\
        <path d='M14 15a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v4a1 1 0 0 1 -1 1h-4a1 1 0 0 1 -1 -1l0 -4'/>\
        <path d='M14 7l6 0'/>\
        <path d='M17 4l0 6'/>";

    pub const FUNCTIONS: &str = "\
        <path d='M12.5 21h-7.5a2 2 0 0 1 -2 -2v-14a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v7.5' />
        <path d='M16 19h6' />
        <path d='M19 16v6' />
        <path d='M7 15V8L10 12L13 8V15' />";
}

/// Output from [`toolbar_composite_button_svg`] with SVG markup and layout dimensions.
pub struct CompositeButtonSvg {
    /// Complete SVG string with `currentColor` and `invertedColor` placeholders.
    pub svg: String,
    /// ViewBox width (divide by 2 to get approximate display width at 12 px height).
    pub viewbox_width: f64,
}

/// Generates a transparent composite toolbar button SVG with inline icons.
///
/// The output contains:
/// - No background (transparent)
/// - A left-aligned icon (stroke-only, using `currentColor`)
/// - A text label (using `currentColor`)
/// - A small right-aligned ▼ chevron (using `currentColor`)
///
/// # Color placeholders
/// The caller replaces these before rendering:
/// - `currentColor` → icon, text, and chevron color (matches other toolbar icons)
///
/// # Arguments
/// - `icon_paths` - raw `<path>` elements from a 24×24 source icon (see [`composite_paths`])
/// - `label` - text shown inside the button
pub fn toolbar_composite_button_svg(icon_paths: &str, label: &str) -> CompositeButtonSvg {
    const VB_H: f64 = 24.0;
    const PAD: f64 = 0.0;
    const ICON_W: f64 = 24.0; // Same as toolbar SVG icons (24×24, no scaling)
    const GAP: f64 = 0.0;
    const CHEVRON_W: f64 = 6.0;
    const CHAR_W: f64 = 6.5; // approximate glyph advance at font-size 11
    const MIN_VB_W: f64 = 100.0;
    const CHEVRON_GAP: f64 = 0.0;

    let text_w = label.len() as f64 * CHAR_W;
    let content_w = PAD + ICON_W + GAP + text_w + CHEVRON_GAP + CHEVRON_W + PAD;
    let vb_w = if label.is_empty() {
        content_w.ceil()
    } else {
        content_w.max(MIN_VB_W).ceil()
    };

    let text_x = PAD + ICON_W + GAP;
    let chevron_cx = vb_w - PAD - (CHEVRON_W / 2.0);

    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 {vbw:.0} {vbh:.0}' fill='none'>\
         <g stroke='currentColor' stroke-width='1' \
            stroke-linecap='round' stroke-linejoin='round'>\
         {paths}</g>\
         <text x='{tx:.0}' y='16' font-size='11' font-family='sans-serif' \
               fill='currentColor'>{label}</text>\
         <path d='M{cx:.1} 14l3 -4l-6 0z' fill='currentColor'/>\
         </svg>",
        vbw = vb_w,
        vbh = VB_H,
        paths = icon_paths,
        tx = text_x,
        label = label,
        cx = chevron_cx,
    );

    CompositeButtonSvg {
        svg,
        viewbox_width: vb_w,
    }
}

/// Returns inline SVG markup for toolbar icons.
///
/// The SVGs are intentionally stored here to keep toolbar icon assets centralized.
pub fn toolbar_icon_svg(icon: ToolbarIcon) -> &'static str {
    match icon {
        ToolbarIcon::H1 => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-h-1'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M19 18v-8l-2 2' /><path d='M4 6v12' /><path d='M12 6v12' /><path d='M11 18h2' /><path d='M3 18h2' /><path d='M4 12h8' /><path d='M3 6h2' /><path d='M11 6h2' /></svg>"#
        }
        ToolbarIcon::H2 => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-h-2'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M17 12a2 2 0 1 1 4 0c0 .591 -.417 1.318 -.816 1.858l-3.184 4.143l4 0' /><path d='M4 6v12' /><path d='M12 6v12' /><path d='M11 18h2' /><path d='M3 18h2' /><path d='M4 12h8' /><path d='M3 6h2' /><path d='M11 6h2' /></svg>"#
        }
        ToolbarIcon::H3 => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-h-3'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M19 14a2 2 0 1 0 -2 -2' /><path d='M17 16a2 2 0 1 0 2 -2' /><path d='M4 6v12' /><path d='M12 6v12' /><path d='M11 18h2' /><path d='M3 18h2' /><path d='M4 12h8' /><path d='M3 6h2' /><path d='M11 6h2' /></svg>"#
        }
        ToolbarIcon::H4 => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-h-4'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M20 18v-8l-4 6h5' /><path d='M4 6v12' /><path d='M12 6v12' /><path d='M11 18h2' /><path d='M3 18h2' /><path d='M4 12h8' /><path d='M3 6h2' /><path d='M11 6h2' /></svg>"#
        }
        ToolbarIcon::H5 => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-h-5'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M17 18h2a2 2 0 1 0 0 -4h-2v-4h4' /><path d='M4 6v12' /><path d='M12 6v12' /><path d='M11 18h2' /><path d='M3 18h2' /><path d='M4 12h8' /><path d='M3 6h2' /><path d='M11 6h2' /></svg>"#
        }
        ToolbarIcon::H6 => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-h-6'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M19 14a2 2 0 1 0 0 4a2 2 0 0 0 0 -4' /><path d='M21 12a2 2 0 1 0 -4 0v4' /><path d='M4 6v12' /><path d='M12 6v12' /><path d='M11 18h2' /><path d='M3 18h2' /><path d='M4 12h8' /><path d='M3 6h2' /><path d='M11 6h2' /></svg>"#
        }
        ToolbarIcon::Paragraph => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-paragraph'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M13 4v16'/><path d='M17 4v16'/><path d='M19 4h-9a4 4 0 0 0 0 8h7'/></svg>"#
        }
        ToolbarIcon::ChevronDown => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-chevron-down'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M6 9l6 6l6 -6'/></svg>"#
        }
        ToolbarIcon::Bold => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-bold'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M7 5h6a3.5 3.5 0 0 1 0 7h-6l0 -7' /><path d='M13 12h1a3.5 3.5 0 0 1 0 7h-7v-7' /></svg>"#
        }
        ToolbarIcon::Italic => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-italic'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M11 5l6 0' /><path d='M7 19l6 0' /><path d='M14 5l-4 14' /></svg>"#
        }
        ToolbarIcon::Code => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-code-dots'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M15 12h.01' /><path d='M12 12h.01' /><path d='M9 12h.01' /><path d='M6 19a2 2 0 0 1 -2 -2v-4l-1 -1l1 -1v-4a2 2 0 0 1 2 -2' /><path d='M18 19a2 2 0 0 0 2 -2v-4l1 -1l-1 -1v-4a2 2 0 0 0 -2 -2' /></svg>"#
        }
        ToolbarIcon::Strikethrough => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-strikethrough'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M5 12l14 0' /><path d='M16 6.5a4 2 0 0 0 -4 -1.5h-1a3.5 3.5 0 0 0 0 7h2a3.5 3.5 0 0 1 0 7h-1.5a4 2 0 0 1 -4 -1.5' /></svg>"#
        }
        ToolbarIcon::Highlight => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-highlight'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M3 19h4l10.5 -10.5a2.828 2.828 0 1 0 -4 -4l-10.5 10.5v4' /><path d='M12.5 5.5l4 4' /><path d='M4.5 13.5l4 4' /><path d='M21 15v4h-8l4 -4l4 0' /></svg>"#
        }
        ToolbarIcon::BulletList => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-list'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M9 6l11 0' /><path d='M9 12l11 0' /><path d='M9 18l11 0' /><path d='M5 6l0 .01' /><path d='M5 12l0 .01' /><path d='M5 18l0 .01' /></svg>"#
        }
        ToolbarIcon::NumberedList => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-list-numbers'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M11 6h9' /><path d='M11 12h9' /><path d='M12 18h8' /><path d='M4 16a2 2 0 1 1 4 0c0 .591 -.5 1 -1 1.5l-3 2.5h4' /><path d='M6 10v-6l-2 2' /></svg>"#
        }
        ToolbarIcon::Checkbox => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-list-details'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M13 5h8' /><path d='M13 9h5' /><path d='M13 15h8' /><path d='M13 19h5' /><path d='M3 5a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v4a1 1 0 0 1 -1 1h-4a1 1 0 0 1 -1 -1l0 -4' /><path d='M3 15a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v4a1 1 0 0 1 -1 1h-4a1 1 0 0 1 -1 -1l0 -4' /></svg>"#
        }
        ToolbarIcon::CreateTable => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-table-plus'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M12.5 21h-7.5a2 2 0 0 1 -2 -2v-14a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v7.5' /><path d='M3 10h18' /><path d='M10 3v18' /><path d='M16 19h6' /><path d='M19 16v6' /></svg>"#
        }
        ToolbarIcon::Link => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-link'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M9 15l6 -6'/><path d='M11 6l.463 -.536a5 5 0 0 1 7.071 7.072l-.534 .464'/><path d='M13 18l-.397 .534a5.068 5.068 0 0 1 -7.127 0a4.972 4.972 0 0 1 0 -7.071l.524 -.463'/></svg>"#
        }
        ToolbarIcon::LinkReference => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-layers-linked'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M19 8.268a2 2 0 0 1 1 1.732v8a2 2 0 0 1 -2 2h-8a2 2 0 0 1 -2 -2v-8a2 2 0 0 1 2 -2h3' /><path d='M5 15.734a2 2 0 0 1 -1 -1.734v-8a2 2 0 0 1 2 -2h8a2 2 0 0 1 2 2v8a2 2 0 0 1 -2 2h-3' /></svg>"#
        }
        ToolbarIcon::Blockquote => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-blockquote'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M6 15h15'/><path d='M21 19h-15'/><path d='M15 11h6'/><path d='M21 7h-6'/><path d='M9 9h1a1 1 0 1 1 -1 1v-2.5a2 2 0 0 1 2 -2'/><path d='M3 9h1a1 1 0 1 1 -1 1v-2.5a2 2 0 0 1 2 -2'/></svg>"#
        }
        ToolbarIcon::CodeBlock => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-braces'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M7 4a2 2 0 0 0 -2 2v3a2 3 0 0 1 -2 3a2 3 0 0 1 2 3v3a2 2 0 0 0 2 2'/><path d='M17 4a2 2 0 0 1 2 2v3a2 3 0 0 0 2 3a2 3 0 0 0 -2 3v3a2 2 0 0 1 -2 2'/><path d='M9 8l6 0'/><path d='M9 12l6 0'/><path d='M9 16l6 0'/></svg>"#
        }
        ToolbarIcon::TaskList => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-list-details'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M13 5h8' /><path d='M13 9h5' /><path d='M13 15h8' /><path d='M13 19h5' /><path d='M3 5a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v4a1 1 0 0 1 -1 1h-4a1 1 0 0 1 -1 -1l0 -4' /><path d='M3 15a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v4a1 1 0 0 1 -1 1h-4a1 1 0 0 1 -1 -1l0 -4' /></svg>"#
        }
        ToolbarIcon::Image => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-photo'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M15 8h.01'/><path d='M3 6a3 3 0 0 1 3 -3h12a3 3 0 0 1 3 3v12a3 3 0 0 1 -3 3h-12a3 3 0 0 1 -3 -3v-12'/><path d='M3 16l5 -5c.928 -.893 2.072 -.893 3 0l5 5'/><path d='M14 14l1 -1c.928 -.893 2.072 -.893 3 0l3 3'/></svg>"#
        }
        ToolbarIcon::Table => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-table'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M3 5a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v14a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-14'/><path d='M3 10h18'/><path d='M10 3v18'/></svg>"#
        }
        ToolbarIcon::ThematicBreak => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-separator'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M3 12l0 .01' /><path d='M7 12l10 0' /><path d='M21 12l0 .01' /></svg>"#
        }
        ToolbarIcon::Undo => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-arrow-back-up'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M9 14l-4 -4l4 -4'/><path d='M5 10h11a4 4 0 1 1 0 8h-1'/></svg>"#
        }
        ToolbarIcon::Redo => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-arrow-forward-up'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M15 14l4 -4l-4 -4'/><path d='M19 10h-11a4 4 0 1 0 0 8h1'/></svg>"#
        }
        ToolbarIcon::Functions => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-dots-circle-horizontal'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M5 12m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0'/><path d='M12 12m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0'/><path d='M19 12m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0'/><path d='M12 3a9 9 0 1 0 0 18a9 9 0 0 0 0 -18'/></svg>"#
        }
        ToolbarIcon::HeadingId => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-link-plus'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M9 15l6 -6'/><path d='M11 6l.463 -.536a5 5 0 0 1 7.071 7.072l-.534 .464'/><path d='M13 18l-.397 .534a5.068 5.068 0 0 1 -7.127 0a4.972 4.972 0 0 1 0 -7.071l.524 -.463'/><path d='M19 16v6'/><path d='M16 19h6'/></svg>"#
        }
        ToolbarIcon::Admonition => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-scale'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M7 20l10 0' /><path d='M6 6l6 -1l6 1' /><path d='M12 3l0 17' /><path d='M9 12l-3 -6l-3 6a3 3 0 0 0 6 0' /><path d='M21 12l-3 -6l-3 6a3 3 0 0 0 6 0' /></svg>"#
        }
        ToolbarIcon::Footnote => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none'><path d='M20.5 7.263a5 5 0 0 1-.607.475.75.75 0 0 1-.832-1.248c.764-.51 1.33-1.321 1.456-1.9A.75.75 0 0 1 22 4.75v6.5a.75.75 0 0 1-1.5 0zm-8.278 10.493a.88.88 0 0 1-.222-.61V7.91c0-.246.074-.468.237-.645l.003-.004a.84.84 0 0 1 .628-.26c.243 0 .457.085.622.262a.9.9 0 0 1 .245.647v2.84q.273-.286.625-.492l.001-.001a3 3 0 0 1 1.524-.396c1.004 0 1.829.38 2.45 1.135.62.752.916 1.739.916 2.935 0 1.2-.297 2.19-.916 2.941-.623.752-1.455 1.128-2.471 1.128a3.03 3.03 0 0 1-1.546-.396 3 3 0 0 1-.64-.508v.05a.84.84 0 0 1-.243.61.84.84 0 0 1-.61.244.8.8 0 0 1-.6-.24zm4.75-1.976c.335-.435.516-1.043.516-1.85 0-.801-.181-1.406-.516-1.842-.329-.427-.77-.64-1.35-.64-.55 0-.995.216-1.352.665-.354.446-.542 1.045-.542 1.818 0 .778.188 1.378.542 1.825.356.443.8.658 1.352.658.581 0 1.022-.212 1.35-.634M2 17.186c0 .235.093.438.27.594l.007.007c.18.147.399.213.64.213a.9.9 0 0 0 .55-.174c.156-.118.265-.287.336-.484l.88-2.39H8.56l.886 2.39v.002c.072.196.18.364.336.482.159.12.348.174.55.174a.96.96 0 0 0 .637-.217.76.76 0 0 0 .28-.597 1.3 1.3 0 0 0-.098-.446L7.71 7.774a1.24 1.24 0 0 0-.413-.572A1.1 1.1 0 0 0 6.64 7c-.254 0-.486.063-.682.202a1.23 1.23 0 0 0-.425.58l-3.435 8.959c-.06.155-.098.307-.098.445m4.625-7.603L8 13.36H5.25z' fill='currentColor'/></svg>"#
        }
        ToolbarIcon::InlineFootnote => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none'><path d='M20.5 7.263a5 5 0 0 1-.607.475.75.75 0 0 1-.832-1.248c.764-.51 1.33-1.321 1.456-1.9A.75.75 0 0 1 22 4.75v6.5a.75.75 0 0 1-1.5 0zm-8.278 10.493a.88.88 0 0 1-.222-.61V7.91c0-.246.074-.468.237-.645l.003-.004a.84.84 0 0 1 .628-.26c.243 0 .457.085.622.262a.9.9 0 0 1 .245.647v2.84q.273-.286.625-.492l.001-.001a3 3 0 0 1 1.524-.396c1.004 0 1.829.38 2.45 1.135.62.752.916 1.739.916 2.935 0 1.2-.297 2.19-.916 2.941-.623.752-1.455 1.128-2.471 1.128a3.03 3.03 0 0 1-1.546-.396 3 3 0 0 1-.64-.508v.05a.84.84 0 0 1-.243.61.84.84 0 0 1-.61.244.8.8 0 0 1-.6-.24zm4.75-1.976c.335-.435.516-1.043.516-1.85 0-.801-.181-1.406-.516-1.842-.329-.427-.77-.64-1.35-.64-.55 0-.995.216-1.352.665-.354.446-.542 1.045-.542 1.818 0 .778.188 1.378.542 1.825.356.443.8.658 1.352.658.581 0 1.022-.212 1.35-.634M2 17.186c0 .235.093.438.27.594l.007.007c.18.147.399.213.64.213a.9.9 0 0 0 .55-.174c.156-.118.265-.287.336-.484l.88-2.39H8.56l.886 2.39v.002c.072.196.18.364.336.482.159.12.348.174.55.174a.96.96 0 0 0 .637-.217.76.76 0 0 0 .28-.597 1.3 1.3 0 0 0-.098-.446L7.71 7.774a1.24 1.24 0 0 0-.413-.572A1.1 1.1 0 0 0 6.64 7c-.254 0-.486.063-.682.202a1.23 1.23 0 0 0-.425.58l-3.435 8.959c-.06.155-.098.307-.098.445m4.625-7.603L8 13.36H5.25z' fill='currentColor'/></svg>"#
        }
        ToolbarIcon::Superscript => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path d='M5 7l8 10m-8 0l8 -10'/><path d='M21 11h-4l3.5 -4a1.73 1.73 0 0 0 -3.5 -2'/></svg>"#
        }
        ToolbarIcon::Subscript => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path d='M5 7l8 10m-8 0l8 -10'/><path d='M21 20h-4l3.5 -4a1.73 1.73 0 0 0 -3.5 -2'/></svg>"#
        }
        ToolbarIcon::Emoji => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path d='M3 12a9 9 0 1 0 18 0a9 9 0 1 0 -18 0'/><path d='M9 10l.01 0'/><path d='M15 10l.01 0'/><path d='M9.5 15a3.5 3.5 0 0 0 5 0'/></svg>"#
        }
        ToolbarIcon::Mention => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-at'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M16 12a4 4 0 1 0 -4 4'/><path d='M16 12v1a2 2 0 1 0 4 0v-1a8 8 0 1 0 -4 6.93'/></svg>"#
        }
        ToolbarIcon::TabBlock => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path d='M4 4h16a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-16a2 2 0 0 1 -2 -2v-12a2 2 0 0 1 2 -2'/><path d='M4 8h16'/><path d='M4 4h6v4h-6z'/><path d='M12 4h6v4h-6z'/></svg>"#
        }
        ToolbarIcon::Slideshow => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path d='M15 6l.01 0'/><path d='M3 6a3 3 0 0 1 3 -3h12a3 3 0 0 1 3 3v8a3 3 0 0 1 -3 3h-12a3 3 0 0 1 -3 -3l0 -8'/><path d='M3 13l4 -4a3 5 0 0 1 3 0l4 4'/><path d='M13 12l2 -2a3 5 0 0 1 3 0l3 3'/><path d='M8 21l.01 0'/><path d='M12 21l.01 0'/><path d='M16 21l.01 0'/></svg>"#
        }
        ToolbarIcon::Math => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path d='M19 5h-7l-4 14l-3 -6h-2'/><path d='M14 13l6 6'/><path d='M14 19l6 -6'/></svg>"#
        }
        ToolbarIcon::Mermaid => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path d='M3 13a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v6a1 1 0 0 1 -1 1h-4a1 1 0 0 1 -1 -1l0 -6'/><path d='M15 9a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v10a1 1 0 0 1 -1 1h-4a1 1 0 0 1 -1 -1l0 -10'/><path d='M9 5a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v14a1 1 0 0 1 -1 1h-4a1 1 0 0 1 -1 -1l0 -14'/><path d='M4 20h14'/></svg>"#
        }
        ToolbarIcon::Issue => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-bug'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M9 9v-1a3 3 0 0 1 6 0v1' /><path d='M8 9h8a6 6 0 0 1 1 3v3a5 5 0 0 1 -10 0v-3a6 6 0 0 1 1 -3' /><path d='M3 13l4 0' /><path d='M17 13l4 0' /><path d='M12 20l0 -6' /><path d='M4 19l3.35 -2' /><path d='M20 19l-3.35 -2' /><path d='M4 7l3.75 2.4' /><path d='M20 7l-3.75 2.4' /></svg>"#
        }
        ToolbarIcon::Terminal => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-terminal-2'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M8 9l3 3l-3 3' /><path d='M13 15l3 0' /><path d='M3 6a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2l0 -12' /></svg>"#
        }
        ToolbarIcon::Toc => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-stack-2'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M12 4l-8 4l8 4l8 -4l-8 -4' /><path d='M4 12l8 4l8 -4' /><path d='M4 16l8 4l8 -4' /></svg>"#
        }
        ToolbarIcon::GutterOn => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24'><rect x='3' y='3' width='4' height='18' fill='currentColor' opacity='0.35'/><text x='5' y='8' font-size='4' text-anchor='middle' fill='currentColor' dominant-baseline='middle'>1</text><text x='5' y='12' font-size='4' text-anchor='middle' fill='currentColor' dominant-baseline='middle'>2</text><text x='5' y='16' font-size='4' text-anchor='middle' fill='currentColor' dominant-baseline='middle'>3</text><rect x='10' y='7' width='10' height='1.6' rx='0.5' fill='currentColor'/><rect x='10' y='11.2' width='9' height='1.6' rx='0.5' fill='currentColor'/><rect x='10' y='15.4' width='8' height='1.6' rx='0.5' fill='currentColor'/></svg>"#
        }
        ToolbarIcon::GutterOff => {
            r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24'><g opacity='0.25'><rect x='3' y='3' width='4' height='18' fill='currentColor' opacity='0.35'/><text x='5' y='8' font-size='4' text-anchor='middle' fill='currentColor' dominant-baseline='middle'>1</text><text x='5' y='12' font-size='4' text-anchor='middle' fill='currentColor' dominant-baseline='middle'>2</text><text x='5' y='16' font-size='4' text-anchor='middle' fill='currentColor' dominant-baseline='middle'>3</text><rect x='10' y='7' width='10' height='1.6' rx='0.5' fill='currentColor'/><rect x='10' y='11.2' width='9' height='1.6' rx='0.5' fill='currentColor'/><rect x='10' y='15.4' width='8' height='1.6' rx='0.5' fill='currentColor'/></g></svg>"#
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_composite_button_contracts_without_label() {
        let with_label = toolbar_composite_button_svg(composite_paths::TABLE, "Table");
        let without_label = toolbar_composite_button_svg(composite_paths::TABLE, "");

        assert!(
            without_label.viewbox_width < with_label.viewbox_width,
            "Icon-only composite button should be narrower"
        );
        assert!(
            without_label.viewbox_width < 60.0,
            "Icon-only composite button should not keep wide text minimum"
        );
    }
}
