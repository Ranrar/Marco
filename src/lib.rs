// Library entry point for integration tests and consumers.
// Re-export the internal modules needed by tests.
pub mod logic;
pub mod components;

// Re-export commonly used types
pub use logic::buffer::{DocumentBuffer, RecentFiles};
pub use components::marco_engine::parser::{parse_document_blocks, MarkdownSyntaxMap, SyntaxRule};