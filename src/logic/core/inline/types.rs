/// Sample visitor that prints inline node types for debugging.
pub struct InlineDebugPrinter;

impl InlineAstVisitor for InlineDebugPrinter {
    fn visit_inline_text(&mut self, node: &InlineNode) {
        if let InlineNode::Text { text, pos } = node {
            println!("Text: '{}' at {:?}", text, pos);
        }
    }
    fn visit_inline_emphasis(&mut self, node: &InlineNode) {
        println!("Emphasis: {:?}", node);
    }
    fn visit_inline_strong(&mut self, node: &InlineNode) {
        println!("Strong: {:?}", node);
    }
    fn visit_inline_code(&mut self, node: &InlineNode) {
        println!("Code: {:?}", node);
    }
    fn visit_inline_link(&mut self, node: &InlineNode) {
        println!("Link: {:?}", node);
    }
    fn visit_inline_image(&mut self, node: &InlineNode) {
        println!("Image: {:?}", node);
    }
    fn visit_inline_math(&mut self, node: &InlineNode) {
        println!("Math: {:?}", node);
    }
    fn visit_inline_html(&mut self, node: &InlineNode) {
        println!("Html: {:?}", node);
    }
    fn visit_inline_entity(&mut self, node: &InlineNode) {
        println!("Entity: {:?}", node);
    }
    fn visit_inline_attribute_block(&mut self, node: &InlineNode) {
        println!("AttributeBlock: {:?}", node);
    }
    fn visit_inline_softbreak(&mut self, node: &InlineNode) {
        println!("SoftBreak: {:?}", node);
    }
    fn visit_inline_linebreak(&mut self, node: &InlineNode) {
        println!("LineBreak: {:?}", node);
    }
}

#[cfg(test)]
mod visitor_tests {
    #[test]
    fn test_delimiter_normalization_spec_cases() {
        use crate::logic::core::inline::parser::parse_phrases;
        // Left/right-flanking detection
        let ast = parse_phrases("a *b* c");
        println!("AST: {:?}", ast);
        assert!(ast.iter().any(|n| matches!(n, InlineNode::Emphasis { children, .. } if children.iter().any(|c| matches!(c, InlineNode::Text { text, .. } if text == "b")))), "Should parse *b* as emphasis");

        // Multiples-of-3 rule and partial consumption
        let ast2 = parse_phrases("***foo** bar*");
        println!("AST2: {:?}", ast2);
        // Should produce strong and emphasis, not triple nesting
        assert!(ast2.iter().any(|n| matches!(n, InlineNode::Strong { .. })), "Should parse strong");
        assert!(ast2.iter().any(|n| matches!(n, InlineNode::Emphasis { .. })), "Should parse emphasis");
        assert!(ast2.iter().any(|n| matches!(n, InlineNode::Text { text, .. } if text.contains("*"))), "Should emit leftover delimiter as text");

        // No parsing inside code spans
        let ast3 = parse_phrases("`*not emph*`");
        assert!(ast3.iter().any(|n| matches!(n, InlineNode::Code { text, .. } if text.contains("*not emph*"))), "Should not parse emphasis inside code span");

        // No empty emphasis/strong nodes
        let ast4 = parse_phrases("** **");
        assert!(!ast4.iter().any(|n| matches!(n, InlineNode::Strong { children, .. } if children.is_empty())), "Should not emit empty strong node");

        // Overlapping and nested delimiters
        let ast5 = parse_phrases("*a **b* c**");
        // Should prefer minimal nesting and correct precedence
        assert!(ast5.iter().any(|n| matches!(n, InlineNode::Emphasis { .. })), "Should parse emphasis");
        assert!(ast5.iter().any(|n| matches!(n, InlineNode::Strong { .. })), "Should parse strong");

        // Intraword emphasis for * but not _
        let ast6 = parse_phrases("foo*bar* baz_baz_");
        assert!(ast6.iter().any(|n| matches!(n, InlineNode::Emphasis { .. })), "Should allow intraword emphasis for *");
        // _ intraword should not parse as emphasis
        let ast6b = parse_phrases("foo_bar_baz_");
        assert!(!ast6b.iter().any(|n| matches!(n, InlineNode::Emphasis { .. })), "Should not allow intraword emphasis for _");

        // Minimal nesting and precedence
        let ast7 = parse_phrases("*em **strong** em*");
        // Should prefer <em><strong>...</strong></em>
        assert!(ast7.iter().any(|n| matches!(n, InlineNode::Emphasis { children, .. } if children.iter().any(|c| matches!(c, InlineNode::Strong { .. })))), "Should nest strong inside emphasis");

        // Unmatched delimiters
        let ast8 = parse_phrases("*unclosed");
        assert!(ast8.iter().any(|n| matches!(n, InlineNode::Text { text, .. } if text.contains("*unclosed"))), "Unclosed emphasis should be text");

        // Complex overlapping
        let ast9 = parse_phrases("***foo* bar**");
        assert!(ast9.iter().any(|n| matches!(n, InlineNode::Strong { children, .. } if children.iter().any(|c| matches!(c, InlineNode::Emphasis { .. })))), "Should nest Emphasis inside Strong for ***foo* bar**");

        let ast10 = parse_phrases("**a *b***");
        assert!(ast10.iter().any(|n| matches!(n, InlineNode::Strong { children, .. } if children.iter().any(|c| matches!(c, InlineNode::Emphasis { .. })))), "Should nest Emphasis inside Strong for **a *b***");

        let ast11 = parse_phrases("*a **b* c**");
        assert!(ast11.iter().any(|n| matches!(n, InlineNode::Emphasis { children, .. } if children.iter().any(|c| matches!(c, InlineNode::Strong { .. })))), "Should nest Strong inside Emphasis for *a **b* c**");

        // Edge: only delimiters
        let ast12 = parse_phrases("***");
        assert!(ast12.iter().any(|n| matches!(n, InlineNode::Text { text, .. } if text == "***")), "Should emit unmatched triple delimiter as text");
    }
    #[test]
    fn test_edge_cases() {
        use crate::logic::core::inline::parser::parse_phrases;
        // Malformed entity (no semicolon)
        let ast = parse_phrases("foo &amp bar");
        assert!(ast.iter().any(|n| matches!(n, InlineNode::Text { text, .. } if text.contains("&amp bar"))), "Malformed entity should be text");

        // Unclosed emphasis
        let ast2 = parse_phrases("*unclosed");
        assert!(ast2.iter().any(|n| matches!(n, InlineNode::Text { text, .. } if text.contains("*unclosed"))), "Unclosed emphasis should be text");

        // Overlapping emphasis/strong
        let ast3 = parse_phrases("**bold *italic** text*");
        // Should not panic, should produce valid AST
        assert!(!ast3.is_empty(), "AST should not be empty for overlapping delimiters");

        // Attribute block with no node to attach
        let ast4 = parse_phrases("{.lonely}");
        assert!(ast4.iter().any(|n| matches!(n, InlineNode::AttributeBlock { text, .. } if text == ".lonely")), "Lone attribute block should be emitted");

        // Attribute block with malformed braces
        let ast5 = parse_phrases("foo {.bad");
        assert!(ast5.iter().any(|n| matches!(n, InlineNode::Text { text, .. } if text.contains("{.bad"))), "Malformed attribute block should be text");
    }
    use super::*;

    #[test]
    fn test_inline_debug_printer_traversal() {
        let ast = vec![
            InlineNode::Text { text: "Hello".into(), pos: SourcePos { line: 1, column: 1 } },
            InlineNode::Emphasis {
                children: vec![InlineNode::Text { text: "emph".into(), pos: SourcePos { line: 1, column: 7 } }],
                pos: SourcePos { line: 1, column: 6 }
            },
            InlineNode::Strong {
                children: vec![InlineNode::Text { text: "strong".into(), pos: SourcePos { line: 1, column: 12 } }],
                pos: SourcePos { line: 1, column: 11 }
            },
            InlineNode::Code { text: "code".into(), pos: SourcePos { line: 1, column: 18 } },
            InlineNode::Math { text: "math".into(), pos: SourcePos { line: 1, column: 24 } },
            InlineNode::Html { text: "<b>html</b>".into(), pos: SourcePos { line: 1, column: 30 } },
            InlineNode::Entity { text: "&amp;".into(), pos: SourcePos { line: 1, column: 40 } },
            InlineNode::AttributeBlock { text: ".class #id".into(), pos: SourcePos { line: 1, column: 50 } },
            InlineNode::SoftBreak { pos: SourcePos { line: 2, column: 1 } },
            InlineNode::LineBreak { pos: SourcePos { line: 2, column: 2 } },
        ];
        let mut printer = InlineDebugPrinter;
        for node in &ast {
            node.accept(&mut printer);
        }
    }

    #[test]
    fn test_entity_and_attribute_block_parsing() {
        use crate::logic::core::inline::parser::parse_phrases;
        // Entity
        let ast = parse_phrases("foo &amp; bar");
        assert!(ast.iter().any(|n| matches!(n, InlineNode::Entity { text, .. } if text == "&amp;")), "Should emit Entity node for &amp;");
        // Attribute block
        let ast2 = parse_phrases("foo {.class}");
        assert!(ast2.iter().any(|n| matches!(n, InlineNode::AttributeBlock { text, .. } if text == ".class")), "Should emit AttributeBlock node for {{.class}}");
    }
}
// types.rs - Token definitions, inline node enums, positions

/// Central location for all token types, node enums, and position tracking structs.

// types.rs - Token definitions, inline node enums, positions

/// Delimiter stack entry for emphasis/strong parsing (CommonMark spec)
/// See: https://spec.commonmark.org/0.31.2/#emphasis-and-strong-emphasis
/// and pulldown-cmark/comrak implementations
#[derive(Debug, Clone)]
pub struct Delim {
    /// Delimiter character: '*' or '_'
    pub ch: char,
    /// Number of consecutive delimiters in this run
    pub count: usize,
    /// Source position (line, column) for error reporting and AST
    pub pos: crate::logic::core::event_types::SourcePos,
    /// Can this delimiter open an emphasis/strong span? (left-flanking)
    pub can_open: bool,
    /// Can this delimiter close an emphasis/strong span? (right-flanking)
    pub can_close: bool,
    /// Index in the input string (for stack processing)
    pub idx: usize,
    /// Is this delimiter active? (for future link/image nesting)
    pub active: bool,
    /// Is this delimiter inside a code span?
    pub in_code: bool,
    /// Is this delimiter inside a link?
    pub in_link: bool,
    /// Is this delimiter inside an image?
    pub in_image: bool,
    /// Is this delimiter inside HTML?
    pub in_html: bool,
    /// Is this delimiter left-flanking?
    pub left_flanking: bool,
    /// Is this delimiter right-flanking?
    pub right_flanking: bool,
    /// Index of previous delimiter in the stack (if any)
    pub prev: Option<usize>,
    /// Index of next delimiter in the stack (if any)
    pub next: Option<usize>,
}

/// Bracket stack entry for link/image parsing (CommonMark spec)
/// See: https://spec.commonmark.org/0.31.2/#links
#[derive(Debug, Clone)]
pub struct Bracket {
    /// Is this an image opener? ('![')
    pub image: bool,
    /// Source position (line, column) for error reporting and AST
    pub pos: crate::logic::core::event_types::SourcePos,
    /// Index in the input string (for stack processing)
    pub idx: usize,
}

/// Inline token types for tokenizer output
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Text(String),
    Star(usize),
    Underscore(usize),
    Backtick(usize),
    Dollar(usize),
    OpenBracket,
    CloseBracket,
    Bang,
    OpenParen,
    CloseParen,
    Backslash(char),
    Ampersand,
    Html(String),
    CodeSpan(String),
    MathSpan(String),
    Entity(String),
    AttributeBlock(String),
    SoftBreak,
    HardBreak,
    // Add more as needed
}
/// Inline AST node definitions for Markdown inlines (CommonMark/GFM)
pub use crate::logic::core::event_types::SourcePos;

#[derive(Debug, Clone, PartialEq)]
pub enum InlineNode {
    Text { text: String, pos: SourcePos },
    Emphasis { children: Vec<InlineNode>, pos: SourcePos },
    Strong { children: Vec<InlineNode>, pos: SourcePos },
    Code { text: String, pos: SourcePos },
    Link { href: String, title: String, children: Vec<InlineNode>, pos: SourcePos },
    Image { src: String, alt: Vec<InlineNode>, title: String, pos: SourcePos },
    Math { text: String, pos: SourcePos },
    Html { text: String, pos: SourcePos },
    Entity { text: String, pos: SourcePos },
    AttributeBlock { text: String, pos: SourcePos },
    SoftBreak { pos: SourcePos },
    LineBreak { pos: SourcePos },
    Strikethrough { children: Vec<InlineNode>, pos: SourcePos },
    TaskListItem { checked: bool, children: Vec<InlineNode>, pos: SourcePos },
    // Extend with more types as needed (emoji, mention, etc.)
}

impl InlineNode {
    pub fn accept<V: InlineAstVisitor>(&self, visitor: &mut V) {
        match self {
            InlineNode::Text { .. } => visitor.visit_inline_text(self),
            InlineNode::Emphasis { children, .. } => {
                visitor.visit_inline_emphasis(self);
                for child in children {
                    child.accept(visitor);
                }
            }
            InlineNode::Strong { children, .. } => {
                visitor.visit_inline_strong(self);
                for child in children {
                    child.accept(visitor);
                }
            }
            InlineNode::Code { .. } => visitor.visit_inline_code(self),
            InlineNode::Link { children, .. } => {
                visitor.visit_inline_link(self);
                for child in children {
                    child.accept(visitor);
                }
            }
            InlineNode::Image { .. } => visitor.visit_inline_image(self),
            InlineNode::Math { .. } => visitor.visit_inline_math(self),
            InlineNode::Html { .. } => visitor.visit_inline_html(self),
            InlineNode::Entity { .. } => visitor.visit_inline_entity(self),
            InlineNode::AttributeBlock { .. } => visitor.visit_inline_attribute_block(self),
            InlineNode::SoftBreak { .. } => visitor.visit_inline_softbreak(self),
            InlineNode::LineBreak { .. } => visitor.visit_inline_linebreak(self),
            InlineNode::Strikethrough { children, .. } => {
                visitor.visit_inline_strikethrough(self);
                for child in children {
                    child.accept(visitor);
                }
            }
            InlineNode::TaskListItem { children, .. } => {
                visitor.visit_inline_tasklistitem(self);
                for child in children {
                    child.accept(visitor);
                }
            }
        }
    }
}

/// Trait for visiting InlineNode AST nodes.
pub trait InlineAstVisitor {
    fn visit_inline_strikethrough(&mut self, _node: &InlineNode) {}
    fn visit_inline_tasklistitem(&mut self, _node: &InlineNode) {}
    fn visit_inline_text(&mut self, _node: &InlineNode) {}
    fn visit_inline_emphasis(&mut self, _node: &InlineNode) {}
    fn visit_inline_strong(&mut self, _node: &InlineNode) {}
    fn visit_inline_code(&mut self, _node: &InlineNode) {}
    fn visit_inline_link(&mut self, _node: &InlineNode) {}
    fn visit_inline_image(&mut self, _node: &InlineNode) {}
    fn visit_inline_math(&mut self, _node: &InlineNode) {}
    fn visit_inline_html(&mut self, _node: &InlineNode) {}
    fn visit_inline_entity(&mut self, _node: &InlineNode) {}
    fn visit_inline_attribute_block(&mut self, _node: &InlineNode) {}
    fn visit_inline_softbreak(&mut self, _node: &InlineNode) {}
    fn visit_inline_linebreak(&mut self, _node: &InlineNode) {}
    // For future extensibility: emoji, mention, etc.
    fn visit_inline_emoji(&mut self, _node: &InlineNode) {}
    fn visit_inline_mention(&mut self, _node: &InlineNode) {}
}
