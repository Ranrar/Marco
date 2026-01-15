// Block-level grammar modules
//
// This module contains individual CommonMark block element parsers.
// Each parser extracts a specific block-level construct and returns nom IResult.

// Implemented modules (Phase 2 Complete - All 10 block grammar modules extracted)
pub mod cm_blockquote;
pub mod cm_fenced_code_block;
pub mod cm_heading;
pub mod cm_html_blocks;
pub mod cm_indented_code_block;
pub mod cm_link_reference;
pub mod cm_list;
pub mod cm_paragraph;
pub mod cm_setext_heading;
pub mod cm_thematic_break;
pub mod gfm_table;
pub mod marco_headerless_table;

// Re-export all block parsers
pub use cm_blockquote::*;
pub use cm_fenced_code_block::*;
pub use cm_heading::*;
pub use cm_html_blocks::*;
pub use cm_indented_code_block::*;
pub use cm_link_reference::*;
pub use cm_list::*;
pub use cm_paragraph::*;
pub use cm_setext_heading::*;
pub use cm_thematic_break::*;
pub use gfm_table::*;
pub use marco_headerless_table::*;
