mod footer;
mod markdown;
mod menu;
mod settings;
mod theme;
mod toolbar;
pub mod ui;
pub mod editor;
pub mod view;
pub mod utils;

use clap::{Arg, Command};
use gtk4::prelude::*;
use gtk4::{gdk, glib, Application, ApplicationWindow, Box, CssProvider, Orientation};

use crate::utils::language;

const APP_ID: &str = "com.example.Marco";

fn extract_number_from_text(text: &str) -> Option<u32> {
    // Extract the first number found in the text
    text.chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .ok()
}

fn extract_position_from_text(text: &str) -> Option<(u32, u32)> {
    // Extract line and column from text like "Line: 1, Col: 5" or similar
    let numbers: Vec<u32> = text
        .split(|c: char| !c.is_ascii_digit())
        .filter_map(|s| s.parse().ok())
        .collect();

    if numbers.len() >= 2 {
        Some((numbers[0], numbers[1]))
    } else {
        None
    }
}

/// Update footer labels with current counts and translations
fn update_footer_labels(
    footer_labels: &footer::FooterLabels,
    word_count: usize,
    char_count: usize,
    line: usize,
    col: usize,
) {
    let word_count_str = word_count.to_string();
    let char_count_str = char_count.to_string();
    let line_str = line.to_string();
    let column_str = col.to_string();

    let word_args: std::collections::HashMap<&str, &str> = [("count", word_count_str.as_str())]
        .iter()
        .cloned()
        .collect();
    let char_args: std::collections::HashMap<&str, &str> = [("count", char_count_str.as_str())]
        .iter()
        .cloned()
        .collect();
    let pos_args: std::collections::HashMap<&str, &str> =
        [("line", line_str.as_str()), ("col", column_str.as_str())]
            .iter()
            .cloned()
            .collect();

    footer_labels
        .word_count
        .set_text(&language::tr_with_args("footer.words", &word_args));
    footer_labels
        .char_count
        .set_text(&language::tr_with_args("footer.characters", &char_args));
    footer_labels
        .cursor_pos
        .set_text(&language::tr_with_args("footer.position", &pos_args));
}

/// Set up language change detection system
fn setup_language_change_detection(
    window: &ApplicationWindow,
    footer_labels: &footer::FooterLabels,
    app: &Application,
    editor: &editor::MarkdownEditor,
    theme_manager: &theme::ThemeManager,
) {
    let _window_clone = window.clone();
    let footer_labels_clone = footer_labels.clone();
    let app_clone = app.clone();
    let editor_clone = editor.clone();
    let theme_manager_clone = theme_manager.clone();

    // Use a more efficient approach - check every 500ms instead of 100ms
    glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
        if language::check_language_changed() {
            // Update window title using editor's dynamic title update
            editor_clone.update_window_title();

            // Use our unified menu rebuilding system
            menu::rebuild_menu_bar(&app_clone, &editor_clone, &theme_manager_clone);

            // Rebuild toolbar (we should add a similar function for toolbar)
            rebuild_toolbar(&app_clone, &editor_clone);

            // Update footer with current values (preserve the numbers, update the format)
            let word_text = footer_labels_clone.word_count.text().to_string();
            let char_text = footer_labels_clone.char_count.text().to_string();
            let pos_text = footer_labels_clone.cursor_pos.text().to_string();

            // Extract numbers from current text
            let word_count = extract_number_from_text(&word_text).unwrap_or(0) as usize;
            let char_count = extract_number_from_text(&char_text).unwrap_or(0) as usize;
            let (line_u32, col_u32) = extract_position_from_text(&pos_text).unwrap_or((1, 1));
            let line = line_u32 as usize;
            let col = col_u32 as usize;

            // Update footer with new translations
            update_footer_labels(&footer_labels_clone, word_count, char_count, line, col);
        }
        glib::ControlFlow::Continue
    });
}

/// Rebuild toolbar with new translations (similar to menu rebuilding)
fn rebuild_toolbar(app: &Application, editor: &editor::MarkdownEditor) {
    toolbar::rebuild_toolbar_in_window(app, editor);
}

fn main() -> glib::ExitCode {
    // DEBUG: Start of main
    println!("[DEBUG] Entered main()");
    // Print all environment variables for debugging menu vs terminal launch
    println!("[DEBUG] Environment variables at startup:");
    for (key, value) in std::env::vars() {
        println!("[DEBUG] {}={}", key, value);
    }
    // Initialize settings system early
    if let Err(e) = settings::core::initialize_settings() {
        eprintln!("Warning: Failed to initialize settings: {}", e);
    }


    // Parse command line arguments before GTK processing
    let args: Vec<String> = std::env::args().collect();

    // DEV-only: Add --register-open-with flag
    #[cfg(debug_assertions)]
    {
        // ...existing DEV-only code...
        println!("[DEBUG] DEV mode block entered");
        use clap::ArgAction;
        let matches = Command::new("marco")
            .version("0.1.0")
            .author("Kim Skov Rasmussen")
            .about("Marco - Markdown Composer")
            .arg(
                Arg::new("debug")
                    .short('d')
                    .long("debug")
                    .help("Enable debug mode with verbose output")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("register-open-with")
                    .long("register-open-with")
                    .help("Register 'Open with Marco' in the OS context menu (DEV mode only)")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("platform")
                    .long("platform")
                    .help("Platform for registration: linux, windows, macos")
                    .value_parser(["linux", "windows", "macos"])
                    .default_value("linux"),
            )
            .arg(
                Arg::new("file")
                    .help("Optional markdown file to open")
                    .value_name("FILE")
                    .index(1),
            )
            .try_get_matches_from(&args);

        let (debug_mode, file_to_open) = match matches {
            Ok(matches) => {
                println!("[DEBUG] Parsed CLI arguments");
                let debug_mode = matches.get_flag("debug");
                let file_to_open = matches.get_one::<String>("file").map(|s| s.as_str());
                let register_open_with = matches.get_flag("register-open-with");
                let platform = matches.get_one::<String>("platform").map(|s| s.as_str()).unwrap_or("linux");

                if register_open_with {
                    println!("[DEBUG] --register-open-with flag detected, platform: {}", platform);
                    match platform {
                        "linux" => {
                            println!("[DEBUG] Entering Linux registration block");
                            use std::fs;
                            use std::path::PathBuf;
                            let home = std::env::var("HOME").unwrap_or_else(|_| String::from("~"));
                            let src = "dev/os-integration/linux/marco.desktop";
                            let dest_dir = format!("{}/.local/share/applications", home);
                            let dest = format!("{}/marco.desktop", dest_dir);
                            if let Err(e) = fs::create_dir_all(&dest_dir) {
                                eprintln!("[ERROR] Failed to create directory {}: {}", dest_dir, e);
                            }
                            let desktop = match fs::read_to_string(src) {
                                Ok(content) => content,
                                Err(e) => {
                                    eprintln!("[ERROR] Failed to read {}: {}", src, e);
                                    std::process::exit(1);
                                }
                            };
                            let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("/usr/bin/marco"));
                            let desktop = desktop.replace("/full/path/to/marco", exe_path.to_str().unwrap_or("/usr/bin/marco"));
                            match fs::write(&dest, &desktop) {
                                Ok(_) => println!("[DEV] Copied marco.desktop to {}", dest),
                                Err(e) => {
                                    eprintln!("[ERROR] Failed to write {}: {}", dest, e);
                                    std::process::exit(1);
                                }
                            }
                            match std::process::Command::new("update-desktop-database").arg(&dest_dir).status() {
                                Ok(status) => println!("[DEV] Ran update-desktop-database (exit code: {})", status),
                                Err(e) => eprintln!("[ERROR] Failed to run update-desktop-database: {}", e),
                            }
                            println!("[DEV] 'Open With Marco' should now appear for .md files in Nautilus/KDE");
                        }
                        "windows" => {
                            println!("[DEV] To register on Windows, import dev/os-integration/open_with_marco.reg after editing the path to your marco.exe");
                        }
                        "macos" => {
                            println!("[DEV] For macOS, add the Info.plist.snippet.xml to your app bundle's Info.plist and rebuild the .app");
                        }
                        _ => println!("[DEV] Unknown platform: {}", platform),
                    }
                    println!("[DEBUG] Exiting after registration block");
                    std::process::exit(0);
                }

                if debug_mode {
                    println!("Debug mode enabled");
                    std::env::set_var("RUST_LOG", "debug");
                    println!("Marco - Debug Mode");
                    println!("Version: 0.1.0");
                    println!("GTK4 Version: {}", gtk4::major_version());
                    println!(
                        "Build Profile: {}",
                        if cfg!(debug_assertions) {
                            "Debug"
                        } else {
                            "Release"
                        }
                    );
                }
                (debug_mode, file_to_open.map(|s| s.to_string()))
            }
            Err(_) => {
                (false, None)
            }
        };

        // Initialize localization
        language::init_localization();

        // Override command line args for GTK
        let app = Application::builder().application_id(APP_ID).build();

        app.connect_activate({
            let file_to_open = file_to_open.clone();
            move |app| build_ui(app, file_to_open.as_deref(), debug_mode)
        });

        let no_args: [&str; 0] = [];
        return app.run_with_args(&no_args);
    }

    #[cfg(not(debug_assertions))]
    {
        // ...existing non-DEV code...
        let matches = Command::new("marco")
            .version("0.1.0")
            .author("Kim Skov Rasmussen")
            .about("Marco - Markdown Composer")
            .arg(
                Arg::new("debug")
                    .short('d')
                    .long("debug")
                    .help("Enable debug mode with verbose output")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("file")
                    .help("Optional markdown file to open")
                    .value_name("FILE")
                    .index(1),
            )
            .try_get_matches_from(&args);

        let (debug_mode, file_to_open) = match matches {
            Ok(matches) => {
                let debug_mode = matches.get_flag("debug");
                let file_to_open = matches.get_one::<String>("file").map(|s| s.as_str());

                if debug_mode {
                    println!("Debug mode enabled");
                    std::env::set_var("RUST_LOG", "debug");

                    // Enable additional debug output
                    println!("Marco - Debug Mode");
                    println!("Version: 0.1.0");
                    println!("GTK4 Version: {}", gtk4::major_version());
                    println!(
                        "Build Profile: {}",
                        if cfg!(debug_assertions) {
                            "Debug"
                        } else {
                            "Release"
                        }
                    );
                }

                (debug_mode, file_to_open.map(|s| s.to_string()))
            }
            Err(_) => {
                // If parsing fails, run without debug mode
                (false, None)
            }
        };

        // Initialize localization
        language::init_localization();

        // Override command line args for GTK
        let app = Application::builder().application_id(APP_ID).build();

        app.connect_activate({
            let file_to_open = file_to_open.clone();
            move |app| build_ui(app, file_to_open.as_deref(), debug_mode)
        });

        let no_args: [&str; 0] = [];
        return app.run_with_args(&no_args);
    }
}

fn build_ui(app: &Application, file_to_open: Option<&str>, debug_mode: bool) {
    if debug_mode {
        println!("Building UI with debug mode enabled");
        if let Some(file) = file_to_open {
            println!("File to open: {}", file);
        }
    }

    // Get window size from settings
    let prefs = settings::get_app_preferences();
    let (default_width, default_height) = prefs.get_window_size();

    // Create the main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Marco") // Initial title, will be updated by editor.update_window_title()
        .default_width(default_width)
        .default_height(default_height)
        .build();

    // Set up CSS for error styling and toolbar button states
    let provider = CssProvider::new();
    provider.load_from_data(
        ".error {
            border: 2px solid #e53e3e;
            background-color: #fed7d7;
        }
        button {
            transition: all 0.3s ease;
            border-radius: 6px;
        }
        button.active-format {
            background-color: #3b82f6;
            color: white;
            font-weight: bold;
            background-image: none;
        }",
    );
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Apply context menu styling globally for popover menus
    // (re-use the preview context menu CSS for all popover menus)
    let context_menu_css = "
        /* PopoverMenu (GMenuModel) styling */
        popover.menu button.model {
            padding: 8px 16px;
            margin: 1px;
            border-radius: 4px;
            transition: background-color 0.1s ease;
            font-family: -gtk-system-font;
            font-size: 0.9em;
            min-width: 200px;
        }
        popover.menu button.model:hover {
            background-color: alpha(@accent_color, 0.1);
            transition: background-color 0.05s ease;
        }
        popover.menu button.model:disabled {
            opacity: 0.5;
            color: alpha(@theme_fg_color, 0.5);
        }
        popover.menu separator {
            min-height: 1px;
            background-color: alpha(@borders, 0.3);
            margin: 4px 8px;
            border: none;
            padding: 0;
            opacity: 1;
        }
        popover.menu button.model label {
            color: @theme_fg_color;
        }
        popover.menu button.model .accelerator {
            color: alpha(@theme_fg_color, 0.7);
            font-size: 0.85em;
            margin-left: 16px;
        }

        /* Custom ListBox-based popover menu styling */
        list, listbox {
            background: transparent;
            border: none;
            padding: 4px 0;
        }
        list row.menuitem {
            padding: 8px 16px;
            margin: 1px 0;
            border-radius: 4px;
            font-family: -gtk-system-font;
            font-size: 0.95em;
            min-width: 200px;
            background: transparent;
            transition: background-color 0.1s ease;
        }
        list row.menuitem:hover, list row.menuitem:selected {
            background-color: alpha(@accent_color, 0.1);
            transition: background-color 0.05s ease;
        }
        list row.menuitem label {
            color: @theme_fg_color;
        }
        list row.menuitem:disabled label {
            opacity: 0.5;
            color: alpha(@theme_fg_color, 0.5);
        }
    ";
    let context_menu_provider = CssProvider::new();
    context_menu_provider.load_from_data(context_menu_css);
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &context_menu_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
    );

    // Create the editor
    let editor = editor::MarkdownEditor::new();

    // Create and set up theme manager
    let theme_manager = theme::ThemeManager::new();

    // Initialize theme manager with default CSS
    if let Err(e) = theme_manager.initialize() {
        eprintln!("Warning: Failed to initialize theme manager: {}", e);
    }

    editor.set_theme_manager(theme_manager.clone());

    // Apply settings CSS
    settings::ui::apply_settings_css();

    // Load preferences and apply them

    // Restore window geometry
    if let Err(e) = settings::preferences::restore_window_geometry(&window) {
        eprintln!("Warning: Failed to restore window geometry: {}", e);
    }

    // Load and apply application state from settings
    if let Err(e) = settings::preferences::load_app_state_from_settings(&editor, &theme_manager) {
        eprintln!("Warning: Failed to load application state: {}", e);
    }

    // Connect to settings changes
    if let Err(e) = settings::preferences::connect_settings_changes(&editor, &theme_manager) {
        eprintln!("Warning: Failed to connect settings changes: {}", e);
    }

    // Set up header bar (without file buttons)
    let header_bar = editor.create_simple_header_bar();

    // Add settings button to header bar
    settings::ui::add_settings_button_to_header_bar(header_bar, &window, &editor, &theme_manager);

    window.set_titlebar(Some(header_bar));

    // Set initial window title
    editor.update_window_title();

    // Create main vertical box
    let main_box = Box::new(Orientation::Vertical, 0);

    // Create and add menu bar (positioned between title and toolbar)
    let menu_bar = menu::create_menu_bar(app, &editor, &theme_manager);
    main_box.append(&menu_bar);

    // Create and add toolbar with markdown formatting (no file buttons)
    let (toolbar, _toolbar_buttons) = toolbar::create_markdown_toolbar(&editor);
    main_box.append(&toolbar);

    // Add editor to main box (takes most of the space)
    main_box.append(editor.widget());
    editor.widget().set_vexpand(true);

    // Add separator
    let separator = gtk4::Separator::new(Orientation::Horizontal);
    main_box.append(&separator);

    // Create footer
    let (footer, footer_labels) = footer::create_footer();
    main_box.append(&footer);

    // Connect editor to footer updates
    editor.add_footer_callback({
        let footer_labels = footer_labels.clone();
        move |text, word_count, char_count, line, column| {
            update_footer_labels(&footer_labels, word_count, char_count, line, column);
            // Also update formatting label
            let formatting_html = crate::footer::get_formatting_at_cursor(text, line);
            crate::footer::update_formatting_label(&footer_labels, &formatting_html);
        }
    });

    // Set up language change detection using a more efficient approach
    setup_language_change_detection(&window, &footer_labels, app, &editor, &theme_manager);

    // Add main box to window
    window.set_child(Some(&main_box));

    // Set up split ratio from settings after paned is fully sized
    // Set up split ratio from settings after main loop starts
    {
        let editor = editor.clone();
        glib::idle_add_local(move || {
            let paned = &editor.widget;
            let width = paned.allocated_width();
            if width > 0 {
                let ratio = settings::get_app_preferences().get_layout_ratio();
                let min = 10;
                let max = 90;
                let ratio = ratio.clamp(min, max);
                let mut pos = (width * ratio / 100).max(200).min(width - 200);
                if pos < 200 { pos = 200; }
                if pos > width - 200 { pos = width - 200; }
                paned.set_position(pos);
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    }

    // Handle window close event to check for unsaved changes
    window.connect_close_request({
        let editor = editor.clone();
        let theme_manager = theme_manager.clone();
        let app = app.clone();
        move |window| {
            // Save window geometry
            if let Err(e) = settings::preferences::save_window_geometry(window) {
                eprintln!("Warning: Failed to save window geometry: {}", e);
            }

            // Save application state (view mode, themes, language, etc.)
            if let Err(e) =
                settings::preferences::save_app_state_to_settings(&editor, &theme_manager)
            {
                eprintln!("Warning: Failed to save application state: {}", e);
            }

            // Cast ApplicationWindow to Window for the editor method
            let window_ref = window.upcast_ref::<gtk4::Window>();
            let app_clone = app.clone();

            // Check for unsaved changes and prevent closing if needed
            let should_close_immediately =
                editor.show_unsaved_changes_dialog_and_quit(Some(window_ref), move || {
                    println!("DEBUG: Confirmed close, calling app.quit()");
                    app_clone.quit();
                });

            if should_close_immediately {
                glib::Propagation::Proceed // Allow closing immediately
            } else {
                glib::Propagation::Stop // Prevent closing, will happen in dialog callback
            }
        }
    });

    // Present the window
    window.present();

    // Open the file if provided via command line
    if let Some(file_path) = file_to_open {
        if debug_mode {
            println!("Opening file: {}", file_path);
        }

        // Use the editor's file operations to open the file
        if let Err(e) = editor.load_file_from_path(file_path) {
            if debug_mode {
                eprintln!("Error opening file '{}': {}", file_path, e);
            }
        } else if debug_mode {
            println!("Successfully opened file: {}", file_path);
        }
    }
}
