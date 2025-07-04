[![DevSkim](https://github.com/Ranrar/Marco/actions/workflows/devskim.yml/badge.svg)](https://github.com/Ranrar/Marco/actions/workflows/devskim.yml)  [![rust-clippy analyze](https://github.com/Ranrar/Marco/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/Ranrar/Marco/actions/workflows/rust-clippy.yml)
# Marco - Markdown Composer

A modern markdown editor built with Rust and GTK4. Features live preview, multiple languages, and advanced markdown support.

## Key Features

### What Makes Marco Special
- **Live Split-Pane Editing** - Write markdown on the left, see formatted preview on the right
- **4 Languages Built-In** - English, Spanish, French, German with instant switching
- **Advanced Markdown Support** - GitHub-style admonitions, tables, task lists, code blocks
- **Custom CSS Themes** - 4 built-in themes plus support for your own CSS files
- **Smart Persistence** - Remembers your preferences, no setup needed
- **Real-time Preview** with syntax highlighting and interactive dialogs
- **Context Menus & Toolbar** for quick access to all formatting options

## Advanced Features

### GitHub-Style Admonitions
```markdown
> [!NOTE]
> This is a note admonition

> [!WARNING]  
> This is a warning admonition
```

### Tables and Task Lists
- **Interactive Table Dialog** - Create tables with visual row/column selection
- **Task Lists** - Basic checkboxes, custom dialogs, single task insertion
- **Smart Code Blocks** - 100+ programming languages with intelligent search
- **Advanced Syntax Highlighting** - Powered by syntect library for high-quality code rendering

## Dependencies

### Core Dependencies
- `gtk4` - GTK4 bindings for Rust (modern UI toolkit)
- `sourceview5` - GTK SourceView for syntax highlighting
- `pulldown-cmark` - Fast CommonMark-compliant markdown parser
- `glib` - GLib bindings for GTK integration
- `syntect` - High-quality syntax highlighting for 100+ programming languages

### Additional Dependencies
- `serde` and `serde_json` - Serialization framework for settings and configuration
- `serde_yaml` - YAML support for translation files
- `lazy_static` - Global state management for settings system
- `regex` - Pattern matching for advanced syntax highlighting and markdown processing
- `webkit6` - WebKit integration for HTML preview rendering

## Recent Improvements

### Smart Code Block Dialog (Latest)
- **100+ Programming Languages** - Upgraded from regex-based to syntect-powered highlighting
- **Intelligent Search** - Smart autocomplete with alias support (js → JavaScript, py → Python)
- **GTK4 SearchEntry** - Modern search interface with real-time filtering
- **Popular Languages First** - Shows commonly used languages when search is empty
- **Professional UI** - Modal dialog with proper keyboard navigation

### Enhanced Theme System
- **Automatic CSS Loading** - Fixed white preview background issue
- **Theme Integration** - Proper light/dark mode detection and CSS injection
- **4 Built-in Themes** - Standard, Academic, GitHub, and Minimal styling
- **Robust Fallbacks** - Graceful handling of missing theme files

### Improved Architecture
- **Modular Design** - Clean separation between editor, themes, and syntax highlighting
- **Better Error Handling** - Comprehensive validation and user feedback
- **Performance Optimized** - Efficient CSS caching and theme switching

## Translation System

Marco uses YAML-based translations in `language/[language]/main.yml`. To add a new language:
1. Create `language/[code]/main.yml` 
2. Copy and translate `en/main.yml`
3. Add language to `src/language.rs` and menu system

**Current Languages**: English (en), German (de), Spanish (es), French (fr)

## Project Structure

### Key Files
- `src/main.rs` - Application entry point and UI setup
- `src/editor/` - Main editor module with split-pane layout and dialogs
- `src/menu/` - Menu system and actions
- `src/syntect_highlight.rs` - Advanced syntax highlighting with 100+ languages
- `src/theme.rs` - Theme management and CSS loading system
- `src/settings.rs` - Persistent settings with JSON storage
- `src/view_html.rs` - HTML preview rendering with theme integration
- `language/` - Translation files for all supported languages (YAML format)
- `themes/` - Built-in CSS themes (Standard, Academic, GitHub, Minimal)

## Screenshots

Marco features a clean, modern interface with split-pane editing and full multi-language support. The UI updates instantly when switching between English, Spanish, French, and German.

![marco](doc/img/marco.png)

## Current Status

### Core Features Complete
- **Split-pane editor** with live preview and syntax highlighting
- **Multi-language support** (4 languages) with instant switching
- **Advanced markdown** including GitHub-style admonitions, tables, task lists
- **Smart code blocks** with intelligent language search supporting 100+ programming languages
- **CSS themes** with 4 built-in styles (Standard, Academic, GitHub, Minimal) plus custom CSS support
- **Modern theme system** with automatic light/dark mode detection and CSS integration
- **Settings persistence** with visual menu checkmarks for current selections
- **Professional dialogs** with modal interfaces and real-time validation
- **Comprehensive UI** with toolbar, context menus, and keyboard shortcuts

### Partially Complete
- **Format detection** - functions exist, status bar display pending
- **Extended syntax** - all features implemented, some refinements needed

### Planned Features
- **Export formats** (PDF, HTML)
- **Search and replace** with regex support
- **Auto-save** and workspace persistence
- **Performance optimization** for large files

## Roadmap

### Next Priority
- **Search and replace** with regex support and find/replace dialog
- **Export formats** (PDF, HTML) with formatting preservation
- **Auto-save** with configurable intervals and document recovery
- **Document outline** with navigable heading structure in sidebar
- **Performance optimization** for large files and multiple documents

### Future Enhancements
- **Zen mode** distraction-free editing environment
- **Advanced accessibility** with screen reader support
- **Cross-platform distribution** with installers for Windows, macOS, and Linux

## Contributing

### Translation Contributions
We welcome translations to additional languages! Please follow the translation format in existing files and submit a pull request.
