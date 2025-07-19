// Stripped-down UI structure modules

mod footer;
mod menu;
mod settings;
mod theme;
mod toolbar;
pub mod ui;
pub mod editor;
pub mod viewer;
pub mod logic;


use gtk4::prelude::*;
use gtk4::{glib, Application, ApplicationWindow, Box, Orientation, HeaderBar, Button};
use crate::editor::editor::create_editor_with_preview;
use crate::logic::ast::blocks_and_inlines::{Block, LeafBlock};
use gtk4::Align;

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

    // Create a HeaderBar
    let header_bar = HeaderBar::builder()
        .title_widget(&gtk4::Label::new(Some("Marco")))
        .show_title_buttons(true)
        .build();


    // Create a Button with a settings icon
    let settings_button = Button::builder()
        .icon_name("emblem-system-symbolic")
        .valign(Align::Center)
        .build();

    // Connect Button to show settings dialog
    let win_clone = window.clone();
    settings_button.connect_clicked(move |_| {
        settings::show_settings_dialog(win_clone.upcast_ref());
    });

    // Add Button to the end (right) of the header bar
    header_bar.pack_end(&settings_button);

    // Set the header bar as the window's titlebar
    window.set_titlebar(Some(&header_bar));

    // Create main vertical box layout
    let main_box = Box::new(Orientation::Vertical, 0);

    // Create basic UI components (structure only)
    let menu_bar = menu::main_menu_structure();
    let toolbar = toolbar::create_toolbar_structure();
    // Create a dummy AST for initial display
    let ast = Block::Leaf(LeafBlock::Paragraph(vec![], None));
    let split = create_editor_with_preview(&ast);
    let footer = footer::create_footer_structure();

    // Add components to main layout
    main_box.append(&menu_bar);
    main_box.append(&toolbar);
    main_box.append(&split);
    main_box.append(&footer);

    // Set editor area to expand
    split.set_vexpand(true);

    // Add main box to window
    window.set_child(Some(&main_box));

    // Present the window
    window.present();
}