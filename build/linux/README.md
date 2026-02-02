# Linux Build System

Debian package builder for Marco markdown editor.

## Quick Start

```bash
# Build package (compiles binaries and creates .deb)
bash build/linux/build_deb.sh --no-bump

# Output: build/installer/marco-suite_alpha_<version>_linux_amd64.deb
```

## Build Script

**`build_deb.sh`** - Main build script that:
1. Compiles Marco and Polo binaries with `cargo build --release --workspace --target x86_64-unknown-linux-gnu`
2. Creates Debian package structure
3. Installs binaries, assets, desktop files, icons, man pages
4. Generates `.deb` package with `dpkg-deb`
5. Outputs to `build/installer/marco-suite_alpha_<version>_linux_amd64.deb`

## Usage

### Standard Build (No Version Bump)
```bash
bash build/linux/build_deb.sh --no-bump
```

### Check Dependencies Only
```bash
bash build/linux/build_deb.sh --check
```

### Version Management

```bash
# Update version and sync Cargo.toml (no build)
bash build/linux/build_deb.sh --version-only --bump patch

# Set specific version
bash build/linux/build_deb.sh --set 1.0.0

# Bump and build
bash build/linux/build_deb.sh --bump minor
```

## Dependencies

### Required
```bash
# Debian/Ubuntu
sudo apt-get install -y \
    python3 \
    build-essential pkg-config \
    libgtk-4-dev libgtksourceview-5-dev libwebkitgtk-6.0-dev libfontconfig-dev \
    dpkg-dev fakeroot gzip
```

### Optional
```bash
# For icon generation
sudo apt-get install imagemagick
```

## Build Target

- **Target**: `x86_64-unknown-linux-gnu`
- **Output**: `target/x86_64-unknown-linux-gnu/release/marco` and `polo`

## Package Contents

```
/usr/bin/
├── marco                          # Main editor binary
└── polo                           # Viewer binary

/usr/share/applications/
├── marco.desktop
└── polo.desktop

/usr/share/icons/hicolor/
└── {16,24,32,48,64,96,128,160,192,256,512}x{size}/apps/
    ├── marco.png
    └── polo.png

/usr/share/man/man1/
├── marco.1.gz
└── polo.1.gz

/usr/share/marco/doc/
├── documentation/
├── README.md
└── LICENSE
```

## Installation

```bash
# Install package
sudo dpkg -i build/installer/marco-suite_alpha_<version>_linux_amd64.deb

# Fix missing dependencies (if any)
sudo apt -f install

# Uninstall
sudo dpkg -r marco-suite
```

## CI/CD

GitHub Actions workflow (`.github/workflows/alpha-deb-release.yml`):

```yaml
- name: Build Debian package
  run: |
    bash build/linux/build_deb.sh --no-bump
```

Workflow:
1. Checks out repository
2. Moves alpha tag to current commit
3. Installs Rust toolchain (1.90.0)
4. Caches cargo dependencies
5. Installs system dependencies
6. Builds package (no version bump)
7. Updates Alpha GitHub Release with new .deb

## Version Tracking

Versions are stored in `build/version.json`:

```json
{
  "linux": {
    "core": "0.16.0",
    "marco": "0.16.0",
    "polo": "0.16.0"
  }
}
```

The build script:
- Reads versions from `version.json`
- Syncs versions to `core/Cargo.toml`, `marco/Cargo.toml`, `polo/Cargo.toml`
- Uses `--no-bump` to prevent version changes in CI

## Troubleshooting

### Missing Dependencies
```bash
# Check what's missing
bash build/linux/build_deb.sh --check

# Install missing packages
sudo apt-get install <missing-package>
```

### Build Fails
```bash
# Clean and rebuild
cargo clean
bash build/linux/build_deb.sh --no-bump
```

### Package Won't Install
```bash
# Check package contents
dpkg-deb --contents build/installer/marco-suite_alpha_*.deb

# Check package info
dpkg-deb --info build/installer/marco-suite_alpha_*.deb

# Force install (not recommended)
sudo dpkg -i --force-all build/installer/marco-suite_alpha_*.deb
```

## Desktop Files

Marco and Polo are automatically added to application menus with:
- Icons (multiple sizes for HiDPI support)
- Desktop entries (`.desktop` files)
- Man pages (compressed with gzip)

Launch from:
- Application menu (search for "Marco" or "Polo")
- Terminal: `marco` or `polo`
- With file: `marco document.md`

## Package Metadata

- **Package**: marco-suite
- **Section**: editors
- **Priority**: optional
- **Architecture**: amd64
- **Maintainer**: Kim Skov Rasmussen <kim@skovrasmussen.com>
- **Homepage**: https://github.com/Ranrar/marco
- **License**: MIT

## Advanced Options

```bash
# View all options
bash build/linux/build_deb.sh --help
```

Key options:
- `--no-bump`: Build without changing version (default for CI)
- `--bump patch|minor|major`: Bump version before building
- `--set X.Y.Z`: Set specific version
- `--version-only`: Update versions without building
- `--check`: Check dependencies only
