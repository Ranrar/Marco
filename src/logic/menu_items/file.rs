use gtk4::{gio, glib, prelude::*};
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use std::path::Path;
use anyhow::{Result};
use crate::logic::{DocumentBuffer, RecentFiles};
use log::trace;

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
        }
    }

    /// Register a callback to be invoked whenever the recent files list changes
    pub fn register_recent_changed_callback<F: Fn() + 'static>(&self, cb: F) {
        self.recent_changed_callbacks.borrow_mut().push(Box::new(cb));
    }

    fn invoke_recent_changed_callbacks(&self) {
        for cb in self.recent_changed_callbacks.borrow().iter() {
            cb();
        }
    }

    /// Add file to the recent list and notify callbacks
    fn add_recent_file<P: AsRef<Path>>(&self, path: P) {
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
        F: for<'b> Fn(&'b gtk4::Window, &'b str, &'b str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SaveChangesResult>> + 'b>>,
        G: for<'b> Fn(&'b gtk4::Window, &'b str, Option<&'b str>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<std::path::PathBuf>>> + 'b>>,
    {
        let path = path.as_ref();

        // Check for unsaved changes and prompt user
        if self.buffer.borrow().has_unsaved_changes() {
            let document_title = self.get_document_title();
            match show_save_changes_dialog(parent_window.upcast_ref(), &document_title, "opening a file").await? {
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
                        
                        let file_path = show_save_dialog(parent_window.upcast_ref(), "Save Markdown File", Some(suggested_name)).await?;
                        if let Some(save_path) = file_path {
                            let content = self.get_editor_content(editor_buffer);
                            self.buffer.borrow_mut().save_as_content(&save_path, &content)?;
                            self.buffer.borrow_mut().set_baseline(&content);
                            self.add_recent_file(&save_path);
                        } else {
                            // User cancelled Save As dialog, cancel the entire open operation
                            return Err(anyhow::anyhow!("Save As cancelled, open operation aborted"));
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
    /// If the document is untitled, this will show a Save As dialog.
    /// 
    /// # Arguments
    /// * `parent_window` - Parent window for dialogs
    /// * `editor_buffer` - GTK TextBuffer to get content from
    /// 
    /// # Returns
    /// * `Ok(())` - File saved successfully
    /// * `Err(anyhow::Error)` - Operation failed or was cancelled
    pub fn save_document<W: IsA<gtk4::Window>>(
        &self,
        parent_window: &W,
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
            drop(buffer); // Release borrow
            self.save_as_document(parent_window, editor_buffer)
        }
    }

    /// Saves the document with a new name (Save As)
    /// 
    /// # Arguments
    /// * `parent_window` - Parent window for dialogs
    /// * `editor_buffer` - GTK TextBuffer to get content from
    /// 
    /// # Returns
    /// * `Ok(())` - File saved successfully
    /// * `Err(anyhow::Error)` - Operation failed or was cancelled
    pub fn save_as_document<W: IsA<gtk4::Window>>(
        &self,
        parent_window: &W,
        editor_buffer: &gtk4::TextBuffer,
    ) -> Result<()> {
        let file_path = self.show_save_dialog(parent_window)?;
        
        // Check if file exists and confirm overwrite
        if DocumentBuffer::file_exists(&file_path) {
            if !self.confirm_overwrite(parent_window, &file_path)? {
                return Err(anyhow::anyhow!("Save cancelled: file already exists"));
            }
        }

    let content = self.get_editor_content(editor_buffer);
        self.buffer.borrow_mut().save_as_content(&file_path, &content)?;
    // After Save As, baseline has been updated inside save_as_content but also set here for safety
    self.buffer.borrow_mut().set_baseline(&content);
        
        // Add to recent files
    self.add_recent_file(&file_path);
        
    trace!("audit: saved document as: {}", file_path.display());
    eprintln!("[FileOps] Saved document as: {}", file_path.display());
        Ok(())
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
        self.buffer.borrow_mut().update_modified_from_content(current_content);
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
            F: for<'b> Fn(&'b gtk4::Window, &'b str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<std::path::PathBuf>>> + 'b>>,
            G: for<'b> Fn(&'b gtk4::Window, &'b str, &'b str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SaveChangesResult>> + 'b>>,
            H: for<'b> Fn(&'b gtk4::Window, &'b str, Option<&'b str>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<std::path::PathBuf>>> + 'b>>,
        {
            // Check for unsaved changes and prompt user
            if self.buffer.borrow().has_unsaved_changes() {
                let document_title = self.get_document_title();
                match show_save_changes_dialog(parent_window, &document_title, "opening a file").await? {
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
                            
                            let file_path = show_save_dialog(parent_window, "Save Markdown File", Some(suggested_name)).await?;
                            if let Some(path) = file_path {
                                let content = self.get_editor_content(editor_buffer);
                                self.buffer.borrow_mut().save_as_content(&path, &content)?;
                                self.buffer.borrow_mut().set_baseline(&content);
                                self.add_recent_file(&path);
                            } else {
                                // User cancelled Save As dialog, cancel the entire open operation
                                return Err(anyhow::anyhow!("Save As cancelled, open operation aborted"));
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
            F: for<'b> Fn(&'b gtk4::Window, &'b str, &'b str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SaveChangesResult>> + 'b>>,
            G: for<'b> Fn(&'b gtk4::Window, &'b str, Option<&'b str>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<std::path::PathBuf>>> + 'b>>,
        {
            // Check for unsaved changes and prompt user
            if self.buffer.borrow().has_unsaved_changes() {
                let document_title = self.get_document_title();
                match show_save_changes_dialog(parent_window, &document_title, "starting a new document").await? {
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
                            
                            let file_path = show_save_dialog(parent_window, "Save Markdown File", Some(suggested_name)).await?;
                            if let Some(path) = file_path {
                                let content = self.get_editor_content(editor_buffer);
                                self.buffer.borrow_mut().save_as_content(&path, &content)?;
                                self.buffer.borrow_mut().set_baseline(&content);
                                self.add_recent_file(&path);
                            } else {
                                // User cancelled Save As dialog, cancel the new document operation
                                return Err(anyhow::anyhow!("Save As cancelled, new document operation aborted"));
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
            F: for<'b> Fn(&'b gtk4::Window, &'b str, Option<&'b str>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<std::path::PathBuf>>> + 'b>>,
        {
            let suggested_name = self.get_document_title();
            let file_path = show_save_dialog(parent_window, "Save Markdown File", Some(&suggested_name)).await?;
            if let Some(path) = file_path {
                let start_iter = editor_buffer.start_iter();
                let end_iter = editor_buffer.end_iter();
                let content = editor_buffer.text(&start_iter, &end_iter, false).to_string();
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
            F: for<'b> Fn(&'b gtk4::Window, &'b str, &'b str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SaveChangesResult>> + 'b>>,
            G: for<'b> Fn(&'b gtk4::Window, &'b str, Option<&'b str>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<std::path::PathBuf>>> + 'b>>,
        {
            let is_modified = self.buffer.borrow().has_unsaved_changes();
            let document_title = self.get_document_title();
            if is_modified {
                match show_save_changes_dialog(parent_window, &document_title, "quitting").await? {
                    SaveChangesResult::Save => {
                        let has_file_path = self.buffer.borrow().get_file_path().is_some();
                        if !has_file_path {
                            let suggested_name = self.get_document_title();
                            let file_path = show_save_dialog(parent_window, "Save Markdown File", Some(&suggested_name)).await?;
                            if let Some(path) = file_path {
                                let start_iter = editor_buffer.start_iter();
                                let end_iter = editor_buffer.end_iter();
                                let content = editor_buffer.text(&start_iter, &end_iter, false).to_string();
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
        let new_buffer = DocumentBuffer::new_from_file(path)?;
        let content = new_buffer.read_content()?;
        
        // Update editor
        editor_buffer.set_text(&content);
        
        // Update our buffer
    *self.buffer.borrow_mut() = new_buffer;
    // Set baseline to the loaded content so modifications are tracked correctly
    self.buffer.borrow_mut().set_baseline(&content);
        
        // Add to recent files
    self.add_recent_file(path);
        
        Ok(())
    }

    /// Gets the current content from the editor buffer
    fn get_editor_content(&self, editor_buffer: &gtk4::TextBuffer) -> String {
        let start_iter = editor_buffer.start_iter();
        let end_iter = editor_buffer.end_iter();
        editor_buffer.text(&start_iter, &end_iter, false).to_string()
    }



    /// Shows a save file dialog  
    fn show_save_dialog<W: IsA<gtk4::Window>>(&self, _parent_window: &W) -> Result<std::path::PathBuf> {
        // For initial implementation, use a hardcoded save location
        // TODO: Implement actual file dialog integration
        Ok(std::path::PathBuf::from("saved_document.md"))
    }

    /// Confirms file overwrite
    fn confirm_overwrite<W: IsA<gtk4::Window>>(
        &self,
        _parent_window: &W,
        path: &Path,
    ) -> Result<bool> {
        // For initial implementation, always confirm
        // TODO: Implement actual overwrite confirmation dialog
        eprintln!("[FileOps] Would confirm overwrite of {}, auto-confirming for now", path.display());
        Ok(true)
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
            if i >= 5 { break; } // Limit to 5 recent files

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
) {
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
                    let content = editor_buffer.text(&start_iter, &end_iter, false).to_string();
                    file_ops.mark_document_modified_from_content(&content);
                    // Update visible title label
                    let title = file_ops.get_document_title();
                    title_label.set_text(&title);
                    trace!("audit: editor buffer changed (user edit detected)");
                    if std::env::var("MARCO_DEBUG_POINTERS").is_ok() {
                        eprintln!("[file_ops] title_label ptr={:p} set_text='{}'", title_label.as_ptr(), title);
                    }
                }
            }
        }
    });
}

/// Register remaining file actions that require async dialogs.
///
/// The callbacks are boxed functions that return boxed futures. Callers
/// (main.rs) can pass UI dialog functions (Box::pin(...)) to integrate GTK dialogs.
pub fn register_file_actions_async(
    app: gtk4::Application,
    file_operations: Rc<RefCell<FileOperations>>,
    window: &gtk4::ApplicationWindow,
    editor_buffer: &sourceview5::Buffer,
    title_label: &gtk4::Label,
    show_open_dialog: Arc<dyn for<'b> Fn(&'b gtk4::Window, &'b str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<std::path::PathBuf>>> + 'b>> + Send + Sync + 'static>,
    show_save_changes_dialog: Arc<dyn for<'b> Fn(&'b gtk4::Window, &'b str, &'b str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SaveChangesResult>> + 'b>> + Send + Sync + 'static>,
    show_save_dialog: Arc<dyn for<'b> Fn(&'b gtk4::Window, &'b str, Option<&'b str>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<std::path::PathBuf>>> + 'b>> + Send + Sync + 'static>,
) {
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
                let file_ops_ref = file_ops.borrow();
                let gtk_window: &gtk4::Window = window.upcast_ref();
                let text_buffer: &gtk4::TextBuffer = editor_buffer.upcast_ref();
                let _ = file_ops_ref.open_file_async(
                    gtk_window,
                    text_buffer,
                    |w, title| (show_open_dialog)(w, title),
                    |w, doc_name, action| (show_save_changes_dialog)(w, doc_name, action),
                    |w, title, suggested| (show_save_dialog)(w, title, suggested),
                ).await;
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
        let show_save_changes_dialog = Arc::clone(&show_save_changes_dialog);
        let show_save_dialog = Arc::clone(&show_save_dialog);
        move |_, _| {
            let file_ops = file_ops.clone();
            let window = window.clone();
            let editor_buffer = editor_buffer.clone();
            let title_label = title_label.clone();
            let show_save_changes_dialog = Arc::clone(&show_save_changes_dialog);
            let show_save_dialog = Arc::clone(&show_save_dialog);
            glib::MainContext::default().spawn_local(async move {
                let file_ops_ref = file_ops.borrow();
                let gtk_window: &gtk4::Window = window.upcast_ref();
                let text_buffer: &gtk4::TextBuffer = editor_buffer.upcast_ref();
                let _ = file_ops_ref.new_document_async(
                    gtk_window,
                    text_buffer,
                    |w, doc_name, action| (show_save_changes_dialog)(w, doc_name, action),
                    |w, title, suggested| (show_save_dialog)(w, title, suggested),
                ).await;
                // Update title label after new document is created
                let title = file_ops.borrow().get_document_title();
                title_label.set_text(&title);
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
                let file_ops_ref = file_ops.borrow();
                let gtk_window: &gtk4::Window = window.upcast_ref();
                let text_buffer: &gtk4::TextBuffer = editor_buffer.upcast_ref();
                let _ = file_ops_ref.save_as_async(gtk_window, text_buffer, |w, title, suggested| (show_save_dialog)(w, title, suggested)).await;
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
                let file_ops_ref = file_ops.borrow();
                let gtk_window: &gtk4::Window = window.upcast_ref();
                let text_buffer: &gtk4::TextBuffer = editor_buffer.upcast_ref();
                let _ = file_ops_ref.quit_async(
                    gtk_window,
                    text_buffer,
                    &app,
                    |w, title, action| (show_save_changes_dialog)(w, title, action),
                    |w, title, suggested| (show_save_dialog)(w, title, suggested),
                ).await;
            });
        }
    });

    // Add actions to application
    app.add_action(&new_action);
    app.add_action(&open_action);
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

    // Dynamic recent-file registration is provided by `setup_recent_actions`
}

/// Setup dynamic recent-file actions and menu updates.
pub fn setup_recent_actions(
    app: &gtk4::Application,
    file_operations: Rc<RefCell<FileOperations>>,
    recent_menu: &gio::Menu,
    window: &gtk4::ApplicationWindow,
    editor_buffer: &sourceview5::Buffer,
    title_label: &gtk4::Label,
    show_save_changes_dialog: Arc<dyn for<'b> Fn(&'b gtk4::Window, &'b str, &'b str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SaveChangesResult>> + 'b>> + Send + Sync + 'static>,
    show_save_dialog: Arc<dyn for<'b> Fn(&'b gtk4::Window, &'b str, Option<&'b str>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<std::path::PathBuf>>> + 'b>> + Send + Sync + 'static>,
) {
    // Initialize menu using provided recent_menu from UI
    crate::logic::menu_items::file::update_recent_files_menu(recent_menu, &file_operations.borrow().get_recent_files());

    // Create a simple action 'recent' so we can enable/disable the top-level Recent menu entry
    let recent_action = gio::SimpleAction::new("recent", None);
    recent_action.set_enabled(!file_operations.borrow().get_recent_files().is_empty());
    app.add_action(&recent_action);

    // Clear recent action is managed here via registering the callback below

    // Register callback so that when recent files change we update menu and action sensitivity
    // Clone everything we'll need inside the closure so they have independent ownership
    let app_owned = app.clone();
    let window_owned = window.clone();
    let editor_buffer_owned = editor_buffer.clone();
    let title_label_owned = title_label.clone();
    let recent_menu_owned = recent_menu.clone();
    let recent_action_owned = recent_action.clone();
    // Clone an Rc to the FileOperations for closure capture
    let file_ops_owned = file_operations.clone();
    // Register callback using a pre-cloned handle so the closure only captures owned values
    file_operations.borrow().register_recent_changed_callback(move || {
        let list = file_ops_owned.borrow().get_recent_files();
        crate::logic::menu_items::file::update_recent_files_menu(&recent_menu_owned, &list);
        recent_action_owned.set_enabled(!list.is_empty());

        // Remove old actions
        for i in 0..5 {
            let name = format!("open_recent_{}", i);
            if app_owned.lookup_action(&name).is_some() {
                app_owned.remove_action(&name);
            }
        }

        // Register new actions
        for (i, path) in list.iter().enumerate() {
            if i >= 5 { break; }
            let action_name = format!("open_recent_{}", i);
            let app_action = gio::SimpleAction::new(&action_name, None);
            app_action.set_enabled(true);
            let file_ops_for_action = file_ops_owned.clone();
            let window_for_action = window_owned.clone();
            let editor_for_action = editor_buffer_owned.clone();
            let title_label_for_action = title_label_owned.clone();
            let show_save_changes_for_action = Arc::clone(&show_save_changes_dialog);
            let show_save_for_action = Arc::clone(&show_save_dialog);
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
                    let result = file_ops.borrow().open_file_by_path_async(
                        &path_to_open,
                        gtk_window,
                        text_buffer,
                        |w, doc_name, action| (show_save_changes_dialog)(w, doc_name, action),
                        |w, title, suggested| (show_save_dialog)(w, title, suggested),
                    ).await;
                    
                    match result {
                        Ok(_) => {
                            let title = file_ops.borrow().get_document_title();
                            title_label_async.set_text(&title);
                        }
                        Err(e) => {
                            eprintln!("Failed to open recent file: {} -> {}", path_to_open.display(), e);
                        }
                    }
                });
            });
            app_owned.add_action(&app_action);
        }

        // Update clear_recent action enabled state
        if let Some(clear_action) = app_owned.lookup_action("clear_recent") {
            if let Some(simple_action) = clear_action.downcast_ref::<gio::SimpleAction>() {
                simple_action.set_enabled(!list.is_empty());
            }
        }
    });
}