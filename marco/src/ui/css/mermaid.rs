//! Mermaid dialog CSS
//!
//! Adds theme-aware styles for:
//! - `textview.marco-mermaid-source` — monospace source editor
//! - `.marco-mermaid-error-label`    — inline error message
//! - `scrolledwindow.marco-mermaid-preview-scroll` — preview container border

use super::constants::{DARK_PALETTE, LIGHT_PALETTE};

pub fn generate_css() -> String {
    let mut css = String::with_capacity(1024);

    // ── Monospace source editor ────────────────────────────────────────────────
    // Inherits all other styles from marco-textfield-view; only overrides font.
    css.push_str(
        r#"
    textview.marco-textfield-view.marco-mermaid-source,
    textview.marco-textfield-view.marco-mermaid-source text {
        font-family: "Fira Code", "JetBrains Mono", "Iosevka", monospace;
        font-size: 12px;
    }
"#,
    );

    // ── Per-theme rules ────────────────────────────────────────────────────────
    for (theme, p) in [
        ("marco-theme-light", LIGHT_PALETTE),
        ("marco-theme-dark", DARK_PALETTE),
    ] {
        // Error label colour: a readable soft-red for both themes.
        let (error_fg, error_bg, error_border) = if theme == "marco-theme-light" {
            ("#b91c1c", "#fef2f2", "#fca5a5")
        } else {
            ("#fca5a5", "#3b0a0a", "#7f1d1d")
        };

        css.push_str(&format!(
            r#"
    .{theme} .marco-mermaid-error-label {{
        color: {error_fg};
        background: {error_bg};
        border: 1px solid {error_border};
        border-radius: 6px;
        padding: 4px 8px;
        font-size: 11px;
    }}

    .{theme} scrolledwindow.marco-mermaid-preview-scroll {{
        border: 1px solid {border};
        border-radius: 6px;
        background: transparent;
    }}

    .{theme} scrolledwindow.marco-mermaid-preview-scroll:focus-within {{
        border-color: {accent};
    }}
"#,
            theme = theme,
            error_fg = error_fg,
            error_bg = error_bg,
            error_border = error_border,
            border = p.titlebar_border,
            accent = p.toolbar_button_hover_border,
        ));
    }

    css
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_mermaid_css_generation() {
        let css = generate_css();
        assert!(css.contains("marco-mermaid-source"));
        assert!(css.contains("marco-mermaid-error-label"));
        assert!(css.contains("marco-theme-light"));
        assert!(css.contains("marco-theme-dark"));
    }
}
