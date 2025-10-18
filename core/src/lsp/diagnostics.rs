// Diagnostics: parse errors, broken links, etc.

use crate::parser::{Document, Node, NodeKind, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub span: Span,
    pub severity: DiagnosticSeverity,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

// Compute diagnostics for document
pub fn compute_diagnostics(document: &Document) -> Vec<Diagnostic> {
    log::debug!("Computing diagnostics for {} nodes", document.children.len());
    
    let mut diagnostics = Vec::new();
    
    for node in &document.children {
        collect_diagnostics(node, &mut diagnostics);
    }
    
    log::info!("Found {} diagnostics", diagnostics.len());
    diagnostics
}

// Recursively collect diagnostics from a node and its children
fn collect_diagnostics(node: &Node, diagnostics: &mut Vec<Diagnostic>) {
    if let Some(span) = &node.span {
        match &node.kind {
            NodeKind::Heading { level, text } => {
                // Check for invalid heading levels
                if *level > 6 {
                    diagnostics.push(Diagnostic {
                        span: *span,
                        severity: DiagnosticSeverity::Error,
                        message: format!("Invalid heading level: {}. Must be between 1 and 6", level),
                    });
                }
                
                // Warn about empty headings
                if text.trim().is_empty() {
                    diagnostics.push(Diagnostic {
                        span: *span,
                        severity: DiagnosticSeverity::Warning,
                        message: "Empty heading text".to_string(),
                    });
                }
                
                // Hint: headings should be capitalized
                if !text.is_empty() && !text.chars().next().unwrap().is_uppercase() {
                    diagnostics.push(Diagnostic {
                        span: *span,
                        severity: DiagnosticSeverity::Hint,
                        message: "Heading should start with a capital letter".to_string(),
                    });
                }
            }
            NodeKind::Link { url, .. } => {
                // Check for empty URLs
                if url.trim().is_empty() {
                    diagnostics.push(Diagnostic {
                        span: *span,
                        severity: DiagnosticSeverity::Warning,
                        message: "Empty link URL".to_string(),
                    });
                }
                
                // Check for potentially unsafe protocols
                let lower_url = url.to_lowercase();
                if lower_url.starts_with("javascript:") || lower_url.starts_with("data:") {
                    diagnostics.push(Diagnostic {
                        span: *span,
                        severity: DiagnosticSeverity::Warning,
                        message: format!("Potentially unsafe link protocol: {}", url.split(':').next().unwrap_or("unknown")),
                    });
                }
                
                // Info: suggest using https over http
                if lower_url.starts_with("http:") {
                    diagnostics.push(Diagnostic {
                        span: *span,
                        severity: DiagnosticSeverity::Info,
                        message: "Consider using HTTPS instead of HTTP".to_string(),
                    });
                }
            }
            NodeKind::CodeBlock { language, code } => {
                // Warn about empty code blocks
                if code.trim().is_empty() {
                    diagnostics.push(Diagnostic {
                        span: *span,
                        severity: DiagnosticSeverity::Info,
                        message: "Empty code block".to_string(),
                    });
                }
                
                // Info: suggest language tag for code blocks
                if language.is_none() {
                    diagnostics.push(Diagnostic {
                        span: *span,
                        severity: DiagnosticSeverity::Hint,
                        message: "Consider adding a language identifier for syntax highlighting".to_string(),
                    });
                }
            }
            NodeKind::CodeSpan(code) => {
                // Warn about empty code spans
                if code.trim().is_empty() {
                    diagnostics.push(Diagnostic {
                        span: *span,
                        severity: DiagnosticSeverity::Info,
                        message: "Empty code span".to_string(),
                    });
                }
            }
            _ => {
                // No specific diagnostics for other node types
            }
        }
    }
    
    // Recursively process children
    for child in &node.children {
        collect_diagnostics(child, diagnostics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Position;
    
    #[test]
    fn smoke_test_heading_diagnostics() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Heading {
                        level: 7,  // Invalid level
                        text: "Too deep".to_string(),
                    },
                    span: Some(Span {
                        start: Position { line: 1, column: 1, offset: 0 },
                        end: Position { line: 1, column: 15, offset: 14 },
                    }),
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Heading {
                        level: 1,
                        text: "".to_string(),  // Empty heading
                    },
                    span: Some(Span {
                        start: Position { line: 2, column: 1, offset: 15 },
                        end: Position { line: 2, column: 3, offset: 17 },
                    }),
                    children: vec![],
                },
            ],
        };
        
        let diagnostics = compute_diagnostics(&doc);
        
        assert!(diagnostics.len() >= 2);
        assert!(diagnostics.iter().any(|d| d.severity == DiagnosticSeverity::Error));
        assert!(diagnostics.iter().any(|d| d.severity == DiagnosticSeverity::Warning));
    }
    
    #[test]
    fn smoke_test_link_diagnostics() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Paragraph,
                    span: None,
                    children: vec![
                        Node {
                            kind: NodeKind::Link {
                                url: "".to_string(),  // Empty URL
                                title: None,
                            },
                            span: Some(Span {
                                start: Position { line: 1, column: 1, offset: 0 },
                                end: Position { line: 1, column: 10, offset: 9 },
                            }),
                            children: vec![],
                        },
                        Node {
                            kind: NodeKind::Link {
                                url: "javascript:alert('xss')".to_string(),  // Unsafe protocol
                                title: None,
                            },
                            span: Some(Span {
                                start: Position { line: 1, column: 11, offset: 10 },
                                end: Position { line: 1, column: 40, offset: 39 },
                            }),
                            children: vec![],
                        },
                    ],
                },
            ],
        };
        
        let diagnostics = compute_diagnostics(&doc);
        
        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics.iter().all(|d| d.severity == DiagnosticSeverity::Warning));
    }
    
    #[test]
    fn smoke_test_code_diagnostics() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::CodeBlock {
                        language: None,  // No language tag
                        code: "let x = 42;".to_string(),
                    },
                    span: Some(Span {
                        start: Position { line: 1, column: 1, offset: 0 },
                        end: Position { line: 3, column: 4, offset: 20 },
                    }),
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Paragraph,
                    span: None,
                    children: vec![
                        Node {
                            kind: NodeKind::CodeSpan("".to_string()),  // Empty code span
                            span: Some(Span {
                                start: Position { line: 5, column: 1, offset: 25 },
                                end: Position { line: 5, column: 3, offset: 27 },
                            }),
                            children: vec![],
                        },
                    ],
                },
            ],
        };
        
        let diagnostics = compute_diagnostics(&doc);
        
        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics.iter().any(|d| d.severity == DiagnosticSeverity::Hint));
        assert!(diagnostics.iter().any(|d| d.severity == DiagnosticSeverity::Info));
    }
    
    #[test]
    fn smoke_test_no_diagnostics() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Heading {
                        level: 1,
                        text: "Good heading".to_string(),
                    },
                    span: Some(Span {
                        start: Position { line: 1, column: 1, offset: 0 },
                        end: Position { line: 1, column: 15, offset: 14 },
                    }),
                    children: vec![],
                },
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
                                start: Position { line: 3, column: 1, offset: 16 },
                                end: Position { line: 3, column: 30, offset: 45 },
                            }),
                            children: vec![],
                        },
                    ],
                },
            ],
        };
        
        let diagnostics = compute_diagnostics(&doc);
        
        // Should have no errors or warnings, only possibly hints
        assert!(diagnostics.iter().all(|d| matches!(d.severity, DiagnosticSeverity::Hint)));
    }
}
