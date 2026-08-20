//! Markdown completion helpers.

#[derive(Debug, Clone, PartialEq, Eq)]
/// A single completion candidate returned to editor integrations.
pub struct CompletionItem {
    /// User-visible completion label.
    pub label: String,
    /// Text inserted into the document when completion is accepted.
    pub insert_text: String,
    /// Optional short detail describing completion origin/type.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_get_markdown_completions_empty_query_returns_all() {
        let items = get_markdown_completions("");
        assert!(
            !items.is_empty(),
            "empty query should return all emoji completions"
        );
        // All items must have non-empty insert_text and label
        for item in &items {
            assert!(
                !item.insert_text.is_empty(),
                "insert_text must not be empty"
            );
            assert!(!item.label.is_empty(), "label must not be empty");
            assert_eq!(item.detail, "Emoji shortcode");
        }
    }

    #[test]
    fn smoke_test_get_markdown_completions_filters_by_prefix() {
        let smile_items = get_markdown_completions("smile");
        assert!(
            !smile_items.is_empty(),
            "query 'smile' should match at least one emoji shortcode"
        );
        // All returned items should have 'smile' in their shortcode
        for item in &smile_items {
            assert!(
                item.insert_text.contains("smile"),
                "shortcode '{}' should contain 'smile'",
                item.insert_text
            );
        }
    }

    #[test]
    fn smoke_test_get_markdown_completions_unknown_query_returns_empty() {
        let items = get_markdown_completions("xyzzy_no_such_emoji_exists_42");
        assert!(
            items.is_empty(),
            "nonsense query should return no completions"
        );
    }
}
