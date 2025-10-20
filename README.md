<p align="center">
  <img src="https://raw.githubusercontent.com/Ranrar/marco/refs/heads/main/documentation/user%20guide/logo.png" />
</p>

<p align="center">
  <img src="https://img.shields.io/badge/CommonMark-100%25-brightgreen?style=for-the-badge&logo=markdown&logoColor=white" alt="100% CommonMark Compliant" />
  <img src="https://img.shields.io/badge/International-Characters-blue?style=for-the-badge&logo=translate&logoColor=white" alt="International Characters Support" />

**Marco** is a fast, native Markdown editor built in Rust with live preview, syntax extensions, and a custom parser for technical documentation.

**Polo**, its companion viewer, lets you open and read Markdown documents with identical rendering and minimal resource use.  

Both are built with **GTK4 and Rust**, designed for speed, clarity, and modern technical writing — with features like **executable code blocks**, **document navigation**, and **structured formatting**.

<p align="center">
  <img src="documentation/Screenshot/Screenshot from 2025-09-17 22-21-06.png" />
</p>
<a href="documentation/Screenshot">View more screenshots</a>

## Quickstart

Ready to try Marco? Installation is simple and takes less than a minute:

```bash
# Clone and install
git clone https://github.com/Ranrar/marco.git
cd marco
bash tests/install/install.sh

# Launch and start writing!
marco    # Full editor with live preview
polo     # Lightweight viewer
```

The install script automatically builds everything and sets up desktop integration. No manual configuration needed—just run and write!

For detailed installation options, see [tests/install/README.md](tests/install/README.md).

## Why Marco?

I started building Marco because I couldn't find a simple, reliable Markdown editor for Linux.  
As an IT systems manager, I've always preferred **local software** — fast, safe, and running entirely on my own machine, not in the cloud.  
In my daily work, I write a lot of **technical documentation and manuals**, so I needed a tool that could handle complex documents efficiently and reliably.

That idea became a personal challenge: to create a complete Markdown editor from the ground up — with a **custom-built parser** and a design focused on performance, clarity, and long-term potential.

---

Most Markdown editors focus on simplicity. Marco focuses on **precision**.

It's built for developers, engineers, and writers who need:
- **Native performance** — no login, no cloud, your documents stay on your machine
- **Structured documents** — full control over headings, blocks, and formatting  
- **Custom Markdown grammar** — hand-crafted parser for extensibility and AST-level control  
- **Seamless preview** — rendered with WebKit and perfectly synced with the editor  

Whether you're writing technical docs, tutorials, or long-form text, Marco turns Markdown into a professional writing tool — fast, clear, and extensible.

## Marco Markdown Flavor

- **Full CommonMark support** — complete compatibility with the standard specification (652/652 tests passing)
- **International characters** — Proper handling of international charters like Japanese (日本語), Arabic (العربية), and emoji (🎉)
- **Executable code blocks** — run Bash, Python, or shell snippets directly in the preview
- **Document navigation** — automatic TOC, bookmarks, and cross-file links  
- **Enhanced content blocks** — callouts, admonitions, mentions, and custom icons  
- **Structured formatting** — semantic elements for headings, notes, and exports  

Marco's parser transforms Markdown into a full document model (AST) for advanced features like live TOC generation, PDF page layouts, and multi-document navigation.

## Architecture & internals

Marco uses a **Cargo workspace** with three crates:

- **`core/`** — Pure Rust library with hand-crafted parser, AST builder, HTML renderer, LSP features, and core logic (buffer management, settings, paths, cache, logging). No GTK dependencies.
- **`marco/`** — Full-featured editor binary with GTK4 UI, SourceView5 text editing, and WebKit6 preview. Depends on `core`.
- **`polo/`** — Lightweight viewer binary with GTK4 UI and WebKit6 preview only (no SourceView5). Depends on `core`.
- **`assets/`** — Centralized workspace assets: themes, fonts, icons, settings.

**Key technologies:**

- **GTK4-RS** (`gtk4`, `glib`, `gio`) - Cross-platform GUI framework providing the main application window, widgets, and event handling. Used for the editor interface, menus, toolbars, and all user interactions.

- **SourceView5** (`sourceview5`) - Advanced text editor component with syntax highlighting and code editing features. Provides the main markdown editing area with features like line numbers, search/replace, and text formatting.

- **WebKit6** (`webkit6`) - Modern web engine for HTML rendering and preview. Displays the live markdown preview with support for local images, custom CSS themes, and JavaScript interactions like scroll synchronization.

- **nom** (`nom`) - Parser combinator library for building the custom markdown grammar. nom uses **recursive descent parsing** where you write Rust functions that parse pieces of input and compose them together. This approach provides total control, incremental parsing capability, and native Rust performance. The parser lives in `core/src/grammar/` and generates an AST for fine-grained control over rendering and extensibility.

- **RON** (`ron`) - Rusty Object Notation for configuration files. Used for settings storage, theme definitions, and user preferences with a human-readable format that's easy to edit and version control.

**Current development focus:**
- Maintaining **100% CommonMark compliance** while adding extensions
- Fine-tuning the **parser grammar** for comprehensive markdown support
- Polishing the **AST builder** and **HTML renderer** components
- Implementing **LSP features** (syntax highlighting, diagnostics, completion, hover)
- Implementing robust **error handling** and **edge case coverage**
- Optimizing **parser performance** and **caching** with Moka

## Roadmap

### Core Parser & LSP (Current Focus)
- [ ] Complete LSP integration with SourceView5 (syntax highlighting, diagnostics, completion, hover)
- [ ] Enhanced AST validation and error reporting
- [ ] Advanced syntax features with linting support
- [ ] Optimize parser performance and caching

### Editor Features
- [x] Multiple layout modes: editor+preview, editor only, preview only, detachable preview
- [x] Scroll sync between editor and preview
- [x] Intelligent search
- [ ] Context menus & toolbar: Quick access to formatting and actions
- [ ] Auto-pairing (automatic insertion/closing of brackets, quotes, etc.)
- [ ] Multi-cursor editing support
- [ ] Syntax highlighting in editor (via LSP)

### Document Features
- [ ] Export to HTML and PDF
- [ ] Page size presets for export (A4, US Letter, etc.)
- [ ] Document navigation: TOC sidebar, bookmarks, cross-file links
- [ ] Smart code blocks with 100+ programming languages
- [ ] Template system for common document types
- [ ] Math rendering: KaTeX support for equations and formulas
- [ ] Diagram support: Mermaid for flowcharts and visualizations

### Advanced Features
- [ ] AI-assisted tools: writing suggestions, grammar checking, content improvement
- [ ] Collaborative editing (Yjs/CRDT): shared document model, multi-cursor, presence awareness
- [ ] Language plugin system (add support for new languages via plugins)

### Distribution & Platform
- [ ] Packaging: AppImage, Flatpak, Snap, .deb, .rpm
- [ ] Cross-platform support: Linux and Windows builds


## Contributing

We welcome contributions of all sizes. Short workflow:

1. Open an issue describing the change or bug you plan to address.
2. Fork the repository and create a feature branch.
3. Add tests where appropriate and keep changes small and focused.
4. Run `cargo build` and `cargo test` locally.
5. Open a pull request referencing the issue and describe the change.

Code style & expectations:

- Keep UI code in `marco/src/ui/` and business logic in `core/src/logic/`.
- Follow Rust idioms (use `Result<T, E>`, avoid panics in library code).
- Add unit tests and integration tests in `tests/` when applicable.

### High-value contributions

If you'd like to make a high-impact contribution, consider one of these areas — open an issue first so we can coordinate:

- Collaborative editing (Yjs / CRDT): add a `marco/src/components/collab/` backend that implements a `CollabBackend` trait and provide in-process tests for concurrent patches and cursor sync.
- AI-assisted tools: add a `marco/src/components/ai/` interface for suggestions/edits; keep adapters off the UI thread and provide a small example implementation.

### Component docs & assets

Reference locations for contributors working on components and translations:

- [marco/src/components/ai/README.md](marco/src/components/ai/README.md) — AI component guidance and interface notes
- [marco/src/components/collab/README.md](marco/src/components/collab/README.md) — Collaboration integration notes and references
- [marco/src/components/language/README.md](marco/src/components/language/README.md) — Localization provider contract and workflow
- [assets/language/language matrix.md](assets/language/language%20matrix.md) — language implementation matrix
