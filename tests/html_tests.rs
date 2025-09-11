use marco::components::marco_engine::render_html::{HtmlOptions, HtmlRenderer};
use marco::components::marco_engine::{AstBuilder, MarcoParser, Rule};
use pest::Parser;

fn parse_and_render(input: &str) -> String {
    let pairs = MarcoParser::parse(Rule::document, input).unwrap();
    let ast = AstBuilder::build(pairs).unwrap();

    let renderer = HtmlRenderer::new(HtmlOptions::default());
    renderer.render(&ast)
}

#[test]
fn test_text_rendering() {
    let input = "Hello, world!";
    let html = parse_and_render(input);
    assert!(html.contains("<p>Hello, world!</p>"));
}

#[test]
fn test_emphasis_rendering() {
    let input = "*italic* **bold** ***bold italic***";
    let html = parse_and_render(input);
    println!("HTML Output: {}", html);
    assert!(html.contains("<em>italic</em>"));
    assert!(html.contains("<strong>bold</strong>"));
    assert!(html.contains("<strong><em>bold italic</em></strong>"));
}

#[test]
fn test_heading_rendering() {
    let input = "# Heading 1\n## Heading 2\n### Heading 3";
    let html = parse_and_render(input);
    assert!(html.contains("<h1>Heading 1</h1>"));
    assert!(html.contains("<h2>Heading 2</h2>"));
    assert!(html.contains("<h3>Heading 3</h3>"));
}

#[test]
fn test_code_block_rendering() {
    let input = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
    let html = parse_and_render(input);
    assert!(html.contains("<pre><code class=\"rust\">"));
    assert!(html.contains("fn main()"));
    assert!(html.contains("println!(\"Hello\");"));
    assert!(html.contains("</code></pre>"));
}

#[test]
fn test_inline_code_rendering() {
    let input = "Here is `inline code` in text.";
    let html = parse_and_render(input);
    assert!(html.contains("<code>inline code</code>"));
}

#[test]
fn test_list_rendering() {
    let input = "- Item 1\n- Item 2\n  - Nested item\n- Item 3";
    let html = parse_and_render(input);
    assert!(html.contains("<ul>"));
    assert!(html.contains("<li>Item 1</li>"));
    assert!(html.contains("<li>Item 2"));
    assert!(html.contains("<li>Nested item</li>"));
    assert!(html.contains("</ul>"));
}

#[test]
fn test_ordered_list_rendering() {
    let input = "1. First\n2. Second\n3. Third";
    let html = parse_and_render(input);
    assert!(html.contains("<ol>"));
    assert!(html.contains("<li>First</li>"));
    assert!(html.contains("<li>Second</li>"));
    assert!(html.contains("<li>Third</li>"));
    assert!(html.contains("</ol>"));
}

#[test]
fn test_link_rendering() {
    let input = "[Link text](https://example.com)";
    let html = parse_and_render(input);
    assert!(html.contains("<a href=\"https://example.com\">Link text</a>"));
}

#[test]
fn test_table_rendering() {
    let input = "| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |";
    let html = parse_and_render(input);
    assert!(html.contains("<table class=\"marco-table\">"));
    assert!(html.contains("<thead>"));
    assert!(html.contains("<th>Header 1</th>"));
    assert!(html.contains("<th>Header 2</th>"));
    assert!(html.contains("<tbody>"));
    assert!(html.contains("<td>Cell 1</td>"));
    assert!(html.contains("<td>Cell 2</td>"));
    assert!(html.contains("</table>"));
}

#[test]
fn test_blockquote_rendering() {
    let input = "> This is a blockquote\n> with multiple lines";
    let html = parse_and_render(input);
    assert!(html.contains("<blockquote>"));
    assert!(html.contains("This is a blockquote"));
    assert!(html.contains("with multiple lines"));
    assert!(html.contains("</blockquote>"));
}

#[test]
fn test_user_mention_rendering() {
    let input = "Hello @username, how are you?";
    let html = parse_and_render(input);
    assert!(html.contains("<span class=\"marco-user-mention\">@username</span>"));
}

#[test]
fn test_bookmark_rendering() {
    let input = "#bookmark_name This is a bookmark";
    let html = parse_and_render(input);
    assert!(html.contains("id=\"bookmark_name\""));
    assert!(html.contains("class=\"marco-bookmark\""));
    assert!(html.contains("This is a bookmark"));
}

#[test]
fn test_admonition_rendering() {
    let input = "!!! note\n    This is a note admonition.\n    With multiple lines.";
    let html = parse_and_render(input);
    assert!(html.contains("<div class=\"marco-admonition marco-admonition-note\">"));
    assert!(html.contains("<div class=\"marco-admonition-title\">"));
    assert!(html.contains("note"));
    assert!(html.contains("<div class=\"marco-admonition-content\">"));
    assert!(html.contains("This is a note admonition"));
}

#[test]
fn test_tab_block_rendering() {
    let input = "=== \"Tab 1\"\n    Content of tab 1\n=== \"Tab 2\"\n    Content of tab 2";
    let html = parse_and_render(input);
    assert!(html.contains("<div class=\"marco-tab-block\">"));
    assert!(html.contains("<div class=\"marco-tab-title\">"));
    assert!(html.contains("Tab 1"));
    assert!(html.contains("Tab 2"));
    assert!(html.contains("<div class=\"marco-tabs\">"));
    assert!(html.contains("<div class=\"marco-tab\">"));
    assert!(html.contains("Content of tab 1"));
    assert!(html.contains("Content of tab 2"));
}

#[test]
fn test_math_block_rendering() {
    let input = "$$\nx = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}\n$$";
    let html = parse_and_render(input);
    assert!(html.contains("<div class=\"marco-math-block\">"));
    assert!(html.contains("x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}"));
}

#[test]
fn test_inline_math_rendering() {
    let input = "The formula is $E = mc^2$ in physics.";
    let html = parse_and_render(input);
    assert!(html.contains("<span class=\"marco-math-inline\">E = mc^2</span>"));
}

#[test]
fn test_executable_code_rendering() {
    let input = "```python exec\nprint(\"Hello, World!\")\nresult = 42\n```";
    let html = parse_and_render(input);
    assert!(html.contains("<div class=\"marco-executable-code marco-executable-code-python\">"));
    assert!(html.contains("print(\"Hello, World!\")"));
}

#[test]
fn test_diagram_rendering() {
    let input = "```mermaid\ngraph TD;\n    A-->B;\n    B-->C;\n```";
    let html = parse_and_render(input);
    assert!(html.contains("<div class=\"marco-executable-code marco-executable-code-mermaid\">"));
    assert!(html.contains("graph TD;"));
}

#[test]
fn test_html_escaping() {
    let input = "Here is some <script>alert('xss')</script> content";
    let html = parse_and_render(input);
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("&lt;/script&gt;"));
    assert!(!html.contains("<script>"));
}

#[test]
fn test_complex_document() {
    let input = r#"# Marco Test Document

This is a **complex document** with *multiple* ***features***.

## Code Examples

Here's some `inline code` and a block:

```rust
fn main() {
    println!("Hello, Marco!");
}
```

## Lists and Links

- Item with [link](https://example.com)
- Another item
  1. Nested ordered item
  2. Another nested item

> This is a blockquote with a @user mention.

## Advanced Features

!!! warning
    This is a warning admonition.

=== "Configuration"
    ```yaml
    option: value
    ```
=== "Usage"
    Run the command: `marco --help`

| Feature | Status |
|---------|--------|
| Parsing | ✓ |
| HTML | ✓ |

Math: $f(x) = x^2$ and block math:

$$
\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}
$$
"#;

    let html = parse_and_render(input);

    // Verify major components are present
    assert!(html.contains("<h1>Marco Test Document</h1>"));
    assert!(html.contains("<h2>Code Examples</h2>"));
    assert!(html.contains("<strong>complex document</strong>"));
    assert!(html.contains("<em>multiple</em>"));
    assert!(html.contains("<code>inline code</code>"));
    assert!(html.contains("<pre><code class=\"rust\">"));
    assert!(html.contains("<ul>"));
    assert!(html.contains("<ol>"));
    assert!(html.contains("<a href=\"https://example.com\">link</a>"));
    assert!(html.contains("<blockquote>"));
    assert!(html.contains("<span class=\"marco-user-mention\">@user</span>"));
    assert!(html.contains("<div class=\"marco-admonition marco-admonition-warning\">"));
    assert!(html.contains("<div class=\"marco-tab-block\">"));
    assert!(html.contains("<table class=\"marco-table\">"));
    assert!(html.contains("<span class=\"marco-math-inline\">"));
    assert!(html.contains("<div class=\"marco-math-block\">"));

    // Verify structure is reasonable
    assert!(html.len() > 1000); // Complex document should be substantial
}

#[test]
fn test_line_break_normal_mode() {
    let options = HtmlOptions {
        line_break_mode: "normal".to_string(),
        ..HtmlOptions::default()
    };

    // Test hard line break (2 spaces + newline) - should render as <br>
    let input = "Line one  \nLine two";
    let pairs = MarcoParser::parse(Rule::document, input).unwrap();
    let ast = AstBuilder::build(pairs).unwrap();
    let renderer = HtmlRenderer::new(options.clone());
    let html = renderer.render(&ast);
    assert!(html.contains("<br>"));

    // Test soft line break (just newline) - should render as space
    let input = "Line one\nLine two";
    let pairs = MarcoParser::parse(Rule::document, input).unwrap();
    let ast = AstBuilder::build(pairs).unwrap();
    let renderer = HtmlRenderer::new(options);
    let html = renderer.render(&ast);
    assert!(!html.contains("<br>"));
    assert!(html.contains("Line one Line two"));
}

#[test]
fn test_line_break_reversed_mode() {
    let options = HtmlOptions {
        line_break_mode: "reversed".to_string(),
        ..HtmlOptions::default()
    };

    // Test hard line break (2 spaces + newline) - should render as space in reversed mode
    let input = "Line one  \nLine two";
    let pairs = MarcoParser::parse(Rule::document, input).unwrap();
    let ast = AstBuilder::build(pairs).unwrap();
    let renderer = HtmlRenderer::new(options.clone());
    let html = renderer.render(&ast);
    assert!(!html.contains("<br>"));
    assert!(html.contains("Line one Line two"));

    // Test soft line break (just newline) - should render as <br> in reversed mode
    let input = "Line one\nLine two";
    let pairs = MarcoParser::parse(Rule::document, input).unwrap();
    let ast = AstBuilder::build(pairs).unwrap();
    let renderer = HtmlRenderer::new(options);
    let html = renderer.render(&ast);
    assert!(html.contains("<br>"));
}

#[test]
fn test_settings_integration() {
    // Test that settings loading works correctly
    use marco::logic::swanson::{
        EngineHtmlSettings, EngineRenderSettings, EngineSettings, Settings,
    };

    // Create temporary settings with reversed mode
    let temp_settings = Settings {
        engine: Some(EngineSettings {
            render: Some(EngineRenderSettings {
                html: Some(EngineHtmlSettings {
                    line_break_mode: Some("reversed".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    // Test loading line break mode from settings
    let line_break_mode = temp_settings
        .engine
        .and_then(|e| e.render)
        .and_then(|r| r.html)
        .and_then(|h| h.line_break_mode)
        .unwrap_or_else(|| "normal".to_string());

    assert_eq!(line_break_mode, "reversed");

    // Test creating HtmlOptions with the loaded setting
    let options = HtmlOptions {
        line_break_mode: line_break_mode.clone(),
        ..HtmlOptions::default()
    };
    assert_eq!(options.line_break_mode, "reversed");
}

#[test]
fn test_line_break_backslash_hard_break() {
    let options = HtmlOptions {
        line_break_mode: "normal".to_string(),
        ..HtmlOptions::default()
    };

    // Test backslash + newline as hard line break
    let input = "Line one\\\nLine two";
    let pairs = MarcoParser::parse(Rule::document, input).unwrap();
    let ast = AstBuilder::build(pairs).unwrap();
    let renderer = HtmlRenderer::new(options);
    let html = renderer.render(&ast);
    assert!(html.contains("<br>"));
}
