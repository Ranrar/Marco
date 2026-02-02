# Windows Build Quick Reference

## Prerequisites Check
```powershell
.\build\windows\build.ps1 -CheckPrereqs
```

## Build Commands

### Development Build
```powershell
.\build\windows\build.ps1
```

### Production Build
```powershell
.\build\windows\build.ps1 -Release
```

### Build Both Marco and Polo
```powershell
.\build\windows\build.ps1 -Release -BuildPolo
```

### Clean Build
```powershell
.\build\windows\build.ps1 -Clean -Release
```

## Package Commands

### Portable Build (Recommended - Simple Zip)
```powershell
# Build binaries and create package (default)
.\build\windows\build_portable.ps1

# Skip build (use existing binaries)
.\build\windows\build_portable.ps1 -SkipBuild
```

### Installer Package (Full Installer)
```powershell
# Basic installer (requires manual runtime setup)
.\build\windows\package.ps1 -Release

# Installer with auto-collected runtime DLLs
.\build\windows\package.ps1 -Release -AutoCollectRuntime
```

### Full Auto Build and Package
```powershell
.\build\windows\package.ps1 -Release -BuildBinaries -AutoCollectRuntime
```

## Output Locations

### Binaries
- `target\windows\x86_64-pc-windows-msvc\debug\marco.exe`
- `target\windows\x86_64-pc-windows-msvc\release\marco.exe`

### Portable Package
- `build\installer\marco-suite_alpha_<version>_windows_amd64.zip`

### Installer
- `build\installer\marco-suite_installer_<version>_amd64.exe`

## Required Software

1. **Rust**: https://rustup.rs
2. **MSYS2**: https://www.msys2.org (only for installer with runtime DLLs)
3. **WebView2**: https://go.microsoft.com/fwlink/p/?LinkId=2124703
4. **Inno Setup 6**: https://jrsoftware.org/isdl.php (only for installer)

## MSYS2 Setup (Optional - Only for Installer with Runtime)

```bash
# In MSYS2 MinGW 64-bit shell:
pacman -Syu
pacman -S --needed mingw-w64-x86_64-toolchain
pacman -S --needed mingw-w64-x86_64-pkg-config
pacman -S --needed mingw-w64-x86_64-gtk4
```

## Common Issues

### "MSYS2 not found"
```powershell
.\build\windows\package.ps1 -Release -Msys2Root C:\msys64
```

### "Inno Setup not found"
```powershell
.\build\windows\package.ps1 -Release -InnoCompiler "C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
```

### Build from MSYS2 Shell
If adding to PATH is not desired, run from MSYS2 MinGW 64-bit shell:
```bash
cd /c/code/Marco
./build/windows/build.ps1 -Release
```

## Testing the Build

```powershell
# Run the built binary
.\target\windows\release\marco.exe

# Test the portable package
# Extract MarcoPortable_<version>_windows_x64.zip and run marco.exe

# Test the installer (keep staging for inspection)
.\build\windows\package.ps1 -Release -KeepStaging
```

## CI/CD

```powershell
# Strict mode (fail on missing prerequisites)
.\build\windows\build.ps1 -Release -FailOnMissing

# Full automated portable build (default - builds binaries)
.\build\windows\build_portable.ps1

# Full automated installer build
.\build\windows\build.ps1 -Release -BuildPolo
.\build\windows\package.ps1 -Release -AutoCollectRuntime
```

## Help

```powershell
.\build\windows\build.ps1 -Help
.\build\windows\build_portable.ps1 -Help
.\build\windows\package.ps1 -Help
```

## Portable vs Installer

### Portable Build (Recommended for most users)
- **Pros**: Simple, no installation needed, works from USB drives
- **Cons**: Requires manual WebView2 installation if not present
- **Best for**: USB drives, testing, portable deployment
- **Command**: `.\build\windows\build_portable.ps1 -BuildBinaries`
- **Output**: ZIP file ready to extract and run

### Installer Build
- **Pros**: Professional installer, WebView2 auto-install, system integration
- **Cons**: Requires MSYS2 for runtime DLLs, more complex setup
- **Best for**: Production releases, system-wide installation
- **Command**: `.\build\windows\package.ps1 -Release -BuildBinaries -AutoCollectRuntime`
- **Output**: EXE installer

See [README.md](README.md) for detailed documentation.
