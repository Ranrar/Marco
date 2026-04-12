//! CSS for the preview zoom overlay bar.
//!
//! The bar floats at the bottom-right corner of the preview panel and contains
//! three buttons: zoom-out (`-`), reset (⤢), zoom-in (`+`).

/// Generate zoom bar CSS for both themes.
pub fn generate_css() -> String {
    r#"
/* ── Zoom overlay bar ─────────────────────────────────────────────────────── */
.zoom-bar {
    background-color: rgba(30, 30, 30, 0.72);
    border-radius: 10px;
    padding: 2px 4px;
    margin: 0 10px 10px 0;
    border: 1px solid rgba(255, 255, 255, 0.12);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.30);
}

.marco-theme-light .zoom-bar {
    background-color: rgba(245, 245, 245, 0.88);
    border: 1px solid rgba(0, 0, 0, 0.14);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.14);
}

.zoom-bar-btn {
    background: transparent;
    border: none;
    border-radius: 6px;
    min-width: 30px;
    min-height: 30px;
    padding: 2px 6px;
    color: rgba(220, 220, 220, 0.92);
    font-size: 14pt;
    font-weight: 500;
    transition: background 120ms ease;
}

.marco-theme-light .zoom-bar-btn {
    color: rgba(40, 40, 40, 0.90);
}

.zoom-bar-btn:hover {
    background-color: rgba(255, 255, 255, 0.18);
}

.marco-theme-light .zoom-bar-btn:hover {
    background-color: rgba(0, 0, 0, 0.10);
}

.zoom-bar-btn:active {
    background-color: rgba(255, 255, 255, 0.28);
}

.marco-theme-light .zoom-bar-btn:active {
    background-color: rgba(0, 0, 0, 0.18);
}

.zoom-bar-label {
    color: rgba(200, 200, 200, 0.85);
    font-size: 9pt;
    min-width: 36px;
    padding: 0 2px;
}

.marco-theme-light .zoom-bar-label {
    color: rgba(60, 60, 60, 0.85);
}

.zoom-bar-separator {
    background-color: rgba(255, 255, 255, 0.14);
    min-width: 1px;
    margin: 4px 2px;
}

.marco-theme-light .zoom-bar-separator {
    background-color: rgba(0, 0, 0, 0.12);
}
"#
    .to_string()
}
