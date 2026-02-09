//! Footer module for Marco markdown editor
//!
//! This module provides footer functionality for displaying various editor states including:
//! - Cursor position (row and column)
//! - Line count
//! - Word and character counts
//! - Current encoding
//! - Insert/overwrite mode
//! - Markdown syntax trace for the current line
//!
//! ## Threading Safety
//! All footer update functions are designed to be thread-safe. They use `set_label_text`
//! helper which automatically detects whether it's running on the main GTK thread and
//! schedules updates using `glib::idle_add_local` if necessary.
//!
//! ## Usage
//! Footer updates can be triggered individually using specific update functions, or in
//! batch using `apply_footer_update` with a `FooterUpdate::Snapshot`.

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::components::language::FooterTranslations;

static UPDATE_VIS_COUNTER: AtomicUsize = AtomicUsize::new(0);

// Gate footer debug output behind an env var so normal runs are quiet.
#[macro_export]
macro_rules! footer_dbg {
    ($($arg:tt)*) => {{
        if std::env::var("MARCO_DEBUG_FOOTER").is_ok() {
            eprintln!($($arg)*);
        }
    }};
}

/// Message type used to update the footer from any thread via a MainContext channel
#[derive(Debug)]
pub enum FooterUpdate {
    Snapshot {
        row: usize,
        col: usize,
        // lines removed
        words: usize,
        chars: usize,
        encoding: String,
        is_insert: bool,
    },
}

#[derive(Clone)]
pub struct FooterLabels {
    pub cursor_row: Label,
    pub cursor_col: Label,
    pub encoding: Label,
    pub insert_mode: Label,
    pub word_count: Label,
    pub char_count: Label,
    pub row_label: RefCell<String>,
    pub column_label: RefCell<String>,
    pub words_label: RefCell<String>,
    pub characters_label: RefCell<String>,
    pub ins_label: RefCell<String>,
    pub ovr_label: RefCell<String>,
    pub encoding_label: RefCell<String>,
}

/// Update the row label independently
pub fn update_cursor_row(labels: &FooterLabels, row: usize) {
    let text = format!("{}: {}", labels.row_label.borrow(), row);
    footer_dbg!("[footer] update_cursor_row called: {}", text);
    set_label_text(&labels.cursor_row, text);
}

/// Update the column label independently
pub fn update_cursor_col(labels: &FooterLabels, col: usize) {
    let text = format!("{}: {}", labels.column_label.borrow(), col);
    footer_dbg!("[footer] update_cursor_col called: {}", text);
    set_label_text(&labels.cursor_col, text);
}

// line count removed: no-op omitted

/// Updates the encoding label
pub fn update_encoding(labels: &FooterLabels, encoding: &str) {
    let enc = encoding.to_string();
    footer_dbg!("[footer] update_encoding called: {}", enc);
    set_label_text(&labels.encoding, enc);
}

/// Updates the insert/overwrite mode label
pub fn update_insert_mode(labels: &FooterLabels, is_insert: bool) {
    let text = if is_insert {
        labels.ins_label.borrow()
    } else {
        labels.ovr_label.borrow()
    };
    footer_dbg!("[footer] update_insert_mode called: {}", text);
    set_label_text(&labels.insert_mode, text.to_string());
}

/// Updates the word count label
pub fn update_word_count(labels: &FooterLabels, words: usize) {
    let text = format!("{}: {}", labels.words_label.borrow(), words);
    footer_dbg!("[footer] update_word_count called: {}", text);
    set_label_text(&labels.word_count, text);
}

/// Updates the character count label
pub fn update_char_count(labels: &FooterLabels, chars: usize) {
    let text = format!("{}: {}", labels.characters_label.borrow(), chars);
    footer_dbg!("[footer] update_char_count called: {}", text);
    set_label_text(&labels.char_count, text);
}

/// Update translation labels used by the footer and refresh static text.
pub fn update_footer_translations(
    labels: &FooterLabels,
    translations: &FooterTranslations,
    is_insert: bool,
) {
    *labels.row_label.borrow_mut() = translations.row.clone();
    *labels.column_label.borrow_mut() = translations.column.clone();
    *labels.words_label.borrow_mut() = translations.words.clone();
    *labels.characters_label.borrow_mut() = translations.characters.clone();
    *labels.ins_label.borrow_mut() = translations.ins.clone();
    *labels.ovr_label.borrow_mut() = translations.ovr.clone();
    *labels.encoding_label.borrow_mut() = translations.encoding_utf8.clone();

    update_encoding(labels, &translations.encoding_utf8);
    update_insert_mode(labels, is_insert);
}

/// Apply a FooterUpdate snapshot to the labels. Must be called on main context.
pub fn apply_footer_update(labels: &FooterLabels, update: FooterUpdate) {
    match update {
        FooterUpdate::Snapshot {
            row,
            col,
            /*lines,*/ words,
            chars,
            encoding,
            is_insert,
        } => {
            update_cursor_row(labels, row);
            update_cursor_col(labels, col);
            update_word_count(labels, words);
            update_char_count(labels, chars);
            update_encoding(labels, &encoding);
            update_insert_mode(labels, is_insert);
        }
    }
}

/// Helper: set a Label's text on the main GTK context, scheduling if needed.
/// This function ensures thread safety and provides consistent label updating.
fn set_label_text(label: &Label, text: String) {
    let mut final_text = text.clone();

    // If debug env var set, append a small counter so updates are visually detectable
    if std::env::var("MARCO_DEBUG_FOOTER_VIS").is_ok() {
        let n = UPDATE_VIS_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
        final_text = format!("{} [{}]", text, n);
    }

    let use_markup = std::env::var("MARCO_DEBUG_FOOTER_VIS").is_ok();

    // Check if we're on the main thread
    if glib::MainContext::default().is_owner() {
        // We're on the main thread, update immediately
        update_label_immediate(label, &final_text, use_markup);
    } else {
        // We're not on the main thread, schedule the update
        let lbl = label.clone();
        glib::idle_add_local(move || {
            update_label_immediate(&lbl, &final_text, use_markup);
            glib::ControlFlow::Break
        });
    }
}

/// Immediately update a label on the main thread
fn update_label_immediate(label: &Label, text: &str, use_markup: bool) {
    if use_markup {
        // Escape and set markup for a bold visual (debug mode)
        let escaped_text = glib::markup_escape_text(text);
        label.set_markup(&format!("<b>{}</b>", escaped_text));
    } else {
        label.set_text(text);
    }

    footer_dbg!("[footer] set_label_text immediate -> {}", label.text());
    footer_dbg!(
        "[footer] label visible: {}, parent visible: {}",
        label.is_visible(),
        label.parent().map(|p| p.is_visible()).unwrap_or(false)
    );

    // Ensure widget is visible and request a redraw for better reliability
    label.set_visible(true);
    // Avoid calling queue_draw() directly here; GTK may issue warnings when widgets
    // are not yet allocated. Rely on set_visible and normal GTK redraw scheduling.

    // Also ensure parent is visible
    if let Some(parent) = label.parent() {
        parent.set_visible(true);
    }
}

pub fn create_footer(translations: &FooterTranslations) -> (Box, Rc<FooterLabels>) {
    let footer_box = Box::new(Orientation::Horizontal, 10);
    footer_box.set_margin_top(0);
    footer_box.set_margin_bottom(0);
    footer_box.set_margin_start(0);
    footer_box.set_margin_end(0);

    // Ensure footer is visible and properly allocated
    footer_box.set_visible(true);
    footer_box.set_can_focus(false);
    footer_box.set_vexpand(false);
    footer_box.set_hexpand(true);
    footer_box.set_height_request(0); // Minimum height

    // Add CSS class for potential styling
    footer_box.add_css_class("footer");

    // Spacer to push items to the right
    let spacer = Label::new(None);
    spacer.set_hexpand(true);
    spacer.set_visible(true);
    footer_box.append(&spacer);

    // Info labels (right side)
    let word_count_label = Label::new(Some(&format!("{}: 0", translations.words)));
    word_count_label.set_visible(true);
    footer_box.append(&word_count_label);

    let char_count_label = Label::new(Some(&format!("{}: 0", translations.characters)));
    char_count_label.set_visible(true);
    footer_box.append(&char_count_label);

    let cursor_row_label = Label::new(Some(&format!("{}: 1", translations.row)));
    cursor_row_label.set_visible(true);
    footer_box.append(&cursor_row_label);

    let cursor_col_label = Label::new(Some(&format!("{}: 1", translations.column)));
    cursor_col_label.set_visible(true);
    footer_box.append(&cursor_col_label);

    let encoding_label = Label::new(Some(&translations.encoding_utf8));
    encoding_label.set_visible(true);
    footer_box.append(&encoding_label);

    let insert_mode_label = Label::new(Some(&translations.ins));
    insert_mode_label.set_visible(true);
    footer_box.append(&insert_mode_label);

    let labels = FooterLabels {
        cursor_row: cursor_row_label,
        cursor_col: cursor_col_label,
        encoding: encoding_label,
        insert_mode: insert_mode_label,
        word_count: word_count_label,
        char_count: char_count_label,
        row_label: RefCell::new(translations.row.clone()),
        column_label: RefCell::new(translations.column.clone()),
        words_label: RefCell::new(translations.words.clone()),
        characters_label: RefCell::new(translations.characters.clone()),
        ins_label: RefCell::new(translations.ins.clone()),
        ovr_label: RefCell::new(translations.ovr.clone()),
        encoding_label: RefCell::new(translations.encoding_utf8.clone()),
    };

    (footer_box, Rc::new(labels))
}
