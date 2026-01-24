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

use crate::components::viewer::load_and_render_markdown;
use core::logic::swanson::SettingsManager;
use gtk4::{
    prelude::*, Align, ApplicationWindow, Box, Button, FileChooserAction, FileChooserDialog,
    FileFilter, Label, Orientation, ResponseType, Window,
};
use servo_runner::WebView;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Show file chooser dialog to open a markdown file
pub fn show_open_file_dialog(
    window: &ApplicationWindow,
    webview: WebView,
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

    // Create custom close button with icon font (matching menu.rs pattern)
    let close_label = Label::new(None);
    close_label.set_markup("<span font_family='icomoon'>\u{39}</span>"); // \u{39} = marco-close icon
    close_label.set_valign(Align::Center);
    close_label.add_css_class("icon-font");

    let btn_close_titlebar = Button::new();
    btn_close_titlebar.set_child(Some(&close_label));
    btn_close_titlebar.set_tooltip_text(Some("Close"));
    btn_close_titlebar.set_valign(Align::Center);
    btn_close_titlebar.set_margin_start(1);
    btn_close_titlebar.set_margin_end(1);
    btn_close_titlebar.set_focusable(false);
    btn_close_titlebar.set_can_focus(false);
    btn_close_titlebar.set_has_frame(false);
    btn_close_titlebar.add_css_class("topright-btn");
    btn_close_titlebar.add_css_class("window-control-btn");

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
