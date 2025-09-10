# Marco AST Builders Guide

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Builder Traits](#builder-traits)
4. [Error Handling](#error-handling)
5. [Performance Considerations](#performance-considerations)
6. [Builder Examples](#builder-examples)
7. [Marco Extensions](#marco-extensions)
8. [Testing Strategies](#testing-strategies)
9. [Performance Profiling](#performance-profiling)

## Overview

The Marco AST (Abstract Syntax Tree) builders are a modular system for converting parsed Pest grammar pairs into structured AST nodes. The system is designed with error recovery, performance optimization, and extensibility in mind.

### Key Features

- **Modular Design**: Separate builders for different content types (block, inline, links, tables, Marco extensions)
- **Robust Error Handling**: Multiple error recovery strategies with graceful fallbacks
- **Performance Optimization**: Efficient string handling, minimal allocations, comprehensive profiling
- **Comprehensive Validation**: Input validation with configurable limits
- **Extensible Architecture**: Easy to add new node types and builders

## Architecture

```
AstBuilder (Main Entry Point)
├── BuilderHelpers (Shared utilities)
├── ErrorHandling (Error recovery)
├── BlockBuilder (Block-level elements)
├── InlineBuilder (Inline elements)
├── LinkBuilder (Links and references)
├── TableBuilder (Table structures)
└── MarcoBuilder (Marco-specific extensions)
```

### Core Components

1. **AstBuilder**: Main dispatcher that routes parsing to appropriate specialized builders
2. **BuilderHelpers**: Shared utilities for common operations (span creation, text nodes, etc.)
3. **ErrorHandling**: Advanced error recovery with configurable strategies
4. **Specialized Builders**: Domain-specific builders for different content types

## Builder Traits

### BuilderHelpers

Provides shared functionality across all builders:

```rust
use crate::components::marco_engine::ast::{Node, Span};
use pest::iterators::Pair;

// Create a span from a Pest pair
let span = Self::create_span(&pair);

// Create a text node with efficient string handling
let node = Self::create_text_node("content", span);

// Build wrapper nodes by delegating to inner pairs
let node = Self::build_wrapper_node(pair)?;

// Extract and trim text content efficiently
let content = Self::extract_text_content(text, &['*', '_']);
```

### ErrorHandling

Advanced error recovery with multiple strategies:

```rust
// Determine appropriate recovery strategy
let strategy = Self::get_recovery_strategy(Rule::heading);

// Handle errors according to strategy
match Self::handle_parse_error(error, context, span)? {
    Some(fallback_node) => content.push(fallback_node),
    None => continue, // Skip problematic content
}
```

#### Error Recovery Strategies

1. **Fail**: Critical errors that should stop parsing (document structure)
2. **FallbackToText**: Convert problematic content to text nodes
3. **ContinueWithDefault**: Log error but continue with default content
4. **Skip**: Skip problematic content entirely

## Error Handling

### Strategy Selection

```rust
match rule {
    // Critical structural elements
    Rule::document | Rule::file => ErrorRecoveryStrategy::Fail,
    
    // Block elements can fallback
    Rule::heading | Rule::paragraph => ErrorRecoveryStrategy::FallbackToText,
    
    // Inline elements continue with defaults
    Rule::bold | Rule::italic => ErrorRecoveryStrategy::ContinueWithDefault,
    
    // Unknown elements are skipped
    _ => ErrorRecoveryStrategy::Skip,
}
```

### Example Error Recovery

```rust
// Parse with error recovery
for inner_pair in pair.into_inner() {
    let inner_context = ParseContext::new(&inner_pair, strategy);
    
    match AstBuilder::build_node(inner_pair) {
        Ok(node) => content.push(node),
        Err(e) => {
            match Self::handle_parse_error(e, inner_context, span)? {
                Some(fallback_node) => content.push(fallback_node),
                None => log::debug!("Skipped problematic node"),
            }
        }
    }
}
```

## Performance Considerations

### String Handling

```rust
use std::borrow::Cow;

// Avoid unnecessary allocations with Cow
fn create_text_node_cow(content: Cow<str>, span: Span) -> Node {
    Node::Text {
        content: content.into_owned(), // Only allocate if needed
        span,
    }
}

// Efficient text trimming
fn extract_text_content<'a>(text: &'a str, trim_chars: &[char]) -> Cow<'a, str> {
    // Returns borrowed string if no trimming needed
    // Only allocates if trimming is required
}
```

### Validation Limits

```rust
// Configurable limits for performance and security
pub const MAX_TEXT_LENGTH: usize = 10_000;
pub const MAX_URL_LENGTH: usize = 2_048;
pub const MAX_TABLE_CELLS: usize = 1_000;
pub const MAX_LIST_NESTING: usize = 20;

// Validation example
fn validate_text_content(text: &str) -> Result<()> {
    if text.len() > MAX_TEXT_LENGTH {
        return Err(MarcoError::parse_error(
            format!("Text exceeds maximum length of {}", MAX_TEXT_LENGTH)
        ));
    }
    Ok(())
}
```

## Builder Examples

### BlockBuilder Examples

#### Heading Parsing

```rust
// Input: "# Main Title"
// AST: Heading { level: 1, content: [Text("Main Title")] }

impl BlockBuilder for AstBuilder {
    fn build_heading(pair: Pair<Rule>, span: Span) -> Result<Node> {
        let mut level = 0;
        let mut content = Vec::new();
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::heading_marker => {
                    level = inner_pair.as_str().len();
                    Self::validate_heading_level(level)?;
                }
                Rule::heading_inline => {
                    content = Self::build_content_with_fallback(inner_pair, span.clone())?;
                }
                _ => return Err(MarcoError::parse_error("Unexpected heading content")),
            }
        }
        
        Ok(Node::Heading { level, content, span })
    }
}
```

#### List Parsing with Nesting

```rust
// Input: "1. First item\n   - Nested item\n2. Second item"

fn build_list(pair: Pair<Rule>, span: Span) -> Result<Node> {
    let mut items = Vec::new();
    let mut ordered = false;
    let mut current_depth = 0;
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::list_item => {
                let item = Self::build_list_item(inner_pair)?;
                Self::validate_list_nesting(current_depth)?;
                items.push(item);
            }
            Rule::ordered_marker => ordered = true,
            _ => continue,
        }
    }
    
    Ok(Node::List { ordered, items, span })
}
```

### InlineBuilder Examples

#### Complex Inline Formatting

```rust
// Input: "**bold _italic_ text**"
// AST: Strong { content: [Text("bold "), Emphasis(...), Text(" text")] }

fn build_strong(pair: Pair<Rule>, span: Span) -> Result<Node> {
    let content = Self::build_content_with_fallback(pair, span.clone())?;
    
    // Validate content is not empty
    if content.is_empty() {
        return Ok(Node::Text {
            content: "**".to_string(),
            span,
        });
    }
    
    Ok(Node::Strong { content, span })
}
```

#### Math Expression Parsing

```rust
// Input: "$E = mc^2$"
// AST: Math { expression: "E = mc^2", display_mode: false }

fn build_inline_math(pair: Pair<Rule>, span: Span) -> Result<Node> {
    let content = pair.as_str();
    
    // Extract math content (remove $ delimiters)
    let math_content = content.trim_matches('$');
    Self::validate_math_expression(math_content)?;
    
    Ok(Node::Math {
        expression: math_content.to_string(),
        display_mode: false,
        span,
    })
}
```

### LinkBuilder Examples

#### Complex Link Parsing

```rust
// Input: "[Link text](https://example.com "Title")"
// AST: Link { url: "https://example.com", title: Some("Title"), content: [...] }

fn build_link(pair: Pair<Rule>, span: Span) -> Result<Node> {
    let mut content = Vec::new();
    let mut url = String::new();
    let mut title = None;
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::link_text => {
                content = Self::build_content_with_fallback(inner_pair, span.clone())?;
            }
            Rule::link_url => {
                url = inner_pair.as_str().to_string();
                Self::validate_url(&url)?;
            }
            Rule::link_title => {
                let title_text = inner_pair.as_str().trim_matches('"');
                Self::validate_title(title_text)?;
                title = Some(title_text.to_string());
            }
            _ => continue,
        }
    }
    
    Ok(Node::Link { url, title, content, span })
}
```

## Marco Extensions

### User Mention Example

```rust
// Input: "@username [github](John Doe)"
// AST: UserMention { username: "username", platform: Some("github"), display_name: Some("John Doe") }

fn build_user_mention(pair: Pair<Rule>, span: Span) -> Result<Node> {
    let mut username = String::new();
    let mut platform = None;
    let mut display_name = None;
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::username => {
                username = inner_pair.as_str().to_string();
                Self::validate_username(&username)?;
            }
            Rule::platform => {
                let platform_str = inner_pair.as_str();
                Self::validate_platform(platform_str)?;
                platform = Some(platform_str.to_string());
            }
            Rule::display_name => {
                let name = inner_pair.as_str();
                Self::validate_display_name(name)?;
                display_name = Some(name.to_string());
            }
            _ => continue,
        }
    }
    
    Ok(Node::UserMention {
        username,
        platform,
        display_name,
        span,
    })
}
```

### Bookmark Parsing

```rust
// Input: "[bookmark:section](./file.md=42)"
// AST: Bookmark { label: "section", path: "./file.md", line: Some(42) }

fn build_bookmark(pair: Pair<Rule>, span: Span) -> Result<Node> {
    let mut label = String::new();
    let mut path = String::new();
    let mut line = None;
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::bookmark_label => {
                label = inner_pair.as_str().to_string();
                Self::validate_label(&label)?;
            }
            Rule::bookmark_path => {
                let path_content = inner_pair.as_str();
                
                // Parse path and optional line number
                if let Some((file_path, line_str)) = path_content.split_once('=') {
                    path = file_path.to_string();
                    line = Some(Self::validate_line_number(line_str)?);
                } else {
                    path = path_content.to_string();
                }
                
                Self::validate_path(&path)?;
            }
            _ => continue,
        }
    }
    
    Ok(Node::Bookmark { label, path, line, span })
}
```

### Tabs Block Example

```rust
// Input: ":::\ntabs Example\n@tab First\nContent 1\n@tab Second\nContent 2\n:::"

fn build_tab_block(pair: Pair<Rule>, span: Span) -> Result<Node> {
    let mut title = None;
    let mut tabs = Vec::new();
    let mut current_tab = None;
    let mut current_content = Vec::new();
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::tabs_title => {
                title = Some(inner_pair.as_str().to_string());
            }
            Rule::tab_marker => {
                // Save previous tab if exists
                if let Some(tab_name) = current_tab.take() {
                    tabs.push(TabContent {
                        name: tab_name,
                        content: std::mem::take(&mut current_content),
                    });
                }
                
                // Start new tab
                current_tab = Some(inner_pair.as_str().to_string());
            }
            Rule::tab_content => {
                let content = Self::build_content_with_fallback(inner_pair, span.clone())?;
                current_content.extend(content);
            }
            _ => continue,
        }
    }
    
    // Save final tab
    if let Some(tab_name) = current_tab {
        tabs.push(TabContent {
            name: tab_name,
            content: current_content,
        });
    }
    
    Ok(Node::TabBlock { title, tabs, span })
}
```

## Testing Strategies

### Unit Testing Patterns

```rust
use crate::components::marco_engine::{
    ast::{AstBuilder, Node, Span},
    grammar::{Rule, MarcoParser},
    parser::marco_parser::EnhancedMarcoParser,
};
use pest::Parser;

#[test]
fn test_heading_parsing() {
    let input = "# Main Title";
    let mut pairs = MarcoParser::parse(Rule::heading, input).unwrap();
    let pair = pairs.next().unwrap();
    
    let node = AstBuilder::build_node_for_testing(pair).unwrap();
    
    match node {
        Node::Heading { level, content, .. } => {
            assert_eq!(level, 1);
            assert_eq!(content.len(), 1);
            
            if let Node::Text { content: text, .. } = &content[0] {
                assert_eq!(text, "Main Title");
            } else {
                panic!("Expected text node");
            }
        }
        _ => panic!("Expected heading node"),
    }
}

#[test]
fn test_error_recovery() {
    let input = "**unclosed bold";
    let mut pairs = MarcoParser::parse(Rule::bold, input).unwrap();
    let pair = pairs.next().unwrap();
    
    // Should gracefully handle unclosed formatting
    let node = AstBuilder::build_node_for_testing(pair).unwrap();
    
    // Should fallback to text node
    match node {
        Node::Text { content, .. } => {
            assert_eq!(content, "**unclosed bold");
        }
        _ => panic!("Expected fallback to text node"),
    }
}
```

### Integration Testing

```rust
#[test]
fn test_complex_document() {
    let input = r#"
# Title

This is a paragraph with **bold** and *italic* text.

- List item 1
- List item 2
  - Nested item

@user [github](John Doe) mentioned in [bookmark:section](./file.md=42)
"#;
    
    let node = AstBuilder::build_from_string(input).unwrap();
    
    match node {
        Node::Document { children, .. } => {
            assert_eq!(children.len(), 4); // Title, paragraph, list, mention
            
            // Verify each component
            assert!(matches!(children[0], Node::Heading { level: 1, .. }));
            assert!(matches!(children[1], Node::Paragraph { .. }));
            assert!(matches!(children[2], Node::List { .. }));
            assert!(matches!(children[3], Node::UserMention { .. }));
        }
        _ => panic!("Expected document node"),
    }
}
```

## Performance Profiling

### Using the Performance Infrastructure

```rust
use crate::tests::performance::{PerformanceProfiler, ProfilerConfig};

// Initialize profiler
let config = ProfilerConfig {
    max_samples: 1000,
    enable_cache_monitoring: true,
    enable_rule_tracking: true,
    detailed_timing: true,
};

let mut profiler = PerformanceProfiler::new(config);

// Profile parsing operation
profiler.start_operation("document_parsing");

let start = std::time::Instant::now();
let result = AstBuilder::build_from_string(large_document);
let duration = start.elapsed();

profiler.record_parse_time("document", duration, result.is_ok());
profiler.end_operation("document_parsing");

// Analyze results
let metrics = profiler.get_metrics();
println!("Average parse time: {:.2}ms", metrics.avg_parse_time.as_millis());
println!("Cache hit rate: {:.1}%", metrics.cache_hit_rate * 100.0);

// Export for analysis
let json_report = profiler.export_json().unwrap();
std::fs::write("profile_results.json", json_report).unwrap();
```

### Benchmarking Best Practices

```rust
use crate::tests::performance::MarcoBenchmarks;

#[test]
fn benchmark_parsing_performance() {
    let mut benchmarks = MarcoBenchmarks::new();
    
    // Test various document sizes
    let small_doc = benchmarks.generate_small_document();
    let medium_doc = benchmarks.generate_medium_document();
    let large_doc = benchmarks.generate_large_document();
    
    // Benchmark each size
    for (name, document) in [
        ("small", small_doc),
        ("medium", medium_doc),
        ("large", large_doc),
    ] {
        let start = std::time::Instant::now();
        let result = AstBuilder::build_from_string(&document);
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Parsing failed for {} document", name);
        println!("{} document: {:.2}ms", name, duration.as_millis());
        
        // Performance thresholds
        match name {
            "small" => assert!(duration.as_millis() < 10, "Small doc too slow"),
            "medium" => assert!(duration.as_millis() < 50, "Medium doc too slow"),
            "large" => assert!(duration.as_millis() < 200, "Large doc too slow"),
            _ => {}
        }
    }
}
```

### Memory Profiling

```rust
fn profile_memory_usage() {
    let baseline = get_memory_usage();
    
    // Parse large document
    let large_doc = generate_stress_test_document();
    let result = AstBuilder::build_from_string(&large_doc);
    
    let peak_usage = get_memory_usage();
    let memory_increase = peak_usage - baseline;
    
    // Verify reasonable memory usage
    assert!(memory_increase < 100_000_000, "Memory usage too high: {} bytes", memory_increase);
    
    // Cleanup and verify memory is released
    drop(result);
    std::hint::black_box(()); // Prevent optimization
    
    let after_cleanup = get_memory_usage();
    assert!(after_cleanup < peak_usage, "Memory leak detected");
}
```

## Best Practices

### 1. Error Handling
- Always use appropriate error recovery strategies
- Provide meaningful error messages with context
- Test error scenarios extensively

### 2. Performance
- Minimize string allocations using `Cow<str>`
- Validate input early to prevent expensive operations
- Use profiling infrastructure for optimization

### 3. Extensibility
- Follow existing patterns when adding new builders
- Implement all required traits consistently
- Add comprehensive tests for new functionality

### 4. Validation
- Always validate input according to defined limits
- Provide clear error messages for validation failures
- Consider security implications of input validation

### 5. Testing
- Test both successful and error cases
- Use integration tests for complex scenarios
- Profile performance regularly during development

This guide provides a comprehensive overview of the Marco AST builders system. For specific implementation details, refer to the source code and unit tests in the `tests/` directory.
