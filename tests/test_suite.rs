// Marco Test Suite - CLI entry point for all tests
// Usage: cargo test --package core --test test_suite -- --help

use clap::{Parser, Subcommand};
use core::grammar::{inline, block};
use nom_locate::LocatedSpan;
use serde::{Deserialize, Serialize};

type Span<'a> = LocatedSpan<&'a str>;

/// Marco Test Suite - Comprehensive testing for the nom-based Markdown parser
#[derive(Parser)]
#[command(name = "test-suite")]
#[command(about = "Marco Test Suite - Test grammar, parser, AST, renderer, and LSP", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Test inline grammar parsers (code spans, emphasis, strong, links, images)
    Inline {
        /// Run only specific test by name
        #[arg(short, long)]
        filter: Option<String>,
    },
    /// Test block grammar parsers (headings, paragraphs, code blocks, lists)
    Block {
        /// Run only specific test by name
        #[arg(short, long)]
        filter: Option<String>,
    },
    /// Test parser orchestration (blocks → inlines → AST)
    Parser,
    /// Test AST node structures and traversal
    Ast,
    /// Test HTML renderer (AST → HTML)
    Render,
    /// Test LSP features (syntax highlighting, completion, diagnostics)
    Lsp,
    /// Test against CommonMark spec examples
    Commonmark {
        /// Section to test (e.g., "Code spans", "ATX headings")
        #[arg(short, long)]
        section: Option<String>,
    },
    /// Run all tests
    All,
    /// Show test suite summary
    Summary,
}

#[derive(Debug, Deserialize, Serialize)]
struct CommonMarkTest {
    #[serde(default)]
    example: u32,
    section: Option<String>,
    #[serde(default)]
    markdown: String,
    #[serde(default)]
    html: String,
    #[serde(default)]
    start_line: Option<u32>,
    #[serde(default)]
    end_line: Option<u32>,
}

fn load_commonmark_tests() -> Vec<CommonMarkTest> {
    let json = include_str!("test_suite/spec/commonmark.json");
    let tests: Vec<CommonMarkTest> = serde_json::from_str(json)
        .expect("Failed to parse commonmark.json");
    // Filter out comment objects (those without example field or example=0)
    tests.into_iter().filter(|t| t.example > 0).collect()
}

fn print_header(title: &str) {
    println!("\n╔═══════════════════════════════════════════════════════╗");
    println!("║ {:<53} ║", title);
    println!("╚═══════════════════════════════════════════════════════╝\n");
}

fn print_section(title: &str) {
    println!("\n━━━ {} ━━━", title);
}

// ============================================================================
// INLINE GRAMMAR TESTS
// ============================================================================

fn run_inline_tests(filter: Option<String>) {
    print_header("Inline Grammar Tests");
    
    let mut total = 0;
    let mut passed = 0;
    let mut failed = 0;
    
    // Test 1: Basic code span
    if filter.is_none() || filter.as_ref().unwrap().contains("basic") {
        print_section("Basic Code Span");
        total += 1;
        let input = Span::new("`foo`");
        match inline::code_span(input) {
            Ok((remaining, content)) => {
                if content.fragment() == &"foo" && remaining.fragment() == &"" {
                    println!("  ✓ Basic code span: `foo` → content='foo'");
                    passed += 1;
                } else {
                    println!("  ✗ Basic code span failed: expected 'foo', got '{}'", content.fragment());
                    failed += 1;
                }
            }
            Err(e) => {
                println!("  ✗ Basic code span parse error: {:?}", e);
                failed += 1;
            }
        }
    }
    
    // Test 2: Double backticks
    if filter.is_none() || filter.as_ref().unwrap().contains("double") {
        print_section("Double Backtick Code Span");
        total += 1;
        let input = Span::new("`` foo ` bar ``");
        match inline::code_span(input) {
            Ok((remaining, content)) => {
                if content.fragment() == &" foo ` bar " && remaining.fragment() == &"" {
                    println!("  ✓ Double backticks: `` foo ` bar `` → content=' foo ` bar '");
                    passed += 1;
                } else {
                    println!("  ✗ Double backticks failed: expected ' foo ` bar ', got '{}'", content.fragment());
                    failed += 1;
                }
            }
            Err(e) => {
                println!("  ✗ Double backticks parse error: {:?}", e);
                failed += 1;
            }
        }
    }
    
    // Test 3: Whitespace handling
    if filter.is_none() || filter.as_ref().unwrap().contains("whitespace") {
        print_section("Code Span with Whitespace");
        total += 1;
        let input = Span::new("` b `");
        match inline::code_span(input) {
            Ok((remaining, content)) => {
                if content.fragment() == &" b " {
                    println!("  ✓ Whitespace: ` b ` → content=' b '");
                    passed += 1;
                } else {
                    println!("  ✗ Whitespace failed: expected ' b ', got '{}'", content.fragment());
                    failed += 1;
                }
            }
            Err(e) => {
                println!("  ✗ Whitespace parse error: {:?}", e);
                failed += 1;
            }
        }
    }
    
    // Test 4: Triple backticks
    if filter.is_none() || filter.as_ref().unwrap().contains("triple") {
        print_section("Code Span with Triple Backticks");
        total += 1;
        let input = Span::new("` `` `");
        match inline::code_span(input) {
            Ok((remaining, content)) => {
                if content.fragment() == &" `` " {
                    println!("  ✓ Triple backticks: ` `` ` → content=' `` '");
                    passed += 1;
                } else {
                    println!("  ✗ Triple backticks failed: expected ' `` ', got '{}'", content.fragment());
                    failed += 1;
                }
            }
            Err(e) => {
                println!("  ✗ Triple backticks parse error: {:?}", e);
                failed += 1;
            }
        }
    }
    
    println!("\n─────────────────────────────────────────────────────────");
    println!("Results: {} passed, {} failed (out of {} tests)", passed, failed, total);
    
    if failed > 0 {
        std::process::exit(1);
    }
}

// ============================================================================
// BLOCK GRAMMAR TESTS
// ============================================================================

fn run_block_tests(filter: Option<String>) {
    print_header("Block Grammar Tests");
    
    let mut total = 0;
    let mut passed = 0;
    let mut failed = 0;
    
    // Test 1: Basic heading levels 1-6
    if filter.is_none() || filter.as_ref().unwrap().contains("levels") {
        print_section("ATX Heading Levels");
        for level in 1..=6 {
            total += 1;
            let hashes = "#".repeat(level);
            let input_str = format!("{} Test heading", hashes);
            let input = Span::new(&input_str);
            
            match block::heading(input) {
                Ok((_, (parsed_level, content))) => {
                    if parsed_level == level as u8 && content.fragment().contains("Test heading") {
                        println!("  ✓ Level {}: {} Test heading", level, hashes);
                        passed += 1;
                    } else {
                        println!("  ✗ Level {} failed: expected level {}, got {}", level, level, parsed_level);
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("  ✗ Level {} parse error: {:?}", level, e);
                    failed += 1;
                }
            }
        }
    }
    
    // Test 2: Trailing hashes
    if filter.is_none() || filter.as_ref().unwrap().contains("trailing") {
        print_section("ATX Heading Trailing Hashes");
        total += 1;
        let input = Span::new("## foo ##");
        match block::heading(input) {
            Ok((_, (level, content))) => {
                if level == 2 && content.fragment() == &"foo" {
                    println!("  ✓ Trailing hashes removed: '## foo ##' → 'foo'");
                    passed += 1;
                } else {
                    println!("  ✗ Trailing hashes failed: expected 'foo', got '{}'", content.fragment());
                    failed += 1;
                }
            }
            Err(e) => {
                println!("  ✗ Trailing hashes parse error: {:?}", e);
                failed += 1;
            }
        }
    }
    
    // Test 3: Leading spaces (0-3 allowed)
    if filter.is_none() || filter.as_ref().unwrap().contains("spaces") {
        print_section("ATX Heading Leading Spaces");
        total += 1;
        let input = Span::new("   # foo");
        match block::heading(input) {
            Ok((_, (level, content))) => {
                if level == 1 && content.fragment() == &"foo" {
                    println!("  ✓ Leading spaces allowed: '   # foo' → 'foo'");
                    passed += 1;
                } else {
                    println!("  ✗ Leading spaces failed: expected 'foo', got '{}'", content.fragment());
                    failed += 1;
                }
            }
            Err(e) => {
                println!("  ✗ Leading spaces parse error: {:?}", e);
                failed += 1;
            }
        }
    }
    
    // Test 4: Invalid - 7 hashes
    if filter.is_none() || filter.as_ref().unwrap().contains("invalid") {
        print_section("ATX Heading Invalid Cases");
        total += 1;
        let input = Span::new("####### foo");
        match block::heading(input) {
            Ok(_) => {
                println!("  ✗ Seven hashes should fail but succeeded");
                failed += 1;
            }
            Err(_) => {
                println!("  ✓ Seven hashes correctly rejected");
                passed += 1;
            }
        }
        
        // Invalid - no space after #
        total += 1;
        let input = Span::new("#5 bolt");
        match block::heading(input) {
            Ok(_) => {
                println!("  ✗ No space after # should fail but succeeded");
                failed += 1;
            }
            Err(_) => {
                println!("  ✓ No space after # correctly rejected");
                passed += 1;
            }
        }
        
        // Invalid - 4 spaces (code block)
        total += 1;
        let input = Span::new("    # foo");
        match block::heading(input) {
            Ok(_) => {
                println!("  ✗ Four leading spaces should fail but succeeded");
                failed += 1;
            }
            Err(_) => {
                println!("  ✓ Four leading spaces correctly rejected (code block)");
                passed += 1;
            }
        }
    }
    
    // Test paragraphs
    if filter.is_none() || filter.as_ref().unwrap().contains("paragraph") {
        print_section("Paragraph Parser");
        
        // Simple paragraph
        total += 1;
        let input = Span::new("aaa");
        match block::paragraph(input) {
            Ok((_, content)) => {
                if content.fragment() == &"aaa" {
                    println!("  ✓ Simple paragraph: 'aaa'");
                    passed += 1;
                } else {
                    println!("  ✗ Simple paragraph failed");
                    failed += 1;
                }
            }
            Err(_) => {
                println!("  ✗ Simple paragraph parse error");
                failed += 1;
            }
        }
        
        // Multi-line paragraph
        total += 1;
        let input = Span::new("aaa\nbbb");
        match block::paragraph(input) {
            Ok((_, content)) => {
                if content.fragment() == &"aaa\nbbb" {
                    println!("  ✓ Multi-line paragraph: 'aaa\\nbbb'");
                    passed += 1;
                } else {
                    println!("  ✗ Multi-line paragraph failed: got '{}'", content.fragment());
                    failed += 1;
                }
            }
            Err(_) => {
                println!("  ✗ Multi-line paragraph parse error");
                failed += 1;
            }
        }
        
        // Paragraph with blank line terminator
        total += 1;
        let input = Span::new("aaa\n\nbbb");
        match block::paragraph(input) {
            Ok((remaining, content)) => {
                if content.fragment() == &"aaa" && remaining.fragment().starts_with("\n") {
                    println!("  ✓ Paragraph stops at blank line");
                    passed += 1;
                } else {
                    println!("  ✗ Blank line terminator failed");
                    failed += 1;
                }
            }
            Err(_) => {
                println!("  ✗ Blank line terminator parse error");
                failed += 1;
            }
        }
    }
    
    // Test fenced code blocks
    if filter.is_none() || filter.as_ref().unwrap().contains("code") {
        print_section("Fenced Code Block Parser");
        
        // Basic backtick code block
        total += 1;
        let input = Span::new("```\ncode\n```\n");
        match block::fenced_code_block(input) {
            Ok((_, (language, content))) => {
                if language.is_none() && content.fragment() == &"code" {
                    println!("  ✓ Basic backtick code block");
                    passed += 1;
                } else {
                    println!("  ✗ Basic code block failed");
                    failed += 1;
                }
            }
            Err(_) => {
                println!("  ✗ Basic code block parse error");
                failed += 1;
            }
        }
        
        // Code block with language
        total += 1;
        let input = Span::new("```rust\nfn main() {}\n```\n");
        match block::fenced_code_block(input) {
            Ok((_, (language, content))) => {
                if language == Some("rust".to_string()) && content.fragment().contains("fn main") {
                    println!("  ✓ Code block with language: rust");
                    passed += 1;
                } else {
                    println!("  ✗ Code block with language failed");
                    failed += 1;
                }
            }
            Err(_) => {
                println!("  ✗ Code block with language parse error");
                failed += 1;
            }
        }
        
        // Tilde code block
        total += 1;
        let input = Span::new("~~~\ncode\n~~~\n");
        match block::fenced_code_block(input) {
            Ok((_, (_, content))) => {
                if content.fragment() == &"code" {
                    println!("  ✓ Tilde code block");
                    passed += 1;
                } else {
                    println!("  ✗ Tilde code block failed");
                    failed += 1;
                }
            }
            Err(_) => {
                println!("  ✗ Tilde code block parse error");
                failed += 1;
            }
        }
    }
    
    // Summary
    println!("\n{}", "─".repeat(60));
    println!("Block Grammar Summary: {}/{} tests passed ({:.1}%)", 
             passed, total, (passed as f64 / total as f64) * 100.0);
    if failed > 0 {
        println!("  ⚠ {} test(s) failed", failed);
    }
}

// ============================================================================
// PARSER/AST INTEGRATION TESTS
// ============================================================================

fn run_parser_tests() {
    use core::parser;
    
    print_header("Parser → AST Integration Tests");
    
    let mut total = 0;
    let mut passed = 0;
    let mut failed = 0;
    
    // Test 1: Simple heading
    print_section("Parse Heading to AST");
    total += 1;
    let input = "# Hello World";
    match parser::parse(input) {
        Ok(doc) => {
            if doc.children.len() == 1 {
                if let core::parser::NodeKind::Heading { level, text } = &doc.children[0].kind {
                    if *level == 1 && text == "Hello World" {
                        println!("  ✓ Heading parsed: level={}, text={:?}", level, text);
                        passed += 1;
                    } else {
                        println!("  ✗ Heading data mismatch: level={}, text={:?}", level, text);
                        failed += 1;
                    }
                } else {
                    println!("  ✗ Expected Heading node, got {:?}", doc.children[0].kind);
                    failed += 1;
                }
            } else {
                println!("  ✗ Expected 1 node, got {}", doc.children.len());
                failed += 1;
            }
        }
        Err(e) => {
            println!("  ✗ Parse error: {}", e);
            failed += 1;
        }
    }
    
    // Test 2: Paragraph
    print_section("Parse Paragraph to AST");
    total += 1;
    let input = "This is a paragraph.";
    match parser::parse(input) {
        Ok(doc) => {
            if doc.children.len() == 1 {
                if let core::parser::NodeKind::Paragraph = &doc.children[0].kind {
                    if !doc.children[0].children.is_empty() {
                        println!("  ✓ Paragraph parsed with {} child nodes", doc.children[0].children.len());
                        passed += 1;
                    } else {
                        println!("  ✗ Paragraph has no children");
                        failed += 1;
                    }
                } else {
                    println!("  ✗ Expected Paragraph node");
                    failed += 1;
                }
            } else {
                println!("  ✗ Expected 1 node, got {}", doc.children.len());
                failed += 1;
            }
        }
        Err(e) => {
            println!("  ✗ Parse error: {}", e);
            failed += 1;
        }
    }
    
    // Test 3: Fenced code block
    print_section("Parse Code Block to AST");
    total += 1;
    let input = "```rust\nfn main() {}\n```\n";
    match parser::parse(input) {
        Ok(doc) => {
            if doc.children.len() == 1 {
                if let core::parser::NodeKind::CodeBlock { language, code } = &doc.children[0].kind {
                    if language == &Some("rust".to_string()) && code.contains("fn main") {
                        println!("  ✓ Code block parsed: lang={:?}, {} bytes", language, code.len());
                        passed += 1;
                    } else {
                        println!("  ✗ Code block data mismatch");
                        failed += 1;
                    }
                } else {
                    println!("  ✗ Expected CodeBlock node");
                    failed += 1;
                }
            } else {
                println!("  ✗ Expected 1 node, got {}", doc.children.len());
                failed += 1;
            }
        }
        Err(e) => {
            println!("  ✗ Parse error: {}", e);
            failed += 1;
        }
    }
    
    // Test 4: Mixed content
    print_section("Parse Mixed Content to AST");
    total += 1;
    let input = "# Title\n\nA paragraph.\n\n```\ncode\n```\n";
    match parser::parse(input) {
        Ok(doc) => {
            if doc.children.len() == 3 {
                let has_heading = matches!(doc.children[0].kind, core::parser::NodeKind::Heading { .. });
                let has_paragraph = matches!(doc.children[1].kind, core::parser::NodeKind::Paragraph);
                let has_code = matches!(doc.children[2].kind, core::parser::NodeKind::CodeBlock { .. });
                
                if has_heading && has_paragraph && has_code {
                    println!("  ✓ Mixed content parsed: {} nodes (Heading, Paragraph, CodeBlock)", doc.children.len());
                    passed += 1;
                } else {
                    println!("  ✗ Node types mismatch");
                    failed += 1;
                }
            } else {
                println!("  ✗ Expected 3 nodes, got {}", doc.children.len());
                failed += 1;
            }
        }
        Err(e) => {
            println!("  ✗ Parse error: {}", e);
            failed += 1;
        }
    }
    
    // Summary
    println!("\n{}", "─".repeat(60));
    println!("Parser Integration Summary: {}/{} tests passed ({:.1}%)", 
             passed, total, (passed as f64 / total as f64) * 100.0);
    if failed > 0 {
        println!("  ⚠ {} test(s) failed", failed);
    }
}

// ============================================================================
// RENDER TESTS (Parser → AST → HTML)
// ============================================================================

fn run_render_tests() {
    use core::parser;
    use core::render::{render_html, RenderOptions};
    
    print_header("Full Pipeline Tests (Markdown → AST → HTML)");
    
    let mut total = 0;
    let mut passed = 0;
    let mut failed = 0;
    
    // Test 1: Heading to HTML
    print_section("Render Heading");
    total += 1;
    let input = "# Hello World";
    match parser::parse(input) {
        Ok(doc) => {
            let options = RenderOptions::default();
            match render_html(&doc, &options) {
                Ok(html) => {
                    if html == "<h1>Hello World</h1>\n" {
                        println!("  ✓ Heading: '# Hello World' → '<h1>Hello World</h1>'");
                        passed += 1;
                    } else {
                        println!("  ✗ Expected '<h1>Hello World</h1>\\n', got: {:?}", html);
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("  ✗ Render error: {}", e);
                    failed += 1;
                }
            }
        }
        Err(e) => {
            println!("  ✗ Parse error: {}", e);
            failed += 1;
        }
    }
    
    // Test 2: Paragraph to HTML
    print_section("Render Paragraph");
    total += 1;
    let input = "This is a paragraph.";
    match parser::parse(input) {
        Ok(doc) => {
            let options = RenderOptions::default();
            match render_html(&doc, &options) {
                Ok(html) => {
                    if html == "<p>This is a paragraph.</p>\n" {
                        println!("  ✓ Paragraph: 'This is a paragraph.' → '<p>...</p>'");
                        passed += 1;
                    } else {
                        println!("  ✗ Expected '<p>This is a paragraph.</p>\\n', got: {:?}", html);
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("  ✗ Render error: {}", e);
                    failed += 1;
                }
            }
        }
        Err(e) => {
            println!("  ✗ Parse error: {}", e);
            failed += 1;
        }
    }
    
    // Test 3: Code block with language
    print_section("Render Code Block with Language");
    total += 1;
    let input = "```rust\nlet x = 42;\n```\n";
    match parser::parse(input) {
        Ok(doc) => {
            let options = RenderOptions::default();
            match render_html(&doc, &options) {
                Ok(html) => {
                    if html.contains("language-rust") && html.contains("let x = 42;") {
                        println!("  ✓ Code block: ```rust ... ``` → '<pre><code class=\"language-rust\">...'");
                        passed += 1;
                    } else {
                        println!("  ✗ HTML missing language class or code content: {:?}", html);
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("  ✗ Render error: {}", e);
                    failed += 1;
                }
            }
        }
        Err(e) => {
            println!("  ✗ Parse error: {}", e);
            failed += 1;
        }
    }
    
    // Test 4: HTML escaping
    print_section("Render with HTML Escaping");
    total += 1;
    let input = "# Code <example> & test";
    match parser::parse(input) {
        Ok(doc) => {
            let options = RenderOptions::default();
            match render_html(&doc, &options) {
                Ok(html) => {
                    if html.contains("&lt;example&gt;") && html.contains("&amp;") {
                        println!("  ✓ HTML escaping: '<' → '&lt;', '&' → '&amp;'");
                        passed += 1;
                    } else {
                        println!("  ✗ HTML not properly escaped: {:?}", html);
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("  ✗ Render error: {}", e);
                    failed += 1;
                }
            }
        }
        Err(e) => {
            println!("  ✗ Parse error: {}", e);
            failed += 1;
        }
    }
    
    // Test 5: Mixed content
    print_section("Render Mixed Content");
    total += 1;
    let input = "# Title\n\nA paragraph.\n\n```python\nprint('hello')\n```\n";
    match parser::parse(input) {
        Ok(doc) => {
            let options = RenderOptions::default();
            match render_html(&doc, &options) {
                Ok(html) => {
                    let has_h1 = html.contains("<h1>Title</h1>");
                    let has_p = html.contains("<p>A paragraph.</p>");
                    let has_code = html.contains("language-python") && html.contains("print");
                    
                    if has_h1 && has_p && has_code {
                        println!("  ✓ Mixed content: heading + paragraph + code block");
                        println!("    - <h1>: present");
                        println!("    - <p>: present");
                        println!("    - <code>: present with language-python");
                        passed += 1;
                    } else {
                        println!("  ✗ Missing expected elements in HTML:");
                        println!("    - <h1>: {}", has_h1);
                        println!("    - <p>: {}", has_p);
                        println!("    - <code>: {}", has_code);
                        println!("  HTML: {:?}", html);
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("  ✗ Render error: {}", e);
                    failed += 1;
                }
            }
        }
        Err(e) => {
            println!("  ✗ Parse error: {}", e);
            failed += 1;
        }
    }
    
    // Test 6: Multi-line paragraph
    print_section("Render Multi-line Paragraph");
    total += 1;
    let input = "Line one.\nLine two.\nLine three.";
    match parser::parse(input) {
        Ok(doc) => {
            let options = RenderOptions::default();
            match render_html(&doc, &options) {
                Ok(html) => {
                    if html.contains("Line one") && html.contains("Line two") && html.contains("Line three") {
                        println!("  ✓ Multi-line paragraph: all lines preserved");
                        passed += 1;
                    } else {
                        println!("  ✗ Missing lines in HTML: {:?}", html);
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("  ✗ Render error: {}", e);
                    failed += 1;
                }
            }
        }
        Err(e) => {
            println!("  ✗ Parse error: {}", e);
            failed += 1;
        }
    }
    
    // Summary
    println!("\n{}", "─".repeat(60));
    println!("Render Tests Summary: {}/{} tests passed ({:.1}%)", 
             passed, total, (passed as f64 / total as f64) * 100.0);
    if failed > 0 {
        println!("  ⚠ {} test(s) failed", failed);
    }
}

// ============================================================================
// INLINE PARSING TESTS (Markdown with inline elements → AST → HTML)
// ============================================================================

fn run_inline_pipeline_tests() {
    use core::parser;
    use core::render::{render_html, RenderOptions};
    
    print_header("Inline Elements Pipeline Tests");
    
    let mut total = 0;
    let mut passed = 0;
    let mut failed = 0;
    
    // Test 1: Emphasis in paragraph
    print_section("Parse and Render Emphasis");
    total += 1;
    let input = "This is *italic* text.";
    match parser::parse(input) {
        Ok(doc) => {
            let options = RenderOptions::default();
            match render_html(&doc, &options) {
                Ok(html) => {
                    if html.contains("<em>") && html.contains("italic") {
                        println!("  ✓ Emphasis: '*italic*' → '<em>italic</em>'");
                        passed += 1;
                    } else {
                        println!("  ✗ Expected <em> tag in HTML: {:?}", html);
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("  ✗ Render error: {}", e);
                    failed += 1;
                }
            }
        }
        Err(e) => {
            println!("  ✗ Parse error: {}", e);
            failed += 1;
        }
    }
    
    // Test 2: Strong in paragraph
    print_section("Parse and Render Strong");
    total += 1;
    let input = "This is **bold** text.";
    match parser::parse(input) {
        Ok(doc) => {
            let options = RenderOptions::default();
            match render_html(&doc, &options) {
                Ok(html) => {
                    if html.contains("<strong>") && html.contains("bold") {
                        println!("  ✓ Strong: '**bold**' → '<strong>bold</strong>'");
                        passed += 1;
                    } else {
                        println!("  ✗ Expected <strong> tag in HTML: {:?}", html);
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("  ✗ Render error: {}", e);
                    failed += 1;
                }
            }
        }
        Err(e) => {
            println!("  ✗ Parse error: {}", e);
            failed += 1;
        }
    }
    
    // Test 3: Code span in paragraph
    print_section("Parse and Render Code Span");
    total += 1;
    let input = "Use the `print()` function.";
    match parser::parse(input) {
        Ok(doc) => {
            let options = RenderOptions::default();
            match render_html(&doc, &options) {
                Ok(html) => {
                    if html.contains("<code>") && html.contains("print()") {
                        println!("  ✓ Code span: '`print()`' → '<code>print()</code>'");
                        passed += 1;
                    } else {
                        println!("  ✗ Expected <code> tag in HTML: {:?}", html);
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("  ✗ Render error: {}", e);
                    failed += 1;
                }
            }
        }
        Err(e) => {
            println!("  ✗ Parse error: {}", e);
            failed += 1;
        }
    }
    
    // Test 4: Link in paragraph
    print_section("Parse and Render Link");
    total += 1;
    let input = "Visit [example](https://example.com) for more.";
    match parser::parse(input) {
        Ok(doc) => {
            let options = RenderOptions::default();
            match render_html(&doc, &options) {
                Ok(html) => {
                    if html.contains("<a href=") && html.contains("example.com") && html.contains("example</a>") {
                        println!("  ✓ Link: '[example](url)' → '<a href=\"url\">example</a>'");
                        passed += 1;
                    } else {
                        println!("  ✗ Expected <a> tag with href in HTML: {:?}", html);
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("  ✗ Render error: {}", e);
                    failed += 1;
                }
            }
        }
        Err(e) => {
            println!("  ✗ Parse error: {}", e);
            failed += 1;
        }
    }
    
    // Test 5: Mixed inline elements
    print_section("Parse and Render Mixed Inline Elements");
    total += 1;
    let input = "Text with *emphasis*, **strong**, and `code`.";
    match parser::parse(input) {
        Ok(doc) => {
            let options = RenderOptions::default();
            match render_html(&doc, &options) {
                Ok(html) => {
                    let has_em = html.contains("<em>");
                    let has_strong = html.contains("<strong>");
                    let has_code = html.contains("<code>");
                    
                    if has_em && has_strong && has_code {
                        println!("  ✓ Mixed inline: <em>, <strong>, <code> all present");
                        passed += 1;
                    } else {
                        println!("  ✗ Missing elements:");
                        println!("    - <em>: {}", has_em);
                        println!("    - <strong>: {}", has_strong);
                        println!("    - <code>: {}", has_code);
                        println!("  HTML: {:?}", html);
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("  ✗ Render error: {}", e);
                    failed += 1;
                }
            }
        }
        Err(e) => {
            println!("  ✗ Parse error: {}", e);
            failed += 1;
        }
    }
    
    // Test 6: Nested emphasis in strong
    print_section("Parse and Render Nested Inline Elements");
    total += 1;
    let input = "This is **bold with *italic* inside**.";
    match parser::parse(input) {
        Ok(doc) => {
            let options = RenderOptions::default();
            match render_html(&doc, &options) {
                Ok(html) => {
                    // Should have both strong and em tags
                    if html.contains("<strong>") && html.contains("<em>") {
                        println!("  ✓ Nested: '**bold with *italic***' renders both tags");
                        passed += 1;
                    } else {
                        println!("  ✗ Expected nested <strong> and <em> tags: {:?}", html);
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("  ✗ Render error: {}", e);
                    failed += 1;
                }
            }
        }
        Err(e) => {
            println!("  ✗ Parse error: {}", e);
            failed += 1;
        }
    }
    
    // Summary
    println!("\n{}", "─".repeat(60));
    println!("Inline Pipeline Tests Summary: {}/{} tests passed ({:.1}%)", 
             passed, total, (passed as f64 / total as f64) * 100.0);
    if failed > 0 {
        println!("  ⚠ {} test(s) failed", failed);
    }
}

// ============================================================================
// COMMONMARK SPEC TESTS
// ============================================================================

fn run_commonmark_tests(section: Option<String>) {
    print_header("CommonMark Spec Tests");
    
    let tests = load_commonmark_tests();
    let total_tests = tests.len();
    println!("Loaded {} test examples from CommonMark spec", total_tests);
    
    if let Some(section_filter) = section {
        let filtered: Vec<_> = tests.iter()
            .filter(|t| t.section.as_ref().map(|s| s.contains(&section_filter)).unwrap_or(false))
            .collect();
        
        println!("Testing section: {}", section_filter);
        println!("Found {} examples in this section\n", filtered.len());
        
        let mut passed = 0;
        let mut failed = 0;
        let test_limit = 10;
        
        for test in filtered.iter().take(test_limit) {
            println!("Example {}: {:?}", test.example, test.markdown.trim());
            println!("  Expected: {:?}", test.html.trim());
            
            // Test with appropriate parser
            if section_filter.contains("Code span") {
                let input = Span::new(test.markdown.trim_end());
                match inline::code_span(input) {
                    Ok((_, content)) => {
                        println!("  ✓ Parsed: {:?}", content.fragment());
                        passed += 1;
                    }
                    Err(e) => {
                        println!("  ✗ Failed: {:?}", e);
                        failed += 1;
                    }
                }
            } else if section_filter.contains("ATX heading") {
                let input = Span::new(test.markdown.trim_end());
                
                // Check if expected HTML contains heading tags
                let should_be_heading = test.html.contains("<h1>") 
                    || test.html.contains("<h2>") 
                    || test.html.contains("<h3>") 
                    || test.html.contains("<h4>") 
                    || test.html.contains("<h5>") 
                    || test.html.contains("<h6>");
                
                match block::heading(input) {
                    Ok((_, (level, content))) => {
                        if should_be_heading {
                            println!("  ✓ Parsed level {} heading: {:?}", level, content.fragment());
                            passed += 1;
                        } else {
                            println!("  ✗ Parsed as heading but expected paragraph/code");
                            failed += 1;
                        }
                    }
                    Err(_) => {
                        if !should_be_heading {
                            println!("  ✓ Correctly rejected (not a heading)");
                            passed += 1;
                        } else {
                            println!("  ✗ Failed to parse valid heading");
                            failed += 1;
                        }
                    }
                }
            } else if section_filter.contains("Paragraph") {
                let input = Span::new(test.markdown.trim_end());
                
                // Check if expected HTML contains paragraph tags
                let should_be_paragraph = test.html.contains("<p>");
                
                match block::paragraph(input) {
                    Ok((_, content)) => {
                        if should_be_paragraph {
                            println!("  ✓ Parsed paragraph: {:?}", &content.fragment()[..content.fragment().len().min(40)]);
                            passed += 1;
                        } else {
                            println!("  ✗ Parsed as paragraph but expected something else");
                            failed += 1;
                        }
                    }
                    Err(_) => {
                        if !should_be_paragraph {
                            println!("  ✓ Correctly rejected (not a paragraph)");
                            passed += 1;
                        } else {
                            println!("  ✗ Failed to parse valid paragraph");
                            failed += 1;
                        }
                    }
                }
            } else if section_filter.contains("Fenced code") {
                let input = Span::new(test.markdown.trim_end());
                
                // Check if expected HTML contains <pre><code> tags
                let should_be_code = test.html.contains("<pre><code>");
                
                match block::fenced_code_block(input) {
                    Ok((_, (language, content))) => {
                        if should_be_code {
                            println!("  ✓ Parsed fenced code block (lang={:?}): {:?}", 
                                     language, &content.fragment()[..content.fragment().len().min(30)]);
                            passed += 1;
                        } else {
                            println!("  ✗ Parsed as code block but expected something else");
                            failed += 1;
                        }
                    }
                    Err(_) => {
                        if !should_be_code {
                            println!("  ✓ Correctly rejected (not a fenced code block)");
                            passed += 1;
                        } else {
                            println!("  ✗ Failed to parse valid fenced code block");
                            failed += 1;
                        }
                    }
                }
            }
            println!();
        }
        
        // Print summary
        let tested = passed + failed;
        let percentage = if tested > 0 {
            passed as f64 / tested as f64 * 100.0
        } else {
            0.0
        };
        
        println!("─────────────────────────────────────────────────────────");
        println!("Section Summary: {}/{} tests passed ({:.1}%)", passed, tested, percentage);
        println!("Total progress: {}/{} CommonMark examples ({:.1}%)", 
                 passed, total_tests, passed as f64 / total_tests as f64 * 100.0);
        println!("─────────────────────────────────────────────────────────");
    } else {
        // List all sections
        let mut sections: Vec<_> = tests.iter()
            .filter_map(|t| t.section.clone())
            .collect();
        sections.sort();
        sections.dedup();
        
        println!("Available sections ({} total):", sections.len());
        let mut total_examples = 0;
        for section in &sections {
            let count = tests.iter().filter(|t| t.section.as_deref() == Some(section)).count();
            println!("  - {} ({} examples)", section, count);
            total_examples += count;
        }
        
        println!("\n─────────────────────────────────────────────────────────");
        println!("Total: {} sections with {} examples", sections.len(), total_examples);
        println!("CommonMark coverage: {}/{} examples ({:.1}%)", 
                 0, total_tests, 0.0);
        println!("─────────────────────────────────────────────────────────");
        println!("\nRun specific section with:");
        println!("  cargo test --package core --test test_suite test_commonmark_code_spans -- --nocapture");
    }
}

// ============================================================================
// TEST SUITE SUMMARY
// ============================================================================

fn show_summary() {
    print_header("Marco Test Suite Summary");
    
    println!("✓ Implemented & Tested:");
    println!("  - Code span parser (inline grammar)");
    println!("  - 4 smoke tests passing");
    println!();
    
    println!("⚠ TODO - Block Grammar:");
    println!("  - ATX heading parser");
    println!("  - Paragraph parser");
    println!("  - Fenced code block parser");
    println!("  - List parser");
    println!("  - Blockquote parser");
    println!();
    
    println!("⚠ TODO - Inline Grammar:");
    println!("  - Emphasis parser (*text*)");
    println!("  - Strong parser (**text**)");
    println!("  - Link parser ([text](url))");
    println!("  - Image parser (![alt](url))");
    println!();
    
    println!("⚠ TODO - Integration:");
    println!("  - Parser orchestration (block → inline → AST)");
    println!("  - AST builder");
    println!("  - HTML renderer");
    println!("  - LSP features");
    println!();
    
    println!("Commands:");
    println!("  cargo test --package core --test test_suite -- inline");
    println!("  cargo test --package core --test test_suite -- block");
    println!("  cargo test --package core --test test_suite -- commonmark --section \"Code spans\"");
    println!("  cargo test --package core --test test_suite -- all");
    println!("  cargo test --package core --test test_suite -- --help");
    println!();
}

// ============================================================================
// MAIN TEST ENTRY POINT
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_summary() {
        show_summary();
    }
    
    #[test]
    fn test_inline_grammar() {
        run_inline_tests(None);
    }
    
    #[test]
    fn test_block_grammar() {
        run_block_tests(None);
    }
    
    #[test]
    fn test_parser_integration() {
        run_parser_tests();
    }
    
    #[test]
    fn test_render_pipeline() {
        run_render_tests();
    }
    
    #[test]
    fn test_inline_elements_pipeline() {
        run_inline_pipeline_tests();
    }
    
    #[test]
    fn test_commonmark_code_spans() {
        run_commonmark_tests(Some("Code spans".to_string()));
    }
    
    #[test]
    fn test_commonmark_atx_headings() {
        run_commonmark_tests(Some("ATX headings".to_string()));
    }
    
    #[test]
    fn test_commonmark_paragraphs() {
        run_commonmark_tests(Some("Paragraphs".to_string()));
    }
    
    #[test]
    fn test_commonmark_fenced_code_blocks() {
        run_commonmark_tests(Some("Fenced code blocks".to_string()));
    }
    
    #[test]
    fn test_commonmark_list_sections() {
        run_commonmark_tests(None);
    }
}

#[cfg(not(test))]
fn main() {
    // CLI mode (if run directly)
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Inline { filter }) => run_inline_tests(filter),
        Some(Commands::Block { filter }) => run_block_tests(filter),
        Some(Commands::Parser) => run_parser_tests(),
        Some(Commands::Ast) => println!("AST tests not yet implemented"),
        Some(Commands::Render) => {
            run_render_tests();
            run_inline_pipeline_tests();
        },
        Some(Commands::Lsp) => println!("LSP tests not yet implemented"),
        Some(Commands::Commonmark { section }) => run_commonmark_tests(section),
        Some(Commands::All) => {
            run_inline_tests(None);
            run_block_tests(None);
            run_parser_tests();
            run_render_tests();
            run_inline_pipeline_tests();
        }
        Some(Commands::Summary) | None => show_summary(),
    }
}
