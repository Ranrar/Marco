// Settings structure
use gtk4::prelude::*;
use gtk4::{Dialog, Window};

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
    let dialog = Dialog::with_buttons(
        Some("Settings"),
        Some(parent),
        gtk4::DialogFlags::MODAL,
        &[("OK", gtk4::ResponseType::Ok), ("Cancel", gtk4::ResponseType::Cancel)],
    );
    
    dialog.present();
}