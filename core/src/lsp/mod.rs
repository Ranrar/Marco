// LSP integration: syntax highlighting, autocomplete, hover

pub mod completion;
pub mod diagnostics;
pub mod highlights;
pub mod hover;

pub use completion::*;
pub use diagnostics::*;
pub use highlights::*;
pub use hover::*;

use crate::ast::Document;

// LSP feature provider
pub struct LspProvider {
    document: Option<Document>,
}

impl LspProvider {
    pub fn new() -> Self {
        log::info!("LSP provider initialized");
        Self { document: None }
    }
    
    pub fn update_document(&mut self, document: Document) {
        log::debug!("LSP document updated");
        self.document = Some(document);
    }
}
