use webkit6::prelude::*;
mod components;
mod footer;
mod logic;
mod menu;
mod settings {}
mod theme;
mod toolbar;
pub mod ui;

/*
╔═══════════════════════════════════════════════════════════════════════════╗
║    CRITICAL: This file (main.rs) serves ONLY as an APPLICATION GATEWAY    ║
╚═══════════════════════════════════════════════════════════════════════════╝
*/

use crate::components::editor::editor_ui::{create_editor_with_preview_and_buffer};
use crate::components::editor::footer_updates::wire_footer_updates;
use crate::components::viewer::viewmode::ViewMode;
use crate::theme::ThemeManager;
use gtk4::{glib, Application, ApplicationWindow, Box as GtkBox, Orientation};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
// MarkdownSyntaxMap compatibility removed; footer uses AST parser directly
use crate::logic::menu_items::file::FileOperations;
use crate::logic::{DocumentBuffer, RecentFiles};
use crate::ui::menu_items::files::FileDialogs;
use log::trace;

const APP_ID: &str = "com.example.Marco";

fn main() -> glib::ExitCode {
    // Very early audit: record entering main (before initialization)
    log::trace!("audit: main() entry - very early");

    // Install panic hook to ensure panics are logged and logger is flushed
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Attempt to log panic info (may be no logger yet)
        let panic_msg = match info.payload().downcast_ref::<&str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => s.as_str(),
                None => "Unknown panic payload",
            },
        };
        let location = if let Some(location) = info.location() {
            format!("{}:{}", location.file(), location.line())
        } else {
            "unknown:0".to_string()
        };
        log::error!("PANIC at {}: {}", location, panic_msg);
        // Try to flush and shutdown the file logger cleanly
        crate::logic::logger::shutdown_file_logger();
        // Call the default hook so we preserve existing behavior (printing to stderr)
        default_panic(info);
    }));

    // path detection and environment setup
    use crate::logic::paths::{get_asset_dir_checked, get_font_path, get_settings_path};
    let asset_dir = match get_asset_dir_checked() {
        Ok(asset_dir) => asset_dir,
        Err(e) => {
            eprintln!("Error detecting asset directory: {}", e);
            std::process::exit(1);
        }
    };
    // Set local font dir for Fontconfig/Pango
    crate::logic::loaders::icon_loader::set_local_font_dir(asset_dir.to_str().unwrap());

    // Example: Load font and settings paths
    match get_font_path("ui_menu.ttf") {
        Ok(_font_path) => {}
        Err(e) => eprintln!("Font error: {}", e),
    }
    match get_settings_path() {
        Ok(_settings_path) => {}
        Err(e) => eprintln!("Settings error: {}", e),
    }

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gtk4::gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    // Handle file opening via command line or file manager
    app.connect_open(|app, files, _hint| {
        let file_path = if !files.is_empty() {
            Some(files[0].path().unwrap().to_string_lossy().to_string())
        } else {
            None
        };
        build_ui(app, file_path);
    });

    // Handle normal activation (no files)
    app.connect_activate(|app| {
        build_ui(app, None);
    });

    trace!("audit: app starting");
    let exit_code = app.run();
    trace!("audit: app exiting with code {:?}", exit_code);
    
    // Clean up global resources before shutting down logger
    crate::components::editor::editor_manager::shutdown_editor_manager();
    crate::components::marco_engine::parser_cache::shutdown_global_parser_cache();
    crate::logic::cache::shutdown_global_cache();
    
    // Ensure file logger is flushed and closed on normal exit
    crate::logic::logger::shutdown_file_logger();
    exit_code
}

fn build_ui(app: &Application, initial_file: Option<String>) {
    // Import path functions
    use crate::logic::paths::{get_asset_dir_checked, get_settings_path, get_ui_theme_path};
    use crate::logic::swanson::SettingsManager;
    
    // Load and apply menu.css for menu and titlebar styling
    use gtk4 as gtk;
    use gtk4::gdk::Display;
    use gtk4::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};
    let css_provider = CssProvider::new();
    
    // Use dynamic path resolution instead of hardcoded relative paths
    match get_ui_theme_path("menu.css") {
        Ok(menu_css_path) => {
            css_provider.load_from_path(menu_css_path.to_str().unwrap_or(""));
            if let Some(display) = Display::default() {
                gtk::style_context_add_provider_for_display(
                    &display,
                    &css_provider,
                    STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
            }
        }
        Err(e) => {
            eprintln!("Warning: Failed to load menu.css: {}", e);
        }
    }

    // Load and apply toolbar.css for toolbar styling
    let toolbar_css_provider = CssProvider::new();
    match get_ui_theme_path("toolbar.css") {
        Ok(toolbar_css_path) => {
            toolbar_css_provider.load_from_path(toolbar_css_path.to_str().unwrap_or(""));
            if let Some(display) = Display::default() {
                gtk::style_context_add_provider_for_display(
                    &display,
                    &toolbar_css_provider,
                    STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
            }
        }
        Err(e) => {
            eprintln!("Warning: Failed to load toolbar.css: {}", e);
        }
    }
    
    // Create the main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Marco")
        .default_width(1200)
        .default_height(800)
        .build();
    window.add_css_class("main-window");

    // --- Custom VS Code-like draggable titlebar from menu.rs ---
    let (titlebar_handle, title_label, recent_menu) = menu::create_custom_titlebar(&window);
    window.set_titlebar(Some(&titlebar_handle));

    // --- ThemeManager and settings.ron path ---
    let settings_path = get_settings_path().unwrap_or_else(|_| {
        // Fallback to development path if asset detection fails
        let config_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        config_dir.join("src/assets/settings.ron")
    });
    let asset_dir = get_asset_dir_checked().unwrap_or_else(|_| {
        // Fallback to development paths if asset detection fails
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    });
    
    let dev_ui_theme_dir = asset_dir.join("themes/gtk4");
    let prod_ui_theme_dir = asset_dir.join("themes/ui");
    let ui_theme_dir = if dev_ui_theme_dir.exists() {
        dev_ui_theme_dir
    } else {
        prod_ui_theme_dir
    };

    // Use src/assets/themes/html_viever for preview themes in dev, /themes/ in prod
    let dev_preview_theme_dir = asset_dir.join("themes/html_viever");
    let prod_preview_theme_dir = asset_dir.join("themes");
    let preview_theme_dir = if dev_preview_theme_dir.exists() {
        dev_preview_theme_dir
    } else {
        prod_preview_theme_dir
    };

    // Use src/assets/themes/editor for editor style schemes in dev, /themes/editor in prod
    let dev_editor_theme_dir = asset_dir.join("themes/editor");
    let prod_editor_theme_dir = asset_dir.join("themes/editor");
    let editor_theme_dir = if dev_editor_theme_dir.exists() {
        dev_editor_theme_dir
    } else {
        prod_editor_theme_dir
    };

    // Initialize centralized settings manager - single source of truth for all settings
    let settings_manager = match SettingsManager::initialize(settings_path.clone()) {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!("Failed to initialize settings manager: {}", e);
            eprintln!("Using default settings and continuing...");
            // Create a fallback settings manager with default settings  
            match SettingsManager::initialize(settings_path.clone()) {
                Ok(manager) => manager,
                Err(_) => {
                    eprintln!("Critical: Cannot initialize settings. Exiting.");
                    std::process::exit(1);
                }
            }
        }
    };

    // Initialize file logger according to settings (runtime)
    {
        let app_settings = settings_manager.get_settings();

        // Enable logging if RUST_LOG environment variable is set or if configured in settings
        let rust_log_set = std::env::var("RUST_LOG").is_ok();
        let enabled = app_settings.log_to_file.unwrap_or(false) || rust_log_set;

        // Use Trace level to capture ALL log messages in the codebase
        let level = log::LevelFilter::Trace;

        if let Err(e) = crate::logic::logger::init_file_logger(enabled, level) {
            eprintln!("Failed to initialize file logger: {}", e);
        } else if enabled {
            log::info!("Logger initialized with level: {:?}, RUST_LOG set: {}", level, rust_log_set);
            log::debug!("Debug logging is working");
            log::trace!("Trace logging is working");
        }

        if rust_log_set || enabled {
            println!("Logging enabled (level: {:?}), check log files in ./log/ directory", level);
        }
    }

    // Initialize monospace font cache for fast settings loading
    if let Err(e) = crate::logic::loaders::font_loader::FontLoader::init_monospace_cache() {
        log::warn!("Failed to initialize monospace font cache: {}", e);
    }

    // Initialize the global editor manager with settings manager
    if let Err(e) = crate::components::editor::editor_manager::init_editor_manager(
        settings_manager.clone(),
    ) {
        log::warn!("Failed to initialize editor manager: {}", e);
    }

    // Initialize theme manager with settings manager
    let theme_manager = Rc::new(RefCell::new(ThemeManager::new(
        settings_manager.clone(),
        ui_theme_dir,
        preview_theme_dir.clone(),
        editor_theme_dir,
    )));
    // Pass settings struct to modules as needed

    // Create main vertical box layout
    let main_box = GtkBox::new(Orientation::Vertical, 0);
    main_box.add_css_class("main-container");

    // Create basic UI components (structure only)
    let toolbar = toolbar::create_toolbar_structure();
    toolbar.add_css_class("toolbar");
    toolbar::set_toolbar_height(&toolbar, 0); // Minimum height, matches footer
                                              // --- Determine correct HTML preview theme based on settings and app theme ---
    use crate::logic::loaders::theme_loader::list_html_view_themes;
    let preview_theme_dir_str = preview_theme_dir.clone().to_string_lossy().to_string();
    let html_themes = list_html_view_themes(&preview_theme_dir.clone());
    let settings = theme_manager.borrow().get_settings();
    let mut preview_theme_filename = "standard.css".to_string();
    if let Some(appearance) = &settings.appearance {
        if let Some(ref preview_theme) = appearance.preview_theme {
            if html_themes.iter().any(|t| &t.filename == preview_theme) {
                preview_theme_filename = preview_theme.clone();
            }
        }
    }
    // Initialize theme_mode based on current editor scheme setting
    let initial_theme_mode = {
        let current_scheme = theme_manager.borrow().current_editor_scheme_id();
        theme_manager
            .borrow()
            .preview_theme_mode_from_scheme(&current_scheme)
    };
    let theme_mode = Rc::new(RefCell::new(initial_theme_mode));
    let (footer, footer_labels_rc) = footer::create_footer();

    // Create file operations handler early so we can pass DocumentBuffer to editor
    let file_operations = FileOperations::new(
        Rc::new(RefCell::new(DocumentBuffer::new_untitled())),
        Rc::new(RefCell::new(RecentFiles::new(settings_manager.clone()))),
    );
    let file_operations_rc = Rc::new(RefCell::new(file_operations));
    let document_buffer_ref = Rc::clone(&file_operations_rc.borrow().buffer);

    // Active markdown schema support removed; footer uses AST parser directly.
    let _schema_root = asset_dir.join("markdown_schema");
    let active_schema_map: Rc<RefCell<Option<()>>> = Rc::new(RefCell::new(None));

    let (
        split,
        editor_webview,
        preview_css_rc,
        refresh_preview,
        update_editor_theme,
        update_preview_theme,
        editor_buffer,
        editor_source_view,
        insert_mode_state,
        set_view_mode,
        split_overlay,
    ) = create_editor_with_preview_and_buffer(
        preview_theme_filename.as_str(),
        preview_theme_dir_str.as_str(),
        theme_manager.clone(),
        Rc::clone(&theme_mode),
        footer_labels_rc.clone(),
        settings_path.to_str().unwrap(),
        Some(document_buffer_ref),
    );

    // Wrap setter into Rc so it can be cloned into action callbacks
    let set_view_mode_rc: Rc<Box<dyn Fn(ViewMode)>> = Rc::new(set_view_mode);

    // Wire up live footer updates using the actual editor buffer
    // Wire footer updates directly: wire_footer_updates will run callbacks on
    // the main loop and call `apply_footer_update` directly.
    wire_footer_updates(
        &editor_buffer,
        footer_labels_rc.clone(),
        insert_mode_state.clone(),
    );
    split_overlay.add_css_class("split-view");  // Apply CSS to overlay

    // --- Settings Thread Pool for Proper Resource Management ---
    // Create early so it's available for split ratio saving
    let (settings_tx, settings_rx) = std::sync::mpsc::channel::<Box<dyn FnOnce() + Send>>();
    let settings_thread_handle = std::thread::spawn(move || {
        // Single background thread that processes all settings operations sequentially
        // This prevents race conditions and ensures proper resource cleanup
        while let Ok(task) = settings_rx.recv() {
            task();
        }
        log::debug!("Settings thread pool shutting down");
    });

    // Store the thread handle and sender for cleanup
    let settings_thread_data = std::rc::Rc::new(std::cell::RefCell::new((
        Some(settings_thread_handle),
        settings_tx.clone(),
    )));

    // Apply saved split ratio after paned widget is mapped and sized
    // Use map signal with multiple retry attempts via timeout
    {
        let settings_manager_clone = settings_manager.clone();
        let split_for_init = split.clone();
        let applied = Rc::new(RefCell::new(false));
        
        split_for_init.connect_map(move |paned| {
            let paned_clone = paned.clone();
            let settings_manager = settings_manager_clone.clone();
            let applied_clone = applied.clone();
            let attempt_counter = Rc::new(RefCell::new(0));
            
            // Retry with timeout until widget has allocated width
            glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
                if *applied_clone.borrow() {
                    return glib::ControlFlow::Break;
                }
                
                let paned_width = paned_clone.allocated_width();
                
                if paned_width > 0 {
                    // Successfully got width, apply saved ratio
                    *applied_clone.borrow_mut() = true;
                    
                    let settings = settings_manager.get_settings();
                    if let Some(window_settings) = settings.window {
                        let split_ratio = window_settings.get_split_ratio();
                        let position = (paned_width as f64 * split_ratio as f64 / 100.0) as i32;
                        
                        log::info!("[SPLIT INIT] Applying saved ratio: {}% -> {}px (width: {}px)", split_ratio, position, paned_width);
                        paned_clone.set_position(position);
                    }
                    return glib::ControlFlow::Break;
                }
                
                let mut attempt = attempt_counter.borrow_mut();
                *attempt += 1;
                if *attempt >= 20 {
                    // Give up after 1 second (20 * 50ms)
                    log::warn!("[SPLIT INIT] Failed to get paned width after {} attempts, giving up", *attempt);
                    *applied_clone.borrow_mut() = true;
                    return glib::ControlFlow::Break;
                }
                
                glib::ControlFlow::Continue
            });
        });
    }

    // Save split ratio when user finishes manually dragging the divider
    // Track position changes and save after drag completes (no changes for 200ms)
    {
        let settings_manager_clone = settings_manager.clone();
        let split_for_save = split.clone();
        let settings_tx_clone = settings_tx.clone();
        let last_position = Rc::new(RefCell::new(-1i32));
        let save_timeout: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
        
        split_for_save.connect_notify_local(Some("position"), move |paned, _| {
            let paned_width = paned.allocated_width();
            if paned_width <= 0 {
                return;
            }
            
            let position = paned.position();
            
            // Check if position actually changed
            if *last_position.borrow() == position {
                return;
            }
            *last_position.borrow_mut() = position;
            
            // Cancel any pending save
            if let Some(id) = save_timeout.borrow_mut().take() {
                id.remove();
            }
            
            // Schedule save after 200ms of no changes (drag completed)
            let settings_manager = settings_manager_clone.clone();
            let settings_tx = settings_tx_clone.clone();
            let save_timeout_clone = save_timeout.clone();
            
            let timeout_id = glib::timeout_add_local_once(
                std::time::Duration::from_millis(200),
                move || {
                    *save_timeout_clone.borrow_mut() = None;
                    
                    let ratio = ((position as f64 / paned_width as f64) * 100.0).round() as i32;
                    let ratio = ratio.clamp(10, 90);
                    
                    let task = Box::new(move || {
                        if let Err(e) = settings_manager.update_settings(|s| {
                            let _ = s.update_window_settings(|ws| {
                                ws.split_ratio = Some(ratio);
                            });
                        }) {
                            log::error!("Failed to save split ratio: {}", e);
                        } else {
                            log::info!("[SPLIT SAVE] Drag complete: {}% ({}px / {}px)", ratio, position, paned_width);
                        }
                    });
                    
                    if let Err(e) = settings_tx.send(task) {
                        log::error!("Failed to queue split ratio save task: {}", e);
                    }
                }
            );
            
            *save_timeout.borrow_mut() = Some(timeout_id);
        });
    }

    // Apply saved view mode from settings at startup (if present)
    {
        let s = settings_manager.get_settings();
        if let Some(layout) = s.layout {
            if let Some(vm) = layout.view_mode {
                match vm.as_str() {
                    "HTML Preview" => (set_view_mode_rc)(ViewMode::HtmlPreview),
                    "Source Code" | "Code Preview" => (set_view_mode_rc)(ViewMode::CodePreview),
                    _ => {}
                }
            }
        }
    }

    // Create footer update function using weak references to prevent circular retention
    let trigger_footer_update: std::rc::Rc<dyn Fn()> = {
        // Use weak references to editor components
        let buffer_weak = editor_buffer.downgrade();
        let labels_weak = Rc::downgrade(&footer_labels_rc);
        let test_counter = std::rc::Rc::new(std::cell::Cell::new(0));
        
        std::rc::Rc::new(move || {
            // Check if components are still valid before using
            if let (Some(buffer), Some(labels)) = (
                buffer_weak.upgrade(),
                labels_weak.upgrade(),
            ) {
                // Manual footer trigger invoked; terminal output suppressed.

                // Increment test counter for obvious visual changes
                let count = test_counter.get() + 1;
                test_counter.set(count);

                // Update with test values to make changes obvious
                crate::footer::update_cursor_row(&labels, count + 10);
                crate::footer::update_cursor_col(&labels, count + 20);
                crate::footer::update_word_count(&labels, count * 10);
                crate::footer::update_char_count(&labels, count * 50);
                crate::footer::update_encoding(&labels, &format!("TEST-{}", count));
                crate::footer::update_insert_mode(&labels, count.is_multiple_of(2));

                // Also do the original syntax trace logic
                let offset = buffer.cursor_position();
                let iter = buffer.iter_at_offset(offset);
                let current_line = iter.line();
                let start_iter_opt = buffer.iter_at_line(current_line);
                let end_iter_opt = buffer.iter_at_line(current_line + 1);
                let line_text = match (start_iter_opt, end_iter_opt) {
                    (Some(ref start), Some(ref end)) => buffer.text(start, end, false).to_string(),
                    (Some(ref start), None) => {
                        buffer.text(start, &buffer.end_iter(), false).to_string()
                    }
                    _ => String::new(),
                };
                // Footer uses AST-based parsing internally; pass only labels and line text
                crate::footer::update_syntax_trace(&labels, &line_text);
            } else {
                log::debug!("Footer update callback called after editor components were dropped");
            }
        })
    };

    // Add components to main layout (menu bar is now in titlebar)
    main_box.append(&toolbar);
    main_box.append(&split_overlay);  // Use overlay instead of split
    main_box.append(&footer);

    // Set editor area to expand
    split_overlay.set_vexpand(true);  // Use overlay instead of split

    // Ensure footer is visible and properly positioned
    footer.set_vexpand(false); // Footer should not expand vertically
    footer.set_hexpand(true); // Footer should expand horizontally
    footer.set_visible(true); // Explicitly ensure footer is visible

    // Add main box to window
    window.set_child(Some(&main_box));

    // --- Live HTML preview theme switching ---
    // Store refresh_preview closure for use on theme changes
    let refresh_preview_rc = Rc::new(RefCell::new(refresh_preview));
    // Register 'app.settings' action to show the settings dialog with the callback
    let settings_action = gtk4::gio::SimpleAction::new("settings", None);
    let update_editor_theme_rc = Rc::new(update_editor_theme);
    let update_preview_theme_rc = Rc::new(update_preview_theme);

    // Helper to persist view mode in settings.ron without blocking the UI
    // Uses the dedicated settings thread pool to avoid orphaned threads
    let save_view_mode = {
        let settings_manager = settings_manager.clone();
        let settings_tx = settings_tx.clone();
        Rc::new(move |mode: &str| {
            let settings_manager = settings_manager.clone();
            let mode_owned = mode.to_string();
            let task = Box::new(move || {
                use crate::logic::swanson::LayoutSettings;
                if let Err(e) = settings_manager.update_settings(|s| {
                    if s.layout.is_none() {
                        s.layout = Some(LayoutSettings::default());
                    }
                    if let Some(ref mut l) = s.layout {
                        l.view_mode = Some(mode_owned.clone());
                    }
                }) {
                    log::error!("Failed to save view mode settings: {}", e);
                } else {
                    log::debug!("View mode saved: {}", mode_owned);
                }
            });
            if let Err(e) = settings_tx.send(task) {
                log::error!("Failed to queue view mode save task: {}", e);
            }
        })
    };

    settings_action.connect_activate({
        // Clone directly from original sources to avoid intermediate reference chains
        let window = window.clone();
        let theme_manager = theme_manager.clone();
        let settings_path = settings_path.clone();
        let preview_css_rc = preview_css_rc.clone();
        let refresh_preview_rc = refresh_preview_rc.clone();
        let update_editor_theme_rc = update_editor_theme_rc.clone();
        let update_preview_theme_rc = update_preview_theme_rc.clone();
        let set_view_mode_rc = set_view_mode_rc.clone();
        let save_view_mode = save_view_mode.clone();
        move |_, _| {
            use crate::ui::settings::dialog::show_settings_dialog;

            // Create editor theme callback that updates both editor and preview
            let editor_callback = {
                let update_editor = update_editor_theme_rc.clone();
                let update_preview = update_preview_theme_rc.clone();
                Box::new(move |scheme_id: String| {
                    update_editor(&scheme_id);
                    update_preview(&scheme_id);
                }) as Box<dyn Fn(String) + 'static>
            };

            trace!("audit: opened settings dialog");
            // Build the callbacks struct for the settings dialog to keep the
            // callsite compact and satisfy the updated API.
            use crate::ui::settings::dialog::SettingsDialogCallbacks;

            let callbacks = SettingsDialogCallbacks {
                on_preview_theme_changed: Some(Box::new({
                    // Use weak references to prevent circular retention
                    let theme_manager_weak = Rc::downgrade(&theme_manager);
                    let preview_css_weak = Rc::downgrade(&preview_css_rc);
                    let refresh_preview_weak = Rc::downgrade(&refresh_preview_rc);
                    move |theme_filename: String| {
                        // Check if references are still valid before using
                        if let (Some(theme_manager), Some(preview_css_rc), Some(refresh_preview_rc)) = (
                            theme_manager_weak.upgrade(),
                            preview_css_weak.upgrade(),
                            refresh_preview_weak.upgrade(),
                        ) {
                            // On preview theme change, update CSS and call refresh
                            use std::fs;
                            let theme_manager = theme_manager.borrow();
                            let preview_theme_dir = theme_manager.preview_theme_dir.clone();
                            let css_path = preview_theme_dir.join(&theme_filename);
                            let css = fs::read_to_string(&css_path).unwrap_or_default();
                            *preview_css_rc.borrow_mut() = css;
                            (refresh_preview_rc.borrow())();
                        } else {
                            log::debug!("Preview theme callback called after main components were dropped");
                        }
                    }
                })),
                refresh_preview: Some(refresh_preview_rc.clone()),
                on_editor_theme_changed: Some(editor_callback),
                on_schema_changed: Some(Box::new({
                    // Use weak references to prevent circular retention
                    let active_schema_map_weak = Rc::downgrade(&active_schema_map);
                    let trigger_weak = Rc::downgrade(&trigger_footer_update);
                    move |_selected: Option<String>| {
                        // Check if references are still valid before using
                        if let (Some(active_schema_map), Some(trigger)) = (
                            active_schema_map_weak.upgrade(),
                            trigger_weak.upgrade(),
                        ) {
                            // Schema support removed; clear any existing schema and trigger footer update
                            *active_schema_map.borrow_mut() = None;
                            (trigger)();
                        } else {
                            log::debug!("Schema callback called after main components were dropped");
                        }
                    }
                })),
                // on_view_mode_changed: persist and forward to runtime setter
                on_view_mode_changed: Some(Box::new({
                    // Use weak reference to prevent circular retention
                    let set_view_mode_weak = Rc::downgrade(&set_view_mode_rc);
                    let save = save_view_mode.clone(); // This closure is self-contained, no circular ref risk
                    move |selected: String| {
                        // Persist the selection asynchronously (always works)
                        save(&selected);
                        
                        // Check if view mode setter is still valid before using
                        if let Some(set_view_mode_rc) = set_view_mode_weak.upgrade() {
                            match selected.as_str() {
                                "HTML Preview" => (set_view_mode_rc)(ViewMode::HtmlPreview),
                                "Source Code" | "Code Preview" => (set_view_mode_rc)(ViewMode::CodePreview),
                                _ => {}
                            }
                        } else {
                            log::debug!("View mode callback called after main components were dropped");
                        }
                    }
                }) as Box<dyn Fn(String) + 'static>),
                // on_split_ratio_changed: update the actual paned widget position in real-time
                on_split_ratio_changed: Some(Box::new({
                    // GTK widgets have their own reference counting, but use weak ref for consistency
                    let split_paned_weak = split.downgrade();
                    move |ratio: i32| {
                        log::debug!("[SPLIT LIVE] Callback received ratio: {}%", ratio);
                        // Check if widget is still valid before using
                        if let Some(split_paned) = split_paned_weak.upgrade() {
                            // Calculate the pixel position based on the current paned width
                            let paned_width = split_paned.allocated_width();
                            let new_position = if paned_width > 0 {
                                (paned_width as f64 * ratio as f64 / 100.0) as i32
                            } else {
                                // Fallback to default width calculation
                                (1200.0 * ratio as f64 / 100.0) as i32
                            };

                            split_paned.set_position(new_position);
                            log::debug!(
                                "[SPLIT LIVE] Applied ratio: {}% -> {}px (width: {}px)",
                                ratio,
                                new_position,
                                paned_width
                            );
                        } else {
                            log::debug!("[SPLIT LIVE] Split paned widget was dropped");
                        }
                    }
                }) as Box<dyn Fn(i32) + 'static>),
                // on_sync_scrolling_changed: enable/disable scroll synchronization
                on_sync_scrolling_changed: Some(Box::new({
                    move |enabled: bool| {
                        // Use the global scroll sync API to enable/disable synchronization
                        use crate::components::editor::editor_manager::set_scroll_sync_enabled_globally;
                        let _ = set_scroll_sync_enabled_globally(enabled);
                        log::debug!("Scroll sync toggled: {}", enabled);
                    }
                }) as Box<dyn Fn(bool) + 'static>),
                // on_line_numbers_changed: enable/disable line numbers in the editor
                on_line_numbers_changed: Some(Box::new({
                    move |enabled: bool| {
                        // Use the global line numbers API to update all editors
                        use crate::components::editor::editor_manager::update_line_numbers_globally;
                        let _ = update_line_numbers_globally(enabled);
                        log::debug!("Line numbers toggled: {}", enabled);
                    }
                }) as Box<dyn Fn(bool) + 'static>),
            };

            show_settings_dialog(
                window.upcast_ref(),
                theme_manager.clone(),
                settings_path.clone(),
                &asset_dir,
                callbacks,
            );
        }
    });
    app.add_action(&settings_action);

    // Register view mode actions to switch preview and persist setting
    let view_html_action = gtk4::gio::SimpleAction::new("view_html", None);
    view_html_action.connect_activate({
        let set_view_mode_rc = set_view_mode_rc.clone();
        let save_view_mode = save_view_mode.clone();
        move |_, _| {
            (set_view_mode_rc)(ViewMode::HtmlPreview);
            // Persist setting using the thread pool to avoid race conditions
            (save_view_mode)("HTML Preview");
        }
    });
    app.add_action(&view_html_action);

    let view_code_action = gtk4::gio::SimpleAction::new("view_code", None);
    view_code_action.connect_activate({
        let set_view_mode_rc = set_view_mode_rc.clone();
        let save_view_mode = save_view_mode.clone();
        move |_, _| {
            (set_view_mode_rc)(ViewMode::CodePreview);
            // Persist setting using the thread pool to avoid race conditions
            (save_view_mode)("Source Code");
        }
    });
    app.add_action(&view_code_action);

    // Register search & replace action
    let search_action = gtk4::gio::SimpleAction::new("search", None);
    search_action.connect_activate({
        let window = window.clone();
        let buffer = Rc::new(editor_buffer.clone());
        let source_view = Rc::new(editor_source_view.clone());
        let webview = Rc::new(editor_webview.clone());
        let cache = Rc::new(RefCell::new(crate::logic::cache::SimpleFileCache::new()));
        move |_, _| {
            use crate::logic::menu_items::search::show_search_window;
            show_search_window(window.upcast_ref(), cache.clone(), Rc::clone(&buffer), Rc::clone(&source_view), Rc::clone(&webview));
        }
    });
    app.add_action(&search_action);
    app.set_accels_for_action("app.search", &["<Control>f"]);

    // Populate the Recent Files submenu from FileOperations' recent list
    // If empty, leave the submenu with its placeholder (no entries) so it appears inactive.
    // Register remaining file actions (open, save_as, quit, recent-file handling)
    crate::logic::menu_items::file::register_file_actions_async(
        app.clone(),
        file_operations_rc.clone(),
        &window,
        &editor_buffer,
        &title_label,
        std::sync::Arc::new(|w, title| Box::pin(FileDialogs::show_open_dialog(w, title))),
        std::sync::Arc::new(|w, doc_name, action| {
            Box::pin(FileDialogs::show_save_changes_dialog(w, doc_name, action))
        }),
        std::sync::Arc::new(|w, title, suggested| {
            Box::pin(FileDialogs::show_save_dialog(w, title, suggested))
        }),
    );

    // Wire dynamic recent-file actions using the recent_menu from the UI
    crate::logic::menu_items::file::setup_recent_actions(
        app,
        file_operations_rc.clone(),
        &recent_menu,
        &window,
        &editor_buffer,
        &title_label,
        std::sync::Arc::new(|w, doc_name, action| {
            Box::pin(FileDialogs::show_save_changes_dialog(w, doc_name, action))
        }),
        std::sync::Arc::new(|w, title, suggested| {
            Box::pin(FileDialogs::show_save_dialog(w, title, suggested))
        }),
    );

    // Open initial file if provided via command line
    if let Some(file_path) = initial_file {
        crate::logic::menu_items::file::FileOperations::load_initial_file_async(
            file_operations_rc.clone(),
            file_path,
            window.clone(),
            editor_buffer.clone(),
            title_label.clone(),
            |w, doc_name, action| {
                Box::pin(FileDialogs::show_save_changes_dialog(w, doc_name, action))
            },
            |w, title, suggested| Box::pin(FileDialogs::show_save_dialog(w, title, suggested)),
        );
    }

    // Apply startup editor settings to ensure editor uses settings.ron values
    if let Err(e) = crate::components::editor::editor_manager::apply_startup_editor_settings() {
        log::warn!("Failed to apply startup editor settings: {}", e);
    }

    // Load and apply saved window state
    {
        let settings = settings_manager.get_settings();
        if let Some(window_settings) = settings.window {
            // Apply window size
            let (width, height) = window_settings.get_window_size();
            window.set_default_size(width as i32, height as i32);

            // Apply window position if saved
            if let Some((x, y)) = window_settings.get_window_position() {
                // Note: GTK4 doesn't support programmatic window positioning directly
                // This would need platform-specific implementation if required
                log::debug!(
                    "Would restore window position to ({}, {}) if supported",
                    x,
                    y
                );
            }

            // Apply maximized state
            if window_settings.is_maximized() {
                window.maximize();
            }
        }
    }

    // Connect window state change handlers to persist settings
    {
        let settings_manager_resize = settings_manager.clone();
        let settings_tx_resize = settings_tx.clone();
        window.connect_default_width_notify(move |w| {
            let settings_manager = settings_manager_resize.clone();
            let width = w.default_width();
            let height = w.default_height();
            let settings_tx = settings_tx_resize.clone();

            let task = Box::new(move || {
                if let Err(e) = settings_manager.update_settings(|s| {
                    let _ = s.update_window_settings(|ws| {
                        ws.width = Some(width as u32);
                        ws.height = Some(height as u32);
                    });
                }) {
                    log::error!("Failed to save window size: {}", e);
                } else {
                    log::debug!("Window size saved: {}x{}", width, height);
                }
            });
            if let Err(e) = settings_tx.send(task) {
                log::error!("Failed to queue window size save task: {}", e);
            }
        });

        let settings_manager_resize2 = settings_manager.clone();
        let settings_tx_resize2 = settings_tx.clone();
        window.connect_default_height_notify(move |w| {
            let settings_manager = settings_manager_resize2.clone();
            let width = w.default_width();
            let height = w.default_height();
            let settings_tx = settings_tx_resize2.clone();

            let task = Box::new(move || {
                if let Err(e) = settings_manager.update_settings(|s| {
                    let _ = s.update_window_settings(|ws| {
                        ws.width = Some(width as u32);
                        ws.height = Some(height as u32);
                    });
                }) {
                    log::error!("Failed to save window size: {}", e);
                } else {
                    log::debug!("Window size saved: {}x{}", width, height);
                }
            });
            if let Err(e) = settings_tx.send(task) {
                log::error!("Failed to queue window size save task: {}", e);
            }
        });

        let settings_manager_maximize = settings_manager.clone();
        let settings_tx_maximize = settings_tx.clone();
        window.connect_maximized_notify(move |w| {
            let settings_manager = settings_manager_maximize.clone();
            let is_maximized = w.is_maximized();
            let settings_tx = settings_tx_maximize.clone();

            let task = Box::new(move || {
                if let Err(e) = settings_manager.update_settings(|s| {
                    let _ = s.update_window_settings(|ws| {
                        ws.maximized = Some(is_maximized);
                    });
                }) {
                    log::error!("Failed to save window maximized state: {}", e);
                } else {
                    log::debug!("Window maximized state saved: {}", is_maximized);
                }
            });
            if let Err(e) = settings_tx.send(task) {
                log::error!("Failed to queue window maximized save task: {}", e);
            }
        });
    }

    // Connect to window destroy signal to clean up settings thread
    window.connect_destroy({
        let settings_thread_data = settings_thread_data.clone();
        move |_| {
            log::debug!("Window destroyed, cleaning up settings thread");
            let mut thread_data = settings_thread_data.borrow_mut();
            if let Some(handle) = thread_data.0.take() {
                // Drop all senders to signal the thread to exit
                // We need to drop the channel to close it and signal the thread to shutdown
                std::mem::drop(std::mem::replace(&mut thread_data.1, {
                    let (dummy_tx, _) = std::sync::mpsc::channel();
                    dummy_tx
                }));
                // Wait for the thread to finish (with timeout for safety)
                if let Err(e) = handle.join() {
                    log::error!("Failed to join settings thread: {:?}", e);
                } else {
                    log::debug!("Settings thread cleaned up successfully");
                }
            }
            
            // Clean up global resources
            crate::components::editor::editor_manager::shutdown_editor_manager();
            crate::components::marco_engine::parser_cache::shutdown_global_parser_cache();
            crate::logic::cache::shutdown_global_cache();
        }
    });

    // Present the window
    window.present();
}
