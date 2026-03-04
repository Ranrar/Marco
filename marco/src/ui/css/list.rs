//! List and list-like widget CSS
//!
//! Styles GTK4 ListView usage inside dialogs, including the tabs dialog.

pub fn generate_css() -> String {
    let mut css = String::with_capacity(2048);

    css.push_str(&generate_base_css());
    css.push_str(&generate_theme_css("marco-theme-light"));
    css.push_str(&generate_theme_css("marco-theme-dark"));

    css
}

fn generate_base_css() -> String {
    r#"
    /* No outer border — blends into the dialog background. */
    .marco-tabs-frame {
        border-radius: 8px;
        padding: 2px;
        border: none;
        box-shadow: none;
    }

    .marco-tabs-listview {
        min-height: 96px;
    }

    /* Row is the visual "pill" unit; the inner Entry/button are invisible. */
    .marco-tabs-row {
        padding: 1px 2px;
        border-radius: 6px;
        transition: background 100ms ease;
    }

    /* Entry is invisible — the row background is the visual unit.
     * Remove all borders, focus ring, and background so the row
     * colour shows through uninterrupted. */
    entry.marco-tabs-name-entry {
        min-height: 28px;
        border: none;
        box-shadow: none;
        outline: none;
        background: transparent;
    }

    entry.marco-tabs-name-entry > text {
        background: transparent;
    }

    /* Delete button: icon-only, borderless, blends into the row. */
    .marco-tabs-delete-btn {
        min-width: 30px;
        min-height: 28px;
        border-radius: 6px;
        padding: 0 6px;
        border: none;
    }

    .marco-tabs-action-btn {
        min-height: 30px;
    }

    .marco-tabs-content-title {
        font-size: 13px;
        font-weight: 600;
    }

    /* Empty-state label shown when no tabs have been added yet. */
    .marco-tabs-empty-label {
        font-size: 13px;
        opacity: 0.55;
        padding: 24px 0;
    }

    /* Frame title label ("Tabs" rendered by GTK at the top edge of the frame). */
    .marco-tabs-frame > label {
        font-size: 12px;
        font-weight: 600;
        padding: 0 4px;
    }

    /* Separator between content title and content textarea. */
    .marco-tabs-separator {
        margin: 0;
        min-height: 1px;
    }
"#
    .to_string()
}

fn generate_theme_css(theme: &str) -> String {
    // Colours defined per-theme for readability.
    let (
        frame_bg,
        list_bg,
        row_hover_bg,
        row_selected_bg,
        row_selected_text,
        entry_text,
        entry_caret,
        empty_label_color,
        delete_hover_bg,
        separator_color,
    ) = if theme.contains("light") {
        (
            "#FAFAFA", // frame_bg: matches dialog background
            "#FAFAFA", // list_bg: same as dialog, no contrast wash
            "#DDEAF8", // row_hover_bg: clearly visible cool-blue tint
            "#C8DDF8", // row_selected_bg: a step richer than hover, clearly selected
            "#1A3150", // row_selected_text: deep navy, high contrast on C8DDF8
            "#2c3e50", // entry_text: dark slate
            "#2c3e50", // entry_caret: matches text
            "#5a6a7a", // empty_label_color: muted slate, readable but secondary
            "#E2EAF4", // delete_hover_bg: gentle blue wash matching row tint
            "#ddd",    // separator_color: matches light toolbar_border
        )
    } else {
        (
            "#1E1E1E", // frame_bg: matches dialog background
            "#1E1E1E", // list_bg: same as dialog
            "#2A3142", // row_hover_bg
            "#1E3A5F", // row_selected_bg
            "#C8DEFF", // row_selected_text: light blue on dark bg
            "#e0e0e0", // entry_text
            "#e0e0e0", // entry_caret
            "#9098a8", // empty_label_color
            "#2E3A50", // delete_hover_bg
            "#3c3c3c", // separator_color: matches dark toolbar_border
        )
    };

    format!(
        r#"
    /* Frame: matches dialog background, no visual lift */
    .{theme} .marco-tabs-frame,
    .{theme}.marco-dialog .marco-tabs-frame {{
        background: {frame_bg};
        border: none;
        box-shadow: none;
    }}

    .{theme} .marco-tabs-listview,
    .{theme}.marco-dialog .marco-tabs-listview {{
        background: {list_bg};
    }}

    .{theme} .marco-tabs-listview row,
    .{theme}.marco-dialog .marco-tabs-listview row {{
        background: transparent;
        border-radius: 6px;
        color: {entry_text};
        margin-bottom: 3px;
    }}

    /* Hover */
    .{theme} .marco-tabs-listview row:hover,
    .{theme}.marco-dialog .marco-tabs-listview row:hover {{
        background: {row_hover_bg};
    }}

    /* Selected */
    .{theme} .marco-tabs-listview row:selected,
    .{theme}.marco-dialog .marco-tabs-listview row:selected {{
        background: {row_selected_bg};
        color: {row_selected_text};
    }}

    /* Entry inside row: transparent, inherits row text/caret colour */
    .{theme} entry.marco-tabs-name-entry,
    .{theme}.marco-dialog entry.marco-tabs-name-entry {{
        background: transparent;
        color: {entry_text};
        caret-color: {entry_caret};
        border: none;
        box-shadow: none;
    }}

    .{theme} entry.marco-tabs-name-entry > text,
    .{theme}.marco-dialog entry.marco-tabs-name-entry > text {{
        background: transparent;
        color: {entry_text};
    }}

    /* Entry text colour inside a selected row */
    .{theme} .marco-tabs-listview row:selected entry.marco-tabs-name-entry,
    .{theme}.marco-dialog .marco-tabs-listview row:selected entry.marco-tabs-name-entry {{
        color: {row_selected_text};
        caret-color: {row_selected_text};
    }}

    .{theme} .marco-tabs-listview row:selected entry.marco-tabs-name-entry > text,
    .{theme}.marco-dialog .marco-tabs-listview row:selected entry.marco-tabs-name-entry > text {{
        color: {row_selected_text};
    }}

    /* Delete button: borderless, blends into the row */
    .{theme} .marco-tabs-delete-btn,
    .{theme}.marco-dialog .marco-tabs-delete-btn {{
        background: transparent;
        color: {entry_text};
        border: none;
    }}

    .{theme} .marco-tabs-delete-btn:hover,
    .{theme}.marco-dialog .marco-tabs-delete-btn:hover {{
        background: {delete_hover_bg};
    }}

    .{theme} .marco-tabs-content-title,
    .{theme}.marco-dialog .marco-tabs-content-title {{
        color: {entry_text};
    }}

    /* Empty-state placeholder */
    .{theme} .marco-tabs-empty-label,
    .{theme}.marco-dialog .marco-tabs-empty-label {{
        color: {empty_label_color};
    }}

    /* Frame title label */
    .{theme} .marco-tabs-frame > label,
    .{theme}.marco-dialog .marco-tabs-frame > label {{
        color: {entry_text};
    }}

    /* Separator below the content "Content for:" label */
    .{theme} .marco-tabs-separator,
    .{theme}.marco-dialog .marco-tabs-separator {{
        background: {separator_color};
        color: {separator_color};
        min-height: 1px;
    }}
"#,
        theme = theme,
        frame_bg = frame_bg,
        list_bg = list_bg,
        row_hover_bg = row_hover_bg,
        row_selected_bg = row_selected_bg,
        row_selected_text = row_selected_text,
        entry_text = entry_text,
        entry_caret = entry_caret,
        empty_label_color = empty_label_color,
        delete_hover_bg = delete_hover_bg,
        separator_color = separator_color,
    )
}
