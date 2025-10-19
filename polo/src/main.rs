// Polo - Lightweight Markdown Viewer
// Standalone viewer for Marco markdown files
//
//! # Polo - Lightweight Markdown Viewer
//!
//! Polo is a standalone markdown viewer designed as the lightweight companion to the Marco
//! markdown editor. It provides a read-only view of markdown files with full support for
//! Marco's custom markdown extensions and syntax highlighting.
//!
//! ## Key Features
//!
//! - **Pure Viewer**: No editing capabilities - focused on viewing only
//! - **Marco Integration**: Opens files in Marco editor on demand
//! - **Theme Support**: Light/dark modes with multiple CSS themes
//! - **Fast Rendering**: Uses core's cached parser for instant previews
//! - **Minimal Dependencies**: No SourceView5, just GTK4 + WebKit6
//!
//! ## Architecture
//!
//! Polo follows Marco's architectural patterns:
//! - **main.rs**: Application gateway only - no business logic
//! - **components/**: All UI components and logic organized by function
//! - **core**: Shared parsing, rendering, and settings management
//!
//! ## Settings Integration
//!
//! Polo shares common settings with Marco (themes, appearance) while maintaining
//! its own section for viewer-specific settings (window size, last opened file).
//!
//! ## Command Line Usage
//!
//! ```bash
//! polo <file.md>           # Open markdown file
//! polo --debug <file.md>   # Open with debug logging
//! polo --help              # Show help message
//! ```

mod components;

use components::css::load_css_from_path;
use components::menu::create_custom_titlebar;
use components::utils::{apply_gtk_theme_preference, parse_hex_to_rgba};
use components::viewer::{load_and_render_markdown, show_empty_state_with_theme};
use gtk4::{gio, glib, prelude::*, Application, ApplicationWindow};
use core::paths::PoloPaths;
use webkit6::prelude::WebViewExt;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

const APP_ID: &str = "com.example.Polo";

/// Centralized fatal error handler
/// 
/// This function handles unrecoverable errors during application initialization.
/// It ensures proper cleanup (logger shutdown) before terminating the application.
/// 
/// # Arguments
/// * `message` - User-friendly error message to display
/// 
/// # Panics
/// This function never returns - it always exits the process with code 1
fn fatal_error(message: &str) -> ! {
    log::error!("FATAL: {}", message);
    eprintln!("Fatal error: {}", message);
    core::logic::logger::shutdown_file_logger();
    std::process::exit(1);
}

fn main() -> glib::ExitCode {   
    // Initialize logger early
    if let Err(e) = core::logic::logger::init_file_logger(true, log::LevelFilter::Debug) {
        // Fallback: print to stderr if logger fails
        eprintln!("Failed to initialize logger: {}", e);
    }

    // Setup font directory for IcoMoon icon font (MUST be done before GTK init)
    use core::paths::{PoloPaths, PathProvider};
    let polo_paths = match PoloPaths::new() {
        Ok(paths) => paths,
        Err(e) => {
            fatal_error(&format!("Cannot initialize Polo paths: {:?}", e));
        }
    };
    
    // Set local font dir for Fontconfig/Pango
    // Note: set_local_font_dir expects the parent of fonts/, not fonts/ itself
    // It sets XDG_DATA_HOME, and Fontconfig looks in $XDG_DATA_HOME/fonts/
    let asset_root_for_fonts = polo_paths.asset_root();
    core::logic::loaders::icon_loader::set_local_font_dir(
        asset_root_for_fonts.to_str().expect("Invalid asset root path")
    );
    
    // Verify font is accessible
    let ui_menu_font = polo_paths.shared().font("ui_menu.ttf");
    if !ui_menu_font.exists() {
        log::warn!("UI menu font not found at {:?}", ui_menu_font);
        log::warn!("Icon font may not display correctly");
    } else {
        log::debug!("Icon font loaded: {}", ui_menu_font.display());
    }

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN | gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    // Wrap polo_paths in Rc for sharing across closures
    let polo_paths = std::rc::Rc::new(polo_paths);
    let polo_paths_for_cmdline = polo_paths.clone();
    let polo_paths_for_open = polo_paths.clone();

    // Handle command-line arguments
    app.connect_command_line(move |app, cmd_line| {
        let args: Vec<String> = cmd_line.arguments().iter().map(|s| s.to_string_lossy().to_string()).collect();
        
        // Parse arguments
        if args.len() > 1 {
            for arg in &args[1..] {
                if arg == "--help" || arg == "-h" {
                    println!("Polo - Lightweight Markdown Viewer");
                    println!("\nUsage:");
                    println!("  polo <file.md>           Open markdown file");
                    println!("  polo --debug <file.md>   Open with debug logging");
                    println!("  polo --help              Show this help message");
                    return 0.into();
                } else if arg == "--debug" {
                    // Debug flag already handled by logger init
                    continue;
                } else if arg.ends_with(".md") || arg.ends_with(".markdown") {
                    // Found markdown file
                    build_ui(app, Some(arg.clone()), polo_paths_for_cmdline.clone());
                    return 0.into();
                } else if !arg.starts_with('-') {
                    // Treat as file path
                    build_ui(app, Some(arg.clone()), polo_paths_for_cmdline.clone());
                    return 0.into();
                }
            }
        }
        
        // No file specified - open empty Polo
        build_ui(app, None, polo_paths_for_cmdline.clone());
        0.into()
    });

    // Handle file opening via file manager (drag & drop, right-click)
    app.connect_open(move |app, files, _hint| {
        if let Some(file) = files.first() {
            if let Some(path) = file.path() {
                build_ui(app, Some(path.to_string_lossy().to_string()), polo_paths_for_open.clone());
            }
        }
    });

    let exit_code = app.run();
    
    // Cleanup
    core::logic::logger::shutdown_file_logger();
    exit_code
}

fn build_ui(app: &Application, file_path: Option<String>, polo_paths: std::rc::Rc<PoloPaths>) {
    use core::paths::PathProvider;
    
    // Initialize settings manager early
    let settings_path = polo_paths.settings_file();
    
    let settings_manager = match core::logic::swanson::SettingsManager::initialize(settings_path.clone()) {
        Ok(manager) => {
            log::debug!("Settings loaded successfully");
            manager
        },
        Err(e) => {
            log::warn!("Failed to load settings, using defaults: {}", e);
            // Create default settings and continue
            match core::logic::swanson::SettingsManager::initialize(settings_path) {
                Ok(manager) => manager,
                Err(e) => {
                    fatal_error(&format!("Cannot initialize settings: {}", e));
                }
            }
        }
    };
    
    // Load settings
    let settings = settings_manager.get_settings();
    
    // Get saved theme from COMMON appearance settings (shared with Marco)
    let saved_theme = settings.appearance
        .as_ref()
        .and_then(|a| a.preview_theme.clone())
        .unwrap_or_else(|| "marco.css".to_string());
    
    log::debug!("Using theme from settings: {}", saved_theme);
    
    // Get saved window size from POLO-specific settings
    let (window_width, window_height) = if let Some(polo) = &settings.polo {
        if let Some(polo_window) = &polo.window {
            polo_window.get_window_size()
        } else {
            (1000, 800)  // Default for Polo
        }
    } else {
        (1000, 800)  // Default for Polo
    };
    
    log::debug!("Using window size: {}x{}", window_width, window_height);
    
    // Load CSS styling
    let asset_root = polo_paths.asset_root();
    load_css_from_path(asset_root);
    
    // Apply GTK dark mode preference based on settings
    apply_gtk_theme_preference(&settings_manager);
    
    // Get filename for titlebar
    let filename = file_path.as_ref().and_then(|p| {
        PathBuf::from(p).file_name().map(|n| n.to_string_lossy().to_string())
    });
    
    // Create shared reference to current file path (for theme switching and file opening)
    // Uses RwLock for interior mutability in GTK callbacks. Since GTK runs in a single-threaded
    // event loop, lock poisoning is extremely unlikely. All lock accesses gracefully handle
    // poisoning by using if-let-Ok patterns, treating it as a safe no-op rather than panicking.
    let current_file_path: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(file_path.clone()));
    
    // Set window title based on whether a file is opened
    let window_title = match filename.as_ref() {
        Some(name) => format!("Polo - {}", name),
        None => "Polo".to_string(),
    };
    
    // Create and show window
    let window = ApplicationWindow::builder()
        .application(app)
        .title(window_title)
        .default_width(window_width as i32)
        .default_height(window_height as i32)
        .build();
    window.add_css_class("polo-window");
    
    // Add theme-specific CSS class based on current mode
    let current_theme_mode = {
        let settings = settings_manager.get_settings();
        let editor_mode = settings
            .appearance
            .as_ref()
            .and_then(|a| a.editor_mode.as_ref())
            .map(|m| m.as_str())
            .unwrap_or("light");
        if editor_mode.contains("dark") { "dark" } else { "light" }
    };
    window.add_css_class(&format!("marco-theme-{}", current_theme_mode));
    log::debug!("Applied theme class: marco-theme-{}", current_theme_mode);
    
    // Create WebView for markdown preview
    let webview = webkit6::WebView::new();
    webview.set_vexpand(true);
    webview.set_hexpand(true);
    
    // Set background color to prevent white flash during loading
    // Use dark background matching the theme
    if let Some(rgba) = parse_hex_to_rgba("#1e1e1e") {
        webview.set_background_color(&rgba);
    }
    
    // Configure WebKit security settings to allow local file access
    // This is essential for loading images and other resources from the file system
    if let Some(settings) = webkit6::prelude::WebViewExt::settings(&webview) {
        settings.set_allow_file_access_from_file_urls(true);
        settings.set_allow_universal_access_from_file_urls(true);
        settings.set_auto_load_images(true);
        settings.set_enable_developer_extras(false); // Disable dev tools in viewer
        settings.set_javascript_can_access_clipboard(false); // Security: disable clipboard access
        settings.set_enable_write_console_messages_to_stdout(false); // Reduce noise in logs
    }
    
    // Load and render the markdown file
    let file_path_for_render = file_path.clone();
    let asset_root_for_render = polo_paths.asset_root();
    if let Some(ref path) = file_path_for_render {
        load_and_render_markdown(&webview, path, &saved_theme, &settings_manager, asset_root_for_render);
    } else {
        // Show empty state with theme awareness
        show_empty_state_with_theme(&webview, &settings_manager);
    }
    
    // Create custom titlebar (needs webview and file_path for theme switching)
    let asset_root = polo_paths.asset_root();
    let (titlebar_handle, _open_editor_btn, _title_label) = create_custom_titlebar(
        &window, 
        filename.as_deref().unwrap_or("Untitled"),
        &saved_theme, 
        settings_manager.clone(),
        webview.clone(),
        current_file_path.clone(),
        asset_root,
    );
    window.set_titlebar(Some(&titlebar_handle));
    
    window.set_child(Some(&webview));
    
    // Save window size changes to Polo-specific settings
    let settings_manager_width = settings_manager.clone();
    window.connect_default_width_notify(move |w| {
        let width = w.default_width() as u32;
        let height = w.default_height() as u32;
        
        let _ = settings_manager_width.update_settings(|s| {
            // Ensure polo section exists
            if s.polo.is_none() {
                s.polo = Some(core::logic::swanson::PoloSettings::default());
            }
            // Ensure polo.window exists
            if let Some(ref mut polo) = s.polo {
                if polo.window.is_none() {
                    polo.window = Some(core::logic::swanson::PoloWindowSettings::default());
                }
                if let Some(ref mut win) = polo.window {
                    win.width = Some(width);
                    win.height = Some(height);
                }
            }
        });
        log::debug!("Saved Polo window width: {}", width);
    });
    
    let settings_manager_height = settings_manager.clone();
    window.connect_default_height_notify(move |w| {
        let width = w.default_width() as u32;
        let height = w.default_height() as u32;
        
        let _ = settings_manager_height.update_settings(|s| {
            if s.polo.is_none() {
                s.polo = Some(core::logic::swanson::PoloSettings::default());
            }
            if let Some(ref mut polo) = s.polo {
                if polo.window.is_none() {
                    polo.window = Some(core::logic::swanson::PoloWindowSettings::default());
                }
                if let Some(ref mut win) = polo.window {
                    win.width = Some(width);
                    win.height = Some(height);
                }
            }
        });
        log::debug!("Saved Polo window height: {}", height);
    });
    
    // Save maximized state
    let settings_manager_max = settings_manager.clone();
    window.connect_maximized_notify(move |w| {
        let is_maximized = w.is_maximized();
        
        let _ = settings_manager_max.update_settings(|s| {
            if s.polo.is_none() {
                s.polo = Some(core::logic::swanson::PoloSettings::default());
            }
            if let Some(ref mut polo) = s.polo {
                if polo.window.is_none() {
                    polo.window = Some(core::logic::swanson::PoloWindowSettings::default());
                }
                if let Some(ref mut win) = polo.window {
                    win.maximized = Some(is_maximized);
                }
            }
        });
        log::debug!("Saved Polo maximized state: {}", is_maximized);
    });
    
    // Apply saved maximized state
    if let Some(polo) = &settings.polo {
        if let Some(polo_window) = &polo.window {
            if polo_window.is_maximized() {
                window.maximize();
            }
        }
    }
    
    // Present window
    window.present();
}
