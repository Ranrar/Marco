// Parser tests: validate two-stage parsing (blocks → inlines)

#[cfg(test)]
mod tests {
    use core::parser;
    
    #[test]
    fn test_parse_simple_document() {
        let input = "# Hello\n\nThis is a paragraph.";
        let result = parser::parse(input);
        assert!(result.is_ok());
        log::info!("Simple document parse passed");
    }
    
    #[test]
    fn test_parse_empty_document() {
        let result = parser::parse("");
        assert!(result.is_ok());
        log::info!("Empty document parse passed");
    }
    
    #[test]
    fn test_block_parser() {
        let input = "# Heading\n\nParagraph text.";
        let blocks = parser::parse_blocks(input);
        assert!(blocks.is_ok());
        log::info!("Block parser test passed");
    }
}
