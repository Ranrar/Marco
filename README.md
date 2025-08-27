<p align="center">
  <img src="https://github.com/Ranrar/marco/blob/main/documentation/user%20guide/logo.png" />
</p>

Marco — a lightweight Rust Markdown Composer, it is a GTK-based editor written in Rust. It's an experimental, extensible editor focused on structured editing, syntax-aware features, and custom markdown features.

## Features in Progress

The **UI is mostly complete**, and we’re now turning our focus to the engine behind Marco. We’re actively developing the **grammar, AST, and syntax rules** for the parser and renderer, which will bring the editor to life.

Right now, the editor is in an early stage — features are limited — but this will change as we implement the new Markdown capabilities.

Below is a **preview of what’s coming**:

- Structured **grammar and AST** for Markdown parsing
- Fully-featured **renderer** for inline and block elements
- Interactive **TOC, bookmarks, and page navigation**
- **Run code snippets** safely from the preview
- Enhanced **admonition blocks** and **mentions**
- Smooth **cross-document navigation** and page splitting

You can see a **live snippet** of the Markdown features we’re working on in the roadmap below. This is just the beginning — soon, Marco will let you edit, preview, and navigate Markdown like never before.

## Text Formatting

* **Headings**: Use `#` at the start of a line to create titles and subtitles. These show up in the Table of Contents and help structure your document.
* **Bold**: Surround text with `**` to make it stand out.
* **Italic**: Surround text with `__` to add gentle emphasis.
* **Strikethrough**: Surround text with `--` to mark something as removed or outdated.
* **Highlight**: Surround text with `==` to draw attention without bolding.
* **Emoji**: Write `:smile:` and it will show 😊.

## Code, Math and Diagrams

* **Inline code**: use text between backticks `` ` `` to show short code snippets.
* **Code blocks**: Use triple backticks to create larger blocks of code. You can even add a language for syntax highlighting.
* **inline Math**: Put formulas between `$...$` for inline math.
* **Math block**: Use `$$...$$` for larger math expressions that display on their own line.
* **Diagrams**: Use special fenced blocks (like Mermaid or Graphviz) to render flowcharts, graphs, or diagrams directly in your document.

---

## Run code directly from Marco Preview

Marco lets you run code snippets **directly from your Markdown preview**, making your documentation interactive and hands-on.

* **Supported languages**: Bash, Zsh, Sh, Bat, PowerShell, and Python.

* **Safety first**: Marco always asks before running code. You can choose to run it in a **sandbox** (safe, isolated) or at the **system level**. Nothing runs automatically.

* **How it works**:

  * Use an **inline command** like:

    ```markdown
    @run(bash: echo "Hello World")
    ```
  * Or use a **fenced run block**:

    ````markdown
    ```run@python
    print("Hello from Python")
    ```
    ````

* **Immediate feedback**: The results are shown **right inside the preview**, so you see the output without leaving your document.
Perfect for **tutorials, documentation, and examples** where readers can try things out directly.

---

## Structure & Layout

* **Blockquotes**: Start a line with `>` to create a quote block.
* **Horizontal rule**: Use `---` for a visual divider.
* **Tables**: Create tables with headers, rows, and alignment. Great for structured data.
* **Task lists**: Write checkboxes with `- [ ]` or `- [x]` to track tasks.
* **Ordered lists**: Use `1.` `2.` `3.` for numbered steps.
* **Definition lists**: Use `term : explanation` for dictionary-style lists.

---

## Links, Media & Embeds

* **URLs**: Just paste a web link (http/https/www).
* **Email links**: Use `mailto:` to link to an email address.
* **Local files**: Link to files inside your project.
* **Images**: Insert images with a link to their file.
* **YouTube**: Paste a YouTube URL for an embedded video.
* **Inline links**: `[Text](url)` for a standard link.
* **Block images / YouTube**: Show media as its own block, with captions if needed.

---

## Table of Contents, Bookmarks & Page Navigation

Marco Markdown adds **powerful navigation features** to your Markdown documents, making them feel like interactive books or manuals.


### Table of Contents (TOC)

Automatically generate a **Table of Contents** from your headings.

* Shows headings in a hierarchy (H1–H6)
* Can only be **local** (current page) and can span multiple linked pages
* Optional depth: limit how many heading levels appear
* Collapsible sections supported

**Example usage:**

```markdown
[toc]          # Local TOC
[toc=2]        # Limit depth to H1-H2
[toc=2](@Page) # Include linked pages in the TOC
```

TOC entries are linked to headings in your document, providing **clickable navigation** in previews or exports.

### Bookmarks

Bookmarks link to a **specific file and line number**, making it easy to jump between sections or highlight important points.

```markdown
[Bookmark: Project Overview](./project_overview.md=254)
```

* Links are **local to your project**
* Line numbers are optional; Marco fills them automatically if missing
* Preview shows a **colored snippet or icon**, like a real bookmark

Use bookmarks for **quick navigation or highlighting sections** across files.

### Page Tags & References

#### In-Document Page Splits

Page tags control **page layout and splitting**, useful for export to PDF or structured previews.

```markdown
[Page=A4]   # Start a page in A4 layout
[Page=US]   # Start a page in US Letter layout
[Page=]     # Auto-numbered page
[Page=65]   # Explicit page number
```

* Red vertical lines in the editor show splits
* Marco automatically tracks page numbers
* Works together with TOC and bookmarks

#### Cross-Document Navigation

Link forward or backward across Markdown files using `[@Page]`:

```markdown
[@Page](./chapter_03.md)   # Next document
[@Page](./chapter_02.md)   # Previous document
```

* Place at the **bottom of the current file** (next page) or **top of target file** (previous page)
* Marco generates arrows in previews:

```
← Previous Page 62 | 63 | Next Page 64 →
```

* Only **local Markdown files** are supported
* Works seamlessly with TOC and bookmarks for a unified reading experience

### Unified Navigation Flow

Combine **page splits** (`[Page=]`) with **cross-document references** (`[@Page]`) to create a smooth, book-like flow:

| File            | Action / Marker            | Description                          |
| --------------- | -------------------------- | ------------------------------------ |
| `chapter_02.md` | `[Page=]`                  | Start a new page (in-document split) |
|                 | `[Page=]`                  | Last page of chapter                 |
|                 | `[@Page](./chapter_03.md)` | Next chapter link                    |
| `chapter_03.md` | `[@Page](./chapter_02.md)` | Previous chapter link                |
|                 | `[Page=]`                  | First in-document split              |
|                 | `[Page=]`                  | Last in-document split               |
|                 | `[@Page](./chapter_04.md)` | Next chapter link                    |


* **Arrows** guide readers between pages and chapters
* Works with TOC for multi-page or multi-file documents
* Keeps your Markdown structured, navigable, and export-ready

---

## Admonition Blocks

Admonitions create **highlighted blocks** for notes, tips, warnings, info, or custom icons — helping important information stand out.

### Predefined Types

Use one of the built-in types: `note`, `tip`, `warning`, `danger`, `info`.

```markdown
:::note
Some **content** with Markdown `syntax`.
:::
```

With an optional title:

```markdown
:::note[Optional Title]
Some **content** with Markdown formatting.
:::
```

* Content supports full Markdown
* Title is plain Markdown (no emoji/icons allowed)
* The block ends with `:::`


### Custom Emoji / Icon

Use a custom emoji to visually represent the block type:

```markdown
:::[:smile:] Some **content** with Markdown `syntax`. :::
```

With an optional title:

```markdown
:::[:smile: Your Title]
Some **content** with Markdown formatting.
:::
```

* Emoji goes inside `[ ]` and acts as the block type
* Optional title comes after the emoji (Markdown allowed)
* Content continues until the closing `:::`

---

## Callouts / Mentions

Mentions let you **tag people and link to their public profiles**. The format is:

```
@username[platform](Optional Display Name)
```

* **@username** → The account name (**required**)
* **\[platform]** → The platform (**required**)
* **(Optional Display Name)** → A custom name to show instead of the username

---

### Examples

| Mention Syntax                        | Displayed As           | Link / Profile |
|--------------------------------------|----------------------|----------------|
| `@ranrar[github]`                     | @ranrar              | https://github.com/ranrar |
| `@ranrar[github](Kim Skov Rasmussen)` | Kim Skov Rasmussen   | https://github.com/ranrar |
| `@someone[twitter](Jane Doe)`         | Jane Doe             | https://twitter.com/someone |
| `@teammate[slack](Project Lead)`      | Project Lead         | Slack profile link |

---

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
