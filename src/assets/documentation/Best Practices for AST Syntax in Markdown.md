# Best Practices for Building an AST + Syntax System for Markdown

## 🔑 Core Principles

### 1. Separate Concerns Clearly
- **Lexer (Tokenizer):** Break raw text into tokens (e.g., `HeadingStart`, `EmphasisOpen`, `Text`, `CodeFence`).  
- **Parser:** Convert tokens into an **AST** (e.g., `Heading { level, children }`, `Paragraph { children }`).  
- **Renderer(s):** Convert AST into output formats (HTML, plain text, PDF, etc.).  

👉 This makes the parser extensible: new syntax rules can be added without rewriting everything.

---

### 2. Design a Strong AST Structure
- Use **enums/variants** for node types (`Heading`, `Paragraph`, `List`, `Link`, `Image`, `CodeBlock`, etc.).  
- Store **inline vs block** distinction explicitly:

```rust
enum BlockNode {
    Heading { level: u8, children: Vec<InlineNode> },
    Paragraph(Vec<InlineNode>),
    List { ordered: bool, items: Vec<Vec<BlockNode>> },
    CodeBlock { lang: Option<String>, code: String },
}

enum InlineNode {
    Text(String),
    Emphasis(Vec<InlineNode>),
    Strong(Vec<InlineNode>),
    Link { href: String, children: Vec<InlineNode> },
    Image { src: String, alt: String },
    Code(String),
}
````

* Keep nodes **recursive** to naturally support nesting (`**bold *and italic***`).

---

### 3. Grammar / Syntax Rules as Data

* Don’t hardcode syntax rules—store them in **external configs** (e.g., `.ron`, `.json`, or `.yaml`).
* Supports multiple **dialects**: CommonMark, GFM, Obsidian, etc.

Example (`syntax.ron`):

````ron
[
  Rule::Heading { marker: "#", max_level: 6 },
  Rule::Emphasis { markers: ["*", "_"] },
  Rule::CodeFence { marker: "```" },
]
````

---

### 4. Normalization and Post-Processing

* Markdown has ambiguous cases (`***text***` could be `<em><strong>` or `<strong><em>`).
* Best practice: Parse raw → build raw AST → run a **normalization pass** to resolve ambiguities.

---

### 5. Preserve Source Information

* Store **source span metadata** (line/column ranges) in AST nodes.
* Useful for:

  * Editor integrations (highlighting, error reporting).
  * Round-tripping Markdown → AST → Markdown.

```rust
struct Span { start: usize, end: usize }

struct Node<T> {
    kind: T,
    span: Span,
}
```

---

### 6. Be Extensible

* Allow **custom nodes/plugins**.
* Examples: `[[wikilinks]]`, `:::admonition:::`, math (`$...$`).
* AST should support unknown/custom nodes without breaking:

```rust
enum InlineNode {
    Text(String),
    Custom { kind: String, data: String },
    // …
}
```

---

### 7. Performance Considerations

* Use **zero-copy slices** instead of cloning strings where possible.
* Consider **arena allocation** for AST nodes in large documents.
* Support **incremental parsing** for editor scenarios (re-parse only affected parts).

---

### 8. Testing Strategy

* Build a **conformance test suite**:

  * CommonMark spec tests.
  * Dialect-specific edge cases.
  * Nested/ambiguous emphasis (`*a **b c* d**`).
* Keep **golden files** for AST and rendered output.

---

### 9. Error Handling

* Markdown is forgiving—parser should **recover gracefully**.
* Store **error nodes** or **unknown nodes** instead of panicking.

---

### 10. Keep Renderers Separate

* One AST → many renderers:

  * HTML
  * Plain text
  * Rich text (Pango, PDF, GUI widgets)
  * Markdown (pretty printer)

---

## ✅ Summary

Following this approach ensures a **modular, extensible, and efficient Markdown system**:

* Lexer = surface syntax
* Parser = AST
* Syntax rules = data-driven
* Normalizer = consistency
* Renderers = multiple outputs