//! A4 page boundary and margin rendering for EditorBuffer
use gtk4::cairo::Context;
use crate::logic::editor::core::EditorBuffer;
#[derive(Debug, Clone, Copy)]
pub struct EditorMarginUpdate {
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
    pub right: f64,
}

/// Convert centimeters to pixels (assuming 96 DPI)
pub fn cm_to_px(cm: f64) -> f64 {
    cm * 96.0 / 2.54
}

impl EditorBuffer {
    /// Update all margins at once (called from UI)
    pub fn update_margins(&mut self, update: EditorMarginUpdate) {
        self.top_margin_cm = update.top;
        self.bottom_margin_cm = update.bottom;
        self.left_margin_cm = update.left;
        self.right_margin_cm = update.right;
        self.request_redraw();
    }
    /// Clamp margin value to valid range for A4 (in cm)
    pub fn clamp_margin(cm: f64, min: f64, max: f64) -> f64 {
        cm.max(min).min(max)
    }
    /// Set top margin in centimeters
    pub fn set_top_margin_cm(&mut self, cm: f64) {
        let max = 29.7 - self.bottom_margin_cm;
        self.top_margin_cm = Self::clamp_margin(cm, 0.0, max.max(0.0));
    }
    pub fn lines_per_a4_page(&self) -> usize {
        if !self.a4_mode { return self.lines.len(); }
        let a4_height_cm = 29.7;
        let usable_height_cm = a4_height_cm - self.top_margin_cm - self.bottom_margin_cm;
        let usable_height_px = cm_to_px(usable_height_cm);
        (usable_height_px / self.line_height).floor() as usize
    }
    /// Move cursor down by one A4 page
    pub fn move_a4_page_down(&mut self) {
        if self.a4_mode {
            let lines = self.lines_per_a4_page();
            self.cursor_row = (self.cursor_row + lines).min(self.lines.len().saturating_sub(1));
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
        }
    }
    /// Move cursor up by one A4 page
    pub fn move_a4_page_up(&mut self) {
        if self.a4_mode {
            let lines = self.lines_per_a4_page();
            self.cursor_row = self.cursor_row.saturating_sub(lines);
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
        }
    }
}

/// Render A4 page boundary and margin guides
pub fn render_a4_boundary(buffer: &EditorBuffer, ctx: &Context, width: i32, height: i32) {
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
