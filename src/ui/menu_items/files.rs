use crate::logic::menu_items::file::SaveChangesResult;
use anyhow::Result;
use gtk4::{
    prelude::*, ButtonsType, DialogFlags, FileChooserAction, FileChooserNative, MessageDialog,
    MessageType, ResponseType, Window,
};
use std::path::{Path, PathBuf};

/// UI dialogs for file operations
///
/// This module provides GTK4-based dialog implementations for:
/// - File open/save dialogs using FileChooserNative
/// - Confirmation dialogs for unsaved changes
/// - Error message dialogs
/// - File overwrite confirmations
///
/// All dialogs are designed to be modal and properly parented
/// to ensure correct behavior on all platforms.
pub struct FileDialogs;

impl FileDialogs {
    /// Shows a native file open dialog
    ///
    /// # Arguments
    /// * `parent` - Parent window for the dialog
    /// * `title` - Dialog title (e.g., "Open Markdown File")
    ///
    /// # Returns
    /// * `Ok(Some(PathBuf))` - User selected a file
    /// * `Ok(None)` - User cancelled the dialog
    /// * `Err(anyhow::Error)` - Dialog failed to show
    ///
    /// # Example
    /// ```
    /// let path = FileDialogs::show_open_dialog(&window, "Open Markdown File").await?;
    /// if let Some(file_path) = path {
    ///     println!("Selected file: {}", file_path.display());
    /// }
    /// ```
    pub async fn show_open_dialog<W: IsA<Window>>(
        parent: &W,
        title: &str,
    ) -> Result<Option<PathBuf>> {
        let dialog = FileChooserNative::new(
            Some(title),
            Some(parent),
            FileChooserAction::Open,
            Some("_Open"),
            Some("_Cancel"),
        );

        // Set up file filters for markdown files
        let filter = gtk4::FileFilter::new();
        filter.set_name(Some("Markdown Files"));
        filter.add_pattern("*.md");
        filter.add_pattern("*.markdown");
        filter.add_pattern("*.mdown");
        filter.add_pattern("*.mkd");
        dialog.add_filter(&filter);

        let filter_all = gtk4::FileFilter::new();
        filter_all.set_name(Some("All Files"));
        filter_all.add_pattern("*");
        dialog.add_filter(&filter_all);

        // Show dialog and wait for response
        let response = dialog.run_future().await;

        match response {
            ResponseType::Accept => {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        log::info!("[FileDialogs] User selected file: {}", path.display());
                        return Ok(Some(path));
                    }
                }
                Err(anyhow::anyhow!("No file selected"))
            }
            _ => {
                log::debug!("[FileDialogs] Open dialog cancelled");
                Ok(None)
            }
        }
    }

    /// Shows a native file save dialog
    ///
    /// # Arguments
    /// * `parent` - Parent window for the dialog
    /// * `title` - Dialog title (e.g., "Save Markdown File")
    /// * `suggested_name` - Default filename (optional)
    ///
    /// # Returns
    /// * `Ok(Some(PathBuf))` - User selected a save location
    /// * `Ok(None)` - User cancelled the dialog
    /// * `Err(anyhow::Error)` - Dialog failed to show
    ///
    /// # Example
    /// ```
    /// let path = FileDialogs::show_save_dialog(&window, "Save As", Some("Untitled.md")).await?;
    /// if let Some(file_path) = path {
    ///     println!("Save to: {}", file_path.display());
    /// }
    /// ```
    pub async fn show_save_dialog<W: IsA<Window>>(
        parent: &W,
        title: &str,
        suggested_name: Option<&str>,
    ) -> Result<Option<PathBuf>> {
        let dialog = FileChooserNative::new(
            Some(title),
            Some(parent),
            FileChooserAction::Save,
            Some("_Save"),
            Some("_Cancel"),
        );

        // Set up file filters
        let filter = gtk4::FileFilter::new();
        filter.set_name(Some("Markdown Files"));
        filter.add_pattern("*.md");
        filter.add_pattern("*.markdown");
        dialog.add_filter(&filter);

        let filter_all = gtk4::FileFilter::new();
        filter_all.set_name(Some("All Files"));
        filter_all.add_pattern("*");
        dialog.add_filter(&filter_all);

        // Set suggested filename
        if let Some(name) = suggested_name {
            dialog.set_current_name(name);
        }

        // Show dialog and wait for response
        let response = dialog.run_future().await;

        match response {
            ResponseType::Accept => {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        log::info!("[FileDialogs] User chose save location: {}", path.display());
                        return Ok(Some(path));
                    }
                }
                Err(anyhow::anyhow!("No save location selected"))
            }
            _ => {
                log::debug!("[FileDialogs] Save dialog cancelled");
                Ok(None)
            }
        }
    }

    /// Shows a "Save Changes?" confirmation dialog
    ///
    /// # Arguments
    /// * `parent` - Parent window for the dialog
    /// * `document_name` - Name of the document with unsaved changes
    /// * `action` - What the user is trying to do (e.g., "closing the document")
    ///
    /// # Returns
    /// * `Ok(SaveChangesResult::Save)` - User wants to save
    /// * `Ok(SaveChangesResult::Discard)` - User wants to discard changes
    /// * `Ok(SaveChangesResult::Cancel)` - User cancelled the operation
    /// * `Err(anyhow::Error)` - Dialog failed to show
    ///
    /// # Example
    /// ```
    /// match FileDialogs::show_save_changes_dialog(&window, "Untitled.md", "closing").await? {
    ///     SaveChangesResult::Save => save_document(),
    ///     SaveChangesResult::Discard => close_without_saving(),
    ///     SaveChangesResult::Cancel => return,
    /// }
    /// ```
    pub async fn show_save_changes_dialog<W: IsA<Window>>(
        parent: &W,
        document_name: &str,
        action: &str,
    ) -> Result<SaveChangesResult> {
        let dialog = MessageDialog::new(
            Some(parent),
            DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
            MessageType::Warning,
            ButtonsType::None,
            format!("Save changes to \"{}\" before {}?", document_name, action),
        );

        dialog.set_secondary_text(Some(
            "\nYour changes will be lost if you don't save them.\n",
        ));

        // Add custom buttons to match the requested UI:
        // - "Close without Saving" -> destructive action -> maps to Discard
        // - "Cancel" -> cancels the operation -> maps to Cancel
        // - "Save As..." -> suggested/default action -> maps to Save
        // We'll place the destructive button first to match the screenshot layout.
        dialog.add_button("Close without Saving", ResponseType::Other(1));
        dialog.add_button("Cancel", ResponseType::Cancel);
        dialog.add_button("Save As...", ResponseType::Other(2));

        // Set default response to the suggested "Save As..."
        dialog.set_default_response(ResponseType::Other(2));

        // Style the dialog buttons to match the screenshot: mark the
        // "Close without Saving" button as destructive and the
        // "Save As..." button as the suggested action. We retrieve
        // the widget for each response and downcast to a Button.
        if let Some(widget) = dialog.widget_for_response(ResponseType::Other(1)) {
            if let Ok(button) = widget.downcast::<gtk4::Button>() {
                // GTK4 provides CSS classes for suggested/destructive actions
                button.add_css_class("destructive-action");
                // Add spacing to the right so there's a gap to the next button
                button.set_margin_start(8);
                button.set_margin_end(4);
                button.set_margin_bottom(8);
                button.set_margin_top(4);
            }
        }

        if let Some(widget) = dialog.widget_for_response(ResponseType::Other(2)) {
            if let Ok(button) = widget.downcast::<gtk4::Button>() {
                button.add_css_class("suggested-action");
                // Add spacing to the left so there's a gap from the previous button
                button.set_margin_start(4);
                button.set_margin_end(8);
                button.set_margin_bottom(8);
                button.set_margin_top(4);
            }
        }

        // Also add margin for the Cancel response (standard ResponseType::Cancel)
        if let Some(widget) = dialog.widget_for_response(ResponseType::Cancel) {
            if let Ok(button) = widget.downcast::<gtk4::Button>() {
                // Provide spacing on both sides so it's visually separated
                button.set_margin_start(4);
                button.set_margin_end(4);
                button.set_margin_bottom(8);
                button.set_margin_top(4);
            }
        }

        // Set dialog size using GTK native sizing - this provides better control
        // than CSS for dialog dimensions and ensures proper layout across platforms
        dialog.set_default_size(500, 160);

        // Inline CSS provider: add a small CSS snippet to make destructive
        // and suggested actions more obvious. This is kept local and added
        // before the dialog is shown so it affects the dialog buttons.
        // We scope the CSS to the dialog by using a style class on the dialog
        // itself, then target buttons with the GTK standard classes.
        let css = "
    .save-changes-dialog button.destructive-action {
        background-image: none;
        color: #ffffff;
        background-color: #d9534f; /* bootstrap danger-ish */
    }
    .save-changes-dialog button.suggested-action {
        background-image: none;
        color: #ffffff;
        /* background-color: #5cb85c;  bootstrap success-ish */
    }
    ";

        // Create a CssProvider and load the inline CSS
        let provider = gtk4::CssProvider::new();
        provider.load_from_data(css);
        // add a style class to the dialog so our CSS only affects it
        dialog.add_css_class("save-changes-dialog");
        // Add provider with a high priority so it overrides theme defaults
        // Use gtk4's helper to add a style provider for the display
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().expect("No default display"),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let response = dialog.run_future().await;

        // Explicitly close the dialog to ensure it disappears
        dialog.close();

        // Map the custom response codes to SaveChangesResult.
        // ResponseType::Other(2) -> Save (user chose Save As...)
        // ResponseType::Other(1) -> Discard (user chose Close without Saving)
        // ResponseType::Cancel or others -> Cancel
        let result = match response {
            ResponseType::Other(2) => SaveChangesResult::Save,
            ResponseType::Other(1) => SaveChangesResult::Discard,
            _ => SaveChangesResult::Cancel,
        };

        log::info!("[FileDialogs] Save changes dialog result: {:?}", result);
        Ok(result)
    }

    /// Shows a file overwrite confirmation dialog
    ///
    /// # Arguments
    /// * `parent` - Parent window for the dialog
    /// * `file_path` - Path to the file that would be overwritten
    ///
    /// # Returns
    /// * `Ok(true)` - User confirmed overwrite
    /// * `Ok(false)` - User cancelled overwrite
    /// * `Err(anyhow::Error)` - Dialog failed to show
    ///
    /// # Example
    /// ```
    /// if FileDialogs::show_overwrite_dialog(&window, &path).await? {
    ///     // Proceed with save
    /// } else {
    ///     // Show save dialog again
    /// }
    /// ```
    pub async fn show_overwrite_dialog<W: IsA<Window>>(
        parent: &W,
        file_path: &Path,
    ) -> Result<bool> {
        let filename = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("the file");

        let dialog = MessageDialog::new(
            Some(parent),
            DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
            MessageType::Question,
            ButtonsType::None,
            format!("A file named \"{}\" already exists.", filename),
        );

        dialog.set_secondary_text(Some("Do you want to replace it?"));

        // Add custom buttons
        dialog.add_button("Replace", ResponseType::Yes);
        dialog.add_button("Cancel", ResponseType::Cancel);

        // Set default response
        dialog.set_default_response(ResponseType::Cancel);

        let response = dialog.run_future().await;

        // Explicitly close the dialog to ensure it disappears
        dialog.close();

        let result = matches!(response, ResponseType::Yes);
        log::info!("[FileDialogs] Overwrite dialog result: {}", result);
        Ok(result)
    }

    /// Shows an error message dialog
    ///
    /// # Arguments
    /// * `parent` - Parent window for the dialog
    /// * `title` - Error dialog title
    /// * `message` - Primary error message
    /// * `detail` - Optional detailed error information
    ///
    /// # Example
    /// ```
    /// FileDialogs::show_error_dialog(
    ///     &window,
    ///     "Failed to Open File",
    ///     "Could not read the selected file.",
    ///     Some("Permission denied: /path/to/file.md")
    /// ).await;
    /// ```
    pub async fn show_error_dialog<W: IsA<Window>>(
        parent: &W,
        title: &str,
        message: &str,
        detail: Option<&str>,
    ) {
        let dialog = MessageDialog::new(
            Some(parent),
            DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
            MessageType::Error,
            ButtonsType::Ok,
            message,
        );

        dialog.set_title(Some(title));

        if let Some(detail) = detail {
            dialog.set_secondary_text(Some(detail));
        }

        dialog.run_future().await;

        // Explicitly close the dialog to ensure it disappears
        dialog.close();

        log::info!("[FileDialogs] Showed error dialog: {}", title);
    }

    /// Shows an information dialog
    ///
    /// # Arguments
    /// * `parent` - Parent window for the dialog
    /// * `title` - Dialog title
    /// * `message` - Information message
    ///
    /// # Example
    /// ```
    /// FileDialogs::show_info_dialog(
    ///     &window,
    ///     "File Saved",
    ///     "Your document has been saved successfully."
    /// ).await;
    /// ```
    pub async fn show_info_dialog<W: IsA<Window>>(parent: &W, title: &str, message: &str) {
        let dialog = MessageDialog::new(
            Some(parent),
            DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
            MessageType::Info,
            ButtonsType::Ok,
            message,
        );

        dialog.set_title(Some(title));
        dialog.run_future().await;

        // Explicitly close the dialog to ensure it disappears
        dialog.close();

        log::info!("[FileDialogs] Showed info dialog: {}", title);
    }
}

/// Convenience functions for common dialog operations
impl FileDialogs {
    /// Shows an open dialog and returns the selected file path
    ///
    /// This is a simplified wrapper that handles the common case.
    pub async fn open_markdown_file<W: IsA<Window>>(parent: &W) -> Option<PathBuf> {
        match Self::show_open_dialog(parent, "Open Markdown File").await {
            Ok(path) => path,
            Err(err) => {
                log::error!("[FileDialogs] Error showing open dialog: {}", err);
                None
            }
        }
    }

    /// Shows a save dialog and returns the selected file path
    ///
    /// This is a simplified wrapper that handles the common case.
    pub async fn save_markdown_file<W: IsA<Window>>(
        parent: &W,
        suggested_name: Option<&str>,
    ) -> Option<PathBuf> {
        match Self::show_save_dialog(parent, "Save Markdown File", suggested_name).await {
            Ok(path) => path,
            Err(err) => {
                eprintln!("[FileDialogs] Error showing save dialog: {}", err);
                None
            }
        }
    }

    /// Handles a file operation error by showing an appropriate dialog
    ///
    /// # Arguments
    /// * `parent` - Parent window for the dialog
    /// * `operation` - Description of what failed (e.g., "opening file")
    /// * `error` - The error that occurred
    pub async fn handle_file_error<W: IsA<Window>>(
        parent: &W,
        operation: &str,
        error: &anyhow::Error,
    ) {
        let title = format!("Error {}", operation);
        let message = format!("An error occurred while {}.", operation);
        let detail = format!("{}", error);

        Self::show_error_dialog(parent, &title, &message, Some(&detail)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_changes_result() {
        // Test that SaveChangesResult enum works as expected
        let result = SaveChangesResult::Save;
        assert_eq!(result, SaveChangesResult::Save);

        let result = SaveChangesResult::Cancel;
        assert_ne!(result, SaveChangesResult::Save);
    }

    #[test]
    fn test_dialog_creation() {
        // Basic test to ensure the module compiles
        // Actual GTK dialog testing requires a running GTK application
        // This test passes if the module compiles successfully
    }
}
