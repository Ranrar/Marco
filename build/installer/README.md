# Marco Installer Packages

This directory contains all platform-specific installer packages for Marco.

## Build Outputs

All packages are output directly to this directory:

```
installer/
├── markdown-composer-and-viewer_VERSION_linux_amd64.deb          # Linux package
├── markdown-composer-and-viewer_VERSION_windows_amd64.zip        # Windows portable package
└── markdown-composer-and-viewer_VERSION_windows_amd64_setup.exe  # Windows installer (Inno Setup)
```

## Creating Installers

### Linux (.deb)
```bash
bash build/linux/build_deb.sh --no-bump
# Output: build/installer/markdown-composer-and-viewer_VERSION_linux_amd64.deb
```

### Windows portable (.zip)
```powershell
# Build and package (PowerShell):
.\build\windows\build_portable.ps1

# Skip build (use existing binaries):
.\build\windows\build_portable.ps1 -SkipBuild

# Output: build/installer/markdown-composer-and-viewer_VERSION_windows_amd64.zip
```

### Windows installer (.exe, Inno Setup)
```powershell
# Build and package (requires Inno Setup 6: https://jrsoftware.org/isdl.php):
.\build\windows\build_installer.ps1

# Skip build (use existing binaries):
.\build\windows\build_installer.ps1 -SkipBuild

# Output: build/installer/markdown-composer-and-viewer_VERSION_windows_amd64_setup.exe
```
Self-contained -- does not call `build_portable.ps1` or depend on the
portable zip. See `build/windows/installer.iss` and `.dev/new_fun/inno.md`
for design details (notably why `config/`/`data/` are deliberately excluded
from the installer, unlike the portable zip).

## Installation

### Linux
```bash
sudo dpkg -i build/installer/markdown-composer-and-viewer_VERSION_linux_amd64.deb
# If dependencies are missing:
sudo apt -f install
```

### Windows (portable)
1. Extract the ZIP file to any location
2. Run `marco.exe` or `polo.exe`
3. Settings are stored in the extracted folder (portable mode)

### Windows (installer)
1. Run the `_setup.exe` and follow the wizard (per-user install by default,
   no admin rights required; can be run elevated for an all-users install)
2. Launch from the Start Menu (`Marco` / `Polo`) or the optional desktop
   shortcuts offered during install
3. Settings are stored in `%APPDATA%\marco` (not next to the install
   directory -- unlike the portable zip)

## CI/CD

The `.github/workflows/` directory contains automated build workflows:

- **release.yml**: Builds Linux/Windows packages and publishes the versioned GitHub release. The Windows job builds the portable zip first, then reuses those same binaries (`-SkipBuild`) to build the Inno Setup installer in the same job -- Rust is only compiled once per run.

Both workflows:
- Build binaries with explicit targets (x86_64-unknown-linux-gnu / x86_64-pc-windows-gnu)
- Create installer packages
- Upload to GitHub Releases (`release` tag)
- Don't bump versions (use existing versions from `build/version.json`)

## Naming Convention

All packages follow this release naming pattern:
- Linux: `markdown-composer-and-viewer_<version>_linux_amd64.deb`
- Windows (portable): `markdown-composer-and-viewer_<version>_windows_amd64.zip`
- Windows (installer): `markdown-composer-and-viewer_<version>_windows_amd64_setup.exe`

Where `<version>` comes from `build/version.json` (platform-specific: `linux.marco` or `windows.marco`).

## Build Targets

- **Linux**: `x86_64-unknown-linux-gnu` → `target/x86_64-unknown-linux-gnu/release/`
- **Windows**: `x86_64-pc-windows-msvc` → `target/windows/x86_64-pc-windows-msvc/release/`

## Version Management

Versions are tracked in `build/version.json` with separate versions for Linux and Windows:

```json
{
  "linux": {
    "marco-shared": "0.23.2",
    "marco": "0.23.2",
    "polo": "0.23.2"
  },
  "windows": {
    "marco-shared": "0.23.2",
    "marco": "0.23.2",
    "polo": "0.23.2"
  }
}
```

## Release Artifacts

- Artifacts use versioned release naming (no channel suffixes in filenames).

By default, builds use existing versions. To bump versions:

```bash
# Linux (bump patch)
bash build/linux/build_deb.sh --bump patch

# Linux (set specific version)
bash build/linux/build_deb.sh --set 1.0.0
```
