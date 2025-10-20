//! UTF-8 Input Sanitization and Validation
//!
//! This module provides defensive UTF-8 handling for all text input sources
//! (keyboard, clipboard, files). It ensures that invalid UTF-8 sequences are
//! safely handled and Unicode text is normalized before reaching the parser layer.
//!
//! # Architecture
//! ```text
//! Raw input (keyboard, clipboard, file)
//!        │
//!        ▼
//! [UTF-8 Validation → Unicode Normalization → Control Char Filter]  ← This module
//!        │
//!        ▼
//! Parser (nom, Markdown)
//!        │
//!        ▼
//! Renderer (SourceView5 + WebKit6)
//! ```
//!
//! # Strategy
//! 1. **Validate** - Check if input is valid UTF-8
//! 2. **Sanitize** - Replace invalid sequences with � (U+FFFD)
//! 3. **Normalize** - Apply Unicode NFC normalization (canonical composition)
//! 4. **Filter** - Remove control characters (except \n, \r, \t)
//! 5. **Standardize** - Normalize line endings to \n
//!
//! # Unicode Normalization
//! 
//! **Why NFC (Canonical Composition)?**
//! 
//! Unicode allows multiple representations of visually identical text:
//! - Precomposed form: `é` (U+00E9, single character)
//! - Decomposed form: `e` + `´` (U+0065 + U+0301, two characters)
//!
//! Without normalization:
//! - Parser may treat `café` and `café` as different strings
//! - Emphasis markers like `*café*` might fail if `é` is decomposed
//! - Em dashes (—, U+2014) vs hyphens (-, U+002D) stay distinct
//!
//! NFC normalization ensures:
//! - Canonically equivalent forms are unified
//! - Multi-script text is stable for tokenization
//! - Parser results are deterministic across platforms
//!
//! # Examples
//! ```
//! use core::logic::utf8::{sanitize_input, InputSource};
//!
//! // From keyboard input
//! let safe_text = sanitize_input(raw_bytes, InputSource::Keyboard);
//!
//! // From clipboard
//! let safe_text = sanitize_input(clipboard_bytes, InputSource::Clipboard);
//!
//! // From file
//! let safe_text = sanitize_input(file_bytes, InputSource::File);
//! ```

use std::borrow::Cow;
use unicode_normalization::UnicodeNormalization;

/// Source of the input text (for logging/diagnostics)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputSource {
    /// Direct keyboard input
    Keyboard,
    /// Clipboard paste
    Clipboard,
    /// File load
    File,
    /// Network/API
    Network,
    /// Unknown/other source
    Unknown,
}

impl std::fmt::Display for InputSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputSource::Keyboard => write!(f, "keyboard"),
            InputSource::Clipboard => write!(f, "clipboard"),
            InputSource::File => write!(f, "file"),
            InputSource::Network => write!(f, "network"),
            InputSource::Unknown => write!(f, "unknown"),
        }
    }
}

/// Statistics about UTF-8 sanitization operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizeStats {
    /// Original byte length
    pub original_bytes: usize,
    /// Final byte length (may differ due to replacements)
    pub sanitized_bytes: usize,
    /// Number of invalid UTF-8 sequences replaced
    pub invalid_sequences: usize,
    /// Number of null bytes removed
    pub null_bytes_removed: usize,
    /// Number of control characters removed
    pub control_chars_removed: usize,
    /// Number of line ending normalizations
    pub line_endings_normalized: usize,
    /// Whether Unicode NFC normalization was applied
    pub unicode_normalized: bool,
    /// Whether input was already valid UTF-8
    pub was_valid: bool,
}

impl SanitizeStats {
    /// Check if any sanitization occurred
    pub fn had_issues(&self) -> bool {
        !self.was_valid
            || self.invalid_sequences > 0
            || self.null_bytes_removed > 0
            || self.control_chars_removed > 0
            || self.line_endings_normalized > 0
    }

    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        if !self.had_issues() && !self.unicode_normalized {
            return "Input was clean UTF-8".to_string();
        }

        let mut parts = Vec::new();
        if self.invalid_sequences > 0 {
            parts.push(format!("{} invalid UTF-8 sequences", self.invalid_sequences));
        }
        if self.null_bytes_removed > 0 {
            parts.push(format!("{} null bytes", self.null_bytes_removed));
        }
        if self.control_chars_removed > 0 {
            parts.push(format!("{} control chars", self.control_chars_removed));
        }
        if self.line_endings_normalized > 0 {
            parts.push(format!("{} line endings", self.line_endings_normalized));
        }
        if self.unicode_normalized {
            parts.push("Unicode NFC normalized".to_string());
        }

        if parts.is_empty() {
            "Input was clean".to_string()
        } else {
            format!("Sanitized: {}", parts.join(", "))
        }
    }
}

/// Sanitize raw bytes into safe UTF-8 string
///
/// This is the main entry point for all text input. It:
/// 1. Replaces invalid UTF-8 with � (U+FFFD REPLACEMENT CHARACTER)
/// 2. Removes null bytes (security risk)
/// 3. Normalizes line endings to \n
///
/// # Examples
/// ```
/// use core::logic::utf8::{sanitize_input, InputSource};
///
/// let raw = b"Hello \xF0\x28\x8C\x28 World"; // Invalid UTF-8
/// let safe = sanitize_input(raw, InputSource::Clipboard);
/// assert!(safe.contains('�')); // Replacement character
/// ```
pub fn sanitize_input(bytes: &[u8], source: InputSource) -> String {
    let (sanitized, _stats) = sanitize_input_with_stats(bytes, source);
    sanitized
}

/// Sanitize raw bytes and return statistics
///
/// Same as `sanitize_input()` but also returns detailed statistics
/// about what was sanitized.
///
/// # Examples
/// ```
/// use core::logic::utf8::{sanitize_input_with_stats, InputSource};
///
/// let raw = b"Hello \xF0\x28\x8C\x28 World";
/// let (safe, stats) = sanitize_input_with_stats(raw, InputSource::File);
/// assert!(stats.had_issues());
/// println!("{}", stats.summary());
/// ```
pub fn sanitize_input_with_stats(bytes: &[u8], _source: InputSource) -> (String, SanitizeStats) {
    let original_bytes = bytes.len();

    // Step 1: Convert to UTF-8, replacing invalid sequences
    let (utf8_str, invalid_sequences) = match std::str::from_utf8(bytes) {
        Ok(s) => (Cow::Borrowed(s), 0),
        Err(_) => {
            // Use String::from_utf8_lossy which replaces invalid sequences with �
            let lossy = String::from_utf8_lossy(bytes);
            let invalid_count = lossy.matches('�').count();
            (lossy, invalid_count)
        }
    };

    let was_valid = invalid_sequences == 0;

    // Step 2: Apply Unicode NFC normalization (canonical composition)
    // This ensures that canonically equivalent forms are unified:
    // - Precomposed vs decomposed characters (é vs e + ´)
    // - Multi-script text stability
    // - Deterministic parser results
    let normalized_unicode: String = utf8_str.nfc().collect();
    let unicode_normalized = normalized_unicode.len() != utf8_str.len() || 
                            normalized_unicode != utf8_str.as_ref();

    // Step 3: Remove null bytes (security risk)
    let (no_nulls, null_bytes_removed) = if normalized_unicode.contains('\0') {
        let filtered: String = normalized_unicode.chars().filter(|&c| c != '\0').collect();
        let removed = normalized_unicode.len() - filtered.len();
        (filtered, removed)
    } else {
        (normalized_unicode, 0)
    };

    // Step 4: Filter control characters (except \n, \r, \t)
    // This prevents rendering anomalies and potential injection exploits
    let original_len = no_nulls.len();
    let filtered: String = no_nulls
        .chars()
        .filter(|&c| !c.is_control() || matches!(c, '\n' | '\r' | '\t'))
        .collect();
    let control_chars_removed = original_len - filtered.len();

    // Step 5: Normalize line endings (\r\n → \n, \r → \n)
    let (normalized, line_endings_normalized) = normalize_line_endings(&filtered);

    let sanitized_bytes = normalized.len();

    let stats = SanitizeStats {
        original_bytes,
        sanitized_bytes,
        invalid_sequences,
        null_bytes_removed,
        control_chars_removed,
        line_endings_normalized,
        unicode_normalized,
        was_valid,
    };

    // Log if issues were found (in production, use proper logging)
    if stats.had_issues() {
        #[cfg(debug_assertions)]
        log::debug!("[UTF-8 Sanitizer] {}", stats.summary());
    }

    (normalized.into_owned(), stats)
}

/// Normalize line endings to Unix-style \n
///
/// Converts:
/// - \r\n (Windows) → \n
/// - \r (Old Mac) → \n
fn normalize_line_endings(s: &str) -> (Cow<'_, str>, usize) {
    if !s.contains('\r') {
        return (Cow::Borrowed(s), 0);
    }

    // Count \r occurrences before normalization
    let cr_count = s.matches('\r').count();
    
    let normalized = s.replace("\r\n", "\n").replace('\r', "\n");

    (Cow::Owned(normalized), cr_count)
}

/// Check if a byte index is on a UTF-8 character boundary
///
/// This is useful when you need to slice strings at calculated positions.
/// Always check before slicing!
///
/// # Examples
/// ```
/// let text = "Hello — World"; // Em dash is 3 bytes
/// assert!(is_char_boundary(text, 6)); // After "Hello "
/// assert!(!is_char_boundary(text, 9)); // Inside em dash (bytes 8-10)
/// assert!(is_char_boundary(text, 11)); // After em dash
/// ```
pub fn is_char_boundary(s: &str, index: usize) -> bool {
    s.is_char_boundary(index)
}

/// Find the previous valid char boundary from a given position
///
/// If `index` is already on a boundary, returns `index`.
/// Otherwise, returns the position of the previous character start.
///
/// # Examples
/// ```
/// let text = "Hello — World"; // Em dash is 3 bytes
/// assert_eq!(find_prev_boundary(text, 9), 8); // Inside dash → start of dash
/// assert_eq!(find_prev_boundary(text, 8), 8); // Already on boundary
/// ```
pub fn find_prev_boundary(s: &str, index: usize) -> usize {
    if index >= s.len() {
        return s.len();
    }

    let mut pos = index;
    while pos > 0 && !s.is_char_boundary(pos) {
        pos -= 1;
    }
    pos
}

/// Find the next valid char boundary from a given position
///
/// If `index` is already on a boundary, returns `index`.
/// Otherwise, returns the position of the next character start.
///
/// # Examples
/// ```
/// let text = "Hello — World"; // Em dash is 3 bytes
/// assert_eq!(find_next_boundary(text, 9), 11); // Inside dash → end of dash
/// assert_eq!(find_next_boundary(text, 8), 8); // Already on boundary
/// ```
pub fn find_next_boundary(s: &str, index: usize) -> usize {
    if index >= s.len() {
        return s.len();
    }

    let mut pos = index;
    while pos < s.len() && !s.is_char_boundary(pos) {
        pos += 1;
    }
    pos
}

/// Get the byte length of a character at a given position
///
/// Returns 0 if the position is not on a character boundary.
///
/// # Examples
/// ```
/// let text = "Hello — World";
/// assert_eq!(char_byte_length(text, 0), 1); // 'H' = 1 byte
/// assert_eq!(char_byte_length(text, 8), 3); // '—' = 3 bytes
/// ```
pub fn char_byte_length(s: &str, index: usize) -> usize {
    if !s.is_char_boundary(index) {
        return 0;
    }

    s[index..]
        .chars()
        .next()
        .map(|c| c.len_utf8())
        .unwrap_or(0)
}

/// Safe substring extraction by character count (not bytes!)
///
/// Unlike Rust's `&str[start..end]` which uses byte indices, this function
/// takes character positions and ensures slicing at valid boundaries.
///
/// # Examples
/// ```
/// let text = "Hello — World"; // Em dash is 3 bytes
/// assert_eq!(substring_by_chars(text, 0, 5), "Hello");
/// assert_eq!(substring_by_chars(text, 6, 7), "—"); // Single character
/// assert_eq!(substring_by_chars(text, 8, 13), "World");
/// ```
pub fn substring_by_chars(s: &str, char_start: usize, char_end: usize) -> &str {
    let byte_start = s
        .char_indices()
        .nth(char_start)
        .map(|(i, _)| i)
        .unwrap_or(s.len());

    let byte_end = s
        .char_indices()
        .nth(char_end)
        .map(|(i, _)| i)
        .unwrap_or(s.len());

    &s[byte_start..byte_end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_utf8() {
        let input = b"Hello, World!";
        let (result, stats) = sanitize_input_with_stats(input, InputSource::Keyboard);
        assert_eq!(result, "Hello, World!");
        assert!(!stats.had_issues());
        assert_eq!(stats.invalid_sequences, 0);
    }

    #[test]
    fn test_invalid_utf8_replaced() {
        // Invalid UTF-8 sequence
        let input = b"Hello \xF0\x28\x8C\x28 World";
        let (result, stats) = sanitize_input_with_stats(input, InputSource::Clipboard);
        assert!(result.contains('�'));
        assert!(stats.had_issues());
        assert!(stats.invalid_sequences > 0);
    }

    #[test]
    fn test_null_bytes_removed() {
        let input = b"Hello\x00World\x00";
        let (result, stats) = sanitize_input_with_stats(input, InputSource::File);
        assert_eq!(result, "HelloWorld");
        assert!(stats.had_issues());
        assert_eq!(stats.null_bytes_removed, 2);
    }

    #[test]
    fn test_line_ending_normalization_crlf() {
        let input = b"Line1\r\nLine2\r\nLine3";
        let (result, stats) = sanitize_input_with_stats(input, InputSource::File);
        assert_eq!(result, "Line1\nLine2\nLine3");
        assert!(stats.had_issues());
        assert!(stats.line_endings_normalized > 0);
    }

    #[test]
    fn test_line_ending_normalization_cr() {
        let input = b"Line1\rLine2\rLine3";
        let (result, stats) = sanitize_input_with_stats(input, InputSource::File);
        assert_eq!(result, "Line1\nLine2\nLine3");
        assert!(stats.had_issues());
    }

    #[test]
    fn test_em_dash_char_boundary() {
        let text = "Hello — World"; // Em dash (U+2014) is 3 bytes in UTF-8
        
        // Check boundaries around em dash
        assert!(is_char_boundary(text, 6)); // After "Hello "
        assert!(is_char_boundary(text, 9)); // After em dash (bytes 6-8)
        assert!(!is_char_boundary(text, 7)); // Inside em dash
        assert!(!is_char_boundary(text, 8)); // Inside em dash
    }

    #[test]
    fn test_find_boundaries() {
        let text = "Hello — World";
        
        // Find previous boundary from inside em dash
        assert_eq!(find_prev_boundary(text, 7), 6); // Inside → start
        assert_eq!(find_prev_boundary(text, 6), 6); // Already on boundary
        
        // Find next boundary from inside em dash
        assert_eq!(find_next_boundary(text, 7), 9); // Inside → end
        assert_eq!(find_next_boundary(text, 9), 9); // Already on boundary
    }

    #[test]
    fn test_char_byte_length() {
        let text = "Hello — World 😀"; // Em dash = 3 bytes, emoji = 4 bytes
        
        assert_eq!(char_byte_length(text, 0), 1); // 'H' = 1 byte
        assert_eq!(char_byte_length(text, 6), 3); // '—' = 3 bytes
        assert_eq!(char_byte_length(text, 16), 4); // '😀' = 4 bytes
    }

    #[test]
    fn test_substring_by_chars() {
        let text = "Hello — World"; // 13 characters, but more bytes
        
        assert_eq!(substring_by_chars(text, 0, 5), "Hello");
        assert_eq!(substring_by_chars(text, 6, 7), "—");
        assert_eq!(substring_by_chars(text, 8, 13), "World");
    }

    #[test]
    fn test_emoji_handling() {
        let input = "Hello 😀 World 🎉".as_bytes();
        let (result, stats) = sanitize_input_with_stats(input, InputSource::Keyboard);
        assert_eq!(result, "Hello 😀 World 🎉");
        assert!(!stats.had_issues());
    }

    #[test]
    fn test_cjk_characters() {
        let input = "こんにちは世界".as_bytes();
        let (result, stats) = sanitize_input_with_stats(input, InputSource::Keyboard);
        assert_eq!(result, "こんにちは世界");
        assert!(!stats.had_issues());
    }

    #[test]
    fn test_unicode_nfc_normalization_precomposed() {
        // Test that decomposed form is normalized to precomposed form
        // Decomposed: e (U+0065) + combining acute (U+0301) → Precomposed: é (U+00E9)
        let decomposed = "cafe\u{0301}"; // café with decomposed é
        let input = decomposed.as_bytes();
        let (result, stats) = sanitize_input_with_stats(input, InputSource::Keyboard);
        
        // Should be normalized to precomposed form
        assert_eq!(result, "café"); // café with precomposed é (U+00E9)
        assert!(stats.unicode_normalized);
    }

    #[test]
    fn test_unicode_nfc_already_normalized() {
        // Text already in NFC form should not be changed
        let input = "café".as_bytes(); // Already precomposed
        let (result, _stats) = sanitize_input_with_stats(input, InputSource::Keyboard);
        
        assert_eq!(result, "café");
        // Note: unicode_normalized may still be true if the check detects no difference
    }

    #[test]
    fn test_em_dash_preserved() {
        // Em dash (—, U+2014) should be preserved, not confused with hyphen (-, U+002D)
        let input = "Native performance — no login".as_bytes();
        let (result, _stats) = sanitize_input_with_stats(input, InputSource::Keyboard);
        
        assert_eq!(result, "Native performance — no login");
        assert!(result.contains('—')); // Em dash preserved
        
        // Check character codes - find the em dash
        let em_dash_char = result.chars().find(|&c| c == '—').unwrap();
        assert_eq!(em_dash_char as u32, 0x2014); // Verify it's the em dash
    }

    #[test]
    fn test_hyphen_vs_em_dash() {
        // Test that hyphens and em dashes are distinct after normalization
        let input = "hyphen - and em dash —".as_bytes();
        let (result, _stats) = sanitize_input_with_stats(input, InputSource::Keyboard);
        
        assert_eq!(result, "hyphen - and em dash —");
        
        // Count each type
        let hyphen_count = result.matches('-').count();
        let em_dash_count = result.matches('—').count();
        
        assert_eq!(hyphen_count, 1);
        assert_eq!(em_dash_count, 1);
    }

    #[test]
    fn test_control_characters_filtered() {
        // Control characters (except \n, \r, \t) should be removed
        let input = "Hello\x01\x02World\nNew\tLine\r\n".as_bytes();
        let (result, stats) = sanitize_input_with_stats(input, InputSource::Keyboard);
        
        // \x01 and \x02 should be removed, but \n, \t, \r should be preserved (then normalized)
        assert_eq!(result, "HelloWorld\nNew\tLine\n");
        assert!(stats.control_chars_removed > 0);
    }

    #[test]
    fn test_complex_markdown_with_em_dashes() {
        // Real-world test with markdown containing em dashes
        let input = "- **Bold** — description\n- *Italic* — another item".as_bytes();
        let (result, _stats) = sanitize_input_with_stats(input, InputSource::File);
        
        // Should preserve markdown structure and em dashes
        assert!(result.contains("**Bold** — description"));
        assert!(result.contains("*Italic* — another item"));
        assert_eq!(result.matches('—').count(), 2);
    }

    #[test]
    fn test_mixed_multibyte() {
        // Mix of 1, 2, 3, 4 byte UTF-8 characters
        let input = "ASCII Café 日本語 😀".as_bytes();
        let (result, stats) = sanitize_input_with_stats(input, InputSource::File);
        assert_eq!(result, "ASCII Café 日本語 😀");
        assert!(!stats.had_issues());
    }

    #[test]
    fn test_stats_summary() {
        let input = b"Hello\x00\xF0\x28World\r\n";
        let (_result, stats) = sanitize_input_with_stats(input, InputSource::Clipboard);
        
        assert!(stats.had_issues());
        let summary = stats.summary();
        assert!(summary.contains("Sanitized"));
    }
}
