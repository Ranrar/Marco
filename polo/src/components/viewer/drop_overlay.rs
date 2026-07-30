//! Shared "drop file to open" overlay — the single drag-and-drop visual
//! treatment for every webview page, whether the empty state or a rendered
//! document is currently showing.
//!
//! Earlier versions used two different mechanisms: the empty state swapped
//! its own icon/text in place via CSS, while a loaded document was covered
//! by a separate native GTK panel (dashed border, different sizing/colors)
//! stacked above the webview via `gtk4::Overlay`. That native panel is gone
//! now — this HTML/CSS snippet, appended into *every* page's
//! `<body>`/`<style>`, is the only mechanism, so dragging a file looks and
//! behaves identically regardless of what's on screen underneath, and there
//! is exactly one place to keep the design in sync instead of two (which is
//! exactly how the two previously drifted out of matching colors).
//!
//! `position: fixed` so it covers the full viewport regardless of the
//! underlying document's scroll position — confined to the webview's own
//! viewport, so (like the native panel it replaces) it never extends over
//! the toolbar or titlebar, which live outside the webview entirely. Hidden
//! by default (`opacity: 0; pointer-events: none;`), revealed by toggling
//! `dragging` on `<html>` — see `PlatformWebView::set_dragging_state`,
//! driven from `main.rs`'s wry `DragDropEvent` hover handler. Neither page
//! has JS of its own listening for OS-level file drags; only wry's native
//! handler sees those.

/// Drag-and-drop icon (Tabler `icon-tabler-drag-drop`).
const SVG_DRAG_DROP: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none" /><path d="M19 11v-2a2 2 0 0 0 -2 -2h-8a2 2 0 0 0 -2 2v8a2 2 0 0 0 2 2h2" /><path d="M13 13l9 3l-4 2l-2 4l-3 -9" /><path d="M3 3l0 .01" /><path d="M7 3l0 .01" /><path d="M11 3l0 .01" /><path d="M15 3l0 .01" /><path d="M3 7l0 .01" /><path d="M3 11l0 .01" /><path d="M3 15l0 .01" /></svg>"#;

/// Markup appended once into `<body>` on every page.
pub(crate) fn html() -> String {
    format!(
        r#"<div class="polo-drop-overlay"><div class="polo-drop-overlay-content">{icon}<p>Drop file to open</p></div></div>"#,
        icon = SVG_DRAG_DROP,
    )
}

/// CSS appended once into every page's `<style>` block. Colors deliberately
/// match `empty_state.rs`'s own body/paragraph colors exactly — not
/// `css::constants::ColorPalette`, which describes the surrounding chrome
/// (toolbar/titlebar) and differs from the webview content area in dark
/// mode. Hardcoding the same literals `empty_state.rs` uses, in this one
/// shared place, is what keeps the two from drifting apart the way the old
/// native-panel version once did.
pub(crate) const CSS: &str = r#"
.polo-drop-overlay {
    position: fixed;
    inset: 0;
    /* Always above arbitrary rendered markdown content, however it stacks
       its own elements. */
    z-index: 2147483647;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.15s ease;
    font-family: system-ui, -apple-system, 'Segoe UI', sans-serif;
}
html.dragging .polo-drop-overlay {
    opacity: 1;
}
.polo-drop-overlay-content {
    display: flex;
    flex-direction: column;
    align-items: center;
}
.polo-drop-overlay-content svg {
    width: 3rem;
    height: 3rem;
    stroke: currentColor;
    margin: 0 0 1rem 0;
    flex-shrink: 0;
}
.polo-drop-overlay-content p {
    font-size: 1.2rem;
    margin: 0.5rem 0;
}
.theme-light .polo-drop-overlay {
    background: #ffffff;
    color: #2c3e50;
}
.theme-light .polo-drop-overlay-content p {
    color: #5a6c7d;
}
.theme-dark .polo-drop-overlay {
    background: #1e1e1e;
    color: #e0e0e0;
}
.theme-dark .polo-drop-overlay-content p {
    color: #9198a1;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_html_contains_icon_and_text() {
        let markup = html();
        assert!(markup.contains("polo-drop-overlay"));
        assert!(markup.contains("Drop file to open"));
        assert!(markup.contains("<svg"));
    }

    #[test]
    fn smoke_test_css_covers_both_themes() {
        assert!(CSS.contains(".theme-light .polo-drop-overlay"));
        assert!(CSS.contains(".theme-dark .polo-drop-overlay"));
        assert!(CSS.contains("dragging"));
    }
}
