use gtk4::prelude::*;
use gtk4::Box;

pub fn build_editor_tab() -> Box {
    use gtk4::{Label, ComboBoxText, Scale, Adjustment, Switch, Box as GtkBox, Orientation, Align, SpinButton};

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
        vbox.set_margin_bottom(24); // More space between functions
        vbox
    };

    // Editor Font (Dropdown)
    let font_combo = ComboBoxText::new();
    font_combo.append_text("Monospace");
    font_combo.append_text("Proportional");
    font_combo.set_active(Some(0));
    let font_row = add_row(
        "Editor Font",
        "Select the font used in the editor.",
        font_combo.upcast_ref(),
    );
    container.append(&font_row);

    // Font Size (Slider/SpinButton)
    let font_size_adj = Adjustment::new(14.0, 10.0, 24.0, 1.0, 0.0, 0.0);
    let font_size_slider = Scale::new(Orientation::Horizontal, Some(&font_size_adj));
    font_size_slider.set_draw_value(false);
    font_size_slider.set_hexpand(true);
    font_size_slider.set_round_digits(0); // Discrete steps
    font_size_slider.set_value_pos(gtk4::PositionType::Right);
    font_size_slider.set_increments(1.0, 1.0);
    for size in 10..=24 {
        font_size_slider.add_mark(size as f64, gtk4::PositionType::Bottom, Some(&size.to_string()));
    }
    let font_size_spin = SpinButton::new(Some(&font_size_adj), 1.0, 0);
    let font_size_vbox = GtkBox::new(Orientation::Vertical, 2);
    let font_size_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let font_size_title = bold_label("Font Size");
    let font_size_spacer = GtkBox::new(Orientation::Horizontal, 0);
    font_size_spacer.set_hexpand(true);
    font_size_hbox.append(&font_size_title);
    font_size_hbox.append(&font_size_spacer);
    font_size_hbox.append(&font_size_spin);
    font_size_spin.set_halign(Align::End);
    font_size_hbox.set_hexpand(true);
    font_size_vbox.append(&font_size_hbox);
    font_size_vbox.append(&desc_label("Set the font size for the editor text (10-24 px)."));
    // Slider left-aligned under description
    font_size_slider.set_halign(Align::Start);
    font_size_slider.set_width_request(300);
    font_size_vbox.append(&font_size_slider);
    font_size_vbox.set_spacing(2);
    font_size_vbox.set_margin_bottom(8);
    container.append(&font_size_vbox);

    // Line Height (Slider/SpinButton)
    let line_height_adj = Adjustment::new(1.4, 1.0, 2.0, 0.05, 0.0, 0.0);
    let line_height_slider = Scale::new(Orientation::Horizontal, Some(&line_height_adj));
    line_height_slider.set_draw_value(false);
    line_height_slider.set_hexpand(true);
    for mark in [1.0, 1.2, 1.4, 1.6, 1.8, 2.0].iter() {
        line_height_slider.add_mark(*mark, gtk4::PositionType::Bottom, Some(&format!("{:.1}", mark)));
    }
    let line_height_spin = SpinButton::new(Some(&line_height_adj), 0.05, 2);
    let line_height_vbox = GtkBox::new(Orientation::Vertical, 2);
    let line_height_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let line_height_title = bold_label("Line Height");
    let line_height_spacer = GtkBox::new(Orientation::Horizontal, 0);
    line_height_spacer.set_hexpand(true);
    line_height_hbox.append(&line_height_title);
    line_height_hbox.append(&line_height_spacer);
    line_height_hbox.append(&line_height_spin);
    line_height_spin.set_halign(Align::End);
    line_height_hbox.set_hexpand(true);
    line_height_vbox.append(&line_height_hbox);
    line_height_vbox.append(&desc_label("Adjust the vertical spacing between lines."));
    // Slider left-aligned under description
    line_height_slider.set_halign(Align::Start);
    line_height_slider.set_width_request(300);
    line_height_vbox.append(&line_height_slider);
    line_height_vbox.set_spacing(2);
    line_height_vbox.set_margin_bottom(8);
    container.append(&line_height_vbox);

    // Line Wrapping (Toggle)
    let line_wrap_switch = Switch::new();
    let line_wrap_row = add_row(
        "Line Wrapping",
        "Wrap long lines to fit within the editor window.",
        line_wrap_switch.upcast_ref(),
    );
    container.append(&line_wrap_row);

    // Auto Pairing (Toggle)
    let auto_pair_switch = Switch::new();
    let auto_pair_row = add_row(
        "Auto Pairing",
        "Automatically insert closing characters for **, [], (), and backticks.",
        auto_pair_switch.upcast_ref(),
    );
    container.append(&auto_pair_row);

    // Show Invisible Characters (Toggle)
    let show_invis_switch = Switch::new();
    let show_invis_row = add_row(
        "Show Invisible Characters",
        "Display tabs, spaces, and newlines visually in the editor.",
        show_invis_switch.upcast_ref(),
    );
    container.append(&show_invis_row);

    // Convert Tabs to Spaces (Toggle)
    let tabs_to_spaces_switch = Switch::new();
    let tabs_to_spaces_row = add_row(
        "Convert Tabs to Spaces",
        "Replace tab characters with spaces.",
        tabs_to_spaces_switch.upcast_ref(),
    );
    container.append(&tabs_to_spaces_row);

    // Syntax Colors (Toggle)
    let syntax_colors_switch = Switch::new();
    let syntax_colors_row = add_row(
        "Syntax Colors",
        "Enable or disable syntax-based color highlighting for Markdown.",
        syntax_colors_switch.upcast_ref(),
    );
    container.append(&syntax_colors_row);

    // Enable Markdown Linting (Toggle)
    let linting_switch = Switch::new();
    let linting_row = add_row(
        "Enable Markdown Linting",
        "Check for Markdown syntax issues and style problems.",
        linting_switch.upcast_ref(),
    );
    container.append(&linting_row);

    container
}
