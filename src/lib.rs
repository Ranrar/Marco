// Library entry point for integration tests and consumers.
// Re-export the internal modules needed by tests.
pub mod logic;

// Re-export commonly used types
pub use logic::buffer::{DocumentBuffer, RecentFiles};
pub use logic::menu_items::FileOperations;
