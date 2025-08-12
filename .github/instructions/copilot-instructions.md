# Copilot Instructions for `Marco` Editor Logic

## Big Picture Architecture
- Modular Rust codebase: Each editor feature is in its own module under `src/logic/editor/` (e.g., `core.rs`, `multicursor.rs`, `mouse_selection.rs`).
- Main struct: `EditorBuffer` manages buffer state, cursors, selections, undo/redo, and rendering.
- Rendering logic (A4, margin guides, etc.) is separated from buffer logic for maintainability and testability.
- UI integration is handled in the main app (see `src/ui/`), not in the editor logic library.

## Developer Workflows
- Build: Use `cargo build` or `cargo check` in the workspace root.
- Test: Run `cargo test` for unit and integration tests. Editor logic is designed for headless testing.
- Debug: Use `dbg!()` for temporary logging. For GTK integration, ensure all UI code runs on the main thread.
- Formatting: Use `cargo fmt` and `cargo clippy` for style and linting.

## Project-Specific Conventions
- Each feature (undo, multicursor, selection, etc.) is in its own file under `src/logic/editor/`.
- All modules are exposed via `lib.rs` for library use. Example: `pub use core::EditorBuffer;`.
- Avoid global mutable state; use `Rc<RefCell<T>>` for shared state in tests and logic.
- Multi-cursor and multi-selection logic is always handled via dedicated vectors in `EditorBuffer`.
- Rendering logic (A4, margin guides) is decoupled from buffer logic for testability.

## Integration Points & Dependencies
- No direct GTK or UI dependencies in the editor logic library. All UI code is in the main app (`src/ui/`).
- External crates: `ron`, `serde`, `regex`, `markdown`, etc. for buffer and document logic.
- For collaborative editing, planned integration with Yjs CRDT (see roadmap in `README.md`).
- All cross-component communication is via explicit function calls and shared state structs.

## Examples & Patterns
- To add a new feature, create a new module in `src/logic/editor/` and expose it in `lib.rs`.
- To extend buffer logic, add methods to `EditorBuffer` and update relevant modules.
- For tests, use `#[cfg(test)]` and place them in the same module or in `tests.rs`.
- For rendering, use dedicated modules and keep UI code out of the core logic.

## Key Files
- `src/logic/editor/lib.rs`: Library entry point, exposes all modules.
- `src/logic/editor/core.rs`: Main buffer struct and logic.
- `src/logic/editor/multicursor.rs`: Multi-cursor logic.
- `src/logic/editor/mouse_selection.rs`: Mouse selection logic.
- `src/logic/editor/render.rs`: Rendering logic (A4, margins, guides).
- `src/logic/editor/undo.rs`, etc.: Feature modules.

## Communication
- Always explain your reasoning before making changes.
- Use markdown todo lists for step-by-step plans.
- Reference specific files and modules when describing changes.

---
If any section is unclear or missing, ask the user for feedback and iterate to improve these instructions.
