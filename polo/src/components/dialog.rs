// Dialog management for file picker and Marco editor integration
//
//! # Dialog Module
//!
//! Manages user interaction dialogs for Polo:
//!
//! ## File Operations
//!
//! - **`show_open_file_dialog`**: Native GTK file chooser for opening markdown files
//!   - Filters for .md and .markdown files
//!   - Remembers last opened directory
//!   - Updates window title and settings on file selection
//!
//! ### Security Model
//!
//! Polo's file access follows the principle of **user permission delegation**:
//! - Runs with the user's own filesystem permissions
//! - Can only access files the user can already access
//! - No elevation of privileges or sandbox escape
//! - Markdown parsing is safe (no code execution)
//! - File paths are validated but not restricted beyond OS permissions
//!
//! This means Polo cannot access files the user couldn't access via the file manager
//! or command line. The "unrestricted" file access is actually restricted by the OS
//! user permission model, which is the appropriate security boundary for a desktop application.
//!
//! ## Marco Editor Integration
//!
//! - **`show_open_in_editor_dialog`**: Presents two options for opening file in Marco
//!   - **DualView**: Close Polo, open Marco with editor + preview
//!   - **Editor and View Separate**: Keep Polo open, also launch Marco
//!
//! - **`launch_marco`**: Locates and launches Marco editor binary
//!   - Checks same directory as Polo first
//!   - Falls back to system PATH
//!   - Returns detailed error messages on failure
//!
//! ## Error Handling
//!
//! All dialog operations handle errors gracefully:
//! - File picker failures are logged
//! - Marco launch failures show user-friendly error messages
//! - Invalid paths are validated before attempting operations

use crate::components::viewer::{load_and_render_markdown, platform_webview::PlatformWebView};
use core::logic::swanson::SettingsManager;
use gtk4::{
    prelude::*, Align, ApplicationWindow, Box, Button, FileChooserAction, FileChooserDialog,
    FileFilter, Label, Orientation, ResponseType, Window,
};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Show file chooser dialog to open a markdown file
pub fn show_open_file_dialog(
    window: &ApplicationWindow,
    webview: PlatformWebView,
    settings_manager: Arc<SettingsManager>,
    current_file_path: Arc<RwLock<Option<String>>>,
    open_editor_btn: &Button,
    title_label: &Label,
    asset_root: &std::path::Path,
) {
    use gtk4::gio;

    // Create file chooser dialog
    let dialog = FileChooserDialog::new(
        Some("Open Markdown File"),
        Some(window),
        FileChooserAction::Open,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Open", ResponseType::Accept),
        ],
    );

    // Add markdown file filter
    let filter = FileFilter::new();
    filter.set_name(Some("Markdown Files"));
    filter.add_pattern("*.md");
    filter.add_pattern("*.markdown");
    dialog.add_filter(&filter);

    // Add all files filter
    let filter_all = FileFilter::new();
    filter_all.set_name(Some("All Files"));
    filter_all.add_pattern("*");
    dialog.add_filter(&filter_all);

    // Set initial directory from settings
    let settings = settings_manager.get_settings();
    if let Some(polo) = &settings.polo {
        if let Some(ref last_file) = polo.last_opened_file {
            if let Some(parent) = std::path::Path::new(last_file).parent() {
                let _ = dialog.set_current_folder(Some(&gio::File::for_path(parent)));
            }
        }
    }

    // Handle response
    let window_weak = window.downgrade();
    let open_editor_btn = open_editor_btn.clone();
    let title_label = title_label.clone();
    let asset_root_owned = asset_root.to_path_buf();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            if let Some(file) = dialog.file() {
                if let Some(path) = file.path() {
                    let path_str = path.to_string_lossy().to_string();
                    log::info!("Opening file: {}", path_str);

                    // Get current theme from settings
                    let settings = settings_manager.get_settings();
                    let theme = settings
                        .appearance
                        .and_then(|a| a.preview_theme)
                        .unwrap_or_else(|| "github.css".to_string());

                    // Load and render the file
                    load_and_render_markdown(
                        &webview,
                        &path_str,
                        &theme,
                        &settings_manager,
                        &asset_root_owned,
                    );

                    // Update current file path
                    // RwLock poisoning is not expected in single-threaded GTK event loop.
                    // If it occurs (extremely unlikely), we simply retain the old path (safe fallback).
                    // This prevents the app from crashing on a non-critical state update.
                    if let Ok(mut path_guard) = current_file_path.write() {
                        *path_guard = Some(path_str.clone());
                    }

                    // Enable "Open in Editor" button now that we have a file
                    open_editor_btn.set_sensitive(true);
                    open_editor_btn.set_tooltip_text(Some("Open this file in Marco editor"));

                    // Update window title and title label
                    if let Some(window) = window_weak.upgrade() {
                        if let Some(filename) = path.file_name() {
                            let title_text = format!("Polo - {}", filename.to_string_lossy());
                            window.set_title(Some(&title_text));
                            title_label.set_text(&title_text);
                        }
                    }

                    // Save to settings
                    let _ = settings_manager.update_settings(|s| {
                        if s.polo.is_none() {
                            s.polo = Some(core::logic::swanson::PoloSettings::default());
                        }
                        if let Some(ref mut polo) = s.polo {
                            polo.last_opened_file = Some(PathBuf::from(path_str.clone()));
                        }
                    });
                }
            }
        }
        dialog.close();
    });

    dialog.present();
}

/// Show dialog asking how to open the file in Marco
pub fn show_open_in_editor_dialog(window: &ApplicationWindow, file_path: &str) {
    // Get current theme mode from parent window
    let theme_class = if window.has_css_class("marco-theme-dark") {
        "marco-theme-dark"
    } else {
        "marco-theme-light"
    };

    // Create a Window instead of deprecated Dialog
    let dialog = Window::builder()
        .modal(true)
        .transient_for(window)
        .default_width(420)
        .default_height(200)
        .resizable(false)
        .build();

    // Apply CSS classes for theming
    dialog.add_css_class("polo-dialog");
    dialog.add_css_class(theme_class);

    // Create custom titlebar matching polo's style
    let headerbar = gtk4::HeaderBar::new();
    headerbar.add_css_class("titlebar"); // Shared class for Marco's menu.css
    headerbar.add_css_class("polo-titlebar"); // Polo-specific class for overrides
    headerbar.set_show_title_buttons(false); // We'll add custom close button

    // Set title in headerbar
    let title_label = Label::new(Some("Open in Marco Editor"));
    title_label.set_valign(Align::Center);
    title_label.add_css_class("title-label"); // Shared class for Marco's menu.css
    title_label.add_css_class("polo-title-label"); // Polo-specific class
    headerbar.set_title_widget(Some(&title_label));

    // Create custom close button with SVG icon
    use crate::components::css::constants::{DARK_PALETTE, LIGHT_PALETTE};
    use core::logic::loaders::icon_loader::{window_icon_svg, WindowIcon};
    use rsvg::{CairoRenderer, Loader};
    use gio;
    use gtk4::gdk;

    fn render_svg_icon(icon: WindowIcon, color: &str, icon_size: f64) -> gdk::MemoryTexture {
        let svg = window_icon_svg(icon).replace("currentColor", color);
        let bytes = glib::Bytes::from_owned(svg.into_bytes());
        let stream = gio::MemoryInputStream::from_bytes(&bytes);

        let handle = match Loader::new().read_stream(&stream, None::<&gio::File>, gio::Cancellable::NONE) {
            Ok(h) => h,
            Err(e) => {
                log::error!("load SVG handle: {}", e);
                let bytes = glib::Bytes::from_owned(vec![0u8, 0u8, 0u8, 0u8]);
                return gdk::MemoryTexture::new(1, 1, gdk::MemoryFormat::B8g8r8a8Premultiplied, &bytes, 4);
            }
        };

        let display_scale = gdk::Display::default()
            .and_then(|d| d.monitors().item(0))
            .and_then(|m| m.downcast::<gdk::Monitor>().ok())
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
            renderer.render_document(&cr, &viewport).expect("render SVG");
        }

        let data = surface.data().expect("get surface data").to_vec();
        let bytes = glib::Bytes::from_owned(data);
        gdk::MemoryTexture::new(
            render_size,
            render_size,
            gdk::MemoryFormat::B8g8r8a8Premultiplied,
            &bytes,
            (render_size * 4) as usize,
        )
    }

    fn svg_icon_button(window: &Window, icon: WindowIcon, tooltip: &str, color: &str, icon_size: f64) -> Button {
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

        // Hover and click interactions
        {
            let pic_hover = pic.clone();
            let normal_color = color.to_string();
            let is_dark = window.has_css_class("marco-theme-dark");
            let hover_color = if is_dark { DARK_PALETTE.control_icon_hover.to_string() } else { LIGHT_PALETTE.control_icon_hover.to_string() };
            let active_color = if is_dark { DARK_PALETTE.control_icon_active.to_string() } else { LIGHT_PALETTE.control_icon_active.to_string() };

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

        btn
    }

    let icon_color: std::borrow::Cow<'static, str> = if dialog.has_css_class("marco-theme-dark") {
        std::borrow::Cow::from(DARK_PALETTE.control_icon)
    } else {
        std::borrow::Cow::from(LIGHT_PALETTE.control_icon)
    };

    let btn_close_titlebar = svg_icon_button(&dialog, WindowIcon::Close, "Close", &icon_color, 8.0);

    // Wire up close button
    let dialog_weak_for_close = dialog.downgrade();
    btn_close_titlebar.connect_clicked(move |_| {
        if let Some(dialog) = dialog_weak_for_close.upgrade() {
            dialog.close();
        }
    });

    // Add close button to right side of headerbar
    headerbar.pack_end(&btn_close_titlebar);

    dialog.set_titlebar(Some(&headerbar));

    // Create main content container
    let vbox = Box::new(Orientation::Vertical, 0);
    vbox.add_css_class("polo-dialog-content");

    // Message (removed duplicate title since it's now in titlebar)
    let message = Label::new(Some("Choose how to open this file in Marco:"));
    message.add_css_class("polo-dialog-message");
    message.set_halign(Align::Start);
    message.set_wrap(true);
    message.set_max_width_chars(45); // Constrain text width to match Marco's compact sizing
    vbox.append(&message);

    // Create button container
    let button_box = Box::new(Orientation::Vertical, 8);
    button_box.add_css_class("polo-dialog-button-box");

    // DualView button (primary action)
    let btn_dualview = Button::with_label("DualView");
    btn_dualview.add_css_class("polo-dialog-button");
    btn_dualview.add_css_class("primary");
    btn_dualview.set_tooltip_text(Some("Close Polo and open Marco with editor + preview"));
    button_box.append(&btn_dualview);

    // Editor and View Separate button
    let btn_separate = Button::with_label("Editor and View Separate");
    btn_separate.add_css_class("polo-dialog-button");
    btn_separate.set_tooltip_text(Some("Keep Polo open and also open Marco editor"));
    button_box.append(&btn_separate);

    // Cancel button container (separate with spacing)
    let cancel_container = Box::new(Orientation::Horizontal, 0);
    cancel_container.set_halign(Align::End);
    cancel_container.set_margin_top(8);

    let btn_cancel = Button::with_label("Cancel");
    btn_cancel.add_css_class("polo-dialog-button");
    cancel_container.append(&btn_cancel);

    vbox.append(&button_box);
    vbox.append(&cancel_container);

    dialog.set_child(Some(&vbox));

    // Handle button clicks
    let file_path = file_path.to_string();
    let window_weak = window.downgrade();
    let dialog_weak = dialog.downgrade();

    // DualView button - launch Marco and close Polo
    let file_path_clone = file_path.clone();
    let window_weak_clone = window_weak.clone();
    let dialog_weak_clone = dialog_weak.clone();
    btn_dualview.connect_clicked(move |_| {
        log::info!("DualView selected - launching Marco and closing Polo");

        if let Err(e) = launch_marco(&file_path_clone) {
            log::error!("Failed to launch Marco: {}", e);
        }

        // Close Polo
        if let Some(window) = window_weak_clone.upgrade() {
            window.close();
        }

        if let Some(dialog) = dialog_weak_clone.upgrade() {
            dialog.close();
        }
    });

    // Editor and View Separate button - launch Marco, keep Polo open
    let file_path_clone = file_path.clone();
    let dialog_weak_clone = dialog_weak.clone();
    btn_separate.connect_clicked(move |_| {
        log::info!("EditorAndViewSeparate selected - launching Marco, keeping Polo open");

        if let Err(e) = launch_marco(&file_path_clone) {
            log::error!("Failed to launch Marco: {}", e);
        }

        // Keep Polo open, just close dialog
        if let Some(dialog) = dialog_weak_clone.upgrade() {
            dialog.close();
        }
    });

    // Cancel button
    let dialog_weak_clone = dialog_weak.clone();
    btn_cancel.connect_clicked(move |_| {
        if let Some(dialog) = dialog_weak_clone.upgrade() {
            dialog.close();
        }
    });

    dialog.present();
}

/// Launch Marco editor with the specified file
pub fn launch_marco(file_path: &str) -> Result<(), String> {
    use std::process::Command;

    // Try to find marco binary
    // 1. Check in same directory as polo
    // 2. Check in PATH
    // 3. Check common install locations

    let polo_exe =
        std::env::current_exe().map_err(|e| format!("Failed to get current exe path: {}", e))?;

    let polo_dir = polo_exe
        .parent()
        .ok_or_else(|| "Failed to get polo directory".to_string())?;

    let marco_path = polo_dir.join("marco");

    let command = if marco_path.exists() {
        marco_path.to_string_lossy().to_string()
    } else {
        "marco".to_string() // Try PATH
    };

    Command::new(&command)
        .arg(file_path)
        .spawn()
        .map_err(|e| format!("Failed to spawn Marco process: {}", e))?;

    log::info!("Launched Marco: {} {}", command, file_path);
    Ok(())
}
