use gtk4::prelude::*;
use gtk4::Box;

pub fn build_language_tab() -> Box {
    use gtk4::{Align, Box as GtkBox, DropDown, Label, Orientation, StringList, PropertyExpression, StringObject, Expression};

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("settings-tab-language");
    container.set_margin_top(24);
    container.set_margin_bottom(24);
    container.set_margin_start(32);
    container.set_margin_end(32);

    // Language (Dropdown)
    let lang_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let lang_header = Label::new(Some("Language"));
    lang_header.set_markup("<b>Language</b>");
    lang_header.set_halign(Align::Start);
    lang_header.set_xalign(0.0);

    let lang_spacer = GtkBox::new(Orientation::Horizontal, 0);
    lang_spacer.set_hexpand(true);

    // Create language dropdown with automatic checkmarks
    let language_options = [
        "System Default", "English", "Dansk", "Deutsch", "Français", "العربية", "日本語"
    ];
    
    // Step 1: Create StringList from language options
    let language_string_list = StringList::new(&language_options);
    
    // Step 2: Create PropertyExpression for string matching (required for DropDown)
    let language_expression = PropertyExpression::new(
        StringObject::static_type(),
        None::<Expression>,
        "string",
    );
    
    // Step 3: Create DropDown with automatic checkmarks
    let lang_combo = DropDown::new(Some(language_string_list), Some(language_expression));
    lang_combo.set_selected(0); // Default to "System Default"
    lang_combo.set_halign(Align::End);

    lang_hbox.append(&lang_header);
    lang_hbox.append(&lang_spacer);
    lang_hbox.append(&lang_combo);
    lang_hbox.set_margin_bottom(4);
    container.append(&lang_hbox);

    // Description text under header
    let lang_description = Label::new(Some(
        "Select the language used for menus, labels, and tooltips.",
    ));
    lang_description.set_halign(Align::Start);
    lang_description.set_xalign(0.0);
    lang_description.set_wrap(true);
    lang_description.add_css_class("dim-label");
    lang_description.set_margin_bottom(12);
    container.append(&lang_description);

    container
}
