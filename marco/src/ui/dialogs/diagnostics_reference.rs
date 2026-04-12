//! Diagnostics Reference Dialog
//!
//! Searchable in-app reference for embedded diagnostics metadata.

use core::intelligence::diagnostics_catalog;
use gtk4::prelude::*;
use gtk4::{
    glib, Align, Box as GtkBox, Button, DropDown, Frame, Label, ListBox, ListBoxRow, Orientation,
    PropertyExpression, ScrolledWindow, SearchEntry, StringList, StringObject, Window,
};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SeverityFilter {
    All,
    Error,
    Warning,
    Info,
    Hint,
    Other,
}

impl SeverityFilter {
    fn from_catalog_str(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "error" => Self::Error,
            "warning" => Self::Warning,
            "info" => Self::Info,
            "hint" => Self::Hint,
            _ => Self::Other,
        }
    }

    fn from_selected_index(index: u32) -> Self {
        match index {
            1 => Self::Error,
            2 => Self::Warning,
            3 => Self::Info,
            4 => Self::Hint,
            _ => Self::All,
        }
    }

    fn matches(self, entry_severity: SeverityFilter) -> bool {
        matches!(self, SeverityFilter::All) || self == entry_severity
    }
}

fn make_search_blob(entry: &core::intelligence::DiagnosticsCatalogEntry) -> String {
    format!(
        "{} {} {} {} {} {}",
        entry.code,
        entry.title,
        entry.default_severity,
        entry.description,
        entry.fix_suggestion,
        entry.tags.join(" "),
    )
    .to_ascii_lowercase()
}

fn matches_query(search_blob: &str, query: &str) -> bool {
    let normalized_query = query.trim().to_ascii_lowercase();
    normalized_query.is_empty() || search_blob.contains(&normalized_query)
}

fn matches_filters(
    search_blob: &str,
    entry_severity: SeverityFilter,
    query: &str,
    severity_filter: SeverityFilter,
) -> bool {
    matches_query(search_blob, query) && severity_filter.matches(entry_severity)
}

pub fn show_diagnostics_reference_dialog(parent: &Window) {
    let theme_class = if parent.has_css_class("marco-theme-dark") {
        "marco-theme-dark"
    } else {
        "marco-theme-light"
    };

    let dialog = Window::builder()
        .modal(false)
        .transient_for(parent)
        .default_width(860)
        .default_height(620)
        .resizable(true)
        .build();

    dialog.add_css_class("marco-dialog");
    dialog.add_css_class(theme_class);

    let titlebar_controls = crate::ui::titlebar::create_custom_titlebar_with_buttons(
        &dialog,
        "Diagnostics Reference",
        crate::ui::titlebar::TitlebarButtons {
            close: true,
            minimize: false,
            maximize: false,
        },
    );

    let close_button = titlebar_controls
        .close_button
        .as_ref()
        .expect("Diagnostics Reference dialog requires close button");
    dialog.set_titlebar(Some(&titlebar_controls.headerbar));

    let root = GtkBox::new(Orientation::Vertical, 0);

    let content = GtkBox::new(Orientation::Vertical, 8);
    content.add_css_class("marco-dialog-content");
    content.set_margin_top(10);
    content.set_margin_bottom(8);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let subtitle = Label::new(Some(
        "Search by code, title, severity, tags, or fix suggestion.",
    ));
    subtitle.set_halign(Align::Start);
    subtitle.set_xalign(0.0);
    subtitle.add_css_class("marco-dialog-option-desc");
    content.append(&subtitle);

    let search_row = GtkBox::new(Orientation::Horizontal, 8);
    search_row.set_halign(Align::Fill);

    let search_label = Label::new(Some("Search"));
    search_label.set_halign(Align::Start);
    search_label.set_xalign(0.0);
    search_label.set_width_chars(8);
    search_label.add_css_class("marco-dialog-option-title");
    search_row.append(&search_label);

    let search_entry = SearchEntry::new();
    search_entry.add_css_class("marco-search-entry");
    search_entry.set_hexpand(true);
    search_entry.set_placeholder_text(Some("Search diagnostics (e.g. MD101, heading, link)…"));
    search_row.append(&search_entry);
    content.append(&search_row);

    let filter_row = GtkBox::new(Orientation::Horizontal, 8);
    filter_row.set_halign(Align::Fill);

    let severity_label = Label::new(Some("Severity"));
    severity_label.set_halign(Align::Start);
    severity_label.set_xalign(0.0);
    severity_label.set_width_chars(8);
    severity_label.add_css_class("marco-dialog-option-title");
    filter_row.append(&severity_label);

    let severity_options = StringList::new(&["All", "Error", "Warning", "Info", "Hint"]);
    let severity_expression = PropertyExpression::new(
        StringObject::static_type(),
        None::<&gtk4::Expression>,
        "string",
    );
    let severity_filter = DropDown::new(Some(severity_options), Some(severity_expression));
    severity_filter.add_css_class("marco-dropdown");
    severity_filter.set_selected(0);
    severity_filter.set_hexpand(true);
    filter_row.append(&severity_filter);

    content.append(&filter_row);

    let results_count = Label::new(None);
    results_count.set_halign(Align::Start);
    results_count.set_xalign(0.0);
    results_count.add_css_class("marco-dialog-option-desc");
    content.append(&results_count);

    let list_box = ListBox::new();
    list_box.set_selection_mode(gtk4::SelectionMode::None);
    list_box.add_css_class("marco-diagnostics-reference-list");

    let mut entries = diagnostics_catalog().entries.clone();
    entries.sort_by(|a, b| a.code.cmp(&b.code));

    let row_index: Rc<RefCell<Vec<(String, SeverityFilter, ListBoxRow)>>> =
        Rc::new(RefCell::new(Vec::new()));

    for entry in &entries {
        let row = ListBoxRow::new();
        row.add_css_class("marco-diagnostics-reference-row");

        let row_content = GtkBox::new(Orientation::Vertical, 6);
        row_content.add_css_class("marco-diagnostics-reference-row-content");
        row_content.set_margin_top(8);
        row_content.set_margin_bottom(8);
        row_content.set_margin_start(8);
        row_content.set_margin_end(8);

        let title = Label::new(Some(&format!(
            "{} · {} · {}",
            entry.code, entry.default_severity, entry.title
        )));
        title.set_halign(Align::Start);
        title.set_xalign(0.0);
        title.add_css_class("marco-dialog-option-title");
        row_content.append(&title);

        let description = Label::new(Some(&entry.description));
        description.set_halign(Align::Start);
        description.set_xalign(0.0);
        description.set_wrap(true);
        description.add_css_class("marco-dialog-option-desc");
        row_content.append(&description);

        let fix = Label::new(Some(&format!("Fix: {}", entry.fix_suggestion)));
        fix.set_halign(Align::Start);
        fix.set_xalign(0.0);
        fix.set_wrap(true);
        fix.add_css_class("marco-dialog-option-desc");
        row_content.append(&fix);

        row.set_child(Some(&row_content));
        list_box.append(&row);

        row_index.borrow_mut().push((
            make_search_blob(entry),
            SeverityFilter::from_catalog_str(&entry.default_severity),
            row,
        ));
    }

    let scroller = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .child(&list_box)
        .build();
    scroller.add_css_class("marco-diagnostics-reference-scroller");
    content.append(&scroller);

    let close_button_bottom = Button::with_label("Close");
    close_button_bottom.add_css_class("marco-btn");
    close_button_bottom.add_css_class("marco-btn-yellow");

    let button_box = GtkBox::new(Orientation::Horizontal, 8);
    button_box.set_halign(Align::End);
    button_box.append(&close_button_bottom);

    let bottom_frame = Frame::new(None);
    bottom_frame.add_css_class("marco-dialog-bottom-frame");
    bottom_frame.set_height_request(56);
    bottom_frame.set_vexpand(false);

    let bottom_inner = GtkBox::new(Orientation::Horizontal, 0);
    bottom_inner.set_margin_start(8);
    bottom_inner.set_margin_end(8);
    bottom_inner.set_margin_top(4);
    bottom_inner.set_margin_bottom(4);
    bottom_inner.set_halign(Align::Fill);
    bottom_inner.set_valign(Align::Center);

    let spacer = GtkBox::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    bottom_inner.append(&spacer);
    bottom_inner.append(&button_box);
    bottom_frame.set_child(Some(&bottom_inner));

    root.append(&content);
    root.append(&bottom_frame);
    dialog.set_child(Some(&root));

    let update_results = {
        let row_index = row_index.clone();
        let results_count = results_count.clone();
        move |query: &str, severity_filter: SeverityFilter| {
            let mut visible = 0usize;
            let total = row_index.borrow().len();

            for (search_blob, entry_severity, row) in row_index.borrow().iter() {
                let is_visible =
                    matches_filters(search_blob, *entry_severity, query, severity_filter);
                row.set_visible(is_visible);
                if is_visible {
                    visible += 1;
                }
            }

            results_count.set_text(&format!("Showing {} of {} diagnostics", visible, total));
        }
    };

    update_results("", SeverityFilter::All);

    {
        let update_results = update_results.clone();
        let severity_filter = severity_filter.clone();
        search_entry.connect_search_changed(move |entry| {
            update_results(
                entry.text().as_str(),
                SeverityFilter::from_selected_index(severity_filter.selected()),
            );
        });
    }

    {
        let update_results = update_results.clone();
        let search_entry = search_entry.clone();
        severity_filter.connect_selected_notify(move |dropdown| {
            update_results(
                search_entry.text().as_str(),
                SeverityFilter::from_selected_index(dropdown.selected()),
            );
        });
    }

    {
        let dialog_weak = dialog.downgrade();
        close_button_bottom.connect_clicked(move |_| {
            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.close();
            }
        });
    }

    {
        let dialog_weak = dialog.downgrade();
        close_button.connect_clicked(move |_| {
            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.close();
            }
        });
    }

    {
        let dialog_weak = dialog.downgrade();
        let key_controller = gtk4::EventControllerKey::new();
        key_controller.connect_key_pressed(move |_controller, key, _code, _state| {
            if key == gtk4::gdk::Key::Escape {
                if let Some(dialog) = dialog_weak.upgrade() {
                    dialog.close();
                }
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        dialog.add_controller(key_controller);
    }

    search_entry.grab_focus();
    dialog.present();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_matches_query() {
        let blob = "md101 warning heading too long use shorter heading";

        assert!(matches_query(blob, ""));
        assert!(matches_query(blob, "MD101"));
        assert!(matches_query(blob, "heading"));
        assert!(!matches_query(blob, "table-cell"));
    }

    #[test]
    fn smoke_test_matches_filters() {
        let blob = "md101 warning heading too long use shorter heading";

        assert!(matches_filters(
            blob,
            SeverityFilter::Warning,
            "heading",
            SeverityFilter::All
        ));
        assert!(matches_filters(
            blob,
            SeverityFilter::Warning,
            "md101",
            SeverityFilter::Warning
        ));
        assert!(!matches_filters(
            blob,
            SeverityFilter::Warning,
            "md101",
            SeverityFilter::Error
        ));
    }
}
