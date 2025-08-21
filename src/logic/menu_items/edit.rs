// Format menu structure
use gtk4::{gio};

pub fn create_format_menu() -> gio::Menu {
    let menu = gio::Menu::new();
    menu.append(Some("Bold"), Some("app.bold"));
    menu.append(Some("Italic"), Some("app.italic"));
    menu
}