<p align="center">
  <img src="https://raw.githubusercontent.com/Ranrar/marco/refs/heads/main/documentation/user_guide/Logo_marco_and_polo.png" />
</p>

<p align="center">
  <img src="https://img.shields.io/badge/CommonMark-100%25-brightgreen?style=for-the-badge&logo=markdown&logoColor=white" alt="100% CommonMark Compliant" />
  <img src="https://img.shields.io/badge/International-Characters-blue?style=for-the-badge&logo=translate&logoColor=white" alt="International Characters Support" />

**Marco** is a fast Markdown editor built in Rust with live preview, syntax extensions, and a custom parser for technical documentation.

**Polo**, its companion viewer, lets you open and read Markdown documents with identical rendering and minimal resource use.  

Both are built with **GTK4 and Rust**, designed for speed, clarity, and modern technical writing — with features like **executable code blocks**, **document navigation**, and **structured formatting**.

<p align="center">
  <img src="documentation/Screenshot/Screenshot from 2025-09-17 22-21-06.png" />
</p>
<a href="documentation/Screenshot">View more screenshots</a>

## Quickstart

Ready to try Marco? Installation is simple and takes less than a minute:

## Linux

### Alpha (latest dev build)

Download the latest Alpha `.deb` from the **Alpha** release:

- https://github.com/Ranrar/Marco/releases/tag/alpha

The asset is currently published as:

- `marco-suite_alpha_amd64.deb`

Install it (Debian/Ubuntu):

1. Download the `*.deb` asset for your architecture (typically `amd64`).
2. Install with your package manager (e.g. `dpkg`), then resolve any missing dependencies if prompted.

## Windows

No option yet.



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

## Marco Markdown Functions

Marco aims for **100% CommonMark compliance** (currently 652/652 spec tests passing), plus a growing set of carefully-scoped extensions.

| Markdown / Syntax feature | Status | Notes |
|---|---|---|
| CommonMark core (block + inline) | ✅ Supported | Includes ATX + Setext headings, paragraphs, blockquotes, thematic breaks, lists, code spans/blocks, links, images, HTML blocks/inlines, hard/soft breaks, and entity references. |
| International text (Unicode) | ✅ Supported | Works with non-Latin scripts (e.g. 日本語, العربية) and emoji. |
| Heading IDs (`# Title {#id}`) | ✅ Supported | Extension for stable anchors/links. |
| Autolinks (`<https://…>` / `<user@…>`) | ✅ Supported | CommonMark autolinks (email becomes `mailto:`). |
| GFM-style autolink literals (`https://…`, `www.…`, `user@…`) | ✅ Supported | Rendered as links when detected in text. |
| Reference-style links (`[text][label]`, `[label][]`, `[label]`) | ✅ Supported | Resolved against `[label]: url` definitions (supports forward definitions). |
| Task lists (`- [ ]` / `- [x]`) | ✅ Supported | Rendered with themed checkbox icons. Also supports checklist-style paragraph markers (`[ ]` / `[x]` / `[X]`) and mid-paragraph markers like `Do this [ ] today`. |
| Tables (GFM pipe tables) | ✅ Supported | Header/body separation + per-column alignment. |
| Headerless pipe tables (delimiter-first, no header row) | ✅ Supported | Marco extension: the first line is the delimiter row, followed by 1+ body rows; renders as a normal table with `<tbody>` only. |
| Strikethrough (`~~text~~`) | ✅ Supported | GFM extension. |
| Admonitions / callouts | ✅ Supported | GitHub-style alerts (e.g. Note/Tip/Important/Warning/Caution) plus an extended custom-header form: `> [:joy: Happy Header]` (quote-styled with a custom emoji/icon + title). |
| Footnotes (`[^a]` + `[^a]: …`) | ✅ Supported | Rendered as an end-of-document footnotes section. |
| Inline footnotes (`^[...]`) | ✅ Supported | Marco extension: inline footnote content is defined at the reference point and rendered into the same footnotes section. |
| Highlight/mark (`==text==`) | ✅ Supported | Rendered as `<mark>…</mark>`. |
| Superscript / subscript | ✅ Supported | Rendered as `<sup>…</sup>` / `<sub>…</sub>`. |
| Emoji shortcodes (`:joy:`) | ✅ Supported | Only recognized shortcodes convert; unknown ones stay literal text. |
| User mentions (`@name[platform]`) | ✅ Supported | Marco extension: renders as a profile link when `(platform, username)` maps to a stable public profile URL; otherwise renders as non-link text. |
| Inline checkboxes mid-paragraph (`... [x] ...`) | ✅ Supported | Marco extension: `[ ]` / `[x]` / `[X]` markers are recognized inside normal text (with conservative parsing to avoid breaking link syntax). |
| Tab blocks (`:::tab` + `@tab`) | ✅ Supported | Marco extension: `:::tab` container with `@tab <title>` headers; renders as a no-JS tab UI in the HTML preview (radio+label panels). Nested tab blocks are intentionally not supported. |
| Slideshow decks (`@slidestart` / `@slideend`) | ✅ Supported | Marco extension: author slide decks inside Markdown using `@slidestart[:tN]` … `@slideend`. Use `---` for horizontal slide breaks and `--` for vertical breaks (stored as metadata). Renders as an interactive slideshow in the preview (controls + dots); adds autoplay when a timer is provided. |
| YouTube embeds | Not implemented yet | Planned (URLs render as links today; embed would be opt-in). |
| Math (KaTeX / LaTeX) | Not implemented yet | Planned. |
| Diagrams (Mermaid) | Not implemented yet | Planned. |

## Future functions in pipeline

- **Executable code blocks** — run Bash, Python, or shell snippets directly in the preview
- **Document navigation** — automatic TOC, bookmarks, and cross-file links  
- **Enhanced content blocks** — callouts, admonitions, mentions, and custom icons
- **Structured formatting** — semantic elements for headings, notes, and exports  
- **Export to PDF** - Export into PDF in A4 or US Letter
- **Templates** — start from predefined markdown templates (README, runbook, etc.)

## AI-assisted development

This project is developed with occasional help from AI tools (for example, Copilot-style code suggestions). AI can speed up prototyping and refactors, but:

- Changes are still reviewed by a human.
- Tests and linting are expected to pass before merging.
- If something looks "too magical to be true", please open an issue — bugs don't get a free pass just because a robot wrote the first draft.

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

### Editor Features (Marco)
- [x] Multiple layout modes: editor+preview, editor only, preview only, detachable preview
- [x] Scroll sync between editor and preview
- [x] Intelligent search
- [ ] Context menus & toolbar: Quick access to formatting and actions
- [ ] Auto-pairing (automatic insertion/closing of brackets, quotes, etc.)
- [ ] Multi-cursor editing support
- [ ] Syntax highlighting in editor (via LSP)

### Viewer Fetures (Polo)
- [x] Same viewer engine as Marco
- [ ] Search function
- [ ] Mouse over link information 

### Document Features
- [x] Smart code blocks with programming languages syntax
- [ ] Export to PDF
- [ ] Page size presets for export (A4, US Letter, etc.)
- [ ] Document navigation: TOC sidebar, bookmarks, cross-file links
- [ ] Template system for common document types
- [ ] Math rendering: KaTeX support for equations and formulas
- [ ] Diagram support: Mermaid for flowcharts and visualizations

### Advanced Features
- [ ] Local AI-assisted tools: writing suggestions, grammar checking, content improvement
- [ ] Collaborative editing (Yjs/CRDT): shared document model, multi-cursor, presence awareness
- [ ] Language plugin system (add support for new languages via plugins)

### Distribution & Platform
- [ ] Packaging: Snap, .deb and (.MSI or.exe)
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
