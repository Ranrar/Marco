# Marco Build System

Cross-platform build scripts for Marco (editor) and Polo (viewer).

## Directory Structure

```
build/
├── installer/            # All packages output here (gitignored)
├── linux/
│   ├── debian/           # Debian package (.deb)
│   │   ├── build_deb.sh
│   │   ├── marco.desktop
│   │   └── polo.desktop
│   └── flatpak/          # Flatpak bundles — TWO apps, one per subdirectory
│       ├── build_flatpak_marco.sh
│       ├── build_flatpak_polo.sh
│       ├── _common.sh                          # shared by both scripts
│       ├── marco/
│       │   ├── io.github.ranrar.Marco.yml          # manifest
│       │   ├── io.github.ranrar.Marco.desktop
│       │   ├── io.github.ranrar.Marco.service      # D-Bus activation
│       │   ├── io.github.ranrar.Marco.metainfo.xml # AppStream metadata
│       │   └── icons/
│       ├── polo/
│       │   ├── io.github.ranrar.Marco.Polo.yml
│       │   ├── io.github.ranrar.Marco.Polo.desktop
│       │   ├── io.github.ranrar.Marco.Polo.metainfo.xml
│       │   └── icons/                          # 256x256, app-ID named
├── windows/
│   ├── build_portable.ps1   # Portable zip (recommended)
│   ├── build_installer.ps1  # Inno Setup installer
│   └── installer.iss
└── version.json          # Version tracking (linux/windows sections)
```

## Platform-Specific Builds

### Linux — Debian package

```bash
bash build/linux/debian/build_deb.sh --no-bump

# Output: build/installer/markdown-composer-and-viewer_VERSION_linux_amd64.deb
```

### Linux — Flatpak bundle

Marco and Polo are **two separate Flatpaks** with two manifests and two
scripts. Build whichever you need; neither builds the other.

```bash
bash build/linux/flatpak/build_flatpak_marco.sh
bash build/linux/flatpak/build_flatpak_polo.sh

# Output: build/installer/marco_VERSION_linux_amd64.flatpak
#         build/installer/polo_VERSION_linux_amd64.flatpak
```

Polo installs and runs on its own; with Marco absent its "open in Marco" action
offers to install Marco instead. They are two Flatpaks because the platform
assumes one application per app ID.

Builds, installs into the user's Flatpak for testing, and emits a bundle. Run
it with `--check` first to verify the runtime and SDK are present.

```bash
flatpak run io.github.ranrar.Marco        # Marco (editor)
flatpak run io.github.ranrar.Marco.Polo   # Polo (viewer)
```

Two app IDs, because the platform assumes one application per ID: under a single
ID the two desktop files collide, and anything mapping a running window back to
an application — GNOME Resources among them — labels both with the same name.
Polo reaches Marco over D-Bus rather than by spawning a sibling binary, which is
what allows them to live in separate sandboxes.

### Windows (wry / WebView2)

```powershell
# Portable zip (recommended)
.\build\windows\build_portable.ps1

# Skip build, package existing binaries
.\build\windows\build_portable.ps1 -SkipBuild

# Inno Setup installer
.\build\windows\build_installer.ps1
```

Outputs `build/installer/markdown-composer-and-viewer_VERSION_windows_amd64.zip`
and `..._setup.exe`.

## Release Artifacts

| Platform | Format | Output |
|---|---|---|
| Linux | Debian package | `build/installer/*.deb` |
| Linux | Flatpak bundle | `build/installer/*.flatpak` |
| Windows | Portable zip | `build/installer/*.zip` |
| Windows | Installer | `build/installer/*_setup.exe` |

All are built locally and attached to the GitHub release. Flathub is a separate
channel that builds from source on its own infrastructure — it does not accept
an uploaded bundle. See `linux/flatpak/README.md`.

## Build Targets

| Platform | Target Triple | Binary Location |
|----------|--------------|----------------|
| Linux (deb) | `x86_64-unknown-linux-gnu` | `target/x86_64-unknown-linux-gnu/release/marco` |
| Linux (flatpak) | host default | `target/release/marco`, inside the sandbox |
| Windows | `x86_64-pc-windows-msvc` | `target/x86_64-pc-windows-msvc/release/marco.exe` |

## Installed Names

The editor installs as **`markdowncomposer`**, not `marco`: on Debian/Ubuntu,
`/usr/bin/marco` and friends are owned by the unrelated MATE window manager
package, and dpkg detects conflicts by file path. Polo installs as
`markdownviewer` for symmetry. The Flatpak keeps the same names so the
compile-time constants in `marco-shared/src/paths/mod.rs` hold everywhere.

## Architecture

```
Marco Core (Pure Rust)
        ↓
   wry (unified webview)
        ↓
   ┌────────┴────────┐
   ↓                 ↓
GTK4/WebKit6      WebView2
  (Linux)         (Windows)
```

## Dependencies

### Linux — building natively (deb)

```bash
# Debian/Ubuntu
sudo apt install libgtk-4-dev libgtksourceview-5-dev libwebkitgtk-6.0-dev

# Fedora
sudo dnf install gtk4-devel gtksourceview5-devel webkitgtk6.0-devel

# Arch
sudo pacman -S gtk4 gtksourceview5 webkitgtk-6.0
```

### Linux — building the Flatpak

No system development packages are needed; the runtime supplies everything.

```bash
flatpak install -y flathub org.flatpak.Builder
flatpak install -y flathub org.gnome.Platform//50
flatpak install -y flathub org.gnome.Sdk//50
flatpak install -y flathub org.freedesktop.Sdk.Extension.rust-stable//25.08
```

### Windows

- MSYS2 with MinGW-w64
- GTK4 via `pacman -S mingw-w64-ucrt-x86_64-gtk4`
- WebView2 runtime (included in Windows 10/11)
- Inno Setup 6 (for `build_installer.ps1`)

## Version Management

Version tracking: `build/version.json`.

```bash
# Bump patch version and build
bash build/linux/debian/build_deb.sh

# Bump minor version
bash build/linux/debian/build_deb.sh --bump minor

# Set specific version
bash build/linux/debian/build_deb.sh --set 1.0.0
```

`build_deb.sh` owns version bumping and syncs the `Cargo.toml` files. The two
Flatpak scripts only *read* `version.json` — Marco's takes `.linux.marco` and
Polo's takes `.linux.polo` — and each warns if its own metainfo has no
`<release>` entry for that version. Those files drive what software centres
display, and nothing updates them automatically.
