//! ParserOptions: feature flags for enabling/disabling Markdown extensions

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParserOptions {
    pub gfm: bool,
    pub math: bool,
    // Add more extensions as needed
}

impl Default for ParserOptions {
    fn default() -> Self {
        Self {
            gfm: false,
            math: false,
        }
    }
}
