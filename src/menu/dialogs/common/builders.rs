// Dialog builder utilities
// Common patterns for creating and configuring dialogs

use super::*;
use gtk4::{Button, FileChooserAction, FileChooserDialog, FileFilter};

/// Creates a labeled entry field in a grid
pub fn create_labeled_entry(
    grid: &Grid,
    row: i32,
    label_text: &str,
    placeholder: Option<&str>,
) -> Entry {
    let label = Label::new(Some(label_text));
    label.set_halign(gtk4::Align::End);
    grid.attach(&label, 0, row, 1, 1);

    let entry = Entry::new();
    if let Some(placeholder) = placeholder {
        entry.set_placeholder_text(Some(placeholder));
    }
    grid.attach(&entry, 1, row, 1, 1);

    entry
}

/// Creates a file picker button for dialogs
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
            &[
                ("Cancel", ResponseType::Cancel),
                ("Open", ResponseType::Accept),
            ],
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
