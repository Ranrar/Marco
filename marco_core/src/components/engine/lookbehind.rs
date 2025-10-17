//! Lookbehind/lookahead utilities for emphasis delimiter run validation
//!
//! CommonMark defines complex rules for when emphasis delimiters (* and _) can open or close
//! emphasis based on surrounding context. These rules involve checking characters before and
//! after delimiters, which Pest cannot easily express (no lookbehind support).
//!
//! This module implements the full CommonMark left-flanking and right-flanking delimiter run
//! rules in Rust, allowing post-parse validation of emphasis tokens.
//!
//! ## CommonMark Spec References
//!
//! **Left-flanking delimiter run**: A delimiter run is left-flanking if:
//! - (1) Not followed by Unicode whitespace, AND
//! - (2a) Not followed by a punctuation character, OR
//! - (2b) Followed by a punctuation character AND preceded by Unicode whitespace or punctuation
//!
//! **Right-flanking delimiter run**: A delimiter run is right-flanking if:
//! - (1) Not preceded by Unicode whitespace, AND
//! - (2a) Not preceded by a punctuation character, OR
//! - (2b) Preceded by a punctuation character AND followed by Unicode whitespace or punctuation
//!
//! See: https://spec.commonmark.org/0.31.2/#emphasis-and-strong-emphasis

/// Check if a character is a Unicode punctuation character per CommonMark spec
///
/// CommonMark defines punctuation as ASCII punctuation characters from the general
/// Unicode categories Pc, Pd, Pe, Pf, Pi, Po, or Ps.
///
/// For simplicity and performance, we use a subset that covers common cases.
#[inline]
pub fn is_punctuation(c: char) -> bool {
    matches!(c,
        '!' | '"' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | '-' | '.' | '/' |
        ':' | ';' | '<' | '=' | '>' | '?' | '@' | '[' | '\\' | ']' | '^' | '_' | '`' | '{' | '|' | '}' | '~'
    )
}

/// Check if a delimiter at the given position is left-flanking
///
/// A delimiter is left-flanking if it can potentially open emphasis.
///
/// # Arguments
/// * `text` - The full text containing the delimiter
/// * `delimiter_pos` - The byte position of the delimiter in the text
/// * `delimiter_len` - The length of the delimiter (1 for *, 2 for **)
///
/// # Returns
/// `true` if the delimiter is left-flanking and can open emphasis
///
/// # Examples
/// ```
/// use marco_core::components::engine::lookbehind::is_left_flanking;
///
/// // Valid left-flanking
/// assert!(is_left_flanking("*foo*", 0, 1));
/// assert!(is_left_flanking("foo-*(bar)*", 4, 1));
///
/// // Not left-flanking
/// assert!(!is_left_flanking("* foo*", 0, 1)); // followed by whitespace
/// assert!(!is_left_flanking("a*\"foo\"*", 1, 1)); // preceded by alphanum, followed by punct
/// ```
pub fn is_left_flanking(text: &str, delimiter_pos: usize, delimiter_len: usize) -> bool {
    // Get the character before the delimiter (or start-of-line)
    let prev_char = if delimiter_pos == 0 {
        None
    } else {
        text[..delimiter_pos].chars().rev().next()
    };

    // Get the character after the delimiter
    let next_pos = delimiter_pos + delimiter_len;
    let next_char = if next_pos >= text.len() {
        None
    } else {
        text[next_pos..].chars().next()
    };

    // (1) Not followed by Unicode whitespace
    let not_followed_by_whitespace = match next_char {
        Some(c) => !c.is_whitespace(),
        None => false, // End of text counts as whitespace per spec
    };

    if !not_followed_by_whitespace {
        return false;
    }

    // (2a) Not followed by a punctuation character
    let followed_by_punct = next_char.map_or(false, is_punctuation);
    if !followed_by_punct {
        return true;
    }

    // (2b) Followed by punctuation AND (preceded by whitespace OR punctuation)
    let preceded_by_whitespace_or_punct = match prev_char {
        Some(c) => c.is_whitespace() || is_punctuation(c),
        None => true, // Start of text counts as whitespace per spec
    };

    preceded_by_whitespace_or_punct
}

/// Check if a delimiter at the given position is right-flanking
///
/// A delimiter is right-flanking if it can potentially close emphasis.
///
/// # Arguments
/// * `text` - The full text containing the delimiter
/// * `delimiter_pos` - The byte position of the delimiter in the text
/// * `delimiter_len` - The length of the delimiter (1 for *, 2 for **)
///
/// # Returns
/// `true` if the delimiter is right-flanking and can close emphasis
///
/// # Examples
/// ```
/// use marco_core::components::engine::lookbehind::is_right_flanking;
///
/// // Valid right-flanking
/// assert!(is_right_flanking("*foo*", 4, 1));
/// assert!(is_right_flanking("*(bar)*", 6, 1));
///
/// // Not right-flanking
/// assert!(!is_right_flanking("*foo *", 5, 1)); // preceded by whitespace
/// assert!(!is_right_flanking("*(*foo)", 6, 1)); // preceded by punct, followed by alphanum
/// ```
pub fn is_right_flanking(text: &str, delimiter_pos: usize, delimiter_len: usize) -> bool {
    // Get the character before the delimiter
    let prev_char = if delimiter_pos == 0 {
        None
    } else {
        text[..delimiter_pos].chars().rev().next()
    };

    // Get the character after the delimiter (or end-of-line)
    let next_pos = delimiter_pos + delimiter_len;
    let next_char = if next_pos >= text.len() {
        None
    } else {
        text[next_pos..].chars().next()
    };

    // (1) Not preceded by Unicode whitespace
    let not_preceded_by_whitespace = match prev_char {
        Some(c) => !c.is_whitespace(),
        None => false, // Start of text counts as whitespace per spec
    };

    if !not_preceded_by_whitespace {
        return false;
    }

    // (2a) Not preceded by a punctuation character
    let preceded_by_punct = prev_char.map_or(false, is_punctuation);
    if !preceded_by_punct {
        return true;
    }

    // (2b) Preceded by punctuation AND (followed by whitespace OR punctuation)
    let followed_by_whitespace_or_punct = match next_char {
        Some(c) => c.is_whitespace() || is_punctuation(c),
        None => true, // End of text counts as whitespace per spec
    };

    followed_by_whitespace_or_punct
}

/// Check if an asterisk (*) delimiter can open emphasis at this position
///
/// Per CommonMark Rule 1: A single `*` character can open emphasis iff it is
/// part of a left-flanking delimiter run.
///
/// # Examples
/// ```
/// use marco_core::components::engine::lookbehind::can_asterisk_open_emphasis;
///
/// assert!(can_asterisk_open_emphasis("*foo*", 0));
/// assert!(can_asterisk_open_emphasis("foo*bar*", 3));
/// assert!(!can_asterisk_open_emphasis("* foo*", 0)); // whitespace after
/// ```
pub fn can_asterisk_open_emphasis(text: &str, pos: usize) -> bool {
    is_left_flanking(text, pos, 1)
}

/// Check if an asterisk (*) delimiter can close emphasis at this position
///
/// Per CommonMark Rule 2: A single `*` character can close emphasis iff it is
/// part of a right-flanking delimiter run.
///
/// # Examples
/// ```
/// use marco_core::components::engine::lookbehind::can_asterisk_close_emphasis;
///
/// assert!(can_asterisk_close_emphasis("*foo*", 4));
/// assert!(can_asterisk_close_emphasis("*foo*bar", 4));
/// assert!(!can_asterisk_close_emphasis("*foo *", 5)); // whitespace before
/// ```
pub fn can_asterisk_close_emphasis(text: &str, pos: usize) -> bool {
    is_right_flanking(text, pos, 1)
}

/// Check if an underscore (_) delimiter can open emphasis at this position
///
/// Per CommonMark Rule 3: A single `_` character can open emphasis iff it is
/// part of a left-flanking delimiter run AND either:
/// - (a) Not part of a right-flanking delimiter run, OR
/// - (b) Part of a right-flanking delimiter run preceded by punctuation
///
/// This prevents intraword emphasis with underscores (e.g., `foo_bar_`).
///
/// # Examples
/// ```
/// use marco_core::components::engine::lookbehind::can_underscore_open_emphasis;
///
/// assert!(can_underscore_open_emphasis("_foo_", 0));
/// assert!(can_underscore_open_emphasis("foo-_(bar)_", 4));
/// assert!(!can_underscore_open_emphasis("foo_bar_", 3)); // intraword
/// ```
pub fn can_underscore_open_emphasis(text: &str, pos: usize) -> bool {
    if !is_left_flanking(text, pos, 1) {
        return false;
    }

    // Additional check: if it's also right-flanking, must be preceded by punctuation
    if is_right_flanking(text, pos, 1) {
        // Check if preceded by punctuation
        let prev_char = if pos == 0 {
            return false;
        } else {
            text[..pos].chars().rev().next()
        };

        match prev_char {
            Some(c) => is_punctuation(c),
            None => false,
        }
    } else {
        true
    }
}

/// Check if an underscore (_) delimiter can close emphasis at this position
///
/// Per CommonMark Rule 4: A single `_` character can close emphasis iff it is
/// part of a right-flanking delimiter run AND either:
/// - (a) Not part of a left-flanking delimiter run, OR
/// - (b) Part of a left-flanking delimiter run followed by punctuation
///
/// This prevents intraword emphasis with underscores.
///
/// # Examples
/// ```
/// use marco_core::components::engine::lookbehind::can_underscore_close_emphasis;
///
/// assert!(can_underscore_close_emphasis("_foo_", 4));
/// assert!(can_underscore_close_emphasis("_(bar)_.", 6));
/// assert!(!can_underscore_close_emphasis("foo_bar_", 7)); // intraword
/// ```
pub fn can_underscore_close_emphasis(text: &str, pos: usize) -> bool {
    if !is_right_flanking(text, pos, 1) {
        return false;
    }

    // Additional check: if it's also left-flanking, must be followed by punctuation
    if is_left_flanking(text, pos, 1) {
        // Check if followed by punctuation
        let next_pos = pos + 1;
        let next_char = if next_pos >= text.len() {
            return false;
        } else {
            text[next_pos..].chars().next()
        };

        match next_char {
            Some(c) => is_punctuation(c),
            None => false,
        }
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Smoke Tests - Core Functionality
    // ============================================================================

    #[test]
    fn smoke_test_left_flanking_basic() {
        // Opening * followed by text
        assert!(is_left_flanking("*foo*", 0, 1));
        assert!(is_left_flanking("**foo**", 0, 2));
    }

    #[test]
    fn smoke_test_right_flanking_basic() {
        // Closing * preceded by text
        assert!(is_right_flanking("*foo*", 4, 1));
        assert!(is_right_flanking("**foo**", 5, 2));
    }

    #[test]
    fn smoke_test_not_left_flanking_whitespace() {
        // Delimiter followed by whitespace is not left-flanking
        assert!(!is_left_flanking("* foo*", 0, 1));
        assert!(!is_left_flanking("** foo**", 0, 2));
    }

    #[test]
    fn smoke_test_not_right_flanking_whitespace() {
        // Delimiter preceded by whitespace is not right-flanking
        assert!(!is_right_flanking("*foo *", 5, 1));
        assert!(!is_right_flanking("**foo **", 6, 2));
    }

    // ============================================================================
    // Asterisk Emphasis Tests (CommonMark Examples)
    // ============================================================================

    #[test]
    fn test_asterisk_intraword_allowed() {
        // Example 350: *foo*bar → <em>foo</em>bar
        let text = "*foo*bar";
        assert!(can_asterisk_open_emphasis(text, 0)); // Opening *
        assert!(can_asterisk_close_emphasis(text, 4)); // Closing *
    }

    #[test]
    fn test_asterisk_preceded_by_alphanum_followed_by_punct_fails() {
        // Example 360: a**"foo"** → a**"foo"** (NOT strong emphasis)
        // Similar logic applies to single asterisk: a*"foo"*
        let text = "a*\"foo\"*";
        assert!(!can_asterisk_open_emphasis(text, 1)); // Not left-flanking
    }

    #[test]
    fn test_asterisk_after_whitespace_fails() {
        // Example 346: *foo bar * → *foo bar * (NOT emphasis)
        let text = "a * foo bar*";
        assert!(!can_asterisk_open_emphasis(text, 2)); // Not left-flanking (preceded by space)
    }

    // ============================================================================
    // Underscore Emphasis Tests (CommonMark Examples)
    // ============================================================================

    #[test]
    fn test_underscore_intraword_blocked() {
        // Example 354: _foo_bar → _foo_bar (NOT emphasis)
        let text = "_foo_bar";
        assert!(can_underscore_open_emphasis(text, 0)); // Opens OK
        assert!(!can_underscore_close_emphasis(text, 4)); // Cannot close (intraword)
    }

    #[test]
    fn test_underscore_intraword_unicode_blocked() {
        // Example 355: _пристаням_стремятся → _пристаням_стремятся
        let text = "_пристаням_стремятся";
        assert!(can_underscore_open_emphasis(text, 0));
        let closing_pos = text.find('_').unwrap();
        let closing_pos2 = text[closing_pos + 1..].find('_').unwrap() + closing_pos + 1;
        assert!(!can_underscore_close_emphasis(text, closing_pos2));
    }

    #[test]
    fn test_underscore_after_punctuation_allowed() {
        // Example 344: foo-_(bar)_ → foo-<em>(bar)</em>
        let text = "foo-_(bar)_";
        assert!(can_underscore_open_emphasis(text, 4)); // After hyphen (punct)
        assert!(can_underscore_close_emphasis(text, 10)); // Before end
    }

    // ============================================================================
    // Punctuation Context Tests
    // ============================================================================

    #[test]
    fn test_is_punctuation_chars() {
        assert!(is_punctuation('!'));
        assert!(is_punctuation('.'));
        assert!(is_punctuation('-'));
        assert!(is_punctuation('*'));
        assert!(is_punctuation('_'));
        assert!(!is_punctuation('a'));
        assert!(!is_punctuation('5'));
        assert!(!is_punctuation(' '));
    }

    #[test]
    fn test_delimiter_before_punctuation() {
        // Delimiter followed by punctuation needs to be preceded by whitespace/punct
        let text = "a*."; // a followed by * followed by .
        // The * is not left-flanking because it's preceded by 'a' and followed by '.'
        assert!(!can_asterisk_open_emphasis(text, 1));
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    #[test]
    fn test_start_of_text() {
        // Start of text counts as whitespace for left-flanking
        let text = "*foo*";
        assert!(can_asterisk_open_emphasis(text, 0));
    }

    #[test]
    fn test_end_of_text() {
        // End of text counts as whitespace for right-flanking
        let text = "*foo*";
        assert!(can_asterisk_close_emphasis(text, 4));
    }

    #[test]
    fn test_adjacent_delimiters() {
        // **foo** - double asterisk
        let text = "**foo**";
        assert!(is_left_flanking(text, 0, 2));
        assert!(is_right_flanking(text, 5, 2));
    }
}
