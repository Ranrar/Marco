//! CSS debugging utilities for Marco
//!
//! This module provides tools to debug GTK CSS generation,
//! helping identify problematic selectors and CSS parser errors.

use colored::*;

/// Analyze generated CSS and find potential issues
pub fn dump_css_analysis() -> String {
    let css = marco::ui::css::generate_marco_css();
    
    let lines: Vec<&str> = css.lines().collect();
    
    let mut output = String::new();
    output.push_str(&format!("\n{}\n", "=== CSS Generation Report ===".blue().bold()));
    output.push_str(&format!("Total lines: {}\n", lines.len()));
    output.push_str(&format!("Total bytes: {}\n", css.len()));
    
    // Look for :empty pseudo-class (not supported by GTK)
    let empty_count = css.matches(":empty").count();
    output.push_str(&format!("\n{}\n", "=== Pseudo-class Usage ===".yellow().bold()));
    output.push_str(&format!("Uses of :empty: {}\n", empty_count));
    
    if empty_count > 0 {
        output.push_str(&format!("\n{}\n", "⚠️  Warning: GTK CSS doesn't support :empty pseudo-class".red().bold()));
        output.push_str(&format!("{}\n", "=== Lines with :empty ===".yellow().bold()));
        for (i, line) in lines.iter().enumerate() {
            if line.contains(":empty") {
                output.push_str(&format!("Line {}: {}\n", i + 1, line));
            }
        }
    }
    
    // Look for other potential issues
    output.push_str(&format!("\n{}\n", "=== Potential Issues ===".yellow().bold()));
    
    let mut issues_found = false;
    
    // Check for unsupported selectors
    let unsupported = vec![
        (":first-child", "Consider using explicit class names"),
        (":last-child", "Consider using explicit class names"),
        (":nth-child", "Not supported in GTK CSS"),
        ("::before", "Pseudo-elements have limited support"),
        ("::after", "Pseudo-elements have limited support"),
    ];
    
    for (selector, message) in unsupported {
        if css.contains(selector) {
            issues_found = true;
            output.push_str(&format!("{}: {} - {}\n", "⚠️".yellow(), selector, message));
        }
    }
    
    if !issues_found {
        output.push_str(&format!("{}\n", "✓ No obvious issues detected".green()));
    }
    
    output
}

/// Dump full CSS for inspection
pub fn dump_full_css() -> String {
    marco::ui::css::generate_marco_css()
}

/// Analyze specific line range in generated CSS
pub fn analyze_css_range(start_line: usize, end_line: usize) -> String {
    let css = marco::ui::css::generate_marco_css();
    let lines: Vec<&str> = css.lines().collect();
    
    let mut output = String::new();
    output.push_str(&format!("\n{}\n", format!("=== CSS Lines {} to {} ===", start_line, end_line).blue().bold()));
    
    let start_idx = start_line.saturating_sub(1);
    let end_idx = end_line.min(lines.len());
    
    for i in start_idx..end_idx {
        output.push_str(&format!("Line {}: {}\n", i + 1, lines[i]));
    }
    
    output
}

/// Find all CSS selectors in generated CSS
pub fn list_css_selectors() -> Vec<String> {
    let css = marco::ui::css::generate_marco_css();
    let mut selectors = Vec::new();
    
    for line in css.lines() {
        let line = line.trim();
        if line.ends_with('{') {
            let selector = line.trim_end_matches('{').trim();
            if !selector.is_empty() {
                selectors.push(selector.to_string());
            }
        }
    }
    
    selectors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_css_analysis() {
        let analysis = dump_css_analysis();
        assert!(!analysis.is_empty());
        assert!(analysis.contains("CSS Generation Report"));
    }

    #[test]
    fn smoke_test_css_dump() {
        let css = dump_full_css();
        assert!(!css.is_empty());
        assert!(css.contains("{")); // Should contain CSS syntax
    }

    #[test]
    fn smoke_test_selector_list() {
        let selectors = list_css_selectors();
        assert!(!selectors.is_empty());
        // Common selectors should be present
        assert!(selectors.iter().any(|s| s.contains("titlebar") || s.contains("menu")));
    }
}
