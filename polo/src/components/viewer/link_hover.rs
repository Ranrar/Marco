//! In-page tooltip showing the full address of a hovered preview link.
//!
//! Previously a native `gtk4::Popover` pinned to the cursor, fed by
//! `polo_hover:` IPC messages round-tripping through Rust. That native
//! popover's repeated popup()/popdown()/reposition calls turned out to crash
//! the app on Windows — its underlying native surface sits on the same
//! window hierarchy as the WebView2 child HWND, and churning it at
//! hover-frequency (every `requestAnimationFrame`, i.e. up to ~60/sec while
//! the cursor moves over a link) reliably killed the process. Isolated by
//! disabling first the per-frame WebView2 bounds-sync tick callback (no
//! effect — still crashed) and then the popover's actual GTK calls (crash
//! disappeared, including under real mouse-driven hover, not just a
//! synthetic repro) — see `platform_webview.rs`'s Windows tick callback and
//! `PlatformWebView::set_hover_link_callback`'s removal for that history.
//!
//! This replacement renders the tooltip as ordinary HTML/CSS *inside* the
//! webview's own document — the same technique `viewer::drop_overlay` already
//! uses for the drag-and-drop overlay — so there's no second native window
//! for WebView2's child HWND to collide with, and positioning is trivial
//! (the tooltip and the cursor share the same viewport coordinate space, no
//! GTK-widget-relative translation needed). All of the hover-tracking JS
//! (title-attribute suppression, `mouseover`/`mousemove`/`mouseout`,
//! `requestAnimationFrame` throttling) now lives entirely in
//! `viewer::javascript::HOVER_REPORT_JS` — there is no Rust-side callback or
//! IPC message for this anymore.

/// Link icon — Tabler Icons `icon-tabler-link` (MIT). Identical source to
/// Marco's `LINK_ICON` in `marco/src/footer.rs`.
const LINK_ICON: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M12 21v-4"/><path d="M12 13v-4"/><path d="M12 5v-2"/><path d="M10 21h4"/><path d="M8 5v4h11l2 -2l-2 -2l-11 0"/><path d="M14 13v4h-8l-2 -2l2 -2l8 0"/></svg>"#;

/// Markup appended once into `<body>` on every page. Hidden
/// (`.polo-link-hover-tooltip` has `opacity: 0; pointer-events: none;` by
/// default — see [`CSS`]) until `HOVER_REPORT_JS` shows it on the first
/// hover report.
pub(crate) fn html() -> String {
    format!(
        r#"<div class="polo-link-hover-tooltip" id="polo-link-hover-tooltip"><div class="polo-link-hover-title" id="polo-link-hover-title"></div><div class="polo-link-hover-row">{icon}<span id="polo-link-hover-url"></span></div></div>"#,
        icon = LINK_ICON,
    )
}

/// CSS appended once into every page's `<style>` block. Colors mirror the
/// File/View menu popover's chrome (`css/menu_and_toolbar.rs`'s
/// `.polo-menu-popover`) — the same values the removed native popover used —
/// so the tooltip still reads as Polo chrome, not a separately-themed
/// browser tooltip. Hardcoded here rather than imported from that GTK CSS
/// generator, same reasoning as `drop_overlay::CSS`: one shared literal
/// place beats coupling an HTML/CSS component to a GTK stylesheet module.
pub(crate) const CSS: &str = r#"
.polo-link-hover-tooltip {
    position: fixed;
    left: 0;
    top: 0;
    z-index: 2147483647;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 4px 8px;
    border-radius: 4px;
    border: 1px solid;
    font-family: system-ui, -apple-system, 'Segoe UI', sans-serif;
    font-size: 0.8rem;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.1s ease;
    max-width: min(80vw, 640px);
    white-space: normal;
    word-break: break-all;
}
.polo-link-hover-tooltip.visible {
    opacity: 1;
}
.polo-link-hover-title {
    font-weight: 600;
    opacity: 0.9;
}
.polo-link-hover-row {
    display: flex;
    align-items: center;
    gap: 4px;
}
.polo-link-hover-row svg {
    width: 13px;
    height: 13px;
    stroke: currentColor;
    flex-shrink: 0;
}
.theme-light .polo-link-hover-tooltip {
    background: #ffffff;
    color: #2c3e50;
    border-color: #ccc;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12), 0 1px 3px rgba(0, 0, 0, 0.08);
}
.theme-dark .polo-link-hover-tooltip {
    background: #2d2d2d;
    color: #f0f5f1;
    border-color: #444;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.50), 0 1px 4px rgba(0, 0, 0, 0.30);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_html_contains_icon_and_slots() {
        let markup = html();
        assert!(markup.contains("polo-link-hover-tooltip"));
        assert!(markup.contains("polo-link-hover-title"));
        assert!(markup.contains("polo-link-hover-url"));
        assert!(markup.contains("<svg"));
    }

    #[test]
    fn smoke_test_css_covers_both_themes() {
        assert!(CSS.contains(".theme-light .polo-link-hover-tooltip"));
        assert!(CSS.contains(".theme-dark .polo-link-hover-tooltip"));
        assert!(CSS.contains("visible"));
    }
}
