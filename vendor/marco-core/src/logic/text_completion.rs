//! Reusable text-completion helpers.
//!
//! This module currently exposes emoji shortcode completion data and matching
//! helpers for UI entry completion widgets.

use std::collections::BTreeMap;
use std::sync::OnceLock;

/// A single emoji completion candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmojiCompletionItem {
    /// Canonical shortcode value, always in `:shortcode:` form.
    pub shortcode: String,
    /// Unicode emoji glyph (e.g. `😄`).
    pub emoji: String,
    /// Display label for completion popup (e.g. `😄 smile`).
    pub display: String,
}

static EMOJI_COMPLETION_CACHE: OnceLock<Vec<EmojiCompletionItem>> = OnceLock::new();
static EMOJI_SHORTCODE_CACHE: OnceLock<Vec<String>> = OnceLock::new();

/// Normalize a completion query to be shortcode-friendly.
///
/// - Trims surrounding whitespace
/// - Trims optional `:` delimiters
/// - Lowercases for case-insensitive matching
pub fn normalize_completion_query(query: &str) -> String {
    query.trim().trim_matches(':').to_ascii_lowercase()
}

/// Returns all available emoji shortcodes as `:shortcode:` strings.
///
/// The list is computed once, sorted, and deduplicated.
pub fn emoji_shortcodes_for_completion() -> &'static [String] {
    EMOJI_SHORTCODE_CACHE
        .get_or_init(|| {
            emoji_completion_items()
                .iter()
                .map(|item| item.shortcode.clone())
                .collect::<Vec<_>>()
        })
        .as_slice()
}

/// Returns emoji completion items with both shortcode and visible emoji label.
///
/// The list is computed once, sorted, and deduplicated by shortcode.
pub fn emoji_completion_items() -> &'static [EmojiCompletionItem] {
    EMOJI_COMPLETION_CACHE
        .get_or_init(|| {
            let mut map: BTreeMap<String, String> = BTreeMap::new();

            for emoji in emojis::iter() {
                for shortcode in emoji.shortcodes() {
                    if shortcode.is_empty() {
                        continue;
                    }

                    let shortcode_value = format!(":{}:", shortcode);
                    map.entry(shortcode_value)
                        .or_insert_with(|| emoji.as_str().to_string());
                }
            }

            map.into_iter()
                .map(|(shortcode, emoji)| EmojiCompletionItem {
                    display: format!("{} {}", emoji, shortcode.trim_matches(':')),
                    shortcode,
                    emoji,
                })
                .collect::<Vec<_>>()
        })
        .as_slice()
}

/// Returns true if a shortcode candidate matches a user query.
///
/// Matching is case-insensitive and ignores optional surrounding `:` on both
/// candidate and query.
pub fn emoji_shortcode_matches_query(candidate: &str, query: &str) -> bool {
    let normalized_query = normalize_completion_query(query);
    if normalized_query.is_empty() {
        return true;
    }

    let normalized_candidate = normalize_completion_query(candidate);
    normalized_candidate.starts_with(&normalized_query)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_normalize_completion_query() {
        assert_eq!(normalize_completion_query(":Smile:"), "smile");
        assert_eq!(normalize_completion_query("  smile  "), "smile");
        assert_eq!(normalize_completion_query("::smile::"), "smile");
    }

    #[test]
    fn smoke_test_emoji_shortcodes_for_completion_contains_smile() {
        let shortcodes = emoji_shortcodes_for_completion();
        assert!(shortcodes.contains(&":smile:".to_string()));
    }

    #[test]
    fn smoke_test_emoji_completion_items_include_display_emoji() {
        let items = emoji_completion_items();
        let smile = items
            .iter()
            .find(|item| item.shortcode == ":smile:")
            .expect("expected :smile: completion item");

        assert!(!smile.emoji.is_empty());
        assert!(smile.display.contains(&smile.emoji));
        assert!(smile.display.contains("smile"));
        assert!(!smile.display.contains(":smile:"));
    }

    #[test]
    fn smoke_test_shortcode_match_accepts_colonless_query() {
        assert!(emoji_shortcode_matches_query(":smile:", "smi"));
        assert!(emoji_shortcode_matches_query(":smile:", ":smi"));
        assert!(emoji_shortcode_matches_query(":smile:", "SMI"));
        assert!(!emoji_shortcode_matches_query(":smile:", "joy"));
    }
}
