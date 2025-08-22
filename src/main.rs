use webkit6::prelude::*;
mod logic;
mod components;
// Stripped-down UI structure modules

mod footer;
mod menu;
mod settings {
    // pub use crate::ui::settings::*; // unused import removed
}
mod theme;
mod toolbar;
pub mod ui;

/*
╔══════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                    MARCO ARCHITECTURE GUIDELINES                                 ║
║                                                                                                  ║
║    CRITICAL: This file (main.rs) serves ONLY as an APPLICATION GATEWAY                           ║
║                                                                                                  ║
║    DO NOT ADD:                           ║    ALLOWED IN main.rs:                                ║
║     • Business logic                     ║     • Module imports and declarations                 ║
║     • UI component implementations       ║     • Application initialization                      ║
║     • File operations                    ║     • Window setup and basic layout                   ║
║     • Complex algorithms                 ║     • Async context bridging (spawn_local)            ║
║     • Data processing                    ║                                                       ║
║     • Theme logic                        ║                                                       ║
║     • Parser implementations             ║                                                       ║
║                                          ║                                                       ║
║                                                                                                  ║
║    FILE ORGANIZATION GUIDE:                                                                      ║
║                                                                                                  ║
║    src/logic/                          - All business logic and core functionality               ║
║     ├── buffer.rs                       - Document state management                              ║
║     ├── parser.rs                       - Markdown parsing and syntax analysis                   ║
║     ├── asset_path.rs                   - Asset detection and path resolution                    ║
║     ├── theme_loader.rs                 - Theme discovery and loading                            ║
║     ├── schema_loader.rs                - Schema loading and validation                          ║
║     ├── crossplatforms.rs               - Cross-platform compatibility                           ║
║     ├── swanson.rs                      - Settings management                                    ║
║     └── menu_items/                     - File operation business logic                          ║
║         ├── file.rs                     - File operations (open, save, etc.)                     ║
║         ├── edit.rs                     - Edit operations                                        ║
║         ├── format.rs                   - Format operations                                      ║
║         ├── view.rs                     - View state management                                  ║
║         └── help.rs                     - Help system logic                                      ║
║                                                                                                  ║
║    src/ui/                             - All user interface components                           ║
║     ├── main_editor.rs                  - Main editor UI with preview                            ║
║     ├── code_viewer.rs                  - Code editor component                                  ║
║     ├── html_viewer.rs                  - HTML preview component                                 ║
║     ├── splitview.rs                    - Split view management                                  ║
║     ├── menu_items/                     - UI dialogs and interactions                            ║
║     │   └── files.rs                    - File dialogs (FileChooserNative, etc.)                 ║
║     └── settings/                       - Settings UI components                                 ║
║         ├── settings.rs                 - Settings dialog                                        ║
║         └── tabs/                       - Settings tab components                                ║
║                                                                                                  ║
║    src/ (root level)                   - Application structure and integration                   ║
║     ├── main.rs                         - THIS FILE: Application gateway only                    ║
║     ├── lib.rs                          - Library exports for testing                            ║
║     ├── footer.rs                       - Footer component integration                           ║
║     ├── menu.rs                         - Menu bar and titlebar                                  ║
║     ├── toolbar.rs                      - Toolbar component                                      ║
║     └── theme.rs                        - Theme management integration                           ║
║                                                                                                  ║
║    INTEGRATION PATTERN:                                                                          ║
║     main.rs → UI layer → Logic layer                                                             ║
║     • main.rs sets up window and actions                                                         ║
║     • UI components handle user interactions                                                     ║
║     • Logic components process business rules                                                    ║
║     • Async bridging happens in main.rs via spawn_local                                          ║
║                                                                                                  ║
║    WHEN ADDING NEW FEATURES:                                                                     ║
║     1. Business logic → src/logic/                                                               ║
║     2. UI components → src/ui/                                                                   ║
║     3. Integration only → main.rs                                                                ║
║     4. Keep layers separate and well-defined                                                     ║
║                                                                                                  ║
║    PRINCIPLES:                                                                                   ║
║     • Separation of Concerns: Logic ≠ UI ≠ Integration                                           ║
║     • Single Responsibility: Each file has one clear purpose                                     ║
║     • Dependency Direction: main.rs → ui → logic (never reverse)                                 ║
║     • Clean Architecture: Outer layers depend on inner layers                                    ║
║                                                                                                  ║
╚══════════════════════════════════════════════════════════════════════════════════════════════════╝
*/

use gtk4::{glib, Application, ApplicationWindow, Box as GtkBox, Orientation};
use crate::ui::main_editor::{create_editor_with_preview, wire_footer_updates};
use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;
use crate::theme::ThemeManager;
use crate::components::marco_engine::parser::MarkdownSyntaxMap;
use crate::logic::{DocumentBuffer, RecentFiles};
use crate::logic::menu_items::file::FileOperations;
use crate::ui::menu_items::files::FileDialogs;

const APP_ID: &str = "com.example.Marco";

fn main() -> glib::ExitCode {
    // Asset path detection and environment setup
    use crate::logic::asset_path::{get_asset_dir_checked, get_font_path, get_settings_path};
    let asset_dir = match get_asset_dir_checked() {
        Ok(asset_dir) => {
            println!("Asset directory set: {}", asset_dir.display());
            asset_dir
        }
        Err(e) => {
            eprintln!("Error detecting asset directory: {}", e);
            std::process::exit(1);
        }
    };
    // Set local font dir for Fontconfig/Pango
    crate::logic::loaders::icon_loader::set_local_font_dir(asset_dir.to_str().unwrap());

    // Example: Load font and settings paths
    match get_font_path("ui_menu.ttf") {
        Ok(font_path) => println!("Font path: {}", font_path.display()),
        Err(e) => eprintln!("Font error: {}", e),
    }
    match get_settings_path() {
        Ok(settings_path) => println!("Settings path: {}", settings_path.display()),
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
    
    let exit_code = app.run();
    exit_code
}

fn build_ui(app: &Application, initial_file: Option<String>) {
    // Load and apply menu.css for menu and titlebar styling
    use gtk4::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};
    use gtk4 as gtk;
    use gtk4::gdk::Display;
    let css_provider = CssProvider::new();
    css_provider.load_from_path("src/assets/themes/ui_elements/menu.css");
    if let Some(display) = Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &css_provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    // Load and apply toolbar.css for toolbar styling
    let toolbar_css_provider = CssProvider::new();
    toolbar_css_provider.load_from_path("src/assets/themes/ui_elements/toolbar.css");
    if let Some(display) = Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &toolbar_css_provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    // Create the main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Marco")
        .default_width(1200)
        .default_height(800)
        .build();
    window.add_css_class("main-window");

    // --- Custom VS Code–like draggable titlebar from menu.rs ---
    let (titlebar_handle, title_label, recent_menu) = menu::create_custom_titlebar(&window);
    window.set_titlebar(Some(&titlebar_handle));

    // --- ThemeManager and settings.ron path ---
    let config_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let settings_path = config_dir.join("src/assets/settings.ron");
    let dev_ui_theme_dir = config_dir.join("src/assets/themes/gtk4");
    let prod_ui_theme_dir = config_dir.join("themes/ui");
    let ui_theme_dir = if dev_ui_theme_dir.exists() {
        dev_ui_theme_dir
    } else {
        prod_ui_theme_dir
    };

    // Use src/assets/themes/html_viever for preview themes in dev, /themes/ in prod
    let dev_preview_theme_dir = config_dir.join("src/assets/themes/html_viever");
    let prod_preview_theme_dir = config_dir.join("themes");
    let preview_theme_dir = if dev_preview_theme_dir.exists() {
        dev_preview_theme_dir
    } else {
        prod_preview_theme_dir
    };

    // Use src/assets/themes/editor for editor style schemes in dev, /themes/editor in prod
    let dev_editor_theme_dir = config_dir.join("src/assets/themes/editor");
    let prod_editor_theme_dir = config_dir.join("themes/editor");
    let editor_theme_dir = if dev_editor_theme_dir.exists() {
        dev_editor_theme_dir
    } else {
        prod_editor_theme_dir
    };

    let theme_manager = Rc::new(RefCell::new(ThemeManager::new(
        &settings_path,
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
    let settings = &theme_manager.borrow().settings;
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
        theme_manager.borrow().preview_theme_mode_from_scheme(&current_scheme)
    };
    let theme_mode = Rc::new(RefCell::new(initial_theme_mode));
    let (footer, footer_labels_rc) = footer::create_footer();

    // Load active markdown schema from settings (if available)
    let schema_root = config_dir.join("src/assets/markdown_schema");
    let active_schema_map: Rc<RefCell<Option<MarkdownSyntaxMap>>> = Rc::new(RefCell::new(None));
    if let Ok(Some(map)) = MarkdownSyntaxMap::load_active_schema(settings_path.to_str().unwrap(), schema_root.to_str().unwrap()) {
        *active_schema_map.borrow_mut() = Some(map);
    }

    // Debug: report whether an active schema was found and how many rules it contains
    if let Some(ref map) = *active_schema_map.borrow() {
        eprintln!("[main] Active markdown schema loaded: {} rules", map.rules.len());
    } else {
        eprintln!("[main] No active markdown schema loaded (footer will show Plain text)");
    }

    let (split, _webview, preview_css_rc, refresh_preview, update_editor_theme, update_preview_theme, editor_buffer, insert_mode_state) = create_editor_with_preview(
        preview_theme_filename.as_str(),
        preview_theme_dir_str.as_str(),
        theme_manager.clone(),
        Rc::clone(&theme_mode),
        footer_labels_rc.clone()
    );

    // Wire up live footer updates using the actual editor buffer
    // Wire footer updates directly: wire_footer_updates will run callbacks on
    // the main loop and call `apply_footer_update` directly.
    wire_footer_updates(&editor_buffer, footer_labels_rc.clone(), active_schema_map.clone(), insert_mode_state.clone());
    split.add_css_class("split-view");

    // Closure to trigger an immediate footer syntax update using the active schema map
    let trigger_footer_update: std::rc::Rc<dyn Fn()> = std::rc::Rc::new({
        let buffer = editor_buffer.clone();
        let labels = footer_labels_rc.clone();
        let active_schema_map = active_schema_map.clone();
        let test_counter = std::rc::Rc::new(std::cell::Cell::new(0));
        move || {
            eprintln!("[main] Manual footer trigger called!");
            
            // Increment test counter for obvious visual changes
            let count = test_counter.get() + 1;
            test_counter.set(count);
            
            // Update with test values to make changes obvious
            crate::footer::update_cursor_row(&labels, count + 10);
            crate::footer::update_cursor_col(&labels, count + 20);
            crate::footer::update_word_count(&labels, count * 10);
            crate::footer::update_char_count(&labels, count * 50);
            crate::footer::update_encoding(&labels, &format!("TEST-{}", count));
            crate::footer::update_insert_mode(&labels, count % 2 == 0);
            
            // Also do the original syntax trace logic
            let offset = buffer.cursor_position();
            let iter = buffer.iter_at_offset(offset);
            let current_line = iter.line();
            let start_iter_opt = buffer.iter_at_line(current_line);
            let end_iter_opt = buffer.iter_at_line(current_line + 1);
            let line_text = match (start_iter_opt, end_iter_opt) {
                (Some(ref start), Some(ref end)) => buffer.text(start, end, false).to_string(),
                (Some(ref start), None) => buffer.text(start, &buffer.end_iter(), false).to_string(),
                _ => String::new(),
            };
            if let Some(ref map) = *active_schema_map.borrow() {
                crate::footer::update_syntax_trace(&labels, &line_text, map);
            } else {
                let dummy_map = crate::components::marco_engine::parser::MarkdownSyntaxMap { rules: std::collections::HashMap::new(), display_hints: None };
                crate::footer::update_syntax_trace(&labels, &line_text, &dummy_map);
            }
        }
    });

    // test footer update button removed

    // Add components to main layout (menu bar is now in titlebar)
    main_box.append(&toolbar);
    main_box.append(&split);
    main_box.append(&footer);

    // Set editor area to expand
    split.set_vexpand(true);
    
    // Ensure footer is visible and properly positioned
    footer.set_vexpand(false); // Footer should not expand vertically
    footer.set_hexpand(true);  // Footer should expand horizontally
    footer.set_visible(true);  // Explicitly ensure footer is visible
    
    // Debug output to confirm footer creation
    eprintln!("[main] Footer created and added to layout");
    eprintln!("[main] Footer visible: {}", footer.is_visible());
    eprintln!("[main] Footer height request: {}", footer.height_request());
    
    // Optionally, assign classes to editor/preview if accessible here

    // Add main box to window
    window.set_child(Some(&main_box));

    // --- Live HTML preview theme switching ---
    // Store refresh_preview closure for use on theme changes
    let refresh_preview_rc = Rc::new(RefCell::new(refresh_preview));
    let preview_css_for_settings = preview_css_rc.clone();
    // Register 'app.settings' action to show the settings dialog with the callback
    let settings_action = gtk4::gio::SimpleAction::new("settings", None);
    let win_clone = window.clone();
    let theme_manager_clone = theme_manager.clone();
    let settings_path_clone = settings_path.clone();
    let refresh_preview_for_settings2 = refresh_preview_rc.clone();
    let update_editor_theme_clone = Rc::new(update_editor_theme);
    let update_preview_theme_clone = Rc::new(update_preview_theme);
        settings_action.connect_activate({
            let win_clone = win_clone.clone();
            let theme_manager_clone = theme_manager_clone.clone();
            let settings_path_clone = settings_path_clone.clone();
            let preview_css_for_settings = preview_css_for_settings.clone();
            let refresh_preview_for_settings2 = refresh_preview_for_settings2.clone();
            let update_editor_theme_clone = update_editor_theme_clone.clone();
            let update_preview_theme_clone = update_preview_theme_clone.clone();
            move |_, _| {
                use crate::ui::settings::settings::show_settings_dialog;
                
                // Create editor theme callback that updates both editor and preview
                let editor_callback = {
                    let update_editor = update_editor_theme_clone.clone();
                    let update_preview = update_preview_theme_clone.clone();
                    Box::new(move |scheme_id: String| {
                        update_editor(&scheme_id);
                        update_preview(&scheme_id);
                    }) as Box<dyn Fn(String) + 'static>
                };
                
                show_settings_dialog(
                    win_clone.upcast_ref(),
                    theme_manager_clone.clone(),
                    settings_path_clone.clone(),
                    Some(Box::new({
                        let theme_manager_clone = theme_manager_clone.clone();
                        let preview_css_for_settings = preview_css_for_settings.clone();
                        let refresh_preview_for_settings2 = refresh_preview_for_settings2.clone();
                        move |theme_filename: String| {
                            // On preview theme change, update CSS and call refresh
                            use std::fs;
                            let theme_manager = theme_manager_clone.borrow();
                            let preview_theme_dir = theme_manager.preview_theme_dir.clone();
                            let css_path = preview_theme_dir.join(&theme_filename);
                            let css = fs::read_to_string(&css_path).unwrap_or_default();
                            *preview_css_for_settings.borrow_mut() = css;
                            (refresh_preview_for_settings2.borrow())();
                        }
                    })),
                    Some(refresh_preview_for_settings2.clone()),
                    Some(editor_callback),
                    Some(Box::new({
                        let active_schema_map = active_schema_map.clone();
                        let config_dir = config_dir.clone();
                        let settings_path_clone = settings_path_clone.clone();
                        let trigger = trigger_footer_update.clone();
                        move |_selected: Option<String>| {
                            // Reload parser and update shared map
                            let schema_root = config_dir.join("src/assets/markdown_schema");
                            if let Ok(Some(map)) = crate::components::marco_engine::parser::MarkdownSyntaxMap::load_active_schema(
                                settings_path_clone.to_str().unwrap(),
                                schema_root.to_str().unwrap(),
                            ) {
                                *active_schema_map.borrow_mut() = Some(map);
                            } else {
                                *active_schema_map.borrow_mut() = None;
                            }
                            // Trigger immediate footer update
                            (trigger)();
                        }
                    }) as Box<dyn Fn(Option<String>) + 'static>),
                );
            }
        });
    app.add_action(&settings_action);

    // Create file operations handler
    let file_operations = FileOperations::new(
        Rc::new(RefCell::new(DocumentBuffer::new_untitled())),
        Rc::new(RefCell::new(RecentFiles::new(&settings_path)))
    );
    let file_operations_rc = Rc::new(RefCell::new(file_operations));

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
        std::sync::Arc::new(|w, doc_name, action| Box::pin(FileDialogs::show_save_changes_dialog(w, doc_name, action))),
        std::sync::Arc::new(|w, title, suggested| Box::pin(FileDialogs::show_save_dialog(w, title, suggested))),
    );

    // Wire dynamic recent-file actions using the recent_menu from the UI
    crate::logic::menu_items::file::setup_recent_actions(
        app,
        file_operations_rc.clone(),
        &recent_menu,
        &window,
        &editor_buffer,
        &title_label,
        std::sync::Arc::new(|w, doc_name, action| Box::pin(FileDialogs::show_save_changes_dialog(w, doc_name, action))),
        std::sync::Arc::new(|w, title, suggested| Box::pin(FileDialogs::show_save_dialog(w, title, suggested))),
    );

    // Set up buffer change tracking - delegated to logic/menu_items/file.rs
    // We need to use a flag to prevent infinite recursion during programmatic changes
    let modification_tracking_enabled = Rc::new(RefCell::new(true));
    crate::logic::menu_items::file::attach_change_tracker(
        file_operations_rc.clone(),
        &editor_buffer,
        modification_tracking_enabled.clone(),
        &title_label,
    );

    // Register simple file actions using FileOperations business logic  
    // TODO: Move to FileOperations::register_actions when module visibility is fixed
    
    // New document action - using async version with save changes dialog
    let new_action = gtk4::gio::SimpleAction::new("new", None);
    new_action.connect_activate({
        let window = window.clone();
        let file_operations = file_operations_rc.clone();
        let editor_buffer = editor_buffer.clone();
        let tracking_enabled = modification_tracking_enabled.clone();
        let title_label = title_label.clone();
        move |_, _| {
            let window = window.clone();
            let file_operations = file_operations.clone();
            let editor_buffer = editor_buffer.clone();
            let tracking_enabled = tracking_enabled.clone();
            let title_label_async = title_label.clone();
            glib::MainContext::default().spawn_local(async move {
                *tracking_enabled.borrow_mut() = false;
                let file_ops = file_operations.borrow();
                let gtk_window: &gtk4::Window = window.upcast_ref();
                let text_buffer: &gtk4::TextBuffer = editor_buffer.upcast_ref();
                let _ = file_ops.new_document_async(
                    gtk_window,
                    text_buffer,
                    |w, doc_name, action| Box::pin(FileDialogs::show_save_changes_dialog(w, doc_name, action)),
                    |w, title, suggested| Box::pin(FileDialogs::show_save_dialog(w, title, suggested)),
                ).await;
                // Update title label to reflect new untitled document
                let title = file_operations.borrow().get_document_title();
                title_label_async.set_text(&title);
                *tracking_enabled.borrow_mut() = true;
            });
        }
    });

    // Save document action - moved logic to FileOperations  
    let save_action = gtk4::gio::SimpleAction::new("save", None);
    save_action.connect_activate({
        let window = window.clone();
        let file_operations = file_operations_rc.clone();
        let editor_buffer = editor_buffer.clone();
        let title_label = title_label.clone();
        move |_, _| {
            let file_ops_ref = file_operations.borrow();
            let text_buffer: &gtk4::TextBuffer = editor_buffer.upcast_ref();
            if let Err(e) = file_ops_ref.save_document(&window, text_buffer) {
                eprintln!("Error saving document: {}", e);
            } else {
                // Update title after successful save
                let title = file_operations.borrow().get_document_title();
                title_label.set_text(&title);
            }
        }
    });

    // TODO: Remaining async file actions to be moved to FileOperations  
    // These require async UI dialog integration
    
    let open_action = gtk4::gio::SimpleAction::new("open", None);
    let save_as_action = gtk4::gio::SimpleAction::new("save_as", None);
    let quit_action = gtk4::gio::SimpleAction::new("quit", None);

    // Open file action (async helper)
    open_action.connect_activate({
        let window = window.clone();
        let file_operations = file_operations_rc.clone();
    let editor_buffer = editor_buffer.clone();
    let title_label = title_label.clone();
        move |_, _| {
            let window = window.clone();
            let file_operations = file_operations.clone();
            let editor_buffer = editor_buffer.clone();
            let title_label_async = title_label.clone();
            glib::MainContext::default().spawn_local(async move {
                let file_ops = file_operations.borrow();
                let gtk_window: &gtk4::Window = window.upcast_ref();
                let text_buffer: &gtk4::TextBuffer = editor_buffer.upcast_ref();
                let _ = file_ops.open_file_async(
                    gtk_window,
                    text_buffer,
                    |w, title| Box::pin(FileDialogs::show_open_dialog(w, title)),
                    |w, doc_name, action| Box::pin(FileDialogs::show_save_changes_dialog(w, doc_name, action)),
                    |w, title, suggested| Box::pin(FileDialogs::show_save_dialog(w, title, suggested)),
                ).await;
                // Update title label after open completes
                let title = file_operations.borrow().get_document_title();
                title_label_async.set_text(&title);
            });
        }
    });

    // Save document action
    save_action.connect_activate({
        let window = window.clone();
        let file_operations = file_operations_rc.clone();
        let editor_buffer = editor_buffer.clone();
        move |_, _| {
            let file_ops = file_operations.borrow();
            let text_buffer: &gtk4::TextBuffer = editor_buffer.upcast_ref();
            if let Err(e) = file_ops.save_document(&window, text_buffer) {
                eprintln!("Error saving document: {}", e);
            }
        }
    });

    // Save As action (async helper)
    save_as_action.connect_activate({
        let window = window.clone();
        let file_operations = file_operations_rc.clone();
        let editor_buffer = editor_buffer.clone();
    let title_label = title_label.clone();
    move |_, _| {
            let window = window.clone();
            let file_operations = file_operations.clone();
            let editor_buffer = editor_buffer.clone();
            let title_label_async = title_label.clone();
            glib::MainContext::default().spawn_local(async move {
                let file_ops = file_operations.borrow();
                let gtk_window: &gtk4::Window = window.upcast_ref();
                let text_buffer: &gtk4::TextBuffer = editor_buffer.upcast_ref();
                let _ = file_ops.save_as_async(
                    gtk_window,
                    text_buffer,
                    |w, title, suggested| Box::pin(FileDialogs::show_save_dialog(w, title, suggested)),
                ).await;
                // Update title label after Save As completes
                let title = file_operations.borrow().get_document_title();
                title_label_async.set_text(&title);
            });
        }
    });

    // Quit application action (async helper)
    quit_action.connect_activate({
        let window = window.clone();
        let file_operations = file_operations_rc.clone();
        let editor_buffer = editor_buffer.clone();
        let app = app.clone();
        move |_, _| {
            let window = window.clone();
            let file_operations = file_operations.clone();
            let editor_buffer = editor_buffer.clone();
            let app = app.clone();
            glib::MainContext::default().spawn_local(async move {
                let file_ops = file_operations.borrow();
                let gtk_window: &gtk4::Window = window.upcast_ref();
                let text_buffer: &gtk4::TextBuffer = editor_buffer.upcast_ref();
                let _ = file_ops.quit_async(
                    gtk_window,
                    text_buffer,
                    &app,
                    |w, title, action| Box::pin(FileDialogs::show_save_changes_dialog(w, title, action)),
                    |w, title, suggested| Box::pin(FileDialogs::show_save_dialog(w, title, suggested)),
                ).await;
            });
        }
    });

    // Add file actions to application
    app.add_action(&new_action);
    app.add_action(&open_action);
    app.add_action(&save_action);
    app.add_action(&save_as_action);
    app.add_action(&quit_action);

    // Set keyboard shortcuts for file actions
    app.set_accels_for_action("app.new", &["<Control>n"]);
    app.set_accels_for_action("app.open", &["<Control>o"]);
    app.set_accels_for_action("app.save", &["<Control>s"]);
    app.set_accels_for_action("app.save_as", &["<Control><Shift>s"]);
    app.set_accels_for_action("app.quit", &["<Control>q"]);

    // Open initial file if provided via command line
    if let Some(file_path) = initial_file {
        let file_operations_initial = file_operations_rc.clone();
        let window_initial = window.clone();
        let editor_buffer_initial = editor_buffer.clone();
        let title_label_initial = title_label.clone();
        
        glib::MainContext::default().spawn_local(async move {
            let file_ops = file_operations_initial.borrow();
            let gtk_window: &gtk4::Window = window_initial.upcast_ref();
            let text_buffer: &gtk4::TextBuffer = editor_buffer_initial.upcast_ref();
            
            // Try to open the specified file
            let result = file_ops.open_file_by_path_async(
                &file_path,
                gtk_window,
                text_buffer,
                |w, doc_name, action| Box::pin(FileDialogs::show_save_changes_dialog(w, doc_name, action)),
                |w, title, suggested| Box::pin(FileDialogs::show_save_dialog(w, title, suggested)),
            ).await;
            
            match result {
                Ok(_) => {
                    // Update title label after successful open
                    let title = file_operations_initial.borrow().get_document_title();
                    title_label_initial.set_text(&title);
                    eprintln!("Successfully opened file: {}", file_path);
                }
                Err(e) => {
                    eprintln!("Failed to open file {}: {}", file_path, e);
                }
            }
        });
    }

    // Present the window
    window.present();
}