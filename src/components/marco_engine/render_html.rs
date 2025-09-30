//! Simplified HTML Renderer
//!
//! Direct pattern matching on simplified Node enum without visitor patterns.
//! Follows the grammar-centered design from the documentation.

use crate::components::marco_engine::ast_node::Node;
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct HtmlOptions {
    pub syntax_highlighting: bool,
    pub css_classes: bool,
    pub inline_styles: bool,
    pub class_prefix: String,
    pub sanitize_html: bool,
    pub theme_mode: String, // "light" or "dark" for syntax highlighting themes
}

impl Default for HtmlOptions {
    fn default() -> Self {
        Self {
            syntax_highlighting: true,
            css_classes: true,
            inline_styles: false,
            class_prefix: "marco-".to_string(),
            sanitize_html: true,
            theme_mode: "light".to_string(), // Default to light theme
        }
    }
}

impl HtmlOptions {
    /// Create HtmlOptions with a specific theme mode
    pub fn with_theme_mode(theme_mode: &str) -> Self {
        Self {
            theme_mode: theme_mode.to_string(),
            ..Self::default()
        }
    }
}

pub struct HtmlRenderer {
    output: String,
    options: HtmlOptions,
}

impl HtmlRenderer {
    pub fn new(options: HtmlOptions) -> Self {
        Self {
            output: String::with_capacity(1024),
            options,
        }
    }

    /// Format a language identifier into a human-readable display name
    /// Examples: "rust" -> "Rust", "javascript" -> "JavaScript", "cpp" -> "C++"
    fn format_language_name(lang: &str) -> String {
        match lang.to_lowercase().as_str() {
            "rust" => "Rust".to_string(),
            "python" => "Python".to_string(),
            "javascript" | "js" => "JavaScript".to_string(),
            "typescript" | "ts" => "TypeScript".to_string(),
            "html" => "HTML".to_string(),
            "css" => "CSS".to_string(),
            "json" => "JSON".to_string(),
            "xml" => "XML".to_string(),
            "yaml" | "yml" => "YAML".to_string(),
            "toml" => "TOML".to_string(),
            "bash" | "sh" => "Bash".to_string(),
            "c" => "C".to_string(),
            "cpp" | "c++" | "cxx" => "C++".to_string(),
            "java" => "Java".to_string(),
            "go" => "Go".to_string(),
            "php" => "PHP".to_string(),
            "ruby" => "Ruby".to_string(),
            "sql" => "SQL".to_string(),
            "markdown" | "md" => "Markdown".to_string(),
            "shell" => "Shell".to_string(),
            "powershell" | "ps1" => "PowerShell".to_string(),
            "dockerfile" => "Dockerfile".to_string(),
            "makefile" => "Makefile".to_string(),
            "cmake" => "CMake".to_string(),
            // For unknown languages, capitalize first letter
            _ => {
                let mut chars = lang.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            }
        }
    }

    pub fn render(mut self, ast: &Node) -> String {
        self.render_node(ast);
        self.output
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
                    let formatted_name = Self::format_language_name(lang);
                    write!(
                        self.output,
                        " data-language=\"{}\"",
                        self.escape_html(&formatted_name)
                    )
                    .unwrap();
                }

                write!(self.output, "><code").unwrap();

                if let Some(lang) = language {
                    write!(
                        self.output,
                        " class=\"language-{}\"",
                        self.escape_html(lang)
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
                                write!(self.output, "{}", self.escape_html(content)).unwrap();
                            }
                        }
                    } else {
                        // No language specified, use plain text
                        write!(self.output, "{}", self.escape_html(content)).unwrap();
                    }
                } else {
                    // Syntax highlighting disabled, use plain text
                    write!(self.output, "{}", self.escape_html(content)).unwrap();
                }

                write!(self.output, "</code></pre>").unwrap();
            }

            Node::NestedCodeBlock {
                language,
                level,
                content,
                ..
            } => {
                // Render nested code block with Russian doll-style nesting
                write!(
                    self.output,
                    "<div class=\"nested-code-block level-{}\"",
                    level
                )
                .unwrap();

                if let Some(lang) = language {
                    let formatted_name = Self::format_language_name(lang);
                    write!(
                        self.output,
                        " data-language=\"{}\"",
                        self.escape_html(&formatted_name)
                    )
                    .unwrap();
                }

                write!(self.output, ">").unwrap();

                // Add header with language info
                let display_name = if let Some(lang) = language {
                    Self::format_language_name(lang)
                } else {
                    "Code".to_string()
                };
                write!(
                    self.output,
                    "<div class=\"code-header\">{}</div>",
                    self.escape_html(&display_name)
                )
                .unwrap();

                // Add content container
                write!(self.output, "<div class=\"code-content\">").unwrap();

                // Always render nested content consistently as regular markdown content
                for child in content {
                    self.render_node(child);
                }

                write!(self.output, "</div></div>").unwrap();
            }

            Node::MathBlock { content, .. } => {
                write!(
                    self.output,
                    "<div class=\"{}math-block\">",
                    self.options.class_prefix
                )
                .unwrap();
                write!(self.output, "$${}$$", self.escape_html(content)).unwrap();
                write!(self.output, "</div>").unwrap();
            }

            Node::List { ordered, items, .. } => {
                let tag = if *ordered { "ol" } else { "ul" };
                write!(self.output, "<{}>", tag).unwrap();
                for item in items {
                    self.render_node(item);
                }
                write!(self.output, "</{}>", tag).unwrap();
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

            Node::Table { headers, rows, .. } => {
                write!(
                    self.output,
                    "<table class=\"{}table\">",
                    self.options.class_prefix
                )
                .unwrap();

                // Header row
                if !headers.is_empty() {
                    write!(self.output, "<thead><tr>").unwrap();
                    for header in headers {
                        write!(self.output, "<th>").unwrap();
                        self.render_node(header);
                        write!(self.output, "</th>").unwrap();
                    }
                    write!(self.output, "</tr></thead>").unwrap();
                }

                // Data rows
                if !rows.is_empty() {
                    write!(self.output, "<tbody>").unwrap();
                    for row in rows {
                        write!(self.output, "<tr>").unwrap();
                        for cell in row {
                            write!(self.output, "<td>").unwrap();
                            self.render_node(cell);
                            write!(self.output, "</td>").unwrap();
                        }
                        write!(self.output, "</tr>").unwrap();
                    }
                    write!(self.output, "</tbody>").unwrap();
                }

                write!(self.output, "</table>").unwrap();
            }

            Node::TableCell {
                content, alignment, ..
            } => {
                // TableCell content is rendered by the table logic above
                // TODO: Use alignment for cell styling in future enhancement
                if let Some(_align) = alignment {
                    // Alignment will be used for CSS class generation
                }
                for child in content {
                    self.render_node(child);
                }
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

            Node::DefinitionList { items, .. } => {
                write!(self.output, "<dl>").unwrap();
                for item in items {
                    self.render_node(item);
                }
                write!(self.output, "</dl>").unwrap();
            }

            Node::DefinitionTerm { content, .. } => {
                write!(self.output, "<dt>").unwrap();
                for child in content {
                    self.render_node(child);
                }
                write!(self.output, "</dt>").unwrap();
            }

            Node::DefinitionDescription { content, .. } => {
                write!(self.output, "<dd>").unwrap();
                for child in content {
                    self.render_node(child);
                }
                write!(self.output, "</dd>").unwrap();
            }

            // Inline elements
            Node::Text { content, .. } => {
                write!(self.output, "{}", self.escape_html(content)).unwrap();
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

            Node::Strikethrough { content, .. } => {
                write!(self.output, "<del>").unwrap();
                for child in content {
                    self.render_node(child);
                }
                write!(self.output, "</del>").unwrap();
            }

            Node::Highlight { content, .. } => {
                write!(self.output, "<mark>").unwrap();
                for child in content {
                    self.render_node(child);
                }
                write!(self.output, "</mark>").unwrap();
            }

            Node::Superscript { content, .. } => {
                write!(self.output, "<sup>").unwrap();
                for child in content {
                    self.render_node(child);
                }
                write!(self.output, "</sup>").unwrap();
            }

            Node::Subscript { content, .. } => {
                write!(self.output, "<sub>").unwrap();
                for child in content {
                    self.render_node(child);
                }
                write!(self.output, "</sub>").unwrap();
            }

            Node::Code { content, .. } => {
                write!(self.output, "<code>").unwrap();
                write!(self.output, "{}", self.escape_html(content)).unwrap();
                write!(self.output, "</code>").unwrap();
            }

            Node::Emoji {
                unicode, shortcode, ..
            } => {
                // Render emoji with fallback to shortcode
                write!(
                    self.output,
                    "<span class=\"{}emoji\" title=\":{}\">{}\u{200B}</span>",
                    self.options.class_prefix,
                    self.escape_html(shortcode),
                    self.escape_html(unicode)
                )
                .unwrap();
            }

            Node::MathInline { content, .. } => {
                write!(
                    self.output,
                    "<span class=\"{}math-inline\">",
                    self.options.class_prefix
                )
                .unwrap();
                write!(self.output, "{}", self.escape_html(content)).unwrap();
                write!(self.output, "</span>").unwrap();
            }

            Node::Link {
                text, url, title, ..
            } => {
                write!(self.output, "<a href=\"{}\"", self.escape_html(url)).unwrap();
                if let Some(title_text) = title {
                    write!(self.output, " title=\"{}\"", self.escape_html(title_text)).unwrap();
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
                    self.escape_html(url),
                    self.escape_html(alt)
                )
                .unwrap();
                if let Some(title_text) = title {
                    write!(self.output, " title=\"{}\"", self.escape_html(title_text)).unwrap();
                }
                write!(self.output, ">").unwrap();
            }

            Node::LineBreak { break_type, .. } => {
                // Standard behavior: Hard breaks become <br>, soft breaks become space
                match break_type {
                    crate::components::marco_engine::ast_node::LineBreakType::Hard => {
                        write!(self.output, "<br>").unwrap();
                    }
                    crate::components::marco_engine::ast_node::LineBreakType::Soft => {
                        // Soft breaks are rendered as space
                        write!(self.output, " ").unwrap();
                    }
                }
            }

            Node::EscapedChar { character, .. } => {
                write!(self.output, "{}", self.escape_html(&character.to_string())).unwrap();
            }

            // Marco extensions
            Node::UserMention {
                username,
                platform,
                display_name,
                ..
            } => {
                write!(
                    self.output,
                    "<span class=\"{}user-mention\">",
                    self.options.class_prefix
                )
                .unwrap();
                write!(self.output, "@{}", self.escape_html(username)).unwrap();
                if let Some(platform_name) = platform {
                    write!(self.output, "[{}]", self.escape_html(platform_name)).unwrap();
                }
                if let Some(display) = display_name {
                    write!(self.output, " ({})", self.escape_html(display)).unwrap();
                }
                write!(self.output, "</span>").unwrap();
            }

            Node::Bookmark {
                label, path, line, ..
            } => {
                write!(self.output, "<a href=\"{}\"", self.escape_html(path)).unwrap();
                if let Some(line_num) = line {
                    write!(self.output, " data-line=\"{}\"", line_num).unwrap();
                }
                write!(
                    self.output,
                    " class=\"{}bookmark\">",
                    self.options.class_prefix
                )
                .unwrap();
                write!(self.output, "{}", self.escape_html(label)).unwrap();
                write!(self.output, "</a>").unwrap();
            }

            Node::TabBlock { title, tabs, .. } => {
                write!(
                    self.output,
                    "<div class=\"{}tab-block\">",
                    self.options.class_prefix
                )
                .unwrap();
                if let Some(title_text) = title {
                    write!(
                        self.output,
                        "<h3 class=\"{}tab-title\">{}</h3>",
                        self.options.class_prefix,
                        self.escape_html(title_text)
                    )
                    .unwrap();
                }
                write!(
                    self.output,
                    "<div class=\"{}tabs\">",
                    self.options.class_prefix
                )
                .unwrap();
                for tab in tabs {
                    self.render_node(tab);
                }
                write!(self.output, "</div></div>").unwrap();
            }

            Node::Tab { name, content, .. } => {
                write!(
                    self.output,
                    "<div class=\"{}tab\">",
                    self.options.class_prefix
                )
                .unwrap();
                if let Some(tab_name) = name {
                    write!(
                        self.output,
                        "<h4 class=\"{}tab-name\">{}</h4>",
                        self.options.class_prefix,
                        self.escape_html(tab_name)
                    )
                    .unwrap();
                }
                write!(
                    self.output,
                    "<div class=\"{}tab-content\">",
                    self.options.class_prefix
                )
                .unwrap();
                for child in content {
                    self.render_node(child);
                }
                write!(self.output, "</div></div>").unwrap();
            }

            Node::Admonition { kind, content, .. } => {
                write!(
                    self.output,
                    "<div class=\"{}admonition {}admonition-{}\">",
                    self.options.class_prefix, self.options.class_prefix, kind
                )
                .unwrap();
                write!(
                    self.output,
                    "<div class=\"{}admonition-title\">{}</div>",
                    self.options.class_prefix,
                    self.escape_html(kind)
                )
                .unwrap();
                write!(
                    self.output,
                    "<div class=\"{}admonition-content\">",
                    self.options.class_prefix
                )
                .unwrap();
                for child in content {
                    self.render_node(child);
                }
                write!(self.output, "</div></div>").unwrap();
            }

            Node::TableOfContents {
                depth, document, ..
            } => {
                write!(
                    self.output,
                    "<div class=\"{}toc\"",
                    self.options.class_prefix
                )
                .unwrap();
                if let Some(max_depth) = depth {
                    write!(self.output, " data-depth=\"{}\"", max_depth).unwrap();
                }
                if let Some(doc_ref) = document {
                    write!(
                        self.output,
                        " data-document=\"{}\"",
                        self.escape_html(doc_ref)
                    )
                    .unwrap();
                }
                write!(self.output, ">").unwrap();
                write!(
                    self.output,
                    "<!-- Table of Contents will be generated by JavaScript -->"
                )
                .unwrap();
                write!(self.output, "</div>").unwrap();
            }

            Node::RunInline {
                script_type,
                command,
                ..
            } => {
                write!(
                    self.output,
                    "<code class=\"{}run-inline {}run-{}\" data-script=\"{}\">",
                    self.options.class_prefix, self.options.class_prefix, script_type, script_type
                )
                .unwrap();
                write!(self.output, "{}", self.escape_html(command)).unwrap();
                write!(self.output, "</code>").unwrap();
            }

            Node::RunBlock {
                script_type,
                content,
                ..
            } => {
                write!(
                    self.output,
                    "<div class=\"{}run-block {}run-{}\">",
                    self.options.class_prefix, self.options.class_prefix, script_type
                )
                .unwrap();
                write!(self.output, "<pre><code data-script=\"{}\">", script_type).unwrap();
                write!(self.output, "{}", self.escape_html(content)).unwrap();
                write!(self.output, "</code></pre>").unwrap();
                write!(self.output, "</div>").unwrap();
            }

            Node::DiagramBlock {
                diagram_type,
                content,
                ..
            } => {
                write!(
                    self.output,
                    "<div class=\"{}diagram {}diagram-{}\" data-type=\"{}\">",
                    self.options.class_prefix,
                    self.options.class_prefix,
                    diagram_type,
                    diagram_type
                )
                .unwrap();
                write!(self.output, "<pre><code>").unwrap();
                write!(self.output, "{}", self.escape_html(content)).unwrap();
                write!(self.output, "</code></pre>").unwrap();
                write!(self.output, "</div>").unwrap();
            }

            // Footnotes and references
            Node::FootnoteDef { label, content, .. } => {
                write!(
                    self.output,
                    "<div class=\"{}footnote-def\" id=\"footnote-def-{}\">",
                    self.options.class_prefix,
                    self.escape_html(label)
                )
                .unwrap();
                write!(
                    self.output,
                    "<p><strong>[^{}]:</strong> ",
                    self.escape_html(label)
                )
                .unwrap();
                for child in content {
                    self.render_node(child);
                }
                write!(self.output, "</p></div>").unwrap();
            }

            Node::FootnoteRef { label, .. } => {
                write!(
                    self.output,
                    "<a href=\"#footnote-def-{}\" class=\"{}footnote-ref\" id=\"footnote-ref-{}\">[^{}]</a>",
                    self.escape_html(label),
                    self.options.class_prefix,
                    self.escape_html(label),
                    self.escape_html(label)
                )
                .unwrap();
            }

            Node::InlineFootnoteRef { content, .. } => {
                write!(
                    self.output,
                    "<span class=\"{}inline-footnote\">^[",
                    self.options.class_prefix
                )
                .unwrap();
                for child in content {
                    self.render_node(child);
                }
                write!(self.output, "]</span>").unwrap();
            }

            Node::ReferenceDefinition {
                label, url, title, ..
            } => {
                // Reference definitions are typically not rendered in HTML
                write!(
                    self.output,
                    "<!-- Reference definition: [{}]: {} {} -->",
                    self.escape_html(label),
                    self.escape_html(url),
                    title
                        .as_ref()
                        .map_or(String::new(), |t| format!("\"{}\"", self.escape_html(t)))
                )
                .unwrap();
            }

            Node::ReferenceLink { text, label, .. } => {
                // Note: In a full implementation, you'd resolve the reference
                write!(
                    self.output,
                    "<a href=\"#ref-{}\" class=\"{}reference-link\">",
                    self.escape_html(label),
                    self.options.class_prefix
                )
                .unwrap();
                for child in text {
                    self.render_node(child);
                }
                write!(self.output, "</a>").unwrap();
            }

            Node::ReferenceImage { alt, label, .. } => {
                // Note: In a full implementation, you'd resolve the reference
                write!(
                    self.output,
                    "<img src=\"#ref-{}\" alt=\"{}\" class=\"{}reference-image\">",
                    self.escape_html(label),
                    self.escape_html(alt),
                    self.options.class_prefix
                )
                .unwrap();
            }

            // HTML elements
            Node::HtmlBlock { content, .. } => {
                if self.options.sanitize_html && !self.is_safe_html(content) {
                    write!(
                        self.output,
                        "<pre><code>{}</code></pre>",
                        self.escape_html(content)
                    )
                    .unwrap();
                } else {
                    write!(self.output, "{}", content).unwrap();
                }
            }

            // Error recovery
            Node::Unknown { content, rule, .. } => {
                write!(
                    self.output,
                    "<div class=\"{}unknown\" data-rule=\"{}\">",
                    self.options.class_prefix,
                    self.escape_html(rule)
                )
                .unwrap();
                write!(self.output, "{}", self.escape_html(content)).unwrap();
                write!(self.output, "</div>").unwrap();
            }
        }
    }

    /// Check if HTML content contains only safe elements that should be allowed in GFM
    fn is_safe_html(&self, content: &str) -> bool {
        // List of safe HTML elements commonly used in GFM
        const SAFE_ELEMENTS: &[&str] = &[
            "p",
            "div",
            "span",
            "br",
            "hr",
            "img",
            "a",
            "strong",
            "em",
            "b",
            "i",
            "u",
            "s",
            "code",
            "pre",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "ul",
            "ol",
            "li",
            "dl",
            "dt",
            "dd",
            "table",
            "thead",
            "tbody",
            "tr",
            "th",
            "td",
            "blockquote",
            "center",
            "details",
            "summary",
            "mark",
            "del",
            "ins",
            "sub",
            "sup",
            "src",
            "alt",
            "title",
            "width",
            "height",
            "loading",
            "decoding",
        ];

        // Simple check: extract all element names and verify they're in the safe list
        let content_lower = content.to_lowercase();

        // Find all opening tags
        let mut pos = 0;
        while let Some(start) = content_lower[pos..].find('<') {
            let start = pos + start;
            if let Some(end) = content_lower[start..].find('>') {
                let end = start + end;
                let tag_content = &content_lower[start + 1..end];

                // Skip closing tags and self-closing tags
                if tag_content.starts_with('/') || tag_content.ends_with('/') {
                    pos = end + 1;
                    continue;
                }

                // Extract element name (before space or closing bracket)
                let element_name = tag_content.split_whitespace().next().unwrap_or("");

                // Check if this element is in our safe list
                if !element_name.is_empty() && !SAFE_ELEMENTS.contains(&element_name) {
                    return false;
                }

                pos = end + 1;
            } else {
                break;
            }
        }

        true
    }

    fn escape_html(&self, input: &str) -> String {
        if self.options.sanitize_html {
            input
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
                .replace('\'', "&#x27;")
                .replace('\t', "&nbsp;&nbsp;&nbsp;&nbsp;") // Convert tabs to 4 non-breaking spaces
        } else {
            input.replace('\t', "&nbsp;&nbsp;&nbsp;&nbsp;") // Convert tabs to 4 non-breaking spaces even without sanitization
        }
    }

    /// Render a standalone ListItem without <li> wrapper (for document-level tasks)
    fn render_standalone_list_item(&mut self, node: &Node) {
        if let Node::ListItem {
            content, checked, ..
        } = node
        {
            // Render checkbox if this is a task
            if let Some(is_checked) = checked {
                let checked_attr = if *is_checked { " checked" } else { "" };
                write!(
                    self.output,
                    "<input type=\"checkbox\"{} disabled> ",
                    checked_attr
                )
                .unwrap();
            }

            // Render content without <li> wrapper
            for child in content {
                self.render_node(child);
            }
        }
    }

    /// Try to apply syntax highlighting to code content
    /// Returns highlighted HTML on success, error on failure (will fallback to plain text)
    fn try_syntax_highlight(&self, code: &str, language: &str) -> Result<String, String> {
        use crate::components::viewer::syntax_highlighter::{global_syntax_highlighter, SYNTAX_HIGHLIGHTER};

        // Ensure global highlighter is initialized (this is fast after first call)
        global_syntax_highlighter()
            .map_err(|e| format!("Failed to initialize global syntax highlighter: {}", e))?;

        // Use theme mode from HtmlOptions (connected to Marco's light/dark theme switching)
        let theme_mode = &self.options.theme_mode;

        // Use the global thread-local instance to avoid repeated allocations
        SYNTAX_HIGHLIGHTER.with(|highlighter| {
            let highlighter_ref = highlighter.borrow();
            let syntax_highlighter = highlighter_ref.as_ref()
                .ok_or("Global syntax highlighter not initialized")?;
                
            syntax_highlighter.highlight_to_html(code, language, theme_mode)
                .map_err(|e| format!("Syntax highlighting failed: {}", e))
        })
    }
}
