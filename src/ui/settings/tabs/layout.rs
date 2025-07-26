use gtk4::prelude::*;
use gtk4::Box;

pub fn build_layout_tab() -> Box {
    use gtk4::{Label, ComboBoxText, Scale, Adjustment, Switch, Box as GtkBox, Orientation, Align};

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

    // View Mode (Dropdown)
    let view_mode_combo = ComboBoxText::new();
    view_mode_combo.append_text("HTML Preview");
    view_mode_combo.append_text("Source Code");
    view_mode_combo.set_active(Some(0));
    let view_mode_row = add_row(
        "View Mode",
        "Choose the default mode for previewing Markdown content.",
        view_mode_combo.upcast_ref(),
    );
    container.append(&view_mode_row);

    // Sync Scrolling (Toggle)
    let sync_scroll_switch = Switch::new();
    let sync_scroll_row = add_row(
        "Sync Scrolling",
        "Synchronize scrolling between the editor and the preview pane.",
        sync_scroll_switch.upcast_ref(),
    );
    container.append(&sync_scroll_row);

    // Editor/View Split (Slider)
    let split_adj = Adjustment::new(60.0, 10.0, 90.0, 1.0, 0.0, 0.0);
    let split_slider = Scale::new(Orientation::Horizontal, Some(&split_adj));
    split_slider.set_draw_value(false);
    split_slider.set_hexpand(true);
    split_slider.set_round_digits(0);
    split_slider.set_width_request(300);
    // Add marks for common splits
    for mark in [10, 25, 40, 50, 60, 75, 90].iter() {
        split_slider.add_mark(*mark as f64, gtk4::PositionType::Bottom, Some(&format!("{}%", mark)));
    }
    let split_vbox = GtkBox::new(Orientation::Vertical, 2);
    let split_hbox = GtkBox::new(Orientation::Horizontal, 0);
    let split_title = bold_label("Editor/View Split");
    let split_spacer = GtkBox::new(Orientation::Horizontal, 0);
    split_spacer.set_hexpand(true);
    split_hbox.append(&split_title);
    split_hbox.append(&split_spacer);
    // No spinbutton for now, just slider
    split_hbox.set_hexpand(true);
    split_vbox.append(&split_hbox);
    split_vbox.append(&desc_label("Adjust how much horizontal space the editor takes."));
    split_slider.set_halign(Align::Start);
    split_vbox.append(&split_slider);
    split_vbox.set_spacing(2);
    split_vbox.set_margin_bottom(24);
    container.append(&split_vbox);

    // Show Line Numbers (Toggle)
    let line_numbers_switch = Switch::new();
    let line_numbers_row = add_row(
        "Show Line Numbers",
        "Display line numbers in the editor gutter.",
        line_numbers_switch.upcast_ref(),
    );
    container.append(&line_numbers_row);

    // Text Direction (Dropdown)
    let text_dir_combo = ComboBoxText::new();
    text_dir_combo.append_text("Left-to-Right (LTR)");
    text_dir_combo.append_text("Right-to-Left (RTL)");
    text_dir_combo.set_active(Some(0));
    let text_dir_row = add_row(
        "Text Direction",
        "Switch between Left-to-Right (LTR) and Right-to-Left (RTL) layout.",
        text_dir_combo.upcast_ref(),
    );
    container.append(&text_dir_row);

    container
}
