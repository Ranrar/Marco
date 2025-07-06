pub mod editor;
mod markdown;
pub mod language;
mod menu;
mod toolbar;
mod footer;
pub mod view;
mod theme;
mod settings;

use clap::{Arg, Command};
use gtk4::prelude::*;
use gtk4::{
    glib, Application, ApplicationWindow, Box, Orientation, CssProvider, gdk,
};

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
fn update_footer_labels(footer_labels: &footer::FooterLabels, word_count: usize, char_count: usize, line: usize, col: usize) {
    let word_count_str = word_count.to_string();
    let char_count_str = char_count.to_string();
    let line_str = line.to_string();
    let column_str = col.to_string();
    
    let word_args: std::collections::HashMap<&str, &str> = [("count", word_count_str.as_str())].iter().cloned().collect();
    let char_args: std::collections::HashMap<&str, &str> = [("count", char_count_str.as_str())].iter().cloned().collect();
    let pos_args: std::collections::HashMap<&str, &str> = [("line", line_str.as_str()), ("col", column_str.as_str())].iter().cloned().collect();
    
    footer_labels.word_count.set_text(&language::tr_with_args("footer.words", &word_args));
    footer_labels.char_count.set_text(&language::tr_with_args("footer.characters", &char_args));
    footer_labels.cursor_pos.set_text(&language::tr_with_args("footer.position", &pos_args));
    footer_labels.status.set_text(&language::tr("footer.ready"));
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
    // Initialize settings system early
    if let Err(e) = settings::core::initialize_settings() {
        eprintln!("Warning: Failed to initialize settings: {}", e);
    }
    
    // Parse command line arguments before GTK processing
    let args: Vec<String> = std::env::args().collect();
    
    // Parse command line arguments manually to avoid conflicts with GTK
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
                println!("Build Profile: {}", if cfg!(debug_assertions) { "Debug" } else { "Release" });
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
    
    // Filter out our custom arguments before passing to GTK
    let filtered_args: Vec<String> = args.into_iter()
        .filter(|arg| !arg.starts_with("--debug") && !arg.starts_with("-d"))
        .collect();
    
    // Override command line args for GTK
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    // Pass the file to open to the UI builder
    app.connect_activate({
        let file_to_open = file_to_open.clone();
        move |app| build_ui(app, file_to_open.as_deref(), debug_mode)
    });

    app.run_with_args(&filtered_args)
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
        }"
    );
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
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
        move |_text, word_count, char_count, line, column| {
            update_footer_labels(&footer_labels, word_count, char_count, line, column);
        }
    });

    // Set up language change detection using a more efficient approach
    setup_language_change_detection(&window, &footer_labels, app, &editor, &theme_manager);

    // Add main box to window
    window.set_child(Some(&main_box));

    // Set up 50/50 split after window is properly sized
    window.connect_realize({
        let editor = editor.clone();
        move |window| {
            // Get the default window width and set 50/50 split
            let width = window.default_width();
            editor.set_split_ratio(width);
        }
    });

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
            if let Err(e) = settings::preferences::save_app_state_to_settings(&editor, &theme_manager) {
                eprintln!("Warning: Failed to save application state: {}", e);
            }
            
            // Cast ApplicationWindow to Window for the editor method
            let window_ref = window.upcast_ref::<gtk4::Window>();
            let app_clone = app.clone();
            
            // Check for unsaved changes and prevent closing if needed
            let should_close_immediately = editor.show_unsaved_changes_dialog_and_quit(
                Some(window_ref),
                move || {
                    println!("DEBUG: Confirmed close, calling app.quit()");
                    app_clone.quit();
                }
            );
            
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
