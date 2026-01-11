// Grammar tests: validate nom parsers for block and inline syntax

use super::utils::{print_header, print_section, Span};
use core::grammar::{blocks, inlines};

pub fn run_inline_tests(filter: Option<String>) {
    print_header("Inline Grammar Tests");

    let mut total = 0;
    let mut passed = 0;
    let mut failed = 0;

    // Test 1: Basic code span
    if filter.is_none() || filter.as_ref().unwrap().contains("basic") {
        print_section("Basic Code Span");
        total += 1;
        let input = Span::new("`foo`");
        match inlines::code_span(input) {
            Ok((remaining, content)) => {
                if content.fragment() == &"foo" && remaining.fragment() == &"" {
                    println!("  ✓ Basic code span: `foo` → content='foo'");
                    passed += 1;
                } else {
                    println!(
                        "  ✗ Basic code span failed: expected 'foo', got '{}'",
                        content.fragment()
                    );
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
        match inlines::code_span(input) {
            Ok((remaining, content)) => {
                if content.fragment() == &" foo ` bar " && remaining.fragment() == &"" {
                    println!("  ✓ Double backticks: `` foo ` bar `` → content=' foo ` bar '");
                    passed += 1;
                } else {
                    println!(
                        "  ✗ Double backticks failed: expected ' foo ` bar ', got '{}'",
                        content.fragment()
                    );
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
        match inlines::code_span(input) {
            Ok((_remaining, content)) => {
                if content.fragment() == &" b " {
                    println!("  ✓ Whitespace: ` b ` → content=' b '");
                    passed += 1;
                } else {
                    println!(
                        "  ✗ Whitespace failed: expected ' b ', got '{}'",
                        content.fragment()
                    );
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
        match inlines::code_span(input) {
            Ok((_remaining, content)) => {
                if content.fragment() == &" `` " {
                    println!("  ✓ Triple backticks: ` `` ` → content=' `` '");
                    passed += 1;
                } else {
                    println!(
                        "  ✗ Triple backticks failed: expected ' `` ', got '{}'",
                        content.fragment()
                    );
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
    println!(
        "Results: {} passed, {} failed (out of {} tests)",
        passed, failed, total
    );

    if failed > 0 {
        std::process::exit(1);
    }
}

// ============================================================================
// BLOCK GRAMMAR TESTS
// ============================================================================

pub fn run_block_tests(filter: Option<String>) {
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

            match blocks::heading(input) {
                Ok((_, (parsed_level, content))) => {
                    if parsed_level == level as u8 && content.fragment().contains("Test heading") {
                        println!("  ✓ Level {}: {} Test heading", level, hashes);
                        passed += 1;
                    } else {
                        println!(
                            "  ✗ Level {} failed: expected level {}, got {}",
                            level, level, parsed_level
                        );
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
        match blocks::heading(input) {
            Ok((_, (level, content))) => {
                if level == 2 && content.fragment() == &"foo" {
                    println!("  ✓ Trailing hashes removed: '## foo ##' → 'foo'");
                    passed += 1;
                } else {
                    println!(
                        "  ✗ Trailing hashes failed: expected 'foo', got '{}'",
                        content.fragment()
                    );
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
        match blocks::heading(input) {
            Ok((_, (level, content))) => {
                if level == 1 && content.fragment() == &"foo" {
                    println!("  ✓ Leading spaces allowed: '   # foo' → 'foo'");
                    passed += 1;
                } else {
                    println!(
                        "  ✗ Leading spaces failed: expected 'foo', got '{}'",
                        content.fragment()
                    );
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
        match blocks::heading(input) {
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
        match blocks::heading(input) {
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
        match blocks::heading(input) {
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
        match blocks::paragraph(input) {
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
        match blocks::paragraph(input) {
            Ok((_, content)) => {
                if content.fragment() == &"aaa\nbbb" {
                    println!("  ✓ Multi-line paragraph: 'aaa\\nbbb'");
                    passed += 1;
                } else {
                    println!(
                        "  ✗ Multi-line paragraph failed: got '{}'",
                        content.fragment()
                    );
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
        match blocks::paragraph(input) {
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
        match blocks::fenced_code_block(input) {
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
        match blocks::fenced_code_block(input) {
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
        match blocks::fenced_code_block(input) {
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
    println!(
        "Block Grammar Summary: {}/{} tests passed ({:.1}%)",
        passed,
        total,
        (passed as f64 / total as f64) * 100.0
    );
    if failed > 0 {
        println!("  ⚠ {} test(s) failed", failed);
    }
}

// ============================================================================
// PARSER/AST INTEGRATION TESTS
// ============================================================================
