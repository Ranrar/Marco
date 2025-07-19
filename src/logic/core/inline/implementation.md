Here is a **detailed, English explanation** of the proposed file and module structure for the inline Markdown parser. This is designed for modularity, clarity, and extensibility—suitable for serious parsing needs such as CommonMark, GFM, or custom dialects.



# lexer.rs refractor plan

---

## 📁 Directory Structure: `core/inline/`

```
core/
└── inline/
    ├── mod.rs             # Public API and module imports
    ├── tokenizer.rs       # Converts raw text to inline tokens
    ├── parser.rs          # Core inline parser: token stream → raw AST
    ├── delimiters.rs      # Delimiter stack logic (emphasis, links)
    ├── postprocess.rs     # Resolves delimiter runs into AST structure
    ├── rules.rs           # Rule handlers for inline syntaxes
    └── types.rs           # Token definitions, inline node enums, positions
```


---

## 📄 `mod.rs` – Entry point for the inline module

This file acts as the **public interface** for everything inside the inline parser. It imports all submodules and re-exports key types and functions, such as the `parse_inline()` function and the `InlineNode` type.

```rust
pub mod tokenizer;
pub mod parser;
pub mod delimiters;
pub mod postprocess;
pub mod rules;
pub mod types;

pub use parser::parse_inline;
pub use types::{InlineNode, SourcePos, Token};
```

---

## 📄 `tokenizer.rs` – Raw Markdown to tokens

### Purpose:

* Breaks input text (`&str`) into a **linear token stream** (`Vec<Token>`) with source position metadata.
* It identifies punctuation, delimiters, backticks, brackets, and escapes.

### Responsibilities:

* Tokenizes Markdown characters like `*`, `_`, `\`, `[`, `]`, `` ` ``, `(`, `)` and differentiates them from normal text.
* Collapses text sequences into single `Token::Text(String)` where possible.
* Keeps source location (`line`, `col`, `offset`) for diagnostics and mappings.
* Handles escaping (`\*`, `\_`) and entities (`&amp;`, etc).

---

## 📄 `parser.rs` – Token stream to raw inline AST

### Purpose:

* Parses a token stream into **raw, unprocessed inline nodes** (`InlineNode`).
* Leaves unresolved things like emphasis and links as placeholder nodes or temporary markers.
* Constructs a **flat or shallow node tree**.

### Responsibilities:

* Detect token sequences that could form Markdown constructs:

  * `*text*` → emphasis candidate
  * `[text](url)` → link candidate
* Delegates delimiter logic to `delimiters.rs`
* Produces a `Vec<InlineNode>` as output (may be postprocessed)

---

## 📄 `delimiters.rs` – Delimiter stack handling

### Purpose:

Implements the delimiter stack algorithm for **emphasis**, **strong**, and **links/images** according to the [CommonMark spec](https://spec.commonmark.org/0.30/#emphasis-and-strong-emphasis).

### Responsibilities:

* Maintains a stack of delimiters (`*`, `_`, `[`) with metadata:

  * Can open/close?
  * Length of run (for multiple characters like `***`)
  * Is it part of a link or image?
* Performs **pairing of opening and closing delimiters**.
* Supports nested/emphasized regions like:

  ```markdown
  ***bold and italic***
  [![alt](img.png)](url)
  ```

---

## 📄 `postprocess.rs` – Final AST normalization

### Purpose:

* Takes raw inline node sequences + delimiter stack and **constructs the final AST structure**:

  * Collapses adjacent text nodes
  * Converts delimiters into `Inline::Emphasis`, `Inline::Strong`, etc.
  * Constructs trees from flat sequences

### Responsibilities:

* Applies transformations such as:

  * `*hello*` → `Inline::Emphasis([Text("hello")])`
  * `**bold**` → `Inline::Strong([Text("bold")])`
* Cleans up the structure: removes unmatched delimiters, merges text nodes, etc.

---

## 📄 `rules.rs` – Rule handlers for inline syntax

### Purpose:

* Provides **modular rule implementations** for individual Markdown inline syntaxes.
* Can be invoked from the parser in a rule-dispatch pattern:

  * Each rule handles a syntax like emphasis, code, HTML, math, etc.

### Responsibilities:

* `parse_emphasis`: Handles parsing of `*` or `_`
* `parse_code_span`: Handles backtick-wrapped code `` `code` ``
* `parse_link`: Handles `[text](url)` and `![alt](img)`
* `parse_math`: Handles `$inline_math$` or `$$block_math$$`
* Easily extensible by registering or composing new rules

---

## 📄 `types.rs` – Token and AST definitions

### Purpose:

* Central location for all token types, node enums, and position tracking structs.

### Responsibilities:

* Defines `Token` enum (e.g., `Text`, `Star`, `OpenBracket`, `Html`, `CodeSpan`, etc.)
* Defines `InlineNode` enum (e.g., `Text`, `Emphasis`, `Strong`, `Link`, `Image`, `Math`, `Html`, etc.)
* Defines `SourcePos` struct for positional tracking

```rust
pub enum InlineNode {
    Text(String),
    Emphasis(Vec<InlineNode>),
    Strong(Vec<InlineNode>),
    Code(String),
    Link { href: String, title: String, children: Vec<InlineNode> },
    Image { src: String, alt: String, title: String },
    Math(String),
    Html(String),
    SoftBreak,
    LineBreak,
    // ...
}
```

---

## 🧠 Why this design?

| Advantage                      | Benefit                                                     |
| ------------------------------ | ----------------------------------------------------------- |
| Modular tokenizer/parser       | Can be reused/tested/debugged independently                 |
| Stack-based delimiter engine   | Properly handles nesting and ambiguities per spec           |
| Rule separation                | Clean, maintainable, and easy to extend (e.g., add MathJax) |
| Clear AST model (`InlineNode`) | Works for HTML, GTK, TUI, etc.                              |
| Postprocessing phase           | Clean separation of logic and transformation steps          |
| Source position tracking       | Enables precise diagnostics and editing feedback            |