//! Extension activation traits and registration pattern

use crate::logic::core::options::ParserOptions;
use crate::logic::core::inline::types::InlineNode;

pub trait MarkdownExtension {
    /// Apply extension logic to the AST if enabled in options
    fn apply(&self, ast: &mut Vec<InlineNode>, options: &ParserOptions);
}

// Example: GFM extension registration stub
pub mod github;
// Example: Math extension registration stub
pub mod math;
