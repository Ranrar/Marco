// HTML output generator with syntax highlighting for code blocks

use crate::parser::{Document, Node, NodeKind};
use super::RenderOptions;
use anyhow::Result;

// Render document to HTML
pub fn render_html(document: &Document, options: &RenderOptions) -> Result<String> {
    log::debug!("Rendering {} nodes to HTML", document.len());
    
    let mut html = String::new();
    
    for node in &document.children {
        render_node(node, &mut html, options)?;
    }
    
    Ok(html)
}

// Render individual node
fn render_node(node: &Node, output: &mut String, options: &RenderOptions) -> Result<()> {
    match &node.kind {
        NodeKind::Heading { level, text } => {
            log::trace!("Rendering heading level {}", level);
            let escaped_text = escape_html(text);
            output.push_str(&format!("<h{}>{}</h{}>\n", level, escaped_text, level));
        }
        NodeKind::Paragraph => {
            output.push_str("<p>");
            for child in &node.children {
                render_node(child, output, options)?;
            }
            output.push_str("</p>\n");
        }
        NodeKind::CodeBlock { language, code } => {
            log::trace!("Rendering code block: {:?}", language);
            output.push_str("<pre><code");
            
            // Add language class attribute if language specified
            if let Some(lang) = language {
                if !lang.is_empty() {
                    output.push_str(&format!(" class=\"language-{}\"", escape_html(lang)));
                }
            }
            
            output.push('>');
            output.push_str(&escape_html(code));
            output.push_str("</code></pre>\n");
        }
        NodeKind::Text(text) => {
            output.push_str(&escape_html(text));
        }
        NodeKind::CodeSpan(code) => {
            output.push_str("<code>");
            output.push_str(&escape_html(code));
            output.push_str("</code>");
        }
        NodeKind::Emphasis => {
            output.push_str("<em>");
            for child in &node.children {
                render_node(child, output, options)?;
            }
            output.push_str("</em>");
        }
        NodeKind::Strong => {
            output.push_str("<strong>");
            for child in &node.children {
                render_node(child, output, options)?;
            }
            output.push_str("</strong>");
        }
        NodeKind::Link { url, title } => {
            output.push_str("<a href=\"");
            output.push_str(&escape_html(url));
            output.push('"');
            if let Some(t) = title {
                output.push_str(" title=\"");
                output.push_str(&escape_html(t));
                output.push('"');
            }
            output.push('>');
            for child in &node.children {
                render_node(child, output, options)?;
            }
            output.push_str("</a>");
        }
        _ => {
            log::warn!("Unimplemented node type: {:?}", node.kind);
        }
    }
    
    Ok(())
}

// Escape HTML special characters to prevent XSS and ensure proper display
fn escape_html(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#39;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Document, Node, NodeKind};

    #[test]
    fn smoke_test_escape_html_basic() {
        let input = "Hello <world> & \"friends\"";
        let expected = "Hello &lt;world&gt; &amp; &quot;friends&quot;";
        assert_eq!(escape_html(input), expected);
    }

    #[test]
    fn smoke_test_escape_html_script_tag() {
        let input = "<script>alert('XSS')</script>";
        let expected = "&lt;script&gt;alert(&#39;XSS&#39;)&lt;/script&gt;";
        assert_eq!(escape_html(input), expected);
    }

    #[test]
    fn smoke_test_render_heading_h1() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Heading {
                    level: 1,
                    text: "Hello World".to_string(),
                },
                span: None,
                children: vec![],
            }],
        };
        let options = RenderOptions::default();
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(result, "<h1>Hello World</h1>\n");
    }

    #[test]
    fn smoke_test_render_heading_with_html() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Heading {
                    level: 2,
                    text: "Code <example> & test".to_string(),
                },
                span: None,
                children: vec![],
            }],
        };
        let options = RenderOptions::default();
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(result, "<h2>Code &lt;example&gt; &amp; test</h2>\n");
    }

    #[test]
    fn smoke_test_render_paragraph_with_text() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![Node {
                    kind: NodeKind::Text("This is a paragraph.".to_string()),
                    span: None,
                    children: vec![],
                }],
            }],
        };
        let options = RenderOptions::default();
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(result, "<p>This is a paragraph.</p>\n");
    }

    #[test]
    fn smoke_test_render_code_block_no_language() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::CodeBlock {
                    language: None,
                    code: "fn main() {\n    println!(\"Hello\");\n}".to_string(),
                },
                span: None,
                children: vec![],
            }],
        };
        let options = RenderOptions::default();
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(
            result,
            "<pre><code>fn main() {\n    println!(&quot;Hello&quot;);\n}</code></pre>\n"
        );
    }

    #[test]
    fn smoke_test_render_code_block_with_language() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::CodeBlock {
                    language: Some("rust".to_string()),
                    code: "let x = 42;".to_string(),
                },
                span: None,
                children: vec![],
            }],
        };
        let options = RenderOptions::default();
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(
            result,
            "<pre><code class=\"language-rust\">let x = 42;</code></pre>\n"
        );
    }

    #[test]
    fn smoke_test_render_code_block_escapes_html() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::CodeBlock {
                    language: Some("html".to_string()),
                    code: "<div>Test & verify</div>".to_string(),
                },
                span: None,
                children: vec![],
            }],
        };
        let options = RenderOptions::default();
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(
            result,
            "<pre><code class=\"language-html\">&lt;div&gt;Test &amp; verify&lt;/div&gt;</code></pre>\n"
        );
    }

    #[test]
    fn smoke_test_render_code_span() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![
                    Node {
                        kind: NodeKind::Text("Use ".to_string()),
                        span: None,
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::CodeSpan("println!()".to_string()),
                        span: None,
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::Text(" for output.".to_string()),
                        span: None,
                        children: vec![],
                    },
                ],
            }],
        };
        let options = RenderOptions::default();
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(result, "<p>Use <code>println!()</code> for output.</p>\n");
    }

    #[test]
    fn smoke_test_render_mixed_content() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Heading {
                        level: 1,
                        text: "Title".to_string(),
                    },
                    span: None,
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Paragraph,
                    span: None,
                    children: vec![Node {
                        kind: NodeKind::Text("Some text.".to_string()),
                        span: None,
                        children: vec![],
                    }],
                },
                Node {
                    kind: NodeKind::CodeBlock {
                        language: Some("python".to_string()),
                        code: "print('hello')".to_string(),
                    },
                    span: None,
                    children: vec![],
                },
            ],
        };
        let options = RenderOptions::default();
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(
            result,
            "<h1>Title</h1>\n<p>Some text.</p>\n<pre><code class=\"language-python\">print(&#39;hello&#39;)</code></pre>\n"
        );
    }
}
