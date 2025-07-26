use gtk4::prelude::*;
use gtk4::Box;

pub fn build_appearance_tab() -> Box {
    use gtk4::{Label, ComboBoxText, Scale, Adjustment, Button, Box as GtkBox, Orientation, Align};

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

    // Application Theme (Dropdown)
    let app_theme_combo = ComboBoxText::new();
    app_theme_combo.append_text("Light");
    app_theme_combo.append_text("Dark");
    app_theme_combo.append_text("System Default");
    app_theme_combo.set_active(Some(2));
    let app_theme_row = add_row(
        "Application Theme",
        "Choose between light and dark modes for the user interface.",
        app_theme_combo.upcast_ref(),
    );
    container.append(&app_theme_row);

    // HTML Preview Theme (Dropdown)
    let preview_theme_combo = ComboBoxText::new();
    preview_theme_combo.append_text("GitHub");
    preview_theme_combo.append_text("Minimal");
    preview_theme_combo.append_text("Typora");
    preview_theme_combo.append_text("Dracula");
    preview_theme_combo.append_text("Paper");
    preview_theme_combo.append_text("Custom");
    preview_theme_combo.set_active(Some(0));
    let preview_theme_row = add_row(
        "HTML Preview Theme",
        "Select a CSS theme for rendered Markdown preview.",
        preview_theme_combo.upcast_ref(),
    );
    container.append(&preview_theme_row);

    // Custom CSS for Preview (Button)
    let custom_css_button = Button::with_label("Open CSS Themes Folder");
    let custom_css_row = add_row(
        "Custom CSS for Preview",
        "Add your own custom CSS to override the preview style.",
        custom_css_button.upcast_ref(),
    );
    container.append(&custom_css_row);

    // UI Font (Dropdown)
    let ui_font_combo = ComboBoxText::new();
    ui_font_combo.append_text("System Default");
    ui_font_combo.append_text("Sans");
    ui_font_combo.append_text("Serif");
    ui_font_combo.append_text("Monospace");
    ui_font_combo.set_active(Some(0));
    let ui_font_row = add_row(
        "UI Font",
        "Customize the font used in the application's user interface (menus, sidebars).",
        ui_font_combo.upcast_ref(),
    );
    container.append(&ui_font_row);

    // UI Font Size (Slider)
    let ui_font_size_adj = Adjustment::new(14.0, 10.0, 24.0, 1.0, 0.0, 0.0);
    let ui_font_size_slider = Scale::new(Orientation::Horizontal, Some(&ui_font_size_adj));
    ui_font_size_slider.set_draw_value(false);
    ui_font_size_slider.set_hexpand(true);
    ui_font_size_slider.set_round_digits(0);
    ui_font_size_slider.set_width_request(300);
    for size in 10..=24 {
        ui_font_size_slider.add_mark(size as f64, gtk4::PositionType::Bottom, Some(&size.to_string()));
    }
    let ui_font_size_vbox = GtkBox::new(Orientation::Vertical, 2);
    let ui_font_size_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let ui_font_size_title = bold_label("UI Font Size");
    let ui_font_size_spacer = GtkBox::new(Orientation::Horizontal, 0);
    ui_font_size_spacer.set_hexpand(true);
    ui_font_size_hbox.append(&ui_font_size_title);
    ui_font_size_hbox.append(&ui_font_size_spacer);
    // No spinbutton for now
    ui_font_size_hbox.set_hexpand(true);
    ui_font_size_vbox.append(&ui_font_size_hbox);
    ui_font_size_vbox.append(&desc_label("Customize the font size used in the application's user interface (menus, sidebars)."));
    ui_font_size_slider.set_halign(Align::Start);
    ui_font_size_vbox.append(&ui_font_size_slider);
    ui_font_size_vbox.set_spacing(2);
    ui_font_size_vbox.set_margin_bottom(24);
    container.append(&ui_font_size_vbox);

    // Border & Contrast Style (Dropdown)
    let border_combo = ComboBoxText::new();
    border_combo.append_text("Default");
    border_combo.append_text("High Contrast");
    border_combo.set_active(Some(0));
    let border_row = add_row(
        "Border & Contrast Style",
        "Choose between subtle or high-contrast borders for accessibility.",
        border_combo.upcast_ref(),
    );
    container.append(&border_row);

    container
}
