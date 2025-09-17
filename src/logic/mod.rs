pub mod swanson;
pub mod crossplatforms;
pub mod paths;
pub mod menu_items;
pub mod buffer;
pub mod loaders;
pub mod layoutstate;
pub mod logger;

// Re-export commonly used types
pub use buffer::{DocumentBuffer, RecentFiles};