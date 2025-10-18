// Grammar tests: validate nom parsers for block and inline syntax

#[cfg(test)]
mod tests {
    use core::grammar::{block, inline};
    use nom_locate::LocatedSpan;
    
    type Span<'a> = LocatedSpan<&'a str>;
    
    #[test]
    fn test_heading_parse() {
        let input = Span::new("# Hello");
        let result = block::heading(input);
        assert!(result.is_ok());
        log::info!("Heading test passed");
    }
    
    #[test]
    fn test_emphasis_parse() {
        let input = Span::new("*italic*");
        let result = inline::emphasis(input);
        assert!(result.is_ok());
        log::info!("Emphasis test passed");
    }
}
