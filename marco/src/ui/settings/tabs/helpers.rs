//! Shared helper functions for settings tabs
//!
//! Provides standardized UI construction helpers to ensure perfect visual consistency
//! across all settings tabs with table-like structure.

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, CheckButton, Frame, Label, Orientation, StringList};

use crate::components::language::Translations;
use std::cell::RefCell;
use std::rc::Rc;

/// A text getter used by the settings i18n registry.
///
/// We store getters (instead of string keys) to keep call-sites simple and
/// avoid duplicating a giant match table for all settings strings.
pub type I18nTextFn = Rc<dyn Fn(&Translations) -> String>;

#[derive(Clone, Default)]
pub struct SettingsI18nRegistry {
    bindings: Rc<RefCell<Vec<SettingsI18nBinding>>>,
}

#[derive(Clone)]
enum SettingsI18nBinding {
    LabelText {
        label: Label,
        get: I18nTextFn,
    },
    LabelBoldMarkup {
        label: Label,
        get: I18nTextFn,
    },
    LabelMarkup {
        label: Label,
        get: I18nTextFn,
    },
    ButtonLabel {
        button: Button,
        get: I18nTextFn,
    },
    CheckButtonLabel {
        button: CheckButton,
        get: I18nTextFn,
    },
    StringListItem {
        list: StringList,
        index: u32,
        get: I18nTextFn,
    },
}

impl SettingsI18nRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind_label_text(&self, label: &Label, get: I18nTextFn) {
        self.bindings
            .borrow_mut()
            .push(SettingsI18nBinding::LabelText {
                label: label.clone(),
                get,
            });
    }

    pub fn bind_label_bold_markup(&self, label: &Label, get: I18nTextFn) {
        self.bindings
            .borrow_mut()
            .push(SettingsI18nBinding::LabelBoldMarkup {
                label: label.clone(),
                get,
            });
    }

    pub fn bind_label_markup(&self, label: &Label, get: I18nTextFn) {
        self.bindings
            .borrow_mut()
            .push(SettingsI18nBinding::LabelMarkup {
                label: label.clone(),
                get,
            });
    }

    pub fn bind_button_label(&self, button: &Button, get: I18nTextFn) {
        self.bindings
            .borrow_mut()
            .push(SettingsI18nBinding::ButtonLabel {
                button: button.clone(),
                get,
            });
    }

    pub fn bind_check_button_label(&self, button: &CheckButton, get: I18nTextFn) {
        self.bindings
            .borrow_mut()
            .push(SettingsI18nBinding::CheckButtonLabel {
                button: button.clone(),
                get,
            });
    }

    pub fn bind_string_list_item(&self, list: &StringList, index: u32, get: I18nTextFn) {
        self.bindings
            .borrow_mut()
            .push(SettingsI18nBinding::StringListItem {
                list: list.clone(),
                index,
                get,
            });
    }

    /// Apply updated translations to all registered bindings.
    ///
    /// This updates widgets in-place and intentionally does **not** rebuild any
    /// tabs or rows (rebuilds previously caused crashes).
    pub fn apply(&self, translations: &Translations) {
        for binding in self.bindings.borrow().iter() {
            match binding {
                SettingsI18nBinding::LabelText { label, get } => {
                    label.set_text(&get(translations));
                }
                SettingsI18nBinding::LabelBoldMarkup { label, get } => {
                    let text = get(translations);
                    label.set_markup(&format!(
                        "<b>{}</b>",
                        glib::markup_escape_text(text.as_str())
                    ));
                }
                SettingsI18nBinding::LabelMarkup { label, get } => {
                    let text = get(translations);
                    label.set_markup(&text);
                }
                SettingsI18nBinding::ButtonLabel { button, get } => {
                    button.set_label(&get(translations));
                }
                SettingsI18nBinding::CheckButtonLabel { button, get } => {
                    let text = get(translations);
                    button.set_label(Some(text.as_str()));
                }
                SettingsI18nBinding::StringListItem { list, index, get } => {
                    if list.n_items() > *index {
                        let text = get(translations);
                        let additions = [text.as_str()];
                        // Replace exactly one item at `index`.
                        list.splice(*index, 1, &additions);
                    }
                }
            }
        }
    }
}

// UI Constants for table-like structure
const ROW_FIXED_HEIGHT: i32 = 56; // More compact: 70 → 56
const ROW_PADDING_HORIZONTAL: i32 = 10; // More compact: 12 → 10
const ROW_PADDING_VERTICAL: i32 = 6; // More compact: 8 → 6
const HEADER_LABEL_HEIGHT: i32 = 16; // More compact: 18 → 16
const CONTROL_WIDTH: i32 = 160; // More compact: 180 → 160
const DESC_MARGIN_TOP: i32 = 3; // More compact: 4 → 3
const LEFT_COLUMN_WIDTH: i32 = 360; // More compact: 400 → 360

/// Create a standardized settings row with header, description, and right-aligned control
///
/// Creates a table-like structure with:
/// Create a standardized settings row with header, description, and right-aligned control
///
/// Creates a table-like structure with:
/// - FIXED height (90px) - all rows have same height regardless of content
/// - Border frame around each row
/// - Consistent padding (16px horizontal, 12px vertical)
/// - Two-column layout:
///   - LEFT COLUMN (450px):
///     - Bold header label (top-left, 20px height)
///     - Helper text directly below header (6px gap, max 2 lines)
///   - RIGHT COLUMN (200px):
///     - Control widget at top-right corner
///
/// Visual structure:
/// ```
/// ┌─────────────────────────────────────────────┐
/// │ HEADER (bold)              [Control]    │
/// │ Helper text here...                      │ 90px
/// │ (max 2 lines, ellipsis if longer)       │
/// └─────────────────────────────────────────────┘
/// ```
///
/// # Arguments
///
/// * `title` - Bold header text (will be automatically formatted)
/// * `description` - Description text below the header (dimmed styling)
/// * `control` - The control widget (dropdown, switch, spinbutton, etc.)
/// * `is_first` - Whether this is the first row in the container (affects top margin)
///
/// # Returns
///
/// A `gtk4::Box` containing the complete row structure ready to append to a container
///
/// # Example
///
/// ```rust,no_run
/// use gtk4::prelude::*;
/// use gtk4::{Box, Switch};
///
/// let container = Box::new(gtk4::Orientation::Vertical, 0);
/// let switch = Switch::new();
///
/// let row = add_setting_row(
///     "Enable Feature",
///     "This feature does something useful.",
///     &switch,
///     true  // First row
/// );
///
/// container.append(&row);
/// ```
pub fn add_setting_row(
    title: &str,
    description: &str,
    control: &impl IsA<gtk4::Widget>,
    is_first: bool,
) -> GtkBox {
    // Create outer container for the entire row
    let outer_box = GtkBox::new(Orientation::Vertical, 0);
    outer_box.set_vexpand(false); // Don't expand vertically

    // Create frame for table-like border
    let frame = Frame::new(None);
    frame.add_css_class("marco-settings-row-frame");
    frame.set_height_request(ROW_FIXED_HEIGHT); // FIXED height, not minimum!
    frame.set_vexpand(false); // Don't allow vertical expansion
    frame.set_valign(Align::Fill); // Fill the allocated space

    // Create main horizontal container: LEFT column (header+desc) | RIGHT column (control)
    let main_hbox = GtkBox::new(Orientation::Horizontal, 16);
    main_hbox.set_margin_start(ROW_PADDING_HORIZONTAL);
    main_hbox.set_margin_end(ROW_PADDING_HORIZONTAL);
    main_hbox.set_margin_top(ROW_PADDING_VERTICAL);
    main_hbox.set_margin_bottom(ROW_PADDING_VERTICAL);
    main_hbox.set_vexpand(false); // Don't expand
    main_hbox.set_valign(Align::Fill); // Fill vertically within frame

    // === LEFT COLUMN: Header + Description (stacked vertically) ===
    let left_vbox = GtkBox::new(Orientation::Vertical, 0);
    left_vbox.set_width_request(LEFT_COLUMN_WIDTH);
    left_vbox.set_hexpand(false); // Fixed width, don't expand
    left_vbox.set_vexpand(false); // Don't expand vertically
    left_vbox.set_halign(Align::Start);
    left_vbox.set_valign(Align::Start); // Align to top

    // Create bold header label
    let header = Label::new(Some(title));
    header.set_markup(&format!("<b>{}</b>", glib::markup_escape_text(title)));
    header.set_halign(Align::Start);
    header.set_xalign(0.0);
    header.set_valign(Align::Start);
    header.set_height_request(HEADER_LABEL_HEIGHT);
    header.set_vexpand(false); // Fixed height, don't expand
    header.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    header.set_max_width_chars(50); // Constrain width
    header.add_css_class("marco-settings-header"); // Add CSS class for styling

    // Create description label directly below header
    let desc = Label::new(Some(description));
    desc.set_halign(Align::Start);
    desc.set_xalign(0.0);
    desc.set_valign(Align::Start);
    desc.set_wrap(true);
    desc.set_wrap_mode(gtk4::pango::WrapMode::Word);
    desc.set_lines(2); // Max 2 lines for consistent height
    desc.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    desc.set_vexpand(false); // Don't expand
    desc.add_css_class("dim-label"); // Keep existing class
    desc.add_css_class("marco-settings-description"); // Add CSS class for styling
    desc.set_margin_top(DESC_MARGIN_TOP);
    desc.set_max_width_chars(60); // Constrain width

    // Add header and description to left column
    left_vbox.append(&header);
    left_vbox.append(&desc);

    // === RIGHT COLUMN: Control (top-right corner) ===
    let right_vbox = GtkBox::new(Orientation::Vertical, 0);
    right_vbox.set_width_request(CONTROL_WIDTH);
    right_vbox.set_hexpand(false); // Fixed width, don't expand
    right_vbox.set_vexpand(false); // Don't expand vertically
    right_vbox.set_halign(Align::End);
    right_vbox.set_valign(Align::Start); // Align to top

    // Control positioned at top-right
    control.set_halign(Align::End);
    control.set_valign(Align::Start); // Top of the row
    control.set_vexpand(false); // Don't expand

    right_vbox.append(control);

    // === Assemble main layout ===
    main_hbox.append(&left_vbox);
    main_hbox.append(&right_vbox);

    // Assemble frame
    frame.set_child(Some(&main_hbox));
    outer_box.append(&frame);

    // Apply consistent margins between rows
    if !is_first {
        outer_box.set_margin_top(4); // Small gap between rows
    }

    outer_box
}

/// Create a standardized settings row like [`add_setting_row`], but also bind the
/// row's title + description labels to the i18n registry for runtime updates.
pub fn add_setting_row_i18n(
    i18n: &SettingsI18nRegistry,
    title: &str,
    description: &str,
    title_get: I18nTextFn,
    description_get: I18nTextFn,
    control: &impl IsA<gtk4::Widget>,
    is_first: bool,
) -> GtkBox {
    // Create outer container for the entire row
    let outer_box = GtkBox::new(Orientation::Vertical, 0);
    outer_box.set_vexpand(false); // Don't expand vertically

    // Create frame for table-like border
    let frame = Frame::new(None);
    frame.add_css_class("marco-settings-row-frame");
    frame.set_height_request(ROW_FIXED_HEIGHT); // FIXED height, not minimum!
    frame.set_vexpand(false); // Don't allow vertical expansion
    frame.set_valign(Align::Fill); // Fill the allocated space

    // Create main horizontal container: LEFT column (header+desc) | RIGHT column (control)
    let main_hbox = GtkBox::new(Orientation::Horizontal, 16);
    main_hbox.set_margin_start(ROW_PADDING_HORIZONTAL);
    main_hbox.set_margin_end(ROW_PADDING_HORIZONTAL);
    main_hbox.set_margin_top(ROW_PADDING_VERTICAL);
    main_hbox.set_margin_bottom(ROW_PADDING_VERTICAL);
    main_hbox.set_vexpand(false); // Don't expand
    main_hbox.set_valign(Align::Fill); // Fill vertically within frame

    // === LEFT COLUMN: Header + Description (stacked vertically) ===
    let left_vbox = GtkBox::new(Orientation::Vertical, 0);
    left_vbox.set_width_request(LEFT_COLUMN_WIDTH);
    left_vbox.set_hexpand(false); // Fixed width, don't expand
    left_vbox.set_vexpand(false); // Don't expand vertically
    left_vbox.set_halign(Align::Start);
    left_vbox.set_valign(Align::Start); // Align to top

    // Create bold header label
    let header = Label::new(Some(title));
    header.set_markup(&format!("<b>{}</b>", glib::markup_escape_text(title)));
    header.set_halign(Align::Start);
    header.set_xalign(0.0);
    header.set_valign(Align::Start);
    header.set_height_request(HEADER_LABEL_HEIGHT);
    header.set_vexpand(false); // Fixed height, don't expand
    header.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    header.set_max_width_chars(50); // Constrain width
    header.add_css_class("marco-settings-header"); // Add CSS class for styling

    // Create description label directly below header
    let desc = Label::new(Some(description));
    desc.set_halign(Align::Start);
    desc.set_xalign(0.0);
    desc.set_valign(Align::Start);
    desc.set_wrap(true);
    desc.set_wrap_mode(gtk4::pango::WrapMode::Word);
    desc.set_lines(2); // Max 2 lines for consistent height
    desc.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    desc.set_vexpand(false); // Don't expand
    desc.add_css_class("dim-label"); // Keep existing class
    desc.add_css_class("marco-settings-description"); // Add CSS class for styling
    desc.set_margin_top(DESC_MARGIN_TOP);
    desc.set_max_width_chars(60); // Constrain width

    // Register i18n bindings for runtime updates
    i18n.bind_label_bold_markup(&header, title_get);
    i18n.bind_label_text(&desc, description_get);

    // Add header and description to left column
    left_vbox.append(&header);
    left_vbox.append(&desc);

    // === RIGHT COLUMN: Control (top-right corner) ===
    let right_vbox = GtkBox::new(Orientation::Vertical, 0);
    right_vbox.set_width_request(CONTROL_WIDTH);
    right_vbox.set_hexpand(false); // Fixed width, don't expand
    right_vbox.set_vexpand(false); // Don't expand vertically
    right_vbox.set_halign(Align::End);
    right_vbox.set_valign(Align::Start); // Align to top

    // Control positioned at top-right
    control.set_halign(Align::End);
    control.set_valign(Align::Start); // Top of the row
    control.set_vexpand(false); // Don't expand

    right_vbox.append(control);

    // === Assemble main layout ===
    main_hbox.append(&left_vbox);
    main_hbox.append(&right_vbox);

    // Assemble frame
    frame.set_child(Some(&main_hbox));
    outer_box.append(&frame);

    // Apply consistent margins between rows
    if !is_first {
        outer_box.set_margin_top(4); // Small gap between rows
    }

    outer_box
}

/// Create a bold header label for section titles
///
/// Use this for section headers that don't have controls (e.g., "HTML Output Configuration")
/// The section header automatically gets proper spacing:
/// - 16px top margin (to separate from previous content)
/// - 8px bottom margin (gap before first row)
/// - Larger, bold text for visual hierarchy
///
/// # Example
///
/// ```rust,no_run
/// let header = create_section_header("Advanced Settings");
/// container.append(&header);
/// // First row after header should use is_first=true
/// ```
pub fn create_section_header(text: &str) -> Label {
    let label = Label::new(Some(text));
    label.set_markup(&format!(
        "<span size='large'><b>{}</b></span>",
        glib::markup_escape_text(text)
    ));
    label.set_halign(Align::Start);
    label.set_xalign(0.0);

    // Add consistent spacing for section headers
    label.set_margin_top(16); // Separate from previous content
    label.set_margin_bottom(8); // Gap before first row

    label
}
