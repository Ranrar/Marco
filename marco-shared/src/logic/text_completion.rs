//! Re-export emoji completion helpers from marco-core.
//!
//! The actual implementation lives in `marco_core::logic::text_completion`.
pub use marco_core::logic::text_completion::{
    emoji_completion_items, emoji_shortcode_matches_query, emoji_shortcodes_for_completion,
    normalize_completion_query, EmojiCompletionItem,
};
