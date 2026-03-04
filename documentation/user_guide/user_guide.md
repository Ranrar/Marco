<center>
<img src="Logo_marco_and_polo.png" alt="Marco" width="" height="">
</center>

---

#  User Guide

## Table of Contents
1. [Getting Started](#getting-started)
2. [Interface Overview](#interface-overview)
3. [File Operations](#file-operations)
4. [Text Editing](#text-editing)
5. [Markdown Formatting](#markdown-formatting)
   - [Headings & Heading IDs](#headings)
   - [Text Formatting](#text-formatting) — bold, italic, highlight, super/subscript, strikethrough
   - [Math](#math)
   - [Links, Images & Autolinks](#links-and-images)
   - [Tables (GFM & headerless)](#tables)
   - [Code Blocks](#code-blocks)
   - [Blockquotes](#blockquotes)
   - [Horizontal Rules](#horizontal-rules)
6. [Advanced Features](#advanced-features)
   - [Admonitions (GFM + custom)](#advanced-elements--dialogs)
   - [Tab Blocks](#advanced-elements--dialogs)
   - [Slide Decks](#advanced-elements--dialogs)
   - [Footnotes (GFM + inline)](#advanced-elements--dialogs)
   - [Task Lists](#advanced-elements--dialogs)
   - [Definition Lists](#advanced-elements--dialogs)
   - [Platform Mentions](#advanced-elements--dialogs)
7. [View Options](#view-options)
8. [Settings & Preferences](#settings--preferences)
9. [Keyboard Shortcuts](#keyboard-shortcuts)
10. [Tips & Tricks](#tips--tricks)
11. [Math Cheatsheet](#math-cheatsheet)

## Getting Started

Welcome to **Marco**, a modern and powerful Markdown editor designed for writers, developers, and content creators. Marco provides an intuitive interface with live preview, syntax highlighting, and advanced formatting capabilities.

### First Launch
On the very first launch, Marco displays a **Welcome screen** that lets you:
- Choose your preferred UI language
- Review telemetry information

After completing the welcome screen, you'll see:
- A clean editing interface with syntax highlighting
- A toolbar with commonly used formatting options
- A status bar showing document statistics
- Side-by-side or single-pane view options

### Latest Features
Marco now includes several advanced features:
- **Smart Code Block Dialog** - Search through 100+ programming languages with aliases
- **Enhanced Theme System** - Automatic light/dark mode detection with 5 built-in CSS themes
- **Professional Dialogs** - Modal interfaces with real-time validation and preview
- **New inline formatting** - Highlight (`==`), superscript (`^`), subscript (`~`), dash strikethrough (`--`)
- **Tab blocks and slide decks** - Interactive content containers in Markdown
- **GFM extensions** - Autolink literals, admonitions, task lists (SVG icons), footnotes
- **Platform mentions** - `@user[github]` profile links
- **Extended heading IDs** - `## Title {#id}` with anchor links

## Interface Overview

### Main Components

#### Menu Bar
- **File**: New, Open, Save, Save As, Recent Files, Quit
- **Edit**: Undo, Redo, Cut, Copy, Paste, Find, Replace
- **Insert**: Headings, Lists, Links, Images, Code Blocks
- **Format**: Text styling, Code formatting, Tables
- **Advanced**: Special formatting, Text transformations
- **View**: Themes, View modes, Preferences
- **Help**: User guide, Shortcuts, About

#### Toolbar
Quick access buttons for:
- **Headings dropdown**: H1-H6 heading levels
- **Text formatting**: Bold (𝐁), Italic (𝐼), Code ({}), Strikethrough (S̶)
- **Lists**: Bullet lists (•), Numbered lists (1.), Blockquotes (❝)
- **Insert elements**: Links (🔗), Images (🖼), Horizontal rules (—)

#### Status Bar
Shows real-time information:
- Word count
- Character count
- Current cursor position (line and column)
- Document status

## File Operations

### Creating New Documents
- **Menu**: File → New (`Ctrl+N`)
- **Action**: Creates a blank document ready for editing

### Opening Files
- **Menu**: File → Open (`Ctrl+O`)
- **Action**: Browse and open existing Markdown files
- **Supported formats**: `.md`, `.markdown`, `.txt`

### Saving Documents
- **Save**: File → Save (`Ctrl+S`)
- **Save As**: File → Save As (`Ctrl+Shift+S`)
- **Auto-save**: Marco automatically tracks changes and prompts before closing unsaved documents

### Recent Files
Access recently opened documents through File → Recent Files for quick editing.

## Text Editing

### Basic Operations
- **Undo**: `Ctrl+Z` or Edit → Undo
- **Redo**: `Ctrl+Shift+Z` or Edit → Redo
- **Cut**: `Ctrl+X` or Edit → Cut
- **Copy**: `Ctrl+C` or Edit → Copy
- **Paste**: `Ctrl+V` or Edit → Paste

### Find and Replace
- **Search & Replace**: `Ctrl+F` or Edit → Search & Replace
  - Opens in a separate window for multitasking
  - Search for text in your document
  - Case-sensitive option available
  - Navigate through search results
  - Non-blocking workflow allows editing while searching
- **Replace**: Available in search window
  - Find and replace text
  - Replace individual instances or all occurrences
  - Smart replacement preserves formatting context

### Dialog Interface

Marco uses **modal dialogs** for advanced features that require user input. These dialogs provide a professional, consistent experience:

#### Dialog Behavior
- **Modal Windows**: All dialogs open as modal overlays attached to the main window
- **Focus Management**: You must interact with the dialog before returning to the editor
- **Consistent Design**: All dialogs feature the same header style with a simple X close button
- **Input Validation**: Real-time validation with clear error messages and user feedback
- **Preview Support**: Many dialogs include live preview of your changes

#### Types of Dialogs
- **Content Input**: Link insertion, image properties, code language selection
- **Text Styling**: Color selection, HTML formatting options, text alignment
- **Advanced Media**: Enhanced image options, YouTube embedding, custom links
- **System Information**: Keyboard shortcuts, about information, emoji picker

#### Tips for Dialog Use
- Use **Tab** to navigate between input fields
- Press **Enter** to confirm changes (equivalent to clicking OK)
- Press **Escape** to cancel and close the dialog
- All changes are previewed before being applied to your document

## Markdown Formatting

### Headings
Create headings using the toolbar dropdown or keyboard shortcuts:
- **H1**: `Ctrl+1` or `# Heading 1`
- **H2**: `Ctrl+2` or `## Heading 2`
- **H3**: `Ctrl+3` or `### Heading 3`
- **H4**: `Ctrl+4` or `#### Heading 4`
- **H5**: `Ctrl+5` or `##### Heading 5`
- **H6**: `Ctrl+6` or `###### Heading 6`

#### Custom Heading IDs
Append `{#your-id}` at the end of a heading line to give it an explicit HTML `id` attribute.
Marco renders an anchor icon (⚓) next to the heading that links to that ID.

```
## My Section {#my-section}
```

Renders as: `<h2 id="my-section">My Section <a href="#my-section" class="marco-heading-anchor">…</a></h2>`

**Rules:**
- No space is allowed between `{` and `#` — `{ #bad }` is left as literal text
- Trailing text after `{#id}` on the same line is also left as literal text
- Works on all heading levels (H1-H6)

### Text Formatting
- **Bold**: `Ctrl+B` or surround text with `**bold**`
- **Italic**: `Ctrl+I` or surround text with `*italic*`
- **Inline Code**: `Ctrl+` ` or surround with `` `code` ``
- **Strikethrough (GFM)**: Surround text with `~~text~~` (two tildes on each side)

  **Example:** `~~deleted~~` renders as struck-through text

- **Strikethrough (dash style)**: Marco also supports `--text--` (two hyphens on each side) as an alternative strikethrough

  **Example:** `--deleted--` renders identically to `~~deleted~~`

  > **Note:** Three or more hyphens are **not** treated as dash-strikethrough (they become a thematic break instead).
- **Highlight / Mark**: Surround text with `==text==` to produce a `<mark>` highlight

  **Example:** `==important==` renders as highlighted text

  > **Note:** Three or more `=` signs are **not** treated as a mark delimiter.

- **Superscript**: Surround text with `^text^` to raise it as superscript (`<sup>`)

  **Example:** `E = mc^2^` renders the 2 as a superscript

  > **Note:** `^^` (double caret) is reserved and will not parse as superscript.

- **Subscript**: Surround text with `~text~` to lower it as subscript (`<sub>`)

  **Example:** `H~2~O` renders the 2 as a subscript

  > **Note:** `~~text~~` takes priority as GFM strikethrough — a single `~` on each side is required for subscript.

- **Subscript (arrow style)**: An alternative subscript delimiter using `˅text˅` (modifier letter down arrowhead, U+02C5)

  **Example:** `H˅2˅O` is equivalent to `H~2~O`

  > **Note:** `˅˅` (double arrow) is reserved and will not parse as subscript. This form is useful when `~` conflicts with other syntax.

### Lists
- **Bullet Lists**: Click toolbar button or start line with `-` or `*`
- **Numbered Lists**: Click toolbar button or start line with `1.`
- **Nested Lists**: Indent with spaces or tabs

### Links and Images
- **Links**: `Ctrl+K` or use format `[text](URL)`
- **Images**: Use toolbar or format `![alt text](image_URL)`

#### GFM Autolink Literals
Marco automatically converts bare URLs and email addresses into clickable links without needing `[text](url)` syntax:

| Input | What it becomes |
|-------|----------------|
| `www.example.com` | Link to `http://www.example.com` |
| `https://example.com/path` | Link to that URL |
| `user@example.com` | `mailto:` link |

> **Note:** Trailing punctuation (`.`, `,`, `)`) is not included in the detected URL.

### Code Blocks
- **Inline Code**: Use backticks `` `code` ``
- **Fenced Code Blocks**: Use the smart search dialog or type manually
  ````markdown
  ```python
  def hello():
      print("Hello, World!")
  ```
  ````

#### Smart Language Selection
Marco now features an advanced language picker for fenced code blocks:
- **Access**: Format → Fenced code block... (`Ctrl+Shift+C`)
- **Smart Search**: Type to search among 100+ supported programming languages
- **Alias Support**: Use shortcuts like "js" for JavaScript, "py" for Python
- **Popular Languages**: Shows commonly used languages first
- **Real-time Filtering**: Instant results as you type

**Supported Languages Include**: Rust, JavaScript, TypeScript, Python, Java, C++, C#, Go, PHP, Ruby, HTML, CSS, SQL, Bash, and many more!

### Math

Marco renders math using **KaTeX**. Both inline and display (block) forms are supported.

- **Inline math**: Wrap an expression in single `$...$`

  **Example:** `$E = mc^2$` renders an inline equation

- **Display math**: Wrap an expression in double `$$...$$` (on its own line for block rendering)

  **Example:**
  ```
  $$
  \int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}
  $$
  ```

> **Tip:** See the [Math Cheatsheet](math_cheatsheet.md) for a full reference including arithmetic, algebra, calculus, probability, linear algebra, and Greek letters.

### Blockquotes
- Use toolbar button or start line with `> `
- Can be nested with multiple `>` characters

### Tables

#### Standard GFM Tables
Use pipe `|` characters to create tables. A separator row of `---` after the header is required.

```
| Name    | Age | City      |
|---------|-----|-----------|
| Alice   | 30  | Berlin    |
| Bob     | 25  | Paris     |
```

**Column alignment** is controlled by the separator row:
| Syntax | Alignment |
|--------|-----------|
| `---`  | Default (left) |
| `:---` | Left |
| `---:` | Right |
| `:---:` | Center |

#### Headerless Tables
Marco supports a delimiter-first pipe table — no header row required. Start the table directly with the separator row:

```
|---------|-----|-----------|
| Alice   | 30  | Berlin    |
| Bob     | 25  | Paris     |
```

This produces a `<table>` with no `<thead>` block.

### Horizontal Rules
- Use toolbar button or type `---` on its own line

## Advanced Features


### Text Styling (Requires Text Selection)
Access through Advanced menu when text is selected:

- **Underline Text**: Wraps selected text with `<u>` tags
- **Center Text**: Centers text using `<center>` tags
- **Colored Text**: Opens a color picker dialog for live color preview and applies HTML color styling
- **Indent Text**: Adds indentation to selected text

### Advanced Elements & Dialogs
- **Admonitions**: Callout boxes that draw attention to important content. Two styles are supported:

  **GFM style** — uses `> [!TYPE]` as the first line inside a blockquote:
  ```
  > [!NOTE]
  > Useful information that users should know.

  > [!TIP]
  > A helpful tip.

  > [!WARNING]
  > A warning about potential issues.

  > [!IMPORTANT]
  > Critical information.

  > [!CAUTION]
  > Caution about risks.
  ```

  **Custom / Marco-extended style** — use an emoji or icon with a custom title:
  ```
  > [:joy: Happy Header]
  > This admonition has a custom title and emoji.
  ```

  **Showcase:**
  > 💡**Tip:** You can use admonitions to highlight important information!

- **Emoji Picker**: Insert emoji anywhere using the native GTK4 emoji picker (Edit → Emoji, or shortcut `Ctrl+.`). Also supports emoji shortcodes like `:smile:`.

  **Showcase:**
  - Use the picker or type `:rocket:` → 🚀

- **Platform Mentions**: Link to a user profile on a supported platform using `@username[platform]`. An optional display name can be provided in parentheses.

  **Syntax:**
  | Format | Output |
  |--------|--------|
  | `@ranrar[github]` | Link to `https://github.com/ranrar` with label `ranrar` |
  | `@ranrar[github](Kim)` | Same link but label is `Kim` |
  | `@ranrar[gitlab]` | Link to GitLab profile |
  | `@ranrar[unknown]` | Rendered as a `<span>` (no clickable link) |

  **Supported platforms:** `github`, `gitlab`

- **Smart Code Block Language Search**: When inserting fenced code blocks, use the smart search dialog to filter among 100+ languages and aliases (e.g., "js" for JavaScript, "py" for Python). Popular languages are shown first, and fuzzy search is supported.

  **Showcase:**
  ```JavaScript
  // JavaScript code block using alias "js"
  console.log("Hello, world!");
  ```

- **Task Lists**: Create GFM-style checkable lists. Marco renders checkboxes as **SVG icons** (not plain HTML `<input>` checkboxes), styled consistently across all CSS themes.

  **Syntax:**
  ```
  - [ ] Unchecked item
  - [x] Checked item
  - [X] Also checked (capital X works too)
  ```

  Task checkboxes are also recognized **inline** — inside paragraphs or after hard line breaks:
  ```
  [ ] First task  
  [x] Second task
  ```

  **Showcase:**
  - [x] Write documentation
  - [ ] Add more features
  - [ ] Review pull requests

  > You can also insert a pre-built task list via **Advanced → Task List → Custom Task List**.

- **Custom Definition Lists**: Create definition lists with a custom number of term/definition pairs via Advanced → Definition List → Custom Definition List.

  **Showcase:**
  Term 1
  :   Definition for term 1

  Term 2
  :   Definition for term 2

- **Table of Contents**: Insert a dynamic TOC (Advanced → Table of Contents). Automatically generates links to all headings (H1-H4) in your document.

  **Showcase:**
  #### Table of Contents
  * [Getting Started](#getting-started)
  * [Advanced Features](#advanced-features)

- **Tab Blocks**: Organize content into interactive, no-JavaScript tabs directly in Markdown using the `:::tab` container.

  **Syntax:**
  ```
  :::tab
  @tab First Tab
  Content for the first tab.

  @tab Second Tab
  Content for the **second** tab.
  :::
  ```

  **Rules:**
  - The block opens with `:::tab` and closes with `:::`
  - Each panel starts with `@tab Title` on its own line
  - Markdown content (code blocks, headings, lists) inside each panel is fully parsed
  - `@tab` markers inside fenced code blocks are ignored (not treated as panel separators)
  - Up to 3 leading spaces are allowed before `:::tab`, `@tab`, and `:::`

- **Slide Decks**: Create interactive slideshows inside your Markdown document using `@slidestart` / `@slideend` blocks.

  **Syntax:**
  ```
  @slidestart
  # Slide One
  Content of slide one.

  ---

  ## Slide Two
  Content of slide two.

  --

  ### Vertical slide under Slide Two

  @slideend
  ```

  **Optional timer:** `@slidestart:t5` advances slides automatically every 5 seconds.

  **Slide separators:**
  | Separator | Meaning |
  |-----------|---------|
  | `---` | New horizontal slide |
  | `--` | New vertical slide (nested under the previous horizontal slide) |

  **Rules:**
  - The block opens with `@slidestart` (optionally `:tN` for an auto-advance timer in seconds) and closes with `@slideend`
  - Separators inside fenced code blocks are ignored
  - Up to 3 leading spaces are allowed before `@slidestart`, `@slideend`, and separators
  - Each slide's content is fully parsed Markdown

- **Footnotes**: Marco supports two footnote styles. Both produce numbered superscript references and a `<section class="footnotes">` block at the end of the document.

  **GFM reference-style** — define the footnote body elsewhere in the document:
  ```
  Here is a statement with a footnote.[^1]

  [^1]: This is the footnote content.
  ```

  Multi-line GFM footnotes are supported by indenting continuation lines with 4 spaces.

  **Inline footnotes** — embed the footnote content right where it appears:
  ```
  Here is a statement.^[The footnote content is written inline.]
  ```

  Inline footnote bodies support full inline markup (emphasis, bold, code, etc.).

  > **Note:** If a `[^ref]` is used but no matching `[^ref]:` definition exists, the marker is left as literal text and no footnotes section is added.

- **Spell Check & Linting**: Real-time spell check and Markdown linting highlight misspellings, unclosed tags, malformed tables, and other issues. Warnings are shown inline and in the status bar.
- **Theme Switching & Custom CSS**: Instantly switch between built-in CSS themes (Standard, Academic, GitHub, Minimal, Astro) and light/dark/system UI. You can also load a custom CSS file for preview styling.

  **Showcase:**
  - Switch between themes in View → Themes or Preferences
  - Example: Try the "Astro" theme for a cosmic look

- **About Dialog**: View app version, license, and credits via Help → About.

  **Showcase:**
  - Open Help → About to see version and license info

### HTML Integration
- **HTML Entities**: Insert special characters via Insert → HTML Entity (with preview)
- **Custom HTML**: Direct HTML input is supported in Markdown documents

## View Options

### View Modes
Choose your preferred editing experience:
- **Editor Only**: Focus on writing without distractions
- **Preview Only**: View formatted output
- **Split View**: Side-by-side editing and preview

### Themes
Customize the appearance:
- **Light Theme**: Clean, bright interface
- **Dark Theme**: Easy on the eyes for long writing sessions
- **System Theme**: Follows your system preferences

### CSS Themes
Apply different styling to your preview:
- **Standard**: Clean, professional styling
- **Academic**: Academic paper formatting with serif fonts
- **GitHub**: GitHub-style rendering
- **Minimal**: Clean, distraction-free appearance
- Themes affect how your Markdown renders in preview mode
- Automatic theme integration with light/dark mode detection

## Settings & Preferences

Access preferences through **View → Preferences** (`Ctrl+,`).

The settings dialog is organized into tabs. Labels and tooltips update live when you switch the UI language.

### Editor tab
- Font family and size for the source editor
- Tab/indent width

### Layout tab
- Default view mode (Editor Only, Preview Only, Split)
- Split pane ratio

### Appearance tab
- **UI Theme**: Light, Dark, or System — applies to the entire application interface
- **CSS Theme**: Styling applied to the preview pane (Standard, Academic, GitHub, Minimal, Astro)

### Language tab
- Switch the UI language (English, German, and more)
- The interface updates in-place without restarting

### Markdown tab
- Toggle Marco-specific Markdown extensions on/off

### Advanced tab
- Debug and diagnostic options

### Debug tab
- Internal logging and diagnostic information

## Keyboard Shortcuts

### File Operations
- `Ctrl+N` - New document
- `Ctrl+O` - Open file
- `Ctrl+S` - Save
- `Ctrl+Shift+S` - Save As
- `Ctrl+Q` - Quit application

### Editing
- `Ctrl+Z` - Undo
- `Ctrl+Shift+Z` - Redo
- `Ctrl+X` - Cut
- `Ctrl+C` - Copy
- `Ctrl+V` - Paste
- `Ctrl+F` - Search & Replace (opens in window)

### Formatting
- `Ctrl+B` - Bold
- `Ctrl+I` - Italic
- `Ctrl+K` - Insert Link
- `Ctrl+` ` - Inline Code
- `Ctrl+Shift+C` - Fenced Code Block (opens smart language picker)

### Lists and Structure
- `Ctrl+L` - Bullet List
- `Ctrl+Shift+L` - Numbered List
- `Ctrl+Q` - Blockquote

### Headings
- `Ctrl+1` through `Ctrl+6` - Insert heading levels

### Help
- `Ctrl+?` - Show keyboard shortcuts
- Access this guide through Help → Markdown Guide

## Tips & Tricks

### Productivity Tips
1. **Use keyboard shortcuts** for faster editing
2. **Split view** is great for real-time preview while writing
3. **Find and Replace** with case sensitivity for precise editing
4. **Recent Files** menu for quick access to your documents

### Formatting Best Practices
1. **Consistent heading hierarchy** improves document structure
2. **Use code blocks** for multi-line code instead of inline code
3. **Alt text for images** improves accessibility
4. **Proper link text** makes documents more readable

### Advanced Usage
1. **Select text first** before using Advanced text styling features
2. **HTML mixing** - You can use HTML tags within Markdown for advanced formatting
3. **Theme switching** to match your working environment
4. **Language switching** for international collaboration
5. **Smart code search** - Use aliases like "js", "py", "rs" in the fenced code dialog
6. **100+ programming languages** supported with syntax highlighting

### Troubleshooting
- **Unsaved changes warning**: Marco will prompt you before closing unsaved documents
- **Text selection required**: Some advanced features require text selection first
- **Syntax highlighting**: Automatic highlighting helps identify formatting issues
- **Preview updates**: Split view shows changes in real-time
- **White preview background**: If preview appears white, try switching CSS themes in View menu
- **Code language not found**: Use the smart search in fenced code dialog - try aliases like "js", "py", "rs"
- **Theme not loading**: Restart application if theme changes don't apply immediately

## Math Cheatsheet

Need a focused math reference for Markdown + KaTeX syntax?

- Open: [`math_cheatsheet.md`](math_cheatsheet.md)

It includes inline/display math, arithmetic, algebra, calculus, probability, linear algebra, Greek letters, and copy-ready formula snippets.

## Getting Help

- **User Guide**: Help → Markdown Guide (this document)
- **Keyboard Shortcuts**: Help → Shortcuts (`Ctrl+?`)
- **About**: Help → About (version and license information)

---

**Marco** is designed to make Markdown editing efficient and enjoyable. Whether you're writing documentation, blog posts, or technical content, Marco provides the tools you need for professional results.

*Happy writing! 📝*
