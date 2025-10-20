// Integration tests: end-to-end pipeline validation

#[cfg(test)]
mod tests {
    use core::{parser, render};
    
    #[test]
    fn test_full_pipeline() {
        let input = "# Test\n\nHello **world**!";
        
        log::info!("Testing full pipeline: parse → render");
        
        // Parse
        let doc = parser::parse(input).expect("Parse failed");
        log::debug!("Parse successful");
        
        // Render
        let html = render::render(&doc, &render::RenderOptions::default())
            .expect("Render failed");
        log::debug!("Render successful: {} bytes", html.len());
        
        assert!(!html.is_empty());
        log::info!("Full pipeline test passed");
    }
    
    #[test]
    fn test_pipeline_with_code_blocks() {
        let input = "```rust\nfn main() {}\n```";
        
        let doc = parser::parse(input).expect("Parse failed");
        let html = render::render(&doc, &render::RenderOptions::default())
            .expect("Render failed");
        
        assert!(html.contains("<code>"));
        log::info!("Code block pipeline test passed");
    }
}
