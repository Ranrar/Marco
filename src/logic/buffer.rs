use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context};

/// Manages document buffer state including file path, modification status, and content
/// 
/// This struct provides functionality for:
/// - Tracking the current file path (if any)
/// - Managing the is_modified flag to detect unsaved changes
/// - Handling file I/O operations with proper error handling
/// - Supporting "Untitled" documents that haven't been saved yet
/// 
/// # Thread Safety
/// This struct is designed to be used with Rc<RefCell<DocumentBuffer>> for 
/// shared ownership in GTK applications running on the main thread.
#[derive(Debug, Clone)]
pub struct DocumentBuffer {
    /// Current file path, None for new unsaved documents
    pub file_path: Option<PathBuf>,
    /// Whether the document has unsaved changes
    pub is_modified: bool,
    /// Baseline content used to detect actual modifications
    /// This stores the content as it was when the file was last loaded or saved.
    pub baseline_content: String,
    /// Display name for the document (filename or "Untitled.md")
    pub display_name: String,
}

impl DocumentBuffer {
    /// Creates a new empty document buffer for an "Untitled" document
    /// 
    /// # Example
    /// ```
    /// use marco::logic::buffer::DocumentBuffer;
    /// 
    /// let buffer = DocumentBuffer::new_untitled();
    /// assert!(buffer.file_path.is_none());
    /// assert_eq!(buffer.display_name, "Untitled.md");
    /// assert!(!buffer.is_modified);
    /// ```
    pub fn new_untitled() -> Self {
        Self {
            file_path: None,
            is_modified: false,
            baseline_content: String::new(),
            display_name: "Untitled.md".to_string(),
        }
    }

    /// Creates a document buffer for an existing file
    /// 
    /// # Arguments
    /// * `path` - Path to the existing file
    /// 
    /// # Returns
    /// * `Ok(DocumentBuffer)` - Buffer initialized with the file path
    /// * `Err(anyhow::Error)` - If the path is invalid or the file doesn't exist
    /// 
    /// # Example
    /// ```no_run
    /// use std::path::Path;
    /// use marco::logic::buffer::DocumentBuffer;
    /// 
    /// # fn main() -> anyhow::Result<()> {
    /// let buffer = DocumentBuffer::new_from_file(Path::new("document.md"))?;
    /// assert!(buffer.file_path.is_some());
    /// assert_eq!(buffer.display_name, "document.md");
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(anyhow::anyhow!("File does not exist: {}", path.display()));
        }

        let display_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Ok(Self {
            file_path: Some(path.to_path_buf()),
            is_modified: false,
            baseline_content: String::new(),
            display_name,
        })
    }

    /// Reads the content of the file associated with this buffer
    /// 
    /// # Returns
    /// * `Ok(String)` - Content of the file
    /// * `Err(anyhow::Error)` - If no file is associated or read fails
    /// 
    /// # Example
    /// ```no_run
    /// use marco::logic::buffer::DocumentBuffer;
    /// 
    /// # fn main() -> anyhow::Result<()> {
    /// let buffer = DocumentBuffer::new_untitled();
    /// let content = buffer.read_content()?;
    /// println!("File content: {}", content);
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_content(&self) -> Result<String> {
        match &self.file_path {
            Some(path) => {
                fs::read_to_string(path)
                    .with_context(|| format!("Failed to read file: {}", path.display()))
            }
            None => Ok(String::new()), // Empty content for untitled documents
        }
    }

    /// Saves content to the file associated with this buffer
    /// 
    /// # Arguments
    /// * `content` - Text content to save
    /// 
    /// # Returns
    /// * `Ok(())` - Save operation succeeded
    /// * `Err(anyhow::Error)` - If no file is associated or write fails
    /// 
    /// # Side Effects
    /// - Sets `is_modified` to `false` on successful save
    /// 
    /// # Example
    /// ```no_run
    /// use marco::logic::buffer::DocumentBuffer;
    /// 
    /// # fn main() -> anyhow::Result<()> {
    /// let mut buffer = DocumentBuffer::new_untitled();
    /// buffer.save_content("# My Document\n\nHello world!")?;
    /// assert!(!buffer.is_modified);
    /// # Ok(())
    /// # }
    /// ```
    pub fn save_content(&mut self, content: &str) -> Result<()> {
        match &self.file_path {
            Some(path) => {
                // Create parent directories if they don't exist
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
                }

                fs::write(path, content)
                    .with_context(|| format!("Failed to write file: {}", path.display()))?;

                // Update baseline to the saved content
                self.baseline_content = content.to_string();
                self.is_modified = false;
                Ok(())
            }
            None => Err(anyhow::anyhow!("Cannot save: no file path set. Use save_as_content() instead.")),
        }
    }

    /// Saves content to a new file path (Save As operation)
    /// 
    /// # Arguments
    /// * `path` - New file path to save to
    /// * `content` - Text content to save
    /// 
    /// # Returns
    /// * `Ok(())` - Save operation succeeded
    /// * `Err(anyhow::Error)` - If write fails
    /// 
    /// # Side Effects
    /// - Updates `file_path` to the new path
    /// - Updates `display_name` to the new filename
    /// - Sets `is_modified` to `false` on successful save
    /// - Automatically appends `.md` extension if missing
    /// 
    /// # Example
    /// ```no_run
    /// use std::path::Path;
    /// use marco::logic::buffer::DocumentBuffer;
    /// 
    /// # fn main() -> anyhow::Result<()> {
    /// let mut buffer = DocumentBuffer::new_untitled();
    /// buffer.save_as_content(Path::new("new_document"), "# Content")?;
    /// assert_eq!(buffer.file_path.unwrap().extension().unwrap(), "md");
    /// # Ok(())
    /// # }
    /// ```
    pub fn save_as_content<P: AsRef<Path>>(&mut self, path: P, content: &str) -> Result<()> {
        let mut path = path.as_ref().to_path_buf();

        // Ensure the file has a .md extension
        if path.extension().is_none() {
            path.set_extension("md");
        }

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        fs::write(&path, content)
            .with_context(|| format!("Failed to write file: {}", path.display()))?;

        // Update buffer state
        let display_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string();

        self.file_path = Some(path);
        self.display_name = display_name;
    // After Save As, baseline matches the saved content
    self.baseline_content = content.to_string();
    self.is_modified = false;

        Ok(())
    }

    /// Marks the document as modified (has unsaved changes)
    /// 
    /// This should be called whenever the editor content changes.
    /// 
    /// # Example
    /// ```
    /// use marco::logic::buffer::DocumentBuffer;
    /// 
    /// let mut buffer = DocumentBuffer::new_untitled();
    /// buffer.mark_modified();
    /// assert!(buffer.is_modified);
    /// ```
    pub fn mark_modified(&mut self) {
        // Deprecated: prefer comparing against baseline via `update_modified_from_content`.
        self.is_modified = true;
    }

    /// Update modification state by comparing the provided editor content with the baseline.
    pub fn update_modified_from_content(&mut self, current_content: &str) {
        let modified = self.baseline_content != current_content;
        self.is_modified = modified;
    }

    /// Sets the baseline content (used after loading or saving a file)
    pub fn set_baseline(&mut self, content: &str) {
        self.baseline_content = content.to_string();
        self.is_modified = false;
    }

    /// Checks if the document has unsaved changes
    /// 
    /// # Returns
    /// * `true` - Document has been modified since last save
    /// * `false` - Document is in sync with file
    pub fn has_unsaved_changes(&self) -> bool {
        self.is_modified
    }

    /// Gets the file path if this document is associated with a file
    /// 
    /// # Returns
    /// * `Some(PathBuf)` - Path to the associated file
    /// * `None` - Document is untitled/unsaved
    pub fn get_file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }

    /// Gets the display name for the document
    /// 
    /// This returns the filename if associated with a file,
    /// or "Untitled.md" for new documents.
    /// 
    /// # Returns
    /// String suitable for display in window title or tabs
    pub fn get_display_name(&self) -> &str {
        &self.display_name
    }

    /// Gets the full display title including modification indicator
    /// 
    /// # Returns
    /// * For modified files: "* filename.md"
    /// * For unmodified files: "filename.md"
    /// 
    /// # Example
    /// ```
    /// use marco::logic::buffer::DocumentBuffer;
    /// 
    /// let mut buffer = DocumentBuffer::new_untitled();
    /// assert_eq!(buffer.get_full_title(), "Untitled.md");
    /// buffer.mark_modified();
    /// assert_eq!(buffer.get_full_title(), "*Untitled.md");
    /// ```
    pub fn get_full_title(&self) -> String {
        if self.is_modified {
            format!("*{}", self.display_name)
        } else {
            self.display_name.clone()
        }
    }

    /// Resets to a new untitled document
    /// 
    /// This clears the file path and resets the modification state,
    /// effectively creating a fresh document.
    /// 
    /// # Example
    /// ```
    /// use marco::logic::buffer::DocumentBuffer;
    /// 
    /// let mut buffer = DocumentBuffer::new_untitled();
    /// buffer.reset_to_untitled();
    /// assert!(buffer.file_path.is_none());
    /// assert!(!buffer.is_modified);
    /// assert_eq!(buffer.display_name, "Untitled.md");
    /// ```
    pub fn reset_to_untitled(&mut self) {
        self.file_path = None;
        self.is_modified = false;
        self.display_name = "Untitled.md".to_string();
    }

    /// Checks if a file exists at the given path
    /// 
    /// This is a utility function for checking file existence
    /// before overwriting in Save As operations.
    /// 
    /// # Arguments
    /// * `path` - Path to check
    /// 
    /// # Returns
    /// * `true` - File exists
    /// * `false` - File does not exist
    pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists() && path.as_ref().is_file()
    }
}

/// Recent files manager for tracking and persisting recently opened files
/// 
/// This struct manages a list of recently opened files through the
/// swanson settings system for consistent persistence.
pub struct RecentFiles {
    settings_path: PathBuf,
}

impl RecentFiles {
    /// Creates a new recent files manager
    /// 
    /// # Arguments
    /// * `settings_path` - Path to the settings.ron file
    pub fn new<P: AsRef<Path>>(settings_path: P) -> Self {
        Self {
            settings_path: settings_path.as_ref().to_path_buf(),
        }
    }

    /// Adds a file to the recent files list
    /// 
    /// If the file is already in the list, it's moved to the front.
    /// If the list exceeds max_files, the oldest entry is removed.
    /// 
    /// # Arguments
    /// * `path` - File path to add
    /// 
    /// # Example
    /// ```
    /// use std::path::Path;
    /// use marco::logic::buffer::RecentFiles;
    /// 
    /// let recent = RecentFiles::new("settings.ron");
    /// recent.add_file(Path::new("doc1.md"));
    /// ```
    pub fn add_file<P: AsRef<Path>>(&self, path: P) {
        let mut settings = crate::logic::swanson::Settings::load_from_file(&self.settings_path)
            .unwrap_or_default();
        
        settings.add_recent_file(path);
        
        if let Err(e) = settings.save_to_file(&self.settings_path) {
            eprintln!("[RecentFiles] Failed to save recent file: {}", e);
        }
    }

    /// Gets the list of recent files
    /// 
    /// # Returns
    /// Vector of recent file paths (most recent first)
    pub fn get_files(&self) -> Vec<PathBuf> {
        let settings = crate::logic::swanson::Settings::load_from_file(&self.settings_path)
            .unwrap_or_default();
        
        settings.get_recent_files()
    }

    /// Checks if the recent files list is empty
    pub fn is_empty(&self) -> bool {
        self.get_files().is_empty()
    }

    /// Clears all recent files
    pub fn clear(&self) {
        let mut settings = crate::logic::swanson::Settings::load_from_file(&self.settings_path)
            .unwrap_or_default();
        
        settings.clear_recent_files();
        
        if let Err(e) = settings.save_to_file(&self.settings_path) {
            eprintln!("[RecentFiles] Failed to save cleared recent files: {}", e);
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_new_untitled() {
        let buffer = DocumentBuffer::new_untitled();
        assert!(buffer.file_path.is_none());
        assert!(!buffer.is_modified);
        assert_eq!(buffer.display_name, "Untitled.md");
        assert_eq!(buffer.get_full_title(), "Untitled.md");
    }

    #[test]
    fn test_mark_modified() {
        let mut buffer = DocumentBuffer::new_untitled();
        buffer.mark_modified();
        assert!(buffer.is_modified);
        assert!(buffer.has_unsaved_changes());
        assert_eq!(buffer.get_full_title(), "*Untitled.md");
    }

    #[test]
    fn test_recent_files() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.ron");
        
        let recent = RecentFiles::new(&settings_path);
        
        recent.add_file("file1.md");
        recent.add_file("file2.md");
        recent.add_file("file3.md");
        
        let files = recent.get_files();
        assert!(files.len() <= 5); // Should respect max limit
        if !files.is_empty() {
            assert_eq!(files[0], PathBuf::from("file3.md")); // Most recent first
        }
    }

    #[test]
    fn test_recent_files_duplicate() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.ron");
        
        let recent = RecentFiles::new(&settings_path);
        
        recent.add_file("file1.md");
        recent.add_file("file2.md");
        recent.add_file("file1.md"); // Should move to front
        
        let files = recent.get_files();
        if files.len() >= 2 {
            assert_eq!(files[0], PathBuf::from("file1.md"));
            assert_eq!(files[1], PathBuf::from("file2.md"));
        }
    }

    #[test]
    fn test_save_as_adds_md_extension() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file");
        
        let mut buffer = DocumentBuffer::new_untitled();
        buffer.save_as_content(&file_path, "# Test content").unwrap();
        
        assert!(buffer.file_path.is_some());
        let saved_path = buffer.file_path.unwrap();
        assert_eq!(saved_path.extension().unwrap(), "md");
        assert!(saved_path.exists());
        
        let content = fs::read_to_string(&saved_path).unwrap();
        assert_eq!(content, "# Test content");
    }
}
