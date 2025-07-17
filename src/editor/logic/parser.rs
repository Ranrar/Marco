use crate::editor::logic::ast::blocks_and_inlines::LeafBlock;
use crate::editor::logic::ast::preliminaries::{Line, LineEnding};

/// Preprocess input according to CommonMark Section 2 (Preliminaries):
/// - Normalize line endings and split into lines (2.1)
/// - Expand tabs (2.2)
/// - Replace insecure characters (2.3)
/// - Handle backslash escapes (2.4)
/// - Convert entity/numeric references (2.5)
pub fn preprocess_input(input: &str) -> Vec<Line> {
    let lines = Vec::new();
    let _current: Vec<Line> = Vec::new();
    let _ending: Option<LineEnding> = None;
    let mut chars = input.chars().peekable();
    while chars.next().is_some() {
        // ...existing code...
    }
    // TODO: Backslash escapes and entity/numeric references are handled in inline parsing, not here.
    // ...existing code...
    lines
}
// ...existing code...

// ...existing code...

#[cfg(test)]
mod tests {
    #[test]
    fn test_list_container_bullet() {
        let input = "- one\n- two\n- three\n";
        let root = BlockParser::parse_markdown(input);
        let lists: Vec<_> = root.children.iter().filter(|n| matches!(&n.block, Block::Container(ContainerBlock::List(_)))).collect();
        assert_eq!(lists.len(), 1);
        let list = &lists[0];
        if let Block::Container(ContainerBlock::List(items)) = &list.block {
            assert_eq!(items.len(), 3);
            for (i, item) in items.iter().enumerate() {
                if let ContainerBlock::ListItem(blocks) = item {
                    let para = &blocks[0];
                    if let Block::Leaf(LeafBlock::Paragraph(_)) = para {
                        let para_node = &list.children[i].children[0];
                        let expected = match i {
                            0 => "one",
                            1 => "two",
                            2 => "three",
                            _ => unreachable!(),
                        };
                        assert_eq!(para_node.text_lines.join(" "), expected);
                    } else {
                        panic!("Expected Paragraph in ListItem");
                    }
                } else {
                    panic!("Expected ListItem");
                }
            }
        } else {
            panic!("Expected List container");
        }
    }

    #[test]
    fn test_list_container_ordered() {
        let input = "1. first\n2. second\n3. third\n";
        let root = BlockParser::parse_markdown(input);
        let lists: Vec<_> = root.children.iter().filter(|n| matches!(&n.block, Block::Container(ContainerBlock::List(_)))).collect();
        assert_eq!(lists.len(), 1);
        let list = &lists[0];
        if let Block::Container(ContainerBlock::List(items)) = &list.block {
            assert_eq!(items.len(), 3);
            for (i, item) in items.iter().enumerate() {
                if let ContainerBlock::ListItem(blocks) = item {
                    let para = &blocks[0];
                    if let Block::Leaf(LeafBlock::Paragraph(_)) = para {
                        let para_node = &list.children[i].children[0];
                        let expected = match i {
                            0 => "first",
                            1 => "second",
                            2 => "third",
                            _ => unreachable!(),
                        };
                        assert_eq!(para_node.text_lines.join(" "), expected);
                    } else {
                        panic!("Expected Paragraph in ListItem");
                    }
                } else {
                    panic!("Expected ListItem");
                }
            }
        } else {
            panic!("Expected List container");
        }
    }
    #[test]
    fn test_list_item_bullet() {
        let input = "- item one\n- item two\n";
        let root = BlockParser::parse_markdown(input);
        let lists: Vec<_> = root.children.iter().filter(|n| matches!(&n.block, Block::Container(ContainerBlock::List(_)))).collect();
        assert_eq!(lists.len(), 1);
        let list = &lists[0];
        if let Block::Container(ContainerBlock::List(items)) = &list.block {
            assert_eq!(items.len(), 2);
            for (i, item) in items.iter().enumerate() {
                if let ContainerBlock::ListItem(blocks) = item {
                    let para = &blocks[0];
                    if let Block::Leaf(LeafBlock::Paragraph(_)) = para {
                        let para_node = &list.children[i].children[0];
                        let expected = match i {
                            0 => "item one",
                            1 => "item two",
                            _ => unreachable!(),
                        };
                        assert_eq!(para_node.text_lines.join(" "), expected);
                    } else {
                        panic!("Expected Paragraph in ListItem");
                    }
                } else {
                    panic!("Expected ListItem");
                }
            }
        } else {
            panic!("Expected List container");
        }
    }

    #[test]
    fn test_list_item_ordered() {
        let input = "1. first\n2. second\n";
        let root = BlockParser::parse_markdown(input);
        let lists: Vec<_> = root.children.iter().filter(|n| matches!(&n.block, Block::Container(ContainerBlock::List(_)))).collect();
        assert_eq!(lists.len(), 1);
        let list = &lists[0];
        if let Block::Container(ContainerBlock::List(items)) = &list.block {
            assert_eq!(items.len(), 2);
            for (i, item) in items.iter().enumerate() {
                if let ContainerBlock::ListItem(blocks) = item {
                    let para = &blocks[0];
                    if let Block::Leaf(LeafBlock::Paragraph(_)) = para {
                        let para_node = &list.children[i].children[0];
                        let expected = match i {
                            0 => "first",
                            1 => "second",
                            _ => unreachable!(),
                        };
                        assert_eq!(para_node.text_lines.join(" "), expected);
                    } else {
                        panic!("Expected Paragraph in ListItem");
                    }
                } else {
                    panic!("Expected ListItem");
                }
            }
        } else {
            panic!("Expected List container");
        }
    }
    #[test]
    fn test_block_quote_basic() {
        let input = "> foo\n> bar\n> baz\n";
        let root = BlockParser::parse_markdown(input);
        let quotes: Vec<_> = root.children.iter().filter(|n| matches!(&n.block, Block::Container(ContainerBlock::BlockQuote(_)))).collect();
        assert_eq!(quotes.len(), 1);
        let para = &quotes[0].children[0];
        assert!(matches!(&para.block, Block::Leaf(LeafBlock::Paragraph(_))));
        assert_eq!(para.text_lines.join(" "), "foo bar baz");
    }

    #[test]
    fn test_block_quote_nested() {
        let input = "> > foo\n> > bar\n> baz\n";
        let root = BlockParser::parse_markdown(input);
        let quotes: Vec<_> = root.children.iter().filter(|n| matches!(&n.block, Block::Container(ContainerBlock::BlockQuote(_)))).collect();
        assert_eq!(quotes.len(), 1);
        let inner = &quotes[0].children[0];
        assert!(matches!(&inner.block, Block::Container(ContainerBlock::BlockQuote(_))));
        let para = &inner.children[0];
        assert!(matches!(&para.block, Block::Leaf(LeafBlock::Paragraph(_))));
        assert_eq!(para.text_lines.join(" "), "foo bar");
        let para2 = &quotes[0].children[1];
        assert!(matches!(&para2.block, Block::Leaf(LeafBlock::Paragraph(_))));
        assert_eq!(para2.text_lines.join(" "), "baz");
    }

    #[test]
    fn test_block_quote_lazy_continuation() {
        let input = "> foo\nbar\n> baz\n";
        let root = BlockParser::parse_markdown(input);
        let quotes: Vec<_> = root.children.iter().filter(|n| matches!(&n.block, Block::Container(ContainerBlock::BlockQuote(_)))).collect();
        assert_eq!(quotes.len(), 1);
        let para = &quotes[0].children[0];
        assert!(matches!(&para.block, Block::Leaf(LeafBlock::Paragraph(_))));
        assert_eq!(para.text_lines.join(" "), "foo bar baz");
    }

    #[test]
    fn test_block_quote_consecutive() {
        let input = "> foo\n\n> bar\n";
        let root = BlockParser::parse_markdown(input);
        let quotes: Vec<_> = root.children.iter().filter(|n| matches!(&n.block, Block::Container(ContainerBlock::BlockQuote(_)))).collect();
        assert_eq!(quotes.len(), 2);
        let para1 = &quotes[0].children[0];
        let para2 = &quotes[1].children[0];
        assert!(matches!(&para1.block, Block::Leaf(LeafBlock::Paragraph(_))));
        assert!(matches!(&para2.block, Block::Leaf(LeafBlock::Paragraph(_))));
        assert_eq!(para1.text_lines.join(" "), "foo");
        assert_eq!(para2.text_lines.join(" "), "bar");
    }
    use super::*;

    #[test]
    fn test_preprocessing_section2() {
        let input = "foo\r\nbar\t\0baz\n";
        let lines = preprocess_input(input);
        // Should normalize CRLF and LF, expand tab, and replace NUL
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].chars.iter().map(|c| c.codepoint).collect::<String>(), "foobar    �baz");
        assert_eq!(lines[0].ending, Some(LineEnding::CarriageReturnLineFeed));
        assert_eq!(lines[1].ending, None);
    }

    #[test]
    fn test_basic_block_parsing() {
        let input = "> Blockquote\n\nParagraph text\n\n> - List item 1\n\n> - List item 2\n";
        let _root = BlockParser::parse_markdown(input);
        dbg!(&_root);
        // This is a smoke test: just check the root is present and open
        assert!(_root.open);
        // TODO: Add more detailed assertions as block parsing is implemented
    }

    #[test]
    fn test_thematic_break() {
        let input = "***\n---\n___\n";
        let root = BlockParser::parse_markdown(input);
        let thematic_breaks: Vec<_> = root.children.iter().filter(|n| matches!(&n.block, Block::Leaf(LeafBlock::ThematicBreak))).collect();
        assert_eq!(thematic_breaks.len(), 3);
    }

    #[test]
    fn test_atx_heading() {
        let input = "# Heading 1\n## Heading 2\n###### Heading 6\n";
        let root = BlockParser::parse_markdown(input);
        let headings: Vec<_> = root.children.iter().filter(|n| matches!(&n.block, Block::Leaf(LeafBlock::Heading { .. }))).collect();
        assert_eq!(headings.len(), 3);
    }

    #[test]
    fn test_setext_heading() {
        let input = "Heading 1\n====\nHeading 2\n----\n";
        let root = BlockParser::parse_markdown(input);
        let setexts: Vec<_> = root.children.iter().filter(|n| matches!(&n.block, Block::Leaf(LeafBlock::Heading { .. }))).collect();
        assert_eq!(setexts.len(), 2);
    }

    #[test]
    fn test_indented_code_block() {
        let input = "    code block\n\tcode block tab\n";
        let root = BlockParser::parse_markdown(input);
        let codes: Vec<_> = root.children.iter().filter(|n| matches!(&n.block, Block::Leaf(LeafBlock::CodeBlock(_)))).collect();
        assert_eq!(codes.len(), 2);
    }

    #[test]
    fn test_fenced_code_block() {
        let input = "```rust\nfn main() {}\n```\n~~~\ncode\n~~~\n";
        let root = BlockParser::parse_markdown(input);
        let codes: Vec<_> = root.children.iter().filter(|n| matches!(&n.block, Block::Leaf(LeafBlock::CodeBlock(_)))).collect();
        assert_eq!(codes.len(), 2);
    }

    #[test]
    fn test_html_block() {
        let input = "<div>\n<p>HTML</p>\n</div>\n";
        let root = BlockParser::parse_markdown(input);
        let htmls: Vec<_> = root.children.iter().filter(|n| matches!(&n.block, Block::Leaf(LeafBlock::HtmlBlock(_)))).collect();
        assert_eq!(htmls.len(), 1);
    }

    #[test]
    fn test_link_reference_definition() {
        let input = "[foo]: /url 'title'\n[bar]: /url2\n";
        let root = BlockParser::parse_markdown(input);
        // Skipping LinkReferenceDefinition test as it's not in canonical AST
        // let refs: Vec<_> = root.children.iter().filter(|n| matches!(&n.block, Block::Leaf(LeafBlock::LinkReferenceDefinition(_)))).collect();
        // assert_eq!(refs.len(), 2);
    }

    #[test]
    fn test_paragraphs() {
        let input = "Paragraph one.\n\nParagraph two.\n";
        let root = BlockParser::parse_markdown(input);
        let paras: Vec<_> = root.children.iter().filter(|n| matches!(&n.block, Block::Leaf(LeafBlock::Paragraph(_)))).collect();
        assert_eq!(paras.len(), 2);
    }

    #[test]
    fn test_blank_lines() {
        let input = "\n\n\n";
        let root = BlockParser::parse_markdown(input);
        // Skipping BlankLine test as it's not in canonical AST
        // let blanks: Vec<_> = root.children.iter().filter(|n| matches!(&n.block, Block::Leaf(LeafBlock::BlankLine))).collect();
        // assert_eq!(blanks.len(), 3);
    }
}
pub struct BlockParser {
    /// Stack of open blocks (from root to deepest open block).
    open_blocks: Vec<BlockNode>,
    // Map of link reference definitions (label -> destination/title), built when paragraphs are closed.
    // TODO: Implement actual map type and logic for reference link definitions.
    // link_reference_map: HashMap<String, (String, Option<String>)>,
}

impl BlockParser {

/// Parse a Markdown string into a block tree (AST) using the block parsing phase.
pub fn parse_markdown(input: &str) -> BlockNode {
    // Section 2: Preliminaries
    let lines = preprocess_input(input);
    let mut parser = BlockParser::new();
    for line in lines {
        // TODO: Convert Line struct to &str or similar for process_line, or refactor process_line to accept Line
        // For now, join chars for a simple prototype
        let line_str: String = line.chars.iter().map(|c| c.codepoint).collect();
        parser.process_line(&line_str);
    }
    parser.finalize();
    // Return the root block node (document)
    parser.open_blocks.remove(0)
}
    /// Create a new parser with a root document block.
    pub fn new() -> Self {
        let root = BlockNode::new_container(Block::Container(ContainerBlock::BlockQuote(vec![]))); // Placeholder root
        BlockParser {
            open_blocks: vec![root],
        }
    }

    /// Process a single line of input, updating the block tree.
    pub fn process_line(&mut self, line: &str) {
        let trimmed = line.trim();
        // 4.6 HTML blocks (basic: lines starting with <)
        if trimmed.starts_with('<') {
            let block = Block::Leaf(LeafBlock::HtmlBlock(trimmed.to_string()));
            let node = BlockNode::new_leaf(block);
            self.open_blocks.push(node);
            dbg!("opened HTML block");
            return;
        }
        // 4.5/4.4 Code blocks (fenced or indented)
        let is_fenced_code = {
            let s = trimmed;
            let fence_char = s.chars().next().unwrap_or(' ');
            (fence_char == '`' || fence_char == '~') && s.chars().take_while(|&c| c == fence_char).count() >= 3
        };
        if is_fenced_code {
            let s = trimmed;
            let block = Block::Leaf(LeafBlock::CodeBlock(s.to_string()));
            let node = BlockNode::new_leaf(block);
            self.open_blocks.push(node);
            dbg!("opened code block");
            return;
        }
        if line.starts_with("    ") || line.starts_with('\t') {
            let content = line.trim_start_matches(' ').trim_start_matches('\t').to_string();
            let block = Block::Leaf(LeafBlock::CodeBlock(content));
            let node = BlockNode::new_leaf(block);
            self.open_blocks.push(node);
            dbg!("opened code block");
            return;
        }
        // 4.3/4.2 Headings
        let is_setext_underline = {
            let s = trimmed.trim();
            if s.is_empty() { false }
            else {
                let first = s.chars().next().unwrap();
                (first == '=' || first == '-') && s.chars().all(|c| c == first)
            }
        };
        if is_setext_underline {
            if let Some(last) = self.open_blocks.iter_mut().rev().find(|b| matches!(&b.block, Block::Leaf(LeafBlock::Paragraph(_)))) {
                let level = if trimmed.contains('=') { 1 } else { 2 };
                last.block = Block::Leaf(LeafBlock::Heading { level, content: Vec::new() });
                last.text_lines.clear();
                dbg!("converted paragraph to heading");
                return;
            }
        }
        let is_atx_heading = {
            let s = trimmed;
            let hashes = s.chars().take_while(|&c| c == '#').count();
            hashes >= 1 && hashes <= 6 && s[hashes..].starts_with(' ')
        };
        if is_atx_heading {
            let s = trimmed;
            let hashes = s.chars().take_while(|&c| c == '#').count();
            let block = Block::Leaf(LeafBlock::Heading { level: hashes as u8, content: Vec::new() });
            let node = BlockNode::new_leaf(block);
            self.open_blocks.push(node);
            dbg!("opened heading");
            return;
        }
        // Step 1: Walk open blocks from root, checking if each remains open for this line.
        // For now, just print the stack and the line. Real logic will check block type and line content.
        let mut last_matched = 0;
        for (i, block) in self.open_blocks.iter().enumerate() {
            dbg!(i, &block.block);
            // TODO: For each block, check if the line matches its continuation condition.
            // For now, assume all blocks match (no closing). In future, set last_matched = i if matched.
            last_matched = i;
        }

        // Step 2: Close unmatched blocks (those after last_matched)
        if last_matched + 1 < self.open_blocks.len() {
            // Mark blocks as closed and pop them, attaching as children to parent
            while self.open_blocks.len() > last_matched + 1 {
                let mut closed = self.open_blocks.pop().unwrap();
                closed.open = false;
                dbg!("closed block", &closed.block);
                // Attach closed block as child to its parent (if any)
                if let Some(parent) = self.open_blocks.last_mut() {
                    parent.children.push(closed);
                }
                // Special case: If closing a paragraph, check for reference link definitions.
                // TODO: If closed.block is a paragraph, parse text_lines for link reference definitions.
                // If found, add to link_reference_map and remove from block tree.
            }
        }

        // Step 3: Detect new block starts (block quote, paragraph, lists)
        let mut rest = line;
        let mut block_quote_depth = 0;
        // Parse block quote markers (up to 3 spaces, then '>')
        loop {
            let trimmed = rest.trim_start_matches(' ');
            if trimmed.starts_with('>') {
                block_quote_depth += 1;
                // Remove '>' and one optional following space
                rest = &trimmed[1..];
                if rest.starts_with(' ') { rest = &rest[1..]; }
            } else {
                break;
            }
        }
        // Open block quotes as needed
        let mut open_block_quotes = 0;
        for b in self.open_blocks.iter().rev() {
            if matches!(&b.block, Block::Container(ContainerBlock::BlockQuote(_))) {
                open_block_quotes += 1;
            } else {
                break;
            }
        }
        for _ in open_block_quotes..block_quote_depth {
            let block_quote = Block::Container(ContainerBlock::BlockQuote(vec![]));
            let node = BlockNode::new_container(block_quote);
            self.open_blocks.push(node);
            dbg!("opened block quote");
        }
        // If we have fewer block quotes than open, close them
        for _ in block_quote_depth..open_block_quotes {
            let mut closed = self.open_blocks.pop().unwrap();
            closed.open = false;
            if let Some(parent) = self.open_blocks.last_mut() {
                parent.children.push(closed);
            }
            dbg!("closed block quote");
        }
        let trimmed = rest.trim_start();
        // 5.2 List items (bullet or ordered)
        let mut is_list_item = false;
        let mut list_marker = None;
        let mut after_marker = trimmed;
        // Bullet list: -, +, *
        if let Some(first) = trimmed.chars().next() {
            if (first == '-' || first == '+' || first == '*') && trimmed.len() > 1 && trimmed[1..].starts_with(' ') {
                is_list_item = true;
                list_marker = Some(first.to_string());
                after_marker = trimmed[1..].trim_start();
            }
        }
        // Ordered list: 1-9 digits + '.' or ')'
        if !is_list_item {
            let mut chars = trimmed.chars();
            let mut digits = String::new();
            while let Some(c) = chars.clone().next() {
                if c.is_ascii_digit() && digits.len() < 9 {
                    digits.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            if !digits.is_empty() {
                if let Some(marker) = chars.next() {
                    if (marker == '.' || marker == ')') && chars.clone().next().map_or(false, |c| c == ' ') {
                        is_list_item = true;
                        list_marker = Some(format!("{}{}", digits, marker));
                        after_marker = chars.as_str().trim_start();
                    }
                }
            }
        }
        if is_list_item {
            // Determine list type and marker
            let (list_type, marker) = if let Some(ref m) = list_marker {
                if m == "-" || m == "+" || m == "*" {
                    ("bullet", m.clone())
                } else {
                    ("ordered", m.clone())
                }
            } else {
                ("bullet", String::from("-"))
            };
            // Check if the last open block is a List of the same type/marker
        // Find the last open List container
        let mut found_list = false;
        if let Some(last) = self.open_blocks.last_mut() {
            if let Block::Container(ContainerBlock::List(_)) = last.block {
                found_list = true;
            }
        }
        if !found_list {
            // Open a new List container (empty for now)
            let block = Block::Container(ContainerBlock::List(Vec::new()));
            let node = BlockNode::new_container(block);
            self.open_blocks.push(node);
            dbg!("opened list container");
        }
        // Open a list item block
        let block = Block::Container(ContainerBlock::ListItem(Vec::new()));
        let mut node = BlockNode::new_container(block);
        // Add the rest of the line as a paragraph inside the list item
        if !after_marker.is_empty() {
            let para = Block::Leaf(LeafBlock::Paragraph(Vec::new()));
            let mut para_node = BlockNode::new_leaf(para);
            para_node.text_lines.push(after_marker.to_string());
            node.children.push(para_node);
        }
        // Attach the list item to the last open List container
        if let Some(last) = self.open_blocks.last_mut() {
            if let Block::Container(ContainerBlock::List(ref mut items)) = last.block {
                let blocks: Vec<Block> = node.children.iter().map(|n| n.block.clone()).collect();
                items.push(ContainerBlock::ListItem(blocks));
            }
        }
        self.open_blocks.push(node);
        dbg!("opened list item");
        return;
        }
        if trimmed.is_empty() {
            // Ignore blank lines for now (no AST node)
            return;
        }
        // 4.8 Paragraphs: open if not already in one
        let already_in_paragraph = self.open_blocks.iter().any(|b| matches!(&b.block, Block::Leaf(LeafBlock::Paragraph(_))));
        if !already_in_paragraph {
            let para = Block::Leaf(LeafBlock::Paragraph(Vec::new()));
            let mut para_node = BlockNode::new_leaf(para);
            para_node.text_lines.push(trimmed.to_string());
            self.open_blocks.push(para_node);
            dbg!("opened paragraph");
        } else {
            if let Some(deepest) = self.open_blocks.last_mut() {
                deepest.text_lines.push(trimmed.to_string());
            }
        }
        dbg!("process_line", line);
    }

    /// Finalize the block tree (close all open blocks).
    pub fn finalize(&mut self) {
        // Close all open blocks, attaching as children up to the root
        while self.open_blocks.len() > 1 {
            let mut closed = self.open_blocks.pop().unwrap();
            closed.open = false;
            if let Some(parent) = self.open_blocks.last_mut() {
                parent.children.push(closed);
            }
        }
        dbg!("finalize");
    }
}
// Block parsing phase for CommonMark (Phase 1)
//
// This module implements the block structure parsing phase, following the CommonMark
// Appendix A parsing strategy. It builds a tree of BlockNode structs, which wrap the
// canonical Block AST and track open/closed state and text accumulation.

use crate::editor::logic::ast::blocks_and_inlines::{Block, ContainerBlock};

use crate::editor::logic::inline;

/// Internal node for block parsing (not part of the public AST).
/// Wraps a Block and tracks children, open/closed state, and text accumulation.
#[derive(Debug, Clone)]
pub struct BlockNode {
    /// The block type (container or leaf).
    pub block: Block,
    /// Child nodes (for container blocks).
    pub children: Vec<BlockNode>,
    /// Accumulated text lines (for leaf blocks, before inline parsing).
    pub text_lines: Vec<String>,
    /// Whether this block is open for more input.
    pub open: bool,
}

impl BlockNode {
    /// Create a new container block node (open, with no children).
    pub fn new_container(block: Block) -> Self {
        BlockNode {
            block,
            children: Vec::new(),
            text_lines: Vec::new(),
            open: true,
        }
    }

    /// Create a new leaf block node (open, with empty text).
    pub fn new_leaf(block: Block) -> Self {
        BlockNode {
            block,
            children: Vec::new(),
            text_lines: Vec::new(),
            open: true,
        }
    }
}

/// Recursively walk the block tree, parsing text_lines in paragraphs/headings into Vec<Inline>.
fn parse_inlines_in_tree(node: &mut BlockNode) {
    match &mut node.block {
        Block::Leaf(LeafBlock::Paragraph(_)) => {
            let text = node.text_lines.join("\n");
            let _inlines = inline::parse_inlines(&text);
            node.text_lines.clear();
        }
        _ => {}
    }
    for child in &mut node.children {
        parse_inlines_in_tree(child);
    }
}
