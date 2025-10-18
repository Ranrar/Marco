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
use gtk4::{
    prelude::*, Align, ApplicationWindow, Button, DropDown, Expression, HeaderBar,
    Image, Label, PropertyExpression, StringList, StringObject, WindowHandle,
};
use core::logic::swanson::SettingsManager;
use std::sync::{Arc, RwLock};
use webkit6::WebView;

/// Create custom titlebar with icon, filename, theme dropdown, and "Open in Editor" button
/// 
/// Returns a tuple of (WindowHandle, Button) where the Button is the "Open in Editor" button
/// that should be enabled/disabled based on whether a file is open.
pub fn create_custom_titlebar(
    window: &ApplicationWindow,
    filename: &str,
    initial_theme: &str,
    settings_manager: Arc<SettingsManager>,
    webview: WebView,
    current_file_path: Arc<RwLock<Option<String>>>,
    asset_root: &std::path::Path,
) -> (WindowHandle, Button) {
    // Create WindowHandle wrapper for proper window dragging
    let handle = WindowHandle::new();
    
    // Use GTK4 HeaderBar for proper title centering
    let headerbar = HeaderBar::new();
    headerbar.add_css_class("titlebar");      // Shared class for Marco's menu.css
    headerbar.add_css_class("polo-titlebar"); // Polo-specific class for overrides
    headerbar.set_show_title_buttons(false); // We'll add custom window controls
    
    // LEFT SIDE: App icon + filename
    let icon_path = asset_root.join("icons/favicon.png");
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
    let has_file = current_file_path.read().ok()
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
    
    // Wire up "Open" action - pass open_editor_btn reference
    let window_weak = window.downgrade();
    let webview_clone = webview.clone();
    let settings_manager_clone = settings_manager.clone();
    let current_file_path_clone = current_file_path.clone();
    let open_editor_btn_clone = open_editor_btn.clone();
    let asset_root_for_dialog = asset_root.to_path_buf();
    open_file_btn.connect_clicked(move |_| {
        if let Some(window) = window_weak.upgrade() {
            show_open_file_dialog(
                &window,
                webview_clone.clone(),
                settings_manager_clone.clone(),
                current_file_path_clone.clone(),
                &open_editor_btn_clone,
                &asset_root_for_dialog,
            );
        }
    });
    headerbar.pack_start(&open_file_btn);
    headerbar.pack_start(&open_editor_btn);
    
    // Filename label (centered as title widget)
    // Show just "Polo" if no file is opened (filename is "Untitled")
    let title_text = if filename == "Untitled" {
        "Polo".to_string()
    } else {
        format!("Polo - {}", filename)
    };
    let title_label = Label::new(Some(&title_text));
    title_label.set_valign(Align::Center);
    title_label.add_css_class("title-label");      // Shared class for Marco's menu.css
    title_label.add_css_class("polo-title-label"); // Polo-specific class for overrides
    headerbar.set_title_widget(Some(&title_label));
    
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
    
    // Set initial icon (☀️ for light mode, 🌙 for dark mode)
    let mode_label = Label::new(Some(if current_mode == "dark" {
        "☀️"
    } else {
        "🌙"
    }));
    mode_label.set_valign(Align::Center);
    dark_mode_btn.set_child(Some(&mode_label));
    dark_mode_btn.set_tooltip_text(Some(if current_mode == "dark" {
        "Switch to Light Mode"
    } else {
        "Switch to Dark Mode"
    }));
    
    // Wire up dark mode toggle
    let settings_manager_for_mode = settings_manager.clone();
    let webview_for_mode = webview.clone();
    let current_file_path_for_mode = current_file_path.clone();
    let mode_label_clone = mode_label.clone();
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
        mode_label_clone.set_text(if new_mode == "dark" { "☀️" } else { "🌙" });
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
    let (btn_min, btn_max_toggle, btn_close) =
        create_window_controls(window, &settings_manager);
    
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
    
    (handle, open_editor_btn)
}

/// Create window control buttons (minimize, maximize/restore, close)
fn create_window_controls(
    window: &ApplicationWindow,
    _settings_manager: &Arc<SettingsManager>,
) -> (Button, Button, Button) {
    // Helper to create a button with icon font (matching Marco's pattern)
    fn icon_button(label_text: &str, tooltip: &str) -> Button {
        let markup = format!("<span font_family='icomoon'>{}</span>", label_text);
        let label = Label::new(None);
        label.set_markup(&markup);
        label.set_valign(Align::Center);
        label.add_css_class("icon-font");
        let btn = Button::new();
        btn.set_child(Some(&label));
        btn.set_tooltip_text(Some(tooltip));
        btn.set_valign(Align::Center);
        btn.set_margin_start(1);
        btn.set_margin_end(1);
        btn.set_focusable(false);
        btn.set_can_focus(false);
        btn.set_has_frame(false);
        btn.add_css_class("topright-btn");
        btn.add_css_class("window-control-btn");
        btn
    }
    
    // IcoMoon Unicode glyphs for window controls
    // These unicode characters reference glyphs in the IcoMoon icon font (ui_menu.ttf)
    // loaded from assets/fonts/. The font must be loaded via core::logic::loaders::icon_loader
    // before GTK initialization for these characters to display correctly.
    //
    // | Unicode | Icon Name             | Description   |
    // |---------|-----------------------|--------------|
    // | \u{34}  | marco-minimize        | Minimize      |
    // | \u{36}  | marco-fullscreen      | Maximize      |
    // | \u{35}  | marco-fullscreen_exit | Exit maximize |
    // | \u{39}  | marco-close           | Close         |
    
    let btn_min = icon_button("\u{34}", "Minimize");
    let btn_close = icon_button("\u{39}", "Close");
    
    // Create a single toggle button for maximize/restore and keep its label so we can update it
    let max_label = Label::new(None);
    let initial_glyph = if window.is_maximized() {
        "\u{35}"
    } else {
        "\u{36}"
    };
    max_label.set_markup(&format!(
        "<span font_family='icomoon'>{}</span>",
        initial_glyph
    ));
    max_label.set_valign(Align::Center);
    max_label.add_css_class("icon-font");
    let btn_max_toggle = Button::new();
    btn_max_toggle.set_child(Some(&max_label));
    btn_max_toggle.set_tooltip_text(Some("Maximize / Restore"));
    btn_max_toggle.set_valign(Align::Center);
    btn_max_toggle.set_margin_start(1);
    btn_max_toggle.set_margin_end(1);
    btn_max_toggle.set_focusable(false);
    btn_max_toggle.set_can_focus(false);
    btn_max_toggle.set_has_frame(false);
    btn_max_toggle.add_css_class("topright-btn");
    btn_max_toggle.add_css_class("window-control-btn");
    
    // Wire up window controls
    let window_for_min = window.clone();
    btn_min.connect_clicked(move |_| {
        window_for_min.minimize();
    });
    
    // Click toggles window state and updates glyph immediately
    let label_for_toggle = max_label.clone();
    let window_for_toggle = window.clone();
    btn_max_toggle.connect_clicked(move |_| {
        if window_for_toggle.is_maximized() {
            window_for_toggle.unmaximize();
            label_for_toggle.set_markup(&format!(
                "<span font_family='icomoon'>{}</span>",
                "\u{36}"
            ));
        } else {
            window_for_toggle.maximize();
            label_for_toggle.set_markup(&format!(
                "<span font_family='icomoon'>{}</span>",
                "\u{35}"
            ));
        }
    });
    
    // Keep glyph in sync if window is maximized/unmaximized externally
    let label_for_notify = max_label.clone();
    window.connect_notify_local(Some("is-maximized"), move |w, _| {
        if w.is_maximized() {
            label_for_notify.set_markup(&format!(
                "<span font_family='icomoon'>{}</span>",
                "\u{35}"
            ));
        } else {
            label_for_notify.set_markup(&format!(
                "<span font_family='icomoon'>{}</span>",
                "\u{36}"
            ));
        }
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
    webview: WebView,
    current_file_path: Arc<RwLock<Option<String>>>,
    asset_root: &std::path::Path,
) -> DropDown {
    // List available themes from assets/themes/html_viever/ (without .css extension)
    let theme_list = list_available_themes_from_path(asset_root);
    
    // Create StringList from theme names
    let string_list = StringList::new(
        &theme_list
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>(),
    );
    
    // Create dropdown with PropertyExpression
    let expression = PropertyExpression::new(
        StringObject::static_type(),
        None::<&Expression>,
        "string",
    );
    
    let dropdown = DropDown::new(Some(string_list), Some(expression));
    dropdown.add_css_class("polo-theme-dropdown");
    
    // Set initial selection based on settings (strip .css from saved theme for comparison)
    let initial_theme_without_ext = initial_theme.trim_end_matches(".css");
    if let Some(index) = theme_list.iter().position(|t| t == initial_theme_without_ext) {
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
                    s.appearance =
                        Some(core::logic::swanson::AppearanceSettings::default());
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
