//! Grammar definitions for Markdown syntax
//!
//! This module provides the grammar layer for parsing Markdown. All parsing logic
//! has been refactored into modular files for better maintainability.
//!
//! Phase 2-4 Complete: All grammar functions now live in `blocks/` and `inlines/` subdirectories.

// Modular grammar structure
pub mod blocks;
pub mod inlines;
pub mod shared;

// Re-export shared types and utilities
pub use shared::Span;

// Re-export all block grammar functions
pub use blocks::{
    blockquote, detect_list_marker, fenced_code_block, heading, html_block_tag, html_cdata,
    html_comment, html_complete_tag, html_declaration, html_processing_instruction,
    html_special_tag, indented_code_block, link_reference_definition, list, list_item, paragraph,
    setext_heading, thematic_break, ListItemData, ListMarker,
};

// Re-export all inline grammar functions
pub use inlines::{
    autolink, backslash_escape, code_span, emphasis, hard_line_break, image, inline_html, link,
    soft_line_break, strong,
};
