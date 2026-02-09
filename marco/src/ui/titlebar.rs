//! Custom Titlebar Creation
//!
//! Provides reusable functions for creating Marco-styled titlebars with
//! close buttons and proper theme support.

use crate::ui::css::constants::{DARK_PALETTE, LIGHT_PALETTE};
use core::logic::loaders::icon_loader::{window_icon_svg, WindowIcon};
use gtk4::prelude::*;
use gtk4::{gio, glib, Align, Button, HeaderBar, Label, Window};
use rsvg::{CairoRenderer, Loader};

/// Render an SVG icon to a GdkMemoryTexture
fn render_svg_icon(icon: WindowIcon, color: &str, icon_size: f64) -> gtk4::gdk::MemoryTexture {
    let svg = window_icon_svg(icon).replace("currentColor", color);
    let bytes = glib::Bytes::from_owned(svg.into_bytes());
    let stream = gio::MemoryInputStream::from_bytes(&bytes);

    let handle =
        match Loader::new().read_stream(&stream, None::<&gio::File>, gio::Cancellable::NONE) {
            Ok(h) => h,
            Err(e) => {
                log::error!("load SVG handle: {}", e);
                // Fallback tiny transparent texture
                let bytes = glib::Bytes::from_owned(vec![0u8, 0u8, 0u8, 0u8]);
                return gtk4::gdk::MemoryTexture::new(
                    1,
                    1,
                    gtk4::gdk::MemoryFormat::B8g8r8a8Premultiplied,
                    &bytes,
                    4,
                );
            }
        };

    let display_scale = gtk4::gdk::Display::default()
        .and_then(|d| d.monitors().item(0))
        .and_then(|m| m.downcast::<gtk4::gdk::Monitor>().ok())
        .map(|m| m.scale_factor() as f64)
        .unwrap_or(1.0);

    let render_scale = display_scale * 2.0;
    let render_size = (icon_size * render_scale) as i32;

    let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, render_size, render_size)
        .expect("create surface");
    {
        let cr = cairo::Context::new(&surface).expect("create context");
        cr.scale(render_scale, render_scale);

        let renderer = CairoRenderer::new(&handle);
        let viewport = cairo::Rectangle::new(0.0, 0.0, icon_size, icon_size);
        renderer
            .render_document(&cr, &viewport)
            .expect("render SVG");
    }

    let data = surface.data().expect("get surface data").to_vec();
    let bytes = glib::Bytes::from_owned(data);
    gtk4::gdk::MemoryTexture::new(
        render_size,
        render_size,
        gtk4::gdk::MemoryFormat::B8g8r8a8Premultiplied,
        &bytes,
        (render_size * 4) as usize,
    )
}

/// Create an SVG icon button with hover and click interactions
fn create_svg_icon_button_with_picture(
    window: &Window,
    icon: WindowIcon,
    tooltip: &str,
    color: &str,
    icon_size: f64,
) -> (Button, gtk4::Picture) {
    let pic = gtk4::Picture::new();
    let texture = render_svg_icon(icon, color, icon_size);
    pic.set_paintable(Some(&texture));
    pic.set_size_request(icon_size as i32, icon_size as i32);
    pic.set_can_shrink(false);
    pic.set_halign(Align::Center);
    pic.set_valign(Align::Center);

    let btn = Button::new();
    btn.set_child(Some(&pic));
    btn.set_tooltip_text(Some(tooltip));
    btn.set_valign(Align::Center);
    btn.set_margin_start(1);
    btn.set_margin_end(1);
    btn.set_focusable(false);
    btn.set_can_focus(false);
    btn.set_has_frame(false);
    btn.add_css_class("topright-btn");
    btn.add_css_class("window-control-btn");
    btn.set_width_request((icon_size + 6.0) as i32);
    btn.set_height_request((icon_size + 6.0) as i32);

    // Add hover and click interactions
    {
        let pic_hover = pic.clone();
        let normal_color = color.to_string();
        let is_dark = window.has_css_class("marco-theme-dark");
        let hover_color = if is_dark {
            DARK_PALETTE.control_icon_hover.to_string()
        } else {
            LIGHT_PALETTE.control_icon_hover.to_string()
        };
        let active_color = if is_dark {
            DARK_PALETTE.control_icon_active.to_string()
        } else {
            LIGHT_PALETTE.control_icon_active.to_string()
        };

        let motion_controller = gtk4::EventControllerMotion::new();
        let icon_for_enter = icon;
        let hover_color_enter = hover_color.clone();
        motion_controller.connect_enter(move |_ctrl, _x, _y| {
            let texture = render_svg_icon(icon_for_enter, &hover_color_enter, icon_size);
            pic_hover.set_paintable(Some(&texture));
        });

        let pic_leave = pic.clone();
        let icon_for_leave = icon;
        let normal_color_leave = normal_color.clone();
        motion_controller.connect_leave(move |_ctrl| {
            let texture = render_svg_icon(icon_for_leave, &normal_color_leave, icon_size);
            pic_leave.set_paintable(Some(&texture));
        });
        btn.add_controller(motion_controller);

        let gesture = gtk4::GestureClick::new();
        let pic_pressed = pic.clone();
        let icon_for_pressed = icon;
        let active_color_pressed = active_color.clone();
        gesture.connect_pressed(move |_gesture, _n, _x, _y| {
            let texture = render_svg_icon(icon_for_pressed, &active_color_pressed, icon_size);
            pic_pressed.set_paintable(Some(&texture));
        });

        let pic_released = pic.clone();
        let icon_for_released = icon;
        gesture.connect_released(move |_gesture, _n, _x, _y| {
            let texture = render_svg_icon(icon_for_released, &hover_color, icon_size);
            pic_released.set_paintable(Some(&texture));
        });
        btn.add_controller(gesture);
    }

    (btn, pic)
}

/// Create a standalone SVG icon button without returning the picture handle
fn create_svg_icon_button(
    window: &Window,
    icon: WindowIcon,
    tooltip: &str,
    color: &str,
    icon_size: f64,
) -> Button {
    let (btn, _pic) = create_svg_icon_button_with_picture(window, icon, tooltip, color, icon_size);
    btn
}

/// Titlebar button configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TitlebarButtons {
    pub close: bool,
    pub minimize: bool,
    pub maximize: bool,
}

/// Titlebar controls returned to the caller
pub struct TitlebarControls {
    pub headerbar: HeaderBar,
    pub close_button: Option<Button>,
    pub minimize_button: Option<Button>,
    pub maximize_button: Option<Button>,
}

/// Create a custom Marco-styled titlebar with close button
///
/// # Arguments
/// * `window` - The window this titlebar belongs to (for theme detection and close action)
/// * `title` - The title text to display
///
/// # Returns
/// A tuple of `(HeaderBar, Button)` where the Button is the close button.
/// The caller should connect the close button to close the window.
///
/// # Example
/// ```
/// let (headerbar, close_btn) = create_custom_titlebar(&dialog, "About marco");
/// close_btn.connect_clicked(move |_| {
///     dialog.close();
/// });
/// dialog.set_titlebar(Some(&headerbar));
/// ```
pub fn create_custom_titlebar(window: &Window, title: &str) -> (HeaderBar, Button) {
    let controls = create_custom_titlebar_with_buttons(
        window,
        title,
        TitlebarButtons {
            close: true,
            minimize: false,
            maximize: false,
        },
    );

    let close_button = controls
        .close_button
        .expect("create_custom_titlebar must include a close button");

    (controls.headerbar, close_button)
}

/// Create a custom Marco-styled titlebar with configurable window controls
pub fn create_custom_titlebar_with_buttons(
    window: &Window,
    title: &str,
    buttons: TitlebarButtons,
) -> TitlebarControls {
    let headerbar = HeaderBar::new();
    headerbar.add_css_class("titlebar");
    headerbar.add_css_class("marco-titlebar");
    headerbar.set_show_title_buttons(false);

    // Title label
    let title_label = Label::new(Some(title));
    title_label.set_valign(Align::Center);
    title_label.add_css_class("title-label");
    headerbar.set_title_widget(Some(&title_label));

    // Close button with SVG icon
    let icon_color: std::borrow::Cow<'static, str> = if window.has_css_class("marco-theme-dark") {
        std::borrow::Cow::from(DARK_PALETTE.control_icon)
    } else {
        std::borrow::Cow::from(LIGHT_PALETTE.control_icon)
    };

    let mut minimize_button = None;
    let mut maximize_button = None;
    let mut close_button = None;

    if buttons.minimize {
        let button =
            create_svg_icon_button(window, WindowIcon::Minimize, "Minimize", &icon_color, 8.0);
        let window_for_min = window.clone();
        button.connect_clicked(move |_| {
            window_for_min.minimize();
        });
        headerbar.pack_end(&button);
        minimize_button = Some(button);
    }

    if buttons.maximize {
        let initial_icon = if window.is_maximized() {
            WindowIcon::Restore
        } else {
            WindowIcon::Maximize
        };
        let (button, picture) =
            create_svg_icon_button_with_picture(window, initial_icon, "Maximize", &icon_color, 8.0);
        let window_for_toggle = window.clone();
        button.connect_clicked(move |_| {
            if window_for_toggle.is_maximized() {
                window_for_toggle.unmaximize();
            } else {
                window_for_toggle.maximize();
            }
        });

        let picture_for_notify = picture.clone();
        let icon_color_for_notify = icon_color.to_string();
        window.connect_notify_local(Some("maximized"), move |win, _| {
            let icon = if win.is_maximized() {
                WindowIcon::Restore
            } else {
                WindowIcon::Maximize
            };
            let texture = render_svg_icon(icon, &icon_color_for_notify, 8.0);
            picture_for_notify.set_paintable(Some(&texture));
        });

        headerbar.pack_end(&button);
        maximize_button = Some(button);
    }

    if buttons.close {
        let button = create_svg_icon_button(window, WindowIcon::Close, "Close", &icon_color, 8.0);
        headerbar.pack_end(&button);
        close_button = Some(button);
    }

    TitlebarControls {
        headerbar,
        close_button,
        minimize_button,
        maximize_button,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_titlebar_creation() {
        if gtk4::is_initialized() {
            let window = Window::new();
            window.add_css_class("marco-theme-light");

            let (headerbar, close_btn) = create_custom_titlebar(&window, "Test Title");

            // Verify headerbar has correct classes
            assert!(headerbar.has_css_class("titlebar"));
            assert!(headerbar.has_css_class("marco-titlebar"));

            // Verify close button exists
            assert!(close_btn.has_css_class("window-control-btn"));
        }
    }
}
