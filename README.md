# Marco - Markdown Composer

A feature-rich markdown editor built with Rust and GTK4, supporting multiple languages and comprehensive markdown editing capabilities.

## Features

### Core Functionality
- **Split-pane view** with markdown source on the left and live preview on the right
- **Syntax highlighting** for markdown with GTK SourceView and theme-aware color schemes
- **Real-time preview** updates as you type with debounced rendering
- **File operations** (New, Open, Save, Save As) with proper file handling
- **Theme system** with light/dark mode support, OS detection, and manual switching
- **Preview modes** with HTML rendering and HTML source code view switching

### Internationalization (i18n)
- **Multi-language support** with runtime language switching
- **Supported languages**: English, Spanish (Español), French (Français), German (Deutsch)
- **Complete UI translation** including menus, toolbars, dialogs, and status messages
- **YAML-based translation system** for easy extensibility
- **No restart required** - switch languages instantly via View → Language menu

### Markdown Editing Tools
- **Comprehensive toolbar** with formatting buttons and active state indicators
- **Full menu system** with Insert and Format menus plus context menus
- **Basic syntax support**: Headers (H1-H6), Bold, Italic, Code, Lists, Blockquotes
- **Extended syntax support**: Strikethrough, Subscript, Superscript, Highlight, Code blocks, Tables, Horizontal rules, Task lists, Footnotes, Definition lists
- **Smart insertions**: Links with dialog, Images with file chooser, Custom tables with row/column selection
- **Advanced code support**: Fenced code blocks with syntax highlighting for 10+ programming languages
- **Task list varieties**: Basic lists, custom number dialog with preview, single open/closed task options
- **Interactive dialogs**: Table creation, task list generation, definition list builder, emoji picker
- **Live statistics**: Word count, character count, cursor position with real-time updates
- **Format detection**: Cursor-based formatting detection with toolbar state updates

### User Interface
- **Modern GTK4 interface** with clean design and responsive layout
- **Light and dark theme support** with OS theme detection and manual switching
- **Adjustable split-pane** with minimum size constraints and 50/50 default
- **Comprehensive toolbar** with active state indicators and formatting buttons
- **Full menu system** with Insert, Format, View, and Help menus
- **Context menus** with right-click formatting options and nested submenus
- **Status footer** with real-time document statistics and cursor position
- **Tooltip support** for all interactive elements (fully translated)
- **Error handling** with visual feedback and user-friendly messages
- **Theme integration** for both preview pane and source editor with unified CSS

## Usage

### Basic Operations
- **New** (Ctrl+N): Clear the editor to start a new document
- **Open** (Ctrl+O): Open an existing markdown file
- **Save** (Ctrl+S): Save the current document
- **Save As** (Ctrl+Shift+S): Save with a new filename

### Language Switching
- Navigate to **View → Language** in the menu
- Select from English, Spanish, French, or German
- All UI elements update immediately

### Markdown Formatting
Use the toolbar buttons or menu items to insert:
- **Headers**: H1, H2, H3 buttons or Insert → Headings
- **Text formatting**: Bold (**text**), Italic (*text*), Inline code (`code`)
- **Advanced text**: Strikethrough (~~text~~), Highlight (==text==), Subscript (H~2~O), Superscript (x^2^)
- **Lists**: Bullet lists (•), numbered lists (1.), and task lists with checkboxes
- **Task lists**: Custom number dialog, single open task (- [ ]), single closed task (- [x])
- **Blockquotes**: > quoted text
- **Links**: [text](url) and ![alt](image.png)
- **Tables**: Custom table dialog with row/column selection
- **Code blocks**: ```language fenced code blocks with syntax highlighting
- **Special elements**: Footnotes[^1], definition lists, emoji 🎉
- **Horizontal rules**: --- separators

### Live Preview
- Type markdown in the left pane
- See formatted preview in the right pane
- Statistics displayed in footer: word count, character count, cursor position

## Dependencies

### Core Dependencies
- `gtk4` - GTK4 bindings for Rust (modern UI toolkit)
- `sourceview5` - GTK SourceView for syntax highlighting
- `pulldown-cmark` - Fast CommonMark-compliant markdown parser
- `glib` - GLib bindings for GTK integration

### Internationalization
- `serde` - Serialization framework for YAML parsing
- `serde_yaml` - YAML support for translation files

## Translation System

Marco uses a custom YAML-based translation system located in the `locales/` directory:

```
locales/
├── en/main.yml    # English translations
├── es/main.yml    # Spanish translations  
├── fr/main.yml    # French translations
└── de/main.yml    # German translations
```

### Adding New Languages

1. Create a new directory: `locales/[language_code]/`
2. Copy `en/main.yml` to your new directory
3. Translate all text values (keep keys unchanged)
4. Add the language to `get_available_locales()` in `src/localization.rs`
5. Add a menu action in `create_menu_actions()` in `src/main.rs`

### Translation Format

```yaml
en:
  app:
    title: "Marco - Markdown Composer"
  menu:
    file: "File"
    new: "New"
  # ... more translations
```

## Project Structure

### Source Code
- `src/main.rs` - Application entry point, UI setup, and i18n integration
- `src/editor.rs` - Main editor widget with split-pane layout and markdown tools
- `src/menu.rs` - Menu system, actions, and dialog management
- `src/toolbar.rs` - Toolbar buttons and formatting controls
- `src/footer.rs` - Status bar with statistics and format detection
- `src/preview.rs` - Live preview rendering and HTML generation
- `src/syntax_basic.rs` - Basic markdown syntax highlighting and parsing
- `src/syntax_extended.rs` - Extended markdown features (tables, tasks, etc.)
- `src/code_languages.rs` - Programming language definitions and syntax highlighting
- `src/localization.rs` - Translation system and language management

### Configuration
- `Cargo.toml` - Project dependencies and metadata
- `build.rs` - Build script for GTK resources
- `locales/` - Translation files for all supported languages

### Documentation
- `ADDING_LANGUAGES.md` - Guide for extending programming language support
- `LANGUAGE_REFERENCE.md` - User reference for supported languages
- `doc/` - Additional documentation and guides

### Build Output
- `target/debug/marco` - Debug executable
- `target/release/marco` - Optimized release executable

## Screenshots

### English Interface
- Main window with split-pane markdown editing
- Comprehensive toolbar with formatting buttons
- Full menu system with File, Edit, Insert, Format, View, and Help menus

### Multi-language Support
- Runtime language switching via View → Language menu
- All UI elements update immediately without restart
- Supported languages: English, Spanish, French, German

## Keyboard Shortcuts

| Action | Shortcut | Menu Location |
|--------|----------|---------------|
| New Document | Ctrl+N | File → New |
| Open File | Ctrl+O | File → Open |
| Save | Ctrl+S | File → Save |
| Save As | Ctrl+Shift+S | File → Save As |
| Bold Text | Ctrl+B | Insert → Bold |
| Italic Text | Ctrl+I | Insert → Italic |
| Underline Text | Ctrl+U | Insert → Underline |
| Inline Code | Ctrl+` | Insert → Inline Code |
| Insert Link | Ctrl+K | Insert → Link |
| Heading 1 | Ctrl+1 | Insert → Heading 1 |
| Heading 2 | Ctrl+2 | Insert → Heading 2 |
| Heading 3 | Ctrl+3 | Insert → Heading 3 |
| Heading 4 | Ctrl+4 | Insert → Heading 4 |
| Heading 5 | Ctrl+5 | Insert → Heading 5 |
| Heading 6 | Ctrl+6 | Insert → Heading 6 |
| Bullet List | Ctrl+Shift+8 | Insert → Unordered List |
| Numbered List | Ctrl+Shift+7 | Insert → Ordered List |
| Blockquote | Ctrl+Shift+. | Insert → Blockquote |

## Current Implementation Status

### ✅ Completed Features

#### Core Functionality
- [x] **Split-pane editor** with markdown source and live preview
- [x] **Real-time preview** updates as you type with debounced updates
- [x] **Syntax highlighting** for markdown with GTK SourceView
- [x] **File operations** (New, Open, Save, Save As) with proper file handling
- [x] **Responsive layout** with adjustable split-pane and minimum sizes
- [x] **Modern GTK4 interface** with clean design

#### Theme System
- [x] **Light and dark mode support** for both preview and editor
- [x] **OS theme detection** with automatic system theme following
- [x] **Manual theme switching** via View → Theme menu (System/Light/Dark)
- [x] **Unified CSS system** with single file using CSS variables and media queries
- [x] **Source editor theming** with automatic style scheme switching
- [x] **Instant theme updates** without restart required

#### Internationalization (i18n)
- [x] **Multi-language support** with runtime language switching (4 languages)
- [x] **Complete UI translation** including menus, toolbars, dialogs, and status messages
- [x] **YAML-based translation system** for easy extensibility
- [x] **No restart required** for language changes

#### Markdown Editing
- [x] **Basic syntax support**: Headers (H1-H6), Bold, Italic, Code, Lists, Blockquotes
- [x] **Extended syntax support**: Strikethrough, Highlight, Subscript, Superscript, Tables, Code blocks
- [x] **Advanced text formatting**: `insert_highlight()`, `insert_subscript()`, `insert_superscript()`
- [x] **Smart insertions**: Links with dialog, Images with file chooser
- [x] **Horizontal rules** and line separators
- [x] **Footnotes** with syntax highlighting and reference support
- [x] **Definition lists** with custom number dialog and preview

#### Task Lists
- [x] **Basic task list insertion** with checkboxes
- [x] **Custom task dialogs** with specified number of items and live preview
- [x] **Single task insertion** (open and closed task options)
- [x] **Task list syntax highlighting** in source editor

#### Code Support
- [x] **Fenced code blocks** with syntax highlighting
- [x] **10+ programming languages** with color schemes
- [x] **Language auto-completion** and suggestions
- [x] **Code block insertion dialog** with language selection and sample code

#### Tables
- [x] **Custom table creation dialog** with row/column selection
- [x] **Input validation** with visual error feedback
- [x] **Table syntax highlighting** in source editor

#### User Interface
- [x] **Comprehensive toolbar** with formatting buttons and active state indicators
- [x] **Full menu system** with Insert, Format, View, and Help menus
- [x] **Context menus** with right-click formatting options
- [x] **Tooltip support** for all buttons (fully translated)
- [x] **Status footer** with real-time statistics (word count, character count, cursor position)
- [x] **Error handling** with visual feedback and CSS styling

#### Preview Features
- [x] **HTML preview mode** with live markdown rendering
- [x] **HTML source code view** for debugging
- [x] **Preview mode switching** via View → Preview Mode menu
- [x] **CSS styling** with proper markdown formatting

#### Advanced Features
- [x] **Emoji picker** with searchable categories and insertion
- [x] **Keyboard shortcuts** for all major functions (20+ shortcuts)
- [x] **Format detection** at cursor with toolbar button state updates
- [x] **Modular architecture** with clean separation of concerns

### 🔶 Partially Implemented

#### Format Detection
- [x] **Format detection functions** exist (`find_format_at_cursor()`, `is_cursor_in_format()`)
- [x] **Toolbar state updates** based on cursor position
- [ ] **Status bar format display** with HTML-style indentation and context

#### Extended Syntax
- [x] **All insertion functions implemented** (`insert_footnote()`, `insert_definition_list()`, etc.)
- [x] **Syntax highlighting** for extended features
- [ ] **Advanced footnote management** with reference numbering and linking

### ❌ Not Yet Implemented

#### Advanced Editor Features
- [ ] **Live preview highlighting** in source editor as you type
- [ ] **Detachable preview window** for dual-monitor workflows
- [ ] **Font customization** (size, family, spacing options)
- [ ] **Layout options** (vertical split, horizontal split, preview-only modes)

#### CSS and Styling
- [ ] **Custom CSS loading** and user-defined stylesheets
- [ ] **Multiple built-in CSS themes** (GitHub, Material, etc.)
- [ ] **CSS theme selection** in preview settings

#### Export and Integration
- [ ] **Export formats** (PDF, HTML, DOCX, and other formats)
- [ ] **Print support** with formatting preservation
- [ ] **Plugin system** for custom markdown processors
- [ ] **Git integration** for change tracking

#### Performance and Advanced Features
- [ ] **Large file optimization** for documents with thousands of lines
- [ ] **Auto-save** with configurable intervals and recovery
- [ ] **Enhanced undo/redo** system with branch management
- [ ] **Search and replace** with regex support and find/replace dialog
- [ ] **Workspace persistence** (window size, split position, preferences)

#### Accessibility and Polish
- [ ] **Screen reader support** and full accessibility compliance
- [ ] **High contrast themes** for visual accessibility
- [ ] **Zen mode** distraction-free editing
- [ ] **Status bar format warnings** for malformed markdown syntax

## Roadmap

### Planned Features (TODO)

#### Advanced Editor Enhancements
- **Live preview highlighting**: Show formatting directly in the editor as you type
- **Font customization**: Size, family, and spacing options with user preferences
- **Layout options**: Vertical split, horizontal split, preview-only modes
- **Workspace persistence**: Remember window size, split position, and user preferences
- **Zen mode**: Distraction-free editing environment
- **Enhanced undo/redo**: Advanced history with branch management and visual timeline

#### Preview and Styling Improvements
- **Custom CSS support**: Load and apply user-defined stylesheets with live preview
- **Multiple built-in themes**: GitHub, Material, Solarized, and other popular styles
- **CSS theme hot-swapping**: Change preview styles without restart
- **Detachable preview**: Open preview in separate window for dual-monitor workflows
- **Print support**: Direct printing with formatting preservation and page setup

#### Advanced Markdown Features
- **Enhanced footnote management**: Reference numbering, linking, and validation
- **Status bar format display**: HTML-style format indication with indentation levels
- **Smart format warnings**: Display alerts for malformed markdown syntax
- **Live format context**: Real-time indication of current text formatting

#### Search and Navigation
- **Advanced search and replace**: Full-featured find/replace with regex support
- **Document outline**: Navigable heading structure in sidebar
- **Quick navigation**: Jump to headings, links, and other document elements
- **Global search**: Search across multiple files and documents

#### Export and Integration
- **Export formats**: PDF, HTML, DOCX, LaTeX, and other popular formats
- **Batch processing**: Convert multiple files with customizable templates
- **Plugin system**: Extensions for custom markdown processors and converters
- **Git integration**: Track changes, commit from editor, diff visualization
- **Document linking**: Cross-references between multiple markdown files

#### Performance and Reliability
- **Large file handling**: Optimize for documents with thousands of lines and images
- **Auto-save**: Configurable automatic saving with recovery options and backup
- **Memory optimization**: Efficient handling of large documents and multiple files
- **Background processing**: Non-blocking operations for exports and file operations

#### Accessibility and Usability
- **Screen reader support**: Full accessibility compliance with ARIA labels
- **High contrast themes**: Support for visual accessibility needs
- **Keyboard navigation**: Complete keyboard-only operation support
- **Voice commands**: Basic voice input for common formatting operations
- **User onboarding**: Interactive tutorial and help system

#### Cross-platform and Distribution
- **Windows**: Full installer with Start Menu integration and desktop shortcuts
- **MacOS**: DMG installer with drag-and-drop installation and notarization
- **Linux**: `.deb`, `.rpm`, and `.AppImage` packages with software center support
- **Portable versions**: Self-contained executables for USB drives
- **Auto-updates**: Seamless update system with changelog display

## Contributing

### Translation Contributions
We welcome translations to additional languages! Please follow the translation format in existing files and submit a pull request.