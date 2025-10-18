# Marco & Polo Installation Guide

This directory contains installation scripts and desktop entries for Marco (markdown editor) and Polo (markdown viewer).

## Quick Install

```bash
# Install both Marco and Polo locally (recommended)
bash tests/install/install.sh
```

This automated script will:
1. Check for existing binaries (and ask if you want to rebuild)
2. Build both binaries in release mode if needed
3. Install executables to `~/.local/bin/`
4. Install desktop entries for application menu integration
5. Install system icons at multiple resolutions
6. Install shared assets to `~/.local/share/marco/`
7. Update desktop database and icon cache

**Note:** The script automatically detects if binaries already exist and asks if you want to rebuild them. This means you can simply run the install script without manually building first!

## Installation Locations

The new paths system follows the XDG Base Directory specification:

### Binaries
```
~/.local/bin/marco          # Marco markdown editor
~/.local/bin/polo           # Polo markdown viewer
```

### Shared Assets (Read-only)
```
~/.local/share/marco/       # Asset root (shared between Marco & Polo)
├── fonts/                  # IcoMoon icon font, UI fonts
├── icons/                  # Application icons (icon.png, favicon.png)
├── themes/                 # Editor and preview themes
│   ├── editor/             # GtkSourceView syntax themes (dark.xml, light.xml)
│   └── html_viever/        # HTML preview CSS themes (marco.css, github.css, etc.)
├── language/               # Translation files (*.json)
├── documentation/          # User guide and help files
└── settings.ron            # Default settings template
```

### User Configuration (Per-application)
```
~/.config/marco/            # Marco-specific config
└── settings.ron            # Marco's user settings

~/.config/polo/             # Polo-specific config  
└── settings.ron            # Polo's user settings
```

### User Data & Cache (Per-application)
```
~/.local/share/marco/data/  # Marco's data directory
├── cache/                  # Marco's parser cache
└── logs/                   # Marco's log files

~/.local/share/polo/data/   # Polo's data directory
├── cache/                  # Polo's parser cache
└── logs/                   # Polo's log files
```

### Desktop Integration
```
~/.local/share/applications/  # Desktop entries
├── marco.desktop
└── polo.desktop

~/.local/share/icons/hicolor/ # System icons for desktop environment
├── 256x256/apps/
├── 128x128/apps/
├── 64x64/apps/
└── 48x48/apps/
    ├── marco.png
    └── polo.png
```

## Manual Build

```bash
# Build Marco (editor)
cargo build --release -p marco

# Build Polo (viewer)
cargo build --release -p polo

# Binaries will be in:
target/release/marco
target/release/polo
```

## Desktop Entries

- **marco.desktop** - Marco markdown editor
- **polo.desktop** - Polo markdown viewer

Both entries:
- Support opening `.md` and `.markdown` files
- Integrate with the desktop file manager
- Use the shared icon from `~/.local/share/icons/`

## System Requirements

### Required
- **GTK4** - UI framework
- **WebKit6** - HTML preview rendering
- **Rust 1.70+** - For building from source

### Optional
- **ImageMagick** (`convert`) - For icon scaling during installation
  ```bash
  sudo apt install imagemagick  # Debian/Ubuntu
  sudo dnf install ImageMagick  # Fedora
  ```

## Uninstall

### Interactive Uninstaller (Recommended)

```bash
# Run the interactive uninstaller
bash tests/install/uninstall.sh
```

The uninstaller will ask you what to remove:
- **Binaries** - marco and polo executables
- **Desktop entries** - application menu entries
- **System icons** - icons in ~/.local/share/icons/
- **Shared assets** - themes, fonts, language files
- **User configuration** - your settings (kept by default)
- **User data/cache** - logs and cache (kept by default)

The script provides a summary before removing anything and won't delete user data without confirmation.

### Manual Uninstall

If you prefer to remove everything manually:

```bash
# Remove binaries
rm ~/.local/bin/marco
rm ~/.local/bin/polo

# Remove desktop entries
rm ~/.local/share/applications/marco.desktop
rm ~/.local/share/applications/polo.desktop

# Remove system icons
rm -rf ~/.local/share/icons/hicolor/*/apps/marco.png
rm -rf ~/.local/share/icons/hicolor/*/apps/polo.png

# Remove shared assets
rm -rf ~/.local/share/marco/

# Remove user configuration (optional)
rm -rf ~/.config/marco/
rm -rf ~/.config/polo/

# Update desktop database
update-desktop-database ~/.local/share/applications/
gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor/
```

### Verification

After uninstalling, you can verify all files are removed:

```bash
# Check for remaining files
ls ~/.local/bin/marco ~/.local/bin/polo 2>/dev/null && echo "Binaries still present" || echo "✓ Binaries removed"
ls ~/.local/share/applications/marco.desktop ~/.local/share/applications/polo.desktop 2>/dev/null && echo "Desktop entries still present" || echo "✓ Desktop entries removed"
ls -d ~/.local/share/marco 2>/dev/null && echo "Asset directory still present" || echo "✓ Assets removed"
ls -d ~/.config/marco ~/.config/polo 2>/dev/null && echo "Config still present" || echo "✓ Config removed"
```

## Architecture Notes

### New Paths System
The project uses a modular path system (`core/src/components/paths/`) that:
- Automatically detects development vs installed mode
- Provides separate config/data directories for Marco and Polo
- Shares common assets (fonts, themes, language files) between applications
- Follows XDG Base Directory specification

### Asset Detection Priority
1. **Development mode**: `target/*/marco_assets/` (from `build.rs`)
2. **User local**: `~/.local/share/marco/`
3. **System local**: `/usr/local/share/marco/`
4. **System global**: `/usr/share/marco/`

### Build System
- `core/build.rs` copies assets from `assets/` to `target/*/marco_assets/` during development
- The install script copies the same structure to `~/.local/share/marco/` for installed mode
- Both binaries (Marco and Polo) share the same `core` library and asset root

## Troubleshooting

### Icons don't appear in application menu
```bash
# Update icon cache
gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor/

# Update desktop database
update-desktop-database ~/.local/share/applications/

# Log out and back in, or restart your desktop environment
```

### "Asset directory not found" error
The application tries to find assets in this order:
1. Development: `target/debug/marco_assets/` or `target/release/marco_assets/`
2. User local: `~/.local/share/marco/`
3. System: `/usr/local/share/marco/` or `/usr/share/marco/`

Make sure you've run the install script or are building with cargo in the workspace root.

### Settings not persisting
Check that the config directory exists and is writable:
```bash
ls -la ~/.config/marco/
ls -la ~/.config/polo/
```

If missing, the applications will create them on first run.

## Development Workflow

When developing:
```bash
# Assets are automatically copied by build.rs
cargo build -p marco
cargo build -p polo

# Assets will be at:
target/debug/marco_assets/    # Debug build
target/release/marco_assets/  # Release build
```

When testing the installer:
```bash
# Install to ~/.local/
bash tests/install/install.sh

# Run installed versions
~/.local/bin/marco
~/.local/bin/polo
```
