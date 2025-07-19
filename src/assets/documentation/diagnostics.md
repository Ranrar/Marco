# Error/Warning Events & Diagnostics System

## Overview
This documentation describes the design, usage, and extension of the error/warning event system in the Marco Markdown parser. The system ensures that parse errors, warnings, and unsupported features are surfaced as events, allowing renderers and plugins to handle them gracefully and safely.

## Event Types
Diagnostics are emitted as variants of the core `Event` enum:
- **Error(String, Option<SourcePos>)**: Critical parse errors (e.g., invalid syntax)
- **Warning(String, Option<SourcePos>)**: Non-fatal issues (e.g., deprecated or ambiguous syntax)
- **Unsupported(String, Option<SourcePos>)**: Features not yet implemented or recognized

All diagnostics events carry a message and optional source position for precise reporting.

## Emission
- **Lexer & Parser**: Emit diagnostics events during parsing. Events are pushed to the event stream, not just logged or ignored.
- **Streaming**: Diagnostics are part of the same iterator as normal events, so consumers (renderers, plugins) can handle them uniformly.

## Thread Safety
- Diagnostics are passed as a mutable reference (`&mut Diagnostics`) to the parser and lexer.
- No global mutable state or unsafe code. Each thread can have its own diagnostics instance.
- For multi-threaded parsing, create a diagnostics instance per thread, or use `Arc<Mutex<Diagnostics>>` for aggregation.

## Renderer Integration
- Renderers match on diagnostics events and display them in output (e.g., as `<span class='error'>...` in HTML).
- Example:
  ```rust
  let mut diagnostics = Diagnostics::new();
  for event in EventIter::new(ast, Some(&mut diagnostics)) {
      match event {
          Event::Error(msg, pos) => html.push_str(&format!("<span class='error'>Error: {} at {:?}</span>", msg, pos)),
          Event::Warning(msg, pos) => html.push_str(&format!("<span class='warning'>Warning: {} at {:?}</span>", msg, pos)),
          Event::Unsupported(msg, pos) => html.push_str(&format!("<span class='unsupported'>Unsupported: {} at {:?}</span>", msg, pos)),
          _ => { /* ...normal rendering... */ }
      }
  }
  ```

## Plugin Support
- Plugins can intercept diagnostics events via the `DiagnosticsInterceptor` trait.
- Example:
  ```rust
  struct LoggingDiagnosticsPlugin;
  impl DiagnosticsInterceptor for LoggingDiagnosticsPlugin {
      fn intercept(&mut self, event: &Event) {
          match event {
              Event::Error(msg, pos) => eprintln!("[PLUGIN] Error at {:?}: {}", pos, msg),
              Event::Warning(msg, pos) => println!("[PLUGIN] Warning at {:?}: {}", pos, msg),
              Event::Unsupported(msg, pos) => println!("[PLUGIN] Unsupported at {:?}: {}", pos, msg),
              _ => {}
          }
      }
  }
  ```

## Example Usage
- **Single-threaded:**
  ```rust
  let mut diagnostics = Diagnostics::new();
  for event in EventIter::new(ast, Some(&mut diagnostics)) {
      // handle event
  }
  diagnostics.report();
  ```
- **Multi-threaded:**
  ```rust
  use std::sync::{Arc, Mutex};
  let diagnostics = Arc::new(Mutex::new(Diagnostics::new()));
  let diagnostics_clone = diagnostics.clone();
  std::thread::spawn(move || {
      let mut diag = diagnostics_clone.lock().unwrap();
      for event in EventIter::new(ast, Some(&mut *diag)) {
          // handle event
      }
  });
  ```

## Extensibility
- New diagnostics types or metadata can be added to the `Event` enum.
- Renderers and plugins can be customized to handle diagnostics differently (e.g., suppress, style, aggregate).
- Diagnostics can be aggregated, filtered, or transformed via plugins or event pipelines.

## Best Practices
- Always surface parse errors, warnings, and unsupported features as events.
- Avoid global mutable state; use local diagnostics or synchronization primitives for shared state.
- Document diagnostics handling for plugin authors and contributors.

## References
- [Rust Book: Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- [Rust std::thread docs](https://doc.rust-lang.org/std/thread/)
- [users.rust-lang.org: Thread-safe diagnostics](https://users.rust-lang.org/search?q=thread%20safe%20diagnostics)

---

This system ensures robust, extensible, and thread-safe diagnostics handling for Markdown parsing and rendering in Rust.
