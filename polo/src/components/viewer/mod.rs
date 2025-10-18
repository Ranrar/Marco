// Viewer module - Markdown rendering orchestration
//
//! # Viewer Module
//!
//! Orchestrates markdown file loading and HTML rendering to WebKit WebView.
//!
//! ## Submodules
//!
//! - **`empty_state`**: Welcome screen when no file is opened
//! - **`rendering`**: Core markdown-to-HTML conversion and WebView loading
//!
//! ## Public API
//!
//! - **`show_empty_state_with_theme`**: Display theme-aware welcome screen
//! - **`load_and_render_markdown`**: Load file, parse, and render to WebView
//!
//! ## Rendering Pipeline
//!
//! ```text
//! File Path
//!    ↓
//! Read file content
//!    ↓
//! Parse with core (cached)
//!    ↓
//! Generate HTML with theme CSS
//!    ↓
//! Load into WebView with base URI
//! ```
//!
//! ## Features
//!
//! - **Caching**: Uses global parser cache for performance
//! - **Theme Support**: Injects selected CSS theme into HTML
//! - **Syntax Highlighting**: Generates theme-aware code block CSS
//! - **Error Display**: Shows user-friendly error messages in WebView
//! - **Base URI**: Properly sets base path for relative image/link resolution

pub mod empty_state;
pub mod rendering;

pub use empty_state::show_empty_state_with_theme;
pub use rendering::load_and_render_markdown;
// parse_markdown_to_html is internal, not re-exported
