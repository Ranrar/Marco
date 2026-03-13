/// Sets the height of the toolbar widget (Box or similar)
pub fn set_toolbar_height(toolbar_box: &gtk4::Box, height: i32) {
    toolbar_box.set_height_request(height);
}

/// Wire the gutter on/off toggle buttons (binary state for line numbers).
///
/// The toolbar's first two children are expected to be the gutter-on and gutter-off
/// buttons. Their visibility is toggled so only one is shown at a time, and clicking
/// one hides it and shows the other while persisting the setting.
pub fn wire_gutter_toggle(
    toolbar: &gtk4::Box,
    settings_manager: &std::sync::Arc<core::logic::swanson::SettingsManager>,
) {
    use gtk4::prelude::*;

    let gutter_on_button = toolbar
        .observe_children()
        .item(0)
        .and_then(|child| child.downcast::<gtk4::Button>().ok());
    let gutter_off_button = toolbar
        .observe_children()
        .item(1)
        .and_then(|child| child.downcast::<gtk4::Button>().ok());

    if let (Some(gutter_on), Some(gutter_off)) = (gutter_on_button, gutter_off_button) {
        let current_line_numbers = settings_manager
            .get_settings()
            .layout
            .as_ref()
            .and_then(|l| l.show_line_numbers)
            .unwrap_or(true);

        gutter_on.set_visible(current_line_numbers);
        gutter_off.set_visible(!current_line_numbers);
        gutter_on.set_sensitive(true);
        gutter_off.set_sensitive(true);

        gutter_on.connect_clicked({
            let settings_manager = settings_manager.clone();
            let gutter_on = gutter_on.clone();
            let gutter_off = gutter_off.clone();
            move |_| {
                use crate::components::editor::editor_manager::update_line_numbers_globally;
                use core::logic::swanson::LayoutSettings;

                let _ = update_line_numbers_globally(false);
                if let Err(e) = settings_manager.update_settings(|settings| {
                    if settings.layout.is_none() {
                        settings.layout = Some(LayoutSettings::default());
                    }
                    if let Some(ref mut layout) = settings.layout {
                        layout.show_line_numbers = Some(false);
                    }
                }) {
                    log::warn!("Failed to persist line numbers=off from toolbar: {}", e);
                }
                gutter_on.set_visible(false);
                gutter_off.set_visible(true);
            }
        });

        gutter_off.connect_clicked({
            let settings_manager = settings_manager.clone();
            let gutter_on = gutter_on.clone();
            let gutter_off = gutter_off.clone();
            move |_| {
                use crate::components::editor::editor_manager::update_line_numbers_globally;
                use core::logic::swanson::LayoutSettings;

                let _ = update_line_numbers_globally(true);
                if let Err(e) = settings_manager.update_settings(|settings| {
                    if settings.layout.is_none() {
                        settings.layout = Some(LayoutSettings::default());
                    }
                    if let Some(ref mut layout) = settings.layout {
                        layout.show_line_numbers = Some(true);
                    }
                }) {
                    log::warn!("Failed to persist line numbers=on from toolbar: {}", e);
                }
                gutter_on.set_visible(true);
                gutter_off.set_visible(false);
            }
        });
    }
}

/// Updates toolbar button tooltips with new translations (in-place, without rebuilding)
pub fn update_toolbar_translations(toolbar: &gtk4::Box, translations: &Translations) {
    use gtk4::prelude::*;

    fn find_button_by_css_class(root: &gtk4::Widget, css_class: &str) -> Option<Button> {
        if let Ok(button) = root.clone().downcast::<Button>() {
            if button.has_css_class(css_class) {
                return Some(button);
            }
        }

        let mut child = root.first_child();
        while let Some(widget) = child {
            if let Some(found) = find_button_by_css_class(&widget, css_class) {
                return Some(found);
            }
            child = widget.next_sibling();
        }

        None
    }

    fn set_tooltip(toolbar: &gtk4::Box, css_class: &str, tooltip: &str) {
        if let Some(button) =
            find_button_by_css_class(toolbar.upcast_ref::<gtk4::Widget>(), css_class)
        {
            button.set_tooltip_text(Some(tooltip));
        }
    }

    // Core buttons/dropdowns
    set_tooltip(toolbar, "toolbar-headings-btn", "Block Type");
    set_tooltip(toolbar, "toolbar-btn-bold", &translations.toolbar.bold);
    set_tooltip(toolbar, "toolbar-btn-italic", &translations.toolbar.italic);
    set_tooltip(
        toolbar,
        "toolbar-btn-strikethrough",
        &translations.toolbar.strikethrough,
    );
    set_tooltip(toolbar, "toolbar-btn-highlight", "Highlight");
    set_tooltip(toolbar, "toolbar-btn-text-inline", "Inline");
    set_tooltip(toolbar, "toolbar-btn-inline-items", "Insert");
    set_tooltip(toolbar, "toolbar-btn-lists", "Lists");
    set_tooltip(toolbar, "toolbar-btn-hr", "Horizontal Rule");
    set_tooltip(toolbar, "toolbar-btn-block-items", "Blocks");
    set_tooltip(toolbar, "toolbar-btn-container-items", "Modules");
    set_tooltip(toolbar, "toolbar-btn-mention", "Mentions");

    // Gutter toggles
    if let Some(button) = find_button_by_css_class(
        toolbar.upcast_ref::<gtk4::Widget>(),
        "toolbar-btn-gutter-on",
    ) {
        button.set_tooltip_text(Some(&format!(
            "{} (On)",
            translations.settings.layout.line_numbers_label
        )));
    }

    if let Some(button) = find_button_by_css_class(
        toolbar.upcast_ref::<gtk4::Widget>(),
        "toolbar-btn-gutter-off",
    ) {
        button.set_tooltip_text(Some(&format!(
            "{} (Off)",
            translations.settings.layout.line_numbers_label
        )));
    }

    // Block-type popover item tooltips
    if let Some(button) =
        find_button_by_css_class(toolbar.upcast_ref::<gtk4::Widget>(), "toolbar-headings-btn")
    {
        if let Some(popover) = button.first_child() {
            if let Ok(popover_widget) = popover.downcast::<gtk4::Popover>() {
                if let Some(popover_child) = popover_widget.child() {
                    if let Ok(popover_box) = popover_child.downcast::<gtk4::Box>() {
                        let block_types = [
                            "Paragraph",
                            "Quote",
                            "Heading 1",
                            "Heading 2",
                            "Heading 3",
                            "Heading 4",
                            "Heading 5",
                            "Heading 6",
                            "Heading ID",
                        ];
                        for (i, block_type) in block_types.iter().enumerate() {
                            if let Some(btn_widget) = popover_box.observe_children().item(i as u32)
                            {
                                if let Ok(btn) = btn_widget.downcast::<Button>() {
                                    btn.set_tooltip_text(Some(block_type));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

use crate::ui::toolbar::{
    composite_paths, toolbar_composite_button_svg, toolbar_icon_svg, ToolbarIcon,
};
use gio::prelude::*;
use gtk4::prelude::*;
use gtk4::{
    gdk, glib, Align, Box, Button, DropDown, EventControllerMotion, Label, Orientation, Picture,
    Separator,
};
use log::trace;
use rsvg::{CairoRenderer, Loader};

use crate::components::language::Translations;
use crate::ui::css::constants::{DARK_PALETTE, LIGHT_PALETTE};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

/// Toolbar button references for updating active states
// Note: This struct is not currently used, but may be useful for managing toolbar state (e.g., enabling/disabling buttons, updating active states, or connecting signals) in the future.
#[allow(dead_code)]
pub struct ToolbarButtons {
    pub headings_dropdown: DropDown,
    pub bold_button: Button,
    pub italic_button: Button,
    pub code_button: Button,
    pub strikethrough_button: Button,
}

const TOOLBAR_ICON_SIZE: f64 = 12.0;

fn composite_label(label: &str) -> &str {
    let _ = label;
    ""
}

// Admonition SVG icons (from core render)
#[allow(dead_code)]
const ADMONITION_NOTE_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" focusable="false" aria-hidden="true"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M3 12a9 9 0 1 0 18 0a9 9 0 0 0 -18 0" /><path d="M12 9h.01" /><path d="M11 12h1v4h1" /></svg>"#;
#[allow(dead_code)]
const ADMONITION_TIP_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" focusable="false" aria-hidden="true"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M15.02 19.52c-2.341 .736 -5 .606 -7.32 -.52l-4.7 1l1.3 -3.9c-2.324 -3.437 -1.426 -7.872 2.1 -10.374c3.526 -2.501 8.59 -2.296 11.845 .48c1.649 1.407 2.575 3.253 2.742 5.152" /><path d="M19 22v.01" /><path d="M19 19a2.003 2.003 0 0 0 .914 -3.782a1.98 1.98 0 0 0 -2.414 .483" /></svg>"#;
#[allow(dead_code)]
const ADMONITION_IMPORTANT_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" focusable="false" aria-hidden="true"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M8 9h8" /><path d="M8 13h6" /><path d="M15 18l-3 3l-3 -3h-3a3 3 0 0 1 -3 -3v-8a3 3 0 0 1 3 -3h12a3 3 0 0 1 3 3v5.5" /><path d="M19 16v3" /><path d="M19 22v.01" /></svg>"#;
#[allow(dead_code)]
const ADMONITION_WARNING_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" focusable="false" aria-hidden="true"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M10.363 3.591l-8.106 13.534a1.914 1.914 0 0 0 1.636 2.871h16.214a1.914 1.914 0 0 0 1.636 -2.87l-8.106 -13.536a1.914 1.914 0 0 0 -3.274 0" /><path d="M12 9h.01" /><path d="M11 12h1v4h1" /></svg>"#;
#[allow(dead_code)]
const ADMONITION_CAUTION_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" focusable="false" aria-hidden="true"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M19.875 6.27c.7 .398 1.13 1.143 1.125 1.948v7.284c0 .809 -.443 1.555 -1.158 1.948l-6.75 4.27a2.269 2.269 0 0 1 -2.184 0l-6.75 -4.27a2.225 2.225 0 0 1 -1.158 -1.948v-7.285c0 -.809 .443 -1.554 1.158 -1.947l6.75 -3.98a2.33 2.33 0 0 1 2.25 0l6.75 3.98h-.033" /><path d="M12 8v4" /><path d="M12 16h.01" /></svg>"#;
#[allow(dead_code)]
const ADMONITION_CUSTOM_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-scale"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M7 20l10 0" /><path d="M6 6l6 -1l6 1" /><path d="M12 3l0 17" /><path d="M9 12l-3 -6l-3 6a3 3 0 0 0 6 0" /><path d="M21 12l-3 -6l-3 6a3 3 0 0 0 6 0" /></svg>"#;

fn is_dark_theme(widget: &gtk4::Widget) -> bool {
    widget
        .root()
        .and_then(|r| r.downcast::<gtk4::Window>().ok())
        .map(|w| w.has_css_class("marco-theme-dark"))
        .unwrap_or(false)
}

fn toolbar_icon_color_for_flags(widget: &gtk4::Widget, flags: gtk4::StateFlags) -> &'static str {
    let dark = is_dark_theme(widget);
    if flags.contains(gtk4::StateFlags::ACTIVE) {
        if dark {
            DARK_PALETTE.control_icon_active
        } else {
            LIGHT_PALETTE.control_icon_active
        }
    } else if flags.contains(gtk4::StateFlags::PRELIGHT) {
        if dark {
            DARK_PALETTE.control_icon_hover
        } else {
            LIGHT_PALETTE.control_icon_hover
        }
    } else if dark {
        DARK_PALETTE.control_icon
    } else {
        LIGHT_PALETTE.control_icon
    }
}

fn fallback_texture() -> gdk::MemoryTexture {
    let bytes = glib::Bytes::from_owned(vec![0u8, 0u8, 0u8, 0u8]);
    gdk::MemoryTexture::new(1, 1, gdk::MemoryFormat::B8g8r8a8Premultiplied, &bytes, 4)
}

fn render_toolbar_svg_icon(icon: ToolbarIcon, color: &str, icon_size: f64) -> gdk::MemoryTexture {
    let svg = toolbar_icon_svg(icon).replace("currentColor", color);
    let bytes = glib::Bytes::from_owned(svg.into_bytes());
    let stream = gio::MemoryInputStream::from_bytes(&bytes);

    let handle =
        match Loader::new().read_stream(&stream, None::<&gio::File>, gio::Cancellable::NONE) {
            Ok(h) => h,
            Err(e) => {
                log::error!("load toolbar SVG handle: {}", e);
                return fallback_texture();
            }
        };

    let display_scale = gdk::Display::default()
        .and_then(|d| d.monitors().item(0))
        .and_then(|m| m.downcast::<gdk::Monitor>().ok())
        .map(|m| m.scale_factor() as f64)
        .unwrap_or(1.0);

    let render_scale = display_scale * 2.0;
    let render_size = (icon_size * render_scale) as i32;

    let mut surface =
        match cairo::ImageSurface::create(cairo::Format::ARgb32, render_size, render_size) {
            Ok(s) => s,
            Err(e) => {
                log::error!("create toolbar SVG image surface: {}", e);
                return fallback_texture();
            }
        };

    {
        let cr = match cairo::Context::new(&surface) {
            Ok(c) => c,
            Err(e) => {
                log::error!("create toolbar SVG cairo context: {}", e);
                return fallback_texture();
            }
        };

        cr.scale(render_scale, render_scale);

        let renderer = CairoRenderer::new(&handle);
        let viewport = cairo::Rectangle::new(0.0, 0.0, icon_size, icon_size);
        if let Err(e) = renderer.render_document(&cr, &viewport) {
            log::error!("render toolbar SVG: {}", e);
            return fallback_texture();
        }
    }

    let data = match surface.data() {
        Ok(d) => d.to_vec(),
        Err(e) => {
            log::error!("get toolbar SVG surface data: {}", e);
            return fallback_texture();
        }
    };

    let bytes = glib::Bytes::from_owned(data);
    gdk::MemoryTexture::new(
        render_size,
        render_size,
        gdk::MemoryFormat::B8g8r8a8Premultiplied,
        &bytes,
        (render_size * 4) as usize,
    )
}

/// Pick an inverted (contrasting) color for content drawn on top of `hex_color`.
///
/// Uses perceived-brightness (ITU-R BT.601) to decide:
///   bright background → dark content, dark background → light content.
fn inverted_color_for(hex_color: &str) -> &'static str {
    let r = u8::from_str_radix(hex_color.get(1..3).unwrap_or("80"), 16).unwrap_or(128);
    let g = u8::from_str_radix(hex_color.get(3..5).unwrap_or("80"), 16).unwrap_or(128);
    let b = u8::from_str_radix(hex_color.get(5..7).unwrap_or("80"), 16).unwrap_or(128);
    let brightness = 0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64;
    if brightness > 140.0 {
        "#1a1a1a" // dark content for light/medium backgrounds
    } else {
        "#f0f0f0" // light content for dark backgrounds
    }
}

/// Render a rectangular (non-square) SVG with two-color replacement.
///
/// Replaces `currentColor` → `fg_color` and `invertedColor` → `inv_color` before
/// rasterising into a `gdk::MemoryTexture` at `display_w × display_h` logical pixels
/// (super-sampled by 2× the display scale for crisp output).
fn render_toolbar_rect_svg(
    svg: &str,
    fg_color: &str,
    inv_color: &str,
    display_w: f64,
    display_h: f64,
) -> gdk::MemoryTexture {
    let svg_colored = svg
        .replace("currentColor", fg_color)
        .replace("invertedColor", inv_color);
    let bytes = glib::Bytes::from_owned(svg_colored.into_bytes());
    let stream = gio::MemoryInputStream::from_bytes(&bytes);

    let handle =
        match Loader::new().read_stream(&stream, None::<&gio::File>, gio::Cancellable::NONE) {
            Ok(h) => h,
            Err(e) => {
                log::error!("load composite SVG handle: {}", e);
                return fallback_texture();
            }
        };

    let display_scale = gdk::Display::default()
        .and_then(|d| d.monitors().item(0))
        .and_then(|m| m.downcast::<gdk::Monitor>().ok())
        .map(|m| m.scale_factor() as f64)
        .unwrap_or(1.0);

    let render_scale = display_scale * 2.0;
    let render_w = (display_w * render_scale) as i32;
    let render_h = (display_h * render_scale) as i32;

    let mut surface = match cairo::ImageSurface::create(cairo::Format::ARgb32, render_w, render_h) {
        Ok(s) => s,
        Err(e) => {
            log::error!("create composite SVG surface: {}", e);
            return fallback_texture();
        }
    };

    {
        let cr = match cairo::Context::new(&surface) {
            Ok(c) => c,
            Err(e) => {
                log::error!("create composite SVG cairo context: {}", e);
                return fallback_texture();
            }
        };

        cr.scale(render_scale, render_scale);

        let renderer = CairoRenderer::new(&handle);
        let viewport = cairo::Rectangle::new(0.0, 0.0, display_w, display_h);
        if let Err(e) = renderer.render_document(&cr, &viewport) {
            log::error!("render composite SVG: {}", e);
            return fallback_texture();
        }
    }

    let data = match surface.data() {
        Ok(d) => d.to_vec(),
        Err(e) => {
            log::error!("get composite SVG surface data: {}", e);
            return fallback_texture();
        }
    };

    let bytes = glib::Bytes::from_owned(data);
    gdk::MemoryTexture::new(
        render_w,
        render_h,
        gdk::MemoryFormat::B8g8r8a8Premultiplied,
        &bytes,
        (render_w * 4) as usize,
    )
}

fn create_toolbar_icon_button(
    icon: ToolbarIcon,
    tooltip: &str,
    css_class: &str,
    icon_size: f64,
) -> Button {
    let button = Button::new();

    let picture = Picture::new();
    let initial_flags = button.state_flags();
    let color = toolbar_icon_color_for_flags(button.upcast_ref(), initial_flags);
    let texture = render_toolbar_svg_icon(icon, color, icon_size);
    picture.set_paintable(Some(&texture));
    picture.set_size_request(icon_size as i32, icon_size as i32);
    picture.set_can_shrink(false);
    picture.set_halign(Align::Center);
    picture.set_valign(Align::Center);

    button.set_child(Some(&picture));
    button.set_tooltip_text(Some(tooltip));
    button.add_css_class(css_class);
    button.add_css_class("toolbar-icon-btn");
    button.set_has_frame(false);
    button.set_width_request((icon_size + 2.0) as i32);
    button.set_height_request((icon_size + 2.0) as i32);

    {
        let pic_update = picture.clone();
        let btn_update = button.clone();
        let update_icon = move || {
            let flags = btn_update.state_flags();
            let color = toolbar_icon_color_for_flags(btn_update.upcast_ref(), flags);
            let texture = render_toolbar_svg_icon(icon, color, icon_size);
            pic_update.set_paintable(Some(&texture));
        };

        // Recompute icon color when state changes (hover/active/normal).
        let update_for_state = update_icon.clone();
        button.connect_state_flags_changed(move |_, _| {
            update_for_state();
        });

        // Recompute after map so theme class from the root window is available.
        let update_for_map = update_icon.clone();
        button.connect_map(move |_| {
            update_for_map();
        });

        // Also sync after click activation in case state changes quickly.
        button.connect_clicked(move |_| {
            update_icon();
        });
    }

    button
}

/// Create a pill-shaped composite dropdown button rendered as a single SVG.
///
/// The button contains a rounded-rect background, left icon, text label,
/// and right ▼ chevron — all in one rasterised image. Colors update automatically
/// on state changes (hover/active) and on theme switch.
///
/// # Arguments
/// - `icon_paths` - inner `<path>` SVG data from a 24×24 source (see `composite_paths`)
/// - `label` - visible text inside the button
/// - `tooltip` - hover tooltip (may differ from `label`)
/// - `css_class` - unique CSS class for signal wiring / styling
fn create_toolbar_composite_dropdown_button(
    icon_paths: &str,
    label: &str,
    tooltip: &str,
    css_class: &str,
) -> Button {
    let button = Button::new();
    button.set_tooltip_text(Some(tooltip));
    button.add_css_class(css_class);
    button.add_css_class("toolbar-icon-btn");
    button.add_css_class("toolbar-dropdown-btn");
    button.add_css_class("toolbar-composite-btn");
    button.set_has_frame(false);
    render_composite_button_content(&button, icon_paths, label);

    {
        let btn = button.clone();
        let icon_paths = icon_paths.to_string();
        let label = label.to_string();
        let update = move || {
            render_composite_button_content(&btn, &icon_paths, &label);
        };

        let update_for_state = update.clone();
        button.connect_state_flags_changed(move |_, _| {
            update_for_state();
        });

        let update_for_map = update.clone();
        button.connect_map(move |_| {
            update_for_map();
        });

        button.connect_clicked(move |_| {
            update();
        });
    }

    button
}

fn render_composite_button_content(button: &Button, icon_paths: &str, label: &str) {
    let effective_label = composite_label(label);
    let composite = toolbar_composite_button_svg(icon_paths, effective_label);

    // ViewBox height is always 24; map to display using TOOLBAR_ICON_SIZE as height.
    let display_h = TOOLBAR_ICON_SIZE;
    let display_w = composite.viewbox_width * (display_h / 24.0);

    button.set_width_request(display_w as i32 + 1);
    button.set_height_request(display_h as i32 + 2);

    let picture = if let Some(child) = button.child() {
        child
            .downcast::<Picture>()
            .ok()
            .unwrap_or_else(Picture::new)
    } else {
        Picture::new()
    };

    let flags = button.state_flags();
    let fg = toolbar_icon_color_for_flags(button.upcast_ref(), flags);
    let inv = inverted_color_for(fg);
    let texture = render_toolbar_rect_svg(&composite.svg, fg, inv, display_w, display_h);
    picture.set_paintable(Some(&texture));
    picture.set_size_request(display_w as i32, display_h as i32);
    picture.set_can_shrink(false);
    picture.set_halign(Align::Center);
    picture.set_valign(Align::Center);

    if button.child().is_none() {
        button.set_child(Some(&picture));
    }
}

fn connect_hover_popover(button: &Button, popover: &gtk4::Popover, audit_label: &'static str) {
    let over_button = Rc::new(Cell::new(false));
    let over_popover = Rc::new(Cell::new(false));

    let schedule_close = {
        let over_button = over_button.clone();
        let over_popover = over_popover.clone();
        let popover = popover.clone();
        move || {
            let over_button = over_button.clone();
            let over_popover = over_popover.clone();
            let popover = popover.clone();
            glib::timeout_add_local_once(Duration::from_millis(120), move || {
                if !over_button.get() && !over_popover.get() {
                    popover.popdown();
                }
            });
        }
    };

    let button_motion = EventControllerMotion::new();
    {
        let over_button = over_button.clone();
        let button = button.clone();
        let popover = popover.clone();
        button_motion.connect_enter(move |_, _, _| {
            if crate::ui::popover_state::is_toolbar_interaction_blocked() {
                return;
            }

            over_button.set(true);

            // Avoid GTK warnings such as:
            // "Trying to snapshot GtkGizmo ... without a current allocation".
            // Popover opening is deferred until both anchor button and popover parent
            // have a real allocation.
            if !button.is_mapped()
                || button.allocated_width() <= 1
                || button.allocated_height() <= 1
            {
                let button_retry = button.clone();
                let popover_retry = popover.clone();
                glib::timeout_add_local_once(Duration::from_millis(16), move || {
                    if button_retry.is_mapped()
                        && button_retry.allocated_width() > 1
                        && button_retry.allocated_height() > 1
                    {
                        popover_retry.popup();
                    }
                });
                return;
            }

            popover.popup();
            trace!("audit: {} opened (hover)", audit_label);
        });
    }
    {
        let over_button = over_button.clone();
        let schedule_close = schedule_close.clone();
        button_motion.connect_leave(move |_| {
            over_button.set(false);
            schedule_close();
        });
    }
    button.add_controller(button_motion);

    let popover_motion = EventControllerMotion::new();
    {
        let over_popover = over_popover.clone();
        popover_motion.connect_enter(move |_, _, _| {
            over_popover.set(true);
        });
    }
    {
        let over_popover = over_popover.clone();
        let schedule_close = schedule_close.clone();
        popover_motion.connect_leave(move |_| {
            over_popover.set(false);
            schedule_close();
        });
    }
    popover.add_controller(popover_motion);
}

fn create_toolbar_popover_row_button(icon: ToolbarIcon, label: &str, css_class: &str) -> Button {
    create_toolbar_popover_row_button_with_helper(icon, label, label, css_class)
}

fn create_toolbar_popover_row_button_with_helper(
    icon: ToolbarIcon,
    label: &str,
    helper_text: &str,
    css_class: &str,
) -> Button {
    let button = Button::new();
    button.set_has_frame(false);
    button.set_tooltip_text(Some(helper_text));
    button.add_css_class(css_class);
    button.add_css_class("toolbar-icon-btn");

    let row = Box::new(Orientation::Horizontal, 6);
    row.set_halign(Align::Start);

    let icon_picture = Picture::new();
    let texture = render_toolbar_svg_icon(icon, "#9CA3AF", TOOLBAR_ICON_SIZE);
    icon_picture.set_paintable(Some(&texture));
    icon_picture.set_size_request(TOOLBAR_ICON_SIZE as i32, TOOLBAR_ICON_SIZE as i32);
    icon_picture.set_halign(Align::Start);
    row.append(&icon_picture);

    let text_label = Label::new(Some(label));
    text_label.set_xalign(0.0);
    row.append(&text_label);

    button.set_child(Some(&row));
    button
}

pub fn create_toolbar_structure(translations: &Translations) -> Box {
    // Create basic toolbar structure with spacing between buttons
    let toolbar = Box::new(Orientation::Horizontal, 0); // tighter spacing inside groups
    toolbar.add_css_class("toolbar");
    toolbar.set_margin_top(0);
    toolbar.set_margin_bottom(0);
    toolbar.set_margin_start(0);
    toolbar.set_margin_end(0);

    // Gutter toggle buttons (binary state: on/off) - first in toolbar
    let gutter_on_button = create_toolbar_icon_button(
        ToolbarIcon::GutterOn,
        &format!("{} (On)", translations.settings.layout.line_numbers_label),
        "toolbar-btn-gutter-on",
        TOOLBAR_ICON_SIZE,
    );
    toolbar.append(&gutter_on_button);

    let gutter_off_button = create_toolbar_icon_button(
        ToolbarIcon::GutterOff,
        &format!("{} (Off)", translations.settings.layout.line_numbers_label),
        "toolbar-btn-gutter-off",
        TOOLBAR_ICON_SIZE,
    );
    toolbar.append(&gutter_off_button);

    // Separator
    let sep0 = Separator::new(Orientation::Vertical);
    sep0.add_css_class("toolbar-separator");
    toolbar.append(&sep0);

    // Block-type dropdown (Paragraph, Quote, Heading 1-6) — composite SVG button
    let text_paragraph_poover_button = create_toolbar_composite_dropdown_button(
        composite_paths::PARAGRAPH,
        "Block",
        "Block Type",
        "toolbar-headings-btn",
    );

    let block_type_popover = gtk4::Popover::new();
    block_type_popover.set_parent(&text_paragraph_poover_button);
    block_type_popover.add_css_class("marco-toolbar-popover");
    crate::ui::popover_state::enforce_dismiss_behavior(&block_type_popover);
    let popover_box = Box::new(Orientation::Vertical, 4);
    let block_type_items = [
        (
            ToolbarIcon::Paragraph,
            "Paragraph",
            "toolbar-blocktype-paragraph",
        ),
        (ToolbarIcon::Blockquote, "Quote", "toolbar-blocktype-quote"),
        (ToolbarIcon::H1, "Heading 1", "toolbar-blocktype-h1"),
        (ToolbarIcon::H2, "Heading 2", "toolbar-blocktype-h2"),
        (ToolbarIcon::H3, "Heading 3", "toolbar-blocktype-h3"),
        (ToolbarIcon::H4, "Heading 4", "toolbar-blocktype-h4"),
        (ToolbarIcon::H5, "Heading 5", "toolbar-blocktype-h5"),
        (ToolbarIcon::H6, "Heading 6", "toolbar-blocktype-h6"),
        (
            ToolbarIcon::HeadingId,
            "Heading ID",
            "toolbar-btn-heading-id",
        ),
    ];
    for (icon, label, class_name) in block_type_items {
        let btn = create_toolbar_popover_row_button(icon, label, "toolbar-headings-popover-btn");
        btn.add_css_class(class_name);
        popover_box.append(&btn);
    }
    block_type_popover.set_child(Some(&popover_box));
    block_type_popover.set_position(gtk4::PositionType::Bottom);
    connect_hover_popover(
        &text_paragraph_poover_button,
        &block_type_popover,
        "block type dropdown",
    );
    let popover_ref = block_type_popover.clone();
    text_paragraph_poover_button.connect_clicked(move |_| {
        if crate::ui::popover_state::is_toolbar_interaction_blocked() {
            return;
        }
        popover_ref.popup();
        trace!("audit: block type dropdown opened");
    });

    // Text formatting buttons
    let bold_button = create_toolbar_icon_button(
        ToolbarIcon::Bold,
        &translations.toolbar.bold,
        "toolbar-btn-bold",
        TOOLBAR_ICON_SIZE,
    );

    let italic_button = create_toolbar_icon_button(
        ToolbarIcon::Italic,
        &translations.toolbar.italic,
        "toolbar-btn-italic",
        TOOLBAR_ICON_SIZE,
    );

    let strikethrough_button = create_toolbar_icon_button(
        ToolbarIcon::Strikethrough,
        &translations.toolbar.strikethrough,
        "toolbar-btn-strikethrough",
        TOOLBAR_ICON_SIZE,
    );

    let highlight_button = create_toolbar_icon_button(
        ToolbarIcon::Highlight,
        "Highlight",
        "toolbar-btn-highlight",
        TOOLBAR_ICON_SIZE,
    );

    let text_inline_poover_button = create_toolbar_composite_dropdown_button(
        composite_paths::TEXT_INLINE,
        "Inline",
        "Inline",
        "toolbar-btn-text-inline",
    );
    let text_inline_popover = gtk4::Popover::new();
    text_inline_popover.set_parent(&text_inline_poover_button);
    text_inline_popover.add_css_class("marco-toolbar-popover");
    crate::ui::popover_state::enforce_dismiss_behavior(&text_inline_popover);
    let text_inline_box = Box::new(Orientation::Vertical, 4);
    let text_inline_items = [
        (ToolbarIcon::Code, "Code", "Inline code", "toolbar-btn-code"),
        (
            ToolbarIcon::Superscript,
            "Superscript",
            "Superscript text",
            "toolbar-btn-superscript",
        ),
        (
            ToolbarIcon::Subscript,
            "Subscript",
            "Subscript text",
            "toolbar-btn-subscript",
        ),
        (
            ToolbarIcon::Math,
            "Math",
            "Inline math",
            "toolbar-btn-inline-math",
        ),
    ];
    for (icon, label, helper_text, class_name) in text_inline_items {
        let btn = create_toolbar_popover_row_button_with_helper(
            icon,
            label,
            helper_text,
            "toolbar-functions-popover-btn",
        );
        btn.add_css_class(class_name);
        text_inline_box.append(&btn);
    }
    text_inline_popover.set_child(Some(&text_inline_box));
    text_inline_popover.set_position(gtk4::PositionType::Bottom);
    connect_hover_popover(
        &text_inline_poover_button,
        &text_inline_popover,
        "text inline dropdown",
    );
    let text_inline_popover_ref = text_inline_popover.clone();
    text_inline_poover_button.connect_clicked(move |_| {
        if crate::ui::popover_state::is_toolbar_interaction_blocked() {
            return;
        }
        text_inline_popover_ref.popup();
        trace!("audit: text inline dropdown opened");
    });

    let list_button = create_toolbar_icon_button(
        ToolbarIcon::BulletList,
        "Lists",
        "toolbar-btn-lists",
        TOOLBAR_ICON_SIZE,
    );

    let inline_items_poover_button = create_toolbar_composite_dropdown_button(
        composite_paths::INLINE_ITEMS,
        "Insert",
        "Insert",
        "toolbar-btn-inline-items",
    );
    let inline_items_popover = gtk4::Popover::new();
    inline_items_popover.set_parent(&inline_items_poover_button);
    inline_items_popover.add_css_class("marco-toolbar-popover");
    crate::ui::popover_state::enforce_dismiss_behavior(&inline_items_popover);
    let inline_items_box = Box::new(Orientation::Vertical, 4);
    let inline_items = [
        (ToolbarIcon::Link, "Link", "Inline link", "toolbar-btn-link"),
        (
            ToolbarIcon::LinkReference,
            "Link Reference",
            "Link reference",
            "toolbar-btn-link-reference",
        ),
        (
            ToolbarIcon::Image,
            "Image",
            "Inline image",
            "toolbar-btn-image",
        ),
        (
            ToolbarIcon::InlineFootnote,
            "Footnote",
            "Inline footnote",
            "toolbar-btn-inline-footnote",
        ),
        (
            ToolbarIcon::Emoji,
            "Emoji",
            "Inline emoji",
            "toolbar-btn-emoji",
        ),
        (
            ToolbarIcon::Checkbox,
            "Checkbox",
            "Inline checkbox",
            "toolbar-btn-inline-checkbox",
        ),
    ];
    for (icon, label, helper_text, class_name) in inline_items {
        let btn = create_toolbar_popover_row_button_with_helper(
            icon,
            label,
            helper_text,
            "toolbar-functions-popover-btn",
        );
        btn.add_css_class(class_name);
        inline_items_box.append(&btn);
    }
    inline_items_popover.set_child(Some(&inline_items_box));
    inline_items_popover.set_position(gtk4::PositionType::Bottom);
    connect_hover_popover(
        &inline_items_poover_button,
        &inline_items_popover,
        "inline items dropdown",
    );
    let inline_items_popover_ref = inline_items_popover.clone();
    inline_items_poover_button.connect_clicked(move |_| {
        if crate::ui::popover_state::is_toolbar_interaction_blocked() {
            return;
        }
        inline_items_popover_ref.popup();
        trace!("audit: inline items dropdown opened");
    });

    let block_items_poover_button = create_toolbar_composite_dropdown_button(
        composite_paths::BLOCK_ITEMS,
        "Blocks",
        "Blocks",
        "toolbar-btn-block-items",
    );
    let hr_button = create_toolbar_icon_button(
        ToolbarIcon::ThematicBreak,
        "Horizontal Rule",
        "toolbar-btn-hr",
        TOOLBAR_ICON_SIZE,
    );
    let block_items_popover = gtk4::Popover::new();
    block_items_popover.set_parent(&block_items_poover_button);
    block_items_popover.add_css_class("marco-toolbar-popover");
    crate::ui::popover_state::enforce_dismiss_behavior(&block_items_popover);
    let block_items_box = Box::new(Orientation::Vertical, 4);
    let block_items = [
        (
            ToolbarIcon::CodeBlock,
            "Code",
            "Code block",
            "toolbar-btn-fenced-code-block",
        ),
        (ToolbarIcon::Math, "Math", "Math block", "toolbar-btn-math"),
        (
            ToolbarIcon::Footnote,
            "Footnote",
            "Block footnote",
            "toolbar-btn-footnote",
        ),
    ];
    for (icon, label, helper_text, class_name) in block_items {
        let btn = create_toolbar_popover_row_button_with_helper(
            icon,
            label,
            helper_text,
            "toolbar-functions-popover-btn",
        );
        btn.add_css_class(class_name);
        block_items_box.append(&btn);
    }
    block_items_popover.set_child(Some(&block_items_box));
    block_items_popover.set_position(gtk4::PositionType::Bottom);
    connect_hover_popover(
        &block_items_poover_button,
        &block_items_popover,
        "block items dropdown",
    );
    let block_items_popover_ref = block_items_popover.clone();
    block_items_poover_button.connect_clicked(move |_| {
        if crate::ui::popover_state::is_toolbar_interaction_blocked() {
            return;
        }
        block_items_popover_ref.popup();
        trace!("audit: block items dropdown opened");
    });

    let container_items_poover_button = create_toolbar_composite_dropdown_button(
        composite_paths::TABLE,
        "Modules",
        "Modules",
        "toolbar-btn-container-items",
    );
    let container_items_popover = gtk4::Popover::new();
    container_items_popover.set_parent(&container_items_poover_button);
    container_items_popover.add_css_class("marco-toolbar-popover");
    crate::ui::popover_state::enforce_dismiss_behavior(&container_items_popover);
    let container_items_box = Box::new(Orientation::Vertical, 4);
    let container_items = [
        (ToolbarIcon::Table, "Table", "Table", "toolbar-btn-table"),
        (
            ToolbarIcon::TabBlock,
            "Tab block",
            "Tab block",
            "toolbar-btn-tab-block",
        ),
        (
            ToolbarIcon::Slideshow,
            "Slider deck",
            "Slider deck",
            "toolbar-btn-slideshow",
        ),
        (
            ToolbarIcon::Mermaid,
            "Mermaid",
            "Mermaid",
            "toolbar-btn-mermaid",
        ),
        (
            ToolbarIcon::Admonition,
            "Admonition",
            "Admonition",
            "toolbar-btn-admonition",
        ),
    ];
    for (icon, label, helper_text, class_name) in container_items {
        let btn = create_toolbar_popover_row_button_with_helper(
            icon,
            label,
            helper_text,
            "toolbar-functions-popover-btn",
        );
        btn.add_css_class(class_name);
        container_items_box.append(&btn);
    }
    container_items_popover.set_child(Some(&container_items_box));
    container_items_popover.set_position(gtk4::PositionType::Bottom);
    connect_hover_popover(
        &container_items_poover_button,
        &container_items_popover,
        "modules items dropdown",
    );
    let container_items_popover_ref = container_items_popover.clone();
    container_items_poover_button.connect_clicked(move |_| {
        if crate::ui::popover_state::is_toolbar_interaction_blocked() {
            return;
        }
        container_items_popover_ref.popup();
        trace!("audit: modules items dropdown opened");
    });

    let mention_button = create_toolbar_icon_button(
        ToolbarIcon::Mention,
        "Mentions",
        "toolbar-btn-mention",
        TOOLBAR_ICON_SIZE,
    );

    // Group 1: Inline
    let sep1 = Separator::new(Orientation::Vertical);
    sep1.add_css_class("toolbar-separator");
    toolbar.append(&bold_button); // Bold
    toolbar.append(&italic_button); // italic
    toolbar.append(&strikethrough_button); // strikethrough
    toolbar.append(&highlight_button); // highlight
    toolbar.append(&text_inline_poover_button); // Popover: Code (Inline), Superscript, Subscript, Inline Footnote, Inline References, Math (Inline)
    toolbar.append(&inline_items_poover_button); // Popover: Inline Link, Link Reference, Image, Inline Footnote, Emoji, Inline Checkbox
    toolbar.append(&mention_button); // Mention
    toolbar.append(&sep1);

    // Group 2: Block
    let sep2 = Separator::new(Orientation::Vertical);
    sep2.add_css_class("toolbar-separator");
    toolbar.append(&text_paragraph_poover_button); // Popover: Paragraph, Quote, Heading 1-6, Heading ID
    toolbar.append(&list_button); // Button: Lists
    toolbar.append(&hr_button); // Horizontal rule
    toolbar.append(&sep2);

    // Group 3: Modules / composite
    toolbar.append(&block_items_poover_button); // Popover: Code Block, Math (Block), Footnote
    toolbar.append(&container_items_poover_button); // Popover: Table, Tab block, Slider deck, Mermaid, Admonition

    toolbar
}
