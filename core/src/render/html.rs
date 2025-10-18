// HTML output generator with syntax highlighting for code blocks

use crate::ast::{Document, Node, NodeKind};
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
            output.push_str(&format!("<h{}>{}</h{}>\n", level, text, level));
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
            // TODO: Add syntax highlighting using syntect
            output.push_str("<pre><code>");
            output.push_str(code);
            output.push_str("</code></pre>\n");
        }
        NodeKind::Text(text) => {
            output.push_str(text);
        }
        _ => {
            log::warn!("Unimplemented node type: {:?}", node.kind);
        }
    }
    
    Ok(())
}
