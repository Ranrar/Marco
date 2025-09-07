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

use crate::components::editor::editor_ui::create_editor_with_preview;
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

    // Asset path detection and environment setup
    use crate::logic::asset_path::{get_asset_dir_checked, get_font_path, get_settings_path};
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
    // Ensure file logger is flushed and closed on normal exit
    crate::logic::logger::shutdown_file_logger();
    exit_code
}

fn build_ui(app: &Application, initial_file: Option<String>) {
    // Load and apply menu.css for menu and titlebar styling
    use gtk4 as gtk;
    use gtk4::gdk::Display;
    use gtk4::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};
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

    // Initialize file logger according to settings (runtime)
    {
        let app_settings =
            crate::logic::swanson::Settings::load_from_file(settings_path.to_str().unwrap())
                .unwrap_or_default();

        // Enable logging if RUST_LOG environment variable is set or if configured in settings
        let rust_log_set = std::env::var("RUST_LOG").is_ok();
        let enabled = app_settings.log_to_file.unwrap_or(false) || rust_log_set;

        // Use Debug level when RUST_LOG is set, otherwise use Trace
        let level = if rust_log_set {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Trace
        };

        if let Err(e) = crate::logic::logger::init_file_logger(enabled, level) {
            eprintln!("Failed to initialize file logger: {}", e);
        }

        if rust_log_set {
            println!("Debug logging enabled, check log files in ./log/ directory");
        }
    }
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
        theme_manager
            .borrow()
            .preview_theme_mode_from_scheme(&current_scheme)
    };
    let theme_mode = Rc::new(RefCell::new(initial_theme_mode));
    let (footer, footer_labels_rc) = footer::create_footer();

    // Active markdown schema support removed; footer uses AST parser directly.
    let _schema_root = config_dir.join("src/assets/markdown_schema");
    let active_schema_map: Rc<RefCell<Option<()>>> = Rc::new(RefCell::new(None));

    let (
        split,
        _webview,
        preview_css_rc,
        refresh_preview,
        update_editor_theme,
        update_preview_theme,
        editor_buffer,
        insert_mode_state,
        set_view_mode,
    ) = create_editor_with_preview(
        preview_theme_filename.as_str(),
        preview_theme_dir_str.as_str(),
        theme_manager.clone(),
        Rc::clone(&theme_mode),
        footer_labels_rc.clone(),
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
    split.add_css_class("split-view");

    // Apply saved view mode from settings at startup (if present)
    if let Ok(s) = crate::logic::swanson::Settings::load_from_file(settings_path.to_str().unwrap())
    {
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

    // Closure to trigger an immediate footer syntax update using the active schema map
    let trigger_footer_update: std::rc::Rc<dyn Fn()> = std::rc::Rc::new({
        let buffer = editor_buffer.clone();
        let labels = footer_labels_rc.clone();
        let test_counter = std::rc::Rc::new(std::cell::Cell::new(0));
        move || {
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
            crate::footer::update_insert_mode(&labels, count % 2 == 0);

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
        }
    });

    // Add components to main layout (menu bar is now in titlebar)
    main_box.append(&toolbar);
    main_box.append(&split);
    main_box.append(&footer);

    // Set editor area to expand
    split.set_vexpand(true);

    // Ensure footer is visible and properly positioned
    footer.set_vexpand(false); // Footer should not expand vertically
    footer.set_hexpand(true); // Footer should expand horizontally
    footer.set_visible(true); // Explicitly ensure footer is visible

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
    // Clone the runtime setter for the settings dialog callback so the original
    // Rc isn't moved and can still be used for action registration below.
    let set_view_mode_for_dialog = set_view_mode_rc.clone();

    // Helper to persist view mode in settings.ron without blocking the UI
    // This spawns a short-lived thread to perform file I/O. The settings file
    // is small so this is a pragmatic choice; for heavy I/O consider using a
    // dedicated worker queue or async executor.
    let save_view_mode = {
        let settings_path = settings_path.clone();
        move |mode: &str| {
            let path = settings_path.clone();
            let mode_owned = mode.to_string();
            std::thread::spawn(move || {
                use crate::logic::swanson::{LayoutSettings, Settings as AppSettings};
                let mut s = AppSettings::load_from_file(path.to_str().unwrap()).unwrap_or_default();
                if s.layout.is_none() {
                    s.layout = Some(LayoutSettings::default());
                }
                if let Some(ref mut l) = s.layout {
                    l.view_mode = Some(mode_owned.clone());
                }
                let _ = s.save_to_file(path.to_str().unwrap());
            });
        }
    };

    settings_action.connect_activate({
        let win_clone = win_clone.clone();
        let theme_manager_clone = theme_manager_clone.clone();
        let settings_path_clone = settings_path_clone.clone();
        let preview_css_for_settings = preview_css_for_settings.clone();
        let refresh_preview_for_settings2 = refresh_preview_for_settings2.clone();
        let update_editor_theme_clone = update_editor_theme_clone.clone();
        let update_preview_theme_clone = update_preview_theme_clone.clone();
        move |_, _| {
            use crate::ui::settings::dialog::show_settings_dialog;

            // Create editor theme callback that updates both editor and preview
            let editor_callback = {
                let update_editor = update_editor_theme_clone.clone();
                let update_preview = update_preview_theme_clone.clone();
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
                refresh_preview: Some(refresh_preview_for_settings2.clone()),
                on_editor_theme_changed: Some(editor_callback),
                on_schema_changed: Some(Box::new({
                    let active_schema_map = active_schema_map.clone();
                    let trigger = trigger_footer_update.clone();
                    move |_selected: Option<String>| {
                        // Schema support removed; clear any existing schema and trigger footer update
                        *active_schema_map.borrow_mut() = None;
                        (trigger)();
                    }
                })),
                // on_view_mode_changed: persist and forward to runtime setter
                on_view_mode_changed: Some(Box::new({
                    let sv = set_view_mode_for_dialog.clone();
                    let save = save_view_mode.clone();
                    move |selected: String| {
                        // Persist the selection asynchronously
                        save(&selected);
                        match selected.as_str() {
                            "HTML Preview" => (sv)(ViewMode::HtmlPreview),
                            "Source Code" | "Code Preview" => (sv)(ViewMode::CodePreview),
                            _ => {}
                        }
                    }
                }) as Box<dyn Fn(String) + 'static>),
            };

            show_settings_dialog(
                win_clone.upcast_ref(),
                theme_manager_clone.clone(),
                settings_path_clone.clone(),
                callbacks,
            );
        }
    });
    app.add_action(&settings_action);

    // Register view mode actions to switch preview and persist setting
    let sv_clone = set_view_mode_rc.clone();
    let settings_path_clone2 = settings_path.clone();
    let view_html_action = gtk4::gio::SimpleAction::new("view_html", None);
    view_html_action.connect_activate(move |_, _| {
        (sv_clone)(ViewMode::HtmlPreview);
        // Persist setting
        let mut s =
            crate::logic::swanson::Settings::load_from_file(settings_path_clone2.to_str().unwrap())
                .unwrap_or_default();
        if true {
            if s.layout.is_none() {
                s.layout = Some(crate::logic::swanson::LayoutSettings::default());
            }
            if let Some(ref mut l) = s.layout {
                l.view_mode = Some("HTML Preview".to_string());
            }
            let _ = s.save_to_file(settings_path_clone2.to_str().unwrap());
        }
    });
    app.add_action(&view_html_action);

    let sv_clone2 = set_view_mode_rc.clone();
    let settings_path_clone3 = settings_path.clone();
    let view_code_action = gtk4::gio::SimpleAction::new("view_code", None);
    view_code_action.connect_activate(move |_, _| {
        (sv_clone2)(ViewMode::CodePreview);
        let mut s =
            crate::logic::swanson::Settings::load_from_file(settings_path_clone3.to_str().unwrap())
                .unwrap_or_default();
        if true {
            if s.layout.is_none() {
                s.layout = Some(crate::logic::swanson::LayoutSettings::default());
            }
            if let Some(ref mut l) = s.layout {
                l.view_mode = Some("Source Code".to_string());
            }
            let _ = s.save_to_file(settings_path_clone3.to_str().unwrap());
        }
    });
    app.add_action(&view_code_action);

    // Create file operations handler
    let file_operations = FileOperations::new(
        Rc::new(RefCell::new(DocumentBuffer::new_untitled())),
        Rc::new(RefCell::new(RecentFiles::new(&settings_path))),
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

    // Present the window
    window.present();
}
