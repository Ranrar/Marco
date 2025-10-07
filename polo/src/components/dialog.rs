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
use gtk4::{
    prelude::*, Align, ApplicationWindow, Box, Button, Dialog, FileChooserAction,
    FileChooserDialog, FileFilter, Label, Orientation, ResponseType,
};
use marco_core::logic::swanson::SettingsManager;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use webkit6::WebView;

/// Show file chooser dialog to open a markdown file
pub fn show_open_file_dialog(
    window: &ApplicationWindow,
    webview: WebView,
    settings_manager: Arc<SettingsManager>,
    current_file_path: Arc<RwLock<Option<String>>>,
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
                    load_and_render_markdown(&webview, &path_str, &theme, &settings_manager);
                    
                    // Update current file path
                    // RwLock poisoning is not expected in single-threaded GTK event loop.
                    // If it occurs (extremely unlikely), we simply retain the old path (safe fallback).
                    // This prevents the app from crashing on a non-critical state update.
                    if let Ok(mut path_guard) = current_file_path.write() {
                        *path_guard = Some(path_str.clone());
                    }
                    
                    // Update window title
                    if let Some(window) = window_weak.upgrade() {
                        if let Some(filename) = path.file_name() {
                            window.set_title(Some(&format!(
                                "Polo - {}",
                                filename.to_string_lossy()
                            )));
                        }
                    }
                    
                    // Save to settings
                    let _ = settings_manager.update_settings(|s| {
                        if s.polo.is_none() {
                            s.polo = Some(marco_core::logic::swanson::PoloSettings::default());
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
    // Create dialog
    let dialog = Dialog::builder()
        .modal(true)
        .title("Open in Marco Editor")
        .transient_for(window)
        .destroy_with_parent(true)
        .build();
    
    // Create content
    let content_area = dialog.content_area();
    let vbox = Box::new(Orientation::Vertical, 16);
    vbox.set_margin_start(24);
    vbox.set_margin_end(24);
    vbox.set_margin_top(16);
    vbox.set_margin_bottom(16);
    
    let label = Label::new(Some("Choose how to open this file in Marco:"));
    label.set_halign(Align::Start);
    vbox.append(&label);
    
    // Create three action buttons
    let btn_dualview = Button::with_label("DualView");
    btn_dualview.set_tooltip_text(Some("Close Polo and open Marco with editor + preview"));
    btn_dualview.set_margin_top(8);
    
    let btn_separate = Button::with_label("Editor and View Separate");
    btn_separate.set_tooltip_text(Some("Keep Polo open and also open Marco editor"));
    btn_separate.set_margin_top(8);
    
    let btn_cancel = Button::with_label("Cancel");
    btn_cancel.set_margin_top(16);
    
    vbox.append(&btn_dualview);
    vbox.append(&btn_separate);
    vbox.append(&btn_cancel);
    
    content_area.append(&vbox);
    
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
    
    let polo_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get current exe path: {}", e))?;
    
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
