# Polo - Markdown Viewer

A lightweight, standalone markdown viewer for the Marco editor ecosystem.

## Overview

Polo is a **view-only** markdown application that provides fast previews without editing capabilities. It's the perfect companion to Marco for quickly viewing documentation and markdown files.

**Key Features:**
- Fast rendering with cached parsing
- Light/dark modes with multiple themes
- Opens files in Marco editor with one click
- Full support for Marco's custom markdown extensions
- Minimal UI - just a viewer

## Usage

### Command Line

```bash
polo <file.md>           # Open markdown file
polo --debug <file.md>   # Open with debug logging
polo --help              # Show help
```

### UI Controls

**Titlebar:**
- **Open** - Open file picker
- **Open in Editor** - Launch Marco editor
- **Theme Dropdown** - Select CSS theme (github, marco, academic, etc.)
- **Mode Toggle** (☀️/🌙) - Switch light/dark mode
- **Window Controls** - Minimize, maximize, close

**Opening in Marco:**
1. **DualView** - Close Polo, open Marco with editor + preview
2. **Separate** - Keep Polo open, also launch Marco

## Architecture

```
polo/src/
├── main.rs              # Application entry point
└── components/
    ├── css/             # Styling (Marco's menu.css + Polo overrides)
    ├── dialog.rs        # File picker, Marco integration
    ├── menu.rs          # Custom titlebar
    ├── utils.rs         # Helper functions
    └── viewer/          # Markdown rendering
        ├── empty_state.rs
        └── rendering.rs
```

**Component Responsibilities:**
- **css** - Load and manage UI styling
- **dialog** - File operations and Marco editor launching
- **menu** - Custom titlebar with theme controls
- **utils** - Color parsing, theme management
- **viewer** - Load files and render HTML to WebView

## Troubleshooting

**"Marco editor not found"**
- Ensure `marco` is in same directory as `polo` or in system PATH

**"Asset directory not found"**
- Assets must be in `marco_assets/` directory relative to binary

**White screen / No content**
- Check log files for errors
- Verify file path is correct and readable

## Contributing

Polo is part of the Marco project. Contributions welcome

1. Fork the repository
2. Create a feature branch
3. Make your changes following code style guidelines
4. Test thoroughly
5. Open a Pull Request