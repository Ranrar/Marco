use pest::Parser;

// Import from the main project
use marco::components::marco_engine::parser::{MarkdownParser, Rule};

fn main() {
    // Tests converted to assertions to avoid terminal output from prints.
    // Test different space patterns
    let test_cases = [
        "1. test\n",      // single space
        "1.  test\n",     // double space
        "1.\ttest\n",     // tab
        "1. \ttest\n",    // space + tab
    ];

    for test_case in test_cases {
        assert!(MarkdownParser::parse(Rule::ordered_list_item, test_case).is_ok());
    }

    // Test ordered_marker
    assert!(MarkdownParser::parse(Rule::ordered_marker, "1.").is_ok());

    // Test space
    assert!(MarkdownParser::parse(Rule::WHITESPACE, " ").is_ok());

    // Test list_item_content
    assert!(MarkdownParser::parse(Rule::list_item_content, "test").is_ok());

    // Test NEWLINE
    assert!(MarkdownParser::parse(Rule::NEWLINE, "\n").is_ok());
}
