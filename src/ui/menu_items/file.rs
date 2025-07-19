// Basic menu structure
use gtk4::{gio};

pub fn create_basic_menu() -> gio::Menu {
    let menu = gio::Menu::new();
    menu.append(Some("Basic Item"), Some("app.basic"));
    menu
}