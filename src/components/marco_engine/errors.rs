//! Error handling for the Marco engine

use anyhow::{anyhow, Context};
use std::fmt;

/// Main error type for the Marco engine that wraps anyhow::Error
/// but provides structured variants for pattern matching
#[derive(Debug)]
pub enum MarcoError {
    /// Parsing-related errors
    Parse(String),
    /// I/O related errors
    IO(String),
    /// AST building/validation errors
    Ast(String),
    /// Async operation errors
    Async(String),
    /// Generic errors for other cases
    Generic(anyhow::Error),
}

impl MarcoError {
    /// Create a new parse error
    pub fn parse<S: Into<String>>(msg: S) -> Self {
        Self::Parse(msg.into())
    }

    /// Create a new I/O error
    pub fn io<S: Into<String>>(msg: S) -> Self {
        Self::IO(msg.into())
    }

    /// Create a new AST error
    pub fn ast<S: Into<String>>(msg: S) -> Self {
        Self::Ast(msg.into())
    }

    /// Create a new async error
    pub fn async_error<S: Into<String>>(msg: S) -> Self {
        Self::Async(msg.into())
    }

    /// Create a generic error
    pub fn generic<S: Into<String>>(msg: S) -> Self {
        Self::Generic(anyhow!(msg.into()))
    }

    // Legacy constructor methods for compatibility
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::Parse(message.into())
    }

    pub fn ast_error(message: impl Into<String>) -> Self {
        Self::Ast(message.into())
    }

    pub fn ast_validation_error(message: impl Into<String>) -> Self {
        Self::Ast(format!("AST validation error: {}", message.into()))
    }

    pub fn missing_child_error(
        rule: impl Into<String>,
        span_start: usize,
        span_end: usize,
    ) -> Self {
        Self::Ast(format!(
            "Missing expected child node for rule {} at {}..{}",
            rule.into(),
            span_start,
            span_end
        ))
    }

    pub fn invalid_node_structure(
        rule: impl Into<String>,
        message: impl Into<String>,
        span_start: usize,
        span_end: usize,
    ) -> Self {
        Self::Ast(format!(
            "Invalid node structure for rule {}: {} at {}..{}",
            rule.into(),
            message.into(),
            span_start,
            span_end
        ))
    }

    pub fn span_error(message: impl Into<String>) -> Self {
        Self::Ast(format!("Span construction error: {}", message.into()))
    }

    pub fn rule_processing_error(rule: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Parse(format!(
            "Rule processing error for {}: {}",
            rule.into(),
            message.into()
        ))
    }

    pub fn render_error(message: impl Into<String>) -> Self {
        Self::Generic(anyhow!("Render error: {}", message.into()))
    }

    pub fn json_error(message: impl Into<String>) -> Self {
        Self::Generic(anyhow!("JSON error: {}", message.into()))
    }

    pub fn io_error(message: impl Into<String>) -> Self {
        Self::IO(message.into())
    }

    pub fn parallel_error(message: impl Into<String>) -> Self {
        Self::Async(format!("Parallel processing error: {}", message.into()))
    }

    // Validation-specific error constructors
    pub fn max_depth_exceeded(current_depth: usize, max_depth: usize) -> Self {
        Self::Ast(format!(
            "Maximum nesting depth exceeded: {} > {}",
            current_depth, max_depth
        ))
    }

    pub fn invalid_nesting(parent_type: impl Into<String>, child_type: impl Into<String>) -> Self {
        Self::Ast(format!(
            "Invalid nesting: {} cannot be nested inside {}",
            child_type.into(),
            parent_type.into()
        ))
    }

    pub fn invalid_content_type(
        node_type: impl Into<String>,
        expected: impl Into<String>,
        found: impl Into<String>,
    ) -> Self {
        Self::Ast(format!(
            "Invalid content type for {}: expected {}, found {}",
            node_type.into(),
            expected.into(),
            found.into()
        ))
    }

    pub fn empty_content(node_type: impl Into<String>) -> Self {
        Self::Ast(format!("{} requires non-empty content", node_type.into()))
    }

    pub fn content_overflow(node_type: impl Into<String>, max_length: usize) -> Self {
        Self::Ast(format!(
            "Content overflow: {} exceeds maximum length of {}",
            node_type.into(),
            max_length
        ))
    }

    pub fn circular_reference(
        node_type: impl Into<String>,
        span_start: usize,
        span_end: usize,
    ) -> Self {
        Self::Ast(format!(
            "Circular reference detected in {} at {}..{}",
            node_type.into(),
            span_start,
            span_end
        ))
    }

    pub fn duplicate_identifier(identifier: impl Into<String>, context: impl Into<String>) -> Self {
        Self::Ast(format!(
            "Duplicate identifier '{}' in {}",
            identifier.into(),
            context.into()
        ))
    }

    pub fn unresolved_reference(reference: impl Into<String>, context: impl Into<String>) -> Self {
        Self::Ast(format!(
            "Unresolved reference to '{}' in {}",
            reference.into(),
            context.into()
        ))
    }

    pub fn type_mismatch(expected_type: impl Into<String>, actual_type: impl Into<String>) -> Self {
        Self::Ast(format!(
            "Type mismatch: expected {}, found {}",
            expected_type.into(),
            actual_type.into()
        ))
    }

    pub fn memory_limit_exceeded(current_usage: usize, limit: usize) -> Self {
        Self::Generic(anyhow!(
            "Memory limit exceeded: {} > {}",
            current_usage,
            limit
        ))
    }

    pub fn processing_timeout(timeout_ms: u64) -> Self {
        Self::Generic(anyhow!(
            "Processing timeout: operation exceeded {}ms",
            timeout_ms
        ))
    }

    pub fn cache_error(reason: impl Into<String>) -> Self {
        Self::Generic(anyhow!("Cache error: {}", reason.into()))
    }

    pub fn unexpected_token(token: impl Into<String>, position: usize) -> Self {
        Self::Parse(format!(
            "Unexpected token '{}' at position {}",
            token.into(),
            position
        ))
    }

    pub fn unterminated_construct(construct: impl Into<String>, start_position: usize) -> Self {
        Self::Parse(format!(
            "Unterminated {} starting at position {}",
            construct.into(),
            start_position
        ))
    }

    pub fn malformed_construct(construct: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::Parse(format!("Malformed {}: {}", construct.into(), reason.into()))
    }

    pub fn invalid_admonition_type(admonition_type: impl Into<String>) -> Self {
        Self::Parse(format!(
            "Invalid admonition type '{}': supported types are note, warning, tip, danger, info",
            admonition_type.into()
        ))
    }

    pub fn invalid_user_mention(reason: impl Into<String>) -> Self {
        Self::Parse(format!("Invalid user mention format: {}", reason.into()))
    }

    pub fn invalid_bookmark(path: impl Into<String>, line: Option<usize>) -> Self {
        match line {
            Some(line) => Self::Parse(format!(
                "Invalid bookmark reference: {} (line {})",
                path.into(),
                line
            )),
            None => Self::Parse(format!("Invalid bookmark reference: {}", path.into())),
        }
    }

    pub fn tabs_error(reason: impl Into<String>) -> Self {
        Self::Parse(format!("Tabs block error: {}", reason.into()))
    }
}

impl fmt::Display for MarcoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(msg) => write!(f, "Parse error: {}", msg),
            Self::IO(msg) => write!(f, "I/O error: {}", msg),
            Self::Ast(msg) => write!(f, "AST error: {}", msg),
            Self::Async(msg) => write!(f, "Async error: {}", msg),
            Self::Generic(e) => write!(f, "Error: {}", e),
        }
    }
}

impl std::error::Error for MarcoError {}

impl From<anyhow::Error> for MarcoError {
    fn from(error: anyhow::Error) -> Self {
        Self::Generic(error)
    }
}

impl From<std::io::Error> for MarcoError {
    fn from(error: std::io::Error) -> Self {
        Self::IO(error.to_string())
    }
}

impl From<serde_json::Error> for MarcoError {
    fn from(error: serde_json::Error) -> Self {
        Self::Generic(anyhow!(error))
    }
}

impl Clone for MarcoError {
    fn clone(&self) -> Self {
        match self {
            Self::Parse(msg) => Self::Parse(msg.clone()),
            Self::IO(msg) => Self::IO(msg.clone()),
            Self::Ast(msg) => Self::Ast(msg.clone()),
            Self::Async(msg) => Self::Async(msg.clone()),
            Self::Generic(e) => Self::Generic(anyhow!(e.to_string())),
        }
    }
}

// Re-export for convenience
pub type MarcoResult<T> = Result<T, MarcoError>;

/// Extension trait for adding context to Results
pub trait MarcoErrorExt<T> {
    /// Add context to an error
    fn with_context<F>(self, f: F) -> MarcoResult<T>
    where
        F: FnOnce() -> String;

    /// Add simple context string
    fn context_str(self, context: &str) -> MarcoResult<T>;
}

impl<T, E> MarcoErrorExt<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_context<F>(self, f: F) -> MarcoResult<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| MarcoError::Generic(anyhow!(e).context(f())))
    }

    fn context_str(self, context: &str) -> MarcoResult<T> {
        self.map_err(|e| MarcoError::Generic(anyhow!(e).context(context.to_string())))
    }
}
