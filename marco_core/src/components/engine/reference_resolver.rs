//! # Reference Resolution System
//!
//! Implements two-pass reference link/image resolution per CommonMark spec:
//! 1. **First pass**: Collect all `[label]: url "title"` definitions
//! 2. **Second pass**: Resolve `[text][label]` and `![alt][label]` references
//!
//! ## CommonMark Reference Matching Rules
//!
//! - Labels are **case-insensitive**: `[Foo]` matches `[foo]: url`
//! - Whitespace is collapsed: `[foo bar]` matches `[foo  bar]: url`
//! - First definition wins on duplicates: `[x]: a` then `[x]: b` → uses `a`
//! - Three reference types:
//!   - **Full**: `[text][label]` - explicit label
//!   - **Collapsed**: `[text][]` - uses text as label
//!   - **Shortcut**: `[text]` - uses text as label (if no inline link)
//!
//! ## Usage Example
//!
//! ```rust
//! use marco_core::components::engine::reference_resolver::ReferenceResolver;
//! use marco_core::components::engine::ast_node::Node;
//!
//! let mut resolver = ReferenceResolver::new();
//! 
//! // Collect definitions from AST
//! resolver.collect_definitions(&ast);
//!
//! // Resolve references in AST (mutates AST in-place)
//! resolver.resolve_references(&mut ast);
//! ```

use crate::components::engine::ast_node::Node;
use std::collections::HashMap;

/// Reference resolver for CommonMark reference links and images
#[derive(Debug, Clone)]
pub struct ReferenceResolver {
    /// Map of normalized label -> (url, optional title)
    /// 
    /// Labels are normalized by:
    /// - Converting to lowercase
    /// - Collapsing consecutive whitespace to single space
    /// - Trimming leading/trailing whitespace
    definitions: HashMap<String, (String, Option<String>)>,
}

impl ReferenceResolver {
    /// Create a new reference resolver with empty definitions
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
        }
    }

    /// Normalize a reference label per CommonMark spec
    ///
    /// Rules:
    /// - Convert to lowercase
    /// - Collapse consecutive whitespace (spaces, tabs, newlines) to single space
    /// - Trim leading and trailing whitespace
    ///
    /// # Examples
    ///
    /// ```
    /// use marco_core::components::engine::reference_resolver::ReferenceResolver;
    ///
    /// assert_eq!(
    ///     ReferenceResolver::normalize_label("Foo Bar"),
    ///     "foo bar"
    /// );
    /// assert_eq!(
    ///     ReferenceResolver::normalize_label("  Foo   Bar  "),
    ///     "foo bar"
    /// );
    /// assert_eq!(
    ///     ReferenceResolver::normalize_label("Foo\n\tBar"),
    ///     "foo bar"
    /// );
    /// ```
    pub fn normalize_label(label: &str) -> String {
        // Convert to lowercase
        let lower = label.to_lowercase();
        
        // Collapse whitespace and trim
        let mut result = String::new();
        let mut prev_was_space = true; // Start with true to trim leading spaces
        
        for ch in lower.chars() {
            if ch.is_whitespace() {
                if !prev_was_space {
                    result.push(' ');
                    prev_was_space = true;
                }
            } else {
                result.push(ch);
                prev_was_space = false;
            }
        }
        
        // Trim trailing space
        result.trim_end().to_string()
    }

    /// Collect all reference definitions from the AST
    ///
    /// Walks the AST recursively to find `ReferenceDefinition` nodes.
    /// For each definition:
    /// - Normalizes the label
    /// - Stores (url, title) in the definitions map
    /// - First definition wins on duplicate labels (per CommonMark)
    ///
    /// # Arguments
    ///
    /// * `ast` - The root AST node to collect definitions from
    pub fn collect_definitions(&mut self, ast: &Node) {
        self.collect_definitions_recursive(ast);
    }

    /// Recursive helper for collecting definitions
    fn collect_definitions_recursive(&mut self, node: &Node) {
        match node {
            Node::ReferenceDefinition { label, url, title, .. } => {
                let normalized_label = Self::normalize_label(label);
                
                // First definition wins - don't overwrite existing entries
                if !self.definitions.contains_key(&normalized_label) {
                    self.definitions.insert(
                        normalized_label,
                        (url.clone(), title.clone())
                    );
                }
            }
            
            // Recursively walk through container nodes
            Node::Document { children, .. } => {
                for child in children {
                    self.collect_definitions_recursive(child);
                }
            }
            Node::Heading { content, .. } => {
                for child in content {
                    self.collect_definitions_recursive(child);
                }
            }
            Node::Paragraph { content, .. } => {
                for child in content {
                    self.collect_definitions_recursive(child);
                }
            }
            Node::BlockQuote { content, .. } => {
                for child in content {
                    self.collect_definitions_recursive(child);
                }
            }
            Node::List { items, .. } => {
                for item in items {
                    self.collect_definitions_recursive(item);
                }
            }
            Node::ListItem { content, .. } => {
                for child in content {
                    self.collect_definitions_recursive(child);
                }
            }
            Node::Table { rows, .. } => {
                for row in rows {
                    for cell in row {
                        self.collect_definitions_recursive(cell);
                    }
                }
            }
            Node::TableCell { content, .. } => {
                for child in content {
                    self.collect_definitions_recursive(child);
                }
            }
            Node::Strong { content, .. } => {
                for child in content {
                    self.collect_definitions_recursive(child);
                }
            }
            Node::Emphasis { content, .. } => {
                for child in content {
                    self.collect_definitions_recursive(child);
                }
            }
            Node::Strikethrough { content, .. } => {
                for child in content {
                    self.collect_definitions_recursive(child);
                }
            }
            Node::Link { text, .. } => {
                for child in text {
                    self.collect_definitions_recursive(child);
                }
            }
            Node::ReferenceLink { text, .. } => {
                for child in text {
                    self.collect_definitions_recursive(child);
                }
            }
            Node::FootnoteDef { content, .. } => {
                for child in content {
                    self.collect_definitions_recursive(child);
                }
            }
            Node::InlineFootnoteRef { content, .. } => {
                for child in content {
                    self.collect_definitions_recursive(child);
                }
            }
            
            // Leaf nodes - no recursion needed
            Node::Text { .. }
            | Node::Code { .. }
            | Node::Image { .. }
            | Node::ReferenceImage { .. }
            | Node::LineBreak { .. }
            | Node::EscapedChar { .. }
            | Node::HorizontalRule { .. }
            | Node::CodeBlock { .. }
            | Node::HtmlBlock { .. }
            | Node::FootnoteRef { .. }
            | Node::Unknown { .. } => {
                // No children to recurse into
            }
        }
    }

    /// Resolve all reference links and images in the AST
    ///
    /// Walks the AST recursively to find `ReferenceLink` and `ReferenceImage` nodes.
    /// For each reference:
    /// - Normalizes the label
    /// - Looks up in definitions map
    /// - Replaces with resolved `Link` or `Image` node if found
    /// - Converts to literal text if undefined
    ///
    /// # Arguments
    ///
    /// * `ast` - The root AST node to resolve references in (mutated in-place)
    pub fn resolve_references(&self, ast: &mut Node) {
        self.resolve_references_recursive(ast);
    }

    /// Recursive helper for resolving references
    fn resolve_references_recursive(&self, node: &mut Node) {
        match node {
            Node::ReferenceLink { text, label, span } => {
                let normalized_label = Self::normalize_label(label);
                
                if let Some((url, title)) = self.definitions.get(&normalized_label) {
                    // Replace with resolved Link node
                    *node = Node::Link {
                        text: text.clone(),
                        url: url.clone(),
                        title: title.clone(),
                        span: span.clone(),
                    };
                } else {
                    // Undefined reference - convert to literal text
                    // Render as: [text][label]
                    let literal = if text.is_empty() {
                        format!("[{}]", label)
                    } else {
                        // Extract text content
                        let text_str = self.extract_text_content(text);
                        if text_str == *label {
                            format!("[{}]", label)
                        } else {
                            format!("[{}][{}]", text_str, label)
                        }
                    };
                    
                    *node = Node::Text {
                        content: literal,
                        span: span.clone(),
                    };
                }
            }
            
            Node::ReferenceImage { alt, label, span } => {
                let normalized_label = Self::normalize_label(label);
                
                if let Some((url, title)) = self.definitions.get(&normalized_label) {
                    // Replace with resolved Image node
                    *node = Node::Image {
                        alt: alt.clone(),
                        url: url.clone(),
                        title: title.clone(),
                        span: span.clone(),
                    };
                } else {
                    // Undefined reference - convert to literal text
                    // Render as: ![alt][label]
                    let literal = if alt.is_empty() {
                        format!("![{}]", label)
                    } else if alt == label {
                        format!("![{}]", alt)
                    } else {
                        format!("![{}][{}]", alt, label)
                    };
                    
                    *node = Node::Text {
                        content: literal,
                        span: span.clone(),
                    };
                }
            }
            
            // Recursively resolve in container nodes
            Node::Document { children, .. } => {
                for child in children {
                    self.resolve_references_recursive(child);
                }
            }
            Node::Heading { content, .. } => {
                for child in content {
                    self.resolve_references_recursive(child);
                }
            }
            Node::Paragraph { content, .. } => {
                for child in content {
                    self.resolve_references_recursive(child);
                }
            }
            Node::BlockQuote { content, .. } => {
                for child in content {
                    self.resolve_references_recursive(child);
                }
            }
            Node::List { items, .. } => {
                for item in items {
                    self.resolve_references_recursive(item);
                }
            }
            Node::ListItem { content, .. } => {
                for child in content {
                    self.resolve_references_recursive(child);
                }
            }
            Node::Table { rows, .. } => {
                for row in rows {
                    for cell in row {
                        self.resolve_references_recursive(cell);
                    }
                }
            }
            Node::TableCell { content, .. } => {
                for child in content {
                    self.resolve_references_recursive(child);
                }
            }
            Node::Strong { content, .. } => {
                for child in content {
                    self.resolve_references_recursive(child);
                }
            }
            Node::Emphasis { content, .. } => {
                for child in content {
                    self.resolve_references_recursive(child);
                }
            }
            Node::Strikethrough { content, .. } => {
                for child in content {
                    self.resolve_references_recursive(child);
                }
            }
            Node::Link { text, .. } => {
                for child in text {
                    self.resolve_references_recursive(child);
                }
            }
            Node::FootnoteDef { content, .. } => {
                for child in content {
                    self.resolve_references_recursive(child);
                }
            }
            Node::InlineFootnoteRef { content, .. } => {
                for child in content {
                    self.resolve_references_recursive(child);
                }
            }
            
            // Leaf nodes or already resolved - no action needed
            Node::Text { .. }
            | Node::Code { .. }
            | Node::Image { .. }
            | Node::LineBreak { .. }
            | Node::EscapedChar { .. }
            | Node::HorizontalRule { .. }
            | Node::CodeBlock { .. }
            | Node::HtmlBlock { .. }
            | Node::ReferenceDefinition { .. }
            | Node::FootnoteRef { .. }
            | Node::Unknown { .. } => {
                // No action needed
            }
        }
    }

    /// Extract plain text content from inline nodes (for literal text rendering)
    fn extract_text_content(&self, nodes: &[Node]) -> String {
        let mut result = String::new();
        for node in nodes {
            match node {
                Node::Text { content, .. } => result.push_str(content),
                Node::Code { content, .. } => result.push_str(content),
                Node::Strong { content, .. } => result.push_str(&self.extract_text_content(content)),
                Node::Emphasis { content, .. } => result.push_str(&self.extract_text_content(content)),
                Node::Strikethrough { content, .. } => result.push_str(&self.extract_text_content(content)),
                _ => {}
            }
        }
        result
    }

    /// Get the number of collected definitions (for testing/debugging)
    pub fn definition_count(&self) -> usize {
        self.definitions.len()
    }

    /// Check if a label has a definition (for testing/debugging)
    pub fn has_definition(&self, label: &str) -> bool {
        let normalized = Self::normalize_label(label);
        self.definitions.contains_key(&normalized)
    }

    /// Get a definition by label (for testing/debugging)
    pub fn get_definition(&self, label: &str) -> Option<&(String, Option<String>)> {
        let normalized = Self::normalize_label(label);
        self.definitions.get(&normalized)
    }
}

impl Default for ReferenceResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::engine::ast_node::Span;

    fn dummy_span() -> Span {
        Span { start: 0, end: 0, line: 0, column: 0 }
    }

    #[test]
    fn test_normalize_label() {
        assert_eq!(ReferenceResolver::normalize_label("foo"), "foo");
        assert_eq!(ReferenceResolver::normalize_label("Foo"), "foo");
        assert_eq!(ReferenceResolver::normalize_label("FOO"), "foo");
        assert_eq!(ReferenceResolver::normalize_label("Foo Bar"), "foo bar");
        assert_eq!(ReferenceResolver::normalize_label("  Foo   Bar  "), "foo bar");
        assert_eq!(ReferenceResolver::normalize_label("Foo\n\tBar"), "foo bar");
        assert_eq!(ReferenceResolver::normalize_label("Foo  \n  Bar"), "foo bar");
    }

    #[test]
    fn test_collect_single_definition() {
        let mut resolver = ReferenceResolver::new();
        
        let ast = Node::Document {
            children: vec![
                Node::ReferenceDefinition {
                    label: "example".to_string(),
                    url: "https://example.com".to_string(),
                    title: Some("Example Site".to_string()),
                    span: dummy_span(),
                }
            ],
            span: dummy_span(),
        };
        
        resolver.collect_definitions(&ast);
        
        assert_eq!(resolver.definition_count(), 1);
        assert!(resolver.has_definition("example"));
        assert!(resolver.has_definition("Example")); // Case-insensitive
        
        let def = resolver.get_definition("example").unwrap();
        assert_eq!(def.0, "https://example.com");
        assert_eq!(def.1, Some("Example Site".to_string()));
    }

    #[test]
    fn test_collect_multiple_definitions() {
        let mut resolver = ReferenceResolver::new();
        
        let ast = Node::Document {
            children: vec![
                Node::ReferenceDefinition {
                    label: "foo".to_string(),
                    url: "https://foo.com".to_string(),
                    title: None,
                    span: dummy_span(),
                },
                Node::ReferenceDefinition {
                    label: "bar".to_string(),
                    url: "https://bar.com".to_string(),
                    title: Some("Bar Site".to_string()),
                    span: dummy_span(),
                },
            ],
            span: dummy_span(),
        };
        
        resolver.collect_definitions(&ast);
        
        assert_eq!(resolver.definition_count(), 2);
        assert!(resolver.has_definition("foo"));
        assert!(resolver.has_definition("bar"));
    }

    #[test]
    fn test_first_definition_wins() {
        let mut resolver = ReferenceResolver::new();
        
        let ast = Node::Document {
            children: vec![
                Node::ReferenceDefinition {
                    label: "foo".to_string(),
                    url: "https://first.com".to_string(),
                    title: None,
                    span: dummy_span(),
                },
                Node::ReferenceDefinition {
                    label: "foo".to_string(),
                    url: "https://second.com".to_string(),
                    title: None,
                    span: dummy_span(),
                },
            ],
            span: dummy_span(),
        };
        
        resolver.collect_definitions(&ast);
        
        assert_eq!(resolver.definition_count(), 1);
        let def = resolver.get_definition("foo").unwrap();
        assert_eq!(def.0, "https://first.com"); // First wins
    }

    #[test]
    fn test_resolve_reference_link() {
        let mut resolver = ReferenceResolver::new();
        
        // Collect definition
        let def_ast = Node::Document {
            children: vec![
                Node::ReferenceDefinition {
                    label: "example".to_string(),
                    url: "https://example.com".to_string(),
                    title: Some("Example".to_string()),
                    span: dummy_span(),
                }
            ],
            span: dummy_span(),
        };
        resolver.collect_definitions(&def_ast);
        
        // Create reference link and resolve it
        let mut ast = Node::Document {
            children: vec![
                Node::Paragraph {
                    content: vec![
                        Node::ReferenceLink {
                            text: vec![Node::Text { content: "click here".to_string(), span: dummy_span() }],
                            label: "example".to_string(),
                            span: dummy_span(),
                        }
                    ],
                    indent_level: None,
                    span: dummy_span(),
                }
            ],
            span: dummy_span(),
        };
        
        resolver.resolve_references(&mut ast);
        
        // Verify it was resolved to a Link node
        if let Node::Document { children, .. } = &ast {
            if let Node::Paragraph { content, .. } = &children[0] {
                if let Node::Link { url, title, .. } = &content[0] {
                    assert_eq!(url, "https://example.com");
                    assert_eq!(title, &Some("Example".to_string()));
                } else {
                    panic!("Expected Link node, got: {:?}", content[0]);
                }
            }
        }
    }

    #[test]
    fn test_resolve_undefined_reference() {
        let resolver = ReferenceResolver::new(); // No definitions
        
        let mut ast = Node::Document {
            children: vec![
                Node::Paragraph {
                    content: vec![
                        Node::ReferenceLink {
                            text: vec![Node::Text { content: "text".to_string(), span: dummy_span() }],
                            label: "undefined".to_string(),
                            span: dummy_span(),
                        }
                    ],
                    indent_level: None,
                    span: dummy_span(),
                }
            ],
            span: dummy_span(),
        };
        
        resolver.resolve_references(&mut ast);
        
        // Verify it was converted to literal text
        if let Node::Document { children, .. } = &ast {
            if let Node::Paragraph { content, .. } = &children[0] {
                if let Node::Text { content, .. } = &content[0] {
                    assert!(content.contains("[text]"));
                    assert!(content.contains("[undefined]"));
                } else {
                    panic!("Expected Text node, got: {:?}", content[0]);
                }
            }
        }
    }

    #[test]
    fn test_resolve_reference_image() {
        let mut resolver = ReferenceResolver::new();
        
        // Collect definition
        let def_ast = Node::Document {
            children: vec![
                Node::ReferenceDefinition {
                    label: "logo".to_string(),
                    url: "/images/logo.png".to_string(),
                    title: Some("Company Logo".to_string()),
                    span: dummy_span(),
                }
            ],
            span: dummy_span(),
        };
        resolver.collect_definitions(&def_ast);
        
        // Create reference image and resolve it
        let mut ast = Node::Document {
            children: vec![
                Node::Paragraph {
                    content: vec![
                        Node::ReferenceImage {
                            alt: "Logo".to_string(),
                            label: "logo".to_string(),
                            span: dummy_span(),
                        }
                    ],
                    indent_level: None,
                    span: dummy_span(),
                }
            ],
            span: dummy_span(),
        };
        
        resolver.resolve_references(&mut ast);
        
        // Verify it was resolved to an Image node
        if let Node::Document { children, .. } = &ast {
            if let Node::Paragraph { content, .. } = &children[0] {
                if let Node::Image { url, title, alt, .. } = &content[0] {
                    assert_eq!(url, "/images/logo.png");
                    assert_eq!(title, &Some("Company Logo".to_string()));
                    assert_eq!(alt, "Logo");
                } else {
                    panic!("Expected Image node, got: {:?}", content[0]);
                }
            }
        }
    }

    #[test]
    fn test_case_insensitive_resolution() {
        let mut resolver = ReferenceResolver::new();
        
        let def_ast = Node::Document {
            children: vec![
                Node::ReferenceDefinition {
                    label: "FooBar".to_string(),
                    url: "https://example.com".to_string(),
                    title: None,
                    span: dummy_span(),
                }
            ],
            span: dummy_span(),
        };
        resolver.collect_definitions(&def_ast);
        
        let mut ast = Node::Document {
            children: vec![
                Node::Paragraph {
                    content: vec![
                        Node::ReferenceLink {
                            text: vec![Node::Text { content: "link".to_string(), span: dummy_span() }],
                            label: "foobar".to_string(), // Different case
                            span: dummy_span(),
                        }
                    ],
                    indent_level: None,
                    span: dummy_span(),
                }
            ],
            span: dummy_span(),
        };
        
        resolver.resolve_references(&mut ast);
        
        // Should resolve successfully despite case difference
        if let Node::Document { children, .. } = &ast {
            if let Node::Paragraph { content, .. } = &children[0] {
                if let Node::Link { url, .. } = &content[0] {
                    assert_eq!(url, "https://example.com");
                } else {
                    panic!("Expected Link node, got: {:?}", content[0]);
                }
            }
        }
    }
}
