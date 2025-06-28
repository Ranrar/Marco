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
- **Extended syntax support**: Strikethrough, Code blocks, Tables, Horizontal rules
- **Smart insertions**: Links, Images, Custom tables with dialog
- **Live statistics**: Word count, character count, cursor position

### User Interface
- **Modern GTK4 interface** with clean design
- **Responsive layout** with adjustable split-pane
- **Tooltip support** for all toolbar buttons (translated)
- **Status footer** with real-time document statistics
- **Error handling** with visual feedback

## Prerequisites

Before building this application, you need to install Rust and GTK4 development libraries.

### Install Rust

First, install Rust using rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Install GTK4 Development Libraries

### Ubuntu/Debian:
```bash
sudo apt update
sudo apt install libgtk-4-dev libgtksourceview-5-dev build-essential
```

### Fedora:
```bash
sudo dnf install gtk4-devel gtksourceview5-devel
```

### Arch Linux:
```bash
sudo pacman -S gtk4 gtksourceview5
```

## Building and Running

1. Clone or navigate to this directory
2. Build and run the application:

```bash
cargo run
```

Or build for release:

```bash
cargo build --release
```

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
- **Lists**: Bullet lists (•) and numbered lists (1.)
- **Blockquotes**: > quoted text
- **Links**: [text](url) and ![alt](image.png)
- **Tables**: Custom table dialog with row/column selection
- **Code blocks**: ```language fenced code blocks
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
- `src/localization.rs` - Translation system and language management
- `src/markdown_basic.rs` - Basic markdown parsing and syntax definitions
- `src/markdown_extended.rs` - Extended markdown features

### Configuration
- `Cargo.toml` - Project dependencies and metadata
- `build.rs` - Build script for GTK resources
- `locales/` - Translation files for all supported languages

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
| Bold Text | Select text + Bold button | Insert → Bold |
| Italic Text | Select text + Italic button | Insert → Italic |
| Insert Heading | H1/H2/H3 buttons | Insert → Heading |
| Insert Link | Link button | Insert → Link |
| Insert Table | Table dialog | Format → Table |

## Contributing

### Development Setup
1. Install prerequisites (Rust + GTK4 development libraries)
2. Clone the repository
3. Run `cargo build` to compile
4. Run `cargo run` to start the application

### Adding Features
- Follow Rust best practices and GTK4 patterns
- Ensure all new UI text is translatable using `localization::tr()`
- Add translations to all language files in `locales/`
- Test with different languages to ensure proper UI layout

### Translation Contributions
We welcome translations to additional languages! Please follow the translation format in existing files and submit a pull request.

## License

This project is open source. Please refer to the LICENSE file for details.

## Technical Notes

### GTK4 Integration
- Uses modern GTK4 widgets and patterns
- Implements proper widget lifecycle management
- Follows GTK4 best practices for UI updates and event handling

### Performance
- Efficient real-time markdown parsing with pulldown-cmark
- Minimal UI updates using GTK's signal system
- Low memory footprint with careful widget management

### Architecture
- Modular design with separated concerns
- Translation system designed for easy extensibility
- Clean separation between UI, markdown processing, and file operations
