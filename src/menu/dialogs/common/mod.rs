// Common dialog utilities and components
// This module contains shared functionality used across different dialog types

pub mod builders;
pub mod preview;
pub mod validation;

// Re-export commonly used imports for dialog modules
pub use gtk4::prelude::*;
pub use gtk4::{
    Adjustment, Dialog, Entry, Grid, Label, Orientation, ResponseType, ScrolledWindow, SpinButton,
    TextView,
};

// Re-export builder functions
pub use builders::*;

/// Creates a content box with standard margins
pub fn create_content_box(orientation: Orientation, spacing: i32) -> gtk4::Box {
    let box_widget = gtk4::Box::new(orientation, spacing);
    box_widget.set_margin_top(12);
    box_widget.set_margin_bottom(12);
    box_widget.set_margin_start(12);
    box_widget.set_margin_end(12);
    box_widget
}

/// Creates a preview area with label and text view
pub fn create_preview_area(label: &str) -> (gtk4::Box, TextView) {
    let preview_box = gtk4::Box::new(Orientation::Vertical, 8);

    let label_widget = Label::new(Some(label));
    label_widget.set_halign(gtk4::Align::Start);
    preview_box.append(&label_widget);

    let scrolled = ScrolledWindow::new();
    scrolled.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    scrolled.set_min_content_height(120);

    let text_view = TextView::new();
    text_view.set_editable(false);
    text_view.set_wrap_mode(gtk4::WrapMode::Word);
    scrolled.set_child(Some(&text_view));

    preview_box.append(&scrolled);

    (preview_box, text_view)
}
