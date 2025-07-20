impl MathBlock {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_math_block(self);
    }
}

impl MathInline {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_math_inline(self);
    }
}

impl MathType {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_math_type(self);
    }
}
/// Trait for visiting AST nodes in math.rs
pub trait AstVisitor {
    fn visit_math_block(&mut self, block: &MathBlock) {
        self.walk_math_block(block);
    }
    fn walk_math_block(&mut self, block: &MathBlock) {
        self.visit_math_type(&block.math_type);
        // MathBlock is a leaf node, no further recursion
    }
    fn visit_math_inline(&mut self, inline: &MathInline) {
        self.walk_math_inline(inline);
    }
    fn walk_math_inline(&mut self, inline: &MathInline) {
        self.visit_math_type(&inline.math_type);
        // MathInline is a leaf node, no further recursion
    }
    fn visit_math_type(&mut self, _math_type: &MathType) {
        // No children to traverse
    }
}
/// AST node definitions for Markdown math blocks and inline math (GFM/LaTeX).
use anyhow::Error;

/// Type alias for AST results with anyhow error handling.
pub type AstResult<T> = Result<T, Error>;

/// Example: minimal error-producing function for demonstration.
pub fn parse_math_block_safe(is_valid: bool) -> AstResult<MathBlock> {
    if !is_valid {
        Err(Error::msg("Invalid math block"))
    } else {
        Ok(MathBlock {
            content: "dummy".to_string(),
            display: true,
            math_type: MathType::LaTeX,
            position: None,
            attributes: None,
        })
    }
}

use crate::logic::core::attr_parser::Attributes;
use crate::logic::core::event_types::SourcePos;

/// Supported math types for Markdown math blocks and inline math
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MathType {
    LaTeX,
    TeX,
    AsciiMath,
    MathML,
    Chemistry, // For mhchem or similar chemistry notation
    SVG,
    Other(String), // For custom or unknown types
}

/// Block-level math (e.g., $$ ... $$ or ```math ... ```)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MathBlock {
    pub content: String,                // Raw math content
    pub display: bool,                  // true for block, false for inline
    pub math_type: MathType,            // Type of math (LaTeX, AsciiMath, etc.)
    pub position: Option<SourcePos>,    // Source position for diagnostics
    pub attributes: Option<Attributes>, // Custom attributes (class, id, data-*)
}

/// Inline math (e.g., $ ... $)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MathInline {
    pub content: String,
    pub math_type: MathType,            // Type of math (LaTeX, AsciiMath, etc.)
    pub position: Option<SourcePos>,
    pub attributes: Option<Attributes>,
}

// You can add these to your Block/Inline enums:
// enum Block { ... Math(MathBlock), ... }
// enum Inline { ... Math(MathInline), ... }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn math_type_variants_work() {
        let variants = [
            MathType::LaTeX,
            MathType::TeX,
            MathType::AsciiMath,
            MathType::MathML,
            MathType::Chemistry,
            MathType::SVG,
            MathType::Other("CustomType".to_string()),
        ];
        for variant in variants {
            let block = MathBlock {
                content: "x^2".to_string(),
                display: true,
                math_type: variant.clone(),
                position: None,
                attributes: None,
            };
            let inline = MathInline {
                content: "y^2".to_string(),
                math_type: variant.clone(),
                position: None,
                attributes: None,
            };
            match block.math_type {
                MathType::LaTeX => {},
                MathType::TeX => {},
                MathType::AsciiMath => {},
                MathType::MathML => {},
                MathType::Chemistry => {},
                MathType::SVG => {},
                MathType::Other(_) => {},
            }
            match inline.math_type {
                MathType::LaTeX => {},
                MathType::TeX => {},
                MathType::AsciiMath => {},
                MathType::MathML => {},
                MathType::Chemistry => {},
                MathType::SVG => {},
                MathType::Other(_) => {},
            }
        }
    }

    #[test]
    fn test_math_block_traversal() {
        let math = MathBlock {
            content: "x^2".to_string(),
            display: true,
            math_type: MathType::LaTeX,
            position: None,
            attributes: None,
        };
        struct Printer;
        impl AstVisitor for Printer {
            fn visit_math_block(&mut self, block: &MathBlock) {
                assert_eq!(block.content, "x^2");
                self.walk_math_block(block);
            }
        }
        let mut printer = Printer;
        printer.visit_math_block(&math);
    }

    #[test]
    fn test_math_inline_traversal() {
        let math = MathInline {
            content: "y^2".to_string(),
            math_type: MathType::TeX,
            position: None,
            attributes: None,
        };
        struct Printer;
        impl AstVisitor for Printer {
            fn visit_math_inline(&mut self, inline: &MathInline) {
                assert_eq!(inline.content, "y^2");
                self.walk_math_inline(inline);
            }
        }
        let mut printer = Printer;
        printer.visit_math_inline(&math);
    }

    #[test]
    fn test_error_handling() {
        let result = super::parse_math_block_safe(false);
        assert!(result.is_err());
        let result = super::parse_math_block_safe(true);
        assert!(result.is_ok());
    }
}
