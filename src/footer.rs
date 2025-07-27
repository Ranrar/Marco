use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation};

pub struct FooterLabels {
    pub word_count: Label,
    pub char_count: Label,
    pub cursor_pos: Label,
    pub formatting: Label,
}

pub fn create_footer_structure() -> Box {
    let footer_box = Box::new(Orientation::Horizontal, 10);
    footer_box.add_css_class("footer");
    footer_box.set_margin_top(5);
    footer_box.set_margin_bottom(5);
    footer_box.set_margin_start(10);
    footer_box.set_margin_end(10);

    // Formatting label (left side)
    let formatting_label = Label::new(Some("Format:"));
    formatting_label.add_css_class("footer-formatting");
    formatting_label.set_halign(gtk4::Align::Start);
    formatting_label.set_xalign(0.0);
    footer_box.append(&formatting_label);

    // Spacer to push items to the sides
    let spacer = Label::new(None);
    spacer.add_css_class("footer-spacer");
    spacer.set_hexpand(true);
    footer_box.append(&spacer);

    // Info labels (right side)
    let word_count_label = Label::new(Some("Words: 0"));
    word_count_label.add_css_class("footer-word-count");
    footer_box.append(&word_count_label);

    let char_count_label = Label::new(Some("Characters: 0"));
    char_count_label.add_css_class("footer-char-count");
    footer_box.append(&char_count_label);

    let cursor_pos_label = Label::new(Some("Line: 1, Col: 1"));
    cursor_pos_label.add_css_class("footer-cursor-pos");
    footer_box.append(&cursor_pos_label);

    footer_box
}

pub fn create_footer() -> (Box, FooterLabels) {
    let footer_box = Box::new(Orientation::Horizontal, 10);
    footer_box.add_css_class("footer");
    footer_box.set_margin_top(5);
    footer_box.set_margin_bottom(5);
    footer_box.set_margin_start(10);
    footer_box.set_margin_end(10);

    // Formatting label (left side)
    let formatting_label = Label::new(Some("Format:"));
    formatting_label.add_css_class("footer-formatting");
    formatting_label.set_halign(gtk4::Align::Start);
    formatting_label.set_xalign(0.0);
    footer_box.append(&formatting_label);

    // Spacer to push items to the sides
    let spacer = Label::new(None);
    spacer.add_css_class("footer-spacer");
    spacer.set_hexpand(true);
    footer_box.append(&spacer);

    // Info labels (right side)
    let word_count_label = Label::new(Some("Words: 0"));
    word_count_label.add_css_class("footer-word-count");
    footer_box.append(&word_count_label);

    let char_count_label = Label::new(Some("Characters: 0"));
    char_count_label.add_css_class("footer-char-count");
    footer_box.append(&char_count_label);

    let cursor_pos_label = Label::new(Some("Line: 1, Col: 1"));
    cursor_pos_label.add_css_class("footer-cursor-pos");
    footer_box.append(&cursor_pos_label);

    let labels = FooterLabels {
        word_count: word_count_label,
        char_count: char_count_label,
        cursor_pos: cursor_pos_label,
        formatting: formatting_label,
    };

    (footer_box, labels)
}