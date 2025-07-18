# Attribute Propagation and Extension Points in the Markdown Event Stream Parser

## Overview

This document describes how custom attributes (e.g., `{.class #id}` blocks) are parsed, propagated, and exposed for plugin authors in the Markdown event stream parser. It covers the architecture, extension points, and best practices for building robust, extensible plugins that interact with attributes at both the AST and event stream levels.

### Attribute Propagation

- **Parsing**: Custom attributes are parsed from Markdown using a dedicated parser (see `attributes_parser.rs`). Syntax such as `{.class #id}` is recognized and converted into an `Attributes` struct.
- **AST Integration**: Attributes are attached to relevant AST nodes (blocks, inlines) during parsing. Each node type supports an optional `Attributes` field.
- **Event Stream**: When the AST is traversed to produce the event stream, attributes are propagated to corresponding `Event` and `Tag` variants. This ensures that all rendering and transformation logic can access attributes.
- **Source Position**: Events also carry source position information for advanced diagnostics and error reporting.

### Example

```rust
// Example: Parsing Markdown with custom attributes
let markdown = "*emph*{.important} and **strong**{#strong-id}";
let events = parse_markdown_to_events(markdown);
for event in events {
    match event {
        Event::Start(Tag::Emphasis(attr), _, _) => println!("Emphasis with attributes: {:?}", attr),
        Event::Start(Tag::Strong(attr), _, _) => println!("Strong with attributes: {:?}", attr),
        _ => {}
    }
}
```

## Extension Points for Plugin Authors

### 1. Parsing Extensions

**How to Extend Parsing:**
- Implement custom attribute syntaxes by extending the attribute parser (see `attributes_parser.rs`).
- Add new block or inline types in the lexer/parser to support plugin-specific Markdown constructs.
- Use the AST node's `Attributes` field to store plugin-specific metadata.

**Example:**
```rust
// Extend attribute parsing for custom syntax
fn parse_custom_attributes(input: &str) -> Attributes {
    // Parse input and return Attributes with plugin-specific fields
}
```

### 2. Event Transformation

**How to Transform Events:**
- Implement an event transformer function or struct that inspects and modifies events.
- Use Rust's iterator adapters (`map`, `filter`, etc.) to process the event stream.
- Inject, modify, or consume custom attributes as needed for your plugin.

**Example:**
```rust
fn transform_events<I: Iterator<Item = Event>>(events: I) -> impl Iterator<Item = Event> {
    events.map(|event| {
        match event {
            Event::Start(tag, pos, attr) => {
                // Modify attributes for plugin logic
                Event::Start(tag, pos, attr)
            }
            _ => event,
        }
    })
}
```

### 3. Rendering Extensions

**How to Extend Rendering:**
- Implement custom renderers for blocks/inlines with specific attributes.
- Access the `Attributes` field during rendering to customize HTML, styling, or behavior.
- Use the event stream to drive rendering logic, injecting plugin-specific output as needed.

**Example:**
```rust
fn render_html(events: &[Event]) -> String {
    let mut html = String::new();
    for event in events {
        match event {
            Event::Start(tag, _, attr) => {
                if let Some(class) = attr.classes.first() {
                    html.push_str(&format!("<div class='{}'>", class));
                }
            }
            Event::End(_, _, _) => html.push_str("</div>"),
            _ => {}
        }
    }
    html
}
```

### 4. Diagnostics and Error Reporting

**How to Use Source Position Info:**
- Use the `SourcePos` field in events to provide detailed error messages or warnings for malformed attributes.
- Integrate diagnostics into your plugin to help users debug Markdown issues.

**Example:**
```rust
fn check_attributes(events: &[Event]) {
    for event in events {
        if let Event::Start(_, pos, attr) = event {
            if attr.classes.is_empty() {
                eprintln!("Warning: Missing class at {:?}", pos);
            }
        }
    }
}
```

## Code Examples

### Accessing Attributes in Plugins
```rust
// Example: Accessing attributes in a plugin renderer
fn render_event(event: &Event) {
    match event {
        Event::Start(tag, _, attr) => {
            // Access class, id, and data-* attributes
            if !attr.classes.is_empty() {
                println!("Classes: {:?}", attr.classes);
            }
            if let Some(id) = &attr.id {
                println!("ID: {}", id);
            }
            for (key, value) in &attr.data {
                println!("Data attribute: {} = {}", key, value);
            }
        }
        _ => {}
    }
}
```

### Adding New Attributes
```rust
// Extend the Attributes struct for plugin-specific fields
#[derive(Debug, Clone)]
pub struct Attributes {
    pub classes: Vec<String>,
    pub id: Option<String>,
    pub data: HashMap<String, String>,
    pub custom: HashMap<String, String>, // For plugin-specific attributes
}

// Example: Adding a custom attribute in a plugin
fn add_custom_attribute(attr: &mut Attributes, key: &str, value: &str) {
    attr.custom.insert(key.to_string(), value.to_string());
}

// Example: Using custom attributes during rendering
fn render_with_custom_attributes(event: &Event) {
    match event {
        Event::Start(_, _, attr) => {
            if let Some(custom_val) = attr.custom.get("my_plugin") {
                println!("Custom plugin attribute: {}", custom_val);
            }
        }
        _ => {}
    }
}
```

## Best Practices

### Best Practices

- **Strong Typing:** Use well-defined structs/enums for attributes. Avoid raw strings or untyped maps to prevent errors and improve code clarity.
- **Consistent Propagation:** Ensure attributes are propagated through all relevant AST and event types, including block-level, inline, and nested elements.
- **Clear Extension Points:** Document where and how plugins can hook into parsing, event transformation, and rendering. Provide clear APIs and examples.
- **Comprehensive Testing:** Test attribute handling with edge cases, such as deeply nested attributes, unusual syntaxes, and malformed input.
- **Thread Safety:** Avoid global mutable state. Use message passing, event transformers, or thread-safe containers (`Arc<Mutex<T>>`) for shared state.
- **Graceful Error Handling:** Use `Option` and `Result` types for attribute access. Avoid panics from `.unwrap()` or `.expect()`.
- **Documentation:** Clearly document custom attribute syntaxes and plugin APIs for users and contributors.

### Common Pitfalls

- **Incomplete Propagation:** Missing propagation of attributes to certain event types (e.g., only block-level, not inline).
- **Fragile Error Handling:** Overusing `.unwrap()` or `.expect()` can cause panics if attributes are missing or malformed.
- **Opaque APIs:** Not documenting extension points or attribute formats makes plugin development difficult.
- **Unintended Global State:** Using global mutable state can break thread safety and testability.
- **Ignoring Edge Cases:** Failing to test with complex or malformed Markdown can lead to bugs in attribute handling.

## References

### Relevant APIs
- [`markdown-rs` crate & docs](https://github.com/wooorm/markdown-rs) | [API docs](https://docs.rs/markdown/latest/markdown/)
- [`comrak` crate & docs](https://github.com/kivikakk/comrak) | [API docs](https://docs.rs/comrak/latest/comrak/)
- [`pulldown-cmark` crate & docs](https://github.com/raphlinus/pulldown-cmark) | [API docs](https://docs.rs/pulldown-cmark/latest/pulldown_cmark/)

### Community Resources
- [Rust Users Forum: Extensible Markdown Parser](https://users.rust-lang.org/t/extensible-markdown-parser/66259)
- [CommonMark Spec](https://spec.commonmark.org/)
- [mdast Syntax Tree](https://github.com/syntax-tree/mdast)

## Further Reading
- [CommonMark Spec](https://spec.commonmark.org/)
- [mdast Syntax Tree](https://github.com/syntax-tree/mdast)

---

For questions or plugin author support, see the Discussions section in the project repository.
