// Insert menu structure
use gtk4::{gio};

pub fn create_insert_menu() -> gio::Menu {
    let menu = gio::Menu::new();
    menu.append(Some("Link"), Some("app.insert_link"));
    menu.append(Some("Image"), Some("app.insert_image"));
    menu
}