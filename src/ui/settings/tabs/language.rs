use gtk4::prelude::*;
use gtk4::Box;

pub fn build_language_tab() -> Box {
    use gtk4::{Label, ComboBoxText, Box as GtkBox, Orientation, Align};

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.set_margin_top(24);
    container.set_margin_bottom(24);
    container.set_margin_start(32);
    container.set_margin_end(32);

    // Helper for bold label
    let bold_label = |text: &str| {
        let l = Label::new(Some(text));
        l.set_halign(Align::Start);
        l.set_xalign(0.0);
        l.set_markup(&format!("<b>{}</b>", glib::markup_escape_text(text)));
        l
    };

    // Helper for normal description
    let desc_label = |text: &str| {
        let l = Label::new(Some(text));
        l.set_halign(Align::Start);
        l.set_xalign(0.0);
        l.set_wrap(true);
        l
    };

    // Helper for a row: title, desc, control right-aligned, with extra vertical space
    let add_row = |title: &str, desc: &str, control: &gtk4::Widget| {
        let vbox = GtkBox::new(Orientation::Vertical, 2);
        let hbox = GtkBox::new(Orientation::Horizontal, 0);
        let title_label = bold_label(title);
        let spacer = GtkBox::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        let control = control.clone();
        hbox.append(&title_label);
        hbox.append(&spacer); // Expanding spacer
        hbox.append(&control);
        control.set_halign(Align::End);
        hbox.set_hexpand(true);
        vbox.append(&hbox);
        let desc = desc_label(desc);
        vbox.append(&desc);
        vbox.set_spacing(4);
        vbox.set_margin_bottom(24);
        vbox
    };

    // Language selection (Dropdown)
    let lang_combo = ComboBoxText::new();
    lang_combo.append_text("System Default");
    lang_combo.append_text("English");
    lang_combo.append_text("Dansk");
    lang_combo.append_text("Deutsch");
    lang_combo.append_text("Français");
    lang_combo.append_text("العربية");
    lang_combo.append_text("日本語");
    lang_combo.set_active(Some(0));
    let lang_row = add_row(
        "Language",
        "Select the language used for menus, labels, and tooltips.",
        lang_combo.upcast_ref(),
    );
    container.append(&lang_row);

    container
}
