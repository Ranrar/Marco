# Marco Installer Packages

This directory contains all platform-specific installer packages for Marco.

## Build Outputs

All packages are output directly to this directory:

```
installer/
├── marco-suite_alpha_VERSION_linux_amd64.deb     # Linux package
└── marco-suite_alpha_VERSION_windows_amd64.zip   # Windows portable package
```

## Creating Installers

### Linux (.deb)
```bash
bash build/linux/build_deb.sh --no-bump
# Output: build/installer/marco-suite_alpha_VERSION_linux_amd64.deb
```

### Windows (.zip)
```powershell
# Build and package (PowerShell):
.\build\windows\build_portable.ps1

# Skip build (use existing binaries):
.\build\windows\build_portable.ps1 -SkipBuild

# Output: build/installer/marco-suite_alpha_VERSION_windows_amd64.zip
```

## Installation

### Linux
```bash
sudo dpkg -i build/installer/marco-suite_alpha_VERSION_linux_amd64.deb
# If dependencies are missing:
sudo apt -f install
```

### Windows
1. Extract the ZIP file to any location
2. Run `marco.exe` or `polo.exe`
3. Settings are stored in the extracted folder (portable mode)

## CI/CD

The `.github/workflows/` directory contains automated build workflows:

- **alpha-deb-release.yml**: Builds Linux package and updates Alpha release
- **alpha-win-release.yml**: Builds Windows package and updates Alpha release

Both workflows:
- Build binaries with explicit targets (x86_64-unknown-linux-gnu / x86_64-pc-windows-msvc)
- Create installer packages
- Upload to GitHub Releases (Alpha tag)
- Don't bump versions (use existing versions from `build/version.json`)

## Naming Convention

All packages follow this alpha-naming pattern:
- Linux: `marco-suite_alpha_<version>_linux_amd64.deb`
- Windows: `marco-suite_alpha_<version>_windows_amd64.zip`

Where `<version>` comes from `build/version.json` (platform-specific: `linux.marco` or `windows.marco`).

## Build Targets

- **Linux**: `x86_64-unknown-linux-gnu` → `target/x86_64-unknown-linux-gnu/release/`
- **Windows**: `x86_64-pc-windows-msvc` → `target/windows/x86_64-pc-windows-msvc/release/`

## Version Management

Versions are tracked in `build/version.json` with separate versions for Linux and Windows:

```json
{
  "linux": {
    "core": "0.16.0",
    "marco": "0.16.0",
    "polo": "0.16.0"
  },
  "windows": {
    "core": "0.16.0",
    "marco": "0.16.0",
    "polo": "0.16.0"
  }
}
```

By default, builds use existing versions. To bump versions:

```bash
# Linux (bump patch)
bash build/linux/build_deb.sh --bump patch

# Linux (set specific version)
bash build/linux/build_deb.sh --set 1.0.0
```
