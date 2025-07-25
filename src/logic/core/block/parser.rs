#[test]
    fn test_description_list_block() {
        // Simple description list
        let blocks = parse_blocks("Term\n: Definition");
        assert!(matches!(&blocks[0], BlockNode::DescriptionList { .. }));
        if let BlockNode::DescriptionList { items } = &blocks[0] {
            assert_eq!(items.len(), 1);
            if let BlockNode::DescriptionItem { term, details } = &items[0] {
                if let BlockNode::DescriptionTerm { children } = &**term {
                    assert!(children.iter().any(|n| matches!(n, crate::logic::core::inline::types::InlineNode::Text { .. })));
                }
                if let BlockNode::DescriptionDetails { children } = &**details {
                    assert!(children.iter().any(|b| matches!(b, BlockNode::Paragraph { .. })));
                }
            }
        }

        // Multiple items
        let blocks = parse_blocks("Term1\n: Def1\nTerm2\n: Def2");
        assert!(matches!(&blocks[0], BlockNode::DescriptionList { .. }));
        if let BlockNode::DescriptionList { items } = &blocks[0] {
            assert_eq!(items.len(), 2);
        }

        // Multiple definitions for one term
        let blocks = parse_blocks("Term\n: Def1\n: Def2");
        assert!(matches!(&blocks[0], BlockNode::DescriptionList { .. }));
        if let BlockNode::DescriptionList { items } = &blocks[0] {
            if let BlockNode::DescriptionItem { details, .. } = &items[0] {
                if let BlockNode::DescriptionDetails { children } = &**details {
                    assert_eq!(children.len(), 2);
                }
            }
        }

        // Edge case: blank line ends list
        let blocks = parse_blocks("Term\n: Def1\n\nNot in list");
        assert!(matches!(&blocks[0], BlockNode::DescriptionList { .. }));
        assert!(blocks.iter().any(|b| matches!(b, BlockNode::Paragraph { .. })));

        // Nested description list (should not panic)
        let blocks = parse_blocks("Term\n: Def1\n  Term2\n  : Def2");
        assert!(matches!(&blocks[0], BlockNode::DescriptionList { .. }));
    }
    #[test]
    fn test_alert_block() {
        // !!! syntax
        let blocks = parse_blocks("!!! note\nThis is a note.\nSecond line.");
        assert!(matches!(&blocks[0], BlockNode::Alert { kind, .. } if kind == "note"));
        if let BlockNode::Alert { kind, children } = &blocks[0] {
            assert_eq!(kind, "note");
            assert!(children.iter().any(|b| matches!(b, BlockNode::Paragraph { .. })));
        }

        // > [!KIND] syntax
        let blocks = parse_blocks("> [!warning] This is a warning.\n> Second line.");
        assert!(matches!(&blocks[0], BlockNode::Alert { kind, .. } if kind == "warning"));
        if let BlockNode::Alert { kind, children } = &blocks[0] {
            assert_eq!(kind, "warning");
            assert!(children.iter().any(|b| matches!(b, BlockNode::Paragraph { .. })));
        }

        // Edge case: blank line ends alert
        let blocks = parse_blocks("!!! tip\nFirst line.\n\nNot in alert.");
        assert!(matches!(&blocks[0], BlockNode::Alert { kind, .. } if kind == "tip"));
        assert!(blocks.iter().any(|b| matches!(b, BlockNode::Paragraph { .. })));

        // Nested alert
        let blocks = parse_blocks("!!! danger\n!!! note\nInner note.\n");
        assert!(matches!(&blocks[0], BlockNode::Alert { kind, .. } if kind == "danger"));
        if let BlockNode::Alert { children, .. } = &blocks[0] {
            assert!(children.iter().any(|b| matches!(b, BlockNode::Alert { kind, .. } if kind == "note")));
        }
    }
    #[test]
    fn test_heading_and_thematic_break() {
        // ATX headings
        let blocks = parse_blocks("# Heading 1\n## Heading 2\n### Heading 3");
        assert!(matches!(&blocks[0], BlockNode::Heading { level: 1, .. }));
        assert!(matches!(&blocks[1], BlockNode::Heading { level: 2, .. }));
        assert!(matches!(&blocks[2], BlockNode::Heading { level: 3, .. }));

        // Thematic breaks
        let blocks = parse_blocks("---\n***\n___");
        assert!(matches!(&blocks[0], BlockNode::ThematicBreak));
        assert!(matches!(&blocks[1], BlockNode::ThematicBreak));
        assert!(matches!(&blocks[2], BlockNode::ThematicBreak));
    }
/// Visitor trait for block-level AST nodes
pub trait BlockAstVisitor {
    fn visit_paragraph(&mut self, _node: &BlockNode) {}
    fn visit_list(&mut self, _node: &BlockNode) {}
    fn visit_item(&mut self, _node: &BlockNode) {}
    fn visit_blockquote(&mut self, _node: &BlockNode) {}
    fn visit_table(&mut self, _node: &BlockNode) {}
    fn visit_math_block(&mut self, _node: &BlockNode) {}
    fn visit_custom_tag(&mut self, _node: &BlockNode) {}
    fn visit_front_matter(&mut self, _node: &BlockNode) {}
    fn visit_heading(&mut self, _node: &BlockNode) {}
    fn visit_thematic_break(&mut self, _node: &BlockNode) {}
    fn visit_alert(&mut self, _node: &BlockNode) {}
    fn visit_description_list(&mut self, _node: &BlockNode) {}
    fn visit_description_item(&mut self, _node: &BlockNode) {}
    fn visit_description_term(&mut self, _node: &BlockNode) {}
    fn visit_description_details(&mut self, _node: &BlockNode) {}
}

impl BlockNode {
    pub fn accept<V: BlockAstVisitor>(&self, visitor: &mut V) {
        match self {
            BlockNode::Paragraph { .. } => visitor.visit_paragraph(self),
            BlockNode::List { .. } => visitor.visit_list(self),
            BlockNode::Item { .. } => visitor.visit_item(self),
            BlockNode::BlockQuote { .. } => visitor.visit_blockquote(self),
            BlockNode::Table { .. } => visitor.visit_table(self),
            BlockNode::MathBlock { .. } => visitor.visit_math_block(self),
            BlockNode::CustomTag { .. } => visitor.visit_custom_tag(self),
            BlockNode::FrontMatter { .. } => visitor.visit_front_matter(self),
            BlockNode::Heading { .. } => visitor.visit_heading(self),
            BlockNode::ThematicBreak => visitor.visit_thematic_break(self),
            BlockNode::Alert { .. } => visitor.visit_alert(self),
            BlockNode::DescriptionList { .. } => visitor.visit_description_list(self),
            BlockNode::DescriptionItem { .. } => visitor.visit_description_item(self),
            BlockNode::DescriptionTerm { .. } => visitor.visit_description_term(self),
            BlockNode::DescriptionDetails { .. } => visitor.visit_description_details(self),
        }
    }
}
/// Block-level AST node for Markdown
#[derive(Debug, Clone, PartialEq)]
pub enum BlockNode {
    Paragraph { children: Vec<crate::logic::core::inline::types::InlineNode> },
    List { items: Vec<BlockNode>, ordered: bool, tight: bool, delimiter: char },
    Item { children: Vec<BlockNode>, task: Option<bool> },
    BlockQuote { children: Vec<BlockNode> },
    Table { header: Vec<BlockNode>, rows: Vec<Vec<BlockNode>> },
    MathBlock { text: String },
    CustomTag { name: String, children: Vec<BlockNode> },
    FrontMatter { text: String },
    Heading { level: u8, children: Vec<crate::logic::core::inline::types::InlineNode> },
    ThematicBreak,
    Alert { kind: String, children: Vec<BlockNode> },
    DescriptionList { items: Vec<BlockNode> },
    DescriptionItem { term: Box<BlockNode>, details: Box<BlockNode> },
    DescriptionTerm { children: Vec<crate::logic::core::inline::types::InlineNode> },
    DescriptionDetails { children: Vec<BlockNode> },
}

/// Parse a Markdown string into block-level AST nodes
pub fn parse_blocks(input: &str) -> Vec<BlockNode> {
    // Description list (Pandoc/kramdown style: Term\n: Definition)
    let mut blocks = Vec::new();
    let mut lines = input.lines().peekable();
    let mut para_accum = Vec::new();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            // Blank line: flush any accumulated paragraph
            if !para_accum.is_empty() {
                let para_text = para_accum.join(" ");
                let children = crate::logic::core::inline::parser::parse_phrases(&para_text);
                blocks.push(BlockNode::Paragraph { children });
                para_accum.clear();
            }
            continue;
        }
        // Description list (Pandoc/kramdown style: Term\n: Definition)
        // Description list (Pandoc/kramdown style: Term\n: Definition)
        if let Some(next_line) = lines.peek() {
            let next_trimmed = next_line.trim();
            if next_trimmed.starts_with(": ") {
                if !para_accum.is_empty() {
                    let para_text = para_accum.join(" ");
                    let children = crate::logic::core::inline::parser::parse_phrases(&para_text);
                    blocks.push(BlockNode::Paragraph { children });
                    para_accum.clear();
                }
                let mut items = Vec::new();
                let mut current_term = trimmed.to_string();
                let mut current_defs = Vec::new();
                while let Some(next_line) = lines.peek() {
                    let next_trimmed = next_line.trim();
                    if next_trimmed.starts_with(": ") {
                        current_defs.push(next_trimmed[2..].trim().to_string());
                        lines.next();
                    } else if !next_trimmed.is_empty() && !next_trimmed.starts_with(": ") {
                        let term_node = BlockNode::DescriptionTerm {
                            children: crate::logic::core::inline::parser::parse_phrases(&current_term),
                        };
                        let details_node = BlockNode::DescriptionDetails {
                            children: current_defs.iter().map(|d| BlockNode::Paragraph {
                                children: crate::logic::core::inline::parser::parse_phrases(d),
                            }).collect(),
                        };
                        items.push(BlockNode::DescriptionItem {
                            term: Box::new(term_node),
                            details: Box::new(details_node),
                        });
                        current_term = next_trimmed.to_string();
                        current_defs.clear();
                        lines.next();
                    } else {
                        break;
                    }
                }
                if !current_defs.is_empty() {
                    let term_node = BlockNode::DescriptionTerm {
                        children: crate::logic::core::inline::parser::parse_phrases(&current_term),
                    };
                    let details_node = BlockNode::DescriptionDetails {
                        children: current_defs.iter().map(|d| BlockNode::Paragraph {
                            children: crate::logic::core::inline::parser::parse_phrases(d),
                        }).collect(),
                    };
                    items.push(BlockNode::DescriptionItem {
                        term: Box::new(term_node),
                        details: Box::new(details_node),
                    });
                }
                blocks.push(BlockNode::DescriptionList { items });
                continue;
            }
            if trimmed.starts_with("!!!") {
                if !para_accum.is_empty() {
                    let para_text = para_accum.join(" ");
                    let children = crate::logic::core::inline::parser::parse_phrases(&para_text);
                    blocks.push(BlockNode::Paragraph { children });
                    para_accum.clear();
                }
                let after = &trimmed[3..].trim_start();
                let kind = after.split_whitespace().next().unwrap_or("").to_string();
                let mut alert_lines = Vec::new();
                let first_content = after[kind.len()..].trim_start();
                if !first_content.is_empty() {
                    alert_lines.push(first_content.to_string());
                }
                while let Some(next_line) = lines.peek() {
                    let next_trimmed = next_line.trim();
                    if next_trimmed.is_empty() || next_trimmed.starts_with("> [!") {
                        break;
                    }
                    // Allow nested alerts: do not break on next_trimmed.starts_with("!!!")
                    alert_lines.push(next_line.to_string());
                    lines.next();
                }
                let children = parse_blocks(&alert_lines.join("\n"));
                blocks.push(BlockNode::Alert { kind, children });
                continue;
            }
        }
        // Alert block (> [!KIND])
        if trimmed.starts_with("> [!") {
            if !para_accum.is_empty() {
                let para_text = para_accum.join(" ");
                let children = crate::logic::core::inline::parser::parse_phrases(&para_text);
                blocks.push(BlockNode::Paragraph { children });
                para_accum.clear();
            }
            let after = &trimmed[2..].trim_start();
            let re = regex::Regex::new(r"^\[!([a-zA-Z0-9_-]+)\](.*)").unwrap();
            if let Some(caps) = re.captures(after) {
                let kind = caps.get(1).map_or("", |m| m.as_str()).to_string();
                let first_content = caps.get(2).map_or("", |m| m.as_str()).trim_start();
                let mut alert_lines = Vec::new();
                if !first_content.is_empty() {
                    alert_lines.push(first_content.to_string());
                }
                // Collect subsequent '> ' lines
                while let Some(next_line) = lines.peek() {
                    let next_trimmed = next_line.trim();
                    if next_trimmed.starts_with('>') {
                        let content = next_trimmed.trim_start_matches('>').trim_start();
                        alert_lines.push(content.to_string());
                        lines.next();
                    } else {
                        break;
                    }
                }
                let children = parse_blocks(&alert_lines.join("\n"));
                blocks.push(BlockNode::Alert { kind, children });
                continue;
            }
        }
        if trimmed.starts_with('#') {
            if !para_accum.is_empty() {
                let para_text = para_accum.join(" ");
                let children = crate::logic::core::inline::parser::parse_phrases(&para_text);
                blocks.push(BlockNode::Paragraph { children });
                para_accum.clear();
            }
            let hashes = trimmed.chars().take_while(|&c| c == '#').count();
            if hashes > 0 && hashes <= 6 && trimmed.chars().nth(hashes).map_or(false, |c| c.is_whitespace()) {
                let text = trimmed[hashes..].trim();
                let children = crate::logic::core::inline::parser::parse_phrases(text);
                blocks.push(BlockNode::Heading { level: hashes as u8, children });
                continue;
            }
        }
        // Thematic break (---, ***, ___)
        let is_thematic = {
            let s = trimmed.trim_matches(|c: char| c == ' ' || c == '\t');
            (s.starts_with("---") || s.starts_with("***") || s.starts_with("___")) && s.chars().all(|c| c == '-' || c == '*' || c == '_' || c.is_whitespace())
        };
        if is_thematic {
            if !para_accum.is_empty() {
                let para_text = para_accum.join(" ");
                let children = crate::logic::core::inline::parser::parse_phrases(&para_text);
                blocks.push(BlockNode::Paragraph { children });
                para_accum.clear();
            }
            blocks.push(BlockNode::ThematicBreak);
            continue;
        }
        if trimmed.starts_with("$$") {
            if !para_accum.is_empty() {
                let para_text = para_accum.join(" ");
                let children = crate::logic::core::inline::parser::parse_phrases(&para_text);
                blocks.push(BlockNode::Paragraph { children });
                para_accum.clear();
            }
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
            if !para_accum.is_empty() {
                let para_text = para_accum.join(" ");
                let children = crate::logic::core::inline::parser::parse_phrases(&para_text);
                blocks.push(BlockNode::Paragraph { children });
                para_accum.clear();
            }
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
            if !para_accum.is_empty() {
                let para_text = para_accum.join(" ");
                let children = crate::logic::core::inline::parser::parse_phrases(&para_text);
                blocks.push(BlockNode::Paragraph { children });
                para_accum.clear();
            }
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
            if !para_accum.is_empty() {
                let para_text = para_accum.join(" ");
                let children = crate::logic::core::inline::parser::parse_phrases(&para_text);
                blocks.push(BlockNode::Paragraph { children });
                para_accum.clear();
            }
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
        // List (unordered, ordered, task)
        let unordered = trimmed.starts_with('-') || trimmed.starts_with('*') || trimmed.starts_with('+');
        let ordered = {
            let mut chars = trimmed.chars();
            let mut found_digit = false;
            while let Some(c) = chars.next() {
                if c.is_ascii_digit() {
                    found_digit = true;
                } else if found_digit && (c == '.' || c == ')') {
                    break;
                } else if found_digit {
                    found_digit = false;
                    break;
                } else {
                    break;
                }
            }
            found_digit && (trimmed.contains('.') || trimmed.contains(')'))
        };
        if unordered || ordered {
            if !para_accum.is_empty() {
                let para_text = para_accum.join(" ");
                let children = crate::logic::core::inline::parser::parse_phrases(&para_text);
                blocks.push(BlockNode::Paragraph { children });
                para_accum.clear();
            }
            let delimiter = if unordered {
                trimmed.chars().next().unwrap()
            } else {
                // Find the delimiter after the digits
                let mut chars = trimmed.chars();
                let mut delimiter = '.';
                while let Some(c) = chars.next() {
                    if c == '.' || c == ')' {
                        delimiter = c;
                        break;
                    }
                }
                delimiter
            };
            let mut items = Vec::new();
            let tight = true;
            // Parse consecutive list items
            let mut current_line = Some(trimmed.to_string());
            while let Some(line_content) = current_line {
                let current_trimmed = line_content.trim();
                let current = if unordered {
                    current_trimmed[1..].trim()
                } else {
                    // Skip digits and delimiter
                    let mut idx = 0;
                    for (i, c) in current_trimmed.char_indices() {
                        if c == '.' || c == ')' {
                            idx = i + 1;
                            break;
                        }
                    }
                    &current_trimmed[idx..].trim()
                };
                // Task list detection: [ ] or [x] at start
                let (task, content) = if current.starts_with("[ ] ") {
                    (Some(false), &current[4..])
                } else if current.starts_with("[x] ") || current.starts_with("[X] ") {
                    (Some(true), &current[4..])
                } else {
                    (None, current)
                };
                let children = vec![BlockNode::Paragraph {
                    children: crate::logic::core::inline::parser::parse_phrases(content)
                }];
                items.push(BlockNode::Item { children, task });
                // Peek next line to see if it's another item
                if let Some(next_line) = lines.peek() {
                    let next_trimmed = next_line.trim();
                    let next_unordered = next_trimmed.starts_with('-') || next_trimmed.starts_with('*') || next_trimmed.starts_with('+');
                    let next_ordered = {
                        let mut chars = next_trimmed.chars();
                        let mut found_digit = false;
                        while let Some(c) = chars.next() {
                            if c.is_ascii_digit() {
                                found_digit = true;
                            } else if found_digit && (c == '.' || c == ')') {
                                break;
                            } else if found_digit {
                                found_digit = false;
                                break;
                            } else {
                                break;
                            }
                        }
                        found_digit && (next_trimmed.contains('.') || next_trimmed.contains(')'))
                    };
                    if (unordered && next_unordered) || (ordered && next_ordered) {
                        current_line = lines.next().map(|s| s.to_string());
                        continue;
                    }
                }
                break;
            }
            blocks.push(BlockNode::List { items, ordered, tight, delimiter });
            continue;
        }
        // Paragraph (default): accumulate lines
        para_accum.push(trimmed.to_string());
    }
    // Flush any remaining paragraph at end
    if !para_accum.is_empty() {
        let para_text = para_accum.join(" ");
        let children = crate::logic::core::inline::parser::parse_phrases(&para_text);
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
        fn visit_item(&mut self, _node: &BlockNode) { self.seen.push("item"); }
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
        // Unordered list
        let blocks = parse_blocks("- item one\n- item two\n- item three");
        assert!(matches!(&blocks[0], BlockNode::List { .. }));
        if let BlockNode::List { items, ordered, .. } = &blocks[0] {
            assert_eq!(items.len(), 3);
            assert!(!ordered);
            for item in items {
                assert!(matches!(item, BlockNode::Item { .. }));
            }
        }

        // Ordered list
        let blocks = parse_blocks("1. first\n2. second\n3. third");
        assert!(matches!(&blocks[0], BlockNode::List { .. }));
        if let BlockNode::List { items, ordered, .. } = &blocks[0] {
            assert_eq!(items.len(), 3);
            assert!(*ordered);
            for item in items {
                assert!(matches!(item, BlockNode::Item { .. }));
            }
        }

        // Task list (unordered)
        let blocks = parse_blocks("- [x] done\n- [ ] not done");
        assert!(matches!(&blocks[0], BlockNode::List { .. }));
        if let BlockNode::List { items, .. } = &blocks[0] {
            assert_eq!(items.len(), 2);
            if let BlockNode::Item { task, .. } = &items[0] {
                assert_eq!(*task, Some(true));
            }
            if let BlockNode::Item { task, .. } = &items[1] {
                assert_eq!(*task, Some(false));
            }
        }

        // Task list (ordered)
        let blocks = parse_blocks("1. [x] checked\n2. [ ] not checked");
        assert!(matches!(&blocks[0], BlockNode::List { .. }));
        if let BlockNode::List { items, ordered, .. } = &blocks[0] {
            assert_eq!(items.len(), 2);
            assert!(*ordered);
            if let BlockNode::Item { task, .. } = &items[0] {
                assert_eq!(*task, Some(true));
            }
            if let BlockNode::Item { task, .. } = &items[1] {
                assert_eq!(*task, Some(false));
            }
        }

        // Nested list
        // (Simple test: parser currently does not support true nested lists, but should not panic)
        let blocks = parse_blocks("- item one\n  - subitem\n- item two");
        assert!(matches!(&blocks[0], BlockNode::List { .. }));
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
