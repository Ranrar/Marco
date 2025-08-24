use crate::components::marco_engine::parser::{parse_markdown, Node};

pub struct MarkdownRenderer {
    // For now a simple implementation, can be extended to use the dynamic registry
}

impl MarkdownRenderer {
    /// Main function to convert markdown to HTML
    pub fn markdown_to_html(input: &str) -> Result<String, Box<dyn std::error::Error>> {
        let ast = parse_markdown(input)?;
        Ok(Self::render_node(&ast))
    }

    fn render_node(node: &Node) -> String {
        match node.node_type.as_str() {
            "root" => node
                .children
                .iter()
                .map(Self::render_node)
                .collect::<Vec<_>>()
                .join(""),

            "text" => {
                if let Some(value) = node.attributes.get("value") {
                    html_escape(value)
                } else {
                    String::new()
                }
            }

            "heading" => {
                let depth = node
                    .attributes
                    .get("depth")
                    .and_then(|d| d.parse::<u32>().ok())
                    .unwrap_or(1);
                let content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();
                format!("<h{}>{}</h{}>", depth, content, depth)
            }

            "paragraph" => {
                let content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();
                format!("<p>{}</p>", content)
            }

            "strong" => {
                // First try to get content from children (nested nodes)
                let children_content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();

                // If no children content, try to get from attributes (direct text)
                let content = if children_content.is_empty() {
                    node.attributes
                        .get("value")
                        .map_or(String::new(), |v| html_escape(v))
                } else {
                    children_content
                };

                format!("<strong>{}</strong>", content)
            }

            "emphasis" => {
                // First try to get content from children (nested nodes)
                let children_content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();

                // If no children content, try to get from attributes (direct text)
                let content = if children_content.is_empty() {
                    node.attributes
                        .get("value")
                        .map_or(String::new(), |v| html_escape(v))
                } else {
                    children_content
                };

                format!("<em>{}</em>", content)
            }

            "inlineCode" => {
                if let Some(value) = node.attributes.get("value") {
                    format!("<code>{}</code>", html_escape(value))
                } else {
                    String::new()
                }
            }

            "delete" => {
                // First try to get content from children (nested nodes)
                let children_content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();

                // If no children content, try to get from attributes (direct text)
                let content = if children_content.is_empty() {
                    node.attributes
                        .get("value")
                        .map_or(String::new(), |v| html_escape(v))
                } else {
                    children_content
                };

                format!("<del>{}</del>", content)
            }

            "blockquote" => {
                let content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();
                format!("<blockquote>{}</blockquote>", content)
            }

            "codeBlock" => {
                let code = node.attributes.get("value").map_or("", |v| v);
                let language = node.attributes.get("language");

                if let Some(lang) = language {
                    format!(
                        "<pre><code class=\"language-{}\">{}</code></pre>",
                        lang,
                        html_escape(code)
                    )
                } else {
                    format!("<pre><code>{}</code></pre>", html_escape(code))
                }
            }

            "thematicBreak" => "<hr>".to_string(),

            "list" => {
                let ordered = node
                    .attributes
                    .get("ordered")
                    .map(|o| o == "true")
                    .unwrap_or(false);
                let content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();

                if ordered {
                    format!("<ol>{}</ol>", content)
                } else {
                    format!("<ul>{}</ul>", content)
                }
            }

            "listItem" => {
                // Render children to find task markers or content
                let rendered_children = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<Vec<_>>();
                let mut content = rendered_children.join("");

                // If the parser set a 'checked' attribute, honor it first
                if let Some(checked_attr) = node.attributes.get("checked") {
                    let checked = checked_attr == "true";
                    if checked {
                        return format!(
                            "<li><input type=\"checkbox\" checked disabled> {}</li>",
                            content
                        );
                    } else {
                        return format!("<li><input type=\"checkbox\" disabled> {}</li>", content);
                    }
                }

                // Otherwise, detect leading task marker in the raw text (e.g., "[ ] " or "[x] ")
                // Look at the first child rendered string
                if let Some(first) = rendered_children.first() {
                    let raw = first.trim_start();
                    if raw.starts_with("[ ] ") || raw.starts_with("[ ]") {
                        // remove the marker from content
                        content = raw.replacen("[ ]", "", 1).trim_start().to_string()
                            + &rendered_children[1..].join("");
                        return format!("<li><input type=\"checkbox\" disabled> {}</li>", content);
                    } else if raw.to_lowercase().starts_with("[x] ")
                        || raw.to_lowercase().starts_with("[x]")
                    {
                        content = raw.replacen("[x]", "", 1).trim_start().to_string()
                            + &rendered_children[1..].join("");
                        return format!(
                            "<li><input type=\"checkbox\" checked disabled> {}</li>",
                            content
                        );
                    }
                }

                // Default list item rendering
                format!("<li>{}</li>", content)
            }

            "table" => {
                let content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();
                format!("<table>{}</table>", content)
            }

            "tableRow" => {
                let content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();
                format!("<tr>{}</tr>", content)
            }

            "tableCell" => {
                let content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();
                format!("<td>{}</td>", content)
            }

            "link" => {
                let url = node.attributes.get("url").map_or("#", |v| v);
                let content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();
                format!("<a href=\"{}\">{}</a>", html_escape(url), content)
            }

            "image" => {
                let url = node.attributes.get("url").map_or("", |v| v);
                let alt = node.attributes.get("alt").map_or("", |v| v);
                format!(
                    "<img src=\"{}\" alt=\"{}\">",
                    html_escape(url),
                    html_escape(alt)
                )
            }

            // Accept multiple naming variants for breaks (parser may produce different names)
            "hardBreak" | "hard_break" | "break" => "<br>".to_string(),

            // Soft break variants map to a single newline or space depending on rendering policy
            "softBreak" | "soft_break" => "\n".to_string(),

            // Accept HTML node variants produced by parser
            "html" | "htmlBlock" | "html_inline" | "htmlInline" | "htmlTag" | "html_tag" => {
                // Pass through HTML directly (be cautious about security in real applications)
                node.attributes.get("value").map_or("", |v| v).to_string()
            }

            // Frontmatter / yaml / toml nodes - skip in HTML output
            "yaml" | "frontmatter" | "yaml_frontmatter" | "toml_frontmatter" | "toml" => {
                String::new()
            }

            // Math node variants
            "inlineMath" | "mathInline" | "math_inline" => {
                if let Some(value) = node.attributes.get("value") {
                    format!(
                        "<span class=\"math-inline\">\\({}\\)</span>",
                        html_escape(value)
                    )
                } else {
                    String::new()
                }
            }

            "math" | "mathBlock" | "math_block" => {
                if let Some(value) = node.attributes.get("value") {
                    format!(
                        "<div class=\"math-block\">\\[{}\\]</div>",
                        html_escape(value)
                    )
                } else {
                    String::new()
                }
            }

            // Unknown node types - render children
            _ => node
                .children
                .iter()
                .map(Self::render_node)
                .collect::<String>(),
        }
    }
}

/// Simple HTML escape function
fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Compatibility function that matches comrak's signature  
pub fn markdown_to_html(input: &str, _options: &MarkdownOptions) -> String {
    MarkdownRenderer::markdown_to_html(input).unwrap_or_else(|err| {
        eprintln!("Markdown parsing error: {}", err);
        format!(
            "<p>Error parsing markdown: {}</p>",
            html_escape(&err.to_string())
        )
    })
}

/// Compatibility struct to replace ComrakOptions
#[derive(Default)]
pub struct MarkdownOptions {
    pub extension: MarkdownExtensions,
}

#[derive(Default)]
pub struct MarkdownExtensions {
    pub table: bool,
    pub autolink: bool,
    pub strikethrough: bool,
    pub tasklist: bool,
    pub footnotes: bool,
    pub tagfilter: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_markdown() {
        let input = "# Hello World\n\nThis is a **bold** text.\n";
        let output = MarkdownRenderer::markdown_to_html(input).unwrap();
        println!("Input: {:?}", input);
        println!("Output: {:?}", output);
        assert!(output.contains("<h1>Hello World</h1>"));
        assert!(output.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_code_block() {
        let input = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\n";
        let output = MarkdownRenderer::markdown_to_html(input).unwrap();
        assert!(output.contains("<pre><code class=\"language-rust\">"));
        assert!(output.contains("fn main()"));
    }

    #[test]
    fn test_list() {
        let input = "- Item 1\n- Item 2\n- [ ] Task\n- [x] Done\n";
        let output = MarkdownRenderer::markdown_to_html(input).unwrap();
        assert!(output.contains("<ul>"));
        assert!(output.contains("<li>"));
        assert!(output.contains("checkbox"));
    }
}
