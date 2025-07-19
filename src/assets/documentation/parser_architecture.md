---

## 12. Error/Warning Events & Diagnostics Integration

Marco2 emits error, warning, and unsupported feature events in the event stream, allowing renderers and plugins to handle issues gracefully.

- **Event Enum:**
    - `Event::Error(String, Option<SourcePos>)`
    - `Event::Warning(String, Option<SourcePos>)`
    - `Event::Unsupported(String, Option<SourcePos>)`
- **Emission:**
    - Lexer, parser, and extensions emit these events on parse errors, deprecated syntax, or unsupported features.
- **Diagnostics Integration:**
    - Centralized in `diagnostics.rs` for reporting, logging, and analytics.
- **Renderer/Plugin Handling:**
    - Plugins and renderers can intercept these events for display, logging, or custom handling.

### Example
```rust
if invalid_token {
    events.push(Event::Error("Unrecognized token".to_string(), Some(pos)));
}
if deprecated_syntax {
    events.push(Event::Warning("Deprecated syntax used".to_string(), Some(pos)));
}
if unsupported_feature {
    events.push(Event::Unsupported("Footnotes not supported".to_string(), Some(pos)));
}
```

**Best Practice:**
Emit events for all errors/warnings, never panic or silently skip. Let consumers decide how to handle issues.
# Marco Markdown Parser Architecture & Event Stream Design

## Overview
Marco is a modular, extensible Markdown parser designed for advanced event streaming, filtering, and plugin support. This document covers best practices, architecture, and extension points for developers and plugin authors.

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



## 4. Event Filtering & Transformation API

### Traits & Pipeline
```rust
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

## 9. Extension Points for Plugins

### Where to Extend
- **EventPipeline (transform.rs):** Add custom filters/mappers for analytics, logging, or custom rendering.
- **Event/Tag Enums (event.rs):** Add new event/tag types for custom syntax or features.
- **Emitter (emitter.rs):** Intercept or transform events as they are emitted.
- **Diagnostics (diagnostics.rs):** Intercept error/warning events for reporting or profiling.
- **Extensions (extensions.rs):** Add support for new Markdown/GFM features or custom syntax.

### Example: Custom Plugin for Analytics
```rust
use crate::editor::logic::parser::event::Event;
use crate::editor::logic::transform::{EventPipeline, EventFilter};

struct EventCounter {
    pub count: usize,
}

impl EventFilter for EventCounter {
    fn filter(&mut self, event: &mut Event) -> bool {
        self.count += 1;
        true // keep all events
    }
}

let mut counter = EventCounter { count: 0 };
let mut pipeline = EventPipeline::new();
pipeline.add_filter(counter);
// Use pipeline in parser/emitter, then read counter.count
```

### Example: Timing Plugin
```rust
use std::time::Instant;
struct Timer {
    start: Instant,
}
impl EventFilter for Timer {
    fn filter(&mut self, _event: &mut Event) -> bool {
        // Log elapsed time per event
        println!("Elapsed: {:?}", self.start.elapsed());
        true
    }
}
```

### Example: Diagnostics Interception
```rust
// In diagnostics.rs, define a filter for error/warning events
struct ErrorLogger;
impl EventFilter for ErrorLogger {
    fn filter(&mut self, event: &mut Event) -> bool {
        if let Event::Error(msg, pos) = event {
            eprintln!("Error at {:?}: {}", pos, msg);
        }
        true
    }
}
```

---

## 10. Notes for Advanced Plugins & Diagnostics

- **Advanced Plugins:**
    - Consider adding hooks for analytics (e.g., event counters, timing).
    - Use filters/mappers to collect statistics, log events, or profile performance.
- **Diagnostics Integration:**
    - Integrate with `diagnostics.rs` to intercept error/warning events for reporting, logging, or custom handling.
    - Use event stream hooks to monitor and respond to diagnostics in real time.

---

*For questions, plugin development, or advanced extension support, see the source code in `/src/editor/logic/` and the integration tests in each module.*

