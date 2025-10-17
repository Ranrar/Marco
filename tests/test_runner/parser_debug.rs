//! Parser debugging utilities for Marco
//!
//! This module provides tools to debug the Marco parser's grammar,
//! AST building, and full pipeline rendering.

use anyhow::Result;
use colored::*;
use marco_core::{parse_markdown, parse_and_render};

/// Debug a specific grammar rule with input
pub fn debug_grammar_rule(input: &str, rule_name: &str) -> Result<String> {
    let mut output = String::new();
    
    output.push_str(&format!("\n{}\n", format!("=== Grammar Rule Debug: {} ===", rule_name).blue().bold()));
    output.push_str(&format!("📝 Input: {:?}\n\n", input));
    output.push_str(&format!("ℹ️  Note: Using high-level API - parsing full document\n\n"));
    
    // Parse the document
    match parse_markdown(input) {
        Ok(ast) => {
            output.push_str(&format!("{}\n\n", "✓ Parsing succeeded!".green().bold()));
            output.push_str(&format!("{}\n", "AST Structure:".cyan().bold()));
            output.push_str(&format!("{:#?}\n", ast));
        }
        Err(e) => {
            output.push_str(&format!("{} {}\n", "✗ Parsing failed:".red().bold(), e));
        }
    }
    
    Ok(output)
}

/// Debug AST building from parsed markdown
pub fn debug_ast_building(input: &str) -> Result<String> {
    let mut output = String::new();
    
    output.push_str(&format!("\n{}\n", "=== AST Building Debug ===".blue().bold()));
    output.push_str(&format!("📝 Input: {:?}\n\n", input));
    
    // Parse with grammar and build AST
    match parse_markdown(input) {
        Ok(ast) => {
            output.push_str(&format!("{}\n\n", "✓ AST building succeeded".green().bold()));
            output.push_str(&format!("{}\n", "AST structure:".cyan().bold()));
            output.push_str(&format!("{:#?}\n", ast));
        }
        Err(e) => {
            output.push_str(&format!("{} {}\n", "✗ Parsing failed:".red().bold(), e));
        }
    }
    
    Ok(output)
}

/// Debug the full pipeline: parse → AST → HTML
pub fn debug_full_pipeline(input: &str) -> Result<String> {
    let mut output = String::new();
    
    output.push_str(&format!("\n{}\n", "=== Full Pipeline Debug ===".blue().bold()));
    output.push_str(&format!("📝 Input: {:?}\n\n", input));
    
    // Step 1 & 2: Parse and build AST
    match parse_markdown(input) {
        Ok(ast) => {
            output.push_str(&format!("{}\n", "✓ Steps 1-2: Parsing and AST building succeeded".green().bold()));
            output.push_str(&format!("   AST: {:#?}\n\n", ast));
            
            // Step 3: HTML rendering
            match parse_and_render(input, Default::default()) {
                Ok(html) => {
                    output.push_str(&format!("{}\n", "✓ Step 3: HTML rendering succeeded".green().bold()));
                    output.push_str(&format!("   HTML: {}\n\n", html));
                    
                    // Analyze common issues
                    if input.contains("=====") || input.contains("-----") {
                        if html.contains("=====") || html.contains("-----") {
                            output.push_str(&format!("{}\n", "⚠️  Warning: HTML contains underline characters".yellow().bold()));
                            output.push_str("   This suggests setext headers are not being parsed correctly.\n");
                        } else {
                            output.push_str(&format!("{}\n", "✅ Setext headers parsed correctly".green()));
                        }
                    }
                }
                Err(e) => {
                    output.push_str(&format!("{} {}\n", "✗ Step 3: HTML rendering failed:".red().bold(), e));
                }
            }
        }
        Err(e) => {
            output.push_str(&format!("{} {}\n", "✗ Steps 1-2: Parsing failed:".red().bold(), e));
        }
    }
    
    Ok(output)
}

/// Debug setext header parsing with test cases
pub fn debug_setext_headers(input: Option<&str>) -> Result<String> {
    let mut output = String::new();
    
    let test_cases = if let Some(user_input) = input {
        vec![("User Input", user_input.to_string())]
    } else {
        vec![
            ("Simple H1", "Simple Header\n=============".to_string()),
            ("Simple H2", "Simple Header\n-------------".to_string()),
            ("Complex H1", "Header with **bold** text\n========================".to_string()),
            ("Multiline", "Line 1\nLine 2\n======".to_string()),
        ]
    };
    
    output.push_str(&format!("\n{}\n", "=== Setext Header Debug ===".blue().bold()));
    
    for (name, test_input) in test_cases {
        output.push_str(&format!("\n{}\n", format!("--- Testing: {} ---", name).cyan()));
        output.push_str(&format!("📝 Input: {:?}\n", test_input));
        
        // Test full pipeline
        match parse_and_render(&test_input, Default::default()) {
            Ok(html) => {
                output.push_str(&format!("{}\n", "✓ Parsing succeeded".green()));
                output.push_str(&format!("HTML: {}\n", html));
                
                if html.contains("=====") || html.contains("-----") {
                    output.push_str(&format!("{}\n", "❌ PROBLEM: HTML contains underlines".red().bold()));
                } else if html.contains("<h1>") || html.contains("<h2>") {
                    output.push_str(&format!("{}\n", "✅ Setext header rendered correctly".green().bold()));
                } else {
                    output.push_str(&format!("{}\n", "⚠️  Warning: No header tags found".yellow()));
                }
            }
            Err(e) => {
                output.push_str(&format!("{} {}\n", "✗ Parsing failed:".red().bold(), e));
            }
        }
    }
    
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_grammar_debug() {
        let result = debug_grammar_rule("# Hello World", "heading");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Grammar Rule Debug"));
    }

    #[test]
    fn smoke_test_ast_debug() {
        let result = debug_ast_building("# Hello World\n\nParagraph text.");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("AST Building Debug"));
        assert!(output.contains("succeeded"));
    }

    #[test]
    fn smoke_test_pipeline_debug() {
        let result = debug_full_pipeline("**Bold** text");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Full Pipeline Debug"));
    }

    #[test]
    fn smoke_test_setext_debug() {
        let result = debug_setext_headers(None);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Setext Header Debug"));
        assert!(output.contains("Simple H1"));
    }
}
