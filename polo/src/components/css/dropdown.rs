//! Dropdown CSS Generation
//!
//! Generates CSS for Polo's theme selection dropdown.
//!
//! ## Components Styled
//!
//! - `dropdown.polo-theme-dropdown`: The dropdown widget itself
//! - `dropdown > button`: The dropdown trigger button
//! - `dropdown > popover`: The popup menu container
//! - `dropdown > popover > contents`: The visible popup background
//! - `dropdown > popover listview`: The list of theme options
//! - `dropdown > popover listview > row`: Individual theme items
//!
//! ## GTK4 Dropdown Structure
//!
//! GTK4 dropdowns have complex nested structure that requires targeting
//! multiple levels to properly style the popup menu.

use super::constants::*;

/// Generate complete dropdown CSS for both themes
pub fn generate_css() -> String {
    let mut css = String::with_capacity(3072);
    
    // Base dropdown sizing (theme-independent)
    css.push_str(&format!(
        r#"
    /* Dropdown styles - Match Marco's flat design */
    dropdown.polo-theme-dropdown {{
        min-width: {min_width};
        min-height: {min_height};
        font-size: {font_size};
    }}
"#,
        min_width = DROPDOWN_MIN_WIDTH,
        min_height = BUTTON_MIN_HEIGHT,
        font_size = BUTTON_FONT_SIZE,
    ));
    
    // Light theme
    css.push_str(&generate_theme_css("marco-theme-light", &LIGHT_PALETTE));
    
    // Dark theme
    css.push_str(&generate_theme_css("marco-theme-dark", &DARK_PALETTE));
    
    css
}

/// Generate theme-specific dropdown CSS
fn generate_theme_css(theme_class: &str, palette: &ColorPalette) -> String {
    format!(
        r#"
    /* Dropdown button - {theme} */
    .{theme} dropdown.polo-theme-dropdown > button {{
        background: transparent;
        color: {foreground};
        border: 1px solid {border};
        border-radius: {radius};
        padding: {padding};
        transition: {transition};
    }}
    
    .{theme} dropdown.polo-theme-dropdown > button:hover {{
        background: transparent;
        color: {hover_accent};
        border-color: {border_hover};
    }}
    
    .{theme} dropdown.polo-theme-dropdown > button:active {{
        background: transparent;
        color: {active_text};
        border-color: {border_hover};
    }}
    
    .{theme} dropdown.polo-theme-dropdown > button label {{
        color: inherit;
    }}
    
    /* Dropdown popover - {theme} - Target the popover.background class */
    .{theme} dropdown.polo-theme-dropdown > popover.background,
    .{theme} dropdown.polo-theme-dropdown > popover {{
        background: transparent;
        border: none;
        box-shadow: none;
    }}
    
    /* Style the contents node - this is the visible part */
    .{theme} dropdown.polo-theme-dropdown > popover > contents {{
        background: {popover_bg};
        color: {foreground};
        border: none;
        box-shadow: {shadow};
        border-radius: {radius};
    }}
    
    /* Target the listview inside the popover */
    .{theme} dropdown.polo-theme-dropdown > popover listview {{
        background: {popover_bg};
        color: {foreground};
        border: none;
        border-radius: {radius};
    }}
    
    /* Target individual rows (list items) */
    .{theme} dropdown.polo-theme-dropdown > popover listview > row {{
        background: transparent;
        color: {foreground};
        border: none;
        padding: {item_padding};
    }}
    
    .{theme} dropdown.polo-theme-dropdown > popover listview > row:hover {{
        background: {item_hover_bg};
    }}
    
    /* Target labels inside rows */
    .{theme} dropdown.polo-theme-dropdown > popover listview > row label {{
        color: {foreground};
    }}
"#,
        theme = theme_class,
        foreground = palette.foreground,
        border = palette.border,
        radius = BORDER_RADIUS,
        padding = BUTTON_PADDING,  // Changed from MODE_TOGGLE_PADDING to BUTTON_PADDING
        transition = STANDARD_TRANSITION,
        hover_accent = palette.hover_accent,
        border_hover = palette.border_hover,
        active_text = palette.active_text,
        popover_bg = palette.popover_bg,
        shadow = if theme_class.contains("light") {
            "0 2px 6px rgba(0, 0, 0, 0.15)"
        } else {
            "0 2px 6px rgba(0, 0, 0, 0.4)"
        },
        item_padding = DROPDOWN_ITEM_PADDING,
        item_hover_bg = palette.item_hover_bg,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_dropdown_css_generation() {
        let css = generate_css();
        
        // Verify dropdown classes present
        assert!(css.contains("dropdown.polo-theme-dropdown"));
        assert!(css.contains("> button"));
        assert!(css.contains("> popover"));
        assert!(css.contains("> contents"));
        assert!(css.contains("listview"));
        
        // Verify both themes present
        assert!(css.contains(".marco-theme-light"));
        assert!(css.contains(".marco-theme-dark"));
        
        // Verify essential properties
        assert!(css.contains("min-width: 150px"));
        assert!(css.contains("border-radius: 6px"));
        assert!(css.contains(":hover"));
        
        // Verify substantial output
        assert!(css.len() > 1000);
    }

    #[test]
    fn smoke_test_popover_structure() {
        let css = generate_css();
        
        // Verify complex popover targeting
        assert!(css.contains("popover.background"));
        assert!(css.contains("popover > contents"));
        assert!(css.contains("popover listview"));
        assert!(css.contains("listview > row"));
        assert!(css.contains("row:hover"));
    }

    #[test]
    fn smoke_test_theme_colors_differ() {
        let css = generate_css();
        
        // Light should use #ffffff, dark should use #2d2d2d for popover
        assert!(css.contains("#ffffff"));  // Light popover bg
        assert!(css.contains("#2d2d2d"));  // Dark popover bg
        
        // Different hover backgrounds
        assert!(css.contains("#e8e8e8"));  // Light item hover
        assert!(css.contains("#3d3d3d"));  // Dark item hover
    }

    #[test]
    fn smoke_test_theme_css_structure() {
        let light_css = generate_theme_css("marco-theme-light", &LIGHT_PALETTE);
        
        // Verify selector structure
        assert!(light_css.contains(".marco-theme-light dropdown"));
        assert!(light_css.contains("> button"));
        assert!(light_css.contains("> popover"));
        
        // Verify uses palette colors
        assert!(light_css.contains(LIGHT_PALETTE.foreground));
        assert!(light_css.contains(LIGHT_PALETTE.popover_bg));
        assert!(light_css.contains(LIGHT_PALETTE.item_hover_bg));
    }
}
