//! Block-level HTML rendering for CommonMark
//!
//! Handles rendering of block-level AST nodes (Document, Heading, Paragraph, etc.)
//! Marco extensions NOT included (tables, admonitions, tabs, run blocks, etc.)

use crate::components::engine::ast_node::Node;
use crate::components::engine::renderers::HtmlOptions; // Use HtmlOptions from mod.rs
use crate::components::syntax_highlighter::SyntaxHighlighter;
use std::collections::HashMap;
use std::fmt::Write;

use super::helpers;

// HtmlOptions moved to renderers/mod.rs to avoid duplication

pub struct BlockRenderer {
    output: String,
    options: HtmlOptions,
    /// Reference definitions: label -> (url, optional title)
    references: HashMap<String, (String, Option<String>)>,
}

impl BlockRenderer {
    pub fn new(options: HtmlOptions) -> Self {
        Self {
            output: String::with_capacity(1024),
            options,
            references: HashMap::new(),
        }
    }

    pub fn render(mut self, ast: &Node) -> String {
        // First pass: collect all reference definitions
        self.collect_references(ast);
        // Second pass: render the document
        self.render_node(ast);
        self.output
    }

    /// Collect all reference definitions from the AST
    /// References are case-insensitive according to CommonMark spec
    fn collect_references(&mut self, node: &Node) {
        match node {
            Node::Document { children, .. } => {
                for child in children {
                    self.collect_references(child);
                }
            }
            Node::ReferenceDefinition {
                label, url, title, ..
            } => {
                // Store reference with lowercase label for case-insensitive matching
                let key = label.to_lowercase();
                self.references
                    .insert(key, (url.clone(), title.clone()));
            }
            // Recursively check paragraphs and other containers
            Node::Paragraph { content, .. } => {
                for child in content {
                    self.collect_references(child);
                }
            }
            Node::BlockQuote { content, .. } => {
                for child in content {
                    self.collect_references(child);
                }
            }
            Node::List { items, .. } => {
                for item in items {
                    self.collect_references(item);
                }
            }
            Node::ListItem { content, .. } => {
                for child in content {
                    self.collect_references(child);
                }
            }
            _ => {
                // No nested children to check in other node types
            }
        }
    }

    fn render_node(&mut self, node: &Node) {
        match node {
            // Document structure
            Node::Document { children, .. } => {
                // Group consecutive standalone ListItems into proper list containers
                let mut i = 0;
                while i < children.len() {
                    if let Node::ListItem { .. } = &children[i] {
                        // Found a standalone ListItem - collect all consecutive ones
                        write!(self.output, "<ul>").unwrap();
                        while i < children.len() {
                            if let Node::ListItem { .. } = &children[i] {
                                // Render standalone ListItem WITHOUT <li> wrapper
                                self.render_standalone_list_item(&children[i]);
                                i += 1;
                            } else {
                                break;
                            }
                        }
                        write!(self.output, "</ul>").unwrap();
                    } else {
                        // Not a ListItem - render normally
                        self.render_node(&children[i]);
                        i += 1;
                    }
                }
            }

            // Block elements
            Node::Heading { level, content, .. } => {
                write!(self.output, "<h{}>", level).unwrap();
                for child in content {
                    self.render_node(child);
                }
                write!(self.output, "</h{}>", level).unwrap();
            }

            Node::Paragraph {
                content,
                indent_level,
                ..
            } => {
                write!(self.output, "<p").unwrap();

                // Add indentation class if present
                if let Some(indent) = indent_level {
                    if *indent > 0 {
                        write!(
                            self.output,
                            " class=\"{}indent-level-{}\"",
                            self.options.class_prefix, indent
                        )
                        .unwrap();
                    }
                }

                write!(self.output, ">").unwrap();
                for child in content {
                    self.render_node(child);
                }
                write!(self.output, "</p>").unwrap();
            }

            Node::CodeBlock {
                language,
                content,
                indent_level,
                ..
            } => {
                write!(self.output, "<pre").unwrap();

                // Add indentation class to the pre tag if present
                if let Some(indent) = indent_level {
                    if *indent > 0 {
                        write!(
                            self.output,
                            " class=\"{}indent-level-{}\"",
                            self.options.class_prefix, indent
                        )
                        .unwrap();
                    }
                }

                // Add data-language attribute with formatted language name
                if let Some(lang) = language {
                    let formatted_name = helpers::format_language_name(lang);
                    write!(
                        self.output,
                        " data-language=\"{}\"",
                        helpers::escape_html(&formatted_name)
                    )
                    .unwrap();
                }

                write!(self.output, "><code").unwrap();

                if let Some(lang) = language {
                    write!(
                        self.output,
                        " class=\"language-{}\"",
                        helpers::escape_html(lang)
                    )
                    .unwrap();
                }
                write!(self.output, ">").unwrap();

                // Apply syntax highlighting if enabled and language is specified
                if self.options.syntax_highlighting {
                    if let Some(lang) = language {
                        match self.try_syntax_highlight(content, lang) {
                            Ok(highlighted_html) => {
                                // Use highlighted HTML with CSS classes
                                write!(self.output, "{}", highlighted_html).unwrap();
                            }
                            Err(_) => {
                                // Fallback to escaped plain text
                                write!(self.output, "{}", helpers::escape_html(content)).unwrap();
                            }
                        }
                    } else {
                        // No language specified, use plain text
                        write!(self.output, "{}", helpers::escape_html(content)).unwrap();
                    }
                } else {
                    // Syntax highlighting disabled, use plain text
                    write!(self.output, "{}", helpers::escape_html(content)).unwrap();
                }

                write!(self.output, "</code></pre>").unwrap();
            }

            Node::List { ordered, items, is_tight, start_number, .. } => {
                let tag = if *ordered { "ol" } else { "ul" };
                
                // Open list tag with optional start attribute for ordered lists
                if *ordered {
                    if let Some(start) = start_number {
                        write!(self.output, r#"<{} start="{}">"#, tag, start).unwrap();
                    } else {
                        write!(self.output, "<{}>", tag).unwrap();
                    }
                } else {
                    write!(self.output, "<{}>", tag).unwrap();
                }
                
                // Add newline after opening tag (CommonMark format)
                write!(self.output, "\n").unwrap();
                
                // Render items
                for item in items {
                    // Pass tight/loose info to item rendering
                    if let Node::ListItem { content, checked, indent_level, is_loose, span } = item {
                        self.render_list_item(content, *checked, *indent_level, *is_loose);
                    } else {
                        self.render_node(item);
                    }
                }
                
                write!(self.output, "</{}>", tag).unwrap();
                write!(self.output, "\n").unwrap();
            }

            Node::ListItem {
                content,
                checked,
                indent_level,
                ..
            } => {
                write!(self.output, "<li").unwrap();

                // Build class string combining task item and indentation classes
                let mut classes = Vec::new();

                if let Some(is_checked) = checked {
                    if *is_checked {
                        classes.push(format!("{}task-item checked", self.options.class_prefix));
                    } else {
                        classes.push(format!("{}task-item", self.options.class_prefix));
                    }
                }

                if let Some(indent) = indent_level {
                    if *indent > 0 {
                        classes.push(format!(
                            "{}indent-level-{}",
                            self.options.class_prefix, indent
                        ));
                    }
                }

                if !classes.is_empty() {
                    write!(self.output, " class=\"{}\"", classes.join(" ")).unwrap();
                }

                write!(self.output, ">").unwrap();

                if let Some(is_checked) = checked {
                    let checked_attr = if *is_checked { " checked" } else { "" };
                    write!(
                        self.output,
                        "<input type=\"checkbox\"{} disabled> ",
                        checked_attr
                    )
                    .unwrap();
                }

                for child in content {
                    self.render_node(child);
                }
                write!(self.output, "</li>").unwrap();
            }

            Node::BlockQuote {
                content,
                indent_level,
                ..
            } => {
                write!(self.output, "<blockquote").unwrap();

                // Apply indentation class if present
                if let Some(indent) = indent_level {
                    if *indent > 0 {
                        write!(
                            self.output,
                            " class=\"{}indent-level-{}\"",
                            self.options.class_prefix, indent
                        )
                        .unwrap();
                    }
                }

                write!(self.output, ">").unwrap();

                // Handle blockquote content with proper line breaks between text nodes
                for (i, child) in content.iter().enumerate() {
                    if i > 0 {
                        // Add line break between consecutive text nodes or other inline content
                        if let (Some(Node::Text { .. }), Node::Text { .. }) =
                            (content.get(i - 1), child)
                        {
                            write!(self.output, "<br>").unwrap();
                        }
                    }
                    self.render_node(child);
                }

                write!(self.output, "</blockquote>").unwrap();
            }

            Node::HorizontalRule { .. } => {
                write!(self.output, "<hr>").unwrap();
            }

            // Inline elements (basic support for block content)
            Node::Text { content, .. } => {
                write!(self.output, "{}", helpers::escape_html(content)).unwrap();
            }

            Node::Strong { content, .. } => {
                write!(self.output, "<strong>").unwrap();
                for child in content {
                    self.render_node(child);
                }
                write!(self.output, "</strong>").unwrap();
            }

            Node::Emphasis { content, .. } => {
                write!(self.output, "<em>").unwrap();
                for child in content {
                    self.render_node(child);
                }
                write!(self.output, "</em>").unwrap();
            }

            Node::Code { content, .. } => {
                write!(self.output, "<code>").unwrap();
                write!(self.output, "{}", helpers::escape_html(content)).unwrap();
                write!(self.output, "</code>").unwrap();
            }

            Node::Link {
                text, url, title, ..
            } => {
                // Normalize www. URLs to http:// for proper handling
                let normalized_url = if url.to_lowercase().starts_with("www.") {
                    format!("http://{}", url)
                } else {
                    url.to_string()
                };

                write!(
                    self.output,
                    "<a href=\"{}\"",
                    helpers::escape_html(&normalized_url)
                )
                .unwrap();
                if let Some(title_text) = title {
                    write!(
                        self.output,
                        " title=\"{}\"",
                        helpers::escape_html(title_text)
                    )
                    .unwrap();
                }
                // Add target="_blank" for external links
                if helpers::is_external_url(&normalized_url) {
                    write!(self.output, " target=\"_blank\" rel=\"noopener noreferrer\"").unwrap();
                }
                write!(self.output, ">").unwrap();
                for child in text {
                    self.render_node(child);
                }
                write!(self.output, "</a>").unwrap();
            }

            Node::Image {
                alt, url, title, ..
            } => {
                write!(
                    self.output,
                    "<img src=\"{}\" alt=\"{}\"",
                    helpers::escape_html(url),
                    helpers::escape_html(alt)
                )
                .unwrap();
                if let Some(title_text) = title {
                    write!(
                        self.output,
                        " title=\"{}\"",
                        helpers::escape_html(title_text)
                    )
                    .unwrap();
                }
                write!(self.output, ">").unwrap();
            }

            Node::LineBreak { break_type, .. } => {
                // Hard breaks become <br>, soft breaks become space
                match break_type {
                    crate::components::engine::ast_node::LineBreakType::Hard => {
                        write!(self.output, "<br>").unwrap();
                    }
                    crate::components::engine::ast_node::LineBreakType::Soft => {
                        write!(self.output, " ").unwrap();
                    }
                }
            }

            Node::EscapedChar { character, .. } => {
                write!(
                    self.output,
                    "{}",
                    helpers::escape_html(&character.to_string())
                )
                .unwrap();
            }

            Node::ReferenceDefinition { .. } => {
                // Reference definitions don't render anything
            }

            Node::ReferenceLink { label, text, .. } => {
                // Resolve reference link
                let key = label.to_lowercase();
                if let Some((url, title)) = self.references.get(&key) {
                    write!(self.output, "<a href=\"{}\"", helpers::escape_html(url)).unwrap();
                    if let Some(title_text) = title {
                        write!(
                            self.output,
                            " title=\"{}\"",
                            helpers::escape_html(title_text)
                        )
                        .unwrap();
                    }
                    write!(self.output, ">").unwrap();
                    for child in text {
                        self.render_node(child);
                    }
                    write!(self.output, "</a>").unwrap();
                } else {
                    // Unresolved reference - render as plain text
                    write!(self.output, "[").unwrap();
                    for child in text {
                        self.render_node(child);
                    }
                    write!(self.output, "][{}]", helpers::escape_html(label)).unwrap();
                }
            }

            Node::ReferenceImage { label, alt, .. } => {
                // Resolve reference image
                let key = label.to_lowercase();
                if let Some((url, title)) = self.references.get(&key) {
                    write!(
                        self.output,
                        "<img src=\"{}\" alt=\"{}\"",
                        helpers::escape_html(url),
                        helpers::escape_html(alt)
                    )
                    .unwrap();
                    if let Some(title_text) = title {
                        write!(
                            self.output,
                            " title=\"{}\"",
                            helpers::escape_html(title_text)
                        )
                        .unwrap();
                    }
                    write!(self.output, ">").unwrap();
                } else {
                    // Unresolved reference - render as plain text
                    write!(
                        self.output,
                        "![{}][{}]",
                        helpers::escape_html(alt),
                        helpers::escape_html(label)
                    )
                    .unwrap();
                }
            }

            Node::HtmlBlock { content, .. } => {
                // Raw HTML block - output as-is without escaping
                // Content is already HTML, so no need to escape
                write!(self.output, "{}", content).unwrap();
                // Add newline for proper block separation
                writeln!(self.output).unwrap();
            }

            // Marco extensions are NOT supported - ignore them
            _ => {
                // Ignore unsupported Marco extension nodes
            }
        }
    }

    /// Render a list item with tight/loose formatting
    fn render_list_item(
        &mut self,
        content: &[Node],
        checked: Option<bool>,
        indent_level: Option<u8>,
        is_loose: bool, // true if item should be wrapped in <p> tags
    ) {
        write!(self.output, "<li").unwrap();

        // Build class string combining task item and indentation classes
        let mut classes = Vec::new();

        if let Some(is_checked) = checked {
            if is_checked {
                classes.push(format!("{}task-item checked", self.options.class_prefix));
            } else {
                classes.push(format!("{}task-item", self.options.class_prefix));
            }
        }

        if let Some(indent) = indent_level {
            if indent > 0 {
                classes.push(format!(
                    "{}indent-level-{}",
                    self.options.class_prefix, indent
                ));
            }
        }

        if !classes.is_empty() {
            write!(self.output, r#" class="{}""#, classes.join(" ")).unwrap();
        }

        write!(self.output, ">").unwrap();

        // For loose lists, wrap content in <p> tags
        if is_loose {
            write!(self.output, "\n<p>").unwrap();
        }

        // Render content - separate inline content from nested lists
        let mut has_nested_list = false;
        for child in content {
            if matches!(child, Node::List { .. }) {
                has_nested_list = true;
                // Add newline before nested list
                write!(self.output, "\n").unwrap();
                self.render_node(child);
            } else {
                self.render_node(child);
            }
        }

        // Close <p> tag for loose lists
        if is_loose {
            write!(self.output, "</p>\n").unwrap();
        } else if has_nested_list {
            // Add newline before closing tag if there's a nested list
            // (even in tight lists)
            // Don't add extra newline - nested list already added one
        }

        write!(self.output, "</li>\n").unwrap();
    }

    fn render_standalone_list_item(&mut self, node: &Node) {
        if let Node::ListItem {
            content, checked, ..
        } = node
        {
            write!(self.output, "<li").unwrap();

            if let Some(is_checked) = checked {
                if *is_checked {
                    write!(
                        self.output,
                        " class=\"{}task-item checked\"",
                        self.options.class_prefix
                    )
                    .unwrap();
                } else {
                    write!(
                        self.output,
                        " class=\"{}task-item\"",
                        self.options.class_prefix
                    )
                    .unwrap();
                }
            }

            write!(self.output, ">").unwrap();

            if let Some(is_checked) = checked {
                let checked_attr = if *is_checked { " checked" } else { "" };
                write!(
                    self.output,
                    "<input type=\"checkbox\"{} disabled> ",
                    checked_attr
                )
                .unwrap();
            }

            for child in content {
                self.render_node(child);
            }
            write!(self.output, "</li>").unwrap();
        }
    }

    fn try_syntax_highlight(&self, code: &str, language: &str) -> Result<String, String> {
        // Use SyntaxHighlighter from marco_core
        SyntaxHighlighter::new()
            .map_err(|e| format!("Highlighter init failed: {}", e))?
            .highlight_to_html(code, language, &self.options.theme_mode)
            .map_err(|e| format!("Highlighting failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::engine::ast_node::Span;

    fn empty_span() -> Span {
        Span::new(0, 0, 0, 0)
    }

    #[test]
    fn smoke_test_heading_rendering() {
        let ast = Node::heading(
            1,
            vec![Node::text("Test Heading".to_string(), empty_span())],
            empty_span(),
        );

        let renderer = BlockRenderer::new(HtmlOptions::default());
        let html = renderer.render(&ast);

        assert!(html.contains("<h1>"));
        assert!(html.contains("Test Heading"));
        assert!(html.contains("</h1>"));
    }

    #[test]
    fn smoke_test_paragraph_rendering() {
        let ast = Node::paragraph(
            vec![Node::text("Test paragraph".to_string(), empty_span())],
            None,
            empty_span(),
        );

        let renderer = BlockRenderer::new(HtmlOptions::default());
        let html = renderer.render(&ast);

        assert!(html.contains("<p>"));
        assert!(html.contains("Test paragraph"));
        assert!(html.contains("</p>"));
    }

    #[test]
    fn smoke_test_code_block_rendering() {
        let ast = Node::code_block(
            Some("rust".to_string()),
            "fn main() {}".to_string(),
            None,
            empty_span(),
        );

        let renderer = BlockRenderer::new(HtmlOptions::default());
        let html = renderer.render(&ast);

        assert!(html.contains("<pre"));
        assert!(html.contains("<code"));
        assert!(html.contains("class=\"language-rust\""));
        // Code content might be syntax highlighted or escaped
        assert!(html.contains("main") && html.contains("fn"));
        assert!(html.contains("</code></pre>"));
    }

    #[test]
    fn smoke_test_list_rendering() {
        let list_items = vec![
            Node::list_item(
                vec![Node::text("Item 1".to_string(), empty_span())],
                None,
                None,
                false, // tight item
                empty_span(),
            ),
            Node::list_item(
                vec![Node::text("Item 2".to_string(), empty_span())],
                None,
                None,
                false, // tight item
                empty_span(),
            ),
        ];

        // Create a tight unordered list with no start number
        let ast = Node::list(false, list_items, true, None, empty_span());

        let renderer = BlockRenderer::new(HtmlOptions::default());
        let html = renderer.render(&ast);

        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>"));
        assert!(html.contains("Item 1"));
        assert!(html.contains("Item 2"));
        assert!(html.contains("</ul>"));
    }
}

