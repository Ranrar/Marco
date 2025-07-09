use crate::editor::core::MarkdownEditor;
use gtk4::prelude::*;
use gtk4::{FileChooserAction, FileChooserDialog, ResponseType};

impl MarkdownEditor {
    pub fn new_file(&self) {
        self.source_buffer.set_text("");
        *self.current_file.borrow_mut() = None;
        // Reset original content to empty string for new file
        *self.original_content.borrow_mut() = String::new();
        self.mark_as_saved(); // New file is not modified (also updates window title)

        // Update base path for image resolution
        self.html_view.set_base_path(None);
    }

    pub fn open_file_from_menu(&self, window: &gtk4::Window) {
        self.show_open_dialog(Some(window));
    }

    pub fn save_file_from_menu(&self, window: &gtk4::Window) {
        self.save_current_file(Some(window));
    }

    pub fn save_as_file_from_menu(&self, window: &gtk4::Window) {
        self.show_save_as_dialog(Some(window));
    }

    fn show_open_dialog(&self, parent: Option<&gtk4::Window>) {
        let dialog = FileChooserDialog::new(
            Some("Open File"),
            parent,
            FileChooserAction::Open,
            &[
                ("Cancel", ResponseType::Cancel),
                ("Open", ResponseType::Accept),
            ],
        );

        let source_buffer = self.source_buffer.clone();
        let current_file = self.current_file.clone();
        let is_modified = self.is_modified.clone();
        let original_content = self.original_content.clone();
        let editor_for_title = self.clone();

        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        match std::fs::read_to_string(&path) {
                            Ok(content) => {
                                source_buffer.set_text(&content);
                                *current_file.borrow_mut() = Some(path.clone());
                                *is_modified.borrow_mut() = false; // Mark as saved after opening
                                                                   // Set original content to the loaded content
                                *original_content.borrow_mut() = content;
                                println!("DEBUG: File opened and marked as saved");
                                // Update window title after opening file
                                editor_for_title.update_window_title();
                                // Update base path for image resolution
                                editor_for_title.html_view.set_base_path(Some(path));
                            }
                            Err(e) => {
                                eprintln!("Failed to open file: {}", e);
                            }
                        }
                    }
                }
            }
            dialog.close();
        });

        dialog.present();
    }

    fn save_current_file(&self, parent: Option<&gtk4::Window>) {
        println!("DEBUG: save_current_file called");
        if let Some(path) = self.current_file.borrow().clone() {
            // Save to existing file
            println!("DEBUG: Saving to existing file: {:?}", path);
            let start = self.source_buffer.start_iter();
            let end = self.source_buffer.end_iter();
            let text = self.source_buffer.text(&start, &end, false);
            if std::fs::write(&path, text).is_ok() {
                println!("DEBUG: File saved successfully, marking as saved");
                self.mark_as_saved(); // Mark as saved after successful write
            } else {
                println!("DEBUG: Failed to save file");
            }
        } else {
            // No file selected, show save as dialog
            println!("DEBUG: No current file, showing save as dialog");
            self.show_save_as_dialog(parent);
        }
    }

    /// Save current file with a callback that's only called on successful save
    pub(crate) fn save_current_file_with_callback<F>(
        &self,
        parent: Option<&gtk4::Window>,
        on_save_complete: F,
    ) where
        F: Fn() + 'static,
    {
        println!("DEBUG: save_current_file_with_callback called");
        if let Some(path) = self.current_file.borrow().clone() {
            // Save to existing file
            println!("DEBUG: Saving to existing file: {:?}", path);
            let start = self.source_buffer.start_iter();
            let end = self.source_buffer.end_iter();
            let text = self.source_buffer.text(&start, &end, false);
            if std::fs::write(&path, text).is_ok() {
                println!("DEBUG: File saved successfully, marking as saved and calling callback");
                self.mark_as_saved(); // Mark as saved after successful write
                on_save_complete(); // Call the callback only on successful save
            } else {
                println!("DEBUG: Failed to save file, not calling callback");
            }
        } else {
            // No file selected, show save as dialog with callback
            println!("DEBUG: No current file, showing save as dialog with callback");
            self.show_save_as_dialog_with_callback(parent, on_save_complete);
        }
    }

    fn show_save_as_dialog(&self, parent: Option<&gtk4::Window>) {
        let dialog = FileChooserDialog::new(
            Some("Save File"),
            parent,
            FileChooserAction::Save,
            &[
                ("Cancel", ResponseType::Cancel),
                ("Save", ResponseType::Accept),
            ],
        );

        let source_buffer = self.source_buffer.clone();
        let current_file = self.current_file.clone();
        let is_modified = self.is_modified.clone();
        let editor_for_title = self.clone();

        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        let start = source_buffer.start_iter();
                        let end = source_buffer.end_iter();
                        let text = source_buffer.text(&start, &end, false);

                        match std::fs::write(&path, text) {
                            Ok(_) => {
                                *current_file.borrow_mut() = Some(path.clone());
                                *is_modified.borrow_mut() = false; // Mark as saved after successful write
                                println!("DEBUG: File saved as and marked as saved");
                                // Update window title after saving file
                                editor_for_title.update_window_title();
                                // Update base path for image resolution
                                editor_for_title.html_view.set_base_path(Some(path));
                            }
                            Err(e) => {
                                eprintln!("Failed to save file: {}", e);
                            }
                        }
                    }
                }
            }
            dialog.close();
        });

        dialog.present();
    }

    /// Show save as dialog with a callback that's only called on successful save
    fn show_save_as_dialog_with_callback<F>(
        &self,
        parent: Option<&gtk4::Window>,
        on_save_complete: F,
    ) where
        F: Fn() + 'static,
    {
        println!("DEBUG: show_save_as_dialog_with_callback called");
        let dialog = FileChooserDialog::new(
            Some("Save File"),
            parent,
            FileChooserAction::Save,
            &[
                ("Cancel", ResponseType::Cancel),
                ("Save", ResponseType::Accept),
            ],
        );

        let source_buffer = self.source_buffer.clone();
        let current_file = self.current_file.clone();
        let is_modified = self.is_modified.clone();
        let editor_for_title = self.clone();

        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                println!("DEBUG: User clicked Save in Save As dialog");
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        let start = source_buffer.start_iter();
                        let end = source_buffer.end_iter();
                        let text = source_buffer.text(&start, &end, false);
                        match std::fs::write(&path, text) {
                            Ok(_) => {
                                println!("DEBUG: File saved successfully, marking as saved and calling callback");
                                *current_file.borrow_mut() = Some(path.clone());
                                *is_modified.borrow_mut() = false; // Mark as saved after successful write
                                // Update window title after saving file
                                editor_for_title.update_window_title();
                                // Update base path for image resolution
                                editor_for_title.html_view.set_base_path(Some(path));
                                on_save_complete(); // Call the callback only on successful save
                            }
                            Err(e) => {
                                println!("DEBUG: Failed to save file: {}, not calling callback", e);
                                eprintln!("Failed to save file: {}", e);
                            }
                        }
                    }
                }
            } else {
                println!("DEBUG: User cancelled Save As dialog, not calling callback");
            }
            dialog.close();
        });

        dialog.present();
    }

    /// Load a file from a given path (for command-line usage)
    pub fn load_file_from_path(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = std::path::Path::new(file_path);

        if !path.exists() {
            return Err(format!("File does not exist: {}", file_path).into());
        }

        let content = std::fs::read_to_string(path)?;

        // Set the content in the source buffer
        self.source_buffer.set_text(&content);

        // Update the current file
        *self.current_file.borrow_mut() = Some(path.to_path_buf());

        // Mark as saved (since we just loaded it)
        *self.is_modified.borrow_mut() = false;

        // Set original content to the loaded content
        *self.original_content.borrow_mut() = content;

        // Update window title
        self.update_window_title();

        // Update base path for image resolution
        self.html_view.set_base_path(Some(path.to_path_buf()));

        Ok(())
    }
}
