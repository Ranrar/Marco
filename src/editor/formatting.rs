use gtk4::prelude::*;
use crate::editor::core::MarkdownEditor;

impl MarkdownEditor {
    pub fn insert_bold(&self) {
        self.toggle_format_selection_only_with_dialog("**", "**", "bold");
    }

    pub fn insert_italic(&self) {
        self.toggle_format_selection_only_with_dialog("*", "*", "italic");
    }

    pub fn insert_inline_code(&self) {
        self.toggle_format_selection_only_with_dialog("`", "`", "inline code");
    }

    pub fn insert_strikethrough(&self) {
        self.toggle_format_selection_only_with_dialog("~~", "~~", "strikethrough");
    }

    pub fn insert_highlight(&self) {
        self.toggle_format_selection_only_with_dialog("==", "==", "highlight");
    }

    pub fn insert_subscript(&self) {
        self.toggle_format_selection_only_with_dialog("~", "~", "subscript");
    }

    pub fn insert_superscript(&self) {
        self.toggle_format_selection_only_with_dialog("^", "^", "superscript");
    }

    /// Show a dialog notifying the user that text must be selected for the formatting function
    fn show_text_selection_required_dialog(&self, parent: &gtk4::Window, feature_name: &str) {
        use gtk4::MessageDialog;
        
        let title = "Text Selection Required";
        let message = format!("Please select text in the editor before applying {} formatting.", feature_name);
        
        let dialog = MessageDialog::builder()
            .transient_for(parent)
            .modal(true)
            .message_type(gtk4::MessageType::Info)
            .text(title)
            .secondary_text(message)
            .build();
        
        dialog.add_button("OK", gtk4::ResponseType::Ok);
        dialog.set_default_response(gtk4::ResponseType::Ok);
        
        dialog.connect_response(|dialog, _| {
            dialog.close();
        });
        
        dialog.present();
    }

    fn toggle_format_selection_only(&self, prefix: &str, suffix: &str) {
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        
        println!("DEBUG: toggle_format_selection_only called with prefix='{}', suffix='{}'", prefix, suffix);
        
        // Only work if text is selected
        if !gtk_buffer.has_selection() {
            println!("DEBUG: No selection available in toggle_format_selection_only");
            return;
        }
        
        println!("DEBUG: Has selection at start of toggle_format_selection_only");
        
        // Store original selection bounds for restoration
        let (original_start, original_end) = gtk_buffer.selection_bounds().unwrap();
        
        // Try to expand selection to include formatting if user selected inner text
        let (actual_start, actual_end, text) = if let Some((expanded_start, expanded_end, expanded_text)) = self.expand_selection_to_include_formatting() {
            println!("DEBUG: Expanded selection to include formatting: '{}'", expanded_text);
            // Update selection to expanded range temporarily
            gtk_buffer.select_range(&expanded_start, &expanded_end);
            (expanded_start, expanded_end, expanded_text)
        } else {
            // Use original selection
            let text = gtk_buffer.text(&original_start, &original_end, false).to_string();
            println!("DEBUG: Using original selection: '{}'", text);
            (original_start, original_end, text)
        };
        
        // Calculate new text after formatting
        let new_text = if self.has_specific_formatting(&text, prefix, suffix) {
            println!("DEBUG: Removing existing formatting");
            let result = self.remove_specific_formatting(&text, prefix, suffix);
            println!("DEBUG: New text after removing formatting: '{}'", result);
            result
        } else {
            println!("DEBUG: Applying new formatting");  
            let result = self.apply_smart_formatting(&text, prefix, suffix);
            println!("DEBUG: New text after applying formatting: '{}'", result);
            result
        };
        
        // Perform the text replacement atomically
        buffer.begin_user_action();
        
        // Create marks to track position
        let start_mark = gtk_buffer.create_mark(None, &actual_start, false);
        
        // Replace text
        let mut start_mut = actual_start;
        let mut end_mut = actual_end;
        buffer.delete(&mut start_mut, &mut end_mut);
        
        // Insert new text
        let mut insert_iter = gtk_buffer.iter_at_mark(&start_mark);
        buffer.insert(&mut insert_iter, &new_text);
        
        // Calculate selection range for the new text
        let new_start_iter = gtk_buffer.iter_at_mark(&start_mark);
        let mut new_end_iter = new_start_iter;
        new_end_iter.forward_chars(new_text.chars().count() as i32);
        
        // Restore selection immediately 
        buffer.select_range(&new_start_iter, &new_end_iter);
        
        // Clean up mark
        gtk_buffer.delete_mark(&start_mark);
        
        buffer.end_user_action();
        
        println!("DEBUG: Completed formatting, has_selection: {}", gtk_buffer.has_selection());
        
        // Use a deferred action to ensure selection persists
        let gtk_buffer_clone = gtk_buffer.clone();
        let source_view = self.source_view().clone();
        
        // Schedule selection verification and restoration
        glib::idle_add_local_once(move || {
            // Force focus back to text view
            source_view.grab_focus();
            
            if gtk_buffer_clone.has_selection() {
                let (sel_start, sel_end) = gtk_buffer_clone.selection_bounds().unwrap();
                let selected = gtk_buffer_clone.text(&sel_start, &sel_end, false);
                println!("DEBUG: Selection preserved successfully: '{}'", selected);
            } else {
                println!("DEBUG: Selection was lost, attempting to restore...");
                
                // Try to restore selection using the new text
                let current_cursor = gtk_buffer_clone.iter_at_mark(&gtk_buffer_clone.get_insert());
                let cursor_offset = current_cursor.offset();
                
                // Attempt to restore based on the position and new text length
                let new_start = gtk_buffer_clone.iter_at_offset(cursor_offset - new_text.chars().count() as i32);
                let new_end = gtk_buffer_clone.iter_at_offset(cursor_offset);
                
                if new_start.offset() >= 0 && new_end.offset() <= gtk_buffer_clone.char_count() {
                    gtk_buffer_clone.select_range(&new_start, &new_end);
                    println!("DEBUG: Selection restored to new text position");
                } else {
                    println!("DEBUG: Could not restore selection - invalid range");
                }
            }
        });
    }
    
    fn toggle_format_selection_only_with_dialog(&self, prefix: &str, suffix: &str, feature_name: &str) {
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        
        // Only work if text is selected
        if gtk_buffer.has_selection() {
            println!("DEBUG: Selection exists before formatting");
            
            // Store the current selection state for debugging
            let (start, end) = gtk_buffer.selection_bounds().unwrap();
            let selected_text = gtk_buffer.text(&start, &end, false);
            println!("DEBUG: Selected text before formatting: '{}'", selected_text);
            
            // Ensure the text view has focus before applying formatting
            let source_view = self.source_view().clone();
            source_view.grab_focus();
            
            // Apply the formatting - this function handles selection preservation internally
            self.toggle_format_selection_only(prefix, suffix);
            
            // Record that we just performed a formatting action
            *self.last_formatting_action.borrow_mut() = Some(std::time::Instant::now());
            
            // Enhanced focus and selection restoration
            let gtk_buffer_clone = gtk_buffer.clone();
            let source_view_clone = source_view.clone();
            
            // Multiple strategies to ensure focus and selection are maintained
            
            // Strategy 1: Immediate focus restoration
            source_view.grab_focus();
            
            // Strategy 2: Delayed restoration to handle async GTK events
            glib::timeout_add_local_once(std::time::Duration::from_millis(1), move || {
                source_view_clone.grab_focus();
                
                if gtk_buffer_clone.has_selection() {
                    let (new_start, new_end) = gtk_buffer_clone.selection_bounds().unwrap();
                    let new_selected_text = gtk_buffer_clone.text(&new_start, &new_end, false);
                    println!("DEBUG: Selection maintained after 1ms delay: '{}'", new_selected_text);
                } else {
                    println!("DEBUG: Selection lost after 1ms delay!");
                }
            });
            
            // Strategy 3: Secondary delayed check for robustness
            let gtk_buffer_clone2 = gtk_buffer.clone();
            let source_view_clone2 = source_view.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(10), move || {
                source_view_clone2.grab_focus();
                
                if gtk_buffer_clone2.has_selection() {
                    let (new_start, new_end) = gtk_buffer_clone2.selection_bounds().unwrap();
                    let new_selected_text = gtk_buffer_clone2.text(&new_start, &new_end, false);
                    println!("DEBUG: Final selection check (10ms): '{}'", new_selected_text);
                } else {
                    println!("DEBUG: Final check - selection lost at 10ms!");
                }
            });
            
        } else {
            // Check if a formatting action was performed very recently (within 500ms)
            // This helps prevent double-triggering issues where the selection is lost
            // between multiple rapid calls to the same formatting function
            let recent_action = self.last_formatting_action.borrow()
                .map(|instant| instant.elapsed().as_millis() < 500)
                .unwrap_or(false);
                
            if recent_action {
                println!("DEBUG: Skipping error dialog due to recent formatting action");
                return; // Skip error dialog for recent formatting actions
            }
            
            // Show error dialog if no text is selected
            if let Some(window) = self.source_view().root()
                .and_then(|root| root.downcast::<gtk4::Window>().ok()) {
                self.show_text_selection_required_dialog(&window, feature_name);
            }
        }
    }

    /// Check if the text has this specific formatting (handles mixed formatting and overlapping patterns)
    fn has_specific_formatting(&self, text: &str, prefix: &str, suffix: &str) -> bool {
        // Special handling for combined bold+italic format (***text***)
        if text.starts_with("***") && text.ends_with("***") && text.len() > 6 {
            // This text has both bold and italic
            if (prefix == "**" && suffix == "**") || (prefix == "*" && suffix == "*") {
                return true;
            }
        }
        
        // Method 1: Check if the text is exactly the format (e.g., "**bold**")
        if text.starts_with(prefix) && text.ends_with(suffix) && text.len() > prefix.len() + suffix.len() {
            // Special handling for bold vs italic conflict
            if prefix == "*" && suffix == "*" {
                // Make sure it's not actually bold (which uses **)
                // If it starts with ** it's bold, not italic
                if text.starts_with("**") {
                    return false;
                }
            }
            return true;
        }
        
        // Method 2: Check if this specific formatting exists as the outermost layer
        if text.len() > prefix.len() + suffix.len() {
            let potential_start = &text[..prefix.len()];
            let potential_end = &text[text.len() - suffix.len()..];
            
            if potential_start == prefix && potential_end == suffix {
                // Additional check for bold vs italic conflict
                if prefix == "*" && suffix == "*" {
                    // Make sure this is actually italic and not bold
                    if text.starts_with("**") {
                        return false;
                    }
                }
                return true;
            }
        }
        
        false
    }

    /// Remove specific formatting while preserving other formatting (handles overlapping patterns)
    fn remove_specific_formatting(&self, text: &str, prefix: &str, suffix: &str) -> String {
        // Special handling for combined bold+italic format (***text***)
        if text.starts_with("***") && text.ends_with("***") && text.len() > 6 {
            let inner_text = &text[3..text.len() - 3];
            if prefix == "**" && suffix == "**" {
                return format!("*{}*", inner_text);
            } else if prefix == "*" && suffix == "*" {
                return format!("**{}**", inner_text);
            }
        }
        
        // Method 1: Direct match - just strip the outer layer
        if text.starts_with(prefix) && text.ends_with(suffix) && text.len() > prefix.len() + suffix.len() {
            // Special handling for bold vs italic conflict
            if prefix == "*" && suffix == "*" {
                // Make sure it's not actually bold (which uses **)
                if text.starts_with("**") {
                    return text.to_string(); // Don't remove if it's actually bold
                }
            }
            return text[prefix.len()..text.len() - suffix.len()].to_string();
        }
        
        // Method 2: Check for outermost layer removal
        if text.len() > prefix.len() + suffix.len() {
            let potential_start = &text[..prefix.len()];
            let potential_end = &text[text.len() - suffix.len()..];
            
            if potential_start == prefix && potential_end == suffix {
                // Additional check for bold vs italic conflict
                if prefix == "*" && suffix == "*" {
                    if text.starts_with("**") {
                        return text.to_string(); // Don't remove if it's actually bold
                    }
                }
                return text[prefix.len()..text.len() - suffix.len()].to_string();
            }
        }
        
        // If we can't find the specific formatting, return as-is
        text.to_string()
    }

    /// Expand selection to include formatting if user selected the inner text
    fn expand_selection_to_include_formatting(&self) -> Option<(gtk4::TextIter, gtk4::TextIter, String)> {
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        
        if !gtk_buffer.has_selection() {
            return None;
        }
        
        let (start, end) = gtk_buffer.selection_bounds()?;
        let selected_text = gtk_buffer.text(&start, &end, false).to_string();
        
        // Get some context around the selection to check for formatting
        let mut expanded_start = start;
        let mut expanded_end = end;
        
        // Look backward for potential formatting markers
        for _ in 0..10 { // Look up to 10 characters back
            if !expanded_start.backward_char() {
                break;
            }
        }
        
        // Look forward for potential formatting markers  
        for _ in 0..10 { // Look up to 10 characters forward
            if !expanded_end.forward_char() {
                break;
            }
        }
        
        let expanded_text = gtk_buffer.text(&expanded_start, &expanded_end, false).to_string();
        
        // Check for formatting patterns around the selection - ORDER MATTERS!
        // Check longer patterns first to avoid substring conflicts
        let formatting_patterns = [
            ("**", "**"), // Bold (check before italic to avoid ** vs * conflict)
            ("~~", "~~"), // Strikethrough  
            ("==", "=="), // Highlight
            ("`", "`"),   // Code
            ("*", "*"),   // Italic (check after bold)
            ("~", "~"),   // Subscript
            ("^", "^"),   // Superscript
        ];
        
        // SPECIAL CASE: Check for combined bold+italic format (***text***) FIRST
        if let Some(selection_pos) = expanded_text.find(&selected_text) {
            let before_selection = &expanded_text[..selection_pos];
            let after_selection = &expanded_text[selection_pos + selected_text.len()..];
            
            if before_selection.ends_with("***") && after_selection.starts_with("***") {
                // Found combined bold+italic format
                let format_start = before_selection.len() - 3;
                let format_end = selection_pos + selected_text.len() + 3;
                let formatted_text = &expanded_text[format_start..format_end];
                
                // Calculate the actual positions
                let mut actual_start = expanded_start;
                actual_start.forward_chars(format_start as i32);
                let mut actual_end = expanded_start;
                actual_end.forward_chars(format_end as i32);
                
                return Some((actual_start, actual_end, formatted_text.to_string()));
            }
        }
        
        for (prefix, suffix) in &formatting_patterns {
            // Find where our original selection fits in the expanded text
            if let Some(selection_pos) = expanded_text.find(&selected_text) {
                let before_selection = &expanded_text[..selection_pos];
                let after_selection = &expanded_text[selection_pos + selected_text.len()..];
                
                // Check if the selection is surrounded by this formatting
                if before_selection.ends_with(prefix) && after_selection.starts_with(suffix) {
                    // Found formatting around the selection
                    let format_start = before_selection.len() - prefix.len();
                    let format_end = selection_pos + selected_text.len() + suffix.len();
                    let formatted_text = &expanded_text[format_start..format_end];
                    
                    // Calculate the actual positions
                    let mut actual_start = expanded_start;
                    actual_start.forward_chars(format_start as i32);
                    let mut actual_end = expanded_start;
                    actual_end.forward_chars(format_end as i32);
                    
                    return Some((actual_start, actual_end, formatted_text.to_string()));
                }
            }
        }
        
        None
    }

    /// Check if cursor is within a specific formatting pattern
    /// Returns true if the cursor position contains the formatting
    #[allow(dead_code)]
    pub fn is_cursor_in_format(&self, prefix: &str, suffix: &str) -> bool {
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        let cursor_iter = gtk_buffer.iter_at_mark(&gtk_buffer.get_insert());
        
        let line_start = gtk_buffer.iter_at_line(cursor_iter.line()).unwrap_or_else(|| cursor_iter);
        let mut line_end = gtk_buffer.iter_at_line(cursor_iter.line()).unwrap_or_else(|| cursor_iter);
        
        if !line_end.ends_line() {
            line_end.forward_to_line_end();
        }
        
        let line_text = gtk_buffer.text(&line_start, &line_end, false);
        let cursor_offset = cursor_iter.line_offset();
        
        self.find_format_at_cursor(&line_text, cursor_offset, prefix, suffix).is_some()
    }

    /// Check if cursor is on a heading line and return the heading level
    #[allow(dead_code)]
    pub fn get_heading_level_at_cursor(&self) -> Option<usize> {
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        let cursor_iter = gtk_buffer.iter_at_mark(&gtk_buffer.get_insert());
        
        let line_start = gtk_buffer.iter_at_line(cursor_iter.line()).unwrap_or_else(|| cursor_iter);
        let mut line_end = gtk_buffer.iter_at_line(cursor_iter.line()).unwrap_or_else(|| cursor_iter);
        
        if !line_end.ends_line() {
            line_end.forward_to_line_end();
        }
        
        let line_text = gtk_buffer.text(&line_start, &line_end, false);
        let trimmed = line_text.trim();
        
        // Check for heading pattern
        if trimmed.starts_with('#') {
            let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
            if hash_count <= 6 && trimmed.len() > hash_count && trimmed.chars().nth(hash_count) == Some(' ') {
                return Some(hash_count);
            }
        }
        
        None
    }

    /// Detect existing formatting on text to handle mixed formatting correctly
    fn detect_existing_formatting(&self, text: &str) -> Vec<(&'static str, &'static str)> {
        let mut found_formats = Vec::new();
        
        // Check for formatting patterns - order matters for overlapping patterns
        // We check longer patterns first to avoid false positives
        let patterns = [
            ("**", "**", "bold"),
            ("~~", "~~", "strikethrough"),
            ("==", "==", "highlight"),
            ("`", "`", "code"),
            ("*", "*", "italic"),
            ("~", "~", "subscript"),
            ("^", "^", "superscript"),
        ];
        
        for (prefix, suffix, _name) in &patterns {
            // Use the same logic as has_specific_formatting to be consistent
            if self.has_specific_formatting(text, prefix, suffix) {
                found_formats.push((*prefix, *suffix));
            }
        }
        
        found_formats
    }
    
    /// Smart formatting application that handles mixed formatting
    fn apply_smart_formatting(&self, text: &str, target_prefix: &str, target_suffix: &str) -> String {
        // Check if we already have this specific formatting - if so, this should be removal, not addition
        if self.has_specific_formatting(text, target_prefix, target_suffix) {
            // This should not happen - if we have the formatting, remove_specific_formatting should be called
            // But as a safety net, remove it here
            return self.remove_specific_formatting(text, target_prefix, target_suffix);
        }
        
        // Special handling for bold + italic combination
        if (target_prefix == "*" && self.has_specific_formatting(text, "**", "**")) ||
           (target_prefix == "**" && self.has_specific_formatting(text, "*", "*")) {
            // Only apply combined formatting if we don't already have both
            // Check if we already have the combined format
            if text.starts_with("***") && text.ends_with("***") && text.len() > 6 {
                // We already have combined format, this should not happen here
                // Return as-is to avoid malformed formatting
                return text.to_string();
            }
            
            // Remove the existing formatting and apply combined bold+italic
            let inner_text = if self.has_specific_formatting(text, "**", "**") {
                self.remove_specific_formatting(text, "**", "**")
            } else {
                self.remove_specific_formatting(text, "*", "*")
            };
            return format!("***{}***", inner_text);
        }
        
        // Detect existing formatting
        let existing_formats = self.detect_existing_formatting(text);
        
        if existing_formats.is_empty() {
            // No existing formatting, just add the new one
            format!("{}{}{}", target_prefix, text, target_suffix)
        } else {
            // For other combinations, use the layered approach
            // Remove all existing formatting first, then reapply with the new one
            let mut inner_text = text.to_string();
            
            // Remove existing formatting from outside to inside
            for (prefix, suffix) in &existing_formats {
                if inner_text.starts_with(prefix) && inner_text.ends_with(suffix) {
                    inner_text = inner_text[prefix.len()..inner_text.len() - suffix.len()].to_string();
                }
            }
            
            // Now build the new formatting with the target at the outermost layer
            let mut result = inner_text;
            
            // Apply existing formats from innermost to outermost (reverse order)
            for (prefix, suffix) in existing_formats.iter().rev() {
                result = format!("{}{}{}", prefix, result, suffix);
            }
            
            // Apply the new target formatting as the outermost layer
            format!("{}{}{}", target_prefix, result, target_suffix)
        }
    }
}
