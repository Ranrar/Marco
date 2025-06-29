mod editor;
mod syntax_basic;
mod syntax_extended;
mod syntax_advanced;
mod code_languages;
mod localization;
mod menu;
mod toolbar;
mod footer;
mod view_code;
mod view_html;
mod emoji;
mod context_menu;
mod theme;
mod settings;

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
    
    footer_labels.word_count.set_text(&localization::tr_with_args("footer.words", &word_args));
    footer_labels.char_count.set_text(&localization::tr_with_args("footer.characters", &char_args));
    footer_labels.cursor_pos.set_text(&localization::tr_with_args("footer.position", &pos_args));
    footer_labels.status.set_text(&localization::tr("footer.ready"));
}

/// Set up language change detection system
fn setup_language_change_detection(
    window: &ApplicationWindow,
    footer_labels: &footer::FooterLabels,
    app: &Application,
    editor: &editor::MarkdownEditor,
    theme_manager: &theme::ThemeManager,
) {
    let window_clone = window.clone();
    let footer_labels_clone = footer_labels.clone();
    let app_clone = app.clone();
    let editor_clone = editor.clone();
    let theme_manager_clone = theme_manager.clone();
    
    // Use a more efficient approach - check every 500ms instead of 100ms
    glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
        if localization::check_language_changed() {
            // Update window title
            window_clone.set_title(Some(&localization::tr("app.title")));
            
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
    // Initialize localization
    localization::init_localization();
    
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run()
}

fn build_ui(app: &Application) {
    // Create the main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title(&localization::tr("app.title"))
        .default_width(800)
        .default_height(600)
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
    editor.set_theme_manager(theme_manager.clone());

    // Initialize settings from current application state
    let current_view_mode = editor.get_view_mode();
    let current_css_theme = editor.get_current_css_theme();
    let current_ui_theme = match theme_manager.get_current_theme() {
        theme::Theme::System => "system",
        theme::Theme::Light => "light",
        theme::Theme::Dark => "dark",
    };
    let current_language = localization::get_current_locale();
    
    settings::initialize_settings_from_app(&current_view_mode, &current_css_theme, current_ui_theme, &current_language);

    // Set up header bar (without file buttons)
    let header_bar = editor.create_simple_header_bar();
    window.set_titlebar(Some(&header_bar));

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
        let app = app.clone();
        move |window| {
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
}
