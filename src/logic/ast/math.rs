//! AST node definitions for Markdown math blocks and inline math (GFM/LaTeX).

use crate::logic::attributes::Attributes;
use crate::logic::core::event::SourcePos;

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
}
