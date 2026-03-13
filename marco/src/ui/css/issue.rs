//! Diagnostics issue list CSS
//!
//! Row colors are taken 1:1 from `buttons.rs` / `constants.rs`:
//! - Error   → Red    (`#d9534f`)
//! - Warning → Yellow (`#f0ad4e`)
//! - Info    → Blue   (`#0066cc` light / `#4f8cff` dark)
//! - Hint    → Grey   (`#6c757d`)
//!
//! Text is `#ffffff` and hover dimming (`opacity: 0.9`) matches button behaviour.

use super::constants::{ColorPalette, DARK_PALETTE, LIGHT_PALETTE};

const YELLOW: &str = "#f0ad4e";
const RED: &str = "#d9534f";
const GREY: &str = "#6c757d";

pub fn generate_css() -> String {
    let mut css = String::with_capacity(4096);
    css.push_str(BASE_CSS);
    css.push_str(&generate_theme_css("marco-theme-light", &LIGHT_PALETTE));
    css.push_str(&generate_theme_css("marco-theme-dark", &DARK_PALETTE));
    css
}

const BASE_CSS: &str = r#"
/* Diagnostics issue list */
.footer-issue-list {
    min-height: 180px;
}

.footer-issue-list row {
    margin: 0 0 4px 0;
    border-radius: 6px;
    background: transparent;
}

.footer-issue-row {
    padding: 6px 8px;
    border-radius: 6px;
    transition: opacity 100ms ease;
}

.footer-issue-row-label {
    font-size: 12px;
    line-height: 1.35;
}

.footer-issue-empty {
    opacity: 0.75;
    padding: 8px 2px;
}
"#;

fn generate_theme_css(theme: &str, palette: &ColorPalette) -> String {
    let blue = palette.toolbar_button_hover_border;
    let text = palette.footer_text;
    let popover_border = palette.toolbar_popover_border;
    let separator = palette.toolbar_separator;

    format!(
        r#"
/* {theme} - popover inner containers: transparent so the popover.rs > contents bg shows through */
.{theme} .footer-diag-filter-bar {{
    background: transparent;
    border-bottom: 1px solid {popover_border};
    padding: 4px 6px;
}}

.{theme} .footer-issue-scrolled {{
    background: transparent;
}}

.{theme} .footer-issue-list {{
    background: transparent;
}}


/* {theme} - scrollbar */
.{theme} .footer-issue-scrolled scrollbar {{
    background-color: transparent;
    border: none;
}}

.{theme} .footer-issue-scrolled scrollbar slider {{
    background-color: {separator};
    border-radius: 4px;
    min-width: 6px;
    min-height: 6px;
    margin: 2px;
}}

.{theme} .footer-issue-scrolled scrollbar slider:hover {{
    background-color: {text};
}}

/* {theme} - issue row severity colors (1:1 with button palette) */

.{theme} .footer-issue-row.footer-issue-row-error,
.{theme}.marco-dialog .footer-issue-row.footer-issue-row-error {{
    background: {red};
    color: #ffffff;
    border: 1px solid {red};
}}
.{theme} .footer-issue-row.footer-issue-row-error .footer-issue-row-label,
.{theme}.marco-dialog .footer-issue-row.footer-issue-row-error .footer-issue-row-label {{
    color: #ffffff;
}}

.{theme} .footer-issue-row.footer-issue-row-warning,
.{theme}.marco-dialog .footer-issue-row.footer-issue-row-warning {{
    background: {yellow};
    color: #ffffff;
    border: 1px solid {yellow};
}}
.{theme} .footer-issue-row.footer-issue-row-warning .footer-issue-row-label,
.{theme}.marco-dialog .footer-issue-row.footer-issue-row-warning .footer-issue-row-label {{
    color: #ffffff;
}}

.{theme} .footer-issue-row.footer-issue-row-info,
.{theme}.marco-dialog .footer-issue-row.footer-issue-row-info {{
    background: {blue};
    color: #ffffff;
    border: 1px solid {blue};
}}
.{theme} .footer-issue-row.footer-issue-row-info .footer-issue-row-label,
.{theme}.marco-dialog .footer-issue-row.footer-issue-row-info .footer-issue-row-label {{
    color: #ffffff;
}}

.{theme} .footer-issue-row.footer-issue-row-hint,
.{theme}.marco-dialog .footer-issue-row.footer-issue-row-hint {{
    background: {grey};
    color: #ffffff;
    border: 1px solid {grey};
}}
.{theme} .footer-issue-row.footer-issue-row-hint .footer-issue-row-label,
.{theme}.marco-dialog .footer-issue-row.footer-issue-row-hint .footer-issue-row-label {{
    color: #ffffff;
}}

/* Hover: same dim effect as marco-btn-*:hover (opacity 0.9) */
.{theme} .footer-issue-list row:hover .footer-issue-row,
.{theme}.marco-dialog .footer-issue-list row:hover .footer-issue-row {{
    opacity: 0.9;
}}
"#,
        theme = theme,
        red = RED,
        yellow = YELLOW,
        blue = blue,
        grey = GREY,
        text = text,
        popover_border = popover_border,
        separator = separator,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_issue_css_generation() {
        let css = generate_css();
        assert!(!css.is_empty());
        assert!(css.contains("footer-issue-list"));
        assert!(css.contains("footer-issue-row-error"));
        assert!(css.contains("footer-issue-row-warning"));
        assert!(css.contains("footer-issue-row-info"));
        assert!(css.contains("footer-issue-row-hint"));
        assert!(css.contains("marco-theme-light"));
        assert!(css.contains("marco-theme-dark"));
        assert!(css.contains("footer-issue-scrolled"));
        assert!(css.contains("footer-diag-filter-bar"));
        assert!(css.contains("scrollbar slider"));
    }
}
