//! Markdown completion helpers.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionItem {
    pub label: String,
    pub insert_text: String,
    pub detail: String,
}

/// Return markdown completions for the given query.
///
/// Current implementation provides emoji shortcode completion and is designed
/// to be extended with markdown-structural completions.
pub fn get_markdown_completions(query: &str) -> Vec<CompletionItem> {
    crate::logic::text_completion::emoji_completion_items()
        .iter()
        .filter(|item| {
            crate::logic::text_completion::emoji_shortcode_matches_query(&item.shortcode, query)
        })
        .map(|item| CompletionItem {
            label: item.display.clone(),
            insert_text: item.shortcode.clone(),
            detail: "Emoji shortcode".to_string(),
        })
        .collect()
}
