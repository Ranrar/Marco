// Block-level parser: first stage of two-stage parsing

use super::ast::{Node, NodeKind, Document};
use crate::grammar::block as grammar;
use anyhow::Result;
use nom_locate::LocatedSpan;

type GrammarSpan<'a> = LocatedSpan<&'a str>;

// Convert grammar LocatedSpan to parser Span
fn to_parser_span(input: GrammarSpan) -> crate::parser::position::Span {
    let start = crate::parser::position::Position::new(
        input.location_line() as usize,
        input.get_column(),
        input.location_offset(),
    );
    
    let end = crate::parser::position::Position::new(
        input.location_line() as usize,
        input.get_column() + input.fragment().len(),
        input.location_offset() + input.fragment().len(),
    );
    
    crate::parser::position::Span::new(start, end)
}

// Convert two grammar LocatedSpans (start and end) to a parser Span
fn to_parser_span_range(start_span: GrammarSpan, end_span: GrammarSpan) -> crate::parser::position::Span {
    let start = crate::parser::position::Position::new(
        start_span.location_line() as usize,
        start_span.get_column(),
        start_span.location_offset(),
    );
    
    let end = crate::parser::position::Position::new(
        end_span.location_line() as usize,
        end_span.get_column() + end_span.fragment().len(),
        end_span.location_offset() + end_span.fragment().len(),
    );
    
    crate::parser::position::Span::new(start, end)
}

// Strip list item indentation from content
// List items can have content indented up to content_indent spaces after the marker
fn dedent_list_item_content(content: &str, content_indent: usize) -> String {
    let had_trailing_newline = content.ends_with('\n');
    
    let mut result = content.lines()
        .map(|line| {
            // First, expand tabs to spaces
            // Tabs must be expanded based on their ACTUAL column position (content_indent + column in line)
            let mut expanded = String::with_capacity(line.len() * 2);
            let mut column = content_indent; // Start at the content_indent column
            
            for ch in line.chars() {
                if ch == '\t' {
                    // Tab advances to next multiple of 4
                    let spaces_to_add = 4 - (column % 4);
                    for _ in 0..spaces_to_add {
                        expanded.push(' ');
                        column += 1;
                    }
                } else {
                    expanded.push(ch);
                    column += 1;
                }
            }
            
            // Now count and strip leading spaces up to content_indent
            let mut spaces_to_strip = 0;
            let mut chars = expanded.chars();
            while spaces_to_strip < content_indent {
                match chars.next() {
                    Some(' ') => spaces_to_strip += 1,
                    _ => break,
                }
            }
            
            // Return the rest of the line after stripping (as owned String)
            expanded[spaces_to_strip..].to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");
    
    // Preserve trailing newline if original had one
    if had_trailing_newline {
        result.push('\n');
    }
    
    result
}

// Parse document into block-level structure, returning a Document
pub fn parse_blocks(input: &str) -> Result<Document> {
    parse_blocks_internal(input, 0)
}

// Internal parser with recursion depth limit
fn parse_blocks_internal(input: &str, depth: usize) -> Result<Document> {
    // Prevent infinite recursion
    const MAX_DEPTH: usize = 100;
    if depth > MAX_DEPTH {
        log::warn!("Maximum recursion depth reached in block parser");
        return Ok(Document::new());
    }
    
    log::debug!("Block parser input: {} bytes at depth {}", input.len(), depth);
    
    let mut nodes = Vec::new();
    let mut remaining = GrammarSpan::new(input);
    
    // Safety: prevent infinite loops
    const MAX_ITERATIONS: usize = 100;  // Reduced to prevent memory issues
    let mut iteration_count = 0;
    let mut last_offset = 0;
    
    while !remaining.fragment().is_empty() {
        iteration_count += 1;
        if iteration_count > MAX_ITERATIONS {
            log::error!("Block parser exceeded MAX_ITERATIONS ({}) at depth {}", MAX_ITERATIONS, depth);
            break;
        }
        
        // Safety: ensure we're making progress
        let current_offset = remaining.location_offset();
        if current_offset == last_offset && iteration_count > 1 {
            log::error!("Block parser not making progress at offset {}, depth {}", current_offset, depth);
            // Force skip one character
            let skip = remaining.fragment().chars().next().map(|c| c.len_utf8()).unwrap_or(1);
            let new_fragment = &remaining.fragment()[skip..];
            remaining = GrammarSpan::new(new_fragment);
            last_offset = remaining.location_offset();
            continue;
        }
        last_offset = current_offset;
        
        // Skip blank lines (lines containing only whitespace)
        // Extract the first line to check if it's blank
        let first_line_end = remaining.fragment().find('\n').unwrap_or(remaining.fragment().len());
        let first_line = &remaining.fragment()[..first_line_end];
        
        // A line is blank if it contains only whitespace (spaces, tabs)
        if first_line.trim().is_empty() {
            // Skip the blank line including its newline
            let skip_len = if first_line_end < remaining.fragment().len() {
                first_line_end + 1  // Include the newline
            } else {
                first_line_end  // End of input, no newline
            };
            
            // Use nom's take to skip bytes while preserving location information
            use nom::bytes::complete::take;
            if let Ok((new_remaining, _)) = take::<_, _, nom::error::Error<GrammarSpan>>(skip_len as u32)(remaining) {
                remaining = new_remaining;
                continue;
            } else {
                // Can't skip, break
                break;
            }
        }
        
        // Try parsing HTML blocks (types 1-6, in order)
        // Type 1: Special raw content tags (script, pre, style, textarea)
        if let Ok((rest, content)) = grammar::html_special_tag(remaining) {
            let span = to_parser_span(content);
            
            let node = Node {
                kind: NodeKind::HtmlBlock {
                    html: content.fragment().to_string(),
                },
                span: Some(span),
                children: Vec::new(),
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Type 2: HTML comments
        if let Ok((rest, content)) = grammar::html_comment(remaining) {
            let span = to_parser_span(content);
            
            let node = Node {
                kind: NodeKind::HtmlBlock {
                    html: content.fragment().to_string(),
                },
                span: Some(span),
                children: Vec::new(),
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Type 3: Processing instructions
        if let Ok((rest, content)) = grammar::html_processing_instruction(remaining) {
            let span = to_parser_span(content);
            
            let node = Node {
                kind: NodeKind::HtmlBlock {
                    html: content.fragment().to_string(),
                },
                span: Some(span),
                children: Vec::new(),
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Type 4: Declarations
        if let Ok((rest, content)) = grammar::html_declaration(remaining) {
            let span = to_parser_span(content);
            
            let node = Node {
                kind: NodeKind::HtmlBlock {
                    html: content.fragment().to_string(),
                },
                span: Some(span),
                children: Vec::new(),
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Type 5: CDATA sections
        if let Ok((rest, content)) = grammar::html_cdata(remaining) {
            let span = to_parser_span(content);
            
            let node = Node {
                kind: NodeKind::HtmlBlock {
                    html: content.fragment().to_string(),
                },
                span: Some(span),
                children: Vec::new(),
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Type 6: Standard block tags (div, table, etc.)
        if let Ok((rest, content)) = grammar::html_block_tag(remaining) {
            let span = to_parser_span(content);
            
            let node = Node {
                kind: NodeKind::HtmlBlock {
                    html: content.fragment().to_string(),
                },
                span: Some(span),
                children: Vec::new(),
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Type 7: Complete tags (CANNOT interrupt paragraphs)
        // Try this but it will fail if we're in the middle of paragraph text
        if let Ok((rest, content)) = grammar::html_complete_tag(remaining) {
            let span = to_parser_span(content);
            
            let node = Node {
                kind: NodeKind::HtmlBlock {
                    html: content.fragment().to_string(),
                },
                span: Some(span),
                children: Vec::new(),
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing heading
        if let Ok((rest, (level, content))) = grammar::heading(remaining) {
            let span = to_parser_span(content);
            let text = content.fragment().to_string();
            
            let node = Node {
                kind: NodeKind::Heading { level, text },
                span: Some(span),
                children: Vec::new(),
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing fenced code block
        if let Ok((rest, (language, content))) = grammar::fenced_code_block(remaining) {
            let span = to_parser_span(content);
            let code = content.fragment().to_string();
            
            let node = Node {
                kind: NodeKind::CodeBlock { language, code },
                span: Some(span),
                children: Vec::new(),
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing thematic break (---, ***, ___)
        if let Ok((rest, content)) = grammar::thematic_break(remaining) {
            let span = to_parser_span(content);
            
            let node = Node {
                kind: NodeKind::ThematicBreak,
                span: Some(span),
                children: Vec::new(),
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing block quote (lines starting with >)
        if let Ok((rest, content)) = grammar::blockquote(remaining) {
            let span = to_parser_span(content);
            
            // Extract the block quote content (remove leading > markers)
            // CRITICAL: Per CommonMark spec, "The setext heading underline cannot be a lazy continuation line"
            // So we need to track which lines had > markers and prevent setext matching on lazy lines
            let content_str = content.fragment();
            let mut cleaned_content = String::with_capacity(content_str.len());
            
            for line in content_str.split_inclusive('\n') {
                let line_trimmed_start = line.trim_start();
                let has_marker = line_trimmed_start.starts_with('>');
                
                if has_marker {
                    // Line has > marker - remove it and optional space
                    let after_marker = line_trimmed_start.strip_prefix('>').unwrap();
                    let cleaned = after_marker.strip_prefix(' ').unwrap_or(after_marker);
                    cleaned_content.push_str(cleaned);
                } else {
                    // Lazy continuation line - no > marker
                    // Check if this looks like a setext underline (all === or all ---)
                    let line_content = line_trimmed_start.trim_end();
                    let line_sans_spaces = line_content.replace([' ', '\t'], "");
                    
                    let is_underline = !line_sans_spaces.is_empty() && 
                        (line_sans_spaces.chars().all(|c| c == '=') ||
                         line_sans_spaces.chars().all(|c| c == '-'));
                    
                    if is_underline {
                        // This lazy continuation looks like setext underline
                        // Per CommonMark: "underline cannot be lazy continuation"
                        // Escape the first character to prevent setext parsing
                        if let Some(first_char) = line_content.chars().next() {
                            if first_char == '=' || first_char == '-' {
                                // Add backslash escape before first underline character
                                cleaned_content.push('\\');
                            }
                        }
                    }
                    
                    // Add the line as-is (or with escape prepended)
                    cleaned_content.push_str(line);
                }
            }
            
            // Recursively parse the block quote content
            let inner_doc = parse_blocks_internal(&cleaned_content, depth + 1)?;
            
            let node = Node {
                kind: NodeKind::Blockquote,
                span: Some(span),
                children: inner_doc.children,  // Use parsed children
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing indented code block (4 spaces or 1 tab)
        // NOTE: Must come BEFORE lists to avoid indented code being consumed as list content
        if let Ok((rest, content)) = grammar::indented_code_block(remaining) {
            let span = to_parser_span(content);
            
            // Remove indentation from the code
            let code = content.fragment().lines()
                .map(|line| {
                    line.strip_prefix("    ")
                        .or_else(|| line.strip_prefix('\t'))
                        .unwrap_or(line)
                })
                .collect::<Vec<_>>()
                .join("\n");
            
            let node = Node {
                kind: NodeKind::CodeBlock {
                    language: None, // Indented code blocks don't have language
                    code,
                },
                span: Some(span),
                children: Vec::new(),
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing list
        // NOTE: Must come BEFORE setext heading to avoid "---" being parsed as underline
        if let Ok((rest, items)) = grammar::list(remaining) {
            // Determine if tight or loose
            // A list is tight if no item has blank lines AND no blank lines between items
            let mut is_tight = true;
            for item in &items {
                if item.2 || item.3 {  // has_blank_in_item or has_blank_before_next
                    is_tight = false;
                    break;
                }
            }
            
            // Determine list type from first marker
            let (ordered, start) = match items[0].0 {
                grammar::ListMarker::Bullet(_) => (false, None),
                grammar::ListMarker::Ordered { number, .. } => (true, Some(number)),
            };
            
            // Create list node
            let list_start = items[0].1;
            let list_end = items.last().unwrap().1;
            let list_span = to_parser_span_range(list_start, list_end);
            
            let mut list_node = Node {
                kind: NodeKind::List { ordered, start, tight: is_tight },
                span: Some(list_span),
                children: Vec::new(),
            };
            
            // Parse each item's content recursively
            for (_marker, content, _has_blank_in, _has_blank_before, content_indent) in items {
                let item_span = to_parser_span(content);
                
                // Dedent the list item content before parsing
                // This allows block structures (blockquotes, code blocks, nested lists) to be recognized
                let dedented_content = dedent_list_item_content(content.fragment(), content_indent);
                
                // Parse the item's content as block elements
                let item_content = match parse_blocks_internal(&dedented_content, depth + 1) {
                    Ok(doc) => doc.children,
                    Err(e) => {
                        log::warn!("Failed to parse list item content: {}", e);
                        vec![]
                    }
                };
                
                let item_node = Node {
                    kind: NodeKind::ListItem,
                    span: Some(item_span),
                    children: item_content,
                };
                
                list_node.children.push(item_node);
            }
            
            nodes.push(list_node);
            remaining = rest;
            continue;
        }
        
        // Try parsing Setext heading (underline style: === or ---)
        // NOTE: Must come AFTER lists to avoid eating list marker patterns like "- foo\n---"
        if let Ok((rest, (level, content))) = grammar::setext_heading(remaining) {
            let span = to_parser_span(content);
            let text = content.fragment().to_string();
            
            let node = Node {
                kind: NodeKind::Heading { level, text },
                span: Some(span),
                children: Vec::new(),
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // Try parsing paragraph
        if let Ok((rest, content)) = grammar::paragraph(remaining) {
            let span = to_parser_span(content);
            
            // Parse inline elements within paragraph text
            let inline_children = match crate::parser::inline_parser::parse_inlines(content.fragment()) {
                Ok(children) => children,
                Err(e) => {
                    log::warn!("Failed to parse inline elements: {}", e);
                    // Fallback to plain text
                    vec![Node {
                        kind: NodeKind::Text(content.fragment().to_string()),
                        span: Some(span),
                        children: Vec::new(),
                    }]
                }
            };
            
            let node = Node {
                kind: NodeKind::Paragraph,
                span: Some(span),
                children: inline_children,
            };
            
            nodes.push(node);
            remaining = rest;
            continue;
        }
        
        // If nothing matched, skip one character to avoid infinite loop
        log::warn!("Could not parse block at offset {}, skipping character", remaining.location_offset());
        let skip = remaining.fragment().chars().next().map(|c| c.len_utf8()).unwrap_or(1);
        let new_fragment = &remaining.fragment()[skip..];
        remaining = GrammarSpan::new(new_fragment);
    }
    
    log::info!("Parsed {} blocks", nodes.len());
    
    let mut document = Document::new();
    document.children = nodes;
    Ok(document)
}
