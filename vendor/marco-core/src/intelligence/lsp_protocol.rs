//! Optional protocol bridge for external LSP transport.
//!
//! The editor intelligence system is in-process first. This module exists as
//! an explicit extension point for protocol adapters.

#[derive(Debug, Default, Clone)]
/// Capability flags used by external protocol adapters.
pub struct ProtocolCapabilities {
    /// Whether diagnostics requests are supported.
    pub diagnostics: bool,
    /// Whether hover requests are supported.
    pub hover: bool,
    /// Whether completion requests are supported.
    pub completion: bool,
    /// Whether semantic highlighting is supported.
    pub semantic_highlighting: bool,
}
