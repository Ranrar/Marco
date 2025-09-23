# Marco Markdown Showcase

This directory contains comprehensive test documents for the Marco markdown engine, covering all supported features, extensions, and edge cases.

## Test Documents Overview

### 📋 [01_basic_markdown.md](./01_basic_markdown.md)
**Standard CommonMark Features**
- Headers (ATX and Setext)
- Text formatting (bold, italic, strikethrough)
- Marco extensions (highlight, superscript, subscript)
- Lists (ordered, unordered, nested)
- Links and images (inline, reference, autolinks)
- Code blocks (fenced, indented, nested)
- Mathematical content (inline and block)
- Blockquotes and nested content
- Special characters and escaping
- Horizontal rules and line breaks

### 🚀 [02_marco_extensions.md](./02_marco_extensions.md)
**Marco-Specific Features**
- Admonitions (note, tip, warning, danger, info)
- Custom emoji admonitions
- Executable code blocks (`@run` syntax)
- Diagrams (Mermaid, Graphviz)
- Tables with advanced formatting
- Page management (`[page=A4]`)
- Document navigation (`[@doc]`, bookmarks)
- Table of contents (`[toc]` with options)
- User mentions (`@user[platform]`)
- Tab blocks for organized content
- Task lists with metadata
- HTML integration
- Comments (block and inline)
- Definition lists
- YouTube embeds
- Footnotes (inline and reference)

### ⚠️ [03_edge_cases.md](./03_edge_cases.md)
**Parsing Challenges and Edge Cases**
- Whitespace handling (leading, trailing, mixed)
- Deep nesting scenarios
- Boundary conditions (empty elements, single characters)
- Adjacent and overlapping formatting
- Unmatched markers and escaped characters
- Complex URLs and malformed links
- Code block variations and special content
- Table edge cases and malformed structures
- Math with special characters
- List interruption and mixed markers
- HTML block vs inline detection
- Unicode and special character handling
- File path variations and complex structures
- Footnote edge cases and label variations

### 🏢 [04_real_world_example.md](./04_real_world_example.md)
**Comprehensive Real-World Usage**
A complete software development guide demonstrating:
- Professional documentation structure
- Multiple Marco features working together
- Realistic content scenarios
- Team collaboration features
- Technical documentation patterns
- Complex nested structures
- Integration examples
- Performance considerations
- Security guidelines
- Troubleshooting workflows

## Testing Guidelines

### Purpose of Each Document

1. **Basic Markdown** - Ensures CommonMark compliance and standard feature compatibility
2. **Marco Extensions** - Validates custom syntax parsing and rendering
3. **Edge Cases** - Tests parser robustness and error handling
4. **Real World** - Demonstrates practical usage and feature integration

### How to Use These Tests

#### Manual Testing
1. Open each document in Marco editor
2. Verify rendering matches expected output
3. Test interactive features (links, executable code)
4. Check for parsing errors or unexpected behavior

#### Automated Testing
```bash
# Parse all test documents
cargo run --bin test_parser tests/markdown_showcase/*.md

# Check for specific parsing issues
cargo run --bin debug_ast_structure tests/markdown_showcase/03_edge_cases.md

# Test whitespace handling
cargo run --bin debug_whitespace tests/markdown_showcase/01_basic_markdown.md
```

#### Integration Testing
```rust
// Example test using these documents
#[test]
fn test_comprehensive_parsing() {
    let test_files = [
        "tests/markdown_showcase/01_basic_markdown.md",
        "tests/markdown_showcase/02_marco_extensions.md",
        "tests/markdown_showcase/03_edge_cases.md",
        "tests/markdown_showcase/04_real_world_example.md",
    ];
    
    for file_path in test_files {
        let content = std::fs::read_to_string(file_path).unwrap();
        let result = parse_and_render(&content);
        assert!(result.is_ok(), "Failed to parse {}", file_path);
    }
}
```

### Expected Behaviors

#### ✅ Should Work
- All standard CommonMark features
- Marco custom syntax (admonitions, @run blocks, etc.)
- Nested structures with proper precedence
- Unicode and international text
- Complex URLs and file paths
- Mathematical expressions

#### ⚠️ May Have Issues
- Very deep nesting (>8 levels)
- Extremely long lines (>10,000 characters)
- Malformed HTML mixed with Marco syntax
- Concurrent formatting markers
- Platform-specific file paths

#### ❌ Known Limitations
- HTML5 semantic elements (treated as generic HTML)
- Custom HTML attributes in markdown context
- Binary file inclusion
- Dynamic content generation

## Performance Benchmarks

These documents can be used for performance testing:

| Document | Size | Expected Parse Time | Features Tested |
|----------|------|-------------------|-----------------|
| 01_basic_markdown.md | ~15KB | <10ms | Standard parsing |
| 02_marco_extensions.md | ~20KB | <15ms | Custom syntax |
| 03_edge_cases.md | ~25KB | <20ms | Error handling |
| 04_real_world_example.md | ~30KB | <25ms | Complex integration |

## Contributing

When adding new test cases:

1. **Choose the right document:**
   - Standard features → `01_basic_markdown.md`
   - Marco extensions → `02_marco_extensions.md`
   - Edge cases → `03_edge_cases.md`
   - Real usage → `04_real_world_example.md`

2. **Follow the structure:**
   - Use clear section headers
   - Include examples and expected behavior
   - Add comments explaining test purpose
   - Group related test cases

3. **Test coverage:**
   - Add positive test cases (should work)
   - Add negative test cases (should fail gracefully)
   - Include boundary conditions
   - Consider cross-feature interactions

## Maintenance

These test documents should be updated when:
- New Marco features are added
- Parsing behavior changes
- Edge cases are discovered
- Performance regressions occur
- CommonMark specification updates

Last updated: 2024-01-15
Maintainer: Marco Development Team