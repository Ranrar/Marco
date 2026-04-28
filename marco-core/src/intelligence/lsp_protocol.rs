//! Optional protocol bridge for external LSP transport.
//!
//! The editor intelligence system is in-process first. This module exists as
//! an explicit extension point for protocol adapters.

#[derive(Debug, Default, Clone)]
pub struct ProtocolCapabilities {
    pub diagnostics: bool,
    pub hover: bool,
    pub completion: bool,
    pub semantic_highlighting: bool,
}
