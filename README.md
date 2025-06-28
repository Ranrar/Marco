# Marco - Markdown Composer

A feature-rich markdown editor built with Rust and GTK4, supporting multiple languages and comprehensive markdown editing capabilities.

## Features

### Core Functionality
- **Split-pane view** with markdown source on the left and live preview on the right
- **Syntax highlighting** for markdown with GTK SourceView
- **Real-time preview** updates as you type
- **File operations** (New, Open, Save, Save As)

### Internationalization (i18n)
- **Multi-language support** with runtime language switching
- **Supported languages**: English, Spanish (Español), French (Français), German (Deutsch)
- **Complete UI translation** including menus, toolbars, dialogs, and status messages
- **YAML-based translation system** for easy extensibility
- **No restart required** - switch languages instantly via View → Language menu

### Markdown Editing Tools
- **Comprehensive toolbar** with formatting buttons
- **Full menu system** with Insert and Format menus
- **Basic syntax support**: Headers (H1-H6), Bold, Italic, Code, Lists, Blockquotes
- **Extended syntax support**: Strikethrough, Subscript, Superscript, Highlight, Code blocks, Tables, Horizontal rules, Task lists, Footnotes, Definition lists, Emoji
- **Smart insertions**: Links, Images, Custom tables with dialog
- **Advanced code support**: Fenced code blocks with syntax highlighting for 10+ programming languages
- **Task list varieties**: Custom number dialog, single open task, single closed task
- **Live statistics**: Word count, character count, cursor position

### User Interface
- **Modern GTK4 interface** with clean design
- **Responsive layout** with adjustable split-pane
- **Tooltip support** for all toolbar buttons (translated)
- **Status footer** with real-time document statistics
- **Error handling** with visual feedback

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

### Completed Features
- **Core Editor**: Split-pane markdown editing with live preview
- **Internationalization**: 4 languages (EN, ES, FR, DE) with runtime switching
- **Basic Markdown**: Headers, bold, italic, lists, blockquotes, links, images
- **Extended Markdown**: Strikethrough, highlight, subscript, superscript, tables, code blocks
- **Task Lists**: Custom number dialog, single open/closed tasks
- **Code Languages**: 10+ programming languages with syntax highlighting
- **Keyboard Shortcuts**: Comprehensive shortcut system with help dialog
- **File Operations**: New, open, save, save as with proper file handling
- **Modular Architecture**: Clean separation of concerns across multiple modules

### Partially Implemented
- **Format Detection**: `find_format_at_cursor()` function exists but not actively used
- **Source View**: `source_view` field available but not fully integrated
- **Extended Syntax**: Functions exist but may need text content (`insert_footnote()`, `insert_definition_list()`, etc.)

### Not Yet Implemented
- **Status Bar Formatting**: HTML-style format display with indentation
- **Context Menus**: Right-click formatting options
- **Preview Modes**: HTML source view, CSS theme selection
- **Custom CSS**: User-defined stylesheet loading
- **Emoji Picker**: Searchable emoji browser
- **Advanced Editor Features**: Live highlighting, detachable preview
- **Export Options**: PDF, DOCX, other format exports

## Roadmap

### Planned Features (TODO)

#### Status Bar Enhancements
- **Format detection at cursor**: Show active formatting in footer (HTML-style with indentation)
- **Smart format warnings**: Display alerts for malformed markdown syntax
- **Live format display**: Real-time indication of current text formatting context

#### Advanced Editor Features
- **Context menus**: Right-click formatting options based on cursor position
- **Live preview highlighting**: Show formatting directly in the editor as you type
- **Format detection function**: `find_format_at_cursor()` implementation for context awareness

#### Preview Window Improvements
- **Flexible preview modes**: Toggle between HTML preview and HTML source code view
- **Multiple CSS themes**: Select from built-in styles ("HTML", "GitHub", "Dark", etc.)
- **Custom CSS support**: Load and apply user-defined stylesheets
- **Detachable preview**: Open preview in separate window for dual-monitor workflows

#### Enhanced Markdown Features
- **Improved footnotes**: `insert_footnote()` with reference numbering and linking
- **Rich definition lists**: `insert_definition_list()` with term/definition pairs
- **Text highlighting**: `insert_highlight()` with ==highlighted text== syntax
- **Scientific notation**: Enhanced `insert_subscript()` and `insert_superscript()` functions
- **Emoji picker**: `insert_emoji()` with searchable emoji browser and categories

#### Task List Enhancements
- **Custom task dialogs**: Specify number of items with preview **(COMPLETED)**
- **Quick task insertion**: Single open/closed task options **(COMPLETED)**
- **Task management**: Edit existing task states, bulk operations

#### Code Block Improvements
- **Language auto-detection**: Suggest language based on code content
- **Code formatting**: Auto-indent and syntax validation
- **Language extensions**: Easy addition of new programming languages
- **Syntax themes**: Multiple color schemes for code highlighting

#### User Interface Enhancements
- **Editor themes**: Dark mode, light mode, custom color schemes
- **Font customization**: Size, family, and spacing options
- **Layout options**: Vertical split, horizontal split, preview-only modes
- **Workspace persistence**: Remember window size, split position, and preferences
- **Zen mode**: A peacefulness with no distractions

#### Export and Integration
- **Export formats**: PDF, HTML, DOCX, and other popular formats
- **Print support**: Direct printing with formatting preservation
- **Plugin system**: Extensions for custom markdown processors
- **Git integration**: Track changes, commit from editor

#### Performance and Reliability
- **Large file handling**: Optimize for documents with thousands of lines
- **Auto-save**: Configurable automatic saving with recovery options
- **Undo/redo system**: Enhanced history with branch management
- **Search and replace**: Advanced find/replace with regex support

#### Accessibility
- **Screen reader support**: Full accessibility compliance
- **High contrast themes**: Support for visual accessibility needs
- **Keyboard navigation**: Complete keyboard-only operation support

#### Cross-platform
- **Windows**: Full installer with Start Menu integration and desktop shortcut support
- **MacOS**: DMG installer with drag-and-drop app installation; notarized for security compliance
- **linux**: Linux: `.deb`, and `.rpm` packages available; supports installation via terminal or software center

## Contributing

### Translation Contributions
We welcome translations to additional languages! Please follow the translation format in existing files and submit a pull request.