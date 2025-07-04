use gtk4::prelude::*;
use gtk4::{Box, Orientation, Label};
use crate::language;

#[derive(Clone)]
pub struct FooterLabels {
    pub status: Label,
    pub word_count: Label,
    pub char_count: Label,
    pub cursor_pos: Label,
}

pub fn create_footer() -> (Box, FooterLabels) {
    let footer_box = Box::new(Orientation::Horizontal, 10);
    footer_box.set_margin_top(5);
    footer_box.set_margin_bottom(5);
    footer_box.set_margin_start(10);
    footer_box.set_margin_end(10);
    
    // Status label (left side)
    let status_label = Label::new(Some(&language::tr("footer.ready")));
    status_label.set_halign(gtk4::Align::Start);
    footer_box.append(&status_label);
    
    // Spacer to push items to the sides
    let spacer = Label::new(None);
    spacer.set_hexpand(true);
    footer_box.append(&spacer);
    
    // Info labels (right side)
    let word_count_label = Label::new(Some("Words: 0"));
    footer_box.append(&word_count_label);
    
    let char_count_label = Label::new(Some("Characters: 0"));
    footer_box.append(&char_count_label);
    
    let cursor_pos_label = Label::new(Some("Line: 1, Col: 1"));
    footer_box.append(&cursor_pos_label);
    
    let labels = FooterLabels {
        status: status_label,
        word_count: word_count_label,
        char_count: char_count_label,
        cursor_pos: cursor_pos_label,
    };
    
    (footer_box, labels)
}
