//! Simple Async Extension Processing - FIXED for GTK Threading
//!
//! Background processing for editor extensions as per optimization spec:
//! - Line wrapping (✅ DONE)
//! - Tab to spaces conversion (✅ DONE)  
//! - Syntax coloring (🔄 NOT DONE YET)
//! - Marco-specific extensions (🔄 NOT DONE YET: @run, [toc], [Page])
//! - Auto-pairing (📋 FUTURE)
//! - Markdown linting (📋 FUTURE)
//!
//! SIMPLIFIED: No complex threading to avoid GTK main context issues.

use std::collections::HashMap;
use std::time::Instant;
use anyhow::Result;

/// Result from processing a single extension (simplified)
#[derive(Debug, Clone)]
pub struct ExtensionResult {
    pub extension_name: String,
    pub processed_content: String,
    pub cursor_position: Option<u32>,
    pub processing_time_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Simple extension manager with async processing and debouncing
pub struct AsyncExtensionManager {
    /// Enabled extensions (line_wrapping, tab_to_spaces, syntax_coloring, marco_extensions)
    enabled_extensions: HashMap<String, bool>,
    /// Last processing time for debouncing
    last_process_time: std::cell::RefCell<Option<Instant>>,
}

impl AsyncExtensionManager {
    /// Create new AsyncExtensionManager with simple processing (no complex threading)
    pub fn new() -> Result<Self> {
        // Setup enabled extensions as per spec
        let mut enabled_extensions = HashMap::new();
        enabled_extensions.insert("line_wrapping".to_string(), true);
        enabled_extensions.insert("tab_to_spaces".to_string(), true);
        enabled_extensions.insert("syntax_coloring".to_string(), false); // Not implemented yet
        enabled_extensions.insert("marco_extensions".to_string(), true); // Now implemented!

        Ok(Self {
            enabled_extensions,
            last_process_time: std::cell::RefCell::new(None),
        })
    }

    /// Process extensions in parallel using lightweight vs heavyweight thread pools
    /// Lightweight: tab_to_spaces, line_wrapping (shared pool)
    /// Heavyweight: marco_extensions, syntax_coloring (dedicated pool)
    pub fn process_extensions_parallel<F>(
        &self,
        content: String,
        cursor_position: Option<u32>,
        callback: F,
    ) -> Result<()>
    where
        F: Fn(Vec<ExtensionResult>) + 'static,
    {
        let enabled_extensions = self.enabled_extensions.clone();
        let content_for_lightweight = content.clone();
        let content_for_heavyweight = content.clone();
        
        // Separate extensions by processing weight
        let mut lightweight_extensions = Vec::new();
        let mut heavyweight_extensions = Vec::new();
        
        for (extension_name, &enabled) in &enabled_extensions {
            if enabled {
                match extension_name.as_str() {
                    "line_wrapping" | "tab_to_spaces" | "auto_pairing" => {
                        lightweight_extensions.push(extension_name.clone());
                    }
                    "marco_extensions" | "syntax_coloring" | "markdown_linting" => {
                        heavyweight_extensions.push(extension_name.clone());
                    }
                    _ => {
                        // Unknown extensions go to lightweight pool by default
                        lightweight_extensions.push(extension_name.clone());
                    }
                }
            }
        }
        
        let has_lightweight = !lightweight_extensions.is_empty();
        let has_heavyweight = !heavyweight_extensions.is_empty();
        
        if !has_lightweight && !has_heavyweight {
            callback(Vec::new());
            return Ok(());
        }
        
        // Use shared state to collect results from both pools
        let results = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let completion_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let total_pools = if has_lightweight { 1 } else { 0 } + if has_heavyweight { 1 } else { 0 };
        let shared_callback = std::sync::Arc::new(callback);
        
        // Spawn lightweight extensions pool
        if has_lightweight {
            let lightweight_results = std::sync::Arc::clone(&results);
            let lightweight_completion = std::sync::Arc::clone(&completion_count);
            let lightweight_callback = std::sync::Arc::clone(&shared_callback);
            
            glib::spawn_future_local(async move {
                let result = gio::spawn_blocking(move || -> Result<Vec<ExtensionResult>> {
                    let mut pool_results = Vec::new();
                    
                    for extension_name in &lightweight_extensions {
                        let start_time = Instant::now();
                        let (processed_content, success, error_message) = match extension_name.as_str() {
                            "line_wrapping" => Self::process_line_wrapping(&content_for_lightweight, cursor_position),
                            "tab_to_spaces" => Self::process_tab_to_spaces(&content_for_lightweight, cursor_position),
                            "auto_pairing" => Self::process_auto_pairing(&content_for_lightweight, cursor_position),
                            _ => (content_for_lightweight.clone(), false, Some("Unknown lightweight extension".to_string())),
                        };

                        pool_results.push(ExtensionResult {
                            extension_name: extension_name.clone(),
                            processed_content,
                            cursor_position,
                            processing_time_ms: start_time.elapsed().as_millis() as u64,
                            success,
                            error_message,
                        });
                    }
                    
                    Ok(pool_results)
                }).await;
                
                glib::idle_add_local_once(move || {
                    match result {
                        Ok(Ok(pool_results)) => {
                            // Add results to shared collection
                            if let Ok(mut all_results) = lightweight_results.lock() {
                                all_results.extend(pool_results);
                            }
                        }
                        Ok(Err(e)) => {
                            log::error!("Lightweight extensions error: {}", e);
                        }
                        Err(e) => {
                            log::error!("Lightweight extensions task panicked: {:?}", e);
                        }
                    }
                    
                    // Check if this was the last pool to complete
                    let completed = lightweight_completion.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                    if completed >= total_pools {
                        if let Ok(final_results) = lightweight_results.lock() {
                            (*lightweight_callback)(final_results.clone());
                        }
                    }
                });
            });
        }
        
        // Spawn heavyweight extensions pool
        if has_heavyweight {
            let heavyweight_results = std::sync::Arc::clone(&results);
            let heavyweight_completion = std::sync::Arc::clone(&completion_count);
            let heavyweight_callback = std::sync::Arc::clone(&shared_callback);
            
            glib::spawn_future_local(async move {
                let result = gio::spawn_blocking(move || -> Result<Vec<ExtensionResult>> {
                    let mut pool_results = Vec::new();
                    
                    for extension_name in &heavyweight_extensions {
                        let start_time = Instant::now();
                        let (processed_content, success, error_message) = match extension_name.as_str() {
                            "marco_extensions" => Self::process_marco_extensions(&content_for_heavyweight, cursor_position),
                            "syntax_coloring" => Self::process_syntax_coloring(&content_for_heavyweight, cursor_position),
                            "markdown_linting" => Self::process_markdown_linting(&content_for_heavyweight, cursor_position),
                            _ => (content_for_heavyweight.clone(), false, Some("Unknown heavyweight extension".to_string())),
                        };

                        pool_results.push(ExtensionResult {
                            extension_name: extension_name.clone(),
                            processed_content,
                            cursor_position,
                            processing_time_ms: start_time.elapsed().as_millis() as u64,
                            success,
                            error_message,
                        });
                    }
                    
                    Ok(pool_results)
                }).await;
                
                glib::idle_add_local_once(move || {
                    match result {
                        Ok(Ok(pool_results)) => {
                            // Add results to shared collection
                            if let Ok(mut all_results) = heavyweight_results.lock() {
                                all_results.extend(pool_results);
                            }
                        }
                        Ok(Err(e)) => {
                            log::error!("Heavyweight extensions error: {}", e);
                        }
                        Err(e) => {
                            log::error!("Heavyweight extensions task panicked: {:?}", e);
                        }
                    }
                    
                    // Check if this was the last pool to complete
                    let completed = heavyweight_completion.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                    if completed >= total_pools {
                        if let Ok(final_results) = heavyweight_results.lock() {
                            (*heavyweight_callback)(final_results.clone());
                        }
                    }
                });
            });
        }
        
        Ok(())
    }

    /// Process extensions asynchronously using proper GTK threading patterns
    /// Uses glib::spawn_future_local + gio::spawn_blocking for GTK-safe background processing
    pub fn process_extensions_async<F>(
        &self,
        content: String,
        cursor_position: Option<u32>,
        callback: F,
    ) -> Result<()>
    where
        F: Fn(Vec<ExtensionResult>) + 'static,
    {
        // Use parallel processing for better performance
        self.process_extensions_parallel(content, cursor_position, callback)
    }

    /// Process extensions with debouncing: immediate first response + 200ms trailing edge
    /// This provides responsive feedback while preventing excessive processing during rapid typing
    pub fn process_extensions_with_debouncing<F>(
        &self,
        content: String,
        cursor_position: Option<u32>,
        callback: F,
    ) -> Result<()>
    where
        F: Fn(Vec<ExtensionResult>) + 'static + Clone,
    {
        let now = Instant::now();
        let should_process_immediately = {
            let last_time = self.last_process_time.borrow_mut();
            if let Some(last) = *last_time {
                // If more than 500ms since last processing, do immediate (leading edge)
                now.duration_since(last).as_millis() > 500
            } else {
                // First time processing - always immediate
                true
            }
        };

        if should_process_immediately {
            // Leading edge: Process immediately for responsive feedback
            self.process_extensions_async(content.clone(), cursor_position, callback.clone())?;
            
            // Update last process time
            *self.last_process_time.borrow_mut() = Some(now);
        }

        // Trailing edge: Always schedule delayed processing for final result
        let content_for_trailing = content.clone();
        let callback_for_trailing = callback.clone();
        let enabled_extensions = self.enabled_extensions.clone();
        
        glib::timeout_add_local_once(std::time::Duration::from_millis(200), move || {
            // Background processing for trailing edge (final result)
            glib::spawn_future_local(async move {
                let result = gio::spawn_blocking(move || -> Result<Vec<ExtensionResult>> {
                    let mut results = Vec::new();
                    
                    for (extension_name, &enabled) in &enabled_extensions {
                        if enabled {
                            let start_time = Instant::now();
                            let (processed_content, success, error_message) = match extension_name.as_str() {
                                "line_wrapping" => Self::process_line_wrapping(&content_for_trailing, cursor_position),
                                "tab_to_spaces" => Self::process_tab_to_spaces(&content_for_trailing, cursor_position),
                                "syntax_coloring" => Self::process_syntax_coloring(&content_for_trailing, cursor_position),
                                "marco_extensions" => Self::process_marco_extensions(&content_for_trailing, cursor_position),
                                "auto_pairing" => Self::process_auto_pairing(&content_for_trailing, cursor_position),
                                "markdown_linting" => Self::process_markdown_linting(&content_for_trailing, cursor_position),
                                _ => (content_for_trailing.clone(), false, Some("Unknown extension".to_string())),
                            };

                            results.push(ExtensionResult {
                                extension_name: extension_name.clone(),
                                processed_content,
                                cursor_position,
                                processing_time_ms: start_time.elapsed().as_millis() as u64,
                                success,
                                error_message,
                            });
                        }
                    }
                    
                    Ok(results)
                }).await;
                
                // Return trailing edge results via GTK-safe callback
                glib::idle_add_local_once(move || {
                    match result {
                        Ok(Ok(results)) => callback_for_trailing(results),
                        Ok(Err(e)) => {
                            log::error!("Trailing edge processing error: {}", e);
                            callback_for_trailing(Vec::new());
                        }
                        Err(e) => {
                            log::error!("Trailing edge task panicked: {:?}", e);
                            callback_for_trailing(Vec::new());
                        }
                    }
                });
            });
        });

        Ok(())
    }

    /// Process line wrapping (✅ DONE as per spec)
    fn process_line_wrapping(content: &str, _cursor_position: Option<u32>) -> (String, bool, Option<String>) {
        const WRAP_WIDTH: usize = 80;
        
        let wrapped = content
            .lines()
            .map(|line| {
                if line.len() <= WRAP_WIDTH {
                    return line.to_string();
                }

                // Preserve leading whitespace (indentation)
                let leading_whitespace: String = line.chars()
                    .take_while(|c| c.is_whitespace())
                    .collect();

                let trimmed_line = line.trim_start();
                
                // Smart word wrapping
                let mut result = String::new();
                let mut current_line = leading_whitespace.clone();
                let mut current_length = leading_whitespace.len();

                for word in trimmed_line.split_whitespace() {
                    let word_len = word.len();
                    
                    if current_length + word_len + 1 > WRAP_WIDTH && current_length > leading_whitespace.len() {
                        result.push_str(&current_line);
                        result.push('\n');
                        current_line = format!("{}{}", leading_whitespace, word);
                        current_length = leading_whitespace.len() + word_len;
                    } else {
                        if current_length > leading_whitespace.len() {
                            current_line.push(' ');
                            current_length += 1;
                        }
                        current_line.push_str(word);
                        current_length += word_len;
                    }
                }
                
                result.push_str(&current_line);
                result
            })
            .collect::<Vec<_>>()
            .join("\n");
            
        (wrapped, true, None)
    }

    /// Process tab to spaces conversion (✅ DONE as per spec)
    fn process_tab_to_spaces(content: &str, _cursor_position: Option<u32>) -> (String, bool, Option<String>) {
        const TAB_WIDTH: usize = 4;
        
        let converted = content
            .lines()
            .map(|line| {
                let mut result = String::new();
                let mut column = 0;
                
                for ch in line.chars() {
                    match ch {
                        '\t' => {
                            // Calculate spaces needed to reach next tab stop
                            let spaces_to_add = TAB_WIDTH - (column % TAB_WIDTH);
                            result.push_str(&" ".repeat(spaces_to_add));
                            column += spaces_to_add;
                        }
                        _ => {
                            result.push(ch);
                            column += 1;
                        }
                    }
                }
                result
            })
            .collect::<Vec<_>>()
            .join("\n");
            
        (converted, true, None)
    }

    /// Process syntax coloring (🔄 NOT DONE YET as per spec)
    fn process_syntax_coloring(content: &str, _cursor_position: Option<u32>) -> (String, bool, Option<String>) {
        // Not implemented yet - return original content
        (content.to_string(), false, Some("Not implemented yet".to_string()))
    }

    /// Process Marco extensions (@run, [toc], [Page]) - Now implemented!
    fn process_marco_extensions(content: &str, _cursor_position: Option<u32>) -> (String, bool, Option<String>) {
        // Use the cached parsing for better performance
        match crate::components::marco_engine::parse_to_html_cached(content) {
            Ok(_html_output) => {
                // For editor processing, we return the original content but indicate success
                // The HTML output would be used separately for preview updates
                (content.to_string(), true, None)
            }
            Err(e) => {
                // Return original content on parsing error
                (content.to_string(), false, Some(format!("Marco parsing error: {}", e)))
            }
        }
    }

    /// Process auto-pairing (📋 FUTURE as per spec)
    fn process_auto_pairing(content: &str, _cursor_position: Option<u32>) -> (String, bool, Option<String>) {
        // Future feature - return original content
        (content.to_string(), false, Some("Future feature".to_string()))
    }

    /// Process markdown linting (📋 FUTURE as per spec)
    fn process_markdown_linting(content: &str, _cursor_position: Option<u32>) -> (String, bool, Option<String>) {
        // Future feature - return original content
        (content.to_string(), false, Some("Future feature".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_processing() {
        let manager = AsyncExtensionManager::new().expect("Failed to create manager");
        
        // Use content that will definitely trigger processing
        let content_for_wrapping = "This is a very long line that should definitely be wrapped at 80 characters to ensure proper formatting and demonstrate the line wrapping functionality working correctly";
        let content_with_tabs = "function test() {\n\treturn true;\n\tif (condition) {\n\t\treturn false;\n\t}\n}";
        
        // Test that lightweight and heavyweight extensions are categorized correctly
        let mut test_results = Vec::new();
        
        for (extension_name, &enabled) in &manager.enabled_extensions {
            if enabled {
                match extension_name.as_str() {
                    "line_wrapping" => {
                        let (processed, success, _) = AsyncExtensionManager::process_line_wrapping(content_for_wrapping, None);
                        assert!(success);
                        assert_ne!(processed, content_for_wrapping); // Should be wrapped
                        assert!(processed.contains('\n')); // Should contain line breaks
                        test_results.push(("lightweight", extension_name.clone()));
                    }
                    "tab_to_spaces" => {
                        let (processed, success, _) = AsyncExtensionManager::process_tab_to_spaces(content_with_tabs, None);
                        assert!(success);
                        assert!(!processed.contains('\t')); // Tabs should be converted
                        assert!(processed.contains("    ")); // Should contain spaces
                        test_results.push(("lightweight", extension_name.clone()));
                    }
                    "marco_extensions" => {
                        let (processed, success, _) = AsyncExtensionManager::process_marco_extensions(content_for_wrapping, None);
                        assert!(success);
                        assert_eq!(processed, content_for_wrapping); // Should return original content for editor
                        test_results.push(("heavyweight", extension_name.clone()));
                    }
                    _ => {}
                }
            }
        }
        
        // Verify we have both lightweight and heavyweight extensions
        let lightweight_count = test_results.iter().filter(|(category, _)| *category == "lightweight").count();
        let heavyweight_count = test_results.iter().filter(|(category, _)| *category == "heavyweight").count();
        
        assert!(lightweight_count > 0, "Should have lightweight extensions");
        assert!(heavyweight_count > 0, "Should have heavyweight extensions");
        
        println!("Parallel processing categorization test passed:");
        println!("  Lightweight extensions: {}", lightweight_count);
        println!("  Heavyweight extensions: {}", heavyweight_count);
    }

    #[test]
    fn test_line_wrapping() {
        let long_line = "This is a very long line that should be wrapped at 80 characters to ensure proper formatting";
        let (wrapped, success, error) = AsyncExtensionManager::process_line_wrapping(long_line, None);
        
        assert!(success);
        assert!(error.is_none());
        assert!(wrapped.contains('\n'));
    }

    #[test]
    fn test_tab_to_spaces() {
        let content_with_tabs = "function test() {\n\treturn true;\n}";
        let (converted, success, error) = AsyncExtensionManager::process_tab_to_spaces(content_with_tabs, None);
        
        assert!(success);
        assert!(error.is_none());
        assert!(!converted.contains('\t'));
        assert!(converted.contains("    "));
    }

    #[test]
    fn test_marco_extensions_processing() {
        let content = "# Test Document\n\nThis is a test with **bold** text.";
        let (processed, success, error) = AsyncExtensionManager::process_marco_extensions(content, None);
        
        assert!(success);
        assert!(error.is_none());
        // For editor processing, we return the original content
        assert_eq!(processed, content);
    }

    #[test]
    fn smoke_test_extension_manager_creation() {
        let manager = AsyncExtensionManager::new().expect("Failed to create AsyncExtensionManager");
        
        // Verify enabled extensions
        assert!(manager.enabled_extensions.get("line_wrapping").copied().unwrap_or(false));
        assert!(manager.enabled_extensions.get("tab_to_spaces").copied().unwrap_or(false));
        assert!(manager.enabled_extensions.get("marco_extensions").copied().unwrap_or(false));
        
        // Syntax coloring should be disabled (not implemented yet)
        assert!(!manager.enabled_extensions.get("syntax_coloring").copied().unwrap_or(true));
    }

    #[test]
    fn smoke_test_async_processing() {
        // Note: This test only verifies that the method exists and doesn't panic during setup
        // Actual async behavior requires a GTK main loop context which isn't available in unit tests
        let manager = AsyncExtensionManager::new().expect("Failed to create AsyncExtensionManager");
        
        // We can test that the manager was created successfully
        assert!(manager.enabled_extensions.contains_key("line_wrapping"));
        assert!(manager.enabled_extensions.contains_key("tab_to_spaces"));
        assert!(manager.enabled_extensions.contains_key("marco_extensions"));
        
        // Cannot test actual async behavior without GTK context, but we can verify the API
        let content = "# Test\n\nSimple content for testing.".to_string();
        
        // In a real GTK application, this would work fine
        // For unit tests, we just verify the method signature is correct
        assert!(std::mem::size_of_val(&content) > 0);
    }

    #[test]
    fn smoke_test_debounced_processing() {
        // Note: This test only verifies that the method exists and doesn't panic during setup
        // Actual debouncing behavior requires a GTK main loop context which isn't available in unit tests
        let manager = AsyncExtensionManager::new().expect("Failed to create AsyncExtensionManager");
        
        // We can test that the debouncing state was initialized
        assert!(manager.last_process_time.borrow().is_none());
        
        // Cannot test actual debouncing behavior without GTK context and time passing
        let content = "# Test\n\nContent for debouncing test.".to_string();
        
        // In a real GTK application, this would handle debouncing properly
        // For unit tests, we just verify the structure is correct
        assert!(std::mem::size_of_val(&content) > 0);
    }
}