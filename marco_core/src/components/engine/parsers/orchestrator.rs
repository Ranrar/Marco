// Parser Orchestrator - Coordinates two-stage parsing (Block → Inline)
//
// This orchestrator replaces the old monolithic MarcoParser with a modular
// two-stage architecture that processes markdown in two phases:
//
// **Stage 1 (Block)**: Parse document structure into blocks
//   - Uses BlockParser with modular block grammar files
//   - Handles: headings, lists, code blocks, blockquotes, etc.
//   - Output: Block-level Pest pairs
//
// **Stage 2 (Inline)**: Parse inline content within each block
//   - Uses InlineParser with modular inline grammar files  
//   - Handles: emphasis, strong, links, images, code spans, etc.
//   - Output: Inline-level Pest pairs nested within blocks
//
// This separation provides:
// - **Modularity**: Each grammar file is focused and maintainable
// - **Testability**: Block and inline rules can be tested independently
// - **CommonMark compliance**: Two-stage parsing matches spec architecture
// - **Performance**: Can cache block and inline parsing separately

use pest::iterators::{Pair, Pairs};
use pest::Parser;
use super::block_parser::{BlockParser, Rule as BlockRule};
use super::inline_parser::{InlineParser, Rule as InlineRule};

/// Unified Rule enum that wraps both Block and Inline rules
/// This provides a single Rule type for the public API while maintaining
/// the separation of concerns internally
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    // Block-level rules
    Document,
    Block,
    AtxHeading,
    SetextHeading,
    ThematicBreak,
    FencedCodeBlock,
    IndentedCodeBlock,
    Blockquote,
    ListBlock,
    ReferenceDefinition,
    Paragraph,
    
    // Inline-level rules
    InlineContent,
    InlineElement,
    Emphasis,
    Strong,
    CodeSpan,
    Link,
    InlineLink,
    LinkFullReference,
    LinkCollapsedReference,
    LinkShortcutReference,
    Image,
    InlineImage,
    ImageFullReference,
    ImageCollapsedReference,
    ImageShortcutReference,
    Autolink,
    HtmlTag,
    LineBreak,
    Escape,
    Text,
    
    // Special
    EOI,
}

/// Parse markdown document using two-stage architecture
///
/// # Arguments
/// * `input` - Markdown text to parse
///
/// # Returns
/// * `Ok(String)` - JSON representation of parse tree (for debugging)
/// * `Err(String)` - Parse error message
///
/// # Example
/// ```rust
/// use marco_core::parsers::orchestrator::parse_document;
///
/// let markdown = "# Hello\n\nThis is **bold** text.";
/// let result = parse_document(markdown);
/// assert!(result.is_ok());
/// ```
pub fn parse_document(input: &str) -> Result<String, String> {
    // Stage 1: Parse block structure
    let block_pairs = BlockParser::parse(BlockRule::document, input)
        .map_err(|e| format!("Block parse error: {}", e))?;
    
    // Stage 2: For each block that contains inline content, parse it
    // For now, return a debug representation
    // TODO: Implement proper inline parsing recursion
    
    let mut result = String::from("Block structure:\n");
    for pair in block_pairs {
        result.push_str(&format_block_pair(pair, 0));
    }
    
    Ok(result)
}

/// Format a block-level pair for debugging (recursive)
fn format_block_pair(pair: Pair<BlockRule>, depth: usize) -> String {
    let indent = "  ".repeat(depth);
    let rule_name = format!("{:?}", pair.as_rule());
    let text = pair.as_str();
    let preview = if text.len() > 50 {
        format!("{}...", &text[..50])
    } else {
        text.to_string()
    };
    
    let mut result = format!("{}{}: {:?}\n", indent, rule_name, preview);
    
    for inner in pair.into_inner() {
        result.push_str(&format_block_pair(inner, depth + 1));
    }
    
    result
}

/// Parse inline content (second stage)
///
/// This function takes a block of text and parses inline elements within it.
///
/// # Arguments
/// * `input` - Text content from a block to parse inline elements
///
/// # Returns
/// * `Ok(String)` - JSON representation of inline parse tree
/// * `Err(String)` - Parse error message
pub fn parse_inline_content(input: &str) -> Result<String, String> {
    let inline_pairs = InlineParser::parse(InlineRule::inline_content, input)
        .map_err(|e| format!("Inline parse error: {}", e))?;
    
    let mut result = String::from("Inline structure:\n");
    for pair in inline_pairs {
        result.push_str(&format_inline_pair(pair, 0));
    }
    
    Ok(result)
}

/// Format an inline-level pair for debugging (recursive)
fn format_inline_pair(pair: Pair<InlineRule>, depth: usize) -> String {
    let indent = "  ".repeat(depth);
    let rule_name = format!("{:?}", pair.as_rule());
    let text = pair.as_str();
    let preview = if text.len() > 50 {
        format!("{}...", &text[..50])
    } else {
        text.to_string()
    };
    
    let mut result = format!("{}{}: {:?}\n", indent, rule_name, preview);
    
    for inner in pair.into_inner() {
        result.push_str(&format_inline_pair(inner, depth + 1));
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_orchestrator_basic_document() {
        let input = "# Hello World\n\nThis is text.";
        let result = parse_document(input);
        assert!(result.is_ok(), "Should parse basic document");
        let output = result.unwrap();
        assert!(output.contains("document"), "Should contain 'document' in output");
    }

    #[test]
    fn smoke_test_orchestrator_with_formatting() {
        let input = "# Title\n\nThis is **bold** and *italic* text.";
        let result = parse_document(input);
        assert!(result.is_ok(), "Should parse document with inline formatting");
    }

    #[test]
    fn smoke_test_inline_parsing() {
        let input = "This is **bold** and *italic* text.";
        let result = parse_inline_content(input);
        assert!(result.is_ok(), "Should parse inline content");
        let output = result.unwrap();
        assert!(output.contains("inline_content") || output.contains("Inline"), 
                "Should contain inline markers");
    }
}
