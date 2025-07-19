// View menu structure
use gtk4::{gio};

pub fn create_view_menu() -> gio::Menu {
    let menu = gio::Menu::new();
    menu.append(Some("HTML Preview"), Some("app.view_html"));
    menu.append(Some("Code View"), Some("app.view_code"));
    menu
}