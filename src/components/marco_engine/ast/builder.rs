use crate::components::marco_engine::{
    ast::{Node, Span},
    errors::Result,
    grammar::Rule,
};
use pest::iterators::{Pair, Pairs};

pub struct AstBuilder;

impl AstBuilder {
    pub fn build(pairs: Pairs<Rule>) -> Result<Node> {
        let mut children = Vec::new();
        let mut document_span = Span::empty();

        for pair in pairs {
            match Self::build_node(pair) {
                Ok(node) => {
                    // Update document span to encompass all children
                    let node_span = node.span();
                    if children.is_empty() {
                        document_span = node_span.clone();
                    } else {
                        document_span = Span::simple(
                            document_span.start.min(node_span.start),
                            document_span.end.max(node_span.end),
                        );
                    }
                    children.push(node);
                }
                Err(e) => {
                    // For now, create a fallback node for errors
                    eprintln!("Parse error: {}", e);
                }
            }
        }

        Ok(Node::Document {
            children,
            span: document_span,
        })
    }

    fn build_node(pair: Pair<Rule>) -> Result<Node> {
        let span = Span::simple(pair.as_span().start(), pair.as_span().end());

        match pair.as_rule() {
            Rule::document => Self::build_document(pair, span),
            Rule::heading => Self::build_heading(pair, span),
            Rule::paragraph => Self::build_paragraph(pair, span),
            Rule::code_block => Self::build_code_block(pair, span),
            Rule::math_block => Self::build_math_block(pair, span),
            Rule::list => Self::build_list(pair, span),
            Rule::bold => Self::build_strong(pair, span),
            Rule::emphasis => Self::build_emphasis(pair, span),
            Rule::code_inline => Ok(Node::Code {
                content: pair.as_str().trim_matches('`').to_string(),
                span,
            }),
            Rule::inline_link => Self::build_link(pair, span),
            Rule::inline_image => Self::build_image(pair, span),
            Rule::text => Ok(Node::Text {
                content: pair.as_str().to_string(),
                span,
            }),
            _ => {
                // For any unhandled rule, create a text node with the raw content
                Ok(Node::Text {
                    content: pair.as_str().to_string(),
                    span,
                })
            }
        }
    }

    fn build_document(pair: Pair<Rule>, span: Span) -> Result<Node> {
        let mut children = Vec::new();

        for inner in pair.into_inner() {
            if let Ok(node) = Self::build_node(inner) {
                children.push(node);
            }
        }

        Ok(Node::Document { children, span })
    }

    fn build_heading(pair: Pair<Rule>, span: Span) -> Result<Node> {
        let mut level = 1;
        let mut content = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::H1 => level = 1,
                Rule::H2 => level = 2,
                Rule::H3 => level = 3,
                Rule::H4 => level = 4,
                Rule::H5 => level = 5,
                Rule::H6 => level = 6,
                Rule::heading_content => {
                    for heading_inner in inner.into_inner() {
                        if let Ok(node) = Self::build_node(heading_inner) {
                            content.push(node);
                        }
                    }
                }
                _ => {
                    // Fallback: treat as text content
                    content.push(Node::Text {
                        content: inner.as_str().to_string(),
                        span: Span::simple(inner.as_span().start(), inner.as_span().end()),
                    });
                }
            }
        }

        Ok(Node::heading(level, content, span))
    }

    fn build_paragraph(pair: Pair<Rule>, span: Span) -> Result<Node> {
        let mut content = Vec::new();

        for inner in pair.into_inner() {
            if let Ok(node) = Self::build_node(inner) {
                content.push(node);
            }
        }

        Ok(Node::paragraph(content, span))
    }

    fn build_code_block(pair: Pair<Rule>, span: Span) -> Result<Node> {
        // Simple extraction from raw text
        let content = pair.as_str();
        let language = if content.starts_with("```") {
            let lines: Vec<&str> = content.lines().collect();
            if lines.len() > 0 {
                let first_line = lines[0];
                if first_line.len() > 3 {
                    let lang = first_line[3..].trim();
                    if lang.is_empty() {
                        None
                    } else {
                        Some(lang.to_string())
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let code_content = if content.starts_with("```") && content.ends_with("```") {
            let lines: Vec<&str> = content.lines().collect();
            if lines.len() >= 2 {
                lines[1..lines.len() - 1].join("\n")
            } else {
                String::new()
            }
        } else {
            // Indented code
            content
                .lines()
                .map(|line| {
                    if line.starts_with("    ") {
                        &line[4..]
                    } else if line.starts_with("\t") {
                        &line[1..]
                    } else {
                        line
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        Ok(Node::code_block(language, code_content, span))
    }

    fn build_math_block(pair: Pair<Rule>, span: Span) -> Result<Node> {
        let content = pair
            .as_str()
            .trim_start_matches("$$")
            .trim_end_matches("$$")
            .to_string();
        Ok(Node::MathBlock { content, span })
    }

    fn build_list(pair: Pair<Rule>, span: Span) -> Result<Node> {
        let mut items = Vec::new();

        // Extract text content before consuming the pair
        let list_text = pair.as_str().to_string();

        for inner in pair.into_inner() {
            if let Ok(item) = Self::build_list_item(inner) {
                items.push(item);
            }
        }

        // Determine if ordered by checking the first item's marker
        let ordered = list_text.contains("1.") || list_text.contains("2.");

        Ok(Node::list(ordered, items, span))
    }
    fn build_list_item(pair: Pair<Rule>) -> Result<Node> {
        let span = Span::simple(pair.as_span().start(), pair.as_span().end());
        let mut content = Vec::new();
        let mut checked = None;

        // Simple text extraction for list items
        let text = pair.as_str();
        if text.contains("[ ]") {
            checked = Some(false);
        } else if text.contains("[x]") || text.contains("[X]") {
            checked = Some(true);
        }

        // Extract content after markers
        let clean_text = text
            .trim_start_matches(|c: char| c.is_ascii_digit() || ".-*+ \t".contains(c))
            .trim_start_matches("[x]")
            .trim_start_matches("[X]")
            .trim_start_matches("[ ]")
            .trim_start();

        if !clean_text.is_empty() {
            content.push(Node::Text {
                content: clean_text.to_string(),
                span: span.clone(),
            });
        }

        Ok(Node::list_item(content, checked, span))
    }

    fn build_strong(pair: Pair<Rule>, span: Span) -> Result<Node> {
        let content = pair
            .as_str()
            .trim_start_matches("**")
            .trim_end_matches("**")
            .trim_start_matches("__")
            .trim_end_matches("__");
        Ok(Node::Strong {
            content: vec![Node::Text {
                content: content.to_string(),
                span: span.clone(),
            }],
            span,
        })
    }

    fn build_emphasis(pair: Pair<Rule>, span: Span) -> Result<Node> {
        let content = pair
            .as_str()
            .trim_start_matches('*')
            .trim_end_matches('*')
            .trim_start_matches('_')
            .trim_end_matches('_');
        Ok(Node::Emphasis {
            content: vec![Node::Text {
                content: content.to_string(),
                span: span.clone(),
            }],
            span,
        })
    }

    fn build_link(pair: Pair<Rule>, span: Span) -> Result<Node> {
        // Simple regex-like extraction for now
        let text = pair.as_str();
        if let Some(close_bracket) = text.find("](") {
            let link_text = &text[1..close_bracket];
            let url_part = &text[close_bracket + 2..];
            let url = if let Some(close_paren) = url_part.find(')') {
                &url_part[..close_paren]
            } else {
                url_part
            };

            Ok(Node::Link {
                text: vec![Node::Text {
                    content: link_text.to_string(),
                    span: span.clone(),
                }],
                url: url.to_string(),
                title: None,
                span,
            })
        } else {
            // Fallback to text
            Ok(Node::Text {
                content: text.to_string(),
                span,
            })
        }
    }

    fn build_image(pair: Pair<Rule>, span: Span) -> Result<Node> {
        // Simple extraction for ![alt](url)
        let text = pair.as_str();
        if text.starts_with("![") {
            if let Some(close_bracket) = text.find("](") {
                let alt_text = &text[2..close_bracket];
                let url_part = &text[close_bracket + 2..];
                let url = if let Some(close_paren) = url_part.find(')') {
                    &url_part[..close_paren]
                } else {
                    url_part
                };

                Ok(Node::Image {
                    alt: alt_text.to_string(),
                    url: url.to_string(),
                    title: None,
                    span,
                })
            } else {
                Ok(Node::Text {
                    content: text.to_string(),
                    span,
                })
            }
        } else {
            Ok(Node::Text {
                content: text.to_string(),
                span,
            })
        }
    }
}
