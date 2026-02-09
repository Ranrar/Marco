//! About dialog for Marco application

use core::logic::loaders::icon_loader::{about_icon_svg, AboutIcon};

use gtk4::prelude::*;
use gtk4::{gio, glib, Align, Box, Button, Label, Orientation, ScrolledWindow, Window};
use rsvg::{CairoRenderer, Loader};

use crate::components::language::DialogTranslations;

/// Render an SVG icon to a GdkMemoryTexture
fn render_about_icon(icon: AboutIcon, color: &str, icon_size: f64) -> gtk4::gdk::MemoryTexture {
    let svg = about_icon_svg(icon).replace("currentColor", color);
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

/// Create a clickable link button with icon and label
fn create_link_button(
    dialog: &Window,
    icon: AboutIcon,
    label_text: &str,
    url: &str,
    is_dark: bool,
    _event_name: &'static str,
) -> Box {
    let link_box = Box::new(Orientation::Vertical, 4);
    link_box.set_halign(Align::Center);
    link_box.set_valign(Align::Start);

    // Icon
    let icon_color = if is_dark { "#E1E1E1" } else { "#2E2E2E" };
    let pic = gtk4::Picture::new();
    let texture = render_about_icon(icon, icon_color, 24.0);
    pic.set_paintable(Some(&texture));
    pic.set_size_request(24, 24);
    pic.set_can_shrink(false);
    pic.set_halign(Align::Center);
    pic.set_valign(Align::Center);
    link_box.append(&pic);

    // Label
    let label = Label::new(Some(label_text));
    label.add_css_class("marco-dialog-message");
    label.set_halign(Align::Center);
    label.set_justify(gtk4::Justification::Center);
    label.set_max_width_chars(10);
    label.set_wrap(true);
    label.set_wrap_mode(gtk4::pango::WrapMode::Word);
    link_box.append(&label);

    // Make the box clickable
    let gesture = gtk4::GestureClick::new();
    {
        let dialog_clone = dialog.clone();
        let url_owned = url.to_string();
        gesture.connect_released(move |_gesture, _n, _x, _y| {
            gtk4::show_uri(Some(&dialog_clone), &url_owned, gtk4::gdk::CURRENT_TIME);
        });
    }
    link_box.add_controller(gesture);

    // Add hover effect
    let hover_controller = gtk4::EventControllerMotion::new();
    {
        let link_box_clone = link_box.clone();
        hover_controller.connect_enter(move |_ctrl, _x, _y| {
            link_box_clone.set_opacity(0.7);
        });
    }
    {
        let link_box_clone = link_box.clone();
        hover_controller.connect_leave(move |_ctrl| {
            link_box_clone.set_opacity(1.0);
        });
    }
    link_box.add_controller(hover_controller);

    // Add cursor pointer style
    link_box.set_cursor_from_name(Some("pointer"));

    link_box
}

/// Show the About dialog with application information
///
/// # Arguments
/// * `parent` - Parent window for the dialog
pub fn show_about_dialog(parent: &impl IsA<gtk4::Window>, translations: &DialogTranslations) {
    // Get current theme mode from parent window
    let theme_class = if let Some(widget) = parent.dynamic_cast_ref::<gtk4::Widget>() {
        if widget.has_css_class("marco-theme-dark") {
            "marco-theme-dark"
        } else {
            "marco-theme-light"
        }
    } else {
        "marco-theme-light" // Default to light theme
    };
    let is_dark = theme_class == "marco-theme-dark";

    // Create dialog window (smaller, resizable)
    let dialog = Window::builder()
        .modal(true)
        .transient_for(parent)
        .default_width(600)
        .default_height(500)
        .resizable(true)
        .build();
    // Ensure window cannot be made smaller than the default
    dialog.set_size_request(600, 500);

    // Apply theme CSS classes
    dialog.add_css_class("marco-dialog");
    dialog.add_css_class(theme_class);

    // Create custom titlebar using reusable function
    let titlebar_controls = crate::ui::titlebar::create_custom_titlebar_with_buttons(
        &dialog,
        &translations.about_title,
        crate::ui::titlebar::TitlebarButtons {
            close: true,
            minimize: false,
            maximize: false,
        },
    );
    let close_button = titlebar_controls
        .close_button
        .as_ref()
        .expect("About dialog requires a close button");
    {
        let dialog_clone = dialog.clone();
        close_button.connect_clicked(move |_| {
            dialog_clone.close();
        });
    }
    dialog.set_titlebar(Some(&titlebar_controls.headerbar));

    // Create content container
    let content_box = Box::new(Orientation::Vertical, 0);
    content_box.add_css_class("marco-dialog-content");
    content_box.set_halign(Align::Fill);
    content_box.set_valign(Align::Center);

    // App name
    let name_label = Label::new(Some(&translations.about_app_name));
    name_label.add_css_class("marco-dialog-title");
    name_label.set_halign(Align::Center);
    content_box.append(&name_label);

    // Mission tagline
    let tagline_label = Label::new(Some(&translations.about_tagline));
    tagline_label.add_css_class("marco-dialog-message");
    tagline_label.set_halign(Align::Center);
    tagline_label.set_margin_bottom(8);
    content_box.append(&tagline_label);

    // Version
    let version_label = Label::new(Some(
        &translations
            .about_version
            .replace("{version}", env!("CARGO_PKG_VERSION")),
    ));
    version_label.add_css_class("marco-dialog-message");
    version_label.set_halign(Align::Center);
    version_label.set_margin_bottom(16);
    content_box.append(&version_label);

    // Main description and features
    let main_text = Label::new(Some(&translations.about_description));
    main_text.add_css_class("marco-dialog-message");
    main_text.set_halign(Align::Center);
    main_text.set_justify(gtk4::Justification::Left);
    main_text.set_max_width_chars(48);
    main_text.set_wrap(true);
    main_text.set_wrap_mode(gtk4::pango::WrapMode::Word);
    main_text.set_margin_bottom(12);
    content_box.append(&main_text);

    // Links section title
    let links_title = Label::new(Some(&translations.about_resources_title));
    links_title.add_css_class("marco-dialog-title");
    links_title.set_halign(Align::Center);
    links_title.set_margin_top(4);
    links_title.set_margin_bottom(10);
    content_box.append(&links_title);

    // First row of links
    let links_box1 = Box::new(Orientation::Horizontal, 16);
    links_box1.set_halign(Align::Center);
    links_box1.set_valign(Align::Start);
    links_box1.set_margin_bottom(12);

    // GitHub repository link
    let github_link = create_link_button(
        &dialog,
        AboutIcon::GitHub,
        &translations.about_link_github,
        "https://github.com/Ranrar/Marco",
        is_dark,
        "dialog_about_link_github",
    );
    links_box1.append(&github_link);

    // Bug reports link
    let bug_link = create_link_button(
        &dialog,
        AboutIcon::Bug,
        &translations.about_link_issues,
        "https://github.com/Ranrar/Marco/issues",
        is_dark,
        "dialog_about_link_issues",
    );
    links_box1.append(&bug_link);

    // Help/discussions link
    let help_link = create_link_button(
        &dialog,
        AboutIcon::Help,
        &translations.about_link_discuss,
        "https://github.com/Ranrar/Marco/discussions",
        is_dark,
        "dialog_about_link_discuss",
    );
    links_box1.append(&help_link);

    // Changelog link
    let changelog_link = create_link_button(
        &dialog,
        AboutIcon::Link,
        &translations.about_link_changelog,
        "https://github.com/Ranrar/Marco/blob/main/changelog/marco.md",
        is_dark,
        "dialog_about_link_changelog",
    );
    links_box1.append(&changelog_link);

    content_box.append(&links_box1);

    // Second row of links for additional resources
    let links_box2 = Box::new(Orientation::Horizontal, 16);
    links_box2.set_halign(Align::Center);
    links_box2.set_valign(Align::Start);
    links_box2.set_margin_bottom(16);

    // Website link
    //    let website_link = create_link_button(
    //        &dialog,
    //        AboutIcon::Link,
    //        &translations.about_link_website,
    //        "https://www.skovrasmussen.com",
    //        is_dark,
    //        "dialog_about_link_website",
    //    );
    //    links_box2.append(&website_link);
    //
    //    content_box.append(&links_box2);

    // License text (paragraph-style for proper reflow and selection)
    let license_text = Label::new(Some(&translations.about_license_text));
    license_text.add_css_class("marco-dialog-message");
    license_text.set_halign(Align::Center);
    license_text.set_justify(gtk4::Justification::Left);
    license_text.set_max_width_chars(48);
    license_text.set_wrap(true);
    license_text.set_wrap_mode(gtk4::pango::WrapMode::Word);
    license_text.set_margin_bottom(12);
    content_box.append(&license_text);

    // Creator
    let about_tech = Label::new(Some(&translations.about_copyright));
    about_tech.add_css_class("marco-dialog-message");
    about_tech.set_halign(Align::Center);
    about_tech.set_justify(gtk4::Justification::Left);
    about_tech.set_max_width_chars(48);
    about_tech.set_wrap(true);
    about_tech.set_wrap_mode(gtk4::pango::WrapMode::Word);
    about_tech.set_margin_bottom(16);
    content_box.append(&about_tech);

    // Close button at the bottom
    let button_box = Box::new(Orientation::Horizontal, 6);
    button_box.add_css_class("marco-dialog-button-box");
    button_box.set_halign(Align::Center);
    button_box.set_margin_top(16);

    let close_btn_bottom = Button::with_label(&translations.about_close_button);
    close_btn_bottom.add_css_class("marco-btn");
    close_btn_bottom.add_css_class("marco-btn-blue");
    {
        let dialog_clone = dialog.clone();
        close_btn_bottom.connect_clicked(move |_| {
            dialog_clone.close();
        });
    }
    button_box.append(&close_btn_bottom);
    content_box.append(&button_box);

    // Put content inside a scrolled window so long content can scroll
    let scroller = ScrolledWindow::new();
    scroller.add_css_class("editor-scrolled");
    scroller.set_min_content_height(300);
    scroller.set_vexpand(true);
    scroller.set_overlay_scrolling(true);
    scroller.set_child(Some(&content_box));
    dialog.set_child(Some(&scroller));

    dialog.connect_close_request(move |_| glib::Propagation::Proceed);

    dialog.present();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::language::LocalizationProvider;

    #[test]
    fn smoke_test_about_dialog_creation() {
        // GTK initialization required for dialog creation
        if gtk4::is_initialized() {
            let window = gtk4::Window::new();

            // Should not panic
            let translations = match crate::components::language::SimpleLocalizationManager::new() {
                Ok(manager) => manager.translations(),
                Err(_) => {
                    println!("Skipping: assets not available in test environment");
                    return;
                }
            };

            show_about_dialog(&window, &translations.dialog);
        }
    }
}
