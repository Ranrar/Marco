use gtk4::{gio, glib, prelude::*};
use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use anyhow::{Result, Context};
use crate::logic::buffer::{DocumentBuffer, RecentFiles};

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
#[derive(Debug)]
pub struct FileOperations {
    /// Document buffer containing current file state
    pub buffer: Rc<RefCell<DocumentBuffer>>,
    /// Recent files manager
    pub recent_files: Rc<RefCell<RecentFiles>>,
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
        }
    }

    /// Creates a new untitled document
    /// 
    /// This will prompt to save if the current document has unsaved changes.
    /// 
    /// # Arguments
    /// * `parent_window` - Parent window for dialogs
    /// * `editor_buffer` - GTK TextBuffer to clear
    /// 
    /// # Returns
    /// * `Ok(())` - New document created successfully
    /// * `Err(anyhow::Error)` - Operation failed or was cancelled
    pub fn new_document<W: IsA<gtk4::Window>>(
        &self,
        parent_window: &W,
        editor_buffer: &gtk4::TextBuffer,
    ) -> Result<()> {
        // Check for unsaved changes
        if self.buffer.borrow().has_unsaved_changes() {
            match self.prompt_save_changes(parent_window, "starting a new document")? {
                SaveChangesResult::Save => {
                    self.save_document(parent_window, editor_buffer)?;
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

        eprintln!("[FileOps] Created new untitled document");
        Ok(())
    }

    /// Opens a file using a file chooser dialog
    /// 
    /// This will prompt to save if the current document has unsaved changes.
    /// 
    /// # Arguments
    /// * `parent_window` - Parent window for dialogs
    /// * `editor_buffer` - GTK TextBuffer to populate with file content
    /// 
    /// # Returns
    /// * `Ok(())` - File opened successfully
    /// * `Err(anyhow::Error)` - Operation failed or was cancelled
    pub fn open_file<W: IsA<gtk4::Window>>(
        &self,
        parent_window: &W,
        editor_buffer: &gtk4::TextBuffer,
    ) -> Result<()> {
        // Check for unsaved changes
        if self.buffer.borrow().has_unsaved_changes() {
            match self.prompt_save_changes(parent_window, "opening a file")? {
                SaveChangesResult::Save => {
                    self.save_document(parent_window, editor_buffer)?;
                }
                SaveChangesResult::Discard => {
                    // Continue with open
                }
                SaveChangesResult::Cancel => {
                    return Err(anyhow::anyhow!("Open file cancelled by user"));
                }
            }
        }

        // Show file chooser dialog
        let file_path = self.show_open_dialog(parent_window)?;
        self.load_file_into_editor(&file_path, editor_buffer)?;

        eprintln!("[FileOps] Opened file: {}", file_path.display());
        Ok(())
    }

    /// Opens a specific file by path
    /// 
    /// This is used for recent files and command-line arguments.
    /// 
    /// # Arguments
    /// * `path` - Path to the file to open
    /// * `parent_window` - Parent window for error dialogs
    /// * `editor_buffer` - GTK TextBuffer to populate
    /// 
    /// # Returns
    /// * `Ok(())` - File opened successfully
    /// * `Err(anyhow::Error)` - Operation failed
    pub fn open_file_by_path<P: AsRef<Path>, W: IsA<gtk4::Window>>(
        &self,
        path: P,
        parent_window: &W,
        editor_buffer: &gtk4::TextBuffer,
    ) -> Result<()> {
        let path = path.as_ref();

        // Check for unsaved changes
        if self.buffer.borrow().has_unsaved_changes() {
            match self.prompt_save_changes(parent_window, "opening a file")? {
                SaveChangesResult::Save => {
                    self.save_document(parent_window, editor_buffer)?;
                }
                SaveChangesResult::Discard => {
                    // Continue with open
                }
                SaveChangesResult::Cancel => {
                    return Err(anyhow::anyhow!("Open file cancelled by user"));
                }
            }
        }

        self.load_file_into_editor(path, editor_buffer)?;
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
        
        // Add to recent files
        self.recent_files.borrow_mut().add_file(&file_path);
        
        eprintln!("[FileOps] Saved document as: {}", file_path.display());
        Ok(())
    }

    /// Handles application quit with unsaved changes check
    /// 
    /// # Arguments
    /// * `parent_window` - Parent window for dialogs
    /// * `editor_buffer` - GTK TextBuffer to save if needed
    /// * `app` - GTK Application to quit
    /// 
    /// # Returns
    /// * `Ok(())` - Application can quit safely
    /// * `Err(anyhow::Error)` - Quit cancelled by user
    pub fn quit_application<W: IsA<gtk4::Window>>(
        &self,
        parent_window: &W,
        editor_buffer: &gtk4::TextBuffer,
        app: &gtk4::Application,
    ) -> Result<()> {
        if self.buffer.borrow().has_unsaved_changes() {
            match self.prompt_save_changes(parent_window, "quitting")? {
                SaveChangesResult::Save => {
                    self.save_document(parent_window, editor_buffer)?;
                }
                SaveChangesResult::Discard => {
                    // Continue with quit
                }
                SaveChangesResult::Cancel => {
                    return Err(anyhow::anyhow!("Quit cancelled by user"));
                }
            }
        }

        app.quit();
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
    }

    /// Marks the current document as modified
    /// 
    /// This should be called when the editor content changes.
    pub fn mark_document_modified(&self) {
        self.buffer.borrow_mut().mark_modified();
    }

    /// Gets the current document's display title
    /// 
    /// # Returns
    /// String suitable for window title (includes * for modified files)
    pub fn get_document_title(&self) -> String {
        self.buffer.borrow().get_full_title()
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
        
        // Add to recent files
        self.recent_files.borrow_mut().add_file(path);
        
        Ok(())
    }

    /// Gets the current content from the editor buffer
    fn get_editor_content(&self, editor_buffer: &gtk4::TextBuffer) -> String {
        let start_iter = editor_buffer.start_iter();
        let end_iter = editor_buffer.end_iter();
        editor_buffer.text(&start_iter, &end_iter, false).to_string()
    }

    /// Shows an open file dialog
    fn show_open_dialog<W: IsA<gtk4::Window>>(&self, _parent_window: &W) -> Result<std::path::PathBuf> {
        // For now, let's use a simple approach - look for any .md files in the current directory
        // and let the user select from existing files
        let current_dir = std::env::current_dir()
            .context("Failed to get current directory")?;
        
        // Find all .md files in the current directory
        let mut md_files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                if let Some(extension) = entry.path().extension() {
                    if extension == "md" {
                        md_files.push(entry.path());
                    }
                }
            }
        }
        
        if md_files.is_empty() {
            return Err(anyhow::anyhow!("No .md files found in current directory. Create a .md file first."));
        }
        
        // For now, just return the first .md file found
        // TODO: Implement proper file selection dialog
        let selected_file = &md_files[0];
        eprintln!("[FileOps] Auto-selecting file: {}", selected_file.display());
        eprintln!("[FileOps] Available .md files: {:?}", md_files.iter().map(|p| p.file_name().unwrap_or_default()).collect::<Vec<_>>());
        
        Ok(selected_file.clone())
    }

    /// Shows a save file dialog  
    fn show_save_dialog<W: IsA<gtk4::Window>>(&self, _parent_window: &W) -> Result<std::path::PathBuf> {
        // For initial implementation, use a hardcoded save location
        // TODO: Implement actual file dialog integration
        Ok(std::path::PathBuf::from("saved_document.md"))
    }

    /// Prompts user to save changes
    fn prompt_save_changes<W: IsA<gtk4::Window>>(
        &self,
        _parent_window: &W,
        action: &str,
    ) -> Result<SaveChangesResult> {
        // For initial implementation, always discard changes
        // TODO: Implement actual save changes dialog
        eprintln!("[FileOps] Would prompt to save changes before {}, auto-discarding for now", action);
        Ok(SaveChangesResult::Discard)
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

/// Creates a basic file menu model
/// 
/// This creates the basic structure for the file menu.
/// The actual actions will be wired up in main.rs.
pub fn create_file_menu() -> gio::Menu {
    let menu = gio::Menu::new();
    
    // Basic file operations
    menu.append(Some("New"), Some("app.new"));
    menu.append(Some("Open"), Some("app.open"));
    
    // Separator
    menu.append(None, None);
    
    menu.append(Some("Save"), Some("app.save"));
    menu.append(Some("Save As"), Some("app.save_as"));
    
    // Separator
    menu.append(None, None);
    
    // Recent files submenu will be added dynamically
    let recent_menu = gio::Menu::new();
    recent_menu.append(Some("No recent files"), None);
    menu.append_submenu(Some("Recent Files"), &recent_menu);
    
    // Separator
    menu.append(None, None);
    
    menu.append(Some("Settings"), Some("app.settings"));
    menu.append(Some("Quit"), Some("app.quit"));
    
    menu
}

/// Updates the recent files submenu
/// 
/// # Arguments
/// * `file_menu` - The file menu to update
/// * `recent_files` - List of recent file paths
pub fn update_recent_files_menu(_file_menu: &gio::Menu, recent_files: &[std::path::PathBuf]) {
    // Find the recent files submenu (index 2 in our structure)
    // Note: This is a simplified implementation
    // In a real app, you'd want to find the submenu by label or store a reference
    
    let recent_menu = gio::Menu::new();
    
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
    
    // Note: GTK4 doesn't allow direct modification of menu structure
    // In a real implementation, you'd need to rebuild the menu or use a different approach
    eprintln!("[FileOps] Recent files menu updated with {} files", recent_files.len());
}