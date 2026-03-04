//! Reusable textfield CSS (single-line + resizable multiline).
//!
//! Classes:
//! - `entry.marco-textfield-entry`
//! - `scrolledwindow.marco-textfield-scroll`
//! - `textview.marco-textfield-view`
//! - `textview.marco-textfield-view text`

use super::constants::{DARK_PALETTE, LIGHT_PALETTE, TOOLBAR_BORDER_RADIUS};

/// Generate themed CSS for reusable text field widgets.
pub fn generate_css() -> String {
    let mut css = String::with_capacity(4096);
    let disabled_texture_light = "data:image/svg+xml;utf8,%3Csvg%20xmlns='http://www.w3.org/2000/svg'%20width='24'%20height='24'%20viewBox='0%200%2024%2024'%20fill='none'%20stroke='%23b3b8bf'%20stroke-width='1'%20stroke-linecap='square'%20shape-rendering='crispEdges'%3E%3Cpath%20d='M-12,24L24,-12M-6,24L24,-6M0,24L24,0M6,24L24,6M12,24L24,12M18,24L24,18M24,24L24,24'/%3E%3C/svg%3E";
    let disabled_texture_dark = "data:image/svg+xml;utf8,%3Csvg%20xmlns='http://www.w3.org/2000/svg'%20width='24'%20height='24'%20viewBox='0%200%2024%2024'%20fill='none'%20stroke='%236b7280'%20stroke-width='1'%20stroke-linecap='square'%20shape-rendering='crispEdges'%3E%3Cpath%20d='M-12,24L24,-12M-6,24L24,-6M0,24L24,0M6,24L24,6M12,24L24,12M18,24L24,18M24,24L24,24'/%3E%3C/svg%3E";

    css.push_str(
        r#"
    /* Shared dimensions */
    entry.marco-textfield-entry {
        min-height: 26px;
        padding: 3px 8px;
        border-radius: 6px;
        font-size: 12px;
    }

    scrolledwindow.marco-textfield-scroll {
        border-radius: 6px;
        min-height: 96px;
    }

    textview.marco-textfield-view {
        background: transparent;
        border: none;
        padding: 6px 8px;
        font-size: 12px;
    }

    textview.marco-textfield-view:focus {
        outline: none;
    }
"#,
    );

    css.push_str(&format!(
        r#"
    /* Light theme */
    .marco-theme-light entry.marco-textfield-entry {{
        background: transparent;
        color: {fg};
        border: 1px solid {border};
        caret-color: {fg};
        outline: none;
    }}

    .marco-theme-light entry.marco-textfield-entry:focus {{
        border-color: {accent};
        box-shadow: 0 0 0 1px {accent};
        outline: none;
    }}

    /* Custom admonition fields: disabled texture state (light) */
    .marco-theme-light entry.marco-textfield-entry.marco-admonition-custom-field:disabled {{
        color: {disabled_fg};
        border: 1px solid {disabled_border};
        background-color: {disabled_bg};
        background-image: url("{disabled_texture_light}");
        background-repeat: repeat;
        background-size: 24px 24px;
        background-position: 0 0;
        box-shadow: none;
    }}

    .marco-theme-light scrolledwindow.marco-textfield-scroll {{
        background: transparent;
        border: 1px solid {border};
    }}

    .marco-theme-light scrolledwindow.marco-textfield-scroll:focus-within {{
        border-color: {accent};
        box-shadow: 0 0 0 1px {accent};
    }}

    .marco-theme-light textview.marco-textfield-view,
    .marco-theme-light textview.marco-textfield-view text {{
        color: {fg};
        background: transparent;
        caret-color: {fg};
    }}

    /* EntryCompletion popup for emoji search entry - Light theme */
    entry.marco-textfield-entry.marco-theme-light .view,
    entry.marco-textfield-entry.marco-theme-light treeview.view {{
        background: {popup_bg};
        color: {fg};
    }}

    entry.marco-textfield-entry.marco-theme-light .view:selected,
    entry.marco-textfield-entry.marco-theme-light treeview.view:selected {{
        background: {accent};
        color: #ffffff;
    }}

    entry.marco-textfield-entry.marco-theme-light .view .cell,
    entry.marco-textfield-entry.marco-theme-light treeview.view cell {{
        color: {fg};
        background: transparent;
    }}

    entry.marco-textfield-entry.marco-theme-light .view .cell:selected,
    entry.marco-textfield-entry.marco-theme-light treeview.view cell:selected {{
        color: #ffffff;
        background: {accent};
    }}
"#,
        fg = LIGHT_PALETTE.titlebar_foreground,
        border = LIGHT_PALETTE.titlebar_border,
        accent = LIGHT_PALETTE.toolbar_button_hover_border,
        disabled_bg = LIGHT_PALETTE.toolbar_button_disabled_bg,
        disabled_fg = LIGHT_PALETTE.toolbar_button_disabled,
        disabled_border = LIGHT_PALETTE.toolbar_button_disabled_border,
        disabled_texture_light = disabled_texture_light,
        popup_bg = LIGHT_PALETTE.toolbar_popover_bg,
    ));

    css.push_str(&format!(
        r#"
    /* Dark theme */
    .marco-theme-dark entry.marco-textfield-entry {{
        background: transparent;
        color: {fg};
        border: 1px solid {border};
        caret-color: {fg};
        outline: none;
    }}

    .marco-theme-dark entry.marco-textfield-entry:focus {{
        border-color: {accent};
        box-shadow: 0 0 0 1px {accent};
        outline: none;
    }}

    /* Custom admonition fields: disabled texture state (dark) */
    .marco-theme-dark entry.marco-textfield-entry.marco-admonition-custom-field:disabled {{
        color: {disabled_fg};
        border: 1px solid {disabled_border};
        background-color: {disabled_bg};
        background-image: url("{disabled_texture_dark}");
        background-repeat: repeat;
        background-size: 24px 24px;
        background-position: 0 0;
        box-shadow: none;
    }}

    .marco-theme-dark scrolledwindow.marco-textfield-scroll {{
        background: transparent;
        border: 1px solid {border};
    }}

    .marco-theme-dark scrolledwindow.marco-textfield-scroll:focus-within {{
        border-color: {accent};
        box-shadow: 0 0 0 1px {accent};
    }}

    .marco-theme-dark textview.marco-textfield-view,
    .marco-theme-dark textview.marco-textfield-view text {{
        color: {fg};
        background: transparent;
        caret-color: {fg};
    }}

    /* EntryCompletion popup for emoji search entry - Dark theme */
    entry.marco-textfield-entry.marco-theme-dark .view,
    entry.marco-textfield-entry.marco-theme-dark treeview.view {{
        background: {popup_bg};
        color: {fg};
    }}

    entry.marco-textfield-entry.marco-theme-dark .view:selected,
    entry.marco-textfield-entry.marco-theme-dark treeview.view:selected {{
        background: {accent};
        color: #ffffff;
    }}

    entry.marco-textfield-entry.marco-theme-dark .view .cell,
    entry.marco-textfield-entry.marco-theme-dark treeview.view cell {{
        color: {fg};
        background: transparent;
    }}

    entry.marco-textfield-entry.marco-theme-dark .view .cell:selected,
    entry.marco-textfield-entry.marco-theme-dark treeview.view cell:selected {{
        color: #ffffff;
        background: {accent};
    }}
"#,
        fg = DARK_PALETTE.titlebar_foreground,
        border = DARK_PALETTE.titlebar_border,
        accent = DARK_PALETTE.toolbar_button_hover_border,
        disabled_bg = DARK_PALETTE.toolbar_button_disabled_bg,
        disabled_fg = DARK_PALETTE.toolbar_button_disabled,
        disabled_border = DARK_PALETTE.toolbar_button_disabled_border,
        disabled_texture_dark = disabled_texture_dark,
        popup_bg = DARK_PALETTE.toolbar_popover_bg,
    ));

    css.push_str(&format!(
        r#"
    /* Reuse shared border radius token to keep style consistency. */
    entry.marco-textfield-entry,
    scrolledwindow.marco-textfield-scroll {{
        border-radius: {radius};
    }}
"#,
        radius = TOOLBAR_BORDER_RADIUS,
    ));

    css
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_textfield_css_generation() {
        let css = generate_css();
        assert!(!css.is_empty());
        assert!(css.contains("marco-textfield-entry"));
        assert!(css.contains("marco-textfield-scroll"));
        assert!(css.contains("marco-textfield-view"));
        assert!(css.contains(".marco-theme-light"));
        assert!(css.contains(".marco-theme-dark"));
    }
}
