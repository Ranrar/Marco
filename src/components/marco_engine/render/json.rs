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

            Node::Heading {
                level,
                content,
                span,
            } => json!({
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

            Node::CodeBlock {
                language,
                content,
                span,
            } => json!({
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

            Node::List {
                ordered,
                items,
                span,
            } => json!({
                "type": "list",
                "ordered": ordered,
                "items": items.iter().map(|item| self.node_to_json(item)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),

            Node::ListItem {
                content,
                checked,
                span,
            } => json!({
                "type": "list_item",
                "content": content.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "checked": checked,
                "span": self.span_to_json(span)
            }),

            Node::Table {
                headers,
                rows,
                span,
            } => json!({
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

            Node::MathInline { content, span } => json!({
                "type": "math_inline",
                "content": content,
                "span": self.span_to_json(span)
            }),

            Node::Link {
                text,
                url,
                title,
                span,
            } => json!({
                "type": "link",
                "text": text.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "url": url,
                "title": title,
                "span": self.span_to_json(span)
            }),

            Node::Image {
                alt,
                url,
                title,
                span,
            } => json!({
                "type": "image",
                "alt": alt,
                "url": url,
                "title": title,
                "span": self.span_to_json(span)
            }),

            Node::Macro {
                name,
                arguments,
                content,
                span,
            } => json!({
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

            Node::Strikethrough { content, span } => json!({
                "type": "strikethrough",
                "content": content.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),

            Node::Highlight { content, span } => json!({
                "type": "highlight",
                "content": content.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),

            Node::Superscript { content, span } => json!({
                "type": "superscript",
                "content": content.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),

            Node::Subscript { content, span } => json!({
                "type": "subscript",
                "content": content.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),

            Node::ReferenceLink { text, label, span } => json!({
                "type": "reference_link",
                "text": text.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "label": label,
                "span": self.span_to_json(span)
            }),

            Node::ReferenceImage { alt, label, span } => json!({
                "type": "reference_image",
                "alt": alt,
                "label": label,
                "span": self.span_to_json(span)
            }),

            Node::FootnoteRef { label, span } => json!({
                "type": "footnote_ref",
                "label": label,
                "span": self.span_to_json(span)
            }),

            Node::InlineFootnote { content, span } => json!({
                "type": "inline_footnote",
                "content": content.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),

            Node::Autolink { url, span } => json!({
                "type": "autolink",
                "url": url,
                "span": self.span_to_json(span)
            }),

            Node::Emoji { name, span } => json!({
                "type": "emoji",
                "name": name,
                "span": self.span_to_json(span)
            }),

            Node::LineBreak { span } => json!({
                "type": "line_break",
                "span": self.span_to_json(span)
            }),

            Node::EscapedChar { character, span } => json!({
                "type": "escaped_char",
                "character": character,
                "span": self.span_to_json(span)
            }),

            Node::InlineHTML { content, span } => json!({
                "type": "inline_html",
                "content": content,
                "span": self.span_to_json(span)
            }),

            Node::BlockHTML { content, span } => json!({
                "type": "block_html",
                "content": content,
                "span": self.span_to_json(span)
            }),

            Node::DefinitionList { items, span } => json!({
                "type": "definition_list",
                "items": items.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),

            Node::DefinitionTerm { content, span } => json!({
                "type": "definition_term",
                "content": content.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),

            Node::DefinitionDescription { content, span } => json!({
                "type": "definition_description",
                "content": content.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),

            Node::FootnoteDefinition {
                label,
                content,
                span,
            } => json!({
                "type": "footnote_definition",
                "label": label,
                "content": content.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),

            Node::ReferenceDefinition {
                label,
                url,
                title,
                span,
            } => json!({
                "type": "reference_definition",
                "label": label,
                "url": url,
                "title": title,
                "span": self.span_to_json(span)
            }),

            Node::UserMention {
                username,
                platform,
                display_name,
                span,
            } => json!({
                "type": "user_mention",
                "username": username,
                "platform": platform,
                "display_name": display_name,
                "span": self.span_to_json(span)
            }),

            Node::Bookmark {
                label,
                path,
                line,
                span,
            } => json!({
                "type": "bookmark",
                "label": label,
                "path": path,
                "line": line,
                "span": self.span_to_json(span)
            }),

            Node::PageTag { format, span } => json!({
                "type": "page_tag",
                "format": format,
                "span": self.span_to_json(span)
            }),

            Node::DocumentReference { path, span } => json!({
                "type": "document_reference",
                "path": path,
                "span": self.span_to_json(span)
            }),

            Node::TableOfContents {
                depth,
                document,
                span,
            } => json!({
                "type": "table_of_contents",
                "depth": depth,
                "document": document,
                "span": self.span_to_json(span)
            }),

            Node::RunInline {
                script_type,
                command,
                span,
            } => json!({
                "type": "run_inline",
                "script_type": script_type,
                "command": command,
                "span": self.span_to_json(span)
            }),

            Node::RunBlock {
                script_type,
                content,
                span,
            } => json!({
                "type": "run_block",
                "script_type": script_type,
                "content": content,
                "span": self.span_to_json(span)
            }),

            Node::DiagramBlock {
                diagram_type,
                content,
                span,
            } => json!({
                "type": "diagram_block",
                "diagram_type": diagram_type,
                "content": content,
                "span": self.span_to_json(span)
            }),

            Node::TabBlock { title, tabs, span } => json!({
                "type": "tab_block",
                "title": title,
                "tabs": tabs.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),

            Node::Tab {
                name,
                content,
                span,
            } => json!({
                "type": "tab",
                "name": name,
                "content": content.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),

            Node::Admonition {
                kind,
                content,
                span,
            } => json!({
                "type": "admonition",
                "kind": kind,
                "content": content.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),

            Node::TaskItem {
                checked,
                content,
                span,
            } => json!({
                "type": "task_item",
                "checked": checked,
                "content": content.iter().map(|child| self.node_to_json(child)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),

            Node::Unknown {
                content,
                rule,
                span,
            } => json!({
                "type": "unknown",
                "content": content,
                "rule": rule,
                "span": self.span_to_json(span)
            }),

            // New Node variants
            Node::SetextHeading {
                level,
                content,
                underline_char,
                span,
            } => json!({
                "type": "setext_heading",
                "level": level,
                "underline_char": underline_char,
                "content": content.iter().map(|node| self.node_to_json(node)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),
            Node::TableHeader { cells, span } => json!({
                "type": "table_header",
                "cells": cells.iter().map(|cell| self.node_to_json(cell)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),
            Node::TableRow { cells, span } => json!({
                "type": "table_row",
                "cells": cells.iter().map(|cell| self.node_to_json(cell)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),
            Node::TableCell {
                content,
                alignment,
                span,
            } => json!({
                "type": "table_cell",
                "content": content.iter().map(|node| self.node_to_json(node)).collect::<Vec<_>>(),
                "alignment": alignment,
                "span": self.span_to_json(span)
            }),
            Node::ThematicBreak { marker, span } => json!({
                "type": "thematic_break",
                "marker": marker.to_string(),
                "span": self.span_to_json(span)
            }),
            Node::SoftLineBreak { span } => json!({
                "type": "soft_line_break",
                "span": self.span_to_json(span)
            }),
            Node::HardLineBreak { span } => json!({
                "type": "hard_line_break",
                "span": self.span_to_json(span)
            }),
            Node::HtmlBlock {
                html_type,
                content,
                span,
            } => json!({
                "type": "html_block",
                "html_type": html_type,
                "content": content,
                "span": self.span_to_json(span)
            }),
            Node::FencedCodeBlock {
                language,
                info_string,
                content,
                fence_char,
                fence_length,
                span,
            } => json!({
                "type": "fenced_code_block",
                "language": language,
                "info_string": info_string,
                "content": content,
                "fence_char": fence_char.to_string(),
                "fence_length": fence_length,
                "span": self.span_to_json(span)
            }),
            Node::IndentedCodeBlock { content, span } => json!({
                "type": "indented_code_block",
                "content": content,
                "span": self.span_to_json(span)
            }),
            Node::LinkReferenceDefinition {
                label,
                destination,
                title,
                span,
            } => json!({
                "type": "link_reference_definition",
                "label": label,
                "destination": destination,
                "title": title,
                "span": self.span_to_json(span)
            }),
            Node::MathBlockDisplay {
                content,
                delimiter,
                span,
            } => json!({
                "type": "math_block_display",
                "content": content,
                "delimiter": delimiter,
                "span": self.span_to_json(span)
            }),
            Node::CodeSpan {
                content,
                backtick_count,
                span,
            } => json!({
                "type": "code_span",
                "content": content,
                "backtick_count": backtick_count,
                "span": self.span_to_json(span)
            }),
            Node::HtmlInlineTag {
                tag_name,
                attributes,
                content,
                is_self_closing,
                span,
            } => json!({
                "type": "html_inline_tag",
                "tag_name": tag_name,
                "attributes": attributes,
                "content": content.as_ref().map(|nodes| nodes.iter().map(|node| self.node_to_json(node)).collect::<Vec<_>>()),
                "is_self_closing": is_self_closing,
                "span": self.span_to_json(span)
            }),
            Node::AutolinkUrl { url, span } => json!({
                "type": "autolink_url",
                "url": url,
                "span": self.span_to_json(span)
            }),
            Node::AutolinkEmail { email, span } => json!({
                "type": "autolink_email",
                "email": email,
                "span": self.span_to_json(span)
            }),
            Node::AdmonitionWithIcon {
                kind,
                icon,
                title,
                content,
                span,
            } => json!({
                "type": "admonition_with_icon",
                "kind": kind,
                "icon": icon,
                "title": title,
                "content": content.iter().map(|node| self.node_to_json(node)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),
            Node::TabWithMetadata {
                name,
                icon,
                active,
                content,
                span,
            } => json!({
                "type": "tab_with_metadata",
                "name": name,
                "icon": icon,
                "active": active,
                "content": content.iter().map(|node| self.node_to_json(node)).collect::<Vec<_>>(),
                "span": self.span_to_json(span)
            }),
            Node::UserMentionWithMetadata {
                username,
                platform,
                display_name,
                user_id,
                avatar_url,
                span,
            } => json!({
                "type": "user_mention_with_metadata",
                "username": username,
                "platform": platform,
                "display_name": display_name,
                "user_id": user_id,
                "avatar_url": avatar_url,
                "span": self.span_to_json(span)
            }),
            Node::Citation { key, locator, span } => json!({
                "type": "citation",
                "key": key,
                "locator": locator,
                "span": self.span_to_json(span)
            }),
            Node::Keyboard { keys, span } => json!({
                "type": "keyboard",
                "keys": keys,
                "span": self.span_to_json(span)
            }),
            Node::Mark {
                content,
                reason,
                span,
            } => json!({
                "type": "mark",
                "content": content.iter().map(|node| self.node_to_json(node)).collect::<Vec<_>>(),
                "reason": reason,
                "span": self.span_to_json(span)
            }),
            Node::Details {
                summary,
                content,
                open,
                span,
            } => json!({
                "type": "details",
                "summary": summary.iter().map(|node| self.node_to_json(node)).collect::<Vec<_>>(),
                "content": content.iter().map(|node| self.node_to_json(node)).collect::<Vec<_>>(),
                "open": open,
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
            children: vec![Node::List {
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
            }],
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
