//! Preview zoom overlay bar.
//!
//! Builds a small floating control bar (`zoom-bar`) added as an overlay on the
//! existing [`gtk4::Overlay`] that wraps the editor/preview paned split.
//!
//! # Layout
//!
//! ```text
//! [−]  100%  [⤢]  [+]
//! ```
//!
//! Positioned at the **bottom-right** corner via `halign=End, valign=End`.
//!
//! Button clicks call the global zoom functions from `editor_manager`, which
//! fire a registered callback to keep the label in sync.  Keyboard shortcuts
//! that call `set_preview_zoom` also trigger the same callback.

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, EventControllerMotion, Label, Orientation, Separator};
use std::rc::Rc;
use std::sync::Arc;

/// Build the zoom bar, add it to `overlay_widget`, and return a handle.
///
/// `paned` is the editor/preview split inside the overlay; it is used to detect
/// whether the pointer is over the **preview** (right) half so the bar is only
/// shown on hover there.
///
/// `settings_manager` — if provided, button clicks persist the new zoom level.
pub fn create_zoom_bar(
    overlay_widget: &gtk4::Overlay,
    paned: &gtk4::Paned,
    settings_manager: Option<Arc<marco_shared::logic::swanson::SettingsManager>>,
) -> ZoomBarHandle {
    // ── Container ──────────────────────────────────────────────────────────
    let bar = GtkBox::new(Orientation::Horizontal, 0);
    bar.add_css_class("zoom-bar");
    // Position at bottom-right of the overlay
    bar.set_halign(Align::End);
    bar.set_valign(Align::End);
    bar.set_hexpand(false);
    bar.set_vexpand(false);

    // ── Zoom-out button (−) ────────────────────────────────────────────────
    let btn_out = Button::with_label("−");
    btn_out.add_css_class("zoom-bar-btn");
    btn_out.set_tooltip_text(Some("Zoom out (Ctrl+−)"));

    // ── Separator ─────────────────────────────────────────────────────────
    let sep1 = Separator::new(Orientation::Vertical);
    sep1.add_css_class("zoom-bar-separator");

    // ── Percentage label ──────────────────────────────────────────────────
    let zoom_label = Label::new(Some("100%"));
    zoom_label.add_css_class("zoom-bar-label");
    zoom_label.set_halign(Align::Center);
    zoom_label.set_xalign(0.5);

    // ── Separator ─────────────────────────────────────────────────────────
    let sep2 = Separator::new(Orientation::Vertical);
    sep2.add_css_class("zoom-bar-separator");

    // ── Reset button (⤢ expand/reset icon) ────────────────────────────────
    let btn_reset = Button::with_label("⤢");
    btn_reset.add_css_class("zoom-bar-btn");
    btn_reset.set_tooltip_text(Some("Reset zoom (Ctrl+0)"));

    // ── Separator ─────────────────────────────────────────────────────────
    let sep3 = Separator::new(Orientation::Vertical);
    sep3.add_css_class("zoom-bar-separator");

    // ── Zoom-in button (+) ─────────────────────────────────────────────────
    let btn_in = Button::with_label("+");
    btn_in.add_css_class("zoom-bar-btn");
    btn_in.set_tooltip_text(Some("Zoom in (Ctrl++)"));

    // ── Assemble ──────────────────────────────────────────────────────────
    bar.append(&btn_out);
    bar.append(&sep1);
    bar.append(&zoom_label);
    bar.append(&sep2);
    bar.append(&btn_reset);
    bar.append(&sep3);
    bar.append(&btn_in);

    // ── Wire up buttons ───────────────────────────────────────────────────
    // Each button: adjust zoom, then optionally persist via settings manager.
    use crate::components::editor::editor_manager as em;

    let sm_out = settings_manager.clone();
    btn_out.connect_clicked(move |_| {
        let new_zoom = em::get_preview_zoom() - em::ZOOM_STEP;
        em::set_preview_zoom(new_zoom); // fires zoom-changed callback → label update
        persist_zoom(em::get_preview_zoom(), sm_out.as_deref());
    });

    let sm_reset = settings_manager.clone();
    btn_reset.connect_clicked(move |_| {
        em::set_preview_zoom(em::ZOOM_DEFAULT);
        persist_zoom(em::ZOOM_DEFAULT, sm_reset.as_deref());
    });

    let sm_in = settings_manager.clone();
    btn_in.connect_clicked(move |_| {
        let new_zoom = em::get_preview_zoom() + em::ZOOM_STEP;
        em::set_preview_zoom(new_zoom);
        persist_zoom(em::get_preview_zoom(), sm_in.as_deref());
    });

    // ── Sync label via zoom-changed callback ──────────────────────────────
    // This keeps the percentage label up-to-date when keyboard shortcuts fire
    // set_preview_zoom, because the callback is invoked from there too.
    let label_for_cb = zoom_label.clone();
    em::set_zoom_changed_callback(Some(Rc::new(move |zoom| {
        label_for_cb.set_text(&zoom_percent_text(zoom));
    })));

    // Initialise the label with the current zoom (may have been set from saved
    // settings before the overlay was created).
    zoom_label.set_text(&zoom_percent_text(em::get_preview_zoom()));

    // ── Register overlay ──────────────────────────────────────────────────
    // Start hidden; revealed only when the pointer is over the preview pane.
    bar.set_visible(false);
    overlay_widget.add_overlay(&bar);

    // ── Hover-to-reveal logic ─────────────────────────────────────────────
    // Attach a motion controller to the overlay (which spans both editor and
    // preview).  The bar is only shown when the cursor is to the right of the
    // paned split position, i.e. in the live-preview half.
    let bar_show = bar.clone();
    let paned_ref = paned.clone();
    let motion = EventControllerMotion::new();
    motion.connect_motion(move |_, x, _| {
        let in_preview = x > paned_ref.position() as f64;
        bar_show.set_visible(in_preview);
    });

    let bar_hide = bar.clone();
    motion.connect_leave(move |_| {
        bar_hide.set_visible(false);
    });

    overlay_widget.add_controller(motion);

    ZoomBarHandle { bar }
}

/// Format a float zoom level as a percentage string, e.g. `1.0` → `"100%"`.
fn zoom_percent_text(zoom: f64) -> String {
    format!("{}%", (zoom * 100.0).round() as u32)
}

/// Persist the zoom level to settings if a manager is available.
fn persist_zoom(zoom: f64, sm: Option<&marco_shared::logic::swanson::SettingsManager>) {
    if let Some(manager) = sm {
        if let Err(e) = manager.update_settings(|s| {
            s.layout
                .get_or_insert_with(marco_shared::logic::swanson::LayoutSettings::default)
                .preview_zoom = Some(zoom);
        }) {
            log::debug!("zoom bar: failed to persist zoom: {}", e);
        }
    }
}

/// Handle returned by [`create_zoom_bar`].
pub struct ZoomBarHandle {
    bar: GtkBox,
}

impl ZoomBarHandle {
    /// Show or hide the zoom bar (e.g. hide in editor-only mode).
    pub fn set_visible(&self, visible: bool) {
        self.bar.set_visible(visible);
    }
}
