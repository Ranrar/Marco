# Marco Build System

This directory contains build scripts for creating distribution packages of Marco, Polo, and the complete Marco Suite.

## Build Scripts

### Three Independent Packages

Marco uses a split build system that produces three separate Debian packages:

1. **`build_marco_suite.sh`** - Complete package with Marco editor + Polo viewer
2. **`build_marco.sh`** - Marco editor only (full-featured markdown editor)
3. **`build_polo.sh`** - Polo viewer only (lightweight markdown viewer with Servo)

Each script is self-contained and can be run independently.

### Build Requirements

**Common Dependencies:**
- Rust toolchain (1.90.0 or later)
- `dpkg-deb` for Debian packaging
- Standard build tools (`gcc`, `make`, etc.)

**Additional for Polo:**
- ImageMagick (`convert` command) - for icon resizing
- Python 3 - used by Servo build system

### Usage

```bash
# Build individual packages
cd /path/to/marco
bash builds/linux/build_marco_suite.sh
bash builds/linux/build_marco.sh
bash builds/linux/build_polo.sh

# Install
sudo dpkg -i marco-suite_0.14.0_amd64.deb
sudo dpkg -i marco_0.14.0_amd64.deb
sudo dpkg -i polo_0.14.0_amd64.deb
```

## Package Details

### Marco Suite (marco-suite)
- **Binary**: `/usr/bin/marco`, `/usr/bin/polo`
- **Assets**: `/usr/share/marco_assets/`
- **Icons**: `/usr/share/icons/hicolor/*/apps/marco.png`, `polo.png`
- **Desktop**: `/usr/share/applications/marco.desktop`, `polo.desktop`
- **Dependencies**: GTK4, SourceView5, WebKit6 (marco), Servo (polo)

### Marco (marco)
- **Binary**: `/usr/bin/marco`
- **Assets**: `/usr/share/marco_assets/`
- **Icons**: `/usr/share/icons/hicolor/*/apps/marco.png`
- **Desktop**: `/usr/share/applications/marco.desktop`
- **Dependencies**: GTK4, SourceView5, WebKit6

### Polo (polo)
- **Binary**: `/usr/bin/polo`, `/usr/bin/servo-runner`
- **Assets**: `/usr/share/marco_assets/` (shared with Marco)
- **Icons**: `/usr/share/icons/hicolor/*/apps/polo.png`
- **Desktop**: `/usr/share/applications/polo.desktop`
- **Dependencies**: GTK4, Servo web engine

## Icon Installation

Icons are installed in multiple sizes following the freedesktop.org icon theme specification:

**Marco Icons:**
- Source: `assets/icons/icon_64x64_marco.png`, `icon_662x662_marco.png`
- Installed: `/usr/share/icons/hicolor/{16,24,32,48,64,96,128,160,192,256,512}x{size}/apps/marco.png`

**Polo Icons:**
- Source: `assets/icons/icon_64x64_polo.png`, `icon_662x662_polo.png`
- Installed: `/usr/share/icons/hicolor/{16,24,32,48,64,96,128,160,192,256,512}x{size}/apps/polo.png`

Icons are resized using ImageMagick with exact sizing (`-resize ${size}x${size}!`).

## Post-Install Scripts

All packages include post-install scripts that:
1. Update GTK icon cache (`gtk-update-icon-cache`)
2. Update desktop database (`update-desktop-database`)
3. Display installation success message

## Build Process

Each build script follows this workflow:

1. **Version Check** - Read from `install/version.json`
2. **Dependency Check** - Verify required tools are available
3. **Build Binaries** - `cargo build --release` for workspace crates
4. **Create Package Structure** - Set up Debian directory hierarchy
5. **Copy Assets** - Install binaries, icons, desktop files, themes
6. **Generate Control File** - Create Debian package metadata
7. **Create Package** - Run `dpkg-deb --build`

## Servo Integration (Polo)

**Platform Support:** Linux only (servo-gtk requires Unix)

Polo uses servo-gtk from `/servo-gtk/` (external repository):

- **Servo Version**: 0.0.2 (git revision b9f5a7920f18bd5294ebb95bbb422199f6371a54)
- **Patch**: Changed from `cargo run --bin servo-runner` to direct `servo-runner` execution
- **Binary**: `servo-runner` is packaged alongside `polo` binary
- **Subprocess Cleanup**: Explicit `force_exit()` call on window close to prevent orphaned processes
- **Repository**: Moved from `third_party/servo-gtk/` to external `/servo-gtk/`

**Note:** Polo on Windows will require a different web engine (servo-gtk is Unix-only).

## Troubleshooting

### Build Failures

**"Rust/Cargo not found"**
- Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

**"dpkg-deb not found"**
- Install: `sudo apt install dpkg-dev`

**"convert not found" (polo only)**
- Install ImageMagick: `sudo apt install imagemagick`

### Runtime Issues

**Icons not appearing**
- Run: `sudo gtk-update-icon-cache -f /usr/share/icons/hicolor/`
- Check icon theme settings in GTK

**servo-runner not found (polo)**
- Ensure `/usr/bin/servo-runner` exists
- Check PATH includes `/usr/bin`

**servo-runner orphaned after closing polo**
- Update to latest version with subprocess cleanup fix
- Check logs in `~/.local/share/marco/log/`

## Version Management

Versions are managed centrally in `install/version.json`:

```json
{
  "core": "0.14.0",
  "marco": "0.14.0",
  "polo": "0.14.0"
}
```

To update versions, use `install/build_deb.sh --version-only` (see root install/ README).

## Contributing

When modifying build scripts:

1. Test all three build scripts independently
2. Verify installed packages work correctly (`dpkg -i`, run binaries)
3. Check lintian for packaging issues: `lintian package.deb`
4. Update this README if adding new dependencies or changing install paths
