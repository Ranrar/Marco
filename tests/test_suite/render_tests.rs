// Renderer tests: validate HTML output

#[cfg(test)]
mod tests {
    use core::ast::Document;
    use core::render::{render, RenderOptions};
    
    #[test]
    fn test_render_empty_document() {
        let doc = Document::new();
        let options = RenderOptions::default();
        let html = render(&doc, &options);
        assert!(html.is_ok());
        log::info!("Empty document render passed");
    }
    
    #[test]
    fn test_render_with_options() {
        let doc = Document::new();
        let mut options = RenderOptions::default();
        options.syntax_highlighting = false;
        options.line_numbers = true;
        
        let html = render(&doc, &options);
        assert!(html.is_ok());
        log::info!("Render with options test passed");
    }
}
