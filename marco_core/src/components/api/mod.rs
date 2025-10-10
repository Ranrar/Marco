//! Marco ↔ Polo IPC API Module
//!
//! This module provides the complete IPC infrastructure for communication
//! between Marco (editor) and Polo (viewer).
//!
//! # Modules
//!
//! - `protocol`: RON-serializable message structures
//! - `session`: Session management and UUID key generation

pub mod protocol;
pub mod session;

// Re-export commonly used items
pub use protocol::{ApiRequest, ApiResponse, ApiNotification, SessionInfo};
pub use session::{SessionData, SessionManager};
