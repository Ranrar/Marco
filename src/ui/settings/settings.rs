// Settings structure
use gtk4::prelude::*;
use gtk4::{Dialog, Window, Notebook, Button, Box as GtkBox, Orientation, Align, Label};

use crate::ui::settings::tabs;

pub struct Settings {
    pub theme: String,
    pub font_size: i32,
}

impl Settings {
    pub fn new() -> Self {
        Settings {
            theme: "light".to_string(),
            font_size: 12,
        }
    }
}

pub fn show_settings_dialog(parent: &Window) {
    let dialog = Dialog::builder()
        .transient_for(parent)
        .modal(true)
        .title("Settings")
        .build();

    let notebook = Notebook::new();
    notebook.set_tab_pos(gtk4::PositionType::Top);

    // Add each tab
    notebook.append_page(&tabs::editor::build_editor_tab(),     Some(&Label::new(Some("Editor"))));
    notebook.append_page(&tabs::layout::build_layout_tab(),     Some(&Label::new(Some("Layout"))));
    notebook.append_page(&tabs::appearance::build_appearance_tab(), Some(&Label::new(Some("Appearance"))));
    notebook.append_page(&tabs::language::build_language_tab(), Some(&Label::new(Some("Language"))));
    notebook.append_page(&tabs::advanced::build_advanced_tab(), Some(&Label::new(Some("Advanced"))));
    notebook.append_page(&tabs::plugins::build_plugin_tab(),     Some(&Label::new(Some("Plugins"))));

    // Layout: notebook + close button at bottom right
    let content_box = GtkBox::new(Orientation::Vertical, 0);
    content_box.append(&notebook);

    let button_box = GtkBox::new(Orientation::Horizontal, 0);
    button_box.set_halign(Align::End);
    let close_button = Button::with_label("Close");
    let dialog_clone = dialog.clone();
    close_button.connect_clicked(move |_| dialog_clone.close());
    button_box.append(&close_button);
    content_box.append(&button_box);

    dialog.set_default_size(700, 600); // Make dialog wider
    dialog.set_child(Some(&content_box));
    dialog.present();
}