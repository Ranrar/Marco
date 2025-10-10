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
//! - **Fast Rendering**: Uses marco_core's cached parser for instant previews
//! - **Minimal Dependencies**: No SourceView5, just GTK4 + WebKit6
//!
//! ## Architecture
//!
//! Polo follows Marco's architectural patterns:
//! - **main.rs**: Application gateway only - no business logic
//! - **components/**: All UI components and logic organized by function
//! - **marco_core**: Shared parsing, rendering, and settings management
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

use components::css::load_css;
use components::menu::create_custom_titlebar;
use components::utils::{apply_gtk_theme_preference, parse_hex_to_rgba};
use components::viewer::{load_and_render_markdown, show_empty_state_with_theme};
use gtk4::{gio, glib, prelude::*, Application, ApplicationWindow};
use webkit6::prelude::WebViewExt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

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
    marco_core::logic::logger::shutdown_file_logger();
    std::process::exit(1);
}

fn main() -> glib::ExitCode {
    // Initialize logger early
    if let Err(e) = marco_core::logic::logger::init_file_logger(true, log::LevelFilter::Debug) {
        // Logger not available yet, must use eprintln
        eprintln!("Failed to initialize logger: {}", e);
    }

    // Setup font directory for IcoMoon icon font (MUST be done before GTK init)
    use marco_core::logic::paths::{get_asset_dir_checked, get_font_path};
    let asset_dir = match get_asset_dir_checked() {
        Ok(asset_dir) => asset_dir,
        Err(e) => {
            fatal_error(&format!("Cannot locate asset directory: {}", e));
        }
    };
    
    // Set local font dir for Fontconfig/Pango to find ui_menu.ttf
    // Use to_string_lossy() to handle potential non-UTF-8 paths gracefully.
    // On Linux, paths can contain arbitrary bytes, but Fontconfig needs a string.
    // The lossy conversion will replace invalid UTF-8 sequences with � (U+FFFD),
    // which is acceptable since such paths are extremely rare and the font system
    // will simply fail to find the font (non-fatal) rather than crashing.
    let asset_dir_str = asset_dir.to_string_lossy();
    marco_core::logic::loaders::icon_loader::set_local_font_dir(&asset_dir_str);
    
    // Verify font is accessible
    match get_font_path("ui_menu.ttf") {
        Ok(font_path) => {
            log::debug!("Icon font loaded: {}", font_path.display());
        }
        Err(e) => {
            log::warn!("Font loading warning: {}", e);
            log::warn!("Icon font may not display correctly");
        }
    }

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN | gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    // Handle command-line arguments
    app.connect_command_line(|app, cmd_line| {
        let args: Vec<String> = cmd_line.arguments().iter().map(|s| s.to_string_lossy().to_string()).collect();
        
        // Parse arguments for IPC mode
        let mut session_key: Option<String> = None;
        let mut socket_name: Option<String> = None;
        let mut file_to_open: Option<String> = None;
        let mut i = 1;
        
        while i < args.len() {
            let arg = &args[i];
            
            if arg == "--help" || arg == "-h" {
                println!("Polo - Lightweight Markdown Viewer");
                println!("\nUsage:");
                println!("  polo <file.md>                              Open markdown file");
                println!("  polo --session <key> --socket <name> [file] IPC mode (Marco integration)");
                println!("  polo --debug <file.md>                      Open with debug logging");
                println!("  polo --help                                 Show this help message");
                return 0.into();
            } else if arg == "--session" {
                // IPC session key
                if i + 1 < args.len() {
                    session_key = Some(args[i + 1].clone());
                    log::info!("IPC session key provided: [REDACTED]");
                    i += 1; // Skip the key argument
                } else {
                    log::error!("--session requires a key argument");
                    return 1.into();
                }
            } else if arg == "--socket" {
                // IPC socket name
                if i + 1 < args.len() {
                    socket_name = Some(args[i + 1].clone());
                    log::info!("IPC socket provided: {}", args[i + 1]);
                    i += 1; // Skip the socket argument
                } else {
                    log::error!("--socket requires a name argument");
                    return 1.into();
                }
            } else if arg == "--debug" {
                // Debug flag already handled by logger init
            } else if arg.ends_with(".md") || arg.ends_with(".markdown") || !arg.starts_with('-') {
                // Found file to open
                file_to_open = Some(arg.clone());
            }
            
            i += 1;
        }
        
        // Build UI with IPC mode if both session and socket are provided
        let ipc_mode = session_key.is_some() && socket_name.is_some();
        if ipc_mode {
            log::info!("Running in IPC mode - Marco integration active");
        }
        
        build_ui_with_mode(app, file_to_open, ipc_mode, session_key, socket_name);
        0.into()
    });

    // Handle file opening via file manager (drag & drop, right-click)
    app.connect_open(|app, files, _hint| {
        if let Some(file) = files.first() {
            if let Some(path) = file.path() {
                build_ui_with_mode(app, Some(path.to_string_lossy().to_string()), false, None, None);
            }
        }
    });

    let exit_code = app.run();
    
    // Cleanup
    marco_core::logic::logger::shutdown_file_logger();
    exit_code
}

fn build_ui_with_mode(
    app: &Application,
    file_path: Option<String>,
    ipc_mode: bool,
    session_key: Option<String>,
    socket_name: Option<String>,
) {
    // Store Compass connection if in IPC mode (keep connection alive)
    let compass_connection: Option<Arc<Mutex<components::compass::Compass>>> = if ipc_mode {
        if let (Some(key), Some(socket)) = (session_key.clone(), socket_name.clone()) {
            log::info!("Initializing Compass IPC client");
            use components::compass::Compass;
            
            match Compass::connect(&socket, key) {
                Ok(mut compass) => {
                    log::info!("Connected to Marco via IPC");
                    
                    // Fetch session data from Marco
                    match compass.fetch_session() {
                        Ok(session_info) => {
                            log::info!("Session data received - theme: {}, editor_theme: {}",
                                      session_info.theme, session_info.editor_theme);
                            // TODO: Use session data to configure Polo
                        }
                        Err(e) => {
                            log::error!("Failed to fetch session data: {}", e);
                        }
                    }
                    
                    // Start command listener in background thread
                    // TODO: Wire these callbacks to actual UI updates
                    match compass.listen_for_commands(
                        |html, scroll_pos| {
                            log::info!("Received RefreshContent ({} bytes, scroll: {:?})", html.len(), scroll_pos);
                            // TODO: Update WebKit view with new HTML
                        },
                        |theme, editor_theme| {
                            log::info!("Received UpdateTheme (theme: {}, editor: {})", theme, editor_theme);
                            // TODO: Update theme
                        },
                        |position| {
                            log::info!("Received ScrollTo (pos: {})", position);
                            // TODO: Scroll WebKit view
                        },
                        || {
                            log::info!("Received Shutdown command");
                            // TODO: Close Polo gracefully
                        },
                    ) {
                        Ok(handle) => {
                            log::info!("Command listener thread started");
                            // Store handle if we need to join later
                            std::mem::forget(handle); // Let it run until process exits
                        }
                        Err(e) => {
                            log::error!("Failed to start command listener: {}", e);
                        }
                    }
                    
                    // Keep compass alive in Arc<Mutex<>> for sharing across threads
                    Some(Arc::new(Mutex::new(compass)))
                }
                Err(e) => {
                    log::error!("Failed to connect to Marco: {}", e);
                    None
                }
            }
        } else {
            log::error!("IPC mode requires both session key and socket name");
            None
        }
    } else {
        None
    };
    
    // Initialize settings manager early
    let settings_path = match marco_core::logic::paths::get_settings_path() {
        Ok(path) => path,
        Err(e) => {
            fatal_error(&format!("Cannot determine settings location: {}", e));
        }
    };
    
    let settings_manager = match marco_core::logic::swanson::SettingsManager::initialize(settings_path.clone()) {
        Ok(manager) => {
            log::debug!("Settings loaded successfully");
            manager
        },
        Err(e) => {
            log::warn!("Failed to load settings, using defaults: {}", e);
            // Create default settings and continue
            match marco_core::logic::swanson::SettingsManager::initialize(settings_path) {
                Ok(manager) => manager,
                Err(e) => {
                    fatal_error(&format!("Cannot initialize settings: {}", e));
                }
            }
        }
    };
    
    // Log IPC mode status
    if ipc_mode {
        log::info!("Running in IPC mode - Marco integration active");
    }
    
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
    load_css();
    
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
    if let Some(ref path) = file_path_for_render {
        load_and_render_markdown(&webview, path, &saved_theme, &settings_manager);
    } else {
        // Show empty state with theme awareness and IPC mode flag
        show_empty_state_with_theme(&webview, &settings_manager, ipc_mode);
    }
    
    // Create custom titlebar (needs webview and file_path for theme switching)
    let (titlebar_handle, _open_editor_btn) = create_custom_titlebar(
        &window, 
        filename.as_deref().unwrap_or("Untitled"),
        &saved_theme, 
        settings_manager.clone(),
        webview.clone(),
        current_file_path.clone(),
        ipc_mode,
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
                s.polo = Some(marco_core::logic::swanson::PoloSettings::default());
            }
            // Ensure polo.window exists
            if let Some(ref mut polo) = s.polo {
                if polo.window.is_none() {
                    polo.window = Some(marco_core::logic::swanson::PoloWindowSettings::default());
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
                s.polo = Some(marco_core::logic::swanson::PoloSettings::default());
            }
            if let Some(ref mut polo) = s.polo {
                if polo.window.is_none() {
                    polo.window = Some(marco_core::logic::swanson::PoloWindowSettings::default());
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
                s.polo = Some(marco_core::logic::swanson::PoloSettings::default());
            }
            if let Some(ref mut polo) = s.polo {
                if polo.window.is_none() {
                    polo.window = Some(marco_core::logic::swanson::PoloWindowSettings::default());
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
