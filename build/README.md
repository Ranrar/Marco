# Marco Build System

Cross-platform build scripts for Marco markdown editor.

## Directory Structure

```
build/
├── installer/           # All installer packages output here
│   ├── linux/          # .deb packages
│   └── windows/        # .zip packages
├── linux/              # Linux build scripts
├── windows/            # Windows build scripts
└── version.json        # Version tracking
```

## Platform-Specific Builds

### Linux (webkit6)
```bash
# Build Debian package (includes compilation)
bash build/linux/build_deb.sh

# Output: build/installer/linux/marco-suite_VERSION_amd64.deb
```

### Windows (wry/WebView2)
```powershell
# Build binary (PowerShell)
.\build\windows\build.ps1 -Release

# Create installer package (in Git Bash/WSL)
bash build/windows/create_installer.sh

# Output: build/installer/windows/marco-suite_VERSION_windows_x64.zip
```

## Build Targets

| Platform | WebView | Binary Location | Installer Output |
|----------|---------|----------------|------------------|
| Linux | webkit6 | `target/linux/release/marco` | `build/installer/linux/*.deb` |
| Windows | wry (WebView2) | `target/windows/release/marco.exe` | `build/installer/windows/*.zip` |


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
