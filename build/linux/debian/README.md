# Linux Build System

Debian package builder for Marco markdown editor.

## Quick Start

```bash
# Build package (compiles binaries and creates .deb)
bash build/linux/debian/build_deb.sh --no-bump

# Output: build/installer/markdowncomposerandviewer_<version>_linux_amd64.deb
```

## Build Script

**`build_deb.sh`** - Main build script that:
1. Compiles Marco and Polo binaries with `cargo build --release --workspace --target x86_64-unknown-linux-gnu`
2. Creates Debian package structure
3. Installs binaries, assets, desktop files, icons, man pages
4. Generates `.deb` package with `dpkg-deb`
5. Outputs to `build/installer/markdowncomposerandviewer_<version>_linux_amd64.deb`

## Usage

### Standard Build (No Version Bump)
```bash
bash build/linux/debian/build_deb.sh --no-bump
```

### Check Dependencies Only
```bash
bash build/linux/debian/build_deb.sh --check
```

### Version Management

```bash
# Update version and sync Cargo.toml (no build)
bash build/linux/debian/build_deb.sh --version-only --bump patch

# Set specific version
bash build/linux/debian/build_deb.sh --set 1.0.0

# Bump and build
bash build/linux/debian/build_deb.sh --bump minor
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
├── markdowncomposer               # Main editor binary
└── markdownviewer                 # Viewer binary

/usr/share/applications/
├── markdowncomposer.desktop
└── markdownviewer.desktop

/usr/share/icons/hicolor/
└── {16,24,32,48,64,96,128,160,192,256,512}x{size}/apps/
    ├── markdowncomposer.png
    └── markdownviewer.png

/usr/share/man/man1/
├── markdowncomposer.1.gz
└── markdownviewer.1.gz

/usr/share/markdowncomposer/doc/
├── documentation/
├── README.md
└── LICENSE
```

> **Why `markdowncomposer` / `markdownviewer` and not `marco` / `polo`?**
> `/usr/bin/marco`, `marco.desktop`, `marco.1`, `apps/marco.png` and
> `/usr/share/marco/` are all owned by the unrelated MATE window manager
> package `marco` in the Debian/Ubuntu archives. dpkg detects conflicts by
> file path rather than package name, so those paths made this package
> impossible to install on any MATE system regardless of it being named
> `marco-suite` ([#41](https://github.com/Ranrar/Marco/issues/41)).
> Per-user config, data and cache moved to `~/.config/markdowncomposer/`,
> `~/.local/share/markdowncomposer/` and `~/.cache/markdowncomposer/` for the
> same reason. **This is a breaking change with no migration** — settings under
> the old `marco` directories are not read, and users must copy anything they
> want to keep across by hand.
>
> `Conflicts`/`Replaces` would be wrong — a window manager and a markdown
> editor are unrelated, and installing an editor must not remove someone's
> window manager — so the installed artifacts are renamed instead. The
> project, the crates and the running applications are all still called Marco
> and Polo; only the installed file and directory names change. Polo is renamed
> alongside for consistency, not because it collides.

## Installation

```bash
# Install package
sudo dpkg -i build/installer/markdowncomposerandviewer_<version>_linux_amd64.deb

# Fix missing dependencies (if any)
sudo apt -f install

# Uninstall
sudo dpkg -r markdowncomposerandviewer
```

## CI/CD

GitHub Actions workflow (`.github/workflows/release.yml`):

```yaml
- name: Build Debian package
  run: |
    bash build/linux/debian/build_deb.sh --no-bump
```

Workflow:
1. Checks out repository
2. Moves `release` tag to current commit
3. Installs Rust toolchain (1.90.0)
4. Caches cargo dependencies
5. Installs system dependencies
6. Builds package (no version bump)
7. Updates the GitHub Release with new .deb

## Version Tracking

Versions are stored in `build/version.json`:

```json
{
  "linux": {
    "marco-shared": "0.23.2",
    "marco": "0.23.2",
    "polo": "0.23.2"
  }
}
```

The build script:
- Reads versions from `version.json`
- Syncs versions to `marco-shared/Cargo.toml`, `marco/Cargo.toml`, `polo/Cargo.toml`
- `marco-core` is published independently on crates.io from its own repository
  (https://github.com/Ranrar/marco-core); the workspace `Cargo.toml` pins the
  consumed version
- Uses `--no-bump` to prevent version changes in CI

## Troubleshooting

### Missing Dependencies
```bash
# Check what's missing
bash build/linux/debian/build_deb.sh --check

# Install missing packages
sudo apt-get install <missing-package>
```

### Build Fails
```bash
# Clean and rebuild
cargo clean
bash build/linux/debian/build_deb.sh --no-bump
```

### Package Won't Install
```bash
# Check package contents
dpkg-deb --contents build/installer/markdowncomposerandviewer_*.deb

# Check package info
dpkg-deb --info build/installer/markdowncomposerandviewer_*.deb

# Force install (not recommended)
sudo dpkg -i --force-all build/installer/markdowncomposerandviewer_*.deb
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

- **Package**: markdowncomposerandviewer
- **Section**: editors
- **Priority**: optional
- **Architecture**: amd64
- **Maintainer**: Kim Skov Rasmussen <kim@skovrasmussen.com>
- **Homepage**: https://github.com/Ranrar/marco
- **License**: MIT

## Advanced Options

```bash
# View all options
bash build/linux/debian/build_deb.sh --help
```

Key options:
- `--no-bump`: Build without changing version (default for CI)
- `--bump patch|minor|major`: Bump version before building
- `--set X.Y.Z`: Set specific version
- `--version-only`: Update versions without building
- `--check`: Check dependencies only
