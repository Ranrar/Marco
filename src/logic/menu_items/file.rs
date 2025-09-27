//! File operations module for handling document lifecycle
#![allow(clippy::await_holding_refcell_ref)]

use crate::logic::{DocumentBuffer, RecentFiles};
use anyhow::Result;
use gtk4::{gio, glib, prelude::*};
use log::trace;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

// Type aliases to simplify complex callback signatures
type OpenDialogCallback = Arc<
    dyn for<'b> Fn(
            &'b gtk4::Window,
            &'b str,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Option<std::path::PathBuf>>> + 'b>,
        > + Send
        + Sync
        + 'static,
>;

type SaveChangesDialogCallback = Arc<
    dyn for<'b> Fn(
            &'b gtk4::Window,
            &'b str,
            &'b str,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<SaveChangesResult>> + 'b>,
        > + Send
        + Sync
        + 'static,
>;

type SaveDialogCallback = Arc<
    dyn for<'b> Fn(
            &'b gtk4::Window,
            &'b str,
            Option<&'b str>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Option<std::path::PathBuf>>> + 'b>,
        > + Send
        + Sync
        + 'static,
>;

// Allow RefCell borrows across await points in this module since we're in single-threaded GTK context

/// File operations manager for handling document lifecycle
///
/// This struct provides all file-related operations including:
/// - Creating new documents
/// - Opening existing files
/// - Saving documents (Save and Save As)
/// - Managing recent files
/// - Handling unsaved changes prompts
///
/// # Usage
/// This is designed to work with GTK4 applications where the buffer
/// and editor are shared via Rc<RefCell<T>> for thread-safe access
/// on the main thread.
pub struct FileOperations {
    /// Document buffer containing current file state
    pub buffer: Rc<RefCell<DocumentBuffer>>,
    /// Recent files manager
    pub recent_files: Rc<RefCell<RecentFiles>>,
    /// Callbacks to run when the recent files list changes
    recent_changed_callbacks: RefCell<Vec<Box<dyn Fn()>>>,
    /// Last recent files list to prevent redundant updates
    last_recent_files: RefCell<Option<Vec<std::path::PathBuf>>>,
    /// Flag to prevent recursive callback cascades
    updating_recent_files: RefCell<bool>,
}

impl FileOperations {
    /// Creates a new file operations manager
    ///
    /// # Arguments
    /// * `buffer` - Shared document buffer
    /// * `recent_files` - Shared recent files manager
    ///
    /// # Example
    /// ```
    /// use std::rc::Rc;
    /// use std::cell::RefCell;
    /// use marco::logic::buffer::{DocumentBuffer, RecentFiles};
    /// use marco::logic::menu_items::file::FileOperations;
    ///
    /// let buffer = Rc::new(RefCell::new(DocumentBuffer::new_untitled()));
    /// let recent = Rc::new(RefCell::new(RecentFiles::default()));
    /// let file_ops = FileOperations::new(buffer, recent);
    /// ```
    pub fn new(
        buffer: Rc<RefCell<DocumentBuffer>>,
        recent_files: Rc<RefCell<RecentFiles>>,
    ) -> Self {
        Self {
            buffer,
            recent_files,
            recent_changed_callbacks: RefCell::new(Vec::new()),
            last_recent_files: RefCell::new(None),
            updating_recent_files: RefCell::new(false),
        }
    }

    /// Register a callback to be invoked whenever the recent files list changes
    pub fn register_recent_changed_callback<F: Fn() + 'static>(&self, cb: F) {
        self.recent_changed_callbacks
            .borrow_mut()
            .push(Box::new(cb));
        log::debug!("Registered recent files callback (total: {})", self.recent_changed_callbacks.borrow().len());
    }

    /// Clear cascade prevention state (useful for testing or manual reset)
    #[allow(dead_code)]
    pub fn reset_cascade_prevention(&self) {
        *self.last_recent_files.borrow_mut() = None;
        *self.updating_recent_files.borrow_mut() = false;
        log::debug!("Reset cascade prevention state");
    }

    /// Get current callback statistics for debugging cascade issues
    #[allow(dead_code)]
    pub fn get_callback_stats(&self) -> (usize, bool, usize) {
        let callback_count = self.recent_changed_callbacks.borrow().len();
        let is_updating = *self.updating_recent_files.borrow();
        let cached_files_count = self.last_recent_files.borrow()
            .as_ref()
            .map(|files| files.len())
            .unwrap_or(0);
        (callback_count, is_updating, cached_files_count)
    }

    fn invoke_recent_changed_callbacks(&self) {
        // Prevent recursive callback cascades
        if *self.updating_recent_files.borrow() {
            log::debug!("Skipping recent files callbacks (already updating)");
            return;
        }

        // Get current recent files list
        let current_files = self.recent_files.borrow().get_files();
        
        // Check if the list actually changed to prevent redundant updates
        if let Some(ref last_files) = *self.last_recent_files.borrow() {
            if last_files == &current_files {
                log::debug!("Skipping recent files callbacks (list unchanged)");
                return;
            }
        }

        log::debug!("Invoking recent files callbacks for {} files", current_files.len());
        
        // Set flag to prevent recursive calls during callback processing
        *self.updating_recent_files.borrow_mut() = true;
        
        // Cache the current list to prevent redundant updates
        *self.last_recent_files.borrow_mut() = Some(current_files);
        
        // Invoke all registered callbacks
        for (i, cb) in self.recent_changed_callbacks.borrow().iter().enumerate() {
            log::debug!("Invoking recent files callback {}", i);
            cb();
        }
        
        // Clear the updating flag
        *self.updating_recent_files.borrow_mut() = false;
    }

    /// Add file to the recent list and notify callbacks
    fn add_recent_file<P: AsRef<Path>>(&self, path: P) {
        let path = path.as_ref();
        
        // Check if this file is already at the top of the recent list to avoid redundant updates
        let current_files = self.recent_files.borrow().get_files();
        if let Some(first_file) = current_files.first() {
            if first_file == path {
                log::debug!("File {} already at top of recent list, skipping update", path.display());
                return;
            }
        }
        
        log::debug!("Adding file to recent list: {}", path.display());
        self.recent_files.borrow_mut().add_file(path);
        self.invoke_recent_changed_callbacks();
    }

    /// Opens a specific file by path (async version with proper Save dialog support)
    ///
    /// This is used for recent files and command-line arguments.
    ///
    /// # Arguments
    /// * `path` - Path to the file to open
    /// * `parent_window` - Parent window for error dialogs
    /// * `editor_buffer` - GTK TextBuffer to populate
    /// * `show_save_changes_dialog` - Callback for save changes dialog
    /// * `show_save_dialog` - Callback for save as dialog
    ///
    /// # Returns
    /// * `Ok(())` - File opened successfully
    /// * `Err(anyhow::Error)` - Operation failed
    pub async fn open_file_by_path_async<'a, P, W, F, G>(
        &self,
        path: P,
        parent_window: &'a W,
        editor_buffer: &'a gtk4::TextBuffer,
        show_save_changes_dialog: F,
        show_save_dialog: G,
    ) -> Result<()>
    where
        P: AsRef<Path>,
        W: IsA<gtk4::Window>,
        F: for<'b> Fn(
            &'b gtk4::Window,
            &'b str,
            &'b str,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<SaveChangesResult>> + 'b>,
        >,
        G: for<'b> Fn(
            &'b gtk4::Window,
            &'b str,
            Option<&'b str>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Option<std::path::PathBuf>>> + 'b>,
        >,
    {
        let path = path.as_ref();

        // Check for unsaved changes and prompt user
        if self.buffer.borrow().has_unsaved_changes() {
            let document_title = self.get_document_title();
            match show_save_changes_dialog(
                parent_window.upcast_ref(),
                &document_title,
                "opening a file",
            )
            .await?
            {
                SaveChangesResult::Save => {
                    // If the document already has a file path, do a normal save
                    if self.buffer.borrow().get_file_path().is_some() {
                        self.save_document(parent_window.upcast_ref(), editor_buffer)?;
                    } else {
                        // Show Save As dialog for new/untitled files
                        let suggested_name = if self.get_document_title().contains("Untitled") {
                            "Untitled.md"
                        } else {
                            &format!("{}.md", self.get_document_title().replace("*", "").trim())
                        };

                        let file_path = show_save_dialog(
                            parent_window.upcast_ref(),
                            "Save Markdown File",
                            Some(suggested_name),
                        )
                        .await?;
                        if let Some(save_path) = file_path {
                            let content = self.get_editor_content(editor_buffer);
                            self.buffer
                                .borrow_mut()
                                .save_as_content(&save_path, &content)?;
                            self.buffer.borrow_mut().set_baseline(&content);
                            self.add_recent_file(&save_path);
                        } else {
                            // User cancelled Save As dialog, cancel the entire open operation
                            return Err(anyhow::anyhow!(
                                "Save As cancelled, open operation aborted"
                            ));
                        }
                    }
                }
                SaveChangesResult::Discard => {
                    // Continue with open operation
                }
                SaveChangesResult::Cancel => {
                    return Err(anyhow::anyhow!("Open file cancelled by user"));
                }
            }
        }

        self.load_file_into_editor(path, editor_buffer)?;
        trace!("audit: opened file_by_path: {}", path.display());
        eprintln!("[FileOps] Opened file by path: {}", path.display());
        Ok(())
    }

    /// Saves the current document
    ///
    /// If the document is untitled, this will return an error since it can't show dialogs.
    /// Use the async save action instead for new documents.
    ///
    /// # Arguments
    /// * `parent_window` - Parent window for dialogs (unused)
    /// * `editor_buffer` - GTK TextBuffer to get content from
    ///
    /// # Returns
    /// * `Ok(())` - File saved successfully
    /// * `Err(anyhow::Error)` - Operation failed or document is untitled
    pub fn save_document<W: IsA<gtk4::Window>>(
        &self,
        _parent_window: &W,
        editor_buffer: &gtk4::TextBuffer,
    ) -> Result<()> {
        let buffer = self.buffer.borrow();
        if buffer.get_file_path().is_some() {
            drop(buffer); // Release borrow before calling save_content

            let content = self.get_editor_content(editor_buffer);
            self.buffer.borrow_mut().save_content(&content)?;
            // Update baseline after successful save
            self.buffer.borrow_mut().set_baseline(&content);

            trace!("audit: saved document to existing path");
            eprintln!("[FileOps] Saved document");
            Ok(())
        } else {
            // For untitled documents, we need to use the async save action with dialogs
            Err(anyhow::anyhow!(
                "Cannot save untitled document synchronously - use async save action instead"
            ))
        }
    }

    /// Gets the list of recent files for menu display
    ///
    /// # Returns
    /// Vector of recent file paths (most recent first)
    pub fn get_recent_files(&self) -> Vec<std::path::PathBuf> {
        self.recent_files.borrow().get_files().to_vec()
    }

    /// Clears all recent files
    pub fn clear_recent_files(&self) {
        self.recent_files.borrow_mut().clear();
        // Notify listeners so menus update
        self.invoke_recent_changed_callbacks();
        trace!("audit: cleared recent files");
    }

    /// Update modified flag by comparing current editor content to baseline
    pub fn mark_document_modified_from_content(&self, current_content: &str) {
        self.buffer
            .borrow_mut()
            .update_modified_from_content(current_content);
    }

    /// Gets the current document's display title
    ///
    /// # Returns
    /// String suitable for window title (includes * for modified files)
    pub fn get_document_title(&self) -> String {
        self.buffer.borrow().get_full_title()
    }

    /// Async open file operation using dialog callbacks
    pub async fn open_file_async<'a, F, G, H>(
        &self,
        parent_window: &'a gtk4::Window,
        editor_buffer: &'a gtk4::TextBuffer,
        show_open_dialog: F,
        show_save_changes_dialog: G,
        show_save_dialog: H,
    ) -> Result<()>
    where
        F: for<'b> Fn(
            &'b gtk4::Window,
            &'b str,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Option<std::path::PathBuf>>> + 'b>,
        >,
        G: for<'b> Fn(
            &'b gtk4::Window,
            &'b str,
            &'b str,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<SaveChangesResult>> + 'b>,
        >,
        H: for<'b> Fn(
            &'b gtk4::Window,
            &'b str,
            Option<&'b str>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Option<std::path::PathBuf>>> + 'b>,
        >,
    {
        // Check for unsaved changes and prompt user
        if self.buffer.borrow().has_unsaved_changes() {
            let document_title = self.get_document_title();
            match show_save_changes_dialog(parent_window, &document_title, "opening a file").await?
            {
                SaveChangesResult::Save => {
                    // If the document already has a file path, do a normal save
                    if self.buffer.borrow().get_file_path().is_some() {
                        self.save_document(parent_window, editor_buffer)?;
                    } else {
                        // Show Save As dialog for new/untitled files
                        let suggested_name = if self.get_document_title().contains("Untitled") {
                            "Untitled.md"
                        } else {
                            &format!("{}.md", self.get_document_title().replace("*", "").trim())
                        };

                        let file_path = show_save_dialog(
                            parent_window,
                            "Save Markdown File",
                            Some(suggested_name),
                        )
                        .await?;
                        if let Some(path) = file_path {
                            let content = self.get_editor_content(editor_buffer);
                            self.buffer.borrow_mut().save_as_content(&path, &content)?;
                            self.buffer.borrow_mut().set_baseline(&content);
                            self.add_recent_file(&path);
                        } else {
                            // User cancelled Save As dialog, cancel the entire open operation
                            return Err(anyhow::anyhow!(
                                "Save As cancelled, open operation aborted"
                            ));
                        }
                    }
                }
                SaveChangesResult::Discard => {
                    // Continue with open operation
                }
                SaveChangesResult::Cancel => {
                    return Err(anyhow::anyhow!("Open file cancelled by user"));
                }
            }
        }

        let file_path = show_open_dialog(parent_window, "Open Markdown File").await?;
        if let Some(path) = file_path {
            self.load_file_into_editor(&path, editor_buffer)?;
            self.add_recent_file(&path);
            trace!("audit: opened file via dialog: {}", path.display());
            eprintln!("[FileOps] Opened file: {}", path.display());
        }
        Ok(())
    }

    /// Async new document operation using dialog callback
    pub async fn new_document_async<'a, F, G>(
        &self,
        parent_window: &'a gtk4::Window,
        editor_buffer: &'a gtk4::TextBuffer,
        show_save_changes_dialog: F,
        show_save_dialog: G,
    ) -> Result<()>
    where
        F: for<'b> Fn(
            &'b gtk4::Window,
            &'b str,
            &'b str,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<SaveChangesResult>> + 'b>,
        >,
        G: for<'b> Fn(
            &'b gtk4::Window,
            &'b str,
            Option<&'b str>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Option<std::path::PathBuf>>> + 'b>,
        >,
    {
        // Check for unsaved changes and prompt user
        if self.buffer.borrow().has_unsaved_changes() {
            let document_title = self.get_document_title();
            match show_save_changes_dialog(
                parent_window,
                &document_title,
                "starting a new document",
            )
            .await?
            {
                SaveChangesResult::Save => {
                    // If the document already has a file path, do a normal save
                    if self.buffer.borrow().get_file_path().is_some() {
                        self.save_document(parent_window, editor_buffer)?;
                    } else {
                        // Show Save As dialog for new/untitled files
                        let suggested_name = if self.get_document_title().contains("Untitled") {
                            "Untitled.md"
                        } else {
                            &format!("{}.md", self.get_document_title().replace("*", "").trim())
                        };

                        let file_path = show_save_dialog(
                            parent_window,
                            "Save Markdown File",
                            Some(suggested_name),
                        )
                        .await?;
                        if let Some(path) = file_path {
                            let content = self.get_editor_content(editor_buffer);
                            self.buffer.borrow_mut().save_as_content(&path, &content)?;
                            self.buffer.borrow_mut().set_baseline(&content);
                            self.add_recent_file(&path);
                        } else {
                            // User cancelled Save As dialog, cancel the new document operation
                            return Err(anyhow::anyhow!(
                                "Save As cancelled, new document operation aborted"
                            ));
                        }
                    }
                }
                SaveChangesResult::Discard => {
                    // Continue with new document
                }
                SaveChangesResult::Cancel => {
                    return Err(anyhow::anyhow!("New document cancelled by user"));
                }
            }
        }

        // Reset buffer and clear editor
        self.buffer.borrow_mut().reset_to_untitled();
        editor_buffer.set_text("");
        trace!("audit: created new untitled document");
        eprintln!("[FileOps] Created new untitled document");
        Ok(())
    }

    /// Async save as operation using dialog callback
    pub async fn save_as_async<'a, F>(
        &self,
        parent_window: &'a gtk4::Window,
        editor_buffer: &'a gtk4::TextBuffer,
        show_save_dialog: F,
    ) -> Result<()>
    where
        F: for<'b> Fn(
            &'b gtk4::Window,
            &'b str,
            Option<&'b str>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Option<std::path::PathBuf>>> + 'b>,
        >,
    {
        let suggested_name = if self.get_document_title().contains("Untitled") {
            "Untitled.md".to_string()
        } else {
            format!("{}.md", self.get_document_title().replace("*", "").trim())
        };
        let file_path =
            show_save_dialog(parent_window, "Save Markdown File", Some(&suggested_name)).await?;
        if let Some(path) = file_path {
            let start_iter = editor_buffer.start_iter();
            let end_iter = editor_buffer.end_iter();
            let content = editor_buffer
                .text(&start_iter, &end_iter, false)
                .to_string();
            self.buffer.borrow_mut().save_as_content(&path, &content)?;
            self.add_recent_file(&path);
            eprintln!("[FileOps] Saved file: {}", path.display());
        }
        Ok(())
    }

    /// Async quit operation using dialog callback
    pub async fn quit_async<'a, F, G>(
        &self,
        parent_window: &'a gtk4::Window,
        editor_buffer: &'a gtk4::TextBuffer,
        app: &'a gtk4::Application,
        show_save_changes_dialog: F,
        show_save_dialog: G,
    ) -> Result<()>
    where
        F: for<'b> Fn(
            &'b gtk4::Window,
            &'b str,
            &'b str,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<SaveChangesResult>> + 'b>,
        >,
        G: for<'b> Fn(
            &'b gtk4::Window,
            &'b str,
            Option<&'b str>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Option<std::path::PathBuf>>> + 'b>,
        >,
    {
        let is_modified = self.buffer.borrow().has_unsaved_changes();
        let document_title = self.get_document_title();
        if is_modified {
            match show_save_changes_dialog(parent_window, &document_title, "quitting").await? {
                SaveChangesResult::Save => {
                    let has_file_path = self.buffer.borrow().get_file_path().is_some();
                    if !has_file_path {
                        let suggested_name = if self.get_document_title().contains("Untitled") {
                            "Untitled.md".to_string()
                        } else {
                            format!("{}.md", self.get_document_title().replace("*", "").trim())
                        };
                        let file_path = show_save_dialog(
                            parent_window,
                            "Save Markdown File",
                            Some(&suggested_name),
                        )
                        .await?;
                        if let Some(path) = file_path {
                            let start_iter = editor_buffer.start_iter();
                            let end_iter = editor_buffer.end_iter();
                            let content = editor_buffer
                                .text(&start_iter, &end_iter, false)
                                .to_string();
                            self.buffer.borrow_mut().save_as_content(&path, &content)?;
                            self.add_recent_file(&path);
                            app.quit();
                        }
                    } else {
                        self.save_document(parent_window, editor_buffer)?;
                        app.quit();
                    }
                }
                SaveChangesResult::Discard => {
                    app.quit();
                }
                SaveChangesResult::Cancel => {
                    eprintln!("[FileDialog] Quit cancelled by user");
                }
            }
        } else {
            app.quit();
        }
        Ok(())
    }

    // Private helper methods

    /// Loads a file into the editor buffer
    fn load_file_into_editor<P: AsRef<Path>>(
        &self,
        path: P,
        editor_buffer: &gtk4::TextBuffer,
    ) -> Result<()> {
        let path = path.as_ref();

        // Create new buffer from file
        let mut new_buffer = DocumentBuffer::new_from_file(path)?;
        let content = new_buffer.load_and_set_baseline()?; // Use optimized method

        // Update our DocumentBuffer FIRST (before editor buffer to ensure preview refresh sees the updated state)
        *self.buffer.borrow_mut() = new_buffer;

        // Update editor (this will trigger preview refresh which should now see the updated DocumentBuffer)
        editor_buffer.set_text(&content);

        // Add to recent files
        self.add_recent_file(path);

        Ok(())
    }

    /// Gets the current content from the editor buffer
    fn get_editor_content(&self, editor_buffer: &gtk4::TextBuffer) -> String {
        let start_iter = editor_buffer.start_iter();
        let end_iter = editor_buffer.end_iter();
        editor_buffer
            .text(&start_iter, &end_iter, false)
            .to_string()
    }



    /// Load an initial file on application startup (for command line arguments)
    ///
    /// This method spawns an async task to load the specified file and update the UI.
    /// It's designed to be called during application initialization.
    ///
    /// # Arguments
    /// * `file_path` - Path to the file to open
    /// * `window` - Application window for dialog parenting
    /// * `editor_buffer` - GTK TextBuffer to populate
    /// * `title_label` - Label to update with document title
    /// * `show_save_changes_dialog` - Callback for save changes dialog
    /// * `show_save_dialog` - Callback for save as dialog
    pub fn load_initial_file_async<F, G>(
        file_operations: Rc<RefCell<Self>>,
        file_path: String,
        window: gtk4::ApplicationWindow,
        editor_buffer: sourceview5::Buffer,
        title_label: gtk4::Label,
        show_save_changes_dialog: F,
        show_save_dialog: G,
    ) where
        F: for<'a> Fn(
                &'a gtk4::Window,
                &'a str,
                &'a str,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<SaveChangesResult>> + 'a>,
            > + 'static,
        G: for<'a> Fn(
                &'a gtk4::Window,
                &'a str,
                Option<&'a str>,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<Option<std::path::PathBuf>>> + 'a>,
            > + 'static,
    {
        glib::MainContext::default().spawn_local(async move {
            #[allow(clippy::await_holding_refcell_ref)]
            let file_ops = file_operations.borrow();
            let gtk_window: &gtk4::Window = window.upcast_ref();
            let text_buffer: &gtk4::TextBuffer = editor_buffer.upcast_ref();

            // Try to open the specified file
            let result = file_ops
                .open_file_by_path_async(
                    &file_path,
                    gtk_window,
                    text_buffer,
                    show_save_changes_dialog,
                    show_save_dialog,
                )
                .await;

            match result {
                Ok(_) => {
                    // Update title label after successful open
                    let title = file_operations.borrow().get_document_title();
                    title_label.set_text(&title);
                    // Successful open - terminal output suppressed.
                }
                Err(e) => {
                    eprintln!("Failed to open file {}: {}", file_path, e);
                }
            }
        });
    }
}

/// Result of the "Save Changes?" prompt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveChangesResult {
    /// User chose to save the document
    Save,
    /// User chose to discard changes
    Discard,
    /// User cancelled the operation
    Cancel,
}

/// Updates the recent files submenu
///
/// # Arguments
/// * `file_menu` - The file menu to update
/// * `recent_files` - List of recent file paths
pub fn update_recent_files_menu(recent_menu: &gio::Menu, recent_files: &[std::path::PathBuf]) {
    // Clear existing items
    while recent_menu.n_items() > 0 {
        recent_menu.remove(0);
    }

    if recent_files.is_empty() {
        recent_menu.append(Some("No recent files"), None);
    } else {
        for (i, path) in recent_files.iter().enumerate() {
            if i >= 5 {
                break;
            } // Limit to 5 recent files

            let filename = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Unknown");

            let action_name = format!("app.open_recent_{}", i);
            recent_menu.append(Some(filename), Some(&action_name));
        }

        // Add separator and clear option
        recent_menu.append(None, None);
        recent_menu.append(Some("Clear Recent Files"), Some("app.clear_recent"));
    }

    // Recent files menu updated (debug output suppressed).
}

/// Attach change tracker to the editor buffer so document modified state
/// is updated on user edits. This centralizes the tracking wiring so
/// callers (e.g. `main.rs`) don't have to duplicate the closure logic.
pub fn attach_change_tracker(
    file_operations: Rc<RefCell<FileOperations>>,
    editor_buffer: &sourceview5::Buffer,
    modification_tracking_enabled: Rc<RefCell<bool>>,
    title_label: &gtk4::Label,
) -> gtk4::glib::SignalHandlerId {
    // Clone buffer for closure capture
    let editor_buffer_clone = editor_buffer.clone();
    editor_buffer_clone.connect_changed({
        let file_operations = file_operations.clone();
        let tracking_enabled = modification_tracking_enabled.clone();
        let title_label = title_label.clone();
        let editor_buffer = editor_buffer_clone.clone();
        move |_| {
            // Only track changes when not loading a file programmatically
            if *tracking_enabled.borrow() {
                if let Ok(file_ops) = file_operations.try_borrow() {
                    // Compare current editor content to the baseline and update modified flag
                    let start_iter = editor_buffer.start_iter();
                    let end_iter = editor_buffer.end_iter();
                    let content = editor_buffer
                        .text(&start_iter, &end_iter, false)
                        .to_string();
                    file_ops.mark_document_modified_from_content(&content);
                    // Update visible title label
                    let title = file_ops.get_document_title();
                    title_label.set_text(&title);
                    trace!("audit: editor buffer changed (user edit detected)");
                    if std::env::var("MARCO_DEBUG_POINTERS").is_ok() {
                        eprintln!(
                            "[file_ops] title_label ptr={:p} set_text='{}'",
                            title_label.as_ptr(),
                            title
                        );
                    }
                }
            }
        }
    })
}

/// Register all file actions including async dialogs and modification tracking.
///
/// This function sets up all file operations, keyboard shortcuts, and change tracking
/// to centralize all file-related setup in one place.
///
/// The callbacks are boxed functions that return boxed futures. Callers
/// (main.rs) can pass UI dialog functions (Box::pin(...)) to integrate GTK dialogs.
#[allow(clippy::too_many_arguments)]
pub fn register_file_actions_async(
    app: gtk4::Application,
    file_operations: Rc<RefCell<FileOperations>>,
    window: &gtk4::ApplicationWindow,
    editor_buffer: &sourceview5::Buffer,
    title_label: &gtk4::Label,
    show_open_dialog: OpenDialogCallback,
    show_save_changes_dialog: SaveChangesDialogCallback,
    show_save_dialog: SaveDialogCallback,
) {
    // Create modification tracking flag
    let modification_tracking_enabled = Rc::new(RefCell::new(true));

    // Set up buffer change tracking - TODO: store and manage this signal ID
    let _change_tracker_signal_id = attach_change_tracker(
        file_operations.clone(),
        editor_buffer,
        modification_tracking_enabled.clone(),
        title_label,
    );
    // Create open action (async)
    let open_action = gio::SimpleAction::new("open", None);
    open_action.connect_activate({
        let file_ops = file_operations.clone();
        let window = window.clone();
        let editor_buffer = editor_buffer.clone();
        let title_label = title_label.clone();
        let show_open_dialog = Arc::clone(&show_open_dialog);
        let show_save_changes_dialog = Arc::clone(&show_save_changes_dialog);
        let show_save_dialog = Arc::clone(&show_save_dialog);
        move |_, _| {
            let file_ops = file_ops.clone();
            let window = window.clone();
            let editor_buffer = editor_buffer.clone();
            let title_label = title_label.clone();
            let show_open_dialog = Arc::clone(&show_open_dialog);
            let show_save_changes_dialog = Arc::clone(&show_save_changes_dialog);
            let show_save_dialog = Arc::clone(&show_save_dialog);
            glib::MainContext::default().spawn_local(async move {
                #[allow(clippy::await_holding_refcell_ref)]
                let file_ops_ref = file_ops.borrow();
                let gtk_window: &gtk4::Window = window.upcast_ref();
                let text_buffer: &gtk4::TextBuffer = editor_buffer.upcast_ref();
                let _ = file_ops_ref
                    .open_file_async(
                        gtk_window,
                        text_buffer,
                        |w, title| (show_open_dialog)(w, title),
                        |w, doc_name, action| (show_save_changes_dialog)(w, doc_name, action),
                        |w, title, suggested| (show_save_dialog)(w, title, suggested),
                    )
                    .await;
                // Update title label after open completes
                let title = file_ops.borrow().get_document_title();
                title_label.set_text(&title);
            });
        }
    });

    // New document action (async)
    let new_action = gio::SimpleAction::new("new", None);
    new_action.connect_activate({
        let file_ops = file_operations.clone();
        let window = window.clone();
        let editor_buffer = editor_buffer.clone();
        let title_label = title_label.clone();
        let tracking_enabled = modification_tracking_enabled.clone();
        let show_save_changes_dialog = Arc::clone(&show_save_changes_dialog);
        let show_save_dialog = Arc::clone(&show_save_dialog);
        move |_, _| {
            let file_ops = file_ops.clone();
            let window = window.clone();
            let editor_buffer = editor_buffer.clone();
            let title_label = title_label.clone();
            let tracking_enabled = tracking_enabled.clone();
            let show_save_changes_dialog = Arc::clone(&show_save_changes_dialog);
            let show_save_dialog = Arc::clone(&show_save_dialog);
            glib::MainContext::default().spawn_local(async move {
                trace!("audit: action new document triggered");
                *tracking_enabled.borrow_mut() = false;
                #[allow(clippy::await_holding_refcell_ref)]
                let file_ops_ref = file_ops.borrow();
                let gtk_window: &gtk4::Window = window.upcast_ref();
                let text_buffer: &gtk4::TextBuffer = editor_buffer.upcast_ref();
                let _ = file_ops_ref
                    .new_document_async(
                        gtk_window,
                        text_buffer,
                        |w, doc_name, action| (show_save_changes_dialog)(w, doc_name, action),
                        |w, title, suggested| (show_save_dialog)(w, title, suggested),
                    )
                    .await;
                // Update title label after new document is created
                let title = file_ops.borrow().get_document_title();
                title_label.set_text(&title);
                *tracking_enabled.borrow_mut() = true;
            });
        }
    });

    // Save As action
    let save_as_action = gio::SimpleAction::new("save_as", None);
    save_as_action.connect_activate({
        let file_ops = file_operations.clone();
        let window = window.clone();
        let editor_buffer = editor_buffer.clone();
        let title_label = title_label.clone();
        let show_save_dialog = Arc::clone(&show_save_dialog);
        move |_, _| {
            let file_ops = file_ops.clone();
            let window = window.clone();
            let editor_buffer = editor_buffer.clone();
            let title_label = title_label.clone();
            let show_save_dialog = Arc::clone(&show_save_dialog);
            glib::MainContext::default().spawn_local(async move {
                #[allow(clippy::await_holding_refcell_ref)]
                let file_ops_ref = file_ops.borrow();
                let gtk_window: &gtk4::Window = window.upcast_ref();
                let text_buffer: &gtk4::TextBuffer = editor_buffer.upcast_ref();
                let _ = file_ops_ref
                    .save_as_async(gtk_window, text_buffer, |w, title, suggested| {
                        (show_save_dialog)(w, title, suggested)
                    })
                    .await;
                // Update title label after Save As completes
                let title = file_ops.borrow().get_document_title();
                title_label.set_text(&title);
            });
        }
    });

    // Quit action
    let quit_action = gio::SimpleAction::new("quit", None);
    quit_action.connect_activate({
        let file_ops = file_operations.clone();
        let window = window.clone();
        let editor_buffer = editor_buffer.clone();
        let app = app.clone();
        let show_save_changes_dialog = Arc::clone(&show_save_changes_dialog);
        let show_save_dialog = Arc::clone(&show_save_dialog);
        move |_, _| {
            let file_ops = file_ops.clone();
            let window = window.clone();
            let editor_buffer = editor_buffer.clone();
            let app = app.clone();
            let show_save_changes_dialog = Arc::clone(&show_save_changes_dialog);
            let show_save_dialog = Arc::clone(&show_save_dialog);
            glib::MainContext::default().spawn_local(async move {
                #[allow(clippy::await_holding_refcell_ref)]
                let file_ops_ref = file_ops.borrow();
                let gtk_window: &gtk4::Window = window.upcast_ref();
                let text_buffer: &gtk4::TextBuffer = editor_buffer.upcast_ref();
                let _ = file_ops_ref
                    .quit_async(
                        gtk_window,
                        text_buffer,
                        &app,
                        |w, title, action| (show_save_changes_dialog)(w, title, action),
                        |w, title, suggested| (show_save_dialog)(w, title, suggested),
                    )
                    .await;
            });
        }
    });

    // Save action (async to support proper dialogs)
    let save_action = gio::SimpleAction::new("save", None);
    save_action.connect_activate({
        let file_ops = file_operations.clone();
        let window = window.clone();
        let editor_buffer = editor_buffer.clone();
        let title_label = title_label.clone();
        let show_save_dialog = Arc::clone(&show_save_dialog);
        move |_, _| {
            let file_ops = file_ops.clone();
            let window = window.clone();
            let editor_buffer = editor_buffer.clone();
            let title_label = title_label.clone();
            let show_save_dialog = Arc::clone(&show_save_dialog);
            glib::MainContext::default().spawn_local(async move {
                #[allow(clippy::await_holding_refcell_ref)]
                let file_ops_ref = file_ops.borrow();
                let gtk_window: &gtk4::Window = window.upcast_ref();
                let text_buffer: &gtk4::TextBuffer = editor_buffer.upcast_ref();
                
                // Check if document has a file path already
                if file_ops_ref.buffer.borrow().get_file_path().is_some() {
                    // Use the synchronous save for existing files
                    if let Err(e) = file_ops_ref.save_document(gtk_window, text_buffer) {
                        eprintln!("Error saving document: {}", e);
                    } else {
                        // Update title after successful save
                        let title = file_ops.borrow().get_document_title();
                        title_label.set_text(&title);
                    }
                } else {
                    // Use async save as for new files (shows proper dialog)
                    let result = file_ops_ref.save_as_async(
                        gtk_window,
                        text_buffer,
                        |w, title, suggested| (show_save_dialog)(w, title, suggested),
                    ).await;
                    
                    match result {
                        Ok(_) => {
                            let title = file_ops.borrow().get_document_title();
                            title_label.set_text(&title);
                        }
                        Err(e) => {
                            eprintln!("Error saving document: {}", e);
                        }
                    }
                }
            });
        }
    });

    // Add actions to application
    app.add_action(&new_action);
    app.add_action(&open_action);
    app.add_action(&save_action);
    app.add_action(&save_as_action);
    app.add_action(&quit_action);

    // Clear recent action
    let recent_list = file_operations.borrow().get_recent_files();
    let clear_recent_action = gio::SimpleAction::new("clear_recent", None);
    clear_recent_action.set_enabled(!recent_list.is_empty());
    let file_ops_for_clear = file_operations.clone();
    clear_recent_action.connect_activate(move |_, _| {
        trace!("audit: clear recent files action triggered");
        file_ops_for_clear.borrow().clear_recent_files();
        eprintln!("[main] Cleared recent files");
    });
    app.add_action(&clear_recent_action);

    // Set keyboard shortcuts for file actions
    app.set_accels_for_action("app.new", &["<Control>n"]);
    app.set_accels_for_action("app.open", &["<Control>o"]);
    app.set_accels_for_action("app.save", &["<Control>s"]);
    app.set_accels_for_action("app.save_as", &["<Control><Shift>s"]);
    app.set_accels_for_action("app.quit", &["<Control>q"]);

    // Dynamic recent-file registration is provided by `setup_recent_actions`
}

/// Helper function to update recent file actions and menu
#[allow(clippy::too_many_arguments)]
fn update_recent_file_actions(
    app: &gtk4::Application,
    file_operations: &Rc<RefCell<FileOperations>>,
    recent_menu: &gio::Menu,
    window: &gtk4::ApplicationWindow,
    editor_buffer: &sourceview5::Buffer,
    title_label: &gtk4::Label,
    show_save_changes_dialog: &SaveChangesDialogCallback,
    show_save_dialog: &SaveDialogCallback,
    recent_action: &gio::SimpleAction,
) {
    let list = file_operations.borrow().get_recent_files();
    crate::logic::menu_items::file::update_recent_files_menu(recent_menu, &list);
    recent_action.set_enabled(!list.is_empty());

    // Remove old actions
    for i in 0..5 {
        let name = format!("open_recent_{}", i);
        if app.lookup_action(&name).is_some() {
            app.remove_action(&name);
        }
    }

    // Register new actions
    for (i, path) in list.iter().enumerate() {
        if i >= 5 {
            break;
        }
        let action_name = format!("open_recent_{}", i);
        let app_action = gio::SimpleAction::new(&action_name, None);
        app_action.set_enabled(true);
        let file_ops_for_action = file_operations.clone();
        let window_for_action = window.clone();
        let editor_for_action = editor_buffer.clone();
        let title_label_for_action = title_label.clone();
        let show_save_changes_for_action = Arc::clone(show_save_changes_dialog);
        let show_save_for_action = Arc::clone(show_save_dialog);
        let path_clone = path.clone();
        app_action.connect_activate(move |_, _| {
            let file_ops = file_ops_for_action.clone();
            let win = window_for_action.clone();
            let editor = editor_for_action.clone();
            let title_label_async = title_label_for_action.clone();
            let show_save_changes_dialog = Arc::clone(&show_save_changes_for_action);
            let show_save_dialog = Arc::clone(&show_save_for_action);
            let path_to_open = path_clone.clone();
            glib::MainContext::default().spawn_local(async move {
                let gtk_window: &gtk4::Window = win.upcast_ref();
                let text_buffer: &gtk4::TextBuffer = editor.upcast_ref();
                #[allow(clippy::await_holding_refcell_ref)]
                let result = file_ops
                    .borrow()
                    .open_file_by_path_async(
                        &path_to_open,
                        gtk_window,
                        text_buffer,
                        |w, doc_name, action| {
                            (show_save_changes_dialog)(w, doc_name, action)
                        },
                        |w, title, suggested| (show_save_dialog)(w, title, suggested),
                    )
                    .await;

                match result {
                    Ok(_) => {
                        let title = file_ops.borrow().get_document_title();
                        title_label_async.set_text(&title);
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to open recent file: {} -> {}",
                            path_to_open.display(),
                            e
                        );
                    }
                }
            });
        });
        app.add_action(&app_action);
    }

    // Update clear_recent action enabled state
    if let Some(clear_action) = app.lookup_action("clear_recent") {
        if let Some(simple_action) = clear_action.downcast_ref::<gio::SimpleAction>() {
            simple_action.set_enabled(!list.is_empty());
        }
    }
}

/// Setup dynamic recent-file actions and menu updates.
#[allow(clippy::too_many_arguments)]
pub fn setup_recent_actions(
    app: &gtk4::Application,
    file_operations: Rc<RefCell<FileOperations>>,
    recent_menu: &gio::Menu,
    window: &gtk4::ApplicationWindow,
    editor_buffer: &sourceview5::Buffer,
    title_label: &gtk4::Label,
    show_save_changes_dialog: SaveChangesDialogCallback,
    show_save_dialog: SaveDialogCallback,
) {
    // Create a simple action 'recent' so we can enable/disable the top-level Recent menu entry
    let recent_action = gio::SimpleAction::new("recent", None);
    app.add_action(&recent_action);

    // Initialize menu and actions using the helper function
    update_recent_file_actions(
        app,
        &file_operations,
        recent_menu,
        window,
        editor_buffer,
        title_label,
        &show_save_changes_dialog,
        &show_save_dialog,
        &recent_action,
    );

    // Register callback so that when recent files change we update menu and action sensitivity
    let app_owned = app.clone();
    let window_owned = window.clone();
    let editor_buffer_owned = editor_buffer.clone();
    let title_label_owned = title_label.clone();
    let recent_menu_owned = recent_menu.clone();
    let recent_action_owned = recent_action.clone();
    let file_ops_owned = file_operations.clone();
    let show_save_changes_owned = Arc::clone(&show_save_changes_dialog);
    let show_save_owned = Arc::clone(&show_save_dialog);

    file_operations
        .borrow()
        .register_recent_changed_callback(move || {
            update_recent_file_actions(
                &app_owned,
                &file_ops_owned,
                &recent_menu_owned,
                &window_owned,
                &editor_buffer_owned,
                &title_label_owned,
                &show_save_changes_owned,
                &show_save_owned,
                &recent_action_owned,
            );
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::buffer::{DocumentBuffer, RecentFiles};
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;
    
    #[test]
    fn smoke_test_recent_files_cascade_prevention() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let settings_path = temp_dir.path().join("test_settings.ron");
        
        // Create test objects
        let buffer = Rc::new(RefCell::new(DocumentBuffer::new_untitled()));
        let recent_files = Rc::new(RefCell::new(RecentFiles::new(&settings_path)));
        let file_ops = FileOperations::new(buffer, recent_files);
        
        // Track callback invocations
        let callback_count = Arc::new(Mutex::new(0));
        let callback_count_clone = Arc::clone(&callback_count);
        
        // Register a callback that counts invocations
        file_ops.register_recent_changed_callback(move || {
            let mut count = callback_count_clone.lock().unwrap();
            *count += 1;
        });
        
        // Test initial state
        let (callbacks, updating, cached) = file_ops.get_callback_stats();
        assert_eq!(callbacks, 1, "Should have 1 registered callback");
        assert!(!updating, "Should not be updating initially");
        assert_eq!(cached, 0, "Should have no cached files initially");
        
        // Test adding a file - should trigger callback
        let test_file1 = temp_dir.path().join("test1.md");
        std::fs::write(&test_file1, "# Test 1").expect("Failed to write test file");
        file_ops.add_recent_file(&test_file1);
        
        assert_eq!(*callback_count.lock().unwrap(), 1, "First file should trigger callback");
        
        // Test adding the same file again - should be deduplicated (file already at top)
        file_ops.add_recent_file(&test_file1);
        assert_eq!(*callback_count.lock().unwrap(), 1, "Same file should be deduplicated");
        
        // Test adding a different file - should trigger callback
        let test_file2 = temp_dir.path().join("test2.md");
        std::fs::write(&test_file2, "# Test 2").expect("Failed to write test file");
        file_ops.add_recent_file(&test_file2);
        
        assert_eq!(*callback_count.lock().unwrap(), 2, "Different file should trigger callback");
        
        // Test adding the same file again - should be deduplicated
        file_ops.add_recent_file(&test_file2);
        assert_eq!(*callback_count.lock().unwrap(), 2, "Same file at top should be deduplicated");
        
        // Test adding the first file again - should trigger callback (moves to top)
        file_ops.add_recent_file(&test_file1);
        assert_eq!(*callback_count.lock().unwrap(), 3, "Moving file to top should trigger callback");
        
        // Test reset functionality
        file_ops.reset_cascade_prevention();
        let (_, updating_after_reset, cached_after_reset) = file_ops.get_callback_stats();
        assert!(!updating_after_reset, "Should not be updating after reset");
        assert_eq!(cached_after_reset, 0, "Should have no cached files after reset");
    }
    
    #[test]
    fn smoke_test_recursive_callback_prevention() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let settings_path = temp_dir.path().join("test_settings.ron");
        
        let buffer = Rc::new(RefCell::new(DocumentBuffer::new_untitled()));
        let recent_files = Rc::new(RefCell::new(RecentFiles::new(&settings_path)));
        let file_ops = Rc::new(FileOperations::new(buffer, recent_files));
        
        let callback_count = Arc::new(Mutex::new(0));
        let callback_count_clone = Arc::clone(&callback_count);
        let file_ops_clone = file_ops.clone();
        
        // Register a callback that tries to add another file (potential infinite loop)
        file_ops.register_recent_changed_callback(move || {
            let mut count = callback_count_clone.lock().unwrap();
            *count += 1;
            
            // Try to add another file from within the callback
            if *count == 1 {
                // This should be prevented by the updating_recent_files flag
                let recursive_file = format!("recursive_{}.md", *count);
                file_ops_clone.add_recent_file(std::path::Path::new(&recursive_file));
            }
        });
        
        // Add a file - this should trigger the callback, but prevent recursion
        let test_file = temp_dir.path().join("trigger.md");
        std::fs::write(&test_file, "# Trigger").expect("Failed to write test file");
        file_ops.add_recent_file(&test_file);
        
        // The callback should only be called once due to recursion prevention
        assert_eq!(*callback_count.lock().unwrap(), 1, "Callback should only be called once due to recursion prevention");
    }
}