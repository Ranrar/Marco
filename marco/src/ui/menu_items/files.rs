use crate::components::language::DialogTranslations;
use crate::logic::menu_items::file::SaveChangesResult;
use gtk4::{
    prelude::*, ButtonsType, DialogFlags, MessageDialog, MessageType, ResponseType, Window,
};

#[cfg(target_os = "linux")]
use gtk4::{FileChooserAction, FileChooserNative};

use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;

// Type aliases to satisfy clippy::type_complexity
type FileDialogResult = Result<Option<PathBuf>, Box<dyn std::error::Error>>;
type SaveChangesDialogResult = Result<SaveChangesResult, Box<dyn std::error::Error>>;

type OpenDialogCallback = Arc<
    dyn for<'b> Fn(
            &'b gtk4::Window,
            &'b str,
        ) -> Pin<Box<dyn Future<Output = FileDialogResult> + 'b>>
        + Send
        + Sync
        + 'static,
>;

type SaveDialogCallback = Arc<
    dyn for<'b> Fn(
            &'b gtk4::Window,
            &'b str,
            Option<&'b str>,
        ) -> Pin<Box<dyn Future<Output = FileDialogResult> + 'b>>
        + Send
        + Sync
        + 'static,
>;

type SaveChangesCallback = Arc<
    dyn for<'b> Fn(
            &'b gtk4::Window,
            &'b str,
            &'b str,
        ) -> Pin<Box<dyn Future<Output = SaveChangesDialogResult> + 'b>>
        + Send
        + Sync
        + 'static,
>;

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
    /// Adapter for `logic::menu_items::file` callback types.
    ///
    /// The `FileOperations` code expects HRTB callbacks returning `Pin<Box<dyn Future + 'b>>`.
    /// Directly returning `Box::pin(async_fn(..))` doesn't reliably coerce to a trait object,
    /// so we box to `Box<dyn Future>` first and then pin it.
    pub fn open_dialog_callback(translations: DialogTranslations) -> OpenDialogCallback {
        Arc::new(move |parent, title| {
            let translations = translations.clone();
            let fut: Box<dyn Future<Output = FileDialogResult> + '_> = Box::new(async move {
                FileDialogs::show_open_dialog(parent, title, &translations).await
            });
            Pin::from(fut)
        })
    }

    pub fn save_dialog_callback(translations: DialogTranslations) -> SaveDialogCallback {
        Arc::new(move |parent, title, suggested_name| {
            let translations = translations.clone();
            let fut: Box<dyn Future<Output = FileDialogResult> + '_> = Box::new(async move {
                FileDialogs::show_save_dialog(parent, title, suggested_name, &translations).await
            });
            Pin::from(fut)
        })
    }

    pub fn save_changes_dialog_callback(translations: DialogTranslations) -> SaveChangesCallback {
        Arc::new(move |parent, document_name, action| {
            let translations = translations.clone();
            let fut: Box<dyn Future<Output = SaveChangesDialogResult> + '_> =
                Box::new(async move {
                    FileDialogs::show_save_changes_dialog(
                        parent,
                        document_name,
                        action,
                        &translations,
                    )
                    .await
                });
            Pin::from(fut)
        })
    }

    /// Shows a native file open dialog
    ///
    /// # Arguments
    /// * `parent` - Parent window for the dialog
    /// * `title` - Dialog title (e.g., "Open Markdown File")
    ///
    /// # Returns
    /// * `Ok(Some(PathBuf))` - User selected a file
    /// * `Ok(None)` - User cancelled the dialog
    /// * `Err(Box<dyn std::error::Error>)` - Dialog failed to show
    ///
    /// # Example
    /// ```no_run
    /// let translations = crate::components::language::SimpleLocalizationManager::new()?
    ///     .translations();
    /// let path = FileDialogs::show_open_dialog(
    ///     &window,
    ///     &translations.dialog.open_markdown_title,
    ///     &translations.dialog,
    ///     None,
    /// ).await?;
    /// if let Some(file_path) = path {
    ///     println!("Selected file: {}", file_path.display());
    /// }
    /// ```no_run
    #[cfg(target_os = "linux")]
    pub async fn show_open_dialog<W: IsA<Window>>(
        parent: &W,
        title: &str,
        translations: &DialogTranslations,
    ) -> FileDialogResult {
        let open_label = format!("_{}", translations.open_button);
        let cancel_label = format!("_{}", translations.cancel_button);
        let dialog = FileChooserNative::new(
            Some(title),
            Some(parent),
            FileChooserAction::Open,
            Some(&open_label),
            Some(&cancel_label),
        );

        // Set up file filters for markdown files
        let filter = gtk4::FileFilter::new();
        filter.set_name(Some(&translations.filter_markdown));
        filter.add_pattern("*.md");
        filter.add_pattern("*.markdown");
        filter.add_pattern("*.mdown");
        filter.add_pattern("*.mkd");
        dialog.add_filter(&filter);

        let filter_all = gtk4::FileFilter::new();
        filter_all.set_name(Some(&translations.filter_all));
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
                Err("No file selected".into())
            }
            _ => {
                log::debug!("[FileDialogs] Open dialog cancelled");
                Ok(None)
            }
        }
    }

    /// Shows a native Windows file open dialog
    ///
    /// Uses Windows native file explorer dialog for better platform integration.
    #[cfg(target_os = "windows")]
    pub async fn show_open_dialog<W: IsA<Window>>(
        _parent: &W,
        _title: &str,
        translations: &DialogTranslations,
    ) -> FileDialogResult {
        use rfd::AsyncFileDialog;

        let dialog = AsyncFileDialog::new()
            .add_filter(
                &translations.filter_markdown,
                &["md", "markdown", "mdown", "mkd"],
            )
            .add_filter(&translations.filter_all, &["*"]);

        match dialog.pick_file().await {
            Some(file) => {
                let path = file.path().to_path_buf();
                log::info!("[FileDialogs] User selected file: {}", path.display());
                Ok(Some(path))
            }
            None => {
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
    /// * `Err(Box<dyn std::error::Error>)` - Dialog failed to show
    ///
    /// # Example
    /// ```no_run
    /// let translations = crate::components::language::SimpleLocalizationManager::new()?
    ///     .translations();
    /// let path = FileDialogs::show_save_dialog(
    ///     &window,
    ///     &translations.dialog.save_markdown_title,
    ///     Some("Untitled.md"),
    ///     &translations.dialog,
    ///     None,
    /// ).await?;
    /// if let Some(file_path) = path {
    ///     println!("Save to: {}", file_path.display());
    /// }
    /// ```no_run
    #[cfg(target_os = "linux")]
    pub async fn show_save_dialog<W: IsA<Window>>(
        parent: &W,
        title: &str,
        suggested_name: Option<&str>,
        translations: &DialogTranslations,
    ) -> FileDialogResult {
        let save_label = format!("_{}", translations.save_button);
        let cancel_label = format!("_{}", translations.cancel_button);
        let dialog = FileChooserNative::new(
            Some(title),
            Some(parent),
            FileChooserAction::Save,
            Some(&save_label),
            Some(&cancel_label),
        );

        // Set up file filters
        let filter = gtk4::FileFilter::new();
        filter.set_name(Some(&translations.filter_markdown));
        filter.add_pattern("*.md");
        filter.add_pattern("*.markdown");
        dialog.add_filter(&filter);

        let filter_all = gtk4::FileFilter::new();
        filter_all.set_name(Some(&translations.filter_all));
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
                Err("No save location selected".into())
            }
            _ => {
                log::debug!("[FileDialogs] Save dialog cancelled");
                Ok(None)
            }
        }
    }

    /// Shows a native Windows file save dialog
    ///
    /// Uses Windows native file explorer dialog for better platform integration.
    #[cfg(target_os = "windows")]
    pub async fn show_save_dialog<W: IsA<Window>>(
        _parent: &W,
        _title: &str,
        suggested_name: Option<&str>,
        translations: &DialogTranslations,
    ) -> FileDialogResult {
        use rfd::AsyncFileDialog;

        let mut dialog = AsyncFileDialog::new()
            .add_filter(&translations.filter_markdown, &["md", "markdown"])
            .add_filter(&translations.filter_all, &["*"]);

        if let Some(name) = suggested_name {
            dialog = dialog.set_file_name(name);
        }

        match dialog.save_file().await {
            Some(file) => {
                let path = file.path().to_path_buf();
                log::info!("[FileDialogs] User chose save location: {}", path.display());
                Ok(Some(path))
            }
            None => {
                log::debug!("[FileDialogs] Save dialog cancelled");
                Ok(None)
            }
        }
    }

    /// Shows a "Save changes?" confirmation dialog
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
    /// * `Err(Box<dyn std::error::Error>)` - Dialog failed to show
    ///
    /// # Example
    /// ```no_run
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
        translations: &DialogTranslations,
    ) -> SaveChangesDialogResult {
        // Delegate to the dedicated save dialog module
        crate::ui::dialogs::save::show_save_changes_dialog(
            parent,
            document_name,
            action,
            translations,
        )
        .await
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
    /// * `Err(Box<dyn std::error::Error>)` - Dialog failed to show
    ///
    /// # Example
    /// ```
    /// let translations = crate::components::language::SimpleLocalizationManager::new()?
    ///     .translations();
    /// if FileDialogs::show_overwrite_dialog(&window, &path, &translations.dialog, None).await? {
    ///     // Proceed with save
    /// } else {
    ///     // Show save dialog again
    /// }
    /// ```
    pub async fn show_overwrite_dialog<W: IsA<Window>>(
        parent: &W,
        file_path: &Path,
        translations: &DialogTranslations,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let filename = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("the file");

        let primary_text = translations.overwrite_title.replace("{filename}", filename);

        let dialog = MessageDialog::new(
            Some(parent),
            DialogFlags::MODAL | DialogFlags::DESTROY_WITH_PARENT,
            MessageType::Question,
            ButtonsType::None,
            primary_text,
        );

        dialog.set_secondary_text(Some(&translations.overwrite_secondary));

        // Add custom buttons
        dialog.add_button(&translations.overwrite_replace, ResponseType::Yes);
        dialog.add_button(&translations.overwrite_cancel, ResponseType::Cancel);

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
    ///     Some("Permission denied: /path/to/file.md"),
    ///     None,
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
    ///     "Your document has been saved successfully.",
    ///     None,
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
    pub async fn open_markdown_file<W: IsA<Window>>(
        parent: &W,
        translations: &DialogTranslations,
    ) -> Option<PathBuf> {
        match Self::show_open_dialog(parent, &translations.open_markdown_title, translations).await
        {
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
        translations: &DialogTranslations,
    ) -> Option<PathBuf> {
        match Self::show_save_dialog(
            parent,
            &translations.save_markdown_title,
            suggested_name,
            translations,
        )
        .await
        {
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
        error: &dyn std::error::Error,
        translations: &DialogTranslations,
    ) {
        let title = format!("{} {}", translations.error_title_prefix, operation);
        let message = format!("{} {}.", translations.error_message_prefix, operation);
        let detail = error.to_string();

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
