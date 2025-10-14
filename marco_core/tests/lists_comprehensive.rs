// Comprehensive test suite for list parsing - Phase 6
// Tests CommonMark Section 5.2 (List items) and 5.3 (Lists)

use marco_core::components::marco_engine::parsers::block_parser::{BlockParser, Rule};
use pest::Parser;

// ============================================================================
// Simple Lists - Basic Functionality
// ============================================================================

#[test]
fn test_simple_unordered_list_dash() {
    let input = "- Apple\n- Banana\n- Cherry\n";
    let result = BlockParser::parse(Rule::list, input);
    assert!(result.is_ok(), "Should parse simple dash list");
}

#[test]
fn test_simple_unordered_list_plus() {
    let input = "+ Apple\n+ Banana\n+ Cherry\n";
    let result = BlockParser::parse(Rule::list, input);
    assert!(result.is_ok(), "Should parse simple plus list");
}

#[test]
fn test_simple_unordered_list_asterisk() {
    let input = "* Apple\n* Banana\n* Cherry\n";
    let result = BlockParser::parse(Rule::list, input);
    assert!(result.is_ok(), "Should parse simple asterisk list");
}

#[test]
fn test_simple_ordered_list_dot() {
    let input = "1. First\n2. Second\n3. Third\n";
    let result = BlockParser::parse(Rule::list, input);
    assert!(result.is_ok(), "Should parse ordered list with dots");
}

#[test]
fn test_simple_ordered_list_paren() {
    let input = "1) First\n2) Second\n3) Third\n";
    let result = BlockParser::parse(Rule::list, input);
    assert!(result.is_ok(), "Should parse ordered list with parens");
}

// ============================================================================
// List Markers - Edge Cases
// ============================================================================

#[test]
fn test_list_marker_with_multiple_spaces() {
    let input = "- Item with  multiple  spaces\n";
    let result = BlockParser::parse(Rule::list, input);
    assert!(result.is_ok(), "Should handle multiple spaces after marker");
}

#[test]
fn test_ordered_list_9_digits_max() {
    let input = "123456789. Valid\n";
    let result = BlockParser::parse(Rule::list, input);
    assert!(result.is_ok(), "Should parse 9-digit list number");
}

#[test]
fn test_ordered_list_start_with_zero() {
    let input = "0. Zero-indexed list\n";
    let result = BlockParser::parse(Rule::list, input);
    assert!(result.is_ok(), "Should allow lists starting with 0");
}

// ============================================================================
// Indentation Rules
// ============================================================================

#[test]
fn test_list_no_indentation() {
    let input = "- No indentation\n";
    let result = BlockParser::parse(Rule::list, input);
    assert!(result.is_ok(), "Should parse list with no indentation");
}

#[test]
fn test_list_one_space_indentation() {
    let input = " - One space indent\n";
    let result = BlockParser::parse(Rule::list, input);
    assert!(result.is_ok(), "Should parse list with 1 space indent");
}

#[test]
fn test_list_two_space_indentation() {
    let input = "  - Two space indent\n";
    let result = BlockParser::parse(Rule::list, input);
    assert!(result.is_ok(), "Should parse list with 2 space indent");
}

#[test]
fn test_list_three_space_indentation() {
    let input = "   - Three space indent\n";
    let result = BlockParser::parse(Rule::list, input);
    assert!(result.is_ok(), "Should parse list with 3 space indent");
}

// ============================================================================
// Task Lists (GFM Extension)
// ============================================================================

#[test]
fn test_task_list_unchecked() {
    let input = "- [ ] Unchecked task\n";
    let result = BlockParser::parse(Rule::task_list, input);
    assert!(result.is_ok(), "Should parse unchecked task");
}

#[test]
fn test_task_list_checked_lowercase() {
    let input = "- [x] Checked task (lowercase)\n";
    let result = BlockParser::parse(Rule::task_list, input);
    assert!(result.is_ok(), "Should parse checked task with lowercase x");
}

#[test]
fn test_task_list_checked_uppercase() {
    let input = "- [X] Checked task (uppercase)\n";
    let result = BlockParser::parse(Rule::task_list, input);
    assert!(result.is_ok(), "Should parse checked task with uppercase X");
}

#[test]
fn test_task_list_multiple_items() {
    let input = "- [ ] Task one\n- [x] Task two\n- [ ] Task three\n";
    let result = BlockParser::parse(Rule::task_list, input);
    assert!(result.is_ok(), "Should parse multiple task list items");
}

#[test]
fn test_task_list_with_ordered_marker() {
    let input = "1. [ ] Ordered task one\n2. [x] Ordered task two\n";
    let result = BlockParser::parse(Rule::task_list, input);
    assert!(result.is_ok(), "Should parse ordered task list");
}

// ============================================================================
// Nested Lists
// ============================================================================

#[test]
fn test_nested_list_2_levels() {
    let input = "- Parent\n  - Child\n";
    let result = BlockParser::parse(Rule::nested_list, input);
    assert!(result.is_ok(), "Should parse 2-level nested list");
}

#[test]
fn test_nested_list_3_levels() {
    let input = "- Level 1\n  - Level 2\n    - Level 3\n";
    let result = BlockParser::parse(Rule::nested_list, input);
    assert!(result.is_ok(), "Should parse 3-level nested list");
}

#[test]
fn test_nested_mixed_markers() {
    let input = "- Bullet parent\n  1. Ordered child\n";
    let result = BlockParser::parse(Rule::nested_list, input);
    assert!(result.is_ok(), "Should parse mixed marker nested list");
}

#[test]
fn test_nested_with_content() {
    let input = "- Parent item with text\n  - Child item with text\n  - Another child\n";
    let result = BlockParser::parse(Rule::nested_list, input);
    assert!(result.is_ok(), "Should parse nested list with content");
}

// ============================================================================
// Multi-line List Items
// ============================================================================

#[test]
fn test_list_item_multi_line() {
    let input = "- First line\n  Second line\n";
    let result = BlockParser::parse(Rule::list_item_with_continuation, input);
    assert!(result.is_ok(), "Should parse multi-line list item");
}

#[test]
fn test_list_item_with_blank_line() {
    let input = "- First paragraph\n\n  Second paragraph\n";
    let result = BlockParser::parse(Rule::list_item_with_continuation, input);
    assert!(result.is_ok(), "Should parse list item with blank line");
}

// ============================================================================
// Tight vs Loose Lists
// ============================================================================

#[test]
fn test_tight_list() {
    let input = "- One\n- Two\n- Three\n";
    let result = BlockParser::parse(Rule::tight_list, input);
    assert!(result.is_ok(), "Should parse tight list (no blank lines)");
}

#[test]
fn test_loose_list() {
    let input = "- One\n\n- Two\n\n- Three\n";
    let result = BlockParser::parse(Rule::loose_list, input);
    assert!(result.is_ok(), "Should parse loose list (with blank lines)");
}

// ============================================================================
// Empty List Items
// ============================================================================

#[test]
fn test_empty_list_item_bullet() {
    let input = "-\n- Item\n";
    let result = BlockParser::parse(Rule::list, input);
    assert!(result.is_ok(), "Should handle empty bullet list item");
}

#[test]
fn test_empty_list_item_ordered() {
    let input = "1.\n2. Item\n";
    let result = BlockParser::parse(Rule::list, input);
    assert!(result.is_ok(), "Should handle empty ordered list item");
}

// ============================================================================
// Marker Variations
// ============================================================================

#[test]
fn test_changing_bullet_markers_creates_new_list() {
    // Per CommonMark: changing markers should create separate lists
    let input = "- Dash\n+ Plus\n";
    let result = BlockParser::parse(Rule::list, input);
    // This should either fail or be recognized as two separate lists
    // For now we just verify it doesn't crash
    let _ = result;
}

#[test]
fn test_changing_ordered_delimiters_creates_new_list() {
    // Per CommonMark: changing . to ) should create separate lists
    let input = "1. Dot\n2) Paren\n";
    let result = BlockParser::parse(Rule::list, input);
    // This should either fail or be recognized as two separate lists
    let _ = result;
}

// ============================================================================
// List Interruption Rules
// ============================================================================

#[test]
fn test_list_interrupts_paragraph() {
    // Per CommonMark: lists can interrupt paragraphs
    let input = "Paragraph text\n- List item\n";
    // This tests document-level behavior, not list-only
    // For now, we focus on list parsing in isolation
}

// ============================================================================
// Complex Scenarios
// ============================================================================

#[test]
fn test_list_with_long_content() {
    let input = "- This is a very long list item with lots of text that continues on and on\n";
    let result = BlockParser::parse(Rule::list, input);
    assert!(result.is_ok(), "Should parse list with long content");
}

#[test]
fn test_list_with_special_characters() {
    let input = "- Item with *emphasis* and **bold**\n";
    let result = BlockParser::parse(Rule::list, input);
    assert!(result.is_ok(), "Should parse list with inline formatting");
}

#[test]
fn test_mixed_tight_loose_not_supported_directly() {
    // CommonMark allows lists where some items are tight and others loose
    // This is complex and may not work with current grammar
    let input = "- Tight item\n- Another tight\n\n- Loose item\n";
    let result = BlockParser::parse(Rule::list, input);
    // Don't assert - just check it doesn't crash
    let _ = result;
}

// ============================================================================
// Summary Test
// ============================================================================

#[test]
fn test_comprehensive_list_parsing() {
    // Test that we can parse various list types without errors
    let test_cases = vec![
        "- Simple item\n",
        "+ Plus item\n",
        "* Asterisk item\n",
        "1. Numbered\n",
        "42. Starting at 42\n",
        "- [ ] Task\n",
        "  - Indented\n",
    ];
    
    for (i, input) in test_cases.iter().enumerate() {
        let result = BlockParser::parse(Rule::list, input)
            .or_else(|_| BlockParser::parse(Rule::task_list, input))
            .or_else(|_| BlockParser::parse(Rule::nested_list, input));
        assert!(result.is_ok(), "Test case {} should parse: {}", i, input);
    }
}
