/// Base structural stylesheet for the Marco HTML preview.
///
/// This contains ALL HTML element rules and Marco component rules.
/// It references CSS custom properties (set by the theme token files)
/// with safe fallback values so it works even if a variable is not defined.
///
/// Theme files (`.css` in `assets/themes/html_viever/`) provide ONLY
/// CSS custom property declarations (`--var: value`) — no structural rules.
///
/// Injection order in the final `<style>` block:
///   1. `inline_bg_style` — instant background colour flash-prevention
///   2. `base_css()`       — this file: all structure, via CSS custom properties
///   3. `css`              — the active theme file: only token overrides
///   4. `table_resize_css` — separate `<style>` block (interactive table/slider/anchor rules)
pub fn base_css() -> &'static str {
    BASE_CSS
}

const BASE_CSS: &str = r#"
/* ═══════════════════════════════════════════════════════════════════════════
   MARCO BASE STYLESHEET
   All structural CSS for the HTML preview pane. Uses CSS custom properties
   defined by the active theme token file. Safe fallbacks are provided for
   every variable so this works standalone.
   ═══════════════════════════════════════════════════════════════════════════ */

/* ── Color-scheme hints (controls scrollbars, form controls, native UI) ─── */
html.theme-light { color-scheme: light; }
html.theme-dark  { color-scheme: dark;  }

/* ── HTML element background (ensures full-page colour in static exports) ── */
html { background-color: var(--bg-color, #fff); }

/* ── Body ─────────────────────────────────────────────────────────────────── */
body {
    font-family: var(--body-font, -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Helvetica Neue', Arial, sans-serif);
    font-size: var(--body-font-size, 16px);
    line-height: var(--body-line-height, 1.6);
    color: var(--text-color, #333);
    background-color: var(--bg-color, #fff);
    max-width: var(--body-max-width, 900px);
    margin: 0 auto;
    padding: var(--body-padding, 2rem);
    transition: background-color 0.2s ease, color 0.2s ease;
}

body {
    --marco-task-primary: var(--link-color, #0066cc);
    --marco-task-accent:  var(--link-hover, #0052a3);
}

/* ── Headings ─────────────────────────────────────────────────────────────── */
h1, h2, h3, h4, h5, h6 {
    font-weight: 600;
    margin-top: 1.5rem;
    margin-bottom: 0.75rem;
    line-height: 1.25;
    /* Provide a real computed color for child elements (anchor icons, etc.). */
    color: var(--heading-color, #1a1a1a);
    /* Gradient: when --heading-gradient-end equals --heading-color the gradient
       is invisible (solid colour). Marco theme sets it to a pink/purple accent. */
    background: linear-gradient(
        45deg,
        var(--heading-color, #1a1a1a),
        var(--heading-gradient-end, var(--heading-color, #1a1a1a))
    );
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    position: relative;
}

h1 { font-size: 2rem; }
h2 { font-size: 1.75rem; }
h3 { font-size: 1.5rem; }
h4 { font-size: 1.25rem; }
h5 { font-size: 1.1rem; }
h6 { font-size: 1rem; }

/* Optional h1/h2 bottom border (e.g. GitHub theme sets this variable). */
h1, h2 {
    border-bottom: var(--h1h2-border-bottom, none);
    padding-bottom: var(--h1h2-padding-bottom, 0);
}

/* Fix code/mark/emoji inside headings: restore visible fill. */
h1 code, h2 code, h3 code, h4 code, h5 code, h6 code,
h1 mark, h2 mark, h3 mark, h4 mark, h5 mark, h6 mark {
    -webkit-text-fill-color: initial;
    color: var(--text-color, #333);
    background: none;
    background-clip: unset;
    -webkit-background-clip: unset;
}

h1 .marco-emoji, h2 .marco-emoji, h3 .marco-emoji,
h4 .marco-emoji, h5 .marco-emoji, h6 .marco-emoji {
    -webkit-text-fill-color: initial;
    color: initial;
}

/* ── Paragraphs ───────────────────────────────────────────────────────────── */
p {
    margin: 0 0 1rem 0;
    color: var(--text-color, #333);
}

/* ── Text formatting ──────────────────────────────────────────────────────── */
strong, b {
    font-weight: 700;
    color: var(--strong-color, var(--text-color, #333));
}

em, i {
    font-style: italic;
    color: var(--text-color, #333);
}

del, s {
    text-decoration: line-through;
    color: var(--text-muted, #888);
}

mark {
    background-color: var(--mark-bg, #fff8c5);
    color: var(--mark-color, #1a1a1a);
    padding: 0.1em 0.25em;
    border-radius: 3px;
}

sup, sub {
    font-size: 0.8em;
    line-height: 0;
    position: relative;
    vertical-align: baseline;
}

sup { top: -0.5em; }
sub { bottom: -0.25em; }

kbd {
    background-color: var(--bg-code, #f5f5f5);
    border: 1px solid var(--border-color, #ddd);
    border-radius: 4px;
    color: var(--text-color, #333);
    font-family: var(--code-font, 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace);
    font-size: 0.875em;
    padding: 0.2em 0.4em;
    vertical-align: baseline;
}

abbr[title] {
    border-bottom: 1px dotted var(--border-color, #ddd);
    cursor: help;
    text-decoration: none;
}

/* ── Links ────────────────────────────────────────────────────────────────── */
/* Basic colour + hover. SVG icon overlays are in the `table_resize_css` block. */
a {
    color: var(--link-color, #0066cc);
    text-decoration: none;
}

a:hover {
    color: var(--link-hover, var(--link-color, #0052a3));
    text-decoration: underline;
}

a:visited {
    color: var(--link-color, #0066cc);
}

/* ── Inline code ──────────────────────────────────────────────────────────── */
code {
    font-family: var(--code-font, 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace);
    font-size: 0.875em;
    background-color: var(--bg-code, #f5f5f5);
    border-radius: 4px;
    padding: 0.2em 0.4em;
}

/* ── Code blocks ──────────────────────────────────────────────────────────── */
pre {
    background-color: var(--bg-pre, #f8f8f8);
    border: 1px solid var(--border-color, #ddd);
    border-radius: 6px;
    margin: 1rem 0;
    overflow: hidden;
    position: relative;
}

pre[data-language]::before {
    content: attr(data-language);
    display: block;
    background-color: var(--bg-pre, #f8f8f8);
    color: var(--text-secondary, #666);
    padding: 0.4rem 0.75rem;
    font-size: 0.8rem;
    font-family: var(--body-font, -apple-system, sans-serif);
    font-weight: 400;
    border-bottom: 1px solid var(--border-color, #ddd);
    opacity: 0.8;
}

pre code {
    display: block;
    padding: 1rem;
    background: transparent;
    border-radius: 0;
    font-size: 0.875em;
    line-height: 1.5;
    overflow-x: auto;
    white-space: pre;
    word-break: normal;
}

/* ── Nested code blocks (.nested-code-block) ──────────────────────────────── */
.nested-code-block {
    border: 1px solid var(--border-color, #ddd);
    border-left: 4px solid var(--link-color, #0066cc);
    border-radius: 6px;
    margin: 1rem 0;
    overflow: hidden;
    background-color: var(--bg-pre, #f8f8f8);
}

.nested-code-block.level-2 { border-left-color: var(--blockquote-border, #ccc); }
.nested-code-block.level-3 { border-left-color: var(--border-strong, #999); }
.nested-code-block.level-4,
.nested-code-block.level-5,
.nested-code-block.level-6,
.nested-code-block.level-7,
.nested-code-block.level-8,
.nested-code-block.level-9,
.nested-code-block.level-10 { border-left-color: var(--text-muted, #888); }

.nested-code-block .code-header {
    background-color: var(--bg-secondary, #f5f5f5);
    color: var(--text-muted, #888);
    padding: 0.35rem 0.75rem;
    font-size: 0.8em;
    font-family: var(--code-font, monospace);
    border-bottom: 1px solid var(--border-color, #ddd);
}

.nested-code-block .code-content {
    padding: 0.75rem 1rem;
    background-color: var(--bg-color, #fff);
    min-height: 2rem;
}

.nested-code-block .code-content pre {
    margin: 0.5rem 0;
    border: 1px solid var(--border-color, #ddd);
}

.nested-code-block .code-content pre:before {
    display: none !important;
    content: none !important;
}

.nested-code-block .code-content p {
    margin: 0.5rem 0;
}

.nested-code-block .code-content p:first-child { margin-top: 0; }
.nested-code-block .code-content p:last-child  { margin-bottom: 0; }

/* ── Code copy button ─────────────────────────────────────────────────────── */
.marco-code-block-wrapper {
    position: relative;
    margin: 1rem 0;
}

.marco-code-copy-btn {
    position: absolute;
    top: 0.4rem;
    right: 0.4rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0.35rem;
    border: none;
    border-radius: 5px;
    background: transparent;
    color: var(--text-muted, #888);
    opacity: 0;
    cursor: pointer;
    transition: opacity 0.15s ease, background-color 0.15s ease, color 0.15s ease;
    z-index: 10;
}

.marco-code-block-wrapper:hover .marco-code-copy-btn {
    opacity: 1;
}

.marco-code-copy-btn svg {
    width: 1.1em;
    height: 1.1em;
    display: block;
    stroke: currentColor;
    fill: none;
}

.marco-code-copy-btn:hover {
    background-color: var(--bg-secondary, #f5f5f5);
    color: var(--link-color, #0066cc);
    opacity: 1;
}

.marco-code-copy-btn:active {
    background-color: var(--bg-code, #eee);
}

.marco-code-copy-btn.copied {
    color: #1a7f37;
    background-color: var(--bg-code, #eee);
    opacity: 1;
}

/* ── Blockquote ───────────────────────────────────────────────────────────── */
blockquote {
    margin: 1rem 0;
    padding: 0.75rem 1rem;
    border-left: 4px solid var(--blockquote-border, #ccc);
    background-color: var(--blockquote-bg, var(--bg-secondary, #f9f9f9));
    color: var(--blockquote-text, var(--text-secondary, #666));
    border-radius: 0 4px 4px 0;
}

blockquote > :first-child { margin-top: 0; }
blockquote > :last-child  { margin-bottom: 0; }

/* ── Lists ────────────────────────────────────────────────────────────────── */
ul, ol {
    margin: 0.5rem 0 1rem 0;
    padding-left: 2rem;
}

li {
    margin-bottom: 0.35rem;
    color: var(--text-color, #333);
}

li + li { margin-top: 0.15rem; }

/* ── Task lists ───────────────────────────────────────────────────────────── */
.task-list-item {
    list-style-type: none;
    margin-left: -1.5rem;
    padding-left: 0;
    display: flex;
    align-items: flex-start;
    gap: 0.4rem;
}

.marco-task-checkbox,
.task-list-item .task-list-item-checkbox {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex: 0 0 auto;
    width: 1.15em;
    height: 1.15em;
    margin-top: 0.15em;
    color: currentColor;
}

.marco-task-checkbox .marco-task-icon,
.task-list-item .task-list-item-checkbox .marco-task-icon {
    width: 1.15em;
    height: 1.15em;
    flex: 0 0 auto;
}

.marco-task-checkbox.checked,
.task-list-item .task-list-item-checkbox.checked {
    color: var(--marco-task-primary, var(--link-color, #0066cc));
}

.marco-task-checkbox.checked .marco-task-check,
.task-list-item .task-list-item-checkbox.checked .marco-task-check {
    stroke: var(--marco-task-accent, var(--link-hover, #0052a3));
}

.task-list-item input[type="checkbox"] {
    margin-right: 0.4rem;
    accent-color: var(--marco-task-primary, var(--link-color, #0066cc));
}

/* ── Definition lists ─────────────────────────────────────────────────────── */
dl { margin: 1rem 0; }

dt {
    font-weight: 700;
    margin-top: 1rem;
    color: var(--heading-color, #1a1a1a);
    -webkit-text-fill-color: var(--heading-color, #1a1a1a);
}

dd {
    margin-left: 2rem;
    margin-bottom: 0.5rem;
    color: var(--text-color, #333);
}

/* ── Tables ───────────────────────────────────────────────────────────────── */
table {
    border-collapse: collapse;
    width: 100%;
    margin: 1rem 0;
    font-size: 0.95em;
}

th, td {
    border: 1px solid var(--table-border, #ddd);
    padding: 0.6rem 0.85rem;
    text-align: left;
}

th {
    background-color: var(--table-header-bg, #f5f5f5);
    color: var(--table-header-color, var(--heading-color, #333));
    font-weight: 600;
}

tr:nth-child(even) {
    background-color: var(--table-stripe-bg, #fafafa);
}

tr:hover {
    background-color: var(--toc-link-hover-bg, var(--bg-secondary, #f5f5f5));
}

/* Right/centre alignment for GFM auto-align tables */
.marco-table-auto-align th[align="right"],
.marco-table-auto-align td[align="right"] { text-align: right; }

.marco-table-auto-align th[align="center"],
.marco-table-auto-align td[align="center"] { text-align: center; }

/* ── Horizontal rule ──────────────────────────────────────────────────────── */
hr {
    border: none;
    border-top: 1px solid var(--border-color, #ddd);
    margin: 1.5rem 0;
}

/* ── Images ───────────────────────────────────────────────────────────────── */
img {
    max-width: 100%;
    height: auto;
    display: block;
    border-radius: 4px;
}

/* Figure caption: italicised <em> directly after an image */
img + em {
    display: block;
    text-align: center;
    font-style: italic;
    color: var(--text-muted, #888);
    margin-top: 0.4rem;
    font-size: 0.9em;
}

/* ── Emoji ────────────────────────────────────────────────────────────────── */
.marco-emoji {
    font-size: 1.2em;
    display: inline-block;
    vertical-align: baseline;
}

/* ── Footnotes ────────────────────────────────────────────────────────────── */
.footnote-ref {
    font-size: 0.8em;
    vertical-align: super;
    color: var(--link-color, #0066cc);
    font-weight: 700;
    line-height: 0;
}

.footnote-backref {
    font-size: 0.85em;
    color: var(--link-color, #0066cc);
    text-decoration: none;
}

.footnotes {
    border-top: 1px solid var(--border-light, var(--border-color, #ddd));
    margin-top: 2rem;
    padding-top: 1rem;
    font-size: 0.9em;
    color: var(--text-secondary, #666);
}

/* ── Inline footnotes ─────────────────────────────────────────────────────── */
.marco-inline-footnote {
    font-size: 0.8em;
    vertical-align: super;
    color: var(--link-color, #0066cc);
    cursor: default;
    border-bottom: 1px dotted var(--link-color, #0066cc);
    line-height: 0;
}

/* ── Admonitions ──────────────────────────────────────────────────────────── */
.admonition {
    margin: 0.75rem 0;
    padding: 0.65rem 1rem;
    border-left: 4px solid var(--admonition-border, var(--border-strong, #ccc));
    background-color: var(--admonition-bg, var(--bg-secondary, #f9f9f9));
    border-radius: 0 6px 6px 0;
}

.admonition p {
    margin: 0.4rem 0;
}

.admonition p:first-child { margin-top: 0; }
.admonition p:last-child  { margin-bottom: 0; }

.admonition .markdown-alert-title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 600;
    line-height: 1.2;
    margin: 0 0 0.5rem 0;
}

.admonition .markdown-alert-icon {
    display: inline-flex;
    flex: 0 0 auto;
    align-items: center;
    justify-content: center;
    line-height: 0;
}

.admonition .markdown-alert-icon svg {
    width: 1.4em;
    height: 1.4em;
    display: block;
    shape-rendering: auto;
}

.admonition .markdown-alert-emoji {
    width: 1.4em;
    height: 1.4em;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    font-size: 1.2em;
}

/* Quote-style admonition (blockquote colours) */
.admonition-quote {
    border-left-color: var(--blockquote-border, #ccc);
    background-color: var(--blockquote-bg, var(--bg-secondary, #f9f9f9));
    color: var(--blockquote-text, var(--text-secondary, #666));
}

.admonition-quote .markdown-alert-title {
    color: var(--link-color, #0066cc);
}

/* Semantic admonition types — use consistent GitHub-style accent colours */
.admonition-note { border-left-color: #0969da; background-color: var(--note-bg, var(--admonition-bg, #ebf5fb)); }
.admonition-note .markdown-alert-title { color: #0969da; }

.admonition-tip  { border-left-color: #1a7f37; background-color: var(--tip-bg, var(--admonition-bg, #eafaf1)); }
.admonition-tip  .markdown-alert-title { color: #1a7f37; }

.admonition-important { border-left-color: #8250df; background-color: var(--important-bg, var(--admonition-bg, #f5f0ff)); }
.admonition-important .markdown-alert-title { color: #8250df; }

.admonition-warning { border-left-color: #d97706; background-color: var(--warning-bg, var(--admonition-bg, #fffbeb)); }
.admonition-warning .markdown-alert-title { color: #d97706; }

.admonition-caution { border-left-color: #cf222e; background-color: var(--caution-bg, var(--admonition-bg, #fef2f2)); }
.admonition-caution .markdown-alert-title { color: #cf222e; }

/* ── Platform mentions ────────────────────────────────────────────────────── */
.marco-mention,
platform-mention {
    display: inline;
    color: var(--link-color, #0066cc);
    font-weight: 500;
}

.marco-mention:hover,
platform-mention:hover {
    color: var(--link-hover, var(--link-color, #0052a3));
    text-decoration: underline;
    cursor: pointer;
}

/* ── GFM autolinks ────────────────────────────────────────────────────────── */
.marco-autolink {
    color: var(--link-color, #0066cc);
    text-decoration: none;
}

.marco-autolink:hover {
    color: var(--link-hover, var(--link-color, #0052a3));
    text-decoration: underline;
}

/* ── Table of Contents (.toc) ─────────────────────────────────────────────── */
.toc {
    margin: 1.5rem 0;
    padding: 1rem 1.25rem;
    background-color: var(--toc-bg, var(--bg-secondary, #f9f9f9));
    border: 1px solid var(--toc-border, var(--border-color, #ddd));
    border-radius: 6px;
    box-shadow: 0 2px 6px var(--toc-shadow, rgba(0, 0, 0, 0.08));
}

.toc h4 {
    margin: 0 0 0.75rem 0;
    font-size: 1rem;
    color: var(--toc-header-color, var(--heading-color, #333));
    -webkit-text-fill-color: var(--toc-header-color, var(--heading-color, #333));
    background: none;
    -webkit-background-clip: unset;
    background-clip: unset;
    border-bottom: 1px solid var(--toc-header-border, var(--border-color, #ddd));
    padding-bottom: 0.5rem;
    font-weight: 600;
}

.toc ul {
    list-style: none;
    margin: 0;
    padding-left: 0;
}

.toc li {
    margin: 0.15rem 0;
    line-height: 1.4;
}

.toc a {
    display: block;
    padding: 0.3rem 0.6rem;
    color: var(--toc-link-color, var(--text-color, #333));
    text-decoration: none;
    border-radius: 4px;
    -webkit-text-fill-color: var(--toc-link-color, var(--text-color, #333));
    transition: background-color 0.12s ease, color 0.12s ease;
}

.toc a:hover {
    background-color: var(--toc-link-hover-bg, var(--bg-secondary, #f0f0f0));
    color: var(--toc-link-hover-color, var(--link-color, #0066cc));
    -webkit-text-fill-color: var(--toc-link-hover-color, var(--link-color, #0066cc));
}

.toc a.active {
    background-color: var(--toc-link-active-bg, #e0e7ff);
    color: var(--toc-link-active-color, var(--link-color, #0066cc));
    -webkit-text-fill-color: var(--toc-link-active-color, var(--link-color, #0066cc));
    font-weight: 600;
}

/* Nested TOC levels */
.toc ul ul {
    padding-left: 1rem;
    margin-top: 0.15rem;
    margin-bottom: 0;
}

.toc ul ul li { margin: 0.1rem 0; }

.toc ul ul a {
    font-size: 0.9em;
    padding: 0.2rem 0.5rem;
}

/* ── Tab blocks (.marco-tabs) ─────────────────────────────────────────────── */
.marco-tabs {
    margin: 1rem 0;
    border: 1px solid var(--border-color, #ddd);
    border-radius: 6px;
    overflow: visible;
}

/* Keep radios in the accessibility tree while hiding them visually */
.marco-tabs__radio {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
    pointer-events: none;
}

.marco-tabs__tablist {
    display: flex;
    flex-wrap: wrap;
    gap: 0;
    border-bottom: 1px solid var(--border-color, #ddd);
    background-color: var(--bg-secondary, #f9f9f9);
    border-radius: 6px 6px 0 0;
    padding: 0 0.25rem;
}

.marco-tabs__tab {
    padding: 0.5rem 1rem;
    cursor: pointer;
    color: var(--text-secondary, #666);
    font-size: 0.9em;
    font-weight: 500;
    border: none;
    background: transparent;
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
    transition: color 0.12s ease, border-color 0.12s ease;
    user-select: none;
}

.marco-tabs__tab:hover {
    color: var(--link-color, #0066cc);
}

.marco-tabs__panels {
    border-radius: 0 0 6px 6px;
}

.marco-tabs__panel {
    display: none;
    padding: 1rem;
}

.marco-tabs__panel > :first-child { margin-top: 0; }
.marco-tabs__panel > :last-child  { margin-bottom: 0; }

/* Show the selected panel + style the selected tab label (up to 12 tabs) */
.marco-tabs > input.marco-tabs__radio:nth-of-type(1):checked ~ .marco-tabs__panels > .marco-tabs__panel:nth-of-type(1),
.marco-tabs > input.marco-tabs__radio:nth-of-type(2):checked ~ .marco-tabs__panels > .marco-tabs__panel:nth-of-type(2),
.marco-tabs > input.marco-tabs__radio:nth-of-type(3):checked ~ .marco-tabs__panels > .marco-tabs__panel:nth-of-type(3),
.marco-tabs > input.marco-tabs__radio:nth-of-type(4):checked ~ .marco-tabs__panels > .marco-tabs__panel:nth-of-type(4),
.marco-tabs > input.marco-tabs__radio:nth-of-type(5):checked ~ .marco-tabs__panels > .marco-tabs__panel:nth-of-type(5),
.marco-tabs > input.marco-tabs__radio:nth-of-type(6):checked ~ .marco-tabs__panels > .marco-tabs__panel:nth-of-type(6),
.marco-tabs > input.marco-tabs__radio:nth-of-type(7):checked ~ .marco-tabs__panels > .marco-tabs__panel:nth-of-type(7),
.marco-tabs > input.marco-tabs__radio:nth-of-type(8):checked ~ .marco-tabs__panels > .marco-tabs__panel:nth-of-type(8),
.marco-tabs > input.marco-tabs__radio:nth-of-type(9):checked ~ .marco-tabs__panels > .marco-tabs__panel:nth-of-type(9),
.marco-tabs > input.marco-tabs__radio:nth-of-type(10):checked ~ .marco-tabs__panels > .marco-tabs__panel:nth-of-type(10),
.marco-tabs > input.marco-tabs__radio:nth-of-type(11):checked ~ .marco-tabs__panels > .marco-tabs__panel:nth-of-type(11),
.marco-tabs > input.marco-tabs__radio:nth-of-type(12):checked ~ .marco-tabs__panels > .marco-tabs__panel:nth-of-type(12) {
    display: block;
}

.marco-tabs > input.marco-tabs__radio:nth-of-type(1):checked ~ .marco-tabs__tablist > .marco-tabs__tab:nth-of-type(1),
.marco-tabs > input.marco-tabs__radio:nth-of-type(2):checked ~ .marco-tabs__tablist > .marco-tabs__tab:nth-of-type(2),
.marco-tabs > input.marco-tabs__radio:nth-of-type(3):checked ~ .marco-tabs__tablist > .marco-tabs__tab:nth-of-type(3),
.marco-tabs > input.marco-tabs__radio:nth-of-type(4):checked ~ .marco-tabs__tablist > .marco-tabs__tab:nth-of-type(4),
.marco-tabs > input.marco-tabs__radio:nth-of-type(5):checked ~ .marco-tabs__tablist > .marco-tabs__tab:nth-of-type(5),
.marco-tabs > input.marco-tabs__radio:nth-of-type(6):checked ~ .marco-tabs__tablist > .marco-tabs__tab:nth-of-type(6),
.marco-tabs > input.marco-tabs__radio:nth-of-type(7):checked ~ .marco-tabs__tablist > .marco-tabs__tab:nth-of-type(7),
.marco-tabs > input.marco-tabs__radio:nth-of-type(8):checked ~ .marco-tabs__tablist > .marco-tabs__tab:nth-of-type(8),
.marco-tabs > input.marco-tabs__radio:nth-of-type(9):checked ~ .marco-tabs__tablist > .marco-tabs__tab:nth-of-type(9),
.marco-tabs > input.marco-tabs__radio:nth-of-type(10):checked ~ .marco-tabs__tablist > .marco-tabs__tab:nth-of-type(10),
.marco-tabs > input.marco-tabs__radio:nth-of-type(11):checked ~ .marco-tabs__tablist > .marco-tabs__tab:nth-of-type(11),
.marco-tabs > input.marco-tabs__radio:nth-of-type(12):checked ~ .marco-tabs__tablist > .marco-tabs__tab:nth-of-type(12) {
    color: var(--link-color, #0066cc);
    border-bottom-color: var(--link-color, #0066cc);
}

/* ── Paged.js page paper colors ───────────────────────────────────────────── */
/* Each theme defines --pagedjs-page-bg and --pagedjs-page-color via
   .theme-light and .theme-dark blocks in its token file.               */
.pagedjs_page {
    background-color: var(--pagedjs-page-bg, #ffffff);
    color: var(--pagedjs-page-color, var(--text-color, #333));
}

.pagedjs_margin .pagedjs_margin-content {
    color: var(--text-muted, #888);
    font-size: 0.8em;
}

/* ── WebKit scrollbar (HTML preview pane) ─────────────────────────────────── */
::-webkit-scrollbar { width: 12px; height: 12px; }

::-webkit-scrollbar-track {
    background: var(--bg-secondary, #f5f5f5);
}

::-webkit-scrollbar-thumb {
    background: var(--border-strong, #ccc);
    border-radius: 0;
}

::-webkit-scrollbar-thumb:hover { opacity: 0.9; }

/* ── Selection ────────────────────────────────────────────────────────────── */
::selection {
    background-color: var(--mark-bg, #b3d9ff);
    color: var(--mark-color, #1a1a1a);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_base_css_non_empty() {
        let css = base_css();
        assert!(!css.is_empty(), "base_css() must not be empty");
    }

    #[test]
    fn smoke_base_css_contains_key_selectors() {
        let css = base_css();
        assert!(css.contains("body {"), "body rule missing");
        assert!(css.contains("blockquote {"), "blockquote rule missing");
        assert!(css.contains(".admonition"), "admonition rule missing");
        assert!(css.contains(".marco-tabs"), "tab block rule missing");
        assert!(css.contains(".toc"), "TOC rule missing");
        assert!(css.contains(".pagedjs_page"), "paged.js page rule missing");
        assert!(
            css.contains("var(--text-color"),
            "text-color variable missing"
        );
        assert!(css.contains("var(--bg-color"), "bg-color variable missing");
        assert!(
            css.contains("var(--heading-color"),
            "heading-color variable missing"
        );
    }

    #[test]
    fn smoke_base_css_no_format_braces() {
        // The CSS must NOT contain unescaped `{` immediately followed by `{`
        // (which would indicate accidental format!()-style escaping).
        let css = base_css();
        // Valid CSS has single braces; doubled braces are a format! artifact.
        assert!(
            !css.contains("{{"),
            "base_css contains escaped braces ('{{') — remove them"
        );
        assert!(
            !css.contains("}}"),
            "base_css contains escaped braces ('}}') — remove them"
        );
    }
}
