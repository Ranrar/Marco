use crate::components::marco_engine::{
    ast::{Node, Span},
    errors::Result,
    grammar::Rule,
};
use log::debug;
use pest::iterators::{Pair, Pairs};

pub struct AstBuilder;

impl AstBuilder {
    pub fn build(pairs: Pairs<Rule>) -> Result<Node> {
        debug!("AstBuilder::build - Starting AST building");
        let mut children = Vec::new();
        let mut document_span = Span::empty();

        for pair in pairs {
            debug!(
                "AstBuilder::build - Processing pair: {:?} with text: '{}'",
                pair.as_rule(),
                pair.as_str()
            );
            match Self::build_node(pair) {
                Ok(node) => {
                    debug!(
                        "AstBuilder::build - Successfully built node: {:?}",
                        std::mem::discriminant(&node)
                    );
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
                    debug!("AstBuilder::build - Parse error: {}", e);
                    eprintln!("Parse error: {}", e);
                }
            }
        }

        debug!(
            "AstBuilder::build - Created document with {} children",
            children.len()
        );
        Ok(Node::Document {
            children,
            span: document_span,
        })
    }

    fn build_node(pair: Pair<Rule>) -> Result<Node> {
        let span = Span::simple(pair.as_span().start(), pair.as_span().end());

        match pair.as_rule() {
            Rule::file => Self::build_node(pair.into_inner().next().unwrap()),
            Rule::block => Self::build_node(pair.into_inner().next().unwrap()),
            Rule::document => Self::build_document(pair, span),
            Rule::heading => Self::build_heading(pair, span),
            Rule::paragraph => Self::build_paragraph(pair, span),
            Rule::paragraph_line => Self::build_paragraph_line(pair, span),
            Rule::inline => Self::build_inline(pair),
            Rule::inline_core => Self::build_inline_core(pair),
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
            Rule::text => {
                // Check if text contains newlines - if so, split into separate text nodes
                let text_content = pair.as_str();
                if text_content.contains('\n') {
                    // Split on newlines and create separate text nodes
                    let lines: Vec<_> = text_content.split('\n').collect();
                    if lines.len() > 1 {
                        // For multiline text, we'll return a special indicator that the parent should handle
                        // For now, just return the full text and let build_paragraph handle the splitting
                        Ok(Node::Text {
                            content: text_content.to_string(),
                            span,
                        })
                    } else {
                        Ok(Node::Text {
                            content: text_content.to_string(),
                            span,
                        })
                    }
                } else {
                    Ok(Node::Text {
                        content: text_content.to_string(),
                        span,
                    })
                }
            }
            // Handle specific heading and inline rules
            Rule::heading_inline => Self::build_node(pair.into_inner().next().unwrap()),
            Rule::word => Ok(Node::Text {
                content: pair.as_str().to_string(),
                span,
            }),
            Rule::bold_asterisk => Self::build_strong(pair, span),
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
                Rule::H1 => {
                    level = 1;
                    // Process H1's children to find heading_content
                    for h1_child in inner.into_inner() {
                        if h1_child.as_rule() == Rule::heading_content {
                            content = Self::build_heading_content_with_spaces(h1_child)?;
                        }
                    }
                }
                Rule::H2 => {
                    level = 2;
                    for h2_child in inner.into_inner() {
                        if h2_child.as_rule() == Rule::heading_content {
                            content = Self::build_heading_content_with_spaces(h2_child)?;
                        }
                    }
                }
                Rule::H3 => {
                    level = 3;
                    for h3_child in inner.into_inner() {
                        if h3_child.as_rule() == Rule::heading_content {
                            content = Self::build_heading_content_with_spaces(h3_child)?;
                        }
                    }
                }
                Rule::H4 => {
                    level = 4;
                    for h4_child in inner.into_inner() {
                        if h4_child.as_rule() == Rule::heading_content {
                            content = Self::build_heading_content_with_spaces(h4_child)?;
                        }
                    }
                }
                Rule::H5 => {
                    level = 5;
                    for h5_child in inner.into_inner() {
                        if h5_child.as_rule() == Rule::heading_content {
                            content = Self::build_heading_content_with_spaces(h5_child)?;
                        }
                    }
                }
                Rule::H6 => {
                    level = 6;
                    for h6_child in inner.into_inner() {
                        if h6_child.as_rule() == Rule::heading_content {
                            content = Self::build_heading_content_with_spaces(h6_child)?;
                        }
                    }
                }
                Rule::heading_content => {
                    // Build content preserving spaces between words
                    content = Self::build_heading_content_with_spaces(inner)?;
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

    fn build_heading_content_with_spaces(pair: Pair<Rule>) -> Result<Vec<Node>> {
        let full_text = pair.as_str();
        let pair_span = pair.as_span();
        let mut content = Vec::new();
        let mut last_processed = 0;

        let inlines: Vec<_> = pair.into_inner().collect();

        for inline in inlines.iter() {
            let relative_start = inline.as_span().start() - pair_span.start();
            let relative_end = inline.as_span().end() - pair_span.start();

            // Add any gap (whitespace) before this element
            if relative_start > last_processed {
                let gap_text = &full_text[last_processed..relative_start];
                if !gap_text.is_empty() {
                    content.push(Node::Text {
                        content: gap_text.to_string(),
                        span: Span::simple(
                            pair_span.start() + last_processed,
                            pair_span.start() + relative_start,
                        ),
                    });
                }
            }

            // Add the actual inline element
            if let Ok(node) = Self::build_node(inline.clone()) {
                content.push(node);
            }

            last_processed = relative_end;
        }

        // Add any remaining text after the last element
        if last_processed < full_text.len() {
            let remaining_text = &full_text[last_processed..];
            if !remaining_text.is_empty() {
                content.push(Node::Text {
                    content: remaining_text.to_string(),
                    span: Span::simple(pair_span.start() + last_processed, pair_span.end()),
                });
            }
        }

        Ok(content)
    }

    fn build_paragraph(pair: Pair<Rule>, span: Span) -> Result<Node> {
        let mut content = Vec::new();

        // Process all inner elements
        for inner in pair.into_inner() {
            if let Ok(node) = Self::build_node(inner) {
                content.push(node);
            }
        }

        // Check if any text nodes contain newlines and need to be split
        let mut has_multiline_text = false;
        let mut multiline_content = String::new();

        for node in &content {
            if let Node::Text { content: text, .. } = node {
                if text.contains('\n') {
                    has_multiline_text = true;
                    multiline_content = text.clone();
                    break;
                }
            }
        }

        if has_multiline_text && !multiline_content.trim().is_empty() {
            // Split the multiline text into separate paragraphs
            let lines: Vec<&str> = multiline_content.split('\n').collect();
            let mut paragraphs = Vec::new();

            for line in lines {
                let trimmed_line = line.trim();
                if !trimmed_line.is_empty() {
                    let text_span = Span::simple(span.start, span.end);
                    let para_span = Span::simple(span.start, span.end);
                    let line_content = vec![Node::Text {
                        content: trimmed_line.to_string(),
                        span: text_span,
                    }];
                    paragraphs.push(Node::paragraph(line_content, para_span));
                }
            }

            if paragraphs.len() > 1 {
                // Return multiple paragraphs as a document fragment
                Ok(Node::Document {
                    children: paragraphs,
                    span,
                })
            } else if paragraphs.len() == 1 {
                // Single paragraph
                Ok(paragraphs.into_iter().next().unwrap())
            } else {
                // Empty paragraph
                Ok(Node::paragraph(vec![], span))
            }
        } else {
            // No multiline text, return single paragraph
            Ok(Node::paragraph(content, span))
        }
    }

    fn build_paragraph_line(pair: Pair<Rule>, span: Span) -> Result<Node> {
        let content = pair.as_str().to_string();

        // paragraph_line is just a container, process its contents
        if let Some(inner) = pair.into_inner().next() {
            return Self::build_node(inner);
        }
        // Fallback to text if no inner content
        Ok(Node::Text { content, span })
    }

    fn build_inline(pair: Pair<Rule>) -> Result<Node> {
        let span = Span::simple(pair.as_span().start(), pair.as_span().end());
        let content = pair.as_str().to_string();

        // inline is just a container, process its first inner element
        if let Some(inner) = pair.into_inner().next() {
            return Self::build_node(inner);
        }
        // Fallback to text if no inner content
        Ok(Node::Text { content, span })
    }

    fn build_inline_core(pair: Pair<Rule>) -> Result<Node> {
        let span = Span::simple(pair.as_span().start(), pair.as_span().end());
        let content = pair.as_str().to_string();

        // inline_core is just a container, process its first inner element
        if let Some(inner) = pair.into_inner().next() {
            return Self::build_node(inner);
        }
        // Fallback to text if no inner content
        Ok(Node::Text { content, span })
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
