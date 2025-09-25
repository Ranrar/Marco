use gtk4::prelude::*;
use gtk4::Box;

pub fn build_language_tab() -> Box {
    use gtk4::{Align, Box as GtkBox, ComboBoxText, Label, Orientation};

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

    let lang_combo = ComboBoxText::new();
    lang_combo.append_text("System Default");
    lang_combo.append_text("English");
    lang_combo.append_text("Dansk");
    lang_combo.append_text("Deutsch");
    lang_combo.append_text("Français");
    lang_combo.append_text("العربية");
    lang_combo.append_text("日本語");
    lang_combo.set_active(Some(0));
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
