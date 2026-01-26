// Menu and titlebar components
//
//! # Menu Module
//!
//! Creates and manages Polo's custom titlebar with integrated controls.
//!
//! ## Titlebar Components
//!
//! ### Left Side
//! - **App Icon**: Polo favicon (16x16)
//! - **"Open" Button**: Opens file picker dialog
//! - **"Open in Editor" Button**: Launches Marco editor
//!
//! ### Center
//! - **Title Label**: Shows "Polo" or "Polo - filename.md"
//!
//! ### Right Side
//! - **Theme Dropdown**: Select HTML preview theme (github, marco, academic, etc.)
//! - **Mode Toggle**: Switch between light/dark modes (☀️/🌙)
//! - **Window Controls**: Minimize, Maximize/Restore, Close buttons
//!
//! ## Theme System
//!
//! The titlebar responds to theme changes:
//! - Applies `.marco-theme-light` or `.marco-theme-dark` CSS class
//! - Updates GTK global theme preference
//! - Reloads WebView content with new theme
//!
//! ## Icon Font
//!
//! Window control buttons use IcoMoon icon font:
//! - `\u{34}` - Minimize
//! - `\u{36}` - Maximize
//! - `\u{35}` - Restore
//! - `\u{39}` - Close
//!
//! ## Functions
//!
//! - **`create_custom_titlebar`**: Main function to build complete titlebar
//! - **`create_window_controls`**: Creates minimize/maximize/close buttons
//! - **`create_theme_dropdown`**: Builds and wires theme selector dropdown

use crate::components::dialog::{show_open_file_dialog, show_open_in_editor_dialog};
use crate::components::utils::{apply_gtk_theme_preference, list_available_themes_from_path};
use crate::components::viewer::{load_and_render_markdown, show_empty_state_with_theme};
use crate::components::viewer::platform_webview::PlatformWebView;
use core::logic::loaders::icon_loader::{window_icon_svg, WindowIcon};
use core::logic::swanson::SettingsManager;
use gtk4::{
    gdk, gio,
    prelude::*,
    Align,
    ApplicationWindow,
    Button,
    DropDown,
    Expression,
    HeaderBar,
    Image,
    Label,
    Picture,
    PropertyExpression,
    StringList,
    StringObject,
    WindowHandle,
};
use rsvg::{CairoRenderer, Loader};
use std::borrow::Cow;
use std::sync::{Arc, RwLock};

/// Create custom titlebar with icon, filename, theme dropdown, and "Open in Editor" button
///
/// Returns a tuple of (WindowHandle, Button, Label) where:
/// - WindowHandle: The titlebar handle
/// - Button: The "Open in Editor" button (enable/disable based on file open state)
/// - Label: The title label (update when file changes)
pub fn create_custom_titlebar(
    window: &ApplicationWindow,
    filename: &str,
    initial_theme: &str,
    settings_manager: Arc<SettingsManager>,
    webview: PlatformWebView,
    current_file_path: Arc<RwLock<Option<String>>>,
    asset_root: &std::path::Path,
) -> (WindowHandle, Button, Label) {
    // Create WindowHandle wrapper for proper window dragging
    let handle = WindowHandle::new();

    // Use GTK4 HeaderBar for proper title centering
    let headerbar = HeaderBar::new();
    headerbar.add_css_class("titlebar"); // Shared class for Marco's menu.css
    headerbar.add_css_class("polo-titlebar"); // Polo-specific class for overrides
    headerbar.set_show_title_buttons(false); // We'll add custom window controls

    // LEFT SIDE: App icon + filename
    let icon_path = asset_root.join("icons/icon_64x64_polo.png");
    let icon = Image::from_file(&icon_path);
    icon.set_pixel_size(16);
    icon.set_halign(Align::Start);
    icon.set_margin_start(5);
    icon.set_margin_end(5);
    icon.set_valign(Align::Center);
    icon.set_tooltip_text(Some("Polo - Markdown Viewer"));
    headerbar.pack_start(&icon);

    // "Open in Editor" button (create first so we can reference it in "Open" callback)
    let open_editor_btn = Button::with_label("Open in Editor");
    open_editor_btn.add_css_class("polo-open-editor-btn");
    open_editor_btn.set_valign(Align::Center);
    open_editor_btn.set_margin_end(6);
    open_editor_btn.set_tooltip_text(Some("Open this file in Marco editor"));

    // Check if a file is currently open and enable/disable button accordingly
    let has_file = current_file_path
        .read()
        .ok()
        .and_then(|guard| guard.as_ref().cloned())
        .is_some();
    open_editor_btn.set_sensitive(has_file);
    if !has_file {
        open_editor_btn.set_tooltip_text(Some("Open a file first to edit in Marco"));
    }

    // Wire up "Open in Editor" action
    let window_weak_for_editor = window.downgrade();
    let current_file_path_for_editor = current_file_path.clone();
    open_editor_btn.connect_clicked(move |_| {
        if let Some(window) = window_weak_for_editor.upgrade() {
            if let Ok(path_guard) = current_file_path_for_editor.read() {
                if let Some(ref path) = *path_guard {
                    show_open_in_editor_dialog(&window, path);
                } else {
                    log::warn!("No file path available to open in editor");
                }
            }
        }
    });

    // "Open" button (left side)
    let open_file_btn = Button::with_label("Open");
    open_file_btn.add_css_class("polo-open-file-btn");
    open_file_btn.set_valign(Align::Center);
    open_file_btn.set_margin_end(6);
    open_file_btn.set_tooltip_text(Some("Open a markdown file"));

    // Filename label (centered as title widget)
    // Show just "Polo" if no file is opened (filename is "Untitled")
    let title_text = if filename == "Untitled" {
        "Polo".to_string()
    } else {
        format!("Polo - {}", filename)
    };
    let title_label = Label::new(Some(&title_text));
    title_label.set_valign(Align::Center);
    title_label.add_css_class("title-label"); // Shared class for Marco's menu.css
    title_label.add_css_class("polo-title-label"); // Polo-specific class for overrides
    headerbar.set_title_widget(Some(&title_label));

    // Wire up "Open" action - pass open_editor_btn and title_label references
    let window_weak = window.downgrade();
    let webview_clone = webview.clone();
    let settings_manager_clone = settings_manager.clone();
    let current_file_path_clone = current_file_path.clone();
    let open_editor_btn_clone = open_editor_btn.clone();
    let title_label_clone = title_label.clone();
    let asset_root_for_dialog = asset_root.to_path_buf();
    open_file_btn.connect_clicked(move |_| {
        if let Some(window) = window_weak.upgrade() {
            show_open_file_dialog(
                &window,
                webview_clone.clone(),
                settings_manager_clone.clone(),
                current_file_path_clone.clone(),
                &open_editor_btn_clone,
                &title_label_clone,
                &asset_root_for_dialog,
            );
        }
    });
    headerbar.pack_start(&open_file_btn);
    headerbar.pack_start(&open_editor_btn);

    // RIGHT SIDE: Theme dropdown, window controls

    // Theme dropdown
    let theme_dropdown = create_theme_dropdown(
        initial_theme,
        settings_manager.clone(),
        webview.clone(),
        current_file_path.clone(),
        asset_root,
    );
    theme_dropdown.set_valign(Align::Center);
    theme_dropdown.set_margin_end(6);
    theme_dropdown.set_tooltip_text(Some("Select preview theme"));

    // Light/Dark mode toggle button
    let dark_mode_btn = Button::new();
    dark_mode_btn.set_valign(Align::Center);
    dark_mode_btn.set_margin_end(6);
    dark_mode_btn.set_focusable(false);
    dark_mode_btn.set_can_focus(false);
    dark_mode_btn.set_has_frame(true);
    dark_mode_btn.add_css_class("polo-mode-toggle-btn");

    // Determine current mode from settings
    let current_mode = {
        let settings = settings_manager.get_settings();
        settings
            .appearance
            .as_ref()
            .and_then(|a| a.editor_mode.as_ref())
            .map(|m| m.to_string())
            .unwrap_or_else(|| "light".to_string())
    };

    // Determine icon color based on current theme
    let mode_icon_color = if current_mode == "dark" {
        "#f0f5f1" // Light color for dark mode
    } else {
        "#2c3e50" // Dark color for light mode
    };

    // Create SVG icon for mode toggle (sun for dark mode, moon for light mode)
    let mode_icon = if current_mode == "dark" {
        WindowIcon::Sun
    } else {
        WindowIcon::Moon
    };
    let mode_pic = create_mode_icon_picture(mode_icon, mode_icon_color, 8.0);
    dark_mode_btn.set_child(Some(&mode_pic));
    dark_mode_btn.set_tooltip_text(Some(if current_mode == "dark" {
        "Switch to Light Mode"
    } else {
        "Switch to Dark Mode"
    }));

    // Wire up dark mode toggle
    let settings_manager_for_mode = settings_manager.clone();
    let webview_for_mode = webview.clone();
    let current_file_path_for_mode = current_file_path.clone();
    let dark_mode_btn_clone = dark_mode_btn.clone();
    let window_for_theme = window.clone();
    let asset_root_for_mode = asset_root.to_path_buf();
    dark_mode_btn.connect_clicked(move |_| {
        // Toggle mode
        let new_mode = {
            let settings = settings_manager_for_mode.get_settings();
            let current = settings
                .appearance
                .as_ref()
                .and_then(|a| a.editor_mode.as_ref())
                .map(|m| m.as_str())
                .unwrap_or("light");
            if current == "dark" {
                "light".to_string()
            } else {
                "dark".to_string()
            }
        };

        log::info!("Toggling color mode to: {}", new_mode);

        // Save to settings
        // Clone is necessary because the closure needs to own the value (move semantics)
        let new_mode_clone = new_mode.clone();
        let _ = settings_manager_for_mode.update_settings(move |s| {
            if s.appearance.is_none() {
                s.appearance = Some(core::logic::swanson::AppearanceSettings::default());
            }
            if let Some(ref mut appearance) = s.appearance {
                appearance.editor_mode = Some(new_mode_clone);
            }
        });

        // Update button icon and tooltip
        let new_icon_color = if new_mode == "dark" {
            "#f0f5f1" // Light color for dark mode
        } else {
            "#2c3e50" // Dark color for light mode
        };
        let new_icon = if new_mode == "dark" {
            WindowIcon::Sun
        } else {
            WindowIcon::Moon
        };
        let new_mode_pic = create_mode_icon_picture(new_icon, new_icon_color, 8.0);
        dark_mode_btn_clone.set_child(Some(&new_mode_pic));
        dark_mode_btn_clone.set_tooltip_text(Some(if new_mode == "dark" {
            "Switch to Light Mode"
        } else {
            "Switch to Dark Mode"
        }));

        // Apply GTK theme preference immediately
        apply_gtk_theme_preference(&settings_manager_for_mode);

        // Toggle CSS class on window for theme-specific styling
        let old_class = if new_mode == "dark" {
            "marco-theme-light"
        } else {
            "marco-theme-dark"
        };
        let new_class = format!("marco-theme-{}", new_mode);
        window_for_theme.remove_css_class(old_class);
        window_for_theme.add_css_class(&new_class);
        log::debug!("Switched CSS class from {} to {}", old_class, new_class);

        // Reload current file with new mode, or reload empty state if no file
        if let Ok(path_guard) = current_file_path_for_mode.read() {
            if let Some(ref path) = *path_guard {
                let theme = {
                    let settings = settings_manager_for_mode.get_settings();
                    settings
                        .appearance
                        .as_ref()
                        .and_then(|a| a.preview_theme.as_ref())
                        .map(|t| t.to_string())
                        .unwrap_or_else(|| "marco.css".to_string())
                };
                load_and_render_markdown(
                    &webview_for_mode,
                    path,
                    &theme,
                    &settings_manager_for_mode,
                    &asset_root_for_mode,
                );
            } else {
                // No file loaded - reload empty state with new theme
                show_empty_state_with_theme(&webview_for_mode, &settings_manager_for_mode);
            }
        }
    });

    // Create window control buttons
    let (btn_min, btn_max_toggle, btn_close) = create_window_controls(window, &settings_manager);

    // Add controls to headerbar from right to left (pack_end order)
    // Since pack_end adds from right to left, we add in reverse visual order:
    // First add window controls (they'll be rightmost)
    headerbar.pack_end(&btn_close); // Rightmost
    headerbar.pack_end(&btn_max_toggle); // Middle
    headerbar.pack_end(&btn_min); // Left of window controls
                                  // Then add dark mode toggle (left of window controls)
    headerbar.pack_end(&dark_mode_btn); // Left of minimize button
                                        // Then add theme dropdown (left of dark mode button)
    headerbar.pack_end(&theme_dropdown); // Left of dark mode button

    // Add the HeaderBar to the WindowHandle
    handle.set_child(Some(&headerbar));

    (handle, open_editor_btn, title_label)
}

/// Helper function to create a Picture widget with an SVG icon for mode toggle
fn create_mode_icon_picture(icon: WindowIcon, color: &str, size: f64) -> Picture {
    let svg = window_icon_svg(icon).replace("currentColor", color);
    let bytes = glib::Bytes::from_owned(svg.into_bytes());
    let stream = gio::MemoryInputStream::from_bytes(&bytes);
    
    let handle = Loader::new()
        .read_stream(&stream, None::<&gio::File>, gio::Cancellable::NONE)
        .expect("load SVG handle");
    
    // Get scale factor for HiDPI displays
    let display_scale = gdk::Display::default()
        .and_then(|d| d.monitors().item(0))
        .and_then(|m| m.downcast::<gdk::Monitor>().ok())
        .map(|m| m.scale_factor() as f64)
        .unwrap_or(1.0);
    
    // Render at 2x the display scale for extra sharpness
    let render_scale = display_scale * 2.0;
    let render_size = (size * render_scale) as i32;
    
    let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, render_size, render_size)
        .expect("create surface");
    {
        let cr = cairo::Context::new(&surface).expect("create context");
        cr.scale(render_scale, render_scale);
        
        let renderer = CairoRenderer::new(&handle);
        let viewport = cairo::Rectangle::new(0.0, 0.0, size, size);
        renderer.render_document(&cr, &viewport).expect("render SVG");
    }
    
    let data = surface.data().expect("get surface data").to_vec();
    let bytes = glib::Bytes::from_owned(data);
    let texture = gdk::MemoryTexture::new(
        render_size,
        render_size,
        gdk::MemoryFormat::B8g8r8a8Premultiplied,
        &bytes,
        (render_size * 4) as usize,
    );

    let pic = Picture::new();
    pic.set_paintable(Some(&texture));
    pic.set_size_request(size as i32, size as i32);
    pic.set_can_shrink(false);
    pic.set_halign(Align::Center);
    pic.set_valign(Align::Center);
    pic
}

/// Create window control buttons (minimize, maximize/restore, close)
fn create_window_controls(
    window: &ApplicationWindow,
    _settings_manager: &Arc<SettingsManager>,
) -> (Button, Button, Button) {
    // Single control point for icon size - change this value to resize all window control icons
    const ICON_SIZE: f64 = 8.0;
    
    // Shared SVG icon rendering function - renders at 2x resolution for crisp display
    fn render_svg_icon(icon: WindowIcon, color: &str, icon_size: f64) -> gdk::MemoryTexture {
        let svg = window_icon_svg(icon).replace("currentColor", color);
        let bytes = glib::Bytes::from_owned(svg.into_bytes());
        let stream = gio::MemoryInputStream::from_bytes(&bytes);
        
        // Use librsvg for native SVG rendering
        let handle = Loader::new()
            .read_stream(&stream, None::<&gio::File>, gio::Cancellable::NONE)
            .expect("load SVG handle");
        
        // Get scale factor for HiDPI displays
        let display_scale = gdk::Display::default()
            .and_then(|d| d.monitors().item(0))
            .and_then(|m| m.downcast::<gdk::Monitor>().ok())
            .map(|m| m.scale_factor() as f64)
            .unwrap_or(1.0);
        
        // Render at 2x the display scale for extra sharpness (prevents pixelation)
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
        } // Drop cr before accessing surface data
        
        // Convert cairo surface to GDK texture
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
    
    // Helper to create a button with SVG icon and hover/active color changes
    fn svg_icon_button(window: &ApplicationWindow, icon: WindowIcon, tooltip: &str, color: &str, icon_size: f64) -> Button {
        let pic = Picture::new();
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
        // Auto-calculate button size: icon + padding for comfortable click target
        btn.set_width_request((icon_size + 6.0) as i32);
        btn.set_height_request((icon_size + 6.0) as i32);
        btn.add_css_class("topright-btn");
        btn.add_css_class("window-control-btn");
        
        // Add hover state handling - regenerate icon with hover color
        {
            use crate::components::css::constants::{DARK_PALETTE, LIGHT_PALETTE};
            let pic_hover = pic.clone();
            let normal_color = color.to_string();
            let is_dark = window.style_context().has_class("marco-theme-dark");
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
            
            // Add click state handling
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

    // Use palette colors for window control icons (not hardcoded)
    let icon_color: Cow<'static, str> = {
        use crate::components::css::constants::{DARK_PALETTE, LIGHT_PALETTE};
        if window.style_context().has_class("marco-theme-dark") {
            Cow::from(DARK_PALETTE.control_icon)
        } else {
            Cow::from(LIGHT_PALETTE.control_icon)
        }
    };

    let btn_min = svg_icon_button(window, WindowIcon::Minimize, "Minimize", &icon_color, ICON_SIZE);
    let btn_close = svg_icon_button(window, WindowIcon::Close, "Close", &icon_color, ICON_SIZE);

    // Create maximize/restore toggle button with its own picture for dynamic icon switching
    let max_pic = Picture::new();
    max_pic.set_size_request(ICON_SIZE as i32, ICON_SIZE as i32);
    max_pic.set_can_shrink(false);
    max_pic.set_halign(Align::Center);
    max_pic.set_valign(Align::Center);

    // Helper closure to update maximize button icon based on window state
    let update_max_icon = {
        let color = icon_color.clone();
        move |is_maximized: bool, pic: &Picture| {
            let icon = if is_maximized {
                WindowIcon::Restore
            } else {
                WindowIcon::Maximize
            };
            let texture = render_svg_icon(icon, &color, ICON_SIZE);
            pic.set_paintable(Some(&texture));
        }
    };

    update_max_icon(window.is_maximized(), &max_pic);

    let btn_max_toggle = Button::new();
    btn_max_toggle.set_child(Some(&max_pic));
    btn_max_toggle.set_tooltip_text(Some("Maximize / Restore"));
    btn_max_toggle.set_valign(Align::Center);
    btn_max_toggle.set_margin_start(1);
    btn_max_toggle.set_margin_end(1);
    btn_max_toggle.set_focusable(false);
    // Auto-calculate button size: icon + padding for comfortable click target
    btn_max_toggle.set_width_request((ICON_SIZE + 6.0) as i32);
    btn_max_toggle.set_height_request((ICON_SIZE + 6.0) as i32);
    btn_max_toggle.set_can_focus(false);
    btn_max_toggle.set_has_frame(false);
    btn_max_toggle.add_css_class("topright-btn");
    btn_max_toggle.add_css_class("window-control-btn");

    // Add hover/active color changes for maximize button
    {
        use crate::components::css::constants::{DARK_PALETTE, LIGHT_PALETTE};
        let is_dark = window.style_context().has_class("marco-theme-dark");
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
        let normal_color = icon_color.to_string();
        
        let motion_controller = gtk4::EventControllerMotion::new();
        let pic_hover = max_pic.clone();
        let hover_color_enter = hover_color.clone();
        let window_hover_enter = window.clone();
        motion_controller.connect_enter(move |_ctrl, _x, _y| {
            let icon = if window_hover_enter.is_maximized() {
                WindowIcon::Restore
            } else {
                WindowIcon::Maximize
            };
            let texture = render_svg_icon(icon, &hover_color_enter, ICON_SIZE);
            pic_hover.set_paintable(Some(&texture));
        });
        
        let pic_leave = max_pic.clone();
        let normal_color_leave = normal_color.clone();
        let window_hover_leave = window.clone();
        motion_controller.connect_leave(move |_ctrl| {
            let icon = if window_hover_leave.is_maximized() {
                WindowIcon::Restore
            } else {
                WindowIcon::Maximize
            };
            let texture = render_svg_icon(icon, &normal_color_leave, ICON_SIZE);
            pic_leave.set_paintable(Some(&texture));
        });
        btn_max_toggle.add_controller(motion_controller);
        
        let gesture = gtk4::GestureClick::new();
        let pic_pressed = max_pic.clone();
        let active_color_pressed = active_color.clone();
        let window_pressed = window.clone();
        gesture.connect_pressed(move |_gesture, _n, _x, _y| {
            let icon = if window_pressed.is_maximized() {
                WindowIcon::Restore
            } else {
                WindowIcon::Maximize
            };
            let texture = render_svg_icon(icon, &active_color_pressed, ICON_SIZE);
            pic_pressed.set_paintable(Some(&texture));
        });
        
        let pic_released = max_pic.clone();
        let hover_color_released = hover_color.clone();
        let window_released = window.clone();
        gesture.connect_released(move |_gesture, _n, _x, _y| {
            let icon = if window_released.is_maximized() {
                WindowIcon::Restore
            } else {
                WindowIcon::Maximize
            };
            let texture = render_svg_icon(icon, &hover_color_released, ICON_SIZE);
            pic_released.set_paintable(Some(&texture));
        });
        btn_max_toggle.add_controller(gesture);
    }

    // Wire up window controls
    let window_for_min = window.clone();
    btn_min.connect_clicked(move |_| {
        window_for_min.minimize();
    });

    // Click toggles window state and updates glyph immediately
    let pic_for_toggle = max_pic.clone();
    let window_for_toggle = window.clone();
    let update_for_toggle = update_max_icon.clone();
    btn_max_toggle.connect_clicked(move |_| {
        if window_for_toggle.is_maximized() {
            window_for_toggle.unmaximize();
            update_for_toggle(false, &pic_for_toggle);
        } else {
            window_for_toggle.maximize();
            update_for_toggle(true, &pic_for_toggle);
        }
    });

    // Keep icon in sync if window is maximized/unmaximized externally
    let pic_for_notify = max_pic.clone();
    let update_for_notify = update_max_icon.clone();
    window.connect_notify_local(Some("is-maximized"), move |w, _| {
        update_for_notify(w.is_maximized(), &pic_for_notify);
    });

    let window_for_close = window.clone();
    btn_close.connect_clicked(move |_| {
        window_for_close.close();
    });

    (btn_min, btn_max_toggle, btn_close)
}

/// Create theme dropdown populated with available CSS themes
fn create_theme_dropdown(
    initial_theme: &str,
    settings_manager: Arc<SettingsManager>,
    webview: PlatformWebView,
    current_file_path: Arc<RwLock<Option<String>>>,
    asset_root: &std::path::Path,
) -> DropDown {
    // List available themes from assets/themes/html_viever/ (without .css extension)
    let theme_list = list_available_themes_from_path(asset_root);

    // Create StringList from theme names
    let string_list = StringList::new(&theme_list.iter().map(|s| s.as_str()).collect::<Vec<_>>());

    // Create dropdown with PropertyExpression
    let expression =
        PropertyExpression::new(StringObject::static_type(), None::<&Expression>, "string");

    let dropdown = DropDown::new(Some(string_list), Some(expression));
    dropdown.add_css_class("polo-theme-dropdown");

    // Set initial selection based on settings (strip .css from saved theme for comparison)
    let initial_theme_without_ext = initial_theme.trim_end_matches(".css");
    if let Some(index) = theme_list
        .iter()
        .position(|t| t == initial_theme_without_ext)
    {
        dropdown.set_selected(index as u32);
        log::debug!(
            "Set initial theme to: {} (index {})",
            initial_theme_without_ext,
            index
        );
    }

    // Save theme changes to COMMON appearance settings (shared with Marco)
    let theme_list_clone = theme_list.clone();
    let webview_clone = webview.clone();
    let asset_root_for_theme = asset_root.to_path_buf();
    dropdown.connect_selected_notify(move |dd| {
        let selected = dd.selected() as usize;
        if let Some(theme_name) = theme_list_clone.get(selected) {
            log::info!("Theme selected: {}", theme_name);

            // Add .css extension back for saving and loading
            let theme_name_with_ext = format!("{}.css", theme_name);

            // Update COMMON appearance.preview_theme (shared with Marco)
            let _ = settings_manager.update_settings(|s| {
                if s.appearance.is_none() {
                    s.appearance = Some(core::logic::swanson::AppearanceSettings::default());
                }
                if let Some(ref mut appearance) = s.appearance {
                    appearance.preview_theme = Some(theme_name_with_ext.clone());
                    log::debug!("Saved theme preference: {}", theme_name_with_ext);
                }
            });

            // Apply theme to WebView by reloading content with current file
            // RwLock poisoning is not expected in single-threaded GTK event loop.
            // If it occurs (extremely unlikely), we simply skip the reload (safe fallback).
            if let Ok(path_guard) = current_file_path.read() {
                if let Some(ref path) = *path_guard {
                    load_and_render_markdown(
                        &webview_clone,
                        path,
                        &theme_name_with_ext,
                        &settings_manager,
                        &asset_root_for_theme,
                    );
                    log::debug!("Reloaded WebView with theme: {}", theme_name_with_ext);
                } else {
                    // No file loaded - reload empty state with new theme
                    show_empty_state_with_theme(&webview_clone, &settings_manager);
                    log::debug!("Reloaded empty state with theme: {}", theme_name_with_ext);
                }
            }
        }
    });

    dropdown
}
