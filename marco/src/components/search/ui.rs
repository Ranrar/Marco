//! UI Widget Builders for Search Controls
//!
//! Creates search, replace, options, and button widgets.

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, CheckButton, Entry, Label, Orientation, Overlay, Separator};

use super::state::CURRENT_MATCH_LABEL;

/// Options panel widgets
#[derive(Clone)]
pub struct OptionsWidgets {
    pub match_case_cb: CheckButton,
    pub match_whole_word_cb: CheckButton,
    pub match_markdown_cb: CheckButton,
    pub use_regex_cb: CheckButton,
}

/// Button panel widgets  
pub struct ButtonWidgets {
    pub prev_button: Button,
    pub next_button: Button,
    pub replace_button: Button,
    pub replace_all_button: Button,
}

/// Create the search controls section
pub fn create_search_controls_section() -> (GtkBox, Entry, Label) {
    let search_box = GtkBox::new(Orientation::Vertical, 4);

    let search_row = GtkBox::new(Orientation::Horizontal, 8);

    let search_label = Label::new(Some("Find:"));
    search_label.set_width_request(60);
    search_label.set_halign(Align::Start);
    search_label.add_css_class("marco-search-label");

    // Create overlay to show match count inside the search input
    let search_overlay = Overlay::new();
    search_overlay.set_hexpand(true);

    let search_entry = Entry::new();
    search_entry.set_hexpand(true);
    search_entry.set_placeholder_text(Some("Enter search text..."));
    search_entry.add_css_class("marco-search-entry");

    // Match count label positioned as overlay inside the search field
    let match_count_label = Label::new(Some(""));
    match_count_label.set_halign(Align::End);
    match_count_label.set_valign(Align::Center);
    match_count_label.add_css_class("dim-label");
    match_count_label.add_css_class("marco-search-match-label");
    match_count_label.set_sensitive(false); // Make it non-interactive

    // Add entry as main child and label as overlay
    search_overlay.set_child(Some(&search_entry));
    search_overlay.add_overlay(&match_count_label);

    // No Find button needed - search happens automatically while typing

    search_row.append(&search_label);
    search_row.append(&search_overlay);

    search_box.append(&search_row);

    // Store label for global access
    CURRENT_MATCH_LABEL.with(|label_ref| {
        *label_ref.borrow_mut() = Some(match_count_label.clone());
    });

    (search_box, search_entry, match_count_label)
}

/// Create the replace controls section
pub fn create_replace_controls_section() -> (GtkBox, Entry) {
    let replace_box = GtkBox::new(Orientation::Vertical, 4);
    // Always visible in the simplified UI

    let replace_row = GtkBox::new(Orientation::Horizontal, 8);

    let replace_label = Label::new(Some("Replace:"));
    replace_label.set_width_request(60);
    replace_label.set_halign(Align::Start);
    replace_label.add_css_class("marco-search-label");

    let replace_entry = Entry::new();
    replace_entry.set_hexpand(true);
    replace_entry.set_placeholder_text(Some("Enter replacement text..."));
    replace_entry.add_css_class("marco-search-entry");

    replace_row.append(&replace_label);
    replace_row.append(&replace_entry);

    replace_box.append(&replace_row);

    (replace_box, replace_entry)
}

/// Create the options panel with checkboxes
pub fn create_options_panel() -> (GtkBox, OptionsWidgets) {
    let options_box = GtkBox::new(Orientation::Vertical, 6);

    // Separator
    let separator = Separator::new(Orientation::Horizontal);
    separator.add_css_class("marco-search-separator");
    options_box.append(&separator);

    // Options grid - two rows of two checkboxes each
    let options_grid = GtkBox::new(Orientation::Vertical, 3);

    // First row
    let row1 = GtkBox::new(Orientation::Horizontal, 16);
    row1.set_homogeneous(true);

    let match_case_cb = CheckButton::with_label("Match Case");
    match_case_cb.add_css_class("marco-search-checkbox");
    let match_markdown_cb = CheckButton::with_label("Match only Markdown syntax");
    match_markdown_cb.add_css_class("marco-search-checkbox");

    row1.append(&match_case_cb);
    row1.append(&match_markdown_cb);

    // Second row
    let row2 = GtkBox::new(Orientation::Horizontal, 16);
    row2.set_homogeneous(true);

    let match_whole_word_cb = CheckButton::with_label("Match Whole Word");
    match_whole_word_cb.add_css_class("marco-search-checkbox");
    let use_regex_cb = CheckButton::with_label("Regular Expressions");
    use_regex_cb.add_css_class("marco-search-checkbox");

    row2.append(&match_whole_word_cb);
    row2.append(&use_regex_cb);

    options_grid.append(&row1);
    options_grid.append(&row2);
    options_box.append(&options_grid);

    let widgets = OptionsWidgets {
        match_case_cb,
        match_whole_word_cb,
        match_markdown_cb,
        use_regex_cb,
    };

    (options_box, widgets)
}

/// Create the button panel for search window (no close button needed)
pub fn create_window_button_panel() -> (GtkBox, ButtonWidgets) {
    let button_box = GtkBox::new(Orientation::Horizontal, 6);
    button_box.set_halign(Align::End);
    button_box.set_margin_top(8);

    // Bottom buttons: [Previous] [Next] [Replace] [Replace All]
    // No close button needed since the window has its own close controls
    let prev_button = Button::with_label("Previous");
    prev_button.add_css_class("marco-search-button");
    let next_button = Button::with_label("Next");
    next_button.add_css_class("marco-search-button");

    let replace_button = Button::with_label("Replace");
    replace_button.add_css_class("marco-search-button");
    replace_button.set_sensitive(false); // Initially disabled when Replace input is empty

    let replace_all_button = Button::with_label("Replace All");
    replace_all_button.add_css_class("marco-search-button");
    replace_all_button.set_sensitive(false); // Initially disabled when Replace input is empty

    button_box.append(&prev_button);
    button_box.append(&next_button);
    button_box.append(&replace_button);
    button_box.append(&replace_all_button);

    let widgets = ButtonWidgets {
        prev_button,
        next_button,
        replace_button,
        replace_all_button,
    };

    (button_box, widgets)
}

