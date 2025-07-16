// Stripped-down UI structure modules
mod footer;
mod menu;
mod settings;
mod theme;
mod toolbar;
pub mod ui;
pub mod editor;
pub mod viewer;

use gtk4::prelude::*;
use gtk4::{glib, Application, ApplicationWindow, Box, Orientation};

const APP_ID: &str = "com.example.Marco";

fn main() -> glib::ExitCode {
    // Initialize GTK4 application
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| build_ui(app));

    let no_args: [&str; 0] = [];
    app.run_with_args(&no_args)
}

fn build_ui(app: &Application) {
    // Create the main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Marco")
        .default_width(1200)
        .default_height(800)
        .build();

    // Create main vertical box layout
    let main_box = Box::new(Orientation::Vertical, 0);

    // Create basic UI components (structure only)
    let menu_bar = menu::create_menu_structure();
    let toolbar = toolbar::create_toolbar_structure();
    let editor_area = editor::create_editor_structure();
    let footer = footer::create_footer_structure();

    // Add components to main layout
    main_box.append(&menu_bar);
    main_box.append(&toolbar);
    main_box.append(&editor_area);
    main_box.append(&footer);

    // Set editor area to expand
    editor_area.set_vexpand(true);

    // Add main box to window
    window.set_child(Some(&main_box));

    // Present the window
    window.present();
}