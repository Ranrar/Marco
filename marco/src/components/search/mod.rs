//! Search & Replace Component
//!
//! Modular search and replace functionality for Marco editor.
//!
//! ## Architecture
//!
//! - `state` - Search state management and thread-local storage
//! - `window` - Window creation and dialog management
//! - `ui` - UI widget builders for search controls
//! - `engine` - Search logic and highlighting
//! - `navigation` - Match navigation and scrolling
//! - `replace` - Replace operations
//!
//! ## Cross-Platform Support
//!
//! - **Linux**: Full WebView integration for preview synchronization
//! - **Windows**: Editor-only search (no WebView support)

pub mod state;
pub mod window;
pub mod ui;
pub mod engine;
pub mod navigation;
pub mod replace;

// Re-export public API types
pub use state::SearchOptions;

// Re-export highlighting functions
pub use engine::{apply_enhanced_search_highlighting, clear_enhanced_search_highlighting};

// Re-export window functions (Linux only)
#[cfg(target_os = "linux")]
pub use window::get_or_create_search_window;
