# Markdown Variant Selection
You can enable one or more Markdown variants. Only compatible variants can be enabled together.

- Enabling **Marco** enables **all** variants.
- Disabling **Marco** restores your previous selection.
- At least one variant must always be selected.
- Disabling the last remaining variant has no effect.
- Enabling a variant filters out incompatible ones.
- Disabling a variant may allow others to reappear.
- **Marco** acts as a "Select All" toggle.

---

## Compatibility Matrix

| Variant         | Can be toggled with...                                     |
|----------------|------------------------------------------------------------|
| CommonMark      | GFM, Markdig, Marco                                        |
| GFM             | CommonMark, Markdig, Marco                                 |
| Pandoc          | Obsidian, Typora, Markdown Extra, Marco                    |
| Obsidian        | Pandoc, Typora, Markdown Extra, Marco                      |
| Typora          | Pandoc, Obsidian, Markdown Extra, Markdig, Marco           |
| Markdown Extra  | Pandoc, Obsidian, Typora, Marco                            |
| Markdig         | CommonMark, GFM, Typora, Marco                             |
| Marco           | **All variants**                                           |

---

## Feature Support Table

| Feature                              | CommonMark | GFM | Marco | Obsidian | Pandoc | Typora | Markdown Extra | Markdig |
| ------------------------------------ | ---------- | --- | ----- | -------- | ------ | ------ | -------------- | ------- |
| **Tables**                           | ❌          | ✅   | ✅     | ❌        | ✅      | ✅      | ✅              | ✅       |
| **Task Lists**                       | ❌          | ✅   | ✅     | ❌        | ❌      | ✅      | ❌              | ✅       |
| **Strikethrough**                    | ❌          | ✅   | ✅     | ❌        | ❌      | ✅      | ❌              | ✅       |
| **Frontmatter (YAML)**               | ❌          | ✅   | ✅     | ✅        | ✅      | ✅      | ✅              | ✅       |
| **Footnotes**                        | ❌          | ❌   | ✅     | ✅        | ✅      | ✅      | ✅              | ✅       |
| **Wiki Links** (`[[Page]]`)          | ❌          | ❌   | ✅     | ✅        | ❌      | ❌      | ❌              | ❌       |
| **Math / LaTeX** (`$x^2$`, `$$`)     | ❌          | ❌   | ✅     | ✅        | ✅      | ✅      | ❌              | ✅       |
| **Auto-links**                       | ✅          | ✅   | ✅     | ❌        | ✅      | ✅      | ✅              | ✅       |
| **Attribute Lists** (`{#id .class}`) | ❌          | ❌   | ✅     | ❌        | ✅      | ✅      | ✅              | ✅       |
| **Callouts** (`::: info`)            | ❌          | ❌   | 🧩    | ✅        | 🧩     | ✅      | ❌              | 🧩      |
| **Code Block IDs / Classes**         | ❌          | ❌   | 🧩    | ❌        | ✅      | ✅      | ✅              | ✅       |
| **@include(file.md)**                | ❌          | ❌   | 🧩    | ❌        | 🧩     | ❌      | ❌              | 🧩      |
| **@toc (Auto TOC)**                  | ❌          | ❌   | 🧩    | 🧩       | 🧩     | 🧩     | ❌              | 🧩      |
| **@lint (markdown spellcheck)**      | ❌          | ❌   | 🧩    | ❌        | ❌      | 🧩     | ❌              | ❌       |
| **@mail (mailto: + subject)**        | ❌          | ❌   | 🧩    | ❌        | ❌      | ❌      | ❌              | ❌       |
| **@if (conditional content)**        | ❌          | ❌   | 🧩    | ❌        | 🧩     | ❌      | ❌              | ❌       |
| **@run (terminal command)**          | ❌          | ❌   | 🧩    | ❌        | ❌      | ❌      | ❌              | ❌       |
| **Mermaid Diagrams**                 | ❌          | ❌   | 🧩    | ✅        | 🧩     | ✅      | ❌              | 🧩      |
| **MathJax (advanced math)**          | ❌          | ❌   | 🧩    | 🧩       | ✅      | ✅      | ❌              | ✅       |
| **KaTeX (fast math rendering)**      | ❌          | ❌   | 🧩    | 🧩       | ✅      | ✅      | ❌              | ✅       |
| **PlantUML (UML diagrams)**          | ❌          | ❌   | 🧩    | 🧩       | 🧩     | ❌      | ❌              | 🧩      |
| **Graphviz/Dot Graphs**              | ❌          | ❌   | 🧩    | 🧩       | ✅      | ❌      | ❌              | 🧩      |

✅ = Standard Support
🧩 = Extension/Plugin
❌ = Not Supported

## Feature Descriptions

| Name                | Description                                                                          |
| ------------------- | ------------------------------------------------------------------------------------ |
| **Tables**              | Pipe-based tables with header alignment and multi-line support.                          |
| **Task Lists**          | `[ ]`, `[x]` checkboxes for todo-style lists.                                            |
| **Strikethrough**       | Use `~~text~~` or `--text--` for crossed-out text.                                       |
| **Frontmatter (YAML)**  | Metadata at the top of the document using `---` and YAML syntax.                         |
| **Footnotes**           | Academic-style notes like `[^1]` and `[1]:` rendered at the bottom with reference links. |
| **Wiki Links**          | `[[Page Name]]` wiki-style links (e.g., Obsidian format).                                |
| **Math / LaTeX**        | Inline (`$...$`) and block (`$$...$$`) mathematical notation.                            |
| **Auto-links**          | Automatically link plain URLs or emails.                                                 |
| **Attribute Lists**     | Add attributes to elements (IDs, classes) via `{#id .class}`.                            |
| **Definition Lists**    | Term–definition style lists.                                                             |
| **Abbreviations**       | Define `*[HTML]: HyperText Markup Language` abbreviations.                               |
| **Highlighting**        | Use `==highlight==` for text highlighting.                                               |
| **Inline HTML**         | Embed raw HTML inside Markdown.                                                          |
| **TOC (@toc)**          | Auto-generate a Table of Contents from document headings.                                |
| **Underline**           | `_text_` or HTML `<u>` to underline text.                                                |
| **Superscript**         | `x^2` → render `2` as superscript.                                                       |
| **Subscript**           | `H~2~O` → render `2` as subscript.                                                       |
| **Callouts**            | Blocks like `::: info`, `::: warning`, or `> [!NOTE]` with icons and styled borders.     |
| **Diagrams**            | Render diagrams using **Mermaid**, **PlantUML**, **Graphviz/DOT** syntax.                |
| **Emoji Shortcodes**    | `:smile:` style emoji converted to unicode emoji.                                        |
| **Inline Comments**     | Special syntax to hide/show developer comments.                                          |
| **Custom Containers**   | Define custom block styles and layouts with extended syntax.                             |
| **Line Breaks (Hard)**  | Treat single line breaks as `<br>` (like GFM).                                           |
| **Escaped Pipes**       | Use `\|` in tables to allow inline pipes without breaking layout.                        |
| **Smart Typography**    | Converts straight quotes and dashes to smart/curly ones.                                 |
| **HTML Entity Support** | Parses HTML named/decimal entities like `&copy;`, `&#169;`.                              |
| **HTML Sanitization**   | Remove unsafe HTML tags/attributes (sandboxed mode).                                     |
| **@include(file.md)** | Modularize documents by including external `.md` files.                                  |
| **@toc**              | Insert Table of Contents automatically at the specified position.                        |
| **@lint**             | Markdown linting + spellcheck with red underlines for incorrect or broken formatting.    |
| **@mail**             | Render email links with subject/body preset for one-click mail composition.              |
| **@if**               | Conditional rendering for multi-language or flavor variants (e.g., `CommonMark`, `GFM`). |
| **@run**              | Execute a shell or terminal command (safe-mode sandboxed by default).                    |
| **MathJax**           | Advanced LaTeX math rendering engine (for complex formulas).                             |
| **KaTeX**             | Lightweight fast LaTeX renderer (subset of MathJax).                                     |
| **PlantUML**          | Create sequence/class/activity diagrams via `@startuml` syntax.                          |
| **Graphviz/Dot**      | Generate graphs with `dot` syntax (`digraph G {}`), rendered as SVG.                     |