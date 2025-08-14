/// Updates the row and column label
pub fn update_cursor_pos(labels: &FooterLabels, row: usize, col: usize) {
    labels.cursor_pos.set_text(&format!("Row {}, Column {}", row, col));
}

/// Updates the line count label
pub fn update_line_count(labels: &FooterLabels, lines: usize) {
    labels.line_count.set_text(&format!("{} line{}", lines, if lines == 1 { "" } else { "s" }));
}

/// Updates the encoding label
pub fn update_encoding(labels: &FooterLabels, encoding: &str) {
    labels.encoding.set_text(encoding);
}

/// Updates the insert/overwrite mode label
pub fn update_insert_mode(labels: &FooterLabels, is_insert: bool) {
    labels.insert_mode.set_text(if is_insert { "INS" } else { "OVR" });
}
use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation};
use crate::logic::parser::{parse_line_syntax, MarkdownSyntaxMap};

#[derive(Clone)]
pub struct FooterLabels {
    pub cursor_pos: Label,
    pub line_count: Label,
    pub encoding: Label,
    pub insert_mode: Label,
    pub formatting: Label,
    pub word_count: Label,
    pub char_count: Label,
}

/// Updates the formatting label with the Markdown syntax trace for the active line
pub fn update_syntax_trace(labels: &FooterLabels, line: &str, syntax_map: &MarkdownSyntaxMap) {
    let chain = parse_line_syntax(line, syntax_map);
    let display = if chain.is_empty() {
        "Plain text".to_string()
    } else {
        chain.join(" → ")
    };
    labels.formatting.set_text(&display);
}
/// Updates the word count label
pub fn update_word_count(labels: &FooterLabels, words: usize) {
    labels.word_count.set_text(&format!("Words: {}", words));
}

/// Updates the character count label
pub fn update_char_count(labels: &FooterLabels, chars: usize) {
    labels.char_count.set_text(&format!("Characters: {}", chars));
}

pub fn create_footer() -> (Box, FooterLabels) {
    let footer_box = Box::new(Orientation::Horizontal, 10);
    footer_box.set_margin_top(5);
    footer_box.set_margin_bottom(5);
    footer_box.set_margin_start(10);
    footer_box.set_margin_end(10);

    // Formatting label (left side)
    let formatting_label = Label::new(Some("Format:"));
    formatting_label.set_halign(gtk4::Align::Start);
    formatting_label.set_xalign(0.0);
    footer_box.append(&formatting_label);

    // Spacer to push items to the sides
    let spacer = Label::new(None);
    spacer.set_hexpand(true);
    footer_box.append(&spacer);

    // Info labels (right side)
    let word_count_label = Label::new(Some("Words: 0"));
    footer_box.append(&word_count_label);

    let char_count_label = Label::new(Some("Characters: 0"));
    footer_box.append(&char_count_label);

    let cursor_pos_label = Label::new(Some("Row 1, Column 1"));
    footer_box.append(&cursor_pos_label);

    let line_count_label = Label::new(Some("1 line"));
    footer_box.append(&line_count_label);

    let encoding_label = Label::new(Some("UTF-8"));
    footer_box.append(&encoding_label);

    let insert_mode_label = Label::new(Some("INS"));
    footer_box.append(&insert_mode_label);

    let labels = FooterLabels {
        cursor_pos: cursor_pos_label,
        line_count: line_count_label,
        encoding: encoding_label,
        insert_mode: insert_mode_label,
        formatting: formatting_label,
        word_count: word_count_label,
        char_count: char_count_label,
    };

    (footer_box, labels)
}