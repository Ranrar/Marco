use crate::logic::cache::{cached, global_cache};
use crate::logic::swanson::SettingsManager;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;

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

        let buffer = Self {
            file_path: Some(path.to_path_buf()),
            is_modified: false,
            baseline_content: String::new(),
            display_name,
        };
        
        log::info!("Created document buffer for file: {} - ready to load content", path.display());
        Ok(buffer)
    }

    /// Reads the content of the file associated with this buffer
    ///
    /// Uses the global file cache to improve performance for repeated reads.
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
                // Use cached file operations for better performance
                cached::read_to_string(path)
                    .with_context(|| format!("Failed to read file: {}", path.display()))
            }
            None => Ok(String::new()), // Empty content for untitled documents
        }
    }

    /// Saves content to the file associated with this buffer
    ///
    /// Uses cached file operations and automatically invalidates the cache.
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
    /// - Invalidates file cache entry
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
                    std::fs::create_dir_all(parent).with_context(|| {
                        format!(
                            "Failed to create parent directories for: {}",
                            path.display()
                        )
                    })?;
                }

                // Write content directly
                std::fs::write(path, content)
                    .with_context(|| format!("Failed to write file: {}", path.display()))?;

                // Invalidate cache after write
                global_cache().invalidate_file(path);

                // Update baseline using optimized method (log first to avoid borrow issues)
                let content_size = content.len();
                log::info!("Saved file: {} ({} bytes) with cached operations", 
                    path.display(), content_size);
                
                self.update_baseline_and_state(content, false);
                self.log_document_state("save_content");
                Ok(())
            }
            None => Err(anyhow::anyhow!(
                "Cannot save: no file path set. Use save_as_content() instead."
            )),
        }
    }

    /// Saves content to a new file path (Save As operation)
    ///
    /// Uses cached file operations for better performance.
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
    /// - Invalidates cache entries
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
            std::fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Failed to create parent directories for: {}",
                    path.display()
                )
            })?;
        }

        // Write content directly
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write file: {}", path.display()))?;

        // Invalidate cache after write
        global_cache().invalidate_file(&path);

        // Update buffer state with enhanced logging
        let display_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let content_size = content.len();
        self.file_path = Some(path.clone());
        self.display_name = display_name;
        
        log::info!("Saved file as: {} ({} bytes) with cached operations - now tracking as open document", 
            path.display(), content_size);
        
        // After Save As, baseline matches the saved content - use optimized method
        self.update_baseline_and_state(content, false);
        self.log_document_state("save_as_content");
        Ok(())
    }

    /// Loads file content and sets it as the baseline (used when opening files)
    ///
    /// This method is useful when opening a file to ensure the baseline content
    /// matches what was loaded from disk, using the cache for better performance.
    ///
    /// # Returns
    /// * `Ok(String)` - The loaded content
    /// * `Err(anyhow::Error)` - If no file is associated or read fails
    ///
    /// # Side Effects
    /// - Sets `baseline_content` to the loaded content
    /// - Sets `is_modified` to `false`
    ///
    /// # Example
    /// ```no_run
    /// use marco::logic::buffer::DocumentBuffer;
    /// use std::path::Path;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let mut buffer = DocumentBuffer::new_from_file(Path::new("document.md"))?;
    /// let content = buffer.load_and_set_baseline()?;
    /// assert!(!buffer.is_modified);
    /// # Ok(())
    /// # }
    /// ```
    pub fn load_and_set_baseline(&mut self) -> Result<String> {
        let content = self.read_content()?;
        let content_size = content.len();
        
        // Enhanced logging with file info
        match &self.file_path {
            Some(path) => {
                log::info!("Loaded file: {} ({} bytes) - now tracking as open document", 
                    path.display(), content_size);
            }
            None => {
                log::debug!("Loaded content and set baseline for untitled document ({} bytes)", content_size);
            }
        }
        
        // Optimize: Set baseline and return content in one operation to avoid clone
        self.baseline_content = content.clone();
        self.is_modified = false;
        Ok(content)
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
    /// Update modification state by comparing the provided editor content with the baseline.
    /// Enhanced with logging for debugging purposes
    pub fn update_modified_from_content(&mut self, current_content: &str) {
        let modified = self.baseline_content != current_content;
        let was_modified = self.is_modified;
        self.is_modified = modified;
        
        // Log state changes for debugging
        if was_modified != modified {
            let state_change = if modified { "clean → modified" } else { "modified → clean" };
            log::debug!("Document state changed ({}): {:?}", state_change, self.file_path);
        }
    }

    /// Sets the baseline content (used after loading or saving a file)
    /// Optimized to avoid unnecessary allocations when content hasn't changed
    pub fn set_baseline(&mut self, content: &str) {
        // Optimization: Only allocate new string if content actually differs
        if self.baseline_content != content {
            let content_size = content.len();
            self.baseline_content = content.to_string();
            log::debug!("Updated baseline content ({} bytes) for: {:?}", 
                content_size, self.file_path);
        }
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

    /// Gets the directory containing the document file
    ///
    /// This is useful for resolving relative image paths in markdown documents.
    ///
    /// # Returns
    /// * `Some(PathBuf)` - Directory path containing the file
    /// * `None` - Document is untitled/unsaved or path has no parent
    ///
    /// # Example
    /// ```no_run
    /// use std::path::Path;
    /// use marco::logic::buffer::DocumentBuffer;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let buffer = DocumentBuffer::new_from_file(Path::new("/home/user/docs/readme.md"))?;
    /// let dir = buffer.get_directory_path();
    /// assert_eq!(dir.unwrap(), Path::new("/home/user/docs"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_directory_path(&self) -> Option<&Path> {
        self.file_path.as_deref()?.parent()
    }

    /// Generates a file:// URI for the document's directory
    ///
    /// This is used as a base URI for WebKit6 to resolve relative image paths
    /// in markdown documents. The base URI points to the directory containing
    /// the markdown file, allowing relative image references to work correctly.
    ///
    /// # Returns
    /// * `Some(String)` - file:// URI for the document directory
    /// * `None` - Document is untitled/unsaved or path has no parent
    ///
    /// # Example
    /// ```no_run
    /// use std::path::Path;
    /// use marco::logic::buffer::DocumentBuffer;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let buffer = DocumentBuffer::new_from_file(Path::new("/home/user/docs/readme.md"))?;
    /// let base_uri = buffer.get_base_uri_for_webview();
    /// assert!(base_uri.unwrap().starts_with("file://"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_base_uri_for_webview(&self) -> Option<String> {
        let dir_path = self.get_directory_path()?;
        Some(format!("file://{}/", dir_path.display()))
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
        let had_file = self.file_path.is_some();
        self.file_path = None;
        self.is_modified = false;
        self.display_name = "Untitled.md".to_string();
        
        if had_file {
            log::info!("Document reset to untitled state - closed file association");
        }
        self.log_document_state("reset_to_untitled");
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

    /// Optimized method to update both baseline content and modified state efficiently
    /// Reduces string allocations when the content hasn't actually changed
    pub fn update_baseline_and_state(&mut self, new_content: &str, mark_modified: bool) {
        // Only update baseline if content has changed to avoid unnecessary allocations
        if self.baseline_content != new_content {
            let old_size = self.baseline_content.len();
            let new_size = new_content.len();
            self.baseline_content = new_content.to_string();
            
            log::debug!("Baseline content updated for: {:?} (size: {} → {} bytes)", 
                self.file_path, old_size, new_size);
        }
        self.is_modified = mark_modified;
    }

    /// Gets document statistics for logging and monitoring
    pub fn get_document_stats(&self) -> DocumentStats {
        DocumentStats {
            file_path: self.file_path.clone(),
            display_name: self.display_name.clone(),
            is_modified: self.is_modified,
            baseline_size: self.baseline_content.len(),
            has_file_association: self.file_path.is_some(),
        }
    }

    /// Logs current document state - useful for debugging memory usage and file operations
    pub fn log_document_state(&self, operation: &str) {
        let stats = self.get_document_stats();
        match &stats.file_path {
            Some(path) => {
                log::info!("Document state after {}: {} ({} bytes, modified: {}) [{}]", 
                    operation, 
                    path.display(), 
                    stats.baseline_size, 
                    stats.is_modified,
                    stats.display_name);
            }
            None => {
                log::debug!("Document state after {}: {} ({} bytes, modified: {})", 
                    operation, 
                    stats.display_name, 
                    stats.baseline_size, 
                    stats.is_modified);
            }
        }
    }
}

/// Recent files manager for tracking and persisting recently opened files
///
/// This struct manages a list of recently opened files through the
/// swanson settings system for consistent persistence.
pub struct RecentFiles {
    settings_manager: Arc<SettingsManager>,
}

impl RecentFiles {
    /// Creates a new recent files manager
    ///
    /// # Arguments
    /// * `settings_manager` - Shared settings manager
    pub fn new(settings_manager: Arc<SettingsManager>) -> Self {
        Self {
            settings_manager,
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
        if let Err(e) = self.settings_manager.update_settings(|settings| {
            settings.add_recent_file(path);
        }) {
            eprintln!("[RecentFiles] Failed to save recent file: {}", e);
        }
    }

    /// Gets the list of recent files
    ///
    /// # Returns
    /// Vector of recent file paths (most recent first)
    pub fn get_files(&self) -> Vec<PathBuf> {
        let settings = self.settings_manager.get_settings();
        settings.get_recent_files()
    }

    /// Clears all recent files
    pub fn clear(&self) {
        if let Err(e) = self.settings_manager.update_settings(|settings| {
            settings.clear_recent_files();
        }) {
            eprintln!("[RecentFiles] Failed to clear recent files: {}", e);
        }
    }
}

/// Statistics about a document buffer for monitoring and debugging
#[derive(Debug, Clone)]
pub struct DocumentStats {
    pub file_path: Option<PathBuf>,
    pub display_name: String,
    pub is_modified: bool,
    pub baseline_size: usize,
    pub has_file_association: bool,
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
    fn test_recent_files() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.ron");

        let settings_manager = SettingsManager::initialize(settings_path).unwrap();
        let recent = RecentFiles::new(settings_manager);

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

        let settings_manager = SettingsManager::initialize(settings_path).unwrap();
        let recent = RecentFiles::new(settings_manager);

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
        buffer
            .save_as_content(&file_path, "# Test content")
            .unwrap();

        // Test the optimized stats method usage
        let stats = buffer.get_document_stats();
        assert!(stats.has_file_association);
        assert_eq!(stats.baseline_size, 14); // "# Test content".len()
        assert!(!stats.is_modified);
        
        assert!(buffer.file_path.is_some());
        let saved_path = buffer.file_path.as_ref().unwrap();
        assert_eq!(saved_path.extension().unwrap(), "md");
        assert!(saved_path.exists());

        let content = fs::read_to_string(saved_path).unwrap();
        assert_eq!(content, "# Test content");
    }

    #[test]
    fn smoke_test_buffer_optimizations() {
        // Test the optimized baseline update method
        let mut buffer = DocumentBuffer::new_untitled();
        
        // Initial state
        assert_eq!(buffer.baseline_content, "");
        assert!(!buffer.is_modified);
        
        // First update should allocate new string
        buffer.update_baseline_and_state("# Hello World", false);
        assert_eq!(buffer.baseline_content, "# Hello World");
        assert!(!buffer.is_modified);
        
        // Same content should not reallocate (optimization)
        let baseline_ptr = buffer.baseline_content.as_ptr();
        buffer.update_baseline_and_state("# Hello World", true);
        assert_eq!(buffer.baseline_content.as_ptr(), baseline_ptr); // Same pointer = no reallocation
        assert!(buffer.is_modified);
        
        // Different content should allocate
        buffer.update_baseline_and_state("# Different Content", false);
        assert_ne!(buffer.baseline_content.as_ptr(), baseline_ptr); // Different pointer = new allocation
        assert_eq!(buffer.baseline_content, "# Different Content");
        assert!(!buffer.is_modified);
    }

    #[test]
    fn smoke_test_document_stats() {
        let mut buffer = DocumentBuffer::new_untitled();
        buffer.baseline_content = "# Test Content".to_string();
        buffer.is_modified = true;
        
        let stats = buffer.get_document_stats();
        assert_eq!(stats.display_name, "Untitled.md");
        assert!(stats.is_modified);
        assert_eq!(stats.baseline_size, 14); // "# Test Content".len()
        assert!(!stats.has_file_association);
        assert!(stats.file_path.is_none());
    }

    #[test]
    fn smoke_test_optimized_set_baseline() {
        let mut buffer = DocumentBuffer::new_untitled();
        
        // First set
        buffer.set_baseline("Initial content");
        assert_eq!(buffer.baseline_content, "Initial content");
        assert!(!buffer.is_modified);
        
        // Same content should not reallocate
        let baseline_ptr = buffer.baseline_content.as_ptr();
        buffer.set_baseline("Initial content");
        assert_eq!(buffer.baseline_content.as_ptr(), baseline_ptr);
        assert!(!buffer.is_modified);
        
        // Different content should allocate
        buffer.set_baseline("New content");
        assert_ne!(buffer.baseline_content.as_ptr(), baseline_ptr);
        assert_eq!(buffer.baseline_content, "New content");
        assert!(!buffer.is_modified);
    }
}
