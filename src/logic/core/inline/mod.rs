//! Inline parsing core module
//!
//! This module provides the tokenizer, parser, delimiter stack logic, normalization pipeline,
//! and rule handlers for Markdown inline parsing. It exposes key types and functions for
//! building and extending the inline parsing pipeline.

pub mod tokenizer;
pub mod parser;
pub mod delimiters;
pub mod postprocess;
pub mod rules;
pub mod types;

// Re-export core types for downstream use
pub use types::{InlineNode, Token, Delim, Bracket};

/// Re-export main delimiter processing function for extension points
pub use delimiters::process_delimiters;