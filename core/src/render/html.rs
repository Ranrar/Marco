// HTML output generator with syntax highlighting for code blocks

use super::RenderOptions;
use crate::parser::{Document, Node, NodeKind};
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
        NodeKind::ThematicBreak => {
            output.push_str("<hr />\n");
        }
        NodeKind::HtmlBlock { html } => {
            // HTML blocks are rendered as-is without escaping
            // They already contain the complete HTML including tags
            output.push_str(html);
            if !html.ends_with('\n') {
                output.push('\n');
            }
        }
        NodeKind::Blockquote => {
            output.push_str("<blockquote>\n");
            for child in &node.children {
                render_node(child, output, options)?;
            }
            output.push_str("</blockquote>\n");
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
        NodeKind::StrongEmphasis => {
            // Triple delimiter: bold + italic.
            output.push_str("<strong><em>");
            for child in &node.children {
                render_node(child, output, options)?;
            }
            output.push_str("</em></strong>");
        }
        NodeKind::Strikethrough => {
            output.push_str("<del>");
            for child in &node.children {
                render_node(child, output, options)?;
            }
            output.push_str("</del>");
        }
        NodeKind::Mark => {
            output.push_str("<mark>");
            for child in &node.children {
                render_node(child, output, options)?;
            }
            output.push_str("</mark>");
        }
        NodeKind::Superscript => {
            output.push_str("<sup>");
            for child in &node.children {
                render_node(child, output, options)?;
            }
            output.push_str("</sup>");
        }
        NodeKind::Subscript => {
            output.push_str("<sub>");
            for child in &node.children {
                render_node(child, output, options)?;
            }
            output.push_str("</sub>");
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
        NodeKind::Image { url, alt } => {
            output.push_str("<img src=\"");
            output.push_str(&escape_html(url));
            output.push_str("\" alt=\"");
            output.push_str(&escape_html(alt));
            output.push_str("\" />");
        }
        NodeKind::InlineHtml(html) => {
            // Pass through inline HTML directly (no escaping)
            output.push_str(html);
        }
        NodeKind::HardBreak => {
            // Hard line break: <br />
            output.push_str("<br />\n");
        }
        NodeKind::SoftBreak => {
            // Soft line break: rendered as single space (or newline in some contexts)
            output.push('\n');
        }
        NodeKind::List {
            ordered,
            start,
            tight,
        } => {
            // Render list opening tag
            if *ordered {
                output.push_str("<ol");
                if let Some(num) = start {
                    if *num != 1 {
                        output.push_str(&format!(" start=\"{}\"", num));
                    }
                }
                output.push_str(">\n");
            } else {
                output.push_str("<ul>\n");
            }

            // Render list items
            for child in &node.children {
                render_list_item(child, output, *tight, options)?;
            }

            // Render list closing tag
            if *ordered {
                output.push_str("</ol>\n");
            } else {
                output.push_str("</ul>\n");
            }
        }
        NodeKind::ListItem => {
            // This should only be called via render_list_item
            log::warn!("ListItem rendered outside of List context");
            output.push_str("<li>");
            for child in &node.children {
                render_node(child, output, options)?;
            }
            output.push_str("</li>\n");
        }
        _ => {
            log::warn!("Unimplemented node type: {:?}", node.kind);
        }
    }

    Ok(())
}

// Render a list item with proper tight/loose handling
fn render_list_item(
    node: &Node,
    output: &mut String,
    tight: bool,
    options: &RenderOptions,
) -> Result<()> {
    output.push_str("<li>");

    if tight {
        // Tight list: don't wrap paragraphs in <p> tags
        for child in &node.children {
            match &child.kind {
                NodeKind::Paragraph => {
                    // Render paragraph children directly without <p> wrapper
                    for grandchild in &child.children {
                        render_node(grandchild, output, options)?;
                    }
                }
                _ => {
                    // Other block elements render normally
                    render_node(child, output, options)?;
                }
            }
        }
    } else {
        // Loose list: render everything normally (paragraphs get <p> tags)
        for child in &node.children {
            render_node(child, output, options)?;
        }
    }

    output.push_str("</li>\n");
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
        };
        let options = RenderOptions::default();
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(result, "<p>This is a paragraph.</p>\n");
    }

    #[test]
    fn smoke_test_render_code_block_without_language() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::CodeBlock {
                    language: None,
                    code: "fn main() {\n    println!(\"Hello\");\n}".to_string(),
                },
                span: None,
                children: vec![],
            }],
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
        };
        let options = RenderOptions::default();
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(result, "<p>Use <code>println!()</code> for output.</p>\n");
    }

    #[test]
    fn smoke_test_render_mixed_inlines() {
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
            ..Default::default()
        };
        let options = RenderOptions::default();
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(
            result,
            "<h1>Title</h1>\n<p>Some text.</p>\n<pre><code class=\"language-python\">print(&#39;hello&#39;)</code></pre>\n"
        );
    }

    #[test]
    fn smoke_test_render_strong_emphasis() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![Node {
                    kind: NodeKind::StrongEmphasis,
                    span: None,
                    children: vec![Node {
                        kind: NodeKind::Text("bold+italic".to_string()),
                        span: None,
                        children: vec![],
                    }],
                }],
            }],
            ..Default::default()
        };

        let options = RenderOptions::default();
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(result, "<p><strong><em>bold+italic</em></strong></p>\n");
    }

    #[test]
    fn smoke_test_render_strike_mark_sup_sub() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: None,
                children: vec![
                    Node {
                        kind: NodeKind::Strikethrough,
                        span: None,
                        children: vec![Node {
                            kind: NodeKind::Text("del".to_string()),
                            span: None,
                            children: vec![],
                        }],
                    },
                    Node {
                        kind: NodeKind::Text(" ".to_string()),
                        span: None,
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::Mark,
                        span: None,
                        children: vec![Node {
                            kind: NodeKind::Text("mark".to_string()),
                            span: None,
                            children: vec![],
                        }],
                    },
                    Node {
                        kind: NodeKind::Text(" ".to_string()),
                        span: None,
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::Superscript,
                        span: None,
                        children: vec![Node {
                            kind: NodeKind::Text("sup".to_string()),
                            span: None,
                            children: vec![],
                        }],
                    },
                    Node {
                        kind: NodeKind::Text(" ".to_string()),
                        span: None,
                        children: vec![],
                    },
                    Node {
                        kind: NodeKind::Subscript,
                        span: None,
                        children: vec![Node {
                            kind: NodeKind::Text("sub".to_string()),
                            span: None,
                            children: vec![],
                        }],
                    },
                ],
            }],
            ..Default::default()
        };

        let options = RenderOptions::default();
        let result = render_html(&doc, &options).unwrap();
        assert_eq!(
            result,
            "<p><del>del</del> <mark>mark</mark> <sup>sup</sup> <sub>sub</sub></p>\n"
        );
    }
}
