<p align="center">
  <img src="https://github.com/Ranrar/marco/blob/main/documentation/user%20guide/logo.png" />
</p>

Marco — a lightweight Rust Markdown Composer, it is a GTK-based editor written in Rust. It's an experimental, extensible editor focused on structured editing, syntax-aware features, and custom markdown features.

## Key features

- Live split-pane editing: Write Markdown on the left, see formatted preview on the right
- Advanced Markdown: GitHub-style admonitions, tables, task lists, code blocks
- Real-time preview: Fast, accurate, and interactive
- Theme configuration for both editor and viewer

## Architecture & internals

- GTK4-based UI with modular components (editor, viewers, toolbar, menus)
- Asset pipeline for icons, fonts, and themes (see `src/assets/`)
- Support for adding custom AST and syntax definitions `src/assets/markdown_schema`

## Quickstart

Prerequisites
- Rust toolchain (stable) with `cargo`
- GTK4 development libraries installed on your system (for Linux: libgtk-4-dev or distro equivalent)

Build and run locally

1. Build:

	cargo build --release

2. Run the app (from repo root):

	cargo run --release

Run tests

	cargo test --lib --tests -- --nocapture

## Project layout (high level)

- `src/` — application code (UI components, logic, menus, theme)
- `logic/` — core parsing, loaders, buffer and layout code
- `tests/` — integration and unit tests
- `Cargo.toml` — Rust manifest and dependencies

## Roadmap

Planned and desired improvements

- AI-assisted tools: assistant for writing and editing suggestions
- Collaborative editing (Yjs/CRDT): shared document model, multi-cursor, presence
- Enhanced AST validation and UI for syntax hints
- Packaging and distribution
- Language plugin system (add new language support via plugins)
- Advanced syntax features with linting support
- Auto-pairing (automatic insertion/closing of brackets, quotes, etc.)
- Multiple layout modes: editor+preview (standard), editor only, preview only, detachable preview
- Export / Save as HTML or PDF
- Page size presets for export (A4, US Letter)
- Scroll sync between editor and preview
- Context menus & toolbar: Quick access to formatting and actions
- Smart code blocks: 100+ programming languages,
- Intelligent search
- Syntax highlighting


## Contributing

We welcome contributions of all sizes. Short workflow:

1. Open an issue describing the change or bug you plan to address.
2. Fork the repository and create a feature branch.
3. Add tests where appropriate and keep changes small and focused.
4. Run `cargo build` and `cargo test` locally.
5. Open a pull request referencing the issue and describe the change.

Code style & expectations:

- Keep UI code in `src/ui/` and business logic in `logic/`.
- Follow Rust idioms (use `Result<T, E>`, avoid panics in library code).
- Add unit tests and integration tests in `tests/` when applicable.

### High-value contributions

If you'd like to make a high-impact contribution, consider one of these areas — open an issue first so we can coordinate:

- Collaborative editing (Yjs / CRDT): add a `components/collab/` backend that implements a `CollabBackend` trait and provide in-process tests for concurrent patches and cursor sync.
- AI-assisted tools: add a `components/ai/` interface for suggestions/edits; keep adapters off the UI thread and provide a small example implementation.

### Component docs & assets

Reference locations for contributors working on components and translations:

- [src/components/ai/README.md](src/components/ai/README.md) — AI component guidance and interface notes
- [src/components/collab/README.md](src/components/collab/README.md) — Collaboration integration notes and references
- [src/components/language/README.md](src/components/language/README.md) — Localization provider contract and workflow
- [src/assets/language/language matrix.md](src/assets/language/language%20matrix.md) — language implementation matrix

If you add new component folders, please include a short `README.md` in the folder that explains the contract, tests, and how to run the component's dev harness.
