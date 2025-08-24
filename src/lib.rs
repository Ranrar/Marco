// Library entry point for integration tests and consumers.
// Re-export the internal modules needed by tests.
pub mod components;
pub mod logic;

// Re-export commonly used types
pub use components::marco_engine::parser::{MarkdownSyntaxMap, SyntaxRule};
pub use logic::buffer::{DocumentBuffer, RecentFiles};
