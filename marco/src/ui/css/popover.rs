//! Toolbar insertion popover CSS
//!
//! Styles `popover.marco-link-popover` following the GTK4 CSS node spec:
//!
//! ```text
//! popover.background.marco-link-popover
//! ├── arrow
//! ╰── contents
//!     ╰── <child>
//! ```
//!
//! Per GTK4 docs: "When styling a popover directly, the popover node should
//! usually not have any background. The visible part of the popover can have a
//! shadow — set the box-shadow of the **contents** node."
//! The arrow node must use `border-bottom-width` and its size must match the
//! contents border width exactly so they render as one connected bubble.
//!
//! ## CSS classes styled
//!
//! - `popover.marco-link-popover`          — transparent, no bg here
//! - `popover.marco-link-popover > contents` — bg, border-radius, shadow
//! - `popover.marco-link-popover > arrow`    — bg + matching border-bottom-width
//! - `.marco-link-popover-separator`         — thin divider

use super::constants::*;

pub fn generate_css() -> String {
    let mut css = String::with_capacity(4096);
    css.push_str(BASE_CSS);

    // marco-link-popover (toolbar insertion: link, code, footnote, emoji)
    css.push_str(&generate_theme_css(
        "marco-theme-light",
        &LIGHT_PALETTE,
        LIGHT_POPOVER_BG,
    ));
    css.push_str(&generate_theme_css(
        "marco-theme-dark",
        &DARK_PALETTE,
        DARK_POPOVER_BG,
    ));

    // All other app popovers — same arrow+contents pattern
    for class in NAMED_POPOVER_CLASSES {
        css.push_str(&generate_named_popover_css(
            class,
            "marco-theme-light",
            &LIGHT_PALETTE,
            LIGHT_POPOVER_BG,
        ));
        css.push_str(&generate_named_popover_css(
            class,
            "marco-theme-dark",
            &DARK_PALETTE,
            DARK_POPOVER_BG,
        ));
    }

    css
}

/// All named popover classes that share the same arrow+contents styling.
///
/// Each maps to a CSS class added in Rust widget code:
/// - `marco-toolbar-popover`: 5 toolbar dropdown popovers (`toolbar.rs`)
/// - `marco-diagnostics-popover`: footer diagnostics panel (`footer.rs`)
/// - `marco-menu-popover`: main menubar menus (`menu.rs`)
/// - `marco-context-menu-popover`: editor right-click context menu (`contextmenu.rs`)
/// - `tools-menu-popover`: Tools menu custom popover (`menu.rs`)
const NAMED_POPOVER_CLASSES: &[&str] = &[
    "marco-toolbar-popover",
    "marco-diagnostics-popover",
    "marco-menu-popover",
    "marco-context-menu-popover",
    "tools-menu-popover",
];

/// Dark bg matches the editor gutter colour (#2d2d30) — user preference.
const DARK_POPOVER_BG: &str = "#2d2d30";
const LIGHT_POPOVER_BG: &str = "#ffffff";

/// Base (theme-independent) geometry rules only — no colours here.
const BASE_CSS: &str = r#"
/* Toolbar insertion popover — let the contents node do the visual work */
popover.marco-link-popover {
    background: transparent;
    box-shadow: none;
    padding: 0;
}

popover.marco-link-popover > contents {
    border-radius: 8px;
    padding: 4px 2px;
}

/* Thin separator inside the popover content box */
.marco-link-popover-separator {
    min-height: 1px;
    margin: 4px 0;
}

/* ── Hover-insight popover — structural/geometry (non-themed) ─────────── */

.marco-hover-title {
    font-size: 13px;
    font-weight: 600;
    line-height: 1.3;
}

.marco-hover-code-chip {
    border-radius: 4px;
    padding: 2px 7px;
    font-family: monospace;
    font-size: 11px;
    font-weight: 600;
}

.marco-hover-section-label {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.4px;
    text-transform: uppercase;
}

.marco-hover-body-text {
    font-size: 12px;
    line-height: 1.45;
}

.marco-hover-separator {
    min-height: 1px;
    margin-top: 0;
    margin-bottom: 0;
}

/* Named popovers — transparent outer node, contents handles the visual */
popover.marco-toolbar-popover,
popover.marco-diagnostics-popover,
popover.marco-menu-popover,
popover.marco-context-menu-popover,
popover.tools-menu-popover {
    background: transparent;
    box-shadow: none;
    padding: 0;
}

popover.marco-toolbar-popover > contents,
popover.marco-diagnostics-popover > contents,
popover.marco-menu-popover > contents,
popover.marco-context-menu-popover > contents,
popover.tools-menu-popover > contents {
    border-radius: 8px;
    padding: 4px;
}
"#;

/// Generate CSS for a named popover class (toolbar dropdowns, diagnostics, menus, context menus).
/// Uses the same arrow+contents grouped pattern as `marco-link-popover`.
fn generate_named_popover_css(
    class: &str,
    theme: &str,
    palette: &ColorPalette,
    bg: &str,
) -> String {
    let border = palette.toolbar_popover_border;
    let shadow = if theme == "marco-theme-dark" {
        "0 4px 16px rgba(0, 0, 0, 0.50), 0 1px 4px rgba(0, 0, 0, 0.30)"
    } else {
        "0 4px 12px rgba(0, 0, 0, 0.12), 0 1px 3px rgba(0, 0, 0, 0.08)"
    };

    format!(
        r#"
/* {theme} - {class} */
.{theme} popover.{class} {{
    background: transparent;
}}

.{theme} popover.{class} > arrow,
.{theme} popover.{class} > contents {{
    background-color: {bg};
    border: 1px solid {border};
    box-shadow: {shadow};
}}

.{theme} popover.{class} > contents {{
    border-radius: 8px;
}}
"#,
        theme = theme,
        class = class,
        bg = bg,
        border = border,
        shadow = shadow,
    )
}

fn generate_theme_css(theme: &str, palette: &ColorPalette, bg: &str) -> String {
    let border = palette.toolbar_popover_border;
    let text = palette.titlebar_foreground;
    let separator = palette.toolbar_separator;

    // Use the same rule for arrow + contents (mirrors how Yaru/GTK themes work):
    // GTK reads border-bottom-width for the arrow stroke — the `border:` shorthand
    // sets that too, making arrow and bubble look like one connected shape with a
    // matching outline. box-shadow on arrow is accepted syntactically; GTK may
    // render it on the arrow or ignore it, but the same value on both nodes
    // ensures visual consistency.
    let shadow = if theme == "marco-theme-dark" {
        "0 4px 16px rgba(0, 0, 0, 0.50), 0 1px 4px rgba(0, 0, 0, 0.30)"
    } else {
        "0 4px 12px rgba(0, 0, 0, 0.12), 0 1px 3px rgba(0, 0, 0, 0.08)"
    };

    let (chip_text, chip_bg, section_label_color) = if theme == "marco-theme-dark" {
        ("#b8bec8", "#3a3d47", "#7a8190")
    } else {
        ("#4a5568", "#edf0f5", "#8892a0")
    };

    format!(
        r#"
/* {theme} - toolbar insertion popover */

.{theme} popover.marco-link-popover {{
    background: transparent;
}}

/* Arrow AND contents share the same bg + border + shadow so they render as
   one connected bubble shape. This mirrors how system themes (Yaru, Adwaita)
   style popovers: same rule block for both nodes, contents-only overrides below. */
.{theme} popover.marco-link-popover > arrow,
.{theme} popover.marco-link-popover > contents {{
    background-color: {bg};
    border: 1px solid {border};
    box-shadow: {shadow};
}}

/* Contents-only: rounded corners (arrow cannot have border-radius) */
.{theme} popover.marco-link-popover > contents {{
    border-radius: 8px;
}}

/* Section title label colour */
.{theme} popover.marco-link-popover .marco-dialog-section-label {{
    color: {text};
}}

/* Separator */
.{theme} .marco-link-popover-separator {{
    background-color: {separator};
}}

/* ── Hover-insight popover — themed colours ───────────────────────────── */

.{theme} popover.marco-link-popover .marco-hover-title {{
    color: {text};
}}

.{theme} popover.marco-link-popover .marco-hover-code-chip {{
    color: {chip_text};
    background-color: {chip_bg};
}}

.{theme} popover.marco-link-popover .marco-hover-section-label {{
    color: {section_label_color};
}}

.{theme} popover.marco-link-popover .marco-hover-body-text {{
    color: {text};
}}

.{theme} popover.marco-link-popover .marco-hover-separator {{
    background-color: {separator};
}}
"#,
        theme = theme,
        bg = bg,
        border = border,
        shadow = shadow,
        text = text,
        separator = separator,
        chip_text = chip_text,
        chip_bg = chip_bg,
        section_label_color = section_label_color,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_popover_css_generation() {
        let css = generate_css();
        assert!(!css.is_empty());
        assert!(css.contains("marco-link-popover"));
        assert!(css.contains("marco-theme-light"));
        assert!(css.contains("marco-theme-dark"));
        assert!(css.contains("marco-dialog-section-label"));
        // arrow and contents must be styled together (mirrors Yaru/system theme pattern)
        assert!(css.contains("popover.marco-link-popover > arrow"));
        assert!(css.contains("popover.marco-link-popover > contents"));
        // Light uses #ffffff, dark uses #2d2d30 (gutter colour)
        assert!(css.contains(LIGHT_POPOVER_BG));
        assert!(css.contains(DARK_POPOVER_BG));
        // Both nodes share box-shadow for visual consistency
        assert!(css.contains("box-shadow"));
        assert!(css.contains("border: 1px solid"));
        // All named popover classes must also be styled
        for class in NAMED_POPOVER_CLASSES {
            assert!(css.contains(class), "missing CSS for {class}");
            assert!(
                css.contains(&format!("popover.{class} > arrow")),
                "missing arrow rule for {class}"
            );
        }
    }
}
