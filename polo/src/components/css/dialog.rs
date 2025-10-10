//! Dialog CSS Generation
//!
//! Generates CSS for custom dialog windows that match Polo's theme.
//! Aligned with Marco's compact sizing standards (24px buttons, 20px padding).
//!
//! ## Components Styled
//!
//! - `.polo-dialog`: Main dialog window styling
//! - `.polo-dialog-content`: Content area with proper spacing (20px padding, 340px min-width)
//! - `.polo-dialog-title`: Dialog title text (15px font, 600 weight)
//! - `.polo-dialog-message`: Dialog message/description text (13px font)
//! - `.polo-dialog-button-box`: Action button container
//! - `.polo-dialog-button`: Action buttons (24px height, matches Marco)
//!
//! ## Theme Support
//!
//! Generates rules for both `.marco-theme-light` and `.marco-theme-dark` classes.
//!
//! ## Sizing Standards
//!
//! All sizing constants match Marco's recent compact optimizations:
//! - Button height: 24px (reduced from 32px)
//! - Content padding: 20px (reduced from 24px)
//! - Min-width: 340px (reduced from 400px)
//! - Font sizes: 15px title, 13px message, 12px buttons
//!
//! ## Usage
//!
//! Dialog windows should:
//! 1. Use `gtk4::Window` (not deprecated `Dialog`)
//! 2. Add CSS class: `.add_css_class("polo-dialog")`
//! 3. Add theme class: `.add_css_class("marco-theme-light")` or `"marco-theme-dark"`
//! 4. Use `.transient_for(parent)` for modal behavior
//! 5. Use `.set_modal(true)` for modal dialogs
//! 6. Set `.set_max_width_chars(45)` on label text for proper wrapping

use super::constants::*;

/// Generate complete dialog CSS for both light and dark themes
pub fn generate_css() -> String {
    let mut css = String::with_capacity(2048);
    
    // Base dialog styling (theme-independent)
    css.push_str(&format!(
        r#"
    /* Base dialog window styling */
    .polo-dialog {{
        border-radius: {border_radius};
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
    }}
    
    .polo-dialog-content {{
        padding: {content_padding};
        min-width: {content_min_width};
    }}
    
    .polo-dialog-title {{
        font-size: {title_font_size};
        font-weight: {title_font_weight};
        margin-bottom: 10px;
    }}
    
    .polo-dialog-message {{
        font-size: {message_font_size};
        line-height: 1.5;
        margin-bottom: 16px;
    }}
    
    .polo-dialog-button-box {{
        margin-top: 14px;
        padding: 0;
    }}
    
    .polo-dialog-button {{
        min-width: {button_min_width};
        min-height: {button_min_height};
        padding: {button_padding};
        border-radius: {border_radius};
        font-size: {button_font_size};
        font-weight: {button_font_weight};
        margin: 0 3px;
        transition: {transition};
    }}
    
    .polo-dialog-button:first-child {{
        margin-left: 0;
    }}
    
    .polo-dialog-button:last-child {{
        margin-right: 0;
    }}
"#,
        border_radius = BORDER_RADIUS,
        content_padding = DIALOG_CONTENT_PADDING,
        content_min_width = DIALOG_MIN_CONTENT_WIDTH,
        title_font_size = DIALOG_TITLE_FONT_SIZE,
        title_font_weight = DIALOG_TITLE_FONT_WEIGHT,
        message_font_size = DIALOG_MESSAGE_FONT_SIZE,
        button_min_width = DIALOG_BUTTON_MIN_WIDTH,
        button_min_height = DIALOG_BUTTON_MIN_HEIGHT,
        button_padding = DIALOG_BUTTON_PADDING,
        button_font_size = DIALOG_BUTTON_FONT_SIZE,
        button_font_weight = DIALOG_BUTTON_FONT_WEIGHT,
        transition = STANDARD_TRANSITION,
    ));
    
    // Light theme
    css.push_str(&generate_theme_css("marco-theme-light", &LIGHT_PALETTE));
    
    // Dark theme
    css.push_str(&generate_theme_css("marco-theme-dark", &DARK_PALETTE));
    
    css
}

/// Generate theme-specific dialog CSS
fn generate_theme_css(theme_class: &str, palette: &ColorPalette) -> String {
    format!(
        r#"
    /* {theme} - Dialog styles */
    .{theme} .polo-dialog {{
        background: {window_bg};
        border: 1px solid {border};
    }}
    
    .{theme} .polo-dialog-content {{
        background: {window_bg};
    }}
    
    .{theme} .polo-dialog-title {{
        color: {foreground};
    }}
    
    .{theme} .polo-dialog-message {{
        color: {foreground};
        opacity: 0.9;
    }}
    
    .{theme} .polo-dialog-button {{
        background: transparent;
        color: {foreground};
        border: 1px solid {border};
    }}
    
    .{theme} .polo-dialog-button:hover {{
        background: {item_hover_bg};
        border-color: {border_hover};
        color: {hover_accent};
    }}
    
    .{theme} .polo-dialog-button:active {{
        background: {item_hover_bg};
        color: {active_text};
    }}
    
    .{theme} .polo-dialog-button.primary {{
        background: {border_hover};
        color: #ffffff;
        border-color: {border_hover};
    }}
    
    .{theme} .polo-dialog-button.primary:hover {{
        background: {hover_accent};
        border-color: {hover_accent};
        color: #ffffff;
    }}
    
    .{theme} .polo-dialog-button.primary:active {{
        background: {active_text};
        border-color: {active_text};
        color: #ffffff;
    }}
    
    .{theme} .polo-dialog-button.destructive {{
        color: #dc3545;
        border-color: #dc3545;
    }}
    
    .{theme} .polo-dialog-button.destructive:hover {{
        background: #dc3545;
        color: #ffffff;
    }}
"#,
        theme = theme_class,
        window_bg = palette.window_bg,
        foreground = palette.foreground,
        border = palette.border,
        border_hover = palette.border_hover,
        hover_accent = palette.hover_accent,
        active_text = palette.active_text,
        item_hover_bg = palette.item_hover_bg,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_dialog_css_generation() {
        let css = generate_css();
        
        // Verify basic structure
        assert!(css.contains(".polo-dialog"));
        assert!(css.contains(".polo-dialog-content"));
        assert!(css.contains(".polo-dialog-title"));
        assert!(css.contains(".polo-dialog-message"));
        assert!(css.contains(".polo-dialog-button"));
        assert!(css.contains(".polo-dialog-button-box"));
        
        // Verify both themes present
        assert!(css.contains(".marco-theme-light"));
        assert!(css.contains(".marco-theme-dark"));
        
        // Verify button variants
        assert!(css.contains(".primary"));
        assert!(css.contains(".destructive"));
        
        // Verify essential properties
        assert!(css.contains("border-radius: 6px"));
        assert!(css.contains("min-width: 340px")); // Updated to compact size
        assert!(css.contains("min-height: 24px")); // Updated to compact size
        assert!(css.contains("transition:"));
        
        // Verify not empty
        assert!(!css.is_empty());
        assert!(css.len() > 500);
    }
    
    #[test]
    fn smoke_test_dialog_has_proper_spacing() {
        let css = generate_css();
        
        // Verify spacing properties (updated to compact sizing)
        assert!(css.contains("padding: 20px")); // Updated from 24px
        assert!(css.contains("margin-bottom:"));
        assert!(css.contains("margin-top:"));
    }
    
    #[test]
    fn smoke_test_dialog_theme_specificity() {
        let css = generate_css();
        
        // Each theme should have background, color, and border rules
        assert!(css.matches(".marco-theme-light").count() >= 5);
        assert!(css.matches(".marco-theme-dark").count() >= 5);
    }
}
