//! Error handling for the Marco engine

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum MarcoError {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("AST error: {0}")]
    AST(String),

    #[error("Render error: {0}")]
    Render(String),

    #[error("JSON error: {0}")]
    Json(String),

    #[error("IO error: {0}")]
    IO(String),

    #[error("Async error: {0}")]
    Async(String),

    #[error("Parallel processing error: {0}")]
    Parallel(String),
}

impl MarcoError {
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::Parse(message.into())
    }

    pub fn ast_error(message: impl Into<String>) -> Self {
        Self::AST(message.into())
    }

    pub fn render_error(message: impl Into<String>) -> Self {
        Self::Render(message.into())
    }

    pub fn json_error(message: impl Into<String>) -> Self {
        Self::Json(message.into())
    }

    pub fn io_error(message: impl Into<String>) -> Self {
        Self::IO(message.into())
    }

    pub fn async_error(message: impl Into<String>) -> Self {
        Self::Async(message.into())
    }

    pub fn parallel_error(message: impl Into<String>) -> Self {
        Self::Parallel(message.into())
    }
}

impl From<serde_json::Error> for MarcoError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}

impl From<std::io::Error> for MarcoError {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, MarcoError>;
