// Render tests: validate HTML output from AST

use super::utils::{print_header, print_section};

pub fn run_render_tests() {
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
                        println!(
                            "  ✗ Expected '<p>This is a paragraph.</p>\\n', got: {:?}",
                            html
                        );
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
                    if html.contains("language-rust")
                        && html.contains("data-language=\"Rust\"")
                        && html.contains("let x = 42;")
                    {
                        println!("  ✓ Code block: ```rust ... ``` → '<pre data-language=\"Rust\"><code class=\"language-rust\">...'");
                        passed += 1;
                    } else {
                        println!(
                            "  ✗ HTML missing language class, data-language label, or code content: {:?}",
                            html
                        );
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
                    if html.contains("Line one")
                        && html.contains("Line two")
                        && html.contains("Line three")
                    {
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
    println!(
        "Render Tests Summary: {}/{} tests passed ({:.1}%)",
        passed,
        total,
        (passed as f64 / total as f64) * 100.0
    );
    if failed > 0 {
        println!("  [WARN] {} test(s) failed", failed);
    }
}

// ============================================================================
// INLINE PARSING TESTS (Markdown with inline elements → AST → HTML)
// ============================================================================

pub fn run_inline_pipeline_tests() {
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
                    if html.contains("<a href=")
                        && html.contains("example.com")
                        && html.contains("example</a>")
                    {
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

    // Test 10: URI Autolink
    print_section("Render URI Autolink");
    total += 1;
    let input = "<https://example.com>";
    match parser::parse(input) {
        Ok(doc) => {
            let options = RenderOptions::default();
            match render_html(&doc, &options) {
                Ok(html) => {
                    if html == "<p><a href=\"https://example.com\">https://example.com</a></p>\n" {
                        println!("  ✓ URI Autolink: '<https://example.com>' → '<a href=\"https://example.com\">https://example.com</a>'");
                        passed += 1;
                    } else {
                        println!("  ✗ Expected '<p><a href=\"https://example.com\">https://example.com</a></p>\\n', got: {:?}", html);
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

    // Test 11: Email Autolink
    print_section("Render Email Autolink");
    total += 1;
    let input = "<user@example.com>";
    match parser::parse(input) {
        Ok(doc) => {
            let options = RenderOptions::default();
            match render_html(&doc, &options) {
                Ok(html) => {
                    if html == "<p><a href=\"mailto:user@example.com\">user@example.com</a></p>\n" {
                        println!("  ✓ Email Autolink: '<user@example.com>' → '<a href=\"mailto:user@example.com\">user@example.com</a>'");
                        passed += 1;
                    } else {
                        println!("  ✗ Expected '<p><a href=\"mailto:user@example.com\">user@example.com</a></p>\\n', got: {:?}", html);
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

    // Test 12: Hard Line Break (spaces)
    print_section("Render Hard Line Break (spaces)");
    total += 1;
    let input = "Line one  \nLine two";
    match parser::parse(input) {
        Ok(doc) => {
            let options = RenderOptions::default();
            match render_html(&doc, &options) {
                Ok(html) => {
                    if html == "<p>Line one<br />\nLine two</p>\n" {
                        println!("  ✓ Hard Break (spaces): 'Line one  \\nLine two' → '<p>Line one<br />\\nLine two</p>'");
                        passed += 1;
                    } else {
                        println!(
                            "  ✗ Expected '<p>Line one<br />\\nLine two</p>\\n', got: {:?}",
                            html
                        );
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

    // Test 13: Hard Line Break (backslash)
    print_section("Render Hard Line Break (backslash)");
    total += 1;
    let input = "Line one\\\nLine two";
    match parser::parse(input) {
        Ok(doc) => {
            let options = RenderOptions::default();
            match render_html(&doc, &options) {
                Ok(html) => {
                    if html == "<p>Line one<br />\nLine two</p>\n" {
                        println!("  ✓ Hard Break (backslash): 'Line one\\\\\\nLine two' → '<p>Line one<br />\\nLine two</p>'");
                        passed += 1;
                    } else {
                        println!(
                            "  ✗ Expected '<p>Line one<br />\\nLine two</p>\\n', got: {:?}",
                            html
                        );
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

    // Test 14: Soft Line Break
    print_section("Render Soft Line Break");
    total += 1;
    let input = "Line one\nLine two";
    match parser::parse(input) {
        Ok(doc) => {
            let options = RenderOptions::default();
            match render_html(&doc, &options) {
                Ok(html) => {
                    if html == "<p>Line one\nLine two</p>\n" {
                        println!(
                            "  ✓ Soft Break: 'Line one\\nLine two' → '<p>Line one\\nLine two</p>'"
                        );
                        passed += 1;
                    } else {
                        println!(
                            "  ✗ Expected '<p>Line one\\nLine two</p>\\n', got: {:?}",
                            html
                        );
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
    println!(
        "Inline Pipeline Tests Summary: {}/{} tests passed ({:.1}%)",
        passed,
        total,
        (passed as f64 / total as f64) * 100.0
    );
    if failed > 0 {
        println!("  [WARN] {} test(s) failed", failed);
    }
}

// ============================================================================
// COMMONMARK SPEC TESTS
// ============================================================================
