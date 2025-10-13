# **Markdown/LSP architecture flowchart**

```
Source Code (Markdown, etc.)
        │
        ▼
┌───────────────────────────────┐
│        Pest Parsers           │
│   (Modular per element)       │
│ ┌───────────────┐ ┌─────────┐ │
│ │ Block Parsers │ │ Inline  │ │
│ │ (paragraph,   │ │ Parsers │ │
│ │ headings,     │ │ (emphasis,│
│ │ lists, etc.)  │ │ links, etc.)│
│ └───────────────┘ └─────────┘ │
└─────────────┬─────────────────┘
              │
              ▼
             AST
       (combined abstract syntax tree)
              │
      ┌───────┴────────┐
      ▼                ▼
 Renderer (HTML, etc.)  Syntax Extraction
                             │
                             ▼
                          LSP Server
                             │
       ┌───────────────┬──────────────┬───────────────┐
       ▼               ▼              ▼               ▼
 Syntax Highlighting  Autocomplete    Markdown Help   Lint / Diagnostics
                     (basic +       (context-aware +  (show errors in editor)
                     context-aware) suggestions)
```

### 🔹 Notes on this structure

1. **Pest Parsers**

   * Separate modular files per element (e.g., paragraph.pest, emphasis.pest).
   * Each parser produces AST fragments; `ast_builder.rs` combines them.

2. **AST (Abstract Syntax Tree)**

   * Single source of truth for both rendering and LSP features.

3. **Renderer**

   * Converts AST into HTML, Markdown, LaTeX, etc.

4. **Syntax Extraction**

   * Captures positions and metadata for LSP: highlighting, linting, autocomplete.

5. **LSP Server**

   * Uses AST + syntax metadata to provide real-time features:

     * Syntax Highlighting
     * Autocomplete (basic + context-aware)
     * Markdown writing assistance + suggestions
     * Linting / diagnostics