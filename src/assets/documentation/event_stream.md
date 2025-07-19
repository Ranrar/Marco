# Event Stream & Profiling Hooks: Safety and Usage Guide

## Overview
This document describes the event stream types, profiling hooks, and best practices for safe, idiomatic Rust and GTK4 code in the marco2 Markdown parser. It is intended for plugin authors, contributors, and advanced users.

---

## Event Stream Types
- The event stream is defined in `src/editor/logic/parser/event.rs`.
- Core event types include: `Start`, `End`, `Text`, `Code`, `Html`, `GroupStart`, `GroupEnd`, `Profile`, and more.
- Custom tags and extensible event variants are supported for plugin/filter development.

---

## Profiling Hooks
- Use `Event::Profile(ProfileType, value, timestamp)` to emit timing and memory usage events.
- `ProfileType` variants include: `ParseStart`, `ParseEnd`, `BlockRender`, `MemoryUsage`.
- Example:
  ```rust
  use crate::editor::logic::parser::event::{Event, ProfileType};
  let event = Event::Profile(ProfileType::ParseEnd, 12345, 1620000000);
  ```

---

## Safety and Threading Guidelines (GTK4 + Rust)
- **GTK4 is NOT thread-safe:** All widget creation, updates, and signal handling must occur on the main thread.
- **Profiling/Diagnostics:** Emit profiling events only from the main thread, or use message passing (e.g., `glib::Sender`, channels) to communicate results from worker threads.
- **Memory Safety:** Use Rust's ownership/borrowing model. For shared state, prefer `Rc<RefCell<T>>` (single-threaded) or `Arc<Mutex<T>>` (multi-threaded). Avoid `.clone()` unless necessary; use `Weak` to break cycles.
- **Thread Safety:** Never update GTK widgets from non-main threads. Use `glib::MainContext` or `glib::idle_add_local()` to schedule UI updates.
- **Anti-patterns:** Avoid `.unwrap()`/`.expect()` in event emission, global mutable state, and nested locks. Release locks promptly.

---

## Example: Safe Profiling Event Emission
```rust
use glib::MainContext;
use crate::editor::logic::parser::event::{Event, ProfileType};
let sender = MainContext::channel(glib::PRIORITY_DEFAULT);
std::thread::spawn(move || {
    let value = 12345;
    let timestamp = 1620000000;
    sender.send(Event::Profile(ProfileType::ParseEnd, value, timestamp)).unwrap();
});
// In main thread: receive and emit event
```

---

## References
- [GTK4 Rust Book](https://gtk-rs.org/gtk4-rs/git/book/)
- [Rust Concurrency Guide](https://doc.rust-lang.org/book/ch20-00-concurrency.html)
- [Relm4 Docs](https://relm4.org/docs/stable/gtk4/index.html)
- [Master Rust Concurrency](https://codezup.com/mastering-rust-concurrency-thread-safe-data-structures/)

---

## For Plugin Authors
- Always emit events in a thread-safe manner.
- If profiling in a worker thread, send results to the main thread before emitting.
- Extend event types and profiling hooks as needed for analytics and diagnostics.

For questions or contributions, see the main code documentation in `src/editor/logic/parser/event.rs`.
