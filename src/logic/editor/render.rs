//! Modular rendering logic for EditorBuffer
use gtk4::pango;

use gtk4::cairo::Context;
use crate::logic::editor::core::EditorBuffer;
/// Convert centimeters to pixels (assuming 96 DPI)
fn cm_to_px(cm: f64) -> f64 {
    cm * 96.0 / 2.54
}

/// Render A4 page boundary and margin guides
fn render_a4_boundary(buffer: &EditorBuffer, ctx: &Context, width: i32, height: i32) {
    if !buffer.a4_mode { return; }
    // A4 size in cm
    let a4_width_cm = 21.0;
    let a4_height_cm = 29.7;
    // Convert to px
    let a4_width_px = cm_to_px(a4_width_cm);
    let a4_height_px = cm_to_px(a4_height_cm);
    // Margins in px
    let left_px = cm_to_px(buffer.left_margin_cm);
    let right_px = cm_to_px(buffer.right_margin_cm);
    let top_px = cm_to_px(buffer.top_margin_cm);
    let bottom_px = cm_to_px(buffer.bottom_margin_cm);
    // Center A4 page in viewport
    let x = ((width as f64) - a4_width_px) / 2.0;
    let y = ((height as f64) - a4_height_px) / 2.0;
    // Draw A4 page boundary
    ctx.set_source_rgba(0.8, 0.8, 0.8, 1.0); // light gray
    ctx.rectangle(x, y, a4_width_px, a4_height_px);
    ctx.set_line_width(2.0);
    ctx.stroke().unwrap_or(());
    // Draw margin guides
    ctx.set_source_rgba(0.2, 0.6, 0.9, 0.5); // blue, semi-transparent
    // Top margin
    ctx.move_to(x, y + top_px);
    ctx.line_to(x + a4_width_px, y + top_px);
    ctx.stroke().unwrap_or(());
    // Bottom margin
    ctx.move_to(x, y + a4_height_px - bottom_px);
    ctx.line_to(x + a4_width_px, y + a4_height_px - bottom_px);
    ctx.stroke().unwrap_or(());
    // Left margin
    ctx.move_to(x + left_px, y);
    ctx.line_to(x + left_px, y + a4_height_px);
    ctx.stroke().unwrap_or(());
    // Right margin
    ctx.move_to(x + a4_width_px - right_px, y);
    ctx.line_to(x + a4_width_px - right_px, y + a4_height_px);
    ctx.stroke().unwrap_or(());
}

const GUTTER_WIDTH: i32 = 40;

/// Helper: parse color string to RGBA
pub fn parse_color(color: &str) -> (f64, f64, f64, f64) {
    if let Some(stripped) = color.strip_prefix('#') {
        match stripped.len() {
            6 => {
                let r = u8::from_str_radix(&stripped[0..2], 16).unwrap_or(0) as f64 / 255.0;
                let g = u8::from_str_radix(&stripped[2..4], 16).unwrap_or(0) as f64 / 255.0;
                let b = u8::from_str_radix(&stripped[4..6], 16).unwrap_or(0) as f64 / 255.0;
                (r, g, b, 1.0)
            }
            8 => {
                let r = u8::from_str_radix(&stripped[0..2], 16).unwrap_or(0) as f64 / 255.0;
                let g = u8::from_str_radix(&stripped[2..4], 16).unwrap_or(0) as f64 / 255.0;
                let b = u8::from_str_radix(&stripped[4..6], 16).unwrap_or(0) as f64 / 255.0;
                let a = u8::from_str_radix(&stripped[6..8], 16).unwrap_or(255) as f64 / 255.0;
                (r, g, b, a)
            }
            _ => (0.0, 0.0, 0.0, 1.0)
        }
    } else if color.starts_with("rgba") {
        let nums: Vec<f64> = color[5..color.len()-1].split(',').filter_map(|s| s.trim().parse().ok()).collect();
        if nums.len() == 4 {
            (nums[0]/255.0, nums[1]/255.0, nums[2]/255.0, nums[3])
        } else {
            (0.0, 0.0, 0.0, 1.0)
        }
    } else {
        (0.0, 0.0, 0.0, 1.0)
    }
}

/// Render the editor buffer to the given Cairo context
pub fn render_editor(buffer: &EditorBuffer, ctx: &Context, width: i32, height: i32) {
    // Background
    let (r, g, b, a) = parse_color(&buffer.bg_color);
    ctx.set_source_rgba(r, g, b, a);
    ctx.rectangle(0.0, 0.0, width as f64, height as f64);
    ctx.fill().unwrap_or(());

    // Draw A4 boundary and margin guides if enabled
    render_a4_boundary(buffer, ctx, width, height);

    // Gutter
    let (r, g, b, a) = parse_color(&buffer.gutter_color);
    ctx.set_source_rgba(r, g, b, a);
    ctx.rectangle(0.0, 0.0, GUTTER_WIDTH as f64, height as f64);
    ctx.fill().unwrap_or(());

    // Lines
    let line_height = buffer.line_height;
    let char_spacing = buffer.character_spacing;
    let font = &buffer.font;
    let font_size = buffer.font_size;
    let pango_ctx = pango::Context::new();
    for (i, line) in buffer.lines.iter().enumerate() {
        let y = i as f64 * line_height;
        // Line number
        let line_num_color = if buffer.highlight_line && i == buffer.cursor_row {
            &buffer.selected_line_number_color
        } else {
            &buffer.line_number_color
        };
        let (r, g, b, a) = parse_color(line_num_color);
        ctx.set_source_rgba(r, g, b, a);
        let layout = pango::Layout::new(&pango_ctx);
        layout.set_text(&format!("{}", i + 1));
        let font_desc = pango::FontDescription::from_string(&format!("{} {}", font, font_size));
        layout.set_font_description(Some(&font_desc));
        layout.set_spacing(char_spacing as i32);
        ctx.move_to(GUTTER_WIDTH as f64, y);
        // No direct show_layout; fallback to layout drawing if available

        // Active/inactive line background
        if buffer.show_active_line_bg && i == buffer.cursor_row {
            let (r, g, b, a) = parse_color(&buffer.active_line_bg_color);
            ctx.set_source_rgba(r, g, b, a);
            ctx.rectangle(GUTTER_WIDTH as f64, y, width as f64 - GUTTER_WIDTH as f64, line_height);
            ctx.fill().unwrap_or(());
        } else if buffer.show_inactive_line_bg {
            let (r, g, b, a) = parse_color(&buffer.inactive_line_bg_color);
            ctx.set_source_rgba(r, g, b, a);
            ctx.rectangle(GUTTER_WIDTH as f64, y, width as f64 - GUTTER_WIDTH as f64, line_height);
            ctx.fill().unwrap_or(());
        }

        // Text
        let (r, g, b, a) = parse_color(&buffer.fg_color);
        ctx.set_source_rgba(r, g, b, a);
        let layout = pango::Layout::new(&pango_ctx);
        layout.set_text(line);
        let font_desc = pango::FontDescription::from_string(&format!("{} {}", font, font_size));
        layout.set_font_description(Some(&font_desc));
        layout.set_spacing(char_spacing as i32);
        // No direct show_layout; fallback to layout drawing if available

        // Diagnostics
        for (row, _msg, kind) in &buffer.diagnostics {
            if *row == i {
                let color = if kind == "error" { &buffer.error_color } else { &buffer.warning_color };
                let (r, g, b, a) = parse_color(color);
                ctx.set_source_rgba(r, g, b, a);
                ctx.rectangle(GUTTER_WIDTH as f64, y, width as f64 - GUTTER_WIDTH as f64, line_height);
                ctx.fill().unwrap_or(());
            }
        }

        // Debug info
        if buffer.debug_mode {
            ctx.set_source_rgba(1.0, 0.0, 0.0, 0.5); // semi-transparent red
            ctx.rectangle(GUTTER_WIDTH as f64, y, 10.0, line_height);
            ctx.fill().unwrap_or(());
        }

        ctx.translate(-(GUTTER_WIDTH as f64), 0.0);
    }

    // Cursor
    let (r, g, b, a) = parse_color(&buffer.cursor_color);
    ctx.set_source_rgba(r, g, b, a);
    let cursor_x = GUTTER_WIDTH as f64 + buffer.cursor_col as f64 * font_size;
    let cursor_y = buffer.cursor_row as f64 * line_height;
    ctx.rectangle(cursor_x, cursor_y, 2.0, line_height);
    ctx.fill().unwrap_or(());
}
