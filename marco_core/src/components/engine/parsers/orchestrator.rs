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

use pest::Parser;
use super::block_parser::{BlockParser, Rule as BlockRule};
use super::inline_parser::{InlineParser, Rule as InlineRule};
use crate::components::engine::{
    ast_node::Node,
    builders::{BlockBuilder, InlineBuilder},
};

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

/// Parse markdown document using two-stage architecture and build AST
///
/// # Arguments
/// * `input` - Markdown text to parse
///
/// # Returns
/// * `Ok(Node)` - Root AST node representing the document
/// * `Err(String)` - Parse or build error message
///
/// # Example
/// ```rust,no_run
/// use marco_core::components::engine::parsers::orchestrator::parse_document;
///
/// let markdown = "# Hello\n\nThis is **bold** text.";
/// let ast = parse_document(markdown).unwrap();
/// ```
pub fn parse_document(input: &str) -> Result<Node, String> {
    // Normalize input: ensure it ends with a newline for EOF handling
    // CommonMark requires trailing newlines, but we want to be lenient
    let normalized_input = if input.is_empty() || input.ends_with('\n') {
        input.to_string()
    } else {
        format!("{}\n", input)
    };
    
    // Stage 1: Parse block structure
    let block_pairs = BlockParser::parse(BlockRule::document, &normalized_input)
        .map_err(|e| format!("Block parse error: {}", e))?;
    
    // Stage 2: Build AST using BlockBuilder
    // The BlockBuilder will handle creating the document node and all block-level children
    let mut builder = BlockBuilder::new();
    builder.build_document(block_pairs)
        .map_err(|e| format!("AST build error: {}", e))
}

/// Parse inline content (second stage)
///
/// This function takes a block of text and parses inline elements within it,
/// returning an AST node.
///
/// # Arguments
/// * `input` - Text content from a block to parse inline elements
///
/// # Returns
/// * `Ok(Node)` - AST node containing the inline elements
/// * `Err(String)` - Parse error message
pub fn parse_inline_content(input: &str) -> Result<Node, String> {
    let inline_pairs = InlineParser::parse(InlineRule::inline_content, input)
        .map_err(|e| format!("Inline parse error: {}", e))?;
    
    // Build AST using InlineBuilder
    let builder = InlineBuilder::new();
    let mut children = Vec::new();
    
    for pair in inline_pairs {
        // inline_content is a wrapper rule, we need to process its children
        if pair.as_rule() == InlineRule::inline_content {
            for inner_pair in pair.into_inner() {
                match builder.build_inline_node(inner_pair) {
                    Ok(node) => children.push(node),
                    Err(e) => return Err(format!("Inline AST build error: {}", e)),
                }
            }
        } else {
            match builder.build_inline_node(pair) {
                Ok(node) => children.push(node),
                Err(e) => return Err(format!("Inline AST build error: {}", e)),
            }
        }
    }
    
    // Return a text node containing all inline elements
    // TODO: Better handling of multiple inline elements
    if children.len() == 1 {
        Ok(children.into_iter().next().unwrap())
    } else {
        // Combine into a single text representation for now
        let combined_text = children.iter()
            .map(|n| format!("{:?}", n))
            .collect::<Vec<_>>()
            .join("");
        Ok(Node::text(combined_text, crate::components::engine::ast_node::Span::new(0, 0, 1, 1)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_orchestrator_basic_document() {
        let input = "# Hello World\n\nThis is text.";
        let result = parse_document(input);
        assert!(result.is_ok(), "Should parse basic document");
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
        if let Err(ref e) = result {
            eprintln!("Parse error: {}", e);
        }
        assert!(result.is_ok(), "Should parse inline content");
    }
}
