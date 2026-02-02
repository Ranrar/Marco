# Marco Build System

Cross-platform build scripts for Marco markdown editor.

## Directory Structure

```
build/
├── installer/           # All installer packages output here (unified directory)
├── linux/              # Linux build scripts
│   └── build_deb.sh   # Main build script for Debian packages
├── windows/            # Windows build scripts
│   ├── build.ps1      # Binary build script
│   ├── build_portable.ps1  # Portable package builder (recommended)
│   └── package.ps1    # Full installer builder (advanced)
└── version.json        # Version tracking (linux/windows sections)
```

## Platform-Specific Builds

### Linux (webkit6)
```bash
# Build Debian package (includes compilation with explicit target)
bash build/linux/build_deb.sh --no-bump

# Output: build/installer/marco-suite_alpha_VERSION_linux_amd64.deb
```

### Windows (wry/WebView2)
```powershell
# Build and package (PowerShell - recommended)
.\build\windows\build_portable.ps1

# Skip build (use existing binaries)
.\build\windows\build_portable.ps1 -SkipBuild

# Output: build/installer/marco-suite_alpha_VERSION_windows_amd64.zip
```

## Build Targets

| Platform | Target Triple | Binary Location | Installer Output |
|----------|--------------|----------------|------------------|
| Linux | `x86_64-unknown-linux-gnu` | `target/x86_64-unknown-linux-gnu/release/marco` | `build/installer/*.deb` |
| Windows | `x86_64-pc-windows-msvc` | `target/windows/x86_64-pc-windows-msvc/release/marco.exe` | `build/installer/*.zip` |


## Architecture

```
Marco Core (Pure Rust)
        ↓
Platform Abstraction Layer
        ↓
   ┌────────┴────────┐
   ↓                 ↓
webkit6          wry/WebView2
(Linux)           (Windows)
```

## Dependencies

### Linux
```bash
# Debian/Ubuntu
sudo apt install libgtk-4-dev libgtksourceview-5-dev libwebkitgtk-6.0-dev

# Fedora
sudo dnf install gtk4-devel gtksourceview5-devel webkit2gtk4.1-devel

# Arch
sudo pacman -S gtk4 gtksourceview5 webkit2gtk-4.1
```

### Windows
- MSYS2 with MinGW-w64
- GTK4 via `pacman -S mingw-w64-ucrt-x86_64-gtk4`
- WebView2 runtime (included in Windows 10/11)

## Version Management

Version tracking: `build/version.json`

```bash
# Bump patch version and build
bash build/linux/build_deb.sh

# Bump minor version
bash build/linux/build_deb.sh --bump minor

# Set specific version
bash build/linux/build_deb.sh --set 1.0.0
```
