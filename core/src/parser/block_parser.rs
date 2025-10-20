// Block-level parser: first stage of two-stage parsing

use super::ast::{Node, NodeKind, Document};
use crate::grammar::block as grammar;
use anyhow::Result;
use nom_locate::LocatedSpan;

type GrammarSpan<'a> = LocatedSpan<&'a str>;

// ============================================================================
// BlockContext: Track open blocks for continuation across blank lines
// ============================================================================

/// Type of block that's currently open
#[derive(Debug, Clone, PartialEq)]
enum BlockContextKind {
    /// Ordered or unordered list container
    List {
        ordered: bool,
    },
    /// Individual list item within a list
    /// content_indent: minimum spaces required for content continuation
    ListItem {
        content_indent: usize,
    },
    /// Block quote container
    /// Each line needs '>' marker OR sufficient indent for lazy continuation
    BlockQuote {
        indent: usize,
    },
}

/// Represents an open block that can accept continuation content
#[derive(Debug, Clone)]
struct BlockContext {
    kind: BlockContextKind,
    /// Whether this block is still accepting content
    /// Set to false when the block is closed (e.g., insufficient indent)
    is_open: bool,
}

impl BlockContext {
    fn new_list(ordered: bool) -> Self {
        Self {
            kind: BlockContextKind::List { ordered },
            is_open: true,
        }
    }
    
    fn new_list_item(content_indent: usize) -> Self {
        Self {
            kind: BlockContextKind::ListItem { content_indent },
            is_open: true,
        }
    }
    
    fn new_blockquote(indent: usize) -> Self {
        Self {
            kind: BlockContextKind::BlockQuote { indent },
            is_open: true,
        }
    }
    
    /// Returns the minimum indentation required for content to continue this block
    fn required_indent(&self) -> usize {
        match &self.kind {
            BlockContextKind::List { .. } => 0,  // Lists continue at any indent (items handle it)
            BlockContextKind::ListItem { content_indent } => *content_indent,
            BlockContextKind::BlockQuote { indent } => *indent,
        }
    }
}

/// Parser state tracking open block contexts
#[derive(Debug)]
struct ParserState {
    /// Stack of open blocks (outermost first, innermost last)
    blocks: Vec<BlockContext>,
}

impl ParserState {
    fn new() -> Self {
        Self {
            blocks: Vec::new(),
        }
    }
    
    /// Add a new block context to the stack
    fn push_block(&mut self, context: BlockContext) {
        log::debug!("ParserState: push {:?}", context.kind);
        self.blocks.push(context);
    }
    
    /// Remove the most recent block context
    fn pop_block(&mut self) -> Option<BlockContext> {
        let context = self.blocks.pop();
        if let Some(ref ctx) = context {
            log::debug!("ParserState: pop {:?}", ctx.kind);
        }
        context
    }
    
    /// Get the current innermost block context
    fn current_block(&self) -> Option<&BlockContext> {
        self.blocks.last()
    }
    
    /// Get the current innermost block context (mutable)
    fn current_block_mut(&mut self) -> Option<&mut BlockContext> {
        self.blocks.last_mut()
    }
    
    /// Get the required indentation for the current context
    fn current_required_indent(&self) -> usize {
        self.current_block()
            .map(|ctx| ctx.required_indent())
            .unwrap_or(0)
    }
    
    /// Check if content at given indent can continue the current block
    /// This is the CORE logic for handling blank lines with continuation
    fn can_continue_at(&self, indent: usize) -> bool {
        if let Some(context) = self.current_block() {
            if !context.is_open {
                return false;
            }
            
            match &context.kind {
                BlockContextKind::ListItem { content_indent } => {
                    // List item content must be indented at least content_indent spaces
                    // This is per CommonMark spec: continuation requires >= content_indent
                    indent >= *content_indent
                }
                BlockContextKind::List { .. } => {
                    // Lists themselves don't have continuation requirements
                    // List items handle their own continuation
                    true
                }
                BlockContextKind::BlockQuote { indent: required } => {
                    // Block quote content must maintain required indentation
                    indent >= *required
                }
            }
        } else {
            // No open context, any content starts new block
            false
        }
    }
    
    /// Close all blocks that cannot continue at the given indentation
    /// Returns the number of blocks closed
    fn close_blocks_until_indent(&mut self, indent: usize) -> usize {
        let mut closed = 0;
        
        while let Some(context) = self.current_block() {
            if context.is_open && !self.can_continue_at(indent) {
                log::debug!("ParserState: closing block due to indent {} < required {}", 
                    indent, context.required_indent());
                self.pop_block();
                closed += 1;
            } else {
                break;
            }
        }
        
        closed
    }
    
    /// Mark the current block as closed (but keep on stack for context)
    fn close_current(&mut self) {
        if let Some(context) = self.current_block_mut() {
            context.is_open = false;
            log::debug!("ParserState: marked current block closed");
        }
    }
}

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
    let mut state = ParserState::new();
    parse_blocks_internal(input, 0, &mut state)
}

// Internal parser with recursion depth limit and state tracking
fn parse_blocks_internal(input: &str, depth: usize, state: &mut ParserState) -> Result<Document> {
    // Prevent infinite recursion
    const MAX_DEPTH: usize = 100;
    if depth > MAX_DEPTH {
        log::warn!("Maximum recursion depth reached in block parser");
        return Ok(Document::new());
    }
    
    log::debug!("Block parser input: {} bytes at depth {}, state depth: {}", 
        input.len(), depth, state.blocks.len());
    
    let mut nodes = Vec::new();
    let mut document = Document::new();  // Create document early to collect references
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
        
        // ========================================================================
        // BLANK LINE HANDLING WITH CONTEXT AWARENESS (Example 307 fix)
        // ========================================================================
        // Extract the first line to check if it's blank
        let first_line_end = remaining.fragment().find('\n').unwrap_or(remaining.fragment().len());
        let first_line = &remaining.fragment()[..first_line_end];
        
        // A line is blank if it contains only whitespace (spaces, tabs)
        if first_line.trim().is_empty() {
            // Peek at the next non-blank line to determine continuation
            let peek_offset = if first_line_end < remaining.fragment().len() {
                first_line_end + 1
            } else {
                first_line_end
            };
            
            // Find the next non-blank line
            let mut next_nonblank_indent: Option<usize> = None;
            let rest_of_input = &remaining.fragment()[peek_offset..];
            
            for peek_line in rest_of_input.lines() {
                if !peek_line.trim().is_empty() {
                    // Count leading spaces (expand tabs)
                    let mut indent = 0;
                    for ch in peek_line.chars() {
                        if ch == ' ' {
                            indent += 1;
                        } else if ch == '\t' {
                            indent += 4 - (indent % 4);  // Tab to next multiple of 4
                        } else {
                            break;
                        }
                    }
                    next_nonblank_indent = Some(indent);
                    break;
                }
            }
            
            // Determine if we should preserve context or close blocks
            let should_continue = if let Some(next_indent) = next_nonblank_indent {
                // Check if the next content can continue the current context
                state.can_continue_at(next_indent)
            } else {
                // No more content, close all contexts
                false
            };
            
            if should_continue {
                // Blank line continues the current block
                // Skip the blank but preserve block context
                log::debug!("Blank line: continuing context at indent {:?}", next_nonblank_indent);
                
                use nom::bytes::complete::take;
                let skip_len = if first_line_end < remaining.fragment().len() {
                    first_line_end + 1  // Include newline
                } else {
                    first_line_end
                };
                
                if let Ok((new_remaining, _)) = take::<_, _, nom::error::Error<GrammarSpan>>(skip_len as u32)(remaining) {
                    remaining = new_remaining;
                    continue;
                } else {
                    break;
                }
            } else {
                // Blank line ends the current context(s)
                // Close blocks that can't continue at the next indent
                if let Some(next_indent) = next_nonblank_indent {
                    let closed = state.close_blocks_until_indent(next_indent);
                    log::debug!("Blank line: closed {} blocks due to indent {}", closed, next_indent);
                } else {
                    // No more content, close everything
                    log::debug!("Blank line: end of input, closing all blocks");
                    while state.pop_block().is_some() {}
                }
                
                // Skip the blank line and continue parsing
                use nom::bytes::complete::take;
                let skip_len = if first_line_end < remaining.fragment().len() {
                    first_line_end + 1
                } else {
                    first_line_end
                };
                
                if let Ok((new_remaining, _)) = take::<_, _, nom::error::Error<GrammarSpan>>(skip_len as u32)(remaining) {
                    remaining = new_remaining;
                    continue;
                } else {
                    break;
                }
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
            // Note: We pass state through but blockquotes create their own sub-context
            let inner_doc = parse_blocks_internal(&cleaned_content, depth + 1, state)?;
            
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
                // Create a sub-state for list item content to track nested structures
                let mut item_state = ParserState::new();
                item_state.push_block(BlockContext::new_list_item(content_indent));
                
                let item_content = match parse_blocks_internal(&dedented_content, depth + 1, &mut item_state) {
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
        
        // Try parsing link reference definition
        // Must come BEFORE paragraph to avoid treating definitions as paragraphs
        if let Ok((rest, (label, url, title))) = grammar::link_reference_definition(remaining) {
            // Store the reference in the document
            document.references.insert(&label, url, title);
            log::debug!("Stored link reference definition: [{}]", label);
            
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
    
    // Add parsed nodes to document
    document.children = nodes;
    Ok(document)
}
