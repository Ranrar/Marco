// Syntax highlighting: map AST nodes to SourceView5 text tags

use crate::parser::{Document, Node, NodeKind, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct Highlight {
    pub span: Span,
    pub tag: HighlightTag,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HighlightTag {
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
    Emphasis,
    Strong,
    Link,
    Image,
    CodeSpan,
    CodeBlock,
    InlineHtml,
    HardBreak,
    SoftBreak,
}

// Generate highlights from AST by walking all nodes
pub fn compute_highlights(document: &Document) -> Vec<Highlight> {
    log::debug!("Computing syntax highlights for {} nodes", document.children.len());
    
    let mut highlights = Vec::new();
    
    for node in &document.children {
        collect_highlights(node, &mut highlights);
    }
    
    log::info!("Generated {} highlights", highlights.len());
    highlights
}

// Recursively collect highlights from a node and its children
fn collect_highlights(node: &Node, highlights: &mut Vec<Highlight>) {
    // Add highlight for this node if it has a span and is highlightable
    if let Some(span) = &node.span {
        match &node.kind {
            NodeKind::Heading { level, .. } => {
                let tag = match level {
                    1 => HighlightTag::Heading1,
                    2 => HighlightTag::Heading2,
                    3 => HighlightTag::Heading3,
                    4 => HighlightTag::Heading4,
                    5 => HighlightTag::Heading5,
                    6 => HighlightTag::Heading6,
                    _ => HighlightTag::Heading1, // Fallback for invalid levels
                };
                
                highlights.push(Highlight {
                    span: *span,
                    tag,
                });
            }
            NodeKind::Emphasis => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::Emphasis,
                });
            }
            NodeKind::Strong => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::Strong,
                });
            }
            NodeKind::Link { .. } => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::Link,
                });
            }
            NodeKind::Image { .. } => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::Image,
                });
            }
            NodeKind::CodeSpan(_) => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::CodeSpan,
                });
            }
            NodeKind::CodeBlock { .. } => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::CodeBlock,
                });
            }
            NodeKind::InlineHtml(_) => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::InlineHtml,
                });
            }
            NodeKind::HardBreak => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::HardBreak,
                });
            }
            NodeKind::SoftBreak => {
                highlights.push(Highlight {
                    span: *span,
                    tag: HighlightTag::SoftBreak,
                });
            }
            NodeKind::Text(_) | NodeKind::Paragraph => {
                // Text and Paragraph nodes don't get highlights themselves,
                // but we process their children
            }
            _ => {
                log::trace!("No highlight for node kind: {:?}", node.kind);
            }
        }
    }
    
    // Recursively process children
    for child in &node.children {
        collect_highlights(child, highlights);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Position;
    
    #[test]
    fn smoke_test_heading_highlights() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Heading {
                        level: 1,
                        text: "Title".to_string(),
                    },
                    span: Some(Span {
                        start: Position { line: 1, column: 1, offset: 0 },
                        end: Position { line: 1, column: 8, offset: 7 },
                    }),
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Heading {
                        level: 2,
                        text: "Subtitle".to_string(),
                    },
                    span: Some(Span {
                        start: Position { line: 3, column: 1, offset: 9 },
                        end: Position { line: 3, column: 12, offset: 20 },
                    }),
                    children: vec![],
                },
            ],
        ..Default::default()
        };
        
        let highlights = compute_highlights(&doc);
        
        assert_eq!(highlights.len(), 2);
        assert_eq!(highlights[0].tag, HighlightTag::Heading1);
        assert_eq!(highlights[1].tag, HighlightTag::Heading2);
    }
    
    #[test]
    fn smoke_test_inline_highlights() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Paragraph,
                    span: Some(Span {
                        start: Position { line: 1, column: 1, offset: 0 },
                        end: Position { line: 1, column: 30, offset: 29 },
                    }),
                    children: vec![
                        Node {
                            kind: NodeKind::Text("Text with ".to_string()),
                            span: None,
                            children: vec![],
                        },
                        Node {
                            kind: NodeKind::Emphasis,
                            span: Some(Span {
                                start: Position { line: 1, column: 11, offset: 10 },
                                end: Position { line: 1, column: 18, offset: 17 },
                            }),
                            children: vec![],
                        },
                        Node {
                            kind: NodeKind::Text(" and ".to_string()),
                            span: None,
                            children: vec![],
                        },
                        Node {
                            kind: NodeKind::Strong,
                            span: Some(Span {
                                start: Position { line: 1, column: 24, offset: 23 },
                                end: Position { line: 1, column: 30, offset: 29 },
                            }),
                            children: vec![],
                        },
                    ],
                },
            ],
        ..Default::default()
        };
        
        let highlights = compute_highlights(&doc);
        
        assert_eq!(highlights.len(), 2);
        assert_eq!(highlights[0].tag, HighlightTag::Emphasis);
        assert_eq!(highlights[1].tag, HighlightTag::Strong);
    }
    
    #[test]
    fn smoke_test_code_highlights() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::CodeBlock {
                        language: Some("rust".to_string()),
                        code: "fn main() {}".to_string(),
                    },
                    span: Some(Span {
                        start: Position { line: 1, column: 1, offset: 0 },
                        end: Position { line: 3, column: 4, offset: 25 },
                    }),
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Paragraph,
                    span: None,
                    children: vec![
                        Node {
                            kind: NodeKind::CodeSpan("code()".to_string()),
                            span: Some(Span {
                                start: Position { line: 5, column: 5, offset: 30 },
                                end: Position { line: 5, column: 13, offset: 38 },
                            }),
                            children: vec![],
                        },
                    ],
                },
            ],
        ..Default::default()
        };
        
        let highlights = compute_highlights(&doc);
        
        assert_eq!(highlights.len(), 2);
        assert_eq!(highlights[0].tag, HighlightTag::CodeBlock);
        assert_eq!(highlights[1].tag, HighlightTag::CodeSpan);
    }
    
    #[test]
    fn smoke_test_link_highlights() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Paragraph,
                    span: None,
                    children: vec![
                        Node {
                            kind: NodeKind::Link {
                                url: "https://example.com".to_string(),
                                title: None,
                            },
                            span: Some(Span {
                                start: Position { line: 1, column: 1, offset: 0 },
                                end: Position { line: 1, column: 30, offset: 29 },
                            }),
                            children: vec![],
                        },
                    ],
                },
            ],
        ..Default::default()
        };
        
        let highlights = compute_highlights(&doc);
        
        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].tag, HighlightTag::Link);
    }
    
    #[test]
    fn smoke_test_image_highlights() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Paragraph,
                    span: None,
                    children: vec![
                        Node {
                            kind: NodeKind::Image {
                                url: "image.png".to_string(),
                                alt: "Alt text".to_string(),
                            },
                            span: Some(Span {
                                start: Position { line: 1, column: 1, offset: 0 },
                                end: Position { line: 1, column: 25, offset: 24 },
                            }),
                            children: vec![],
                        },
                    ],
                },
            ],
        ..Default::default()
        };
        
        let highlights = compute_highlights(&doc);
        
        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].tag, HighlightTag::Image);
    }
    
    #[test]
    fn smoke_test_inline_html_highlights() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Paragraph,
                    span: None,
                    children: vec![
                        Node {
                            kind: NodeKind::InlineHtml("<span>text</span>".to_string()),
                            span: Some(Span {
                                start: Position { line: 1, column: 1, offset: 0 },
                                end: Position { line: 1, column: 18, offset: 17 },
                            }),
                            children: vec![],
                        },
                    ],
                },
            ],
        ..Default::default()
        };
        
        let highlights = compute_highlights(&doc);
        
        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].tag, HighlightTag::InlineHtml);
    }
    
    #[test]
    fn smoke_test_hard_break_highlights() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Paragraph,
                    span: None,
                    children: vec![
                        Node {
                            kind: NodeKind::Text("Line one".to_string()),
                            span: None,
                            children: vec![],
                        },
                        Node {
                            kind: NodeKind::HardBreak,
                            span: Some(Span {
                                start: Position { line: 1, column: 9, offset: 8 },
                                end: Position { line: 2, column: 1, offset: 11 },
                            }),
                            children: vec![],
                        },
                        Node {
                            kind: NodeKind::Text("Line two".to_string()),
                            span: None,
                            children: vec![],
                        },
                    ],
                },
            ],
        ..Default::default()
        };
        
        let highlights = compute_highlights(&doc);
        
        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].tag, HighlightTag::HardBreak);
    }
    
    #[test]
    fn smoke_test_soft_break_highlights() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Paragraph,
                    span: None,
                    children: vec![
                        Node {
                            kind: NodeKind::Text("Line one".to_string()),
                            span: None,
                            children: vec![],
                        },
                        Node {
                            kind: NodeKind::SoftBreak,
                            span: Some(Span {
                                start: Position { line: 1, column: 9, offset: 8 },
                                end: Position { line: 2, column: 1, offset: 9 },
                            }),
                            children: vec![],
                        },
                        Node {
                            kind: NodeKind::Text("Line two".to_string()),
                            span: None,
                            children: vec![],
                        },
                    ],
                },
            ],
        ..Default::default()
        };
        
        let highlights = compute_highlights(&doc);
        
        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].tag, HighlightTag::SoftBreak);
    }
}
