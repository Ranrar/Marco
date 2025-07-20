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
    fn visit_inline_softbreak(&mut self, node: &InlineNode) {
        println!("SoftBreak: {:?}", node);
    }
    fn visit_inline_linebreak(&mut self, node: &InlineNode) {
        println!("LineBreak: {:?}", node);
    }
}

#[cfg(test)]
mod visitor_tests {
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
            InlineNode::SoftBreak { pos: SourcePos { line: 2, column: 1 } },
            InlineNode::LineBreak { pos: SourcePos { line: 2, column: 2 } },
        ];
        let mut printer = InlineDebugPrinter;
        for node in &ast {
            node.accept(&mut printer);
        }
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
    Image { src: String, alt: String, title: String, pos: SourcePos },
    Math { text: String, pos: SourcePos },
    Html { text: String, pos: SourcePos },
    SoftBreak { pos: SourcePos },
    LineBreak { pos: SourcePos },
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
            InlineNode::SoftBreak { .. } => visitor.visit_inline_softbreak(self),
            InlineNode::LineBreak { .. } => visitor.visit_inline_linebreak(self),
        }
    }
}

/// Trait for visiting InlineNode AST nodes.
pub trait InlineAstVisitor {
    fn visit_inline_text(&mut self, _node: &InlineNode) {}
    fn visit_inline_emphasis(&mut self, _node: &InlineNode) {}
    fn visit_inline_strong(&mut self, _node: &InlineNode) {}
    fn visit_inline_code(&mut self, _node: &InlineNode) {}
    fn visit_inline_link(&mut self, _node: &InlineNode) {}
    fn visit_inline_image(&mut self, _node: &InlineNode) {}
    fn visit_inline_math(&mut self, _node: &InlineNode) {}
    fn visit_inline_html(&mut self, _node: &InlineNode) {}
    fn visit_inline_softbreak(&mut self, _node: &InlineNode) {}
    fn visit_inline_linebreak(&mut self, _node: &InlineNode) {}
}
