use std::process::Command;

// Grammar and parser tests
mod parser_tests {
    use marco::components::marco_engine::grammar::{MarcoParser, Rule};
    use marco::components::marco_engine::parse_to_html_cached;
    use pest::Parser;

    #[test]
    fn test_setext_h1_grammar() {
        let input = "Alternative Setext H1\n=====================";
        
        let result = MarcoParser::parse(Rule::setext_h1, input);
        assert!(result.is_ok(), "Should parse setext H1 successfully");
        
        let pairs = result.unwrap();
        let pair = pairs.into_iter().next().unwrap();
        
        // Check structure
        assert_eq!(pair.as_rule(), Rule::setext_h1);
        
        // Look for setext_content child
        let mut has_content = false;
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::setext_content {
                has_content = true;
                assert_eq!(inner.as_str(), "Alternative Setext H1");
            }
        }
        assert!(has_content, "Should have setext_content child rule");
    }

    #[test]
    fn test_setext_h2_grammar() {
        let input = "Alternative Setext H2\n---------------------";
        
        let result = MarcoParser::parse(Rule::setext_h2, input);
        assert!(result.is_ok(), "Should parse setext H2 successfully");
        
        let pairs = result.unwrap();
        let pair = pairs.into_iter().next().unwrap();
        
        // Check structure
        assert_eq!(pair.as_rule(), Rule::setext_h2);
        
        // Look for setext_content child
        let mut has_content = false;
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::setext_content {
                has_content = true;
                assert_eq!(inner.as_str(), "Alternative Setext H2");
            }
        }
        assert!(has_content, "Should have setext_content child rule");
    }

    #[test]
    fn test_setext_content_extraction() {
        let input = "Simple Header\n=============";
        
        let result = MarcoParser::parse(Rule::setext_h1, input);
        assert!(result.is_ok(), "Should parse setext H1");
        
        let pairs = result.unwrap();
        let pair = pairs.into_iter().next().unwrap();
        
        // Debug: print the structure
        println!("Setext H1 structure:");
        print_parser_structure(pair.clone(), 0);
        
        // Extract content
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::setext_content {
                assert_eq!(inner.as_str().trim(), "Simple Header");
                return;
            }
        }
        panic!("No setext_content found in parsed structure");
    }

    #[test]
    fn test_document_with_setext_headers() {
        let input = "First Header\n============\n\nSecond Header\n-------------\n\nRegular text.";
        
        let result = MarcoParser::parse(Rule::document, input);
        assert!(result.is_ok(), "Should parse document with setext headers");
        
        let pairs = result.unwrap();
        
        // Debug: print the full document structure
        for pair in pairs {
            println!("Document structure:");
            print_parser_structure(pair, 0);
        }
    }

    #[test]
    fn test_setext_vs_atx_headers() {
        // Test that setext and ATX headers both work
        let setext_input = "Setext Header\n=============";
        let atx_input = "# ATX Header";
        
        // Parse both
        let setext_result = MarcoParser::parse(Rule::setext_h1, setext_input);
        let atx_result = MarcoParser::parse(Rule::H1, atx_input);
        
        assert!(setext_result.is_ok(), "Should parse setext header");
        assert!(atx_result.is_ok(), "Should parse ATX header");
    }

    #[test] 
    fn test_marco_engine_setext_rendering() {
        // Test the actual HTML rendering through Marco engine
        let setext_h1 = "Test Header H1\n==============";
        let setext_h2 = "Test Header H2\n--------------";
        
        let h1_result = parse_to_html_cached(setext_h1);
        let h2_result = parse_to_html_cached(setext_h2);
        
        assert!(h1_result.is_ok(), "Should render setext H1");
        assert!(h2_result.is_ok(), "Should render setext H2");
        
        let h1_html = h1_result.unwrap();
        let h2_html = h2_result.unwrap();
        
        // Check that underlines are not in the HTML output
        assert!(!h1_html.contains("=============="), "H1 HTML should not contain underline markers");
        assert!(!h2_html.contains("--------------"), "H2 HTML should not contain underline markers");
        
        // Check proper header tags and content
        assert!(h1_html.contains("<h1>Test Header H1</h1>"), "Should contain clean H1");
        assert!(h2_html.contains("<h2>Test Header H2</h2>"), "Should contain clean H2");
    }

    // Helper function to debug parser structure
    fn print_parser_structure(pair: pest::iterators::Pair<Rule>, indent: usize) {
        let indent_str = "  ".repeat(indent);
        println!("{}Rule: {:?}, Text: {:?}", indent_str, pair.as_rule(), pair.as_str());
        
        for inner_pair in pair.into_inner() {
            print_parser_structure(inner_pair, indent + 1);
        }
    }
}

#[test]
fn test_marco_test_binary_basic_functionality() {
    let output = Command::new("./target/debug/marco-test")
        .args(["string", "# Hello World", "--expected", "<h1>Hello World</h1>"])
        .output()
        .expect("Failed to execute marco-test");
    
    assert!(output.status.success(), "marco-test should pass for correct input");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✓ Test passed!"), "Should show success message");
}

#[test]
fn test_marco_test_binary_failure_case() {
    let output = Command::new("./target/debug/marco-test")
        .args(["string", "# Hello World", "--expected", "<h2>Hello World</h2>"])
        .output()
        .expect("Failed to execute marco-test");
    
    assert!(!output.status.success(), "marco-test should fail for incorrect input");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✗ Test failed!"), "Should show failure message");
    assert!(stdout.contains("Similarity"), "Should show similarity percentage");
}

#[test]
fn test_marco_engine_smoke_test() {
    // Basic smoke test for Marco engine through test runner
    use marco::components::marco_engine::parse_to_html_cached;
    
    let result = parse_to_html_cached("# Test Header");
    assert!(result.is_ok(), "Marco engine should parse basic markdown");
    assert!(result.unwrap().contains("h1"), "Should produce header HTML");
}