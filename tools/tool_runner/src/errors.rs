use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Missing definition: {0}")]
    MissingDefinition(String),

    #[error("Grammar error: {0}")]
    Grammar(String),

    #[error("Other: {0}")]
    Other(String),
}

impl ValidationError {
    pub fn missing(s: impl Into<String>) -> Self {
        ValidationError::MissingDefinition(s.into())
    }
}
