use pest::Parser;
use pest::iterators::Pairs;
use pest_derive::Parser;

/// Block-level parser with modular grammar files
/// Each grammar file is loaded via a separate #[grammar] attribute
/// Pest will compose all rules into a single Rule enum
#[derive(Parser)]
#[grammar = "components/marco_engine/grammar/block/_core.pest"]
#[grammar = "components/marco_engine/grammar/block/thematic_break.pest"]
#[grammar = "components/marco_engine/grammar/block/atx_heading.pest"]
#[grammar = "components/marco_engine/grammar/block/setext_heading.pest"]
#[grammar = "components/marco_engine/grammar/block/fenced_code_block.pest"]
#[grammar = "components/marco_engine/grammar/block/indented_code_block.pest"]
#[grammar = "components/marco_engine/grammar/block/blockquote.pest"]
#[grammar = "components/marco_engine/grammar/block/document.pest"]
pub struct BlockParser;

/// Parse document into blocks
pub fn parse_blocks(input: &str) -> Result<Pairs<Rule>, String> {
    BlockParser::parse(Rule::document, input)
        .map_err(|e| format!("Block parse error: {}", e))
}

/// Parse a specific block type (for testing individual rules)
pub fn parse_block_rule(input: &str, rule: Rule) -> Result<Pairs<Rule>, String> {
    BlockParser::parse(rule, input)
        .map_err(|e| format!("Block rule parse error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn smoke_test_thematic_break_dash() {
        let input = "---\n";
        let result = BlockParser::parse(Rule::thematic_break, input);
        assert!(result.is_ok(), "Should parse dash thematic break");
    }
    
    #[test]
    fn smoke_test_thematic_break_asterisk() {
        let input = "***\n";
        let result = BlockParser::parse(Rule::thematic_break, input);
        assert!(result.is_ok(), "Should parse asterisk thematic break");
    }
    
    #[test]
    fn smoke_test_thematic_break_underscore() {
        let input = "___\n";
        let result = BlockParser::parse(Rule::thematic_break, input);
        assert!(result.is_ok(), "Should parse underscore thematic break");
    }
    
    #[test]
    fn smoke_test_thematic_break_with_spaces() {
        let input = "- - -\n";
        let result = BlockParser::parse(Rule::thematic_break, input);
        assert!(result.is_ok(), "Should parse thematic break with spaces");
    }
    
    #[test]
    fn smoke_test_atx_heading_h1() {
        let input = "# Heading\n";
        let result = BlockParser::parse(Rule::atx_heading, input);
        match &result {
            Ok(_) => {},
            Err(e) => eprintln!("Parse error: {}", e),
        }
        assert!(result.is_ok(), "Should parse H1 heading");
    }
    
    #[test]
    fn smoke_test_atx_heading_h6() {
        let input = "###### Deep Heading\n";
        let result = BlockParser::parse(Rule::atx_heading, input);
        match &result {
            Ok(_) => {},
            Err(e) => eprintln!("Parse error: {}", e),
        }
        assert!(result.is_ok(), "Should parse H6 heading");
    }
    
    #[test]
    fn smoke_test_atx_heading_requires_space() {
        let input = "#NoSpace\n";
        let result = BlockParser::parse(Rule::atx_heading, input);
        assert!(result.is_err(), "Should fail without space after #");
    }
    
    #[test]
    fn smoke_test_paragraph() {
        let input = "This is a paragraph.\n";
        let result = BlockParser::parse(Rule::paragraph, input);
        assert!(result.is_ok(), "Should parse simple paragraph");
    }
    
    #[test]
    fn smoke_test_document() {
        let input = "# Heading\n\nThis is a paragraph.\n\n---\n";
        let result = parse_blocks(input);
        assert!(result.is_ok(), "Should parse complete document");
        
        let pairs = result.unwrap();
        let blocks: Vec<_> = pairs.flatten().filter(|p| matches!(
            p.as_rule(),
            Rule::atx_heading | Rule::paragraph | Rule::thematic_break
        )).collect();
        
        assert_eq!(blocks.len(), 3, "Should have 3 blocks: heading, paragraph, thematic_break");
    }
    
    // Fenced code block tests
    #[test]
    fn smoke_test_fenced_code_backticks() {
        let input = "```\ncode here\n```\n";
        let result = BlockParser::parse(Rule::fenced_code_block, input);
        assert!(result.is_ok(), "Should parse backtick fenced code");
    }
    
    #[test]
    fn smoke_test_fenced_code_tildes() {
        let input = "~~~\ncode here\n~~~\n";
        let result = BlockParser::parse(Rule::fenced_code_block, input);
        assert!(result.is_ok(), "Should parse tilde fenced code");
    }
    
    #[test]
    fn smoke_test_fenced_code_with_info() {
        let input = "```rust\nfn main() {}\n```\n";
        let result = BlockParser::parse(Rule::fenced_code_block, input);
        assert!(result.is_ok(), "Should parse fenced code with info string");
    }
    
    // Blockquote tests
    #[test]
    fn smoke_test_blockquote_simple() {
        let input = "> This is a quote\n";
        let result = BlockParser::parse(Rule::blockquote, input);
        assert!(result.is_ok(), "Should parse simple blockquote");
    }
    
    #[test]
    fn smoke_test_blockquote_multi_line() {
        let input = "> Line 1\n> Line 2\n";
        let result = BlockParser::parse(Rule::blockquote, input);
        assert!(result.is_ok(), "Should parse multi-line blockquote");
    }
    
    // Setext heading tests
    #[test]
    fn smoke_test_setext_heading_level1() {
        let input = "Heading\n===\n";
        let result = BlockParser::parse(Rule::setext_heading, input);
        assert!(result.is_ok(), "Should parse setext level 1 heading");
    }
    
    #[test]
    fn smoke_test_setext_heading_level2() {
        let input = "Heading\n---\n";
        let result = BlockParser::parse(Rule::setext_heading, input);
        assert!(result.is_ok(), "Should parse setext level 2 heading");
    }
    
    // Indented code block tests
    #[test]
    fn smoke_test_indented_code() {
        let input = "    code line\n";
        let result = BlockParser::parse(Rule::indented_code_block, input);
        assert!(result.is_ok(), "Should parse indented code block");
    }
    
    // Integration test with multiple block types
    #[test]
    fn smoke_test_complex_document() {
        let input = r#"# Title

This is a paragraph.

```rust
fn main() {}
```

> A quote

---

## Subtitle

    indented code
"#;
        let result = parse_blocks(input);
        assert!(result.is_ok(), "Should parse complex document with multiple block types");
    }
}
