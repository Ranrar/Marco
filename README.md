<p align="center">
  <img src="https://raw.githubusercontent.com/Ranrar/marco/refs/heads/main/documentation/user%20guide/logo.png" />
</p>

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
- **Native performance** — no Electron, no lag, built in Rust + GTK4  
- **Structured documents** — full control over headings, blocks, and formatting  
- **Custom Markdown grammar** — powered by Pest for extensibility and AST-level parsing  
- **Seamless preview** — rendered with WebKit and perfectly synced with the editor  

Whether you're writing technical docs, tutorials, or long-form text, Marco turns Markdown into a professional writing tool — fast, clear, and extensible.

## Marco Markdown Flavor

Marco extends **CommonMark** with features designed for technical and long-form writing:

- **Executable code blocks** — run Bash, Python, or shell snippets directly in the preview
- **Document navigation** — automatic TOC, bookmarks, and cross-file links  
- **Enhanced content blocks** — callouts, admonitions, mentions, and custom icons  
- **Structured formatting** — semantic elements for headings, notes, and exports  

Powered by a **Pest-based parser**, Marco turns Markdown into a full document model (AST) for advanced features like live TOC generation, PDF page layouts, and multi-document navigation.

## Architecture & internals

- **GTK4-RS** (`gtk4`, `glib`, `gio`) - Cross-platform GUI framework providing the main application window, widgets, and event handling. Used for the editor interface, menus, toolbars, and all user interactions.

- **SourceView5** (`sourceview5`) - Advanced text editor component with syntax highlighting and code editing features. Provides the main markdown editing area with features like line numbers, search/replace, and text formatting.

- **WebKit6** (`webkit6`) - Modern web engine for HTML rendering and preview. Displays the live markdown preview with support for local images, custom CSS themes, and JavaScript interactions like scroll synchronization.

- **Pest** (`pest`, `pest_derive`) - Parser generator for creating the custom markdown grammar. Used in the marco_engine component to parse markdown into an AST, enabling fine-grained control over rendering and future extensibility for custom markdown features.

- **RON** (`ron`) - Rusty Object Notation for configuration files. Used for settings storage, theme definitions, and user preferences with a human-readable format that's easy to edit and version control.

**Current development focus:**
- Fine-tuning the **pest-based grammar** for comprehensive markdown support
- Polishing the **AST builder** and **HTML renderer** components
- Implementing robust **error handling** and **edge case coverage**
- Optimizing **parser performance** and **memory usage**

## Roadmap

Planned and desired improvements

- AI-assisted tools: assistant for writing and editing suggestions
- Collaborative editing (Yjs/CRDT): shared document model, multi-cursor, presence
- Enhanced AST validation and UI for syntax hints
- Packaging and distribution
- Language plugin system (add new language support via plugins)
- Advanced syntax features with linting support
- Auto-pairing (automatic insertion/closing of brackets, quotes, etc.)
- [x] Multiple layout modes: editor+preview (standard), editor only, preview only, detachable preview
- Export / Save as HTML or PDF
- Page size presets for export (A4, US Letter)
- [x] Scroll sync between editor and preview
- Context menus & toolbar: Quick access to formatting and actions
- Smart code blocks: 100+ programming languages,
- [x] Intelligent search
- Syntax highlighting


## Contributing

We welcome contributions of all sizes. Short workflow:

1. Open an issue describing the change or bug you plan to address.
2. Fork the repository and create a feature branch.
3. Add tests where appropriate and keep changes small and focused.
4. Run `cargo build` and `cargo test` locally.
5. Open a pull request referencing the issue and describe the change.

Code style & expectations:

- Keep UI code in `marco/src/ui/` and business logic in `marco_core/src/logic/`.
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
