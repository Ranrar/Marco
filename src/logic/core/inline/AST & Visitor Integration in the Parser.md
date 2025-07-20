# AST & Visitor Integration in the Parser

This document describes the step-by-step process for integrating the Abstract Syntax Tree (AST) and Visitor pattern into the Markdown parser. The goal is to ensure that raw Markdown input is converted into a complete AST, which can then be traversed and rendered using visitors (such as the HTML renderer).

---

## Why Integrate AST & Visitor in the Parser?

- **Modularity:** AST nodes represent all Markdown features in a structured, extensible way.
- **Extensibility:** The Visitor pattern allows new operations (rendering, analysis, event emission) without changing AST node definitions.
- **Robustness:** End-to-end parsing and rendering is possible for all Markdown features.

---

## Step-by-Step Integration Plan

1. **Audit the Parser Code**
   - Locate the main parser entry point (e.g., `parse_markdown`).
   - Identify what data structures are currently produced.

2. **Refactor Parser Output to AST**
   - Change parser functions to build and return AST nodes (`Block`, `Inline`, etc.) for every Markdown feature.
   - For each Markdown feature, ensure the parser emits the correct AST node.

3. **Ensure Visitor Coverage**
   - Confirm every AST node type has an `accept<V: AstVisitor>(&self, visitor: &mut V)` method.
   - Update the `AstVisitor` trait to add visit methods for each node type.

4. **Expand Feature Coverage**
   - For each Markdown feature (paragraphs, lists, code blocks, blockquotes, links, images, emphasis, tables, math, custom tags), write parser logic to emit the correct AST node.
   - Handle edge cases and advanced features.

5. **Integrate with Visitors**
   - Ensure the parser output (AST) can be passed to any visitor (renderer, event emitter, etc.).
   - Example: `let ast = parse_markdown(input); let html = HtmlRenderer::render(&ast);`

6. **End-to-End Tests**
   - Write tests that take Markdown input, parse it, traverse the AST with a visitor, and check the output.
   - Cover all major Markdown features.

7. **Debug & Iterate**
   - Fix any parsing or rendering bugs.
   - Add more tests for edge cases.

8. **Document Workflow**
   - Explain how the parser builds the AST and how visitors traverse it.

---

## Example Workflow

```rust
let markdown = "# Title\n\nThis is a paragraph.";
let ast = parse_markdown(markdown); // Step 2: parser returns AST
let html = HtmlRenderer::render(&ast); // Step 5: visitor renders AST
println!("{}", html); // Output: <h1>Title</h1>\n<p>This is a paragraph.</p>
```

---

## Benefits
- **Separation of concerns:** Parsing, AST, and rendering are decoupled.
- **Easy extensibility:** Add new Markdown features or output formats by extending AST and visitors.
- **Testability:** Each step can be tested independently and together.

---

## Next Steps
- Audit parser and AST coverage for all Markdown features.
- Implement missing `accept` methods and visitor trait updates.
- Write integration tests for Markdown → AST → Visitor → Output.
- Document and maintain the workflow for contributors.

---

*This document should be updated as integration progresses and new features are added.*
