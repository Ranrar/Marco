pub mod parser;

// Re-export the parser for convenience
pub use parser::{parse_document, parse_with_rule, print_pairs, MarcoParser, Rule};
