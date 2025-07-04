// Dialog builder utilities
// Common patterns for creating and configuring dialogs

use super::*;
use gtk4::{Button, FileChooserAction, FileChooserDialog, FileFilter};

/// Creates a standard dialog with common configuration
pub fn create_standard_dialog(
    title: &str,
    parent: &gtk4::Window,
    buttons: &[(&str, ResponseType)],
) -> Dialog {
    let dialog = Dialog::with_buttons(
        Some(title),
        Some(parent),
        gtk4::DialogFlags::MODAL,
        buttons,
    );
    
    dialog.set_default_size(400, 300);
    dialog
}

/// Creates a dialog with Accept/Cancel buttons
pub fn create_accept_cancel_dialog(
    title: &str,
    parent: &gtk4::Window,
) -> Dialog {
    create_standard_dialog(
        title,
        parent,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Accept", ResponseType::Accept),
        ],
    )
}

/// Creates a dialog with OK button only
pub fn create_ok_dialog(
    title: &str,
    parent: &gtk4::Window,
) -> Dialog {
    create_standard_dialog(
        title,
        parent,
        &[("OK", ResponseType::Ok)],
    )
}

/// Creates a main content box with standard margins
pub fn create_content_box(orientation: Orientation, spacing: i32) -> gtk4::Box {
    let main_box = gtk4::Box::new(orientation, spacing);
    main_box.set_margin_top(16);
    main_box.set_margin_bottom(16);
    main_box.set_margin_start(16);
    main_box.set_margin_end(16);
    main_box
}

/// Creates a labeled grid row with an entry widget
pub fn create_labeled_entry(
    grid: &Grid,
    row: i32,
    label_text: &str,
    placeholder: Option<&str>,
) -> Entry {
    let label = Label::new(Some(label_text));
    label.set_halign(gtk4::Align::Start);
    grid.attach(&label, 0, row, 1, 1);
    
    let entry = Entry::new();
    if let Some(placeholder) = placeholder {
        entry.set_placeholder_text(Some(placeholder));
    }
    grid.attach(&entry, 1, row, 1, 1);
    
    entry
}

/// Creates a labeled grid row with a spin button
pub fn create_labeled_spin_button(
    grid: &Grid,
    row: i32,
    label_text: &str,
    min: f64,
    max: f64,
    default: f64,
    step: f64,
) -> SpinButton {
    let label = Label::new(Some(label_text));
    label.set_halign(gtk4::Align::Start);
    grid.attach(&label, 0, row, 1, 1);
    
    let adjustment = Adjustment::new(default, min, max, step, 1.0, 0.0);
    let spin_button = SpinButton::new(Some(&adjustment), 1.0, 0);
    grid.attach(&spin_button, 1, row, 1, 1);
    
    spin_button
}

/// Creates a file picker button that opens a file chooser dialog
/// Returns the button and a callback closure to get the selected file path
pub fn create_file_picker_button(
    parent: &gtk4::Window,
    label: &str,
    title: &str,
    file_filters: Option<Vec<(String, String)>>, // (name, pattern) pairs
    url_entry: &Entry,
) -> Button {
    let button = Button::with_label(label);
    
    let parent_clone = parent.clone();
    let title_clone = title.to_string();
    let url_entry_clone = url_entry.clone();
    
    button.connect_clicked(move |_| {
        let dialog = FileChooserDialog::new(
            Some(&title_clone),
            Some(&parent_clone),
            FileChooserAction::Open,
            &[("Cancel", ResponseType::Cancel), ("Open", ResponseType::Accept)],
        );
        
        // Add file filters if provided
        if let Some(filters) = &file_filters {
            // Create a single filter for all image files
            let image_filter = FileFilter::new();
            image_filter.set_name(Some("Image files"));
            for (_, pattern) in filters {
                if pattern != "*" {
                    image_filter.add_pattern(pattern);
                }
            }
            dialog.add_filter(&image_filter);
            
            // Add "All files" filter
            let all_filter = FileFilter::new();
            all_filter.set_name(Some("All files"));
            all_filter.add_pattern("*");
            dialog.add_filter(&all_filter);
        }
        
        let url_entry_inner = url_entry_clone.clone();
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        // Convert to string and update the URL entry
                        let path_str = path.to_string_lossy().to_string();
                        url_entry_inner.set_text(&path_str);
                    }
                }
            }
            dialog.close();
        });
        
        dialog.present();
    });
    
    button
}

/// Creates a file picker button that uses a dialog as parent (for modal dialogs)
/// Returns the button and a callback closure to get the selected file path
pub fn create_file_picker_button_for_dialog(
    parent_dialog: &Dialog,
    label: &str,
    title: &str,
    file_filters: Option<Vec<(String, String)>>, // (name, pattern) pairs
    url_entry: &Entry,
) -> Button {
    let button = Button::with_label(label);
    
    let parent_clone = parent_dialog.clone();
    let title_clone = title.to_string();
    let url_entry_clone = url_entry.clone();
    
    button.connect_clicked(move |_| {
        let dialog = FileChooserDialog::new(
            Some(&title_clone),
            Some(&parent_clone),
            FileChooserAction::Open,
            &[("Cancel", ResponseType::Cancel), ("Open", ResponseType::Accept)],
        );
        
        // Add file filters if provided
        if let Some(filters) = &file_filters {
            // Create a single filter for all image files
            let image_filter = FileFilter::new();
            image_filter.set_name(Some("Image files"));
            for (_, pattern) in filters {
                if pattern != "*" {
                    image_filter.add_pattern(pattern);
                }
            }
            dialog.add_filter(&image_filter);
            
            // Add "All files" filter
            let all_filter = FileFilter::new();
            all_filter.set_name(Some("All files"));
            all_filter.add_pattern("*");
            dialog.add_filter(&all_filter);
        }
        
        let url_entry_inner = url_entry_clone.clone();
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        // Convert to string and update the URL entry
                        let path_str = path.to_string_lossy().to_string();
                        url_entry_inner.set_text(&path_str);
                    }
                }
            }
            dialog.close();
        });
        
        dialog.present();
    });
    
    button
}
