use gtk4::prelude::*;
use gtk4::Box;

// Import unified helper
use super::helpers::add_setting_row;

pub fn build_language_tab() -> Box {
    use gtk4::{
        Box as GtkBox, DropDown, Expression, Orientation, PropertyExpression, StringList,
        StringObject,
    };

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("marco-settings-tab");

    // Language (Dropdown)
    // Create language dropdown with automatic checkmarks
    let language_options = [
        "System Default",
        "English",
        "Dansk",
        "Deutsch",
        "Français",
        "العربية",
        "日本語",
    ];

    // Step 1: Create StringList from language options
    let language_string_list = StringList::new(&language_options);

    // Step 2: Create PropertyExpression for string matching (required for DropDown)
    let language_expression =
        PropertyExpression::new(StringObject::static_type(), None::<Expression>, "string");

    // Step 3: Create DropDown with automatic checkmarks
    let lang_combo = DropDown::new(Some(language_string_list), Some(language_expression));
    lang_combo.add_css_class("marco-dropdown");
    lang_combo.set_selected(0); // Default to "System Default"

    // Create language row using unified helper (first and only row)
    let lang_row = add_setting_row(
        "Language",
        "Select the language used for menus, labels, and tooltips.",
        &lang_combo,
        true, // First row - no top margin
    );
    container.append(&lang_row);

    container
}
