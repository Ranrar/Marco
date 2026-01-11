// Example Runner - Deep inspection of specific test examples
// Shows the complete pipeline: grammar → parser → AST → render

use super::commonmark_tests::{load_commonmark_tests, load_extra_tests, CommonMarkTest};
use super::utils::print_header;
use std::fmt::Write;

/// Run detailed inspection for specific example numbers
pub fn run_example_inspection(example_numbers: Vec<u32>) {
    print_header("Example Deep Inspection");

    // Load all tests (CommonMark + Extra)
    let mut all_tests = load_commonmark_tests();
    all_tests.extend(load_extra_tests());

    if example_numbers.is_empty() {
        println!("No example numbers provided.");
        println!("\nUsage:");
        println!("  cargo test --package core --test test_suite -- inspect --examples 307,318,653");
        return;
    }

    println!(
        "Inspecting {} example(s): {:?}\n",
        example_numbers.len(),
        example_numbers
    );

    for example_num in example_numbers {
        let test = all_tests.iter().find(|t| t.example == example_num);

        match test {
            Some(test) => {
                inspect_example(test);
            }
            None => {
                println!("═══════════════════════════════════════════════════════════");
                println!("Example {} NOT FOUND", example_num);
                println!("═══════════════════════════════════════════════════════════\n");
            }
        }
    }
}

fn inspect_example(test: &CommonMarkTest) {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!(
        "║  Example {} - {}",
        test.example,
        test.section
            .as_ref()
            .unwrap_or(&"Unknown section".to_string())
    );
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    // 1. Show Input
    println!("┌─ INPUT MARKDOWN ─────────────────────────────────────────┐");
    println!("{}", format_multiline(&test.markdown, "│ "));
    println!("└──────────────────────────────────────────────────────────┘\n");

    // 2. Show Expected Output
    println!("┌─ EXPECTED HTML ──────────────────────────────────────────┐");
    println!("{}", format_multiline(&test.html, "│ "));
    println!("└──────────────────────────────────────────────────────────┘\n");

    // 3. Parse and show results at each stage
    match core::parser::parse(&test.markdown) {
        Ok(document) => {
            // Show AST
            println!("┌─ PARSED AST ─────────────────────────────────────────────┐");
            let ast_debug = format!("{:#?}", document);
            println!("{}", format_multiline(&ast_debug, "│ "));
            println!("└──────────────────────────────────────────────────────────┘\n");

            // Show AST structure (simplified tree view)
            println!("┌─ AST TREE STRUCTURE ─────────────────────────────────────┐");
            let tree = format_ast_tree(&document);
            println!("{}", format_multiline(&tree, "│ "));
            println!("└──────────────────────────────────────────────────────────┘\n");

            // Render HTML
            let options = core::render::RenderOptions::default();
            match core::render::render(&document, &options) {
                Ok(rendered_html) => {
                    println!("┌─ RENDERED HTML ──────────────────────────────────────────┐");
                    println!("{}", format_multiline(&rendered_html, "│ "));
                    println!("└──────────────────────────────────────────────────────────┘\n");

                    // Compare
                    println!("┌─ COMPARISON ─────────────────────────────────────────────┐");
                    let expected_normalized = test.html.trim();
                    let actual_normalized = rendered_html.trim();

                    if expected_normalized == actual_normalized {
                        println!("│ ✓ PASS - Output matches expected HTML");
                    } else {
                        println!("│ ✗ FAIL - Output differs from expected HTML");
                        println!("│");
                        println!("│ Differences:");
                        show_diff(expected_normalized, actual_normalized);
                    }
                    println!("└──────────────────────────────────────────────────────────┘\n");
                }
                Err(e) => {
                    println!("┌─ RENDER ERROR ───────────────────────────────────────────┐");
                    println!("│ {}", e);
                    println!("└──────────────────────────────────────────────────────────┘\n");
                }
            }
        }
        Err(e) => {
            println!("┌─ PARSE ERROR ────────────────────────────────────────────┐");
            println!("│ {}", e);
            println!("└──────────────────────────────────────────────────────────┘\n");
        }
    }

    println!("═══════════════════════════════════════════════════════════\n");
}

/// Format AST as a tree structure
fn format_ast_tree(document: &core::parser::ast::Document) -> String {
    let mut output = String::new();
    writeln!(output, "Document").unwrap();

    for (i, child) in document.children.iter().enumerate() {
        let is_last = i == document.children.len() - 1;
        format_node_tree(child, "", is_last, &mut output);
    }

    output
}

fn format_node_tree(
    node: &core::parser::ast::Node,
    prefix: &str,
    is_last: bool,
    output: &mut String,
) {
    use core::parser::ast::NodeKind;

    let connector = if is_last { "└─ " } else { "├─ " };
    let extension = if is_last { "   " } else { "│  " };

    // Format node type and key info
    let node_info = match &node.kind {
        NodeKind::Heading { level, text } => {
            format!("Heading(level={}) \"{}\"", level, truncate(text, 40))
        }
        NodeKind::Paragraph => "Paragraph".to_string(),
        NodeKind::Text(text) => format!("Text \"{}\"", truncate(text, 40)),
        NodeKind::CodeBlock { language, code } => {
            format!("CodeBlock(lang={:?}) {} bytes", language, code.len())
        }
        NodeKind::List {
            ordered,
            start,
            tight,
        } => {
            format!(
                "List({}ordered, tight={}, start={:?})",
                if *ordered { "" } else { "un" },
                tight,
                start
            )
        }
        NodeKind::ListItem => "ListItem".to_string(),
        NodeKind::Blockquote => "Blockquote".to_string(),
        NodeKind::ThematicBreak => "ThematicBreak".to_string(),
        NodeKind::Table => "Table".to_string(),
        NodeKind::HtmlBlock { html } => format!("HtmlBlock {} bytes", html.len()),
        NodeKind::Emphasis => "Emphasis".to_string(),
        NodeKind::Strong => "Strong".to_string(),
        NodeKind::StrongEmphasis => "StrongEmphasis".to_string(),
        NodeKind::Strikethrough => "Strikethrough".to_string(),
        NodeKind::Mark => "Mark".to_string(),
        NodeKind::Superscript => "Superscript".to_string(),
        NodeKind::Subscript => "Subscript".to_string(),
        NodeKind::CodeSpan(code) => format!("CodeSpan \"{}\"", truncate(code, 40)),
        NodeKind::Link { url, title } => {
            format!("Link(url=\"{}\", title={:?})", truncate(url, 30), title)
        }
        NodeKind::Image { url, alt } => {
            format!(
                "Image(url=\"{}\", alt=\"{}\")",
                truncate(url, 20),
                truncate(alt, 20)
            )
        }
        NodeKind::InlineHtml(html) => format!("InlineHtml \"{}\"", truncate(html, 40)),
        NodeKind::HardBreak => "HardBreak".to_string(),
        NodeKind::SoftBreak => "SoftBreak".to_string(),
    };

    writeln!(output, "{}{}{}", prefix, connector, node_info).unwrap();

    // Recurse for children
    let new_prefix = format!("{}{}", prefix, extension);
    for (i, child) in node.children.iter().enumerate() {
        let is_last_child = i == node.children.len() - 1;
        format_node_tree(child, &new_prefix, is_last_child, output);
    }
}

/// Format multiline text with a prefix
fn format_multiline(text: &str, prefix: &str) -> String {
    text.lines()
        .map(|line| format!("{}{}", prefix, line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Truncate text to max length with ellipsis
fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len.saturating_sub(3)])
    }
}

/// Show character-by-character diff
fn show_diff(expected: &str, actual: &str) {
    println!("│");

    // Split into lines and compare
    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();

    let max_lines = expected_lines.len().max(actual_lines.len());

    for i in 0..max_lines {
        let exp_line = expected_lines.get(i).copied().unwrap_or("");
        let act_line = actual_lines.get(i).copied().unwrap_or("");

        if exp_line != act_line {
            println!("│ Line {}:", i + 1);
            println!("│   Expected: {:?}", exp_line);
            println!("│   Actual:   {:?}", act_line);
        }
    }

    if expected_lines.len() != actual_lines.len() {
        println!("│");
        println!("│ Line count mismatch:");
        println!("│   Expected: {} lines", expected_lines.len());
        println!("│   Actual:   {} lines", actual_lines.len());
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_example_runner_smoke() {
        // Smoke test: run example inspection on a known example
        super::run_example_inspection(vec![1]); // ATX heading example
    }
}
