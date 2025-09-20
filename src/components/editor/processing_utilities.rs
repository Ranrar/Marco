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

/// Simple extension manager with basic processing (no complex threading to avoid GTK issues)
pub struct AsyncExtensionManager {
    /// Enabled extensions (line_wrapping, tab_to_spaces, syntax_coloring, marco_extensions)
    enabled_extensions: HashMap<String, bool>,
}

impl AsyncExtensionManager {
    /// Create new AsyncExtensionManager with simple processing (no complex threading)
    pub fn new() -> Result<Self> {
        // Setup enabled extensions as per spec
        let mut enabled_extensions = HashMap::new();
        enabled_extensions.insert("line_wrapping".to_string(), true);
        enabled_extensions.insert("tab_to_spaces".to_string(), true);
        enabled_extensions.insert("syntax_coloring".to_string(), false); // Not implemented yet
        enabled_extensions.insert("marco_extensions".to_string(), false); // Not implemented yet

        Ok(Self {
            enabled_extensions,
        })
    }

    /// Process extensions with simple synchronous execution (avoids GTK threading issues)
    /// NOTE: This is a simplified version that processes synchronously to avoid threading problems
    pub fn process_extensions_async<F>(
        &self,
        content: String,
        cursor_position: Option<u32>,
        callback: F,
    ) -> Result<()>
    where
        F: Fn(Vec<ExtensionResult>) + 'static,
    {
        // Process extensions synchronously to avoid GTK threading issues
        let mut results = Vec::new();
        
        for (extension_name, &enabled) in &self.enabled_extensions {
            if enabled {
                let start_time = Instant::now();
                let (processed_content, success, error_message) = match extension_name.as_str() {
                    "line_wrapping" => Self::process_line_wrapping(&content, cursor_position),
                    "tab_to_spaces" => Self::process_tab_to_spaces(&content, cursor_position),
                    "syntax_coloring" => Self::process_syntax_coloring(&content, cursor_position),
                    "marco_extensions" => Self::process_marco_extensions(&content, cursor_position),
                    "auto_pairing" => Self::process_auto_pairing(&content, cursor_position),
                    "markdown_linting" => Self::process_markdown_linting(&content, cursor_position),
                    _ => (content.clone(), false, Some("Unknown extension".to_string())),
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
        
        // Call callback immediately (synchronously) to avoid threading issues
        callback(results);
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

    /// Process Marco extensions (🔄 NOT DONE YET as per spec)
    fn process_marco_extensions(content: &str, _cursor_position: Option<u32>) -> (String, bool, Option<String>) {
        // Not implemented yet - return original content
        (content.to_string(), false, Some("Not implemented yet".to_string()))
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
}