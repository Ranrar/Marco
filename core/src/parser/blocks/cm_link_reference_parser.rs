//! Link reference definition parser - handles link reference storage
//!
//! Handles link reference definitions which don't produce visible AST nodes
//! but store references in the document for later use by link inline elements.

use crate::parser::ast::Document;

/// Parse a link reference definition and store it in the document.
///
/// Link reference definitions don't create AST nodes; instead, they store
/// reference data (label -> URL + optional title) in the document's reference map.
///
/// # Arguments
/// * `document` - The document to store the reference in
/// * `label` - The reference label (case-insensitive)
/// * `url` - The URL/destination
/// * `title` - Optional title text
///
/// # Example
/// ```ignore
/// let mut doc = Document::new();
/// parse_link_reference(&mut doc, "foo", "http://example.com", Some("Example"));
/// // Reference is now stored and can be resolved by [foo] links
/// ```
pub fn parse_link_reference(
    document: &mut Document,
    label: &str,
    url: String,
    title: Option<String>,
) {
    document.references.insert(label, url, title);
    log::debug!("Stored link reference definition: [{}]", label);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_link_reference_basic() {
        let mut doc = Document::new();
        parse_link_reference(&mut doc, "foo", "http://example.com".to_string(), None);

        let resolved = doc.references.get("foo");
        assert!(resolved.is_some());

        let (url, title) = resolved.unwrap();
        assert_eq!(url, "http://example.com");
        assert_eq!(title, &None);
    }

    #[test]
    fn smoke_test_parse_link_reference_with_title() {
        let mut doc = Document::new();
        parse_link_reference(
            &mut doc,
            "bar",
            "http://test.com".to_string(),
            Some("Test Site".to_string()),
        );

        let resolved = doc.references.get("bar");
        assert!(resolved.is_some());

        let (url, title) = resolved.unwrap();
        assert_eq!(url, "http://test.com");
        assert_eq!(title, &Some("Test Site".to_string()));
    }

    #[test]
    fn smoke_test_link_reference_case_insensitive() {
        let mut doc = Document::new();
        parse_link_reference(&mut doc, "FOO", "http://example.com".to_string(), None);

        // Should be retrievable with different case
        let resolved = doc.references.get("foo");
        assert!(resolved.is_some());

        let resolved2 = doc.references.get("FOO");
        assert!(resolved2.is_some());
    }

    #[test]
    fn smoke_test_multiple_references() {
        let mut doc = Document::new();
        parse_link_reference(&mut doc, "ref1", "http://one.com".to_string(), None);
        parse_link_reference(&mut doc, "ref2", "http://two.com".to_string(), None);
        parse_link_reference(&mut doc, "ref3", "http://three.com".to_string(), None);

        assert!(doc.references.get("ref1").is_some());
        assert!(doc.references.get("ref2").is_some());
        assert!(doc.references.get("ref3").is_some());
    }

    #[test]
    fn smoke_test_reference_overwrite() {
        let mut doc = Document::new();
        parse_link_reference(&mut doc, "foo", "http://old.com".to_string(), None);
        parse_link_reference(&mut doc, "foo", "http://new.com".to_string(), None);

        let resolved = doc.references.get("foo");
        let (url, _) = resolved.unwrap();

        // Last definition wins (HashMap insert behavior)
        assert_eq!(url, "http://new.com");
    }
}
