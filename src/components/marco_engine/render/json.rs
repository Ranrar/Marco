use crate::components::marco_engine::ast::{Node, Visitor};
use serde_json::{json, Value};

pub struct JsonRenderer {
    pretty_print: bool,
}

impl JsonRenderer {
    pub fn new(pretty_print: bool) -> Self {
        Self { pretty_print }
    }
    
    pub fn render(&self, ast: &Node) -> Result<String, serde_json::Error> {
        let json_value = self.node_to_json(ast);
        
        if self.pretty_print {
            serde_json::to_string_pretty(&json_value)
        } else {
            serde_json::to_string(&json_value)
        }
    }
    
    fn node_to_json(&self, node: &Node) -> Value {
        match node {
            Node::Document { children, span } => json!({
                "type": "document",
                "children": children.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),
            
            Node::Heading { level, content, span } => json!({
                "type": "heading",
                "level": level,
                "content": content.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),
            
            Node::Paragraph { content, span } => json!({
                "type": "paragraph",
                "content": content.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),
            
            Node::CodeBlock { language, content, span } => json!({
                "type": "code_block",
                "language": language,
                "content": content,
                "span": self.span_to_json(span)
            }),
            
            Node::MathBlock { content, span } => json!({
                "type": "math_block",
                "content": content,
                "span": self.span_to_json(span)
            }),
            
            Node::List { ordered, items, span } => json!({
                "type": "list",
                "ordered": ordered,
                "items": items.iter().map(|item| self.node_to_json(item)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),
            
            Node::ListItem { content, checked, span } => json!({
                "type": "list_item",
                "content": content.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "checked": checked,
                "span": self.span_to_json(span)
            }),
            
            Node::Table { headers, rows, span } => json!({
                "type": "table",
                "headers": headers.iter().map(|header| self.node_to_json(header)).collect::<Vec<_>>(),
                "rows": rows.iter().map(|row| {
                    row.iter().map(|cell| self.node_to_json(cell)).collect::<Vec<_>>()
                }).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),
            
            Node::Text { content, span } => json!({
                "type": "text",
                "content": content,
                "span": self.span_to_json(span)
            }),
            
            Node::Emphasis { content, span } => json!({
                "type": "emphasis",
                "content": content.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),
            
            Node::Strong { content, span } => json!({
                "type": "strong",
                "content": content.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),
            
            Node::Code { content, span } => json!({
                "type": "code",
                "content": content,
                "span": self.span_to_json(span)
            }),
            
            Node::Link { text, url, title, span } => json!({
                "type": "link",
                "text": text.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "url": url,
                "title": title,
                "span": self.span_to_json(span)
            }),
            
            Node::Image { alt, url, title, span } => json!({
                "type": "image",
                "alt": alt,
                "url": url,
                "title": title,
                "span": self.span_to_json(span)
            }),
            
            Node::Macro { name, arguments, content, span } => json!({
                "type": "macro",
                "name": name,
                "arguments": arguments,
                "content": content.as_ref().map(|content| {
                    content.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>()
                }),
                "span": self.span_to_json(span)
            }),
            
            Node::HorizontalRule { span } => json!({
                "type": "horizontal_rule",
                "span": self.span_to_json(span)
            }),
            
            Node::BlockQuote { content, span } => json!({
                "type": "block_quote",
                "content": content.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),
            
            Node::Unknown { content, rule, span } => json!({
                "type": "unknown",
                "content": content,
                "rule": rule,
                "span": self.span_to_json(span)
            }),
        }
    }
    
    fn span_to_json(&self, span: &crate::components::marco_engine::ast::Span) -> Value {
        json!({
            "start": span.start,
            "end": span.end
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::marco_engine::ast::{Node, Span};
    
    #[test]
    fn test_json_renderer() {
        let ast = Node::Document {
            children: vec![
                Node::heading(1, vec![Node::text("Hello", Span::empty())], Span::empty()),
                Node::paragraph(vec![Node::text("World", Span::empty())], Span::empty()),
            ],
            span: Span::empty(),
        };
        
        let renderer = JsonRenderer::new(true);
        let json = renderer.render(&ast).unwrap();
        
        assert!(json.contains("\"type\": \"document\""));
        assert!(json.contains("\"type\": \"heading\""));
        assert!(json.contains("\"level\": 1"));
        assert!(json.contains("\"content\": \"Hello\""));
    }
    
    #[test]
    fn test_complex_json_structure() {
        let ast = Node::Document {
            children: vec![
                Node::List {
                    ordered: false,
                    items: vec![
                        Node::ListItem {
                            content: vec![Node::text("Item 1", Span::empty())],
                            checked: Some(true),
                            span: Span::empty(),
                        },
                        Node::ListItem {
                            content: vec![Node::text("Item 2", Span::empty())],
                            checked: Some(false),
                            span: Span::empty(),
                        },
                    ],
                    span: Span::empty(),
                },
            ],
            span: Span::empty(),
        };
        
        let renderer = JsonRenderer::new(false);
        let json = renderer.render(&ast).unwrap();
        
        // Parse back to verify structure
        let parsed: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "document");
        assert_eq!(parsed["children"][0]["type"], "list");
        assert_eq!(parsed["children"][0]["ordered"], false);
        assert_eq!(parsed["children"][0]["items"][0]["checked"], true);
        assert_eq!(parsed["children"][0]["items"][1]["checked"], false);
    }
}
