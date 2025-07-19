/// Trait for rendering CommonMark AST to various targets (HTML, GTK SourceView, etc.)
/// Provides methods for rendering, highlighting, and error annotation.
pub trait Renderer {
    /// Render the AST to the target output (HTML, GTK, etc.).
    /// Returns Ok(()) on success, or Err(message) on failure.
    fn render(&mut self, ast: &crate::logic::ast::blocks_and_inlines::Block) -> Result<(), String>;

    /// Highlight a region in the output corresponding to a source position.
    fn highlight(&mut self, pos: &crate::logic::core::event::SourcePos);

    /// Annotate an error at a given source position with a message.
    fn annotate_error(&mut self, pos: &crate::logic::core::event::SourcePos, message: &str);
}
