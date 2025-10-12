# Marco & Polo Installation Scripts

This directory contains installation scripts and desktop entries for Marco (markdown editor) and Polo (markdown viewer).

## Quick Install

```bash
# Install both Marco and Polo locally
bash tests/install/local.sh
```

This will:
1. Build both binaries in release mode
2. Install to `~/.local/bin/marco` and `~/.local/bin/polo`
3. Install desktop entries for both applications
4. Install icons at multiple resolutions
5. Install shared assets (themes, fonts, language files)
6. Install documentation

## Manual Build

```bash
# Build Marco (editor)
cargo build --release -p marco

# Build Polo (viewer)
cargo build --release -p polo

# Binaries will be in target/release/
```

## Desktop Entries

- **marco.desktop** - Marco markdown editor
- **polo.desktop** - Polo markdown viewer

Both applications share the same asset directory (`~/.local/share/marco/`) for themes, fonts, and other resources.

## Uninstall

```bash
# Remove binaries
rm ~/.local/bin/marco
rm ~/.local/bin/polo

# Remove desktop entries
rm ~/.local/share/applications/marco.desktop
rm ~/.local/share/applications/polo.desktop

# Remove icons
rm ~/.local/share/icons/hicolor/*/apps/marco.png
rm ~/.local/share/icons/hicolor/*/apps/polo.png

# Remove shared assets
rm -rf ~/.local/share/marco/

# Update desktop database
update-desktop-database ~/.local/share/applications/
```

## Requirements

- **ImageMagick** (`convert` command) - for icon scaling
- **GTK4** - runtime dependency
- **WebKit6** - for HTML preview rendering

## Notes

- Both applications use the workspace structure with `marco_core` as the shared library
- Assets are located in the workspace root `assets/` directory
- The installation script requires `convert` from ImageMagick for icon generation
- You may need to log out and back in for new applications to appear in your desktop menu
