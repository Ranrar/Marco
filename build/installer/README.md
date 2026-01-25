# Marco Installer Packages

This directory contains all platform-specific installer packages for Marco.

## Structure

```
installer/
├── linux/          # Debian packages (.deb)
└── windows/        # Windows ZIP packages
```

## Creating Installers

### Linux (.deb)
```bash
bash build/linux/build_deb.sh
# Output: build/installer/linux/marco-suite_VERSION_amd64.deb
```

### Windows (.zip)
```bash
# First build the binary (PowerShell):
.\build\windows\build.ps1 -Release

# Then create package (Git Bash/WSL):
bash build/windows/create_installer.sh
# Output: build/installer/windows/marco-suite_VERSION_windows_x64.zip
```

## Installation

### Linux
```bash
sudo dpkg -i build/installer/linux/marco-suite_VERSION_amd64.deb
```

### Windows
1. Extract the ZIP file
2. Run `marco.exe`
3. Optionally move to `C:\Program Files\Marco\`

## CI/CD

The `.github/workflows/` directory contains automated build workflows that:
- Build binaries for each platform
- Create installer packages
- Upload to GitHub Releases

## Naming Convention

All packages follow this naming pattern:
- Linux: `marco-suite_VERSION_amd64.deb`
- Windows: `marco-suite_VERSION_windows_x64.zip`

Where `VERSION` comes from `build/version.json`.
