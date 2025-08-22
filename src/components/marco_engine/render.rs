use crate::components::marco_engine::parser::{parse_markdown, Node};

pub struct MarkdownRenderer {
    // For now a simple implementation, can be extended to use the dynamic registry
}

impl MarkdownRenderer {
    pub fn new() -> Self {
        Self {}
    }

    /// Main function to convert markdown to HTML
    pub fn markdown_to_html(input: &str) -> Result<String, Box<dyn std::error::Error>> {
        let ast = parse_markdown(input)?;
        Ok(Self::render_node(&ast))
    }

    fn render_node(node: &Node) -> String {
        match node.node_type.as_str() {
            "root" => {
                node.children.iter().map(|child| Self::render_node(child)).collect::<Vec<_>>().join("")
            }
            
            "text" => {
                if let Some(value) = node.attributes.get("value") {
                    html_escape(value)
                } else {
                    String::new()
                }
            }
            
            "heading" => {
                let depth = node.attributes.get("depth").and_then(|d| d.parse::<u32>().ok()).unwrap_or(1);
                let content = node.children.iter().map(|child| Self::render_node(child)).collect::<String>();
                format!("<h{}>{}</h{}>", depth, content, depth)
            }
            
            "paragraph" => {
                let content = node.children.iter().map(|child| Self::render_node(child)).collect::<String>();
                format!("<p>{}</p>", content)
            }
            
            "strong" => {
                // First try to get content from children (nested nodes)
                let children_content = node.children.iter().map(|child| Self::render_node(child)).collect::<String>();
                
                // If no children content, try to get from attributes (direct text)  
                let content = if children_content.is_empty() {
                    node.attributes.get("value").map_or(String::new(), |v| html_escape(v))
                } else {
                    children_content
                };
                
                format!("<strong>{}</strong>", content)
            }
            
            "emphasis" => {
                // First try to get content from children (nested nodes)
                let children_content = node.children.iter().map(|child| Self::render_node(child)).collect::<String>();
                
                // If no children content, try to get from attributes (direct text)
                let content = if children_content.is_empty() {
                    node.attributes.get("value").map_or(String::new(), |v| html_escape(v))
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
                let children_content = node.children.iter().map(|child| Self::render_node(child)).collect::<String>();
                
                // If no children content, try to get from attributes (direct text)
                let content = if children_content.is_empty() {
                    node.attributes.get("value").map_or(String::new(), |v| html_escape(v))
                } else {
                    children_content
                };
                
                format!("<del>{}</del>", content)
            }
            
            "blockquote" => {
                let content = node.children.iter().map(|child| Self::render_node(child)).collect::<String>();
                format!("<blockquote>{}</blockquote>", content)
            }
            
            "codeBlock" => {
                let code = node.attributes.get("value").map_or("", |v| v);
                let language = node.attributes.get("language");
                
                if let Some(lang) = language {
                    format!("<pre><code class=\"language-{}\">{}</code></pre>", lang, html_escape(code))
                } else {
                    format!("<pre><code>{}</code></pre>", html_escape(code))
                }
            }
            
            "thematicBreak" => {
                "<hr>".to_string()
            }
            
            "list" => {
                let ordered = node.attributes.get("ordered").map(|o| o == "true").unwrap_or(false);
                let content = node.children.iter().map(|child| Self::render_node(child)).collect::<String>();
                
                if ordered {
                    format!("<ol>{}</ol>", content)
                } else {
                    format!("<ul>{}</ul>", content)
                }
            }
            
            "listItem" => {
                let content = node.children.iter().map(|child| Self::render_node(child)).collect::<String>();
                
                // Handle task list items
                if let Some(checked_attr) = node.attributes.get("checked") {
                    let checked = checked_attr == "true";
                    if checked {
                        format!("<li><input type=\"checkbox\" checked disabled> {}</li>", content)
                    } else {
                        format!("<li><input type=\"checkbox\" disabled> {}</li>", content)
                    }
                } else {
                    format!("<li>{}</li>", content)
                }
            }
            
            "table" => {
                let content = node.children.iter().map(|child| Self::render_node(child)).collect::<String>();
                format!("<table>{}</table>", content)
            }
            
            "tableRow" => {
                let content = node.children.iter().map(|child| Self::render_node(child)).collect::<String>();
                format!("<tr>{}</tr>", content)
            }
            
            "tableCell" => {
                let content = node.children.iter().map(|child| Self::render_node(child)).collect::<String>();
                format!("<td>{}</td>", content)
            }
            
            "link" => {
                let url = node.attributes.get("url").map_or("#", |v| v);
                let content = node.children.iter().map(|child| Self::render_node(child)).collect::<String>();
                format!("<a href=\"{}\">{}</a>", html_escape(url), content)
            }
            
            "image" => {
                let url = node.attributes.get("url").map_or("", |v| v);
                let alt = node.attributes.get("alt").map_or("", |v| v);
                format!("<img src=\"{}\" alt=\"{}\">", html_escape(url), html_escape(alt))
            }
            
            "hardBreak" => {
                "<br>".to_string()
            }
            
            "softBreak" => {
                "\n".to_string()
            }
            
            "html" => {
                // Pass through HTML directly (be cautious about security in real applications)
                node.attributes.get("value").map_or("", |v| v).to_string()
            }
            
            "yaml" => {
                // Skip frontmatter in HTML output
                String::new()
            }
            
            "inlineMath" => {
                if let Some(value) = node.attributes.get("value") {
                    format!("<span class=\"math-inline\">\\({}\\)</span>", html_escape(value))
                } else {
                    String::new()
                }
            }
            
            "math" => {
                if let Some(value) = node.attributes.get("value") {
                    format!("<div class=\"math-block\">\\[{}\\]</div>", html_escape(value))
                } else {
                    String::new()
                }
            }
            
            // Unknown node types - render children
            _ => {
                node.children.iter().map(|child| Self::render_node(child)).collect::<String>()
            }
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
        format!("<p>Error parsing markdown: {}</p>", html_escape(&err.to_string()))
    })
}

/// Compatibility struct to replace ComrakOptions
#[derive(Default)]
pub struct MarkdownOptions {
    pub extension: MarkdownExtensions,
    pub render: MarkdownRender,
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

#[derive(Default)]
pub struct MarkdownRender {
    pub unsafe_: bool,
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
