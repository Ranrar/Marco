// View menu structure
use gtk4::gio;

use crate::components::language::MenuTranslations;

pub fn create_view_menu(translations: &MenuTranslations) -> gio::Menu {
    let menu = gio::Menu::new();
    menu.append(Some(&translations.html_preview), Some("app.view_html"));
    menu.append(Some(&translations.code_view), Some("app.view_code"));
    menu
}