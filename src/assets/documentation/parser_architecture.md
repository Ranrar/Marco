# Marco2 Markdown Parser Architecture & Event Stream Design

## Overview
Marco2 is a modular, extensible Markdown parser designed for advanced event streaming, filtering, and plugin support. This document covers best practices, architecture, and extension points for developers and plugin authors.

---

## 1. AST & Event Stream Design

### A. Store `Block` Directly in Lists
- **Why:** Safe, idiomatic traversal; references are always valid.
- **How:** Use `Vec<Block>` for block containers.
- **Result:** You can safely push `&Block` references to your stack.

### B. Shared Ownership (`Rc<Block>`/`Arc<Block>`)
- **Why:** Enables sharing blocks across the AST.
- **How:** Use `Rc<Block>` (single-threaded) or `Arc<Block>` (multi-threaded).
- **Result:** Clone and share blocks safely.

### C. Arena Allocation
- **Why:** For very large ASTs, use an arena allocator (e.g., `typed-arena`, `bumpalo`).
- **Result:** All blocks live for the lifetime of the arena; references are always valid.

---

## 2. Module Layout

```
/src/editor/logic/
├── mod.rs
├── parser/
│   ├── mod.rs              # Organizes all parser-related modules
│   ├── lexer.rs            # Tokenizer: raw Markdown → Tokens (with SourcePos)
│   ├── event.rs            # Core event types: Event, Tag, TagEnd, SourcePos
│   ├── parser.rs           # Main parser: Token stream → Event stream or AST
│   ├── emitter.rs          # EventEmitter: walk AST, emit Event stream
│   ├── transform.rs        # Event filters, mappers, plugin hooks
│   ├── extensions.rs       # GFM support, custom syntax extensions
│   ├── group.rs            # Groups related events into logical blocks
│   ├── diagnostics.rs      # Error/warning reporting, profiling
│   └── attributes.rs       # Custom attributes: class, id, data-*
```

---

## 3. Advanced Event Stream Features

- **Full Inline Traversal:** Emits events for all inline types (text, code, emphasis, links, images, autolinks, raw HTML, breaks, etc.).
- **Source Position Tracking:** All events include line/column info for error reporting, highlighting, and mapping output to source.
- **Custom Attributes:** Events can carry custom attributes (classes, IDs, data-*). Attribute parsing from Markdown syntax (`{.class #id}`) is supported.
- **Extension Support:** GFM features (tables, task lists, strikethrough, etc.), math blocks, emoji, mentions, and code block info strings are supported via specialized events.
- **Event Filtering/Transformation:** Users can filter, transform, or intercept events for plugins, custom rendering, or analytics.
- **Streaming Output:** Iterator design emits events efficiently for large documents (no buffering).
- **Error/Warning Events:** Events for parse errors, warnings, or unsupported features allow graceful handling by renderers.
- **Custom Tag Types:** Easily add custom tags for user-defined extensions, block types, or inline types.
- **Event Grouping:** Logical groups (e.g., list items, table rows) simplify rendering.
- **Performance/Profiling Hooks:** Emit timing/memory usage events for profiling and optimization.

---

## 4. Event Filtering & Transformation API

### Traits & Pipeline
```rust
pub trait EventFilter {
    fn filter(&mut self, event: &mut Event) -> bool;
}

pub trait EventMapper {
    fn map(&mut self, event: &mut Event);
}

pub struct EventPipeline {
    pub filters: Vec<Box<dyn EventFilter>>,
    pub mappers: Vec<Box<dyn EventMapper>>,
}
```

### Example Usage
```rust
let mut pipeline = EventPipeline::new();
pipeline.add_filter(|event: &mut Event| !matches!(event, Event::SoftBreak(_, _)));
pipeline.add_mapper(|event: &mut Event| {
    if let Event::Text(ref mut s, _, _) = event {
        *s = s.to_uppercase();
    }
});
```

### Integration
- The pipeline is integrated into the parser and emitter, so events can be filtered/transformed as they stream.
- Users can compose multiple filters/mappers for custom behavior.

---

## 5. Extending & Plugin Development

- **Add new event types:** Extend `Event`, `Tag`, and related enums for custom syntax or features.
- **Custom attributes:** Use the attribute system for advanced rendering and plugin hooks.
- **Diagnostics:** Use `diagnostics.rs` for error/warning reporting and profiling.
- **GFM & Extensions:** Add support for new Markdown/GFM features in `extensions.rs`.
- **Performance:** Use event grouping and streaming for efficient rendering of large documents.

---

## 6. Best Practices & Anti-Patterns

- Avoid unnecessary `.clone()`, `.unwrap()`, or panics in filter/mapper logic.
- Prefer trait-based composition for extensibility.
- Use streaming/event-driven design for performance.
- Document extension points for plugin authors.

---

## 7. Summary Table

| Approach         | Pros                        | Cons                        | When to Use                |
|------------------|----------------------------|-----------------------------|----------------------------|
| Store `Block`    | Simple, idiomatic, safe     | Slightly larger AST         | Most Markdown parsers      |
| Use `Rc<Block>`  | Shared ownership, flexible  | More heap allocations       | Complex, shared ASTs       |
| Arena Allocation | Fast, minimal allocations   | More complex setup          | Large, performance-critical|

---

## 8. References & Further Reading
- [Rust Book: Ownership & Lifetimes](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- [Rust Arena Allocation: bumpalo](https://docs.rs/bumpalo)
- [GFM Spec](https://github.github.com/gfm/)
- [Rust Markdown Ecosystem](https://lib.rs/markdown)
- [Plugin Architecture Patterns](https://users.rust-lang.org/)

---

*For questions, plugin development, or advanced extension support, see the source code in `/src/editor/logic/` and the integration tests in each module.*
