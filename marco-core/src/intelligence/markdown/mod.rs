//! Markdown model boundary for intelligence features.

pub mod ast;
pub mod blocks;
pub mod inlines;

pub use ast::{
    is_block_kind, is_inline_kind, node_span, MarkdownDocument, MarkdownNode, MarkdownNodeKind,
};
pub use blocks::{
    classify_block_kind, collect_block_nodes, is_block_node, top_level_blocks, BlockCategory,
};
pub use inlines::{classify_inline_kind, collect_inline_nodes, is_inline_node, InlineCategory};
