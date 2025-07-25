//! Math extension logic stub

use crate::logic::core::options::ParserOptions;
use crate::logic::core::inline::types::InlineNode;
use super::MarkdownExtension;

pub struct MathExtension;

impl MarkdownExtension for MathExtension {
    fn apply(&self, ast: &mut Vec<InlineNode>, options: &ParserOptions) {
        if !options.math {
            return;
        }
        // Inline math: $...$ (GFM/LaTeX)
        let mut i = 0;
        while i < ast.len() {
            if let InlineNode::Text { text, pos } = &ast[i] {
                if text.starts_with('$') && text.ends_with('$') && text.len() > 2 && !text.starts_with("$$") {
                    let math = text[1..text.len()-1].to_string();
                    ast[i] = InlineNode::Math {
                        text: math,
                        pos: *pos,
                    };
                }
                // Block math: $$...$$ (GFM/LaTeX)
                else if text.starts_with("$$") && text.ends_with("$$") && text.len() > 4 {
                    let math = text[2..text.len()-2].to_string();
                    ast[i] = InlineNode::Math {
                        text: math,
                        pos: *pos,
                    };
                }
            }
            i += 1;
        }
    }
}
