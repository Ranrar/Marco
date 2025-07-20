/// Visitor trait for block-level AST nodes
pub trait BlockAstVisitor {
    fn visit_paragraph(&mut self, _node: &BlockNode) {}
    fn visit_list(&mut self, _node: &BlockNode) {}
    fn visit_blockquote(&mut self, _node: &BlockNode) {}
    fn visit_table(&mut self, _node: &BlockNode) {}
    fn visit_math_block(&mut self, _node: &BlockNode) {}
    fn visit_custom_tag(&mut self, _node: &BlockNode) {}
    fn visit_front_matter(&mut self, _node: &BlockNode) {}
}

impl BlockNode {
    pub fn accept<V: BlockAstVisitor>(&self, visitor: &mut V) {
        match self {
            BlockNode::Paragraph { .. } => visitor.visit_paragraph(self),
            BlockNode::List { .. } => visitor.visit_list(self),
            BlockNode::BlockQuote { .. } => visitor.visit_blockquote(self),
            BlockNode::Table { .. } => visitor.visit_table(self),
            BlockNode::MathBlock { .. } => visitor.visit_math_block(self),
            BlockNode::CustomTag { .. } => visitor.visit_custom_tag(self),
            BlockNode::FrontMatter { .. } => visitor.visit_front_matter(self),
        }
    }
}
/// Block-level AST node for Markdown
#[derive(Debug, Clone, PartialEq)]
pub enum BlockNode {
    Paragraph { children: Vec<crate::logic::core::inline::types::InlineNode> },
    List { items: Vec<BlockNode>, ordered: bool },
    BlockQuote { children: Vec<BlockNode> },
    Table { header: Vec<BlockNode>, rows: Vec<Vec<BlockNode>> },
    MathBlock { text: String },
    CustomTag { name: String, children: Vec<BlockNode> },
    FrontMatter { text: String },
}

/// Parse a Markdown string into block-level AST nodes
pub fn parse_blocks(input: &str) -> Vec<BlockNode> {
    let mut blocks = Vec::new();
    let mut lines = input.lines().peekable();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Math block ($$...$$)
        if trimmed.starts_with("$$") {
            let mut math_lines = vec![trimmed.trim_start_matches("$$").to_string()];
            while let Some(next_line) = lines.peek() {
                if next_line.trim().ends_with("$$") {
                    math_lines.push(next_line.trim().trim_end_matches("$$").to_string());
                    lines.next();
                    break;
                } else {
                    math_lines.push(next_line.to_string());
                    lines.next();
                }
            }
            blocks.push(BlockNode::MathBlock { text: math_lines.join("\n") });
            continue;
        }
        // Table (pipe and header separator)
        if trimmed.contains('|') {
            let mut table_lines = vec![trimmed.to_string()];
            while let Some(next_line) = lines.peek() {
                if next_line.contains('|') {
                    table_lines.push(next_line.to_string());
                    lines.next();
                } else {
                    break;
                }
            }
            // Simple table parsing: first line is header, second is separator, rest are rows
            if table_lines.len() >= 2 && table_lines[1].contains("-") {
                let header_cells = table_lines[0].split('|').map(|cell| BlockNode::Paragraph {
                    children: crate::logic::core::inline::parser::parse_phrases(cell.trim())
                }).collect();
                let rows = table_lines[2..].iter().map(|row| {
                    row.split('|').map(|cell| BlockNode::Paragraph {
                        children: crate::logic::core::inline::parser::parse_phrases(cell.trim())
                    }).collect::<Vec<_>>()
                }).collect::<Vec<_>>();
                blocks.push(BlockNode::Table { header: header_cells, rows });
                continue;
            }
        }
        // Custom tag (:::tag ... :::)
        if trimmed.starts_with(":::") {
            let tag_name = trimmed.trim_start_matches(":::").split_whitespace().next().unwrap_or("").to_string();
            let mut tag_lines = Vec::new();
            while let Some(next_line) = lines.peek() {
                if next_line.trim() == ":::" {
                    lines.next();
                    break;
                } else {
                    tag_lines.push(next_line.to_string());
                    lines.next();
                }
            }
            let children = parse_blocks(&tag_lines.join("\n"));
            blocks.push(BlockNode::CustomTag { name: tag_name, children });
            continue;
        }
        // Blockquote
        if trimmed.starts_with('>') {
            let mut quote_lines = vec![trimmed.trim_start_matches('>').trim().to_string()];
            while let Some(next_line) = lines.peek() {
                if next_line.trim().starts_with('>') {
                    quote_lines.push(next_line.trim().trim_start_matches('>').trim().to_string());
                    lines.next();
                } else {
                    break;
                }
            }
            let children = parse_blocks(&quote_lines.join("\n"));
            blocks.push(BlockNode::BlockQuote { children });
            continue;
        }
        // List (unordered)
        if trimmed.starts_with('-') || trimmed.starts_with('*') || trimmed.starts_with('+') {
            let mut items = Vec::new();
            let mut item_lines = vec![trimmed[1..].trim().to_string()];
            while let Some(next_line) = lines.peek() {
                let next_trimmed = next_line.trim();
                if next_trimmed.starts_with('-') || next_trimmed.starts_with('*') || next_trimmed.starts_with('+') {
                    item_lines.push(next_trimmed[1..].trim().to_string());
                    lines.next();
                } else {
                    break;
                }
            }
            for item in item_lines {
                let children = crate::logic::core::inline::parser::parse_phrases(&item);
                items.push(BlockNode::Paragraph { children });
            }
            blocks.push(BlockNode::List { items, ordered: false });
            continue;
        }
        // Paragraph (default)
        let children = crate::logic::core::inline::parser::parse_phrases(trimmed);
        blocks.push(BlockNode::Paragraph { children });
    }
    blocks
}

#[cfg(test)]
mod tests {
    struct TestVisitor {
        pub seen: Vec<&'static str>,
    }

    impl BlockAstVisitor for TestVisitor {
        fn visit_paragraph(&mut self, _node: &BlockNode) { self.seen.push("paragraph"); }
        fn visit_list(&mut self, _node: &BlockNode) { self.seen.push("list"); }
        fn visit_blockquote(&mut self, _node: &BlockNode) { self.seen.push("blockquote"); }
        fn visit_table(&mut self, _node: &BlockNode) { self.seen.push("table"); }
        fn visit_math_block(&mut self, _node: &BlockNode) { self.seen.push("mathblock"); }
        fn visit_custom_tag(&mut self, _node: &BlockNode) { self.seen.push("customtag"); }
        fn visit_front_matter(&mut self, _node: &BlockNode) { self.seen.push("frontmatter"); }
    }

    #[test]
    fn test_block_visitor_dispatch() {
        let blocks = parse_blocks("A | B | C\n---|---|---\n1 | 2 | 3\n4 | 5 | 6\n$$\nmath\n$$\n:::tag\ncontent\n:::\n> quote\n- item\nHello");
        let mut visitor = TestVisitor { seen: Vec::new() };
        for block in &blocks {
            block.accept(&mut visitor);
        }
        // Should see all block types
        assert!(visitor.seen.contains(&"table"));
        assert!(visitor.seen.contains(&"mathblock"));
        assert!(visitor.seen.contains(&"customtag"));
        assert!(visitor.seen.contains(&"blockquote"));
        assert!(visitor.seen.contains(&"list"));
        assert!(visitor.seen.contains(&"paragraph"));
    }
    #[test]
    fn test_table_block() {
        let blocks = parse_blocks("A | B | C\n---|---|---\n1 | 2 | 3\n4 | 5 | 6");
        assert!(matches!(&blocks[0], BlockNode::Table { .. }));
        if let BlockNode::Table { header, rows } = &blocks[0] {
            assert_eq!(header.len(), 3);
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].len(), 3);
        }
    }

    #[test]
    fn test_math_block() {
        let blocks = parse_blocks("$$\nE = mc^2\n$$");
        assert!(matches!(&blocks[0], BlockNode::MathBlock { .. }));
        if let BlockNode::MathBlock { text } = &blocks[0] {
            assert!(text.contains("E = mc^2"));
        }
    }

    #[test]
    fn test_custom_tag_block() {
        let blocks = parse_blocks(":::note\nThis is a note.\n:::");
        assert!(matches!(&blocks[0], BlockNode::CustomTag { .. }));
        if let BlockNode::CustomTag { name, children } = &blocks[0] {
            assert_eq!(name, "note");
            assert!(children.iter().any(|b| matches!(b, BlockNode::Paragraph { .. })));
        }
    }
    use super::*;
    #[test]
    fn test_paragraph_block() {
        let blocks = parse_blocks("Hello **world**!");
        assert!(matches!(&blocks[0], BlockNode::Paragraph { .. }));
    }

    #[test]
    fn test_list_block() {
        let blocks = parse_blocks("- item one\n- item two\n- item three");
        assert!(matches!(&blocks[0], BlockNode::List { .. }));
        if let BlockNode::List { items, .. } = &blocks[0] {
            assert_eq!(items.len(), 3);
        }
    }

    #[test]
    fn test_blockquote_block() {
        let blocks = parse_blocks("> quoted line\n> another quote");
        assert!(matches!(&blocks[0], BlockNode::BlockQuote { .. }));
        if let BlockNode::BlockQuote { children } = &blocks[0] {
            assert!(children.iter().any(|b| matches!(b, BlockNode::Paragraph { .. })));
        }
    }

    #[test]
    fn test_mixed_blocks() {
        let blocks = parse_blocks("Hello\n\n- item\n\n> quote");
        assert!(matches!(&blocks[0], BlockNode::Paragraph { .. }));
        assert!(blocks.iter().any(|b| matches!(b, BlockNode::List { .. })));
        assert!(blocks.iter().any(|b| matches!(b, BlockNode::BlockQuote { .. })));
    }
}
