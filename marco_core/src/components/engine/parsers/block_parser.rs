use pest::Parser;
use pest::iterators::Pairs;
use pest_derive::Parser;

/// Block-level parser with modular grammar files
/// Each grammar file is loaded via a separate #[grammar] attribute
/// Pest will compose all rules into a single Rule enum
#[derive(Parser)]
#[grammar = "components/engine/grammar/block/_core.pest"]
#[grammar = "components/engine/grammar/block/thematic_break.pest"]
#[grammar = "components/engine/grammar/block/atx_heading.pest"]
#[grammar = "components/engine/grammar/block/setext_heading.pest"]
#[grammar = "components/engine/grammar/block/fenced_code_block.pest"]
#[grammar = "components/engine/grammar/block/indented_code_block.pest"]
#[grammar = "components/engine/grammar/block/html_block.pest"]
#[grammar = "components/engine/grammar/block/blockquote.pest"]
#[grammar = "components/engine/grammar/block/list.pest"]
#[grammar = "components/engine/grammar/block/reference_definition.pest"]
#[grammar = "components/engine/grammar/block/document.pest"]
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
    fn smoke_test_atx_heading_without_trailing_newline() {
        let input = "# Hello World";
        let result = BlockParser::parse(Rule::atx_heading, input);
        assert!(result.is_ok(), "Should parse ATX heading without trailing newline (EOF case)");
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
    
    #[test]
    fn smoke_test_document_without_trailing_newline() {
        // Note: This test uses the orchestrator which normalizes input
        use crate::components::engine::parsers::orchestrator;
        
        let input = "# Heading\n\nThis is a paragraph.";
        let result = orchestrator::parse_document(input);
        match &result {
            Ok(_) => println!("✅ Document without trailing newline parsed successfully"),
            Err(e) => println!("❌ Parse error: {}", e),
        }
        assert!(result.is_ok(), "Should parse document without trailing newline via orchestrator");
    }
    
    // Fenced code block tests
    #[test]
    fn smoke_test_fenced_code_backticks() {
        // When testing individual rules, no trailing newline after block
        let input = "```\ncode here\n```";
        let result = BlockParser::parse(Rule::fenced_code_block, input);
        match &result {
            Ok(_) => {},
            Err(e) => eprintln!("Fenced code parse error: {}", e),
        }
        assert!(result.is_ok(), "Should parse backtick fenced code");
    }
    
    #[test]
    fn smoke_test_fenced_code_tildes() {
        // When testing individual rules, no trailing newline after block
        let input = "~~~\ncode here\n~~~";
        let result = BlockParser::parse(Rule::fenced_code_block, input);
        match &result {
            Ok(_) => {},
            Err(e) => eprintln!("Fenced code parse error: {}", e),
        }
        assert!(result.is_ok(), "Should parse tilde fenced code");
    }
    
    #[test]
    fn smoke_test_fenced_code_with_info() {
        // When testing individual rules, no trailing newline after block
        let input = "```rust\nfn main() {}\n```";
        let result = BlockParser::parse(Rule::fenced_code_block, input);
        match &result {
            Ok(_) => {},
            Err(e) => eprintln!("Fenced code parse error: {}", e),
        }
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
    
    // List tests - Phase 6
    #[test]
    fn smoke_test_bullet_list_dash() {
        let input = "- First item\n- Second item\n";
        let result = BlockParser::parse(Rule::list, input);
        assert!(result.is_ok(), "Should parse dash bullet list: {:?}", result.err());
    }
    
    #[test]
    fn smoke_test_bullet_list_asterisk() {
        let input = "* Item one\n* Item two\n";
        let result = BlockParser::parse(Rule::list, input);
        assert!(result.is_ok(), "Should parse asterisk bullet list: {:?}", result.err());
    }
    
    #[test]
    fn smoke_test_ordered_list() {
        let input = "1. First\n2. Second\n3. Third\n";
        let result = BlockParser::parse(Rule::list, input);
        assert!(result.is_ok(), "Should parse ordered list: {:?}", result.err());
    }
    
    #[test]
    fn smoke_test_task_list() {
        let input = "- [ ] Unchecked task\n- [x] Checked task\n";
        let result = BlockParser::parse(Rule::task_list, input);
        assert!(result.is_ok(), "Should parse task list: {:?}", result.err());
    }
    
    #[test]
    fn smoke_test_list_markers() {
        // Test bullet markers
        let dash = BlockParser::parse(Rule::bullet_list_marker, "- ");
        assert!(dash.is_ok(), "Should parse dash marker: {:?}", dash.err());
        
        let plus = BlockParser::parse(Rule::bullet_list_marker, "+ ");
        assert!(plus.is_ok(), "Should parse plus marker: {:?}", plus.err());
        
        let asterisk = BlockParser::parse(Rule::bullet_list_marker, "* ");
        assert!(asterisk.is_ok(), "Should parse asterisk marker: {:?}", asterisk.err());
        
        // Test ordered markers
        let dot = BlockParser::parse(Rule::ordered_list_marker, "1. ");
        assert!(dot.is_ok(), "Should parse dot delimiter: {:?}", dot.err());
        
        let paren = BlockParser::parse(Rule::ordered_list_marker, "1) ");
        assert!(paren.is_ok(), "Should parse paren delimiter: {:?}", paren.err());
    }

    // ============================================================
    // Phase 7: Reference Definitions Tests
    // ============================================================

    #[test]
    fn smoke_test_reference_definition_basic() {
        let input = "[foo]: /url\n";
        let result = BlockParser::parse(Rule::reference_definition, input);
        assert!(result.is_ok(), "Should parse basic reference definition: {:?}", result.err());
    }

    #[test]
    fn smoke_test_reference_definition_with_title() {
        let input = "[foo]: /url \"title\"\n";
        let result = BlockParser::parse(Rule::reference_definition, input);
        assert!(result.is_ok(), "Should parse reference with title: {:?}", result.err());
    }

    #[test]
    fn smoke_test_reference_definition_angle_bracket() {
        let input = "[foo]: <http://example.com>\n";
        let result = BlockParser::parse(Rule::reference_definition, input);
        assert!(result.is_ok(), "Should parse angle bracket destination: {:?}", result.err());
    }

    #[test]
    fn smoke_test_reference_definition_single_quote_title() {
        let input = "[foo]: /url 'title'\n";
        let result = BlockParser::parse(Rule::reference_definition, input);
        assert!(result.is_ok(), "Should parse single quote title: {:?}", result.err());
    }

    #[test]
    fn smoke_test_reference_definition_paren_title() {
        let input = "[foo]: /url (title)\n";
        let result = BlockParser::parse(Rule::reference_definition, input);
        assert!(result.is_ok(), "Should parse paren title: {:?}", result.err());
    }

    // ============================================================
    // Phase 3: HTML Blocks Tests (7 types)
    // ============================================================

    #[test]
    fn smoke_test_html_block_type1_script() {
        let input = "<script>\nalert('hello');\n</script>\n";
        let result = BlockParser::parse(Rule::html_block_type1, input);
        assert!(result.is_ok(), "Should parse Type 1 script tag: {:?}", result.err());
    }

    #[test]
    fn smoke_test_html_block_type1_pre() {
        let input = "<pre>\npreformatted text\n</pre>\n";
        let result = BlockParser::parse(Rule::html_block_type1, input);
        assert!(result.is_ok(), "Should parse Type 1 pre tag: {:?}", result.err());
    }

    #[test]
    fn smoke_test_html_block_type1_style() {
        let input = "<style>\nbody { color: red; }\n</style>\n";
        let result = BlockParser::parse(Rule::html_block_type1, input);
        assert!(result.is_ok(), "Should parse Type 1 style tag: {:?}", result.err());
    }

    #[test]
    fn smoke_test_html_block_type1_textarea() {
        let input = "<textarea>\ntext content\n</textarea>\n";
        let result = BlockParser::parse(Rule::html_block_type1, input);
        assert!(result.is_ok(), "Should parse Type 1 textarea tag: {:?}", result.err());
    }

    #[test]
    fn smoke_test_html_block_type2_comment() {
        let input = "<!-- This is a comment -->\n";
        let result = BlockParser::parse(Rule::html_block_type2, input);
        assert!(result.is_ok(), "Should parse Type 2 HTML comment: {:?}", result.err());
    }

    #[test]
    fn smoke_test_html_block_type2_multiline_comment() {
        let input = "<!-- This is\na multi-line\ncomment -->\n";
        let result = BlockParser::parse(Rule::html_block_type2, input);
        assert!(result.is_ok(), "Should parse Type 2 multi-line comment: {:?}", result.err());
    }

    #[test]
    fn smoke_test_html_block_type3_processing_instruction() {
        let input = "<?xml version=\"1.0\"?>\n";
        let result = BlockParser::parse(Rule::html_block_type3, input);
        assert!(result.is_ok(), "Should parse Type 3 processing instruction: {:?}", result.err());
    }

    #[test]
    fn smoke_test_html_block_type4_declaration() {
        let input = "<!DOCTYPE html>\n";
        let result = BlockParser::parse(Rule::html_block_type4, input);
        assert!(result.is_ok(), "Should parse Type 4 declaration: {:?}", result.err());
    }

    #[test]
    fn smoke_test_html_block_type5_cdata() {
        let input = "<![CDATA[data here]]>\n";
        let result = BlockParser::parse(Rule::html_block_type5, input);
        assert!(result.is_ok(), "Should parse Type 5 CDATA: {:?}", result.err());
    }

    #[test]
    fn smoke_test_html_block_type6_div() {
        let input = "<div>\ncontent\n</div>\n\n";
        let result = BlockParser::parse(Rule::html_block_type6, input);
        assert!(result.is_ok(), "Should parse Type 6 div block: {:?}", result.err());
    }

    #[test]
    fn smoke_test_html_block_type6_table() {
        let input = "<table>\n<tr><td>cell</td></tr>\n</table>\n\n";
        let result = BlockParser::parse(Rule::html_block_type6, input);
        assert!(result.is_ok(), "Should parse Type 6 table block: {:?}", result.err());
    }

    #[test]
    fn smoke_test_html_block_type7_span() {
        let input = "<span>text</span>\n\n";
        let result = BlockParser::parse(Rule::html_block_type7, input);
        assert!(result.is_ok(), "Should parse Type 7 span tag: {:?}", result.err());
    }

    #[test]
    fn smoke_test_html_block_type7_self_closing() {
        let input = "<br />\n\n";
        let result = BlockParser::parse(Rule::html_block_type7, input);
        assert!(result.is_ok(), "Should parse Type 7 self-closing tag: {:?}", result.err());
    }

    #[test]
    fn smoke_test_html_block_dispatcher() {
        // Test the main html_block dispatcher with different types
        let type1 = "<script>code</script>\n";
        assert!(BlockParser::parse(Rule::html_block, type1).is_ok(), "html_block should match type1");
        
        let type2 = "<!-- comment -->\n";
        assert!(BlockParser::parse(Rule::html_block, type2).is_ok(), "html_block should match type2");
        
        let type6 = "<div>content</div>\n\n";
        assert!(BlockParser::parse(Rule::html_block, type6).is_ok(), "html_block should match type6");
    }

    #[test]
    fn smoke_test_html_block_case_insensitive() {
        // HTML tags should be case-insensitive
        let upper = "<SCRIPT>code</SCRIPT>\n";
        assert!(BlockParser::parse(Rule::html_block_type1, upper).is_ok(), "Should parse uppercase SCRIPT");
        
        let mixed = "<ScRiPt>code</script>\n";
        assert!(BlockParser::parse(Rule::html_block_type1, mixed).is_ok(), "Should parse mixed case script");
    }
}
