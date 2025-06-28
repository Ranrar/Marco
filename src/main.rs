mod editor;
mod syntax_basic;
mod syntax_extended;
mod code_languages;
mod localization;
mod menu;
mod toolbar;
mod footer;
mod preview;

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
        }
        /* Menu accelerator styling - make shortcuts faded and smaller */
        menu menuitem accelerator {
            color: alpha(@theme_fg_color, 0.55);
            font-size: 0.8em;
            font-weight: normal;
            margin-left: 20px;
        }
        menu menuitem {
            padding: 4px 8px;
        }
        /* Code block styling */
        .code-block {
            background-color: #f6f8fa;
            color: #24292e;
            font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
            font-size: 0.9em;
            line-height: 1.4;
            padding: 12px;
            border-radius: 6px;
            border: 1px solid #e1e4e8;
            margin: 8px 0;
            overflow-x: auto;
            white-space: pre;
        }
        .code-block .keyword {
            color: #d73a49;
            font-weight: bold;
        }
        .code-block .comment {
            color: #6a737d;
            font-style: italic;
        }
        .code-block .string {
            color: #032f62;
        }
        .code-block .number {
            color: #005cc5;
        }
        .code-block .function {
            color: #6f42c1;
            font-weight: bold;
        }
        .code-block .class {
            color: #e36209;
            font-weight: bold;
        }
        
        /* Language-specific code blocks */
        .code-block-javascript, .code-block-js {
            border-left: 4px solid #f7df1e;
        }
        .code-block-python, .code-block-py {
            border-left: 4px solid #3776ab;
        }
        .code-block-rust, .code-block-rs {
            border-left: 4px solid #ce422b;
        }
        .code-block-java {
            border-left: 4px solid #f89820;
        }
        .code-block-typescript, .code-block-ts {
            border-left: 4px solid #007acc;
        }
        .code-block-csharp, .code-block-cs {
            border-left: 4px solid #239120;
        }
        .code-block-cpp, .code-block-c++ {
            border-left: 4px solid #00599c;
        }
        .code-block-c {
            border-left: 4px solid #a8b9cc;
        }
        .code-block-php {
            border-left: 4px solid #777bb4;
        }
        .code-block-go {
            border-left: 4px solid #00add8;
        }"
    );
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Create the editor
    let editor = editor::MarkdownEditor::new();

    // Set up header bar (without file buttons)
    let header_bar = editor.create_simple_header_bar();
    window.set_titlebar(Some(&header_bar));

    // Create main vertical box
    let main_box = Box::new(Orientation::Vertical, 0);
    
    // Create and add menu bar
    let menu_bar = menu::create_menu_bar(app, &editor);
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
            let word_count_str = word_count.to_string();
            let char_count_str = char_count.to_string();
            let line_str = line.to_string();
            let column_str = column.to_string();
            
            let word_args: std::collections::HashMap<&str, &str> = [("count", word_count_str.as_str())].iter().cloned().collect();
            let char_args: std::collections::HashMap<&str, &str> = [("count", char_count_str.as_str())].iter().cloned().collect();
            let pos_args: std::collections::HashMap<&str, &str> = [("line", line_str.as_str()), ("col", column_str.as_str())].iter().cloned().collect();
            
            footer_labels.word_count.set_text(&localization::tr_with_args("footer.words", &word_args));
            footer_labels.char_count.set_text(&localization::tr_with_args("footer.characters", &char_args));
            footer_labels.cursor_pos.set_text(&localization::tr_with_args("footer.position", &pos_args));
            footer_labels.status.set_text(&localization::tr("footer.ready"));
        }
    });

    // Set up periodic language change checking using GTK timeout
    // We'll use Rc<RefCell<>> to store mutable references to the current widgets
    let window_clone = window.clone();
    let footer_labels_clone = footer_labels.clone();
    let app_clone = app.clone();
    let editor_clone = editor.clone();
    let main_box_clone = main_box.clone();
    
    // Store current widget references that can be updated
    let current_menu_bar = std::rc::Rc::new(std::cell::RefCell::new(menu_bar.clone()));
    let current_toolbar = std::rc::Rc::new(std::cell::RefCell::new(toolbar.clone()));
    
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        if localization::check_language_changed() {
            // Update window title
            window_clone.set_title(Some(&localization::tr("app.title")));
            
            // Replace menu bar with new translations
            if let Ok(old_menu_bar) = current_menu_bar.try_borrow() {
                // Only remove if it's actually in the container
                if old_menu_bar.parent().is_some() {
                    main_box_clone.remove(&*old_menu_bar);
                }
            }
            let new_menu_bar = menu::create_menu_bar(&app_clone, &editor_clone);
            main_box_clone.prepend(&new_menu_bar);
            // Update the reference
            if let Ok(mut menu_ref) = current_menu_bar.try_borrow_mut() {
                *menu_ref = new_menu_bar;
            }
            
            // Replace toolbar with new translations (tooltips)
            if let Ok(old_toolbar) = current_toolbar.try_borrow() {
                // Only remove if it's actually in the container
                if old_toolbar.parent().is_some() {
                    main_box_clone.remove(&*old_toolbar);
                }
            }
            let (new_toolbar, _new_toolbar_buttons) = toolbar::create_markdown_toolbar(&editor_clone);
            // Insert after menu bar (position 1)
            if let Ok(menu_bar_ref) = current_menu_bar.try_borrow() {
                main_box_clone.insert_child_after(&new_toolbar, Some(&*menu_bar_ref));
            }
            // Update the reference
            if let Ok(mut toolbar_ref) = current_toolbar.try_borrow_mut() {
                *toolbar_ref = new_toolbar;
            }
            
            // Update footer with current values (preserve the numbers, update the format)
            let word_text = footer_labels_clone.word_count.text().to_string();
            let char_text = footer_labels_clone.char_count.text().to_string();
            let pos_text = footer_labels_clone.cursor_pos.text().to_string();
            
            // Extract numbers from current text (simple parsing)
            let word_count = extract_number_from_text(&word_text).unwrap_or(0);
            let char_count = extract_number_from_text(&char_text).unwrap_or(0);
            
            // For position, it's more complex as it has line:col format
            let (line, col) = extract_position_from_text(&pos_text).unwrap_or((1, 1));
            
            // Update with new translations
            let word_count_str = word_count.to_string();
            let char_count_str = char_count.to_string();
            let line_str = line.to_string();
            let column_str = col.to_string();
            
            let word_args: std::collections::HashMap<&str, &str> = [("count", word_count_str.as_str())].iter().cloned().collect();
            let char_args: std::collections::HashMap<&str, &str> = [("count", char_count_str.as_str())].iter().cloned().collect();
            let pos_args: std::collections::HashMap<&str, &str> = [("line", line_str.as_str()), ("col", column_str.as_str())].iter().cloned().collect();
            
            footer_labels_clone.word_count.set_text(&localization::tr_with_args("footer.words", &word_args));
            footer_labels_clone.char_count.set_text(&localization::tr_with_args("footer.characters", &char_args));
            footer_labels_clone.cursor_pos.set_text(&localization::tr_with_args("footer.position", &pos_args));
            footer_labels_clone.status.set_text(&localization::tr("footer.ready"));
        }
        glib::ControlFlow::Continue
    });

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

    // Present the window
    window.present();
}
