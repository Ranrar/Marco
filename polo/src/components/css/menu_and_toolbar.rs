//! Menu Bar and Toolbar CSS Generation
//!
//! Generates CSS for Polo's text-only menu bar and icon toolbar.
//!
//! ## Menu Bar (`.polo-menubar`)
//! Horizontal bar below the titlebar with text-only [File] [View] buttons.
//!
//! ## Toolbar (`.polo-toolbar`)
//! Horizontal icon toolbar with SVG icon buttons.

use super::constants::*;

/// Generate all menu bar and toolbar CSS
pub fn generate_css() -> String {
    let mut css = String::with_capacity(4096);

    // Theme-independent base styles
    css.push_str(MENU_AND_TOOLBAR_BASE_CSS);

    // Light theme styles
    css.push_str(&generate_theme_css("marco-theme-light", &LIGHT_PALETTE));

    // Dark theme styles
    css.push_str(&generate_theme_css("marco-theme-dark", &DARK_PALETTE));

    css
}

/// Theme-independent base CSS for menu bar and toolbar
const MENU_AND_TOOLBAR_BASE_CSS: &str = r#"
/* ── Polo Menu Bar ─────────────────────────────────────────────────── */

/* GtkBox that holds [File][View] inside the headerbar.
   No bottom border — the titlebar has none in Marco either.
   The only visible dividing line comes from the toolbar below. */
.polo-menubar {
    padding: 0 2px;
    min-height: 24px;
}

/* Menu bar text buttons — exactly matches Marco's .menu-button geometry */
.polo-menu-btn {
    padding: 2px 8px;
    margin: 4px 1px;
    min-height: 16px;
    border-radius: 5px;
    border: none;
    background: transparent;
    font-size: 12px;
    font-weight: 400;
    transition: background 80ms ease, color 80ms ease;
    box-shadow: none;
    outline: none;
}

.polo-menu-btn:focus {
    outline: none;
    box-shadow: none;
    background: transparent;
}

/* Menu popover — transparent outer node, contents handles the visual.
   Matches Marco's marco-menu-popover pattern (popover.rs). */
popover.polo-menu-popover {
    background: transparent;
    box-shadow: none;
    padding: 0;
}

popover.polo-menu-popover > contents {
    border-radius: 8px;
    padding: 4px;
    min-width: 160px;
}

/* Items inside menu popovers — matches Marco's popover.menu modelbutton sizing exactly */
.polo-menu-item {
    padding: 4px 8px;
    margin: 1px 2px;
    min-height: 20px;
    border-radius: 6px;
    border: none;
    background: transparent;
    font-size: 12px;
    font-weight: 500;
    transition: background 0.15s, color 0.15s;
}

.polo-menu-item:focus {
    outline: none;
    box-shadow: none;
}

/* PopoverMenu (gio::Menu-based) modelbutton items — same geometry as .polo-menu-item */
popover.polo-menu-popover modelbutton {
    padding: 4px 8px;
    margin: 1px 2px;
    min-height: 20px;
    border-radius: 6px;
    background: transparent;
    font-size: 12px;
    font-weight: 500;
    outline: none;
    box-shadow: none;
    transition: background 0.15s, color 0.15s;
}

/* Submenu arrow indicator on model buttons — padding-right (GTK4 does not support padding-end) */
popover.polo-menu-popover modelbutton.flat.has-indicator {
    padding-right: 24px;
}

/* Section separators inside PopoverMenu — same as .polo-menu-separator */
popover.polo-menu-popover > contents separator {
    min-height: 1px;
    margin: 4px 4px;
}

/* Menu separator */
.polo-menu-separator {
    min-height: 1px;
    margin: 4px 4px;
}

/* Theme checkmark item */
.polo-theme-item {
    padding: 3px 8px;
    margin: 1px 2px;
    min-height: 18px;
    border-radius: 6px;
    border: none;
    background: transparent;
    font-size: 12px;
    font-weight: 500;
    transition: background 0.15s, color 0.15s;
}

.polo-theme-item:focus {
    outline: none;
    box-shadow: none;
}

/* ── Polo Toolbar ───────────────────────────────────────────────────── */

/* Same padding/height as Marco's .toolbar */
.polo-toolbar {
    padding: 2px 5px;
    border-bottom-width: 1px;
    border-bottom-style: solid;
}

/* Toolbar icon buttons — same geometry as Marco's .toolbar-btn */
.polo-toolbar-btn {
    padding: 1px 2px;
    min-height: 18px;
    min-width: 18px;
    margin: 0;
    border-radius: 6px;
    border: none;
    background: transparent;
    transition: background 0.15s, color 0.15s;
}

.polo-toolbar-btn:focus {
    outline: none;
    box-shadow: none;
}

/* Toolbar vertical separator — same as Marco's .toolbar-separator */
.polo-toolbar-separator {
    min-width: 1px;
    opacity: 0.65;
    margin: 0 5px;
}
"#;

/// Generate theme-specific CSS for menu bar and toolbar.
/// Uses the same color tokens as Marco's toolbar.css and menu.css.
fn generate_theme_css(theme_class: &str, palette: &ColorPalette) -> String {
    // Matches Marco's popover.rs: light=#ffffff dark=#2d2d2d
    let popover_bg = if theme_class.contains("light") {
        "#ffffff"
    } else {
        "#2d2d2d"
    };
    // Matches Marco's shadow values from popover.rs
    let shadow = if theme_class.contains("light") {
        "0 4px 12px rgba(0, 0, 0, 0.12), 0 1px 3px rgba(0, 0, 0, 0.08)"
    } else {
        "0 4px 16px rgba(0, 0, 0, 0.50), 0 1px 4px rgba(0, 0, 0, 0.30)"
    };
    format!(
        r#"
/* ── {theme} ── Menu Bar ────────────────────────────────────────────── */
.{theme} .polo-menubar {{
    background: {titlebar_bg};
    border-bottom-color: {titlebar_border};
}}

.{theme} .polo-menu-btn {{
    color: {fg};
}}

/* Hover uses the same semi-transparent rgba as Marco's .menu-button:hover —
   no text color change, just a subtle background tint. */
.{theme} .polo-menu-btn:hover {{
    background: rgba(90, 93, 94, 0.31);
    color: {fg};
}}

.{theme} .polo-menu-btn:active {{
    background: rgba(90, 93, 94, 0.45);
    color: {fg};
}}

/* ── {theme} ── Menu Popover (arrow + contents, same as Marco's marco-menu-popover) */
.{theme} popover.polo-menu-popover {{
    background: transparent;
}}

.{theme} popover.polo-menu-popover > arrow,
.{theme} popover.polo-menu-popover > contents {{
    background-color: {popover_bg};
    border: 1px solid {titlebar_border};
    box-shadow: {shadow};
}}

.{theme} popover.polo-menu-popover > contents {{
    border-radius: 8px;
}}

.{theme} .polo-menu-item {{
    color: {fg};
}}

.{theme} .polo-menu-item:hover {{
    background: {item_hover_bg};
    color: {menu_hover};
}}

.{theme} .polo-menu-item:active {{
    background: {item_hover_bg};
    color: {fg};
}}

.{theme} .polo-menu-item:disabled {{
    color: {menu_disabled};
    opacity: 0.6;
}}

/* PopoverMenu modelbutton colors (gio::Menu-based File menu) */
.{theme} popover.polo-menu-popover modelbutton {{
    color: {fg};
}}

.{theme} popover.polo-menu-popover modelbutton:hover {{
    background: {item_hover_bg};
    color: {fg};
}}

.{theme} popover.polo-menu-popover modelbutton:active {{
    background: {item_hover_bg};
    color: {fg};
}}

.{theme} popover.polo-menu-popover modelbutton:disabled {{
    color: {menu_disabled};
    opacity: 0.6;
}}

.{theme} .polo-menu-separator {{
    background: {titlebar_border};
}}

/* PopoverMenu section separators (GTK separator nodes, not widget-based) */
.{theme} popover.polo-menu-popover > contents separator {{
    background: {titlebar_border};
}}

.{theme} .polo-theme-item {{
    color: {fg};
}}

.{theme} .polo-theme-item:hover {{
    background: {item_hover_bg};
    color: {menu_hover};
}}

/* ── {theme} ── Toolbar (uses Marco's toolbar palette, not titlebar palette) */
.{theme} .polo-toolbar {{
    background: {toolbar_bg};
    border-bottom-color: {toolbar_border};
}}

.{theme} .polo-toolbar-btn {{
    color: {toolbar_button};
}}

.{theme} .polo-toolbar-btn:hover {{
    background: transparent;
    color: {toolbar_button_hover};
}}

.{theme} .polo-toolbar-btn:active {{
    background: transparent;
    color: {toolbar_button_active};
}}

.{theme} .polo-toolbar-btn:disabled {{
    background: transparent;
    opacity: 0.45;
}}

.{theme} .polo-toolbar-separator {{
    background: {toolbar_separator};
}}
"#,
        theme = theme_class,
        titlebar_bg = palette.titlebar_bg,
        titlebar_border = palette.border,
        fg = palette.foreground,
        menu_hover = palette.menu_hover,
        item_hover_bg = palette.item_hover_bg,
        popover_bg = popover_bg,
        shadow = shadow,
        toolbar_bg = palette.toolbar_bg,
        toolbar_border = palette.toolbar_border,
        toolbar_button = palette.toolbar_button,
        toolbar_button_hover = palette.toolbar_button_hover,
        toolbar_button_active = palette.toolbar_button_active,
        toolbar_separator = palette.toolbar_separator,
        menu_disabled = palette.menu_disabled,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_generate_css() {
        let css = generate_css();
        assert!(!css.is_empty());
        assert!(css.contains(".polo-menubar"));
        assert!(css.contains(".polo-toolbar"));
        assert!(css.contains(".polo-menu-btn"));
        assert!(css.contains(".polo-toolbar-btn"));
        assert!(css.contains("marco-theme-light"));
        assert!(css.contains("marco-theme-dark"));
    }
}
