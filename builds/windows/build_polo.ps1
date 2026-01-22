# Build Polo Markdown Viewer for Windows
#
# This script builds Polo viewer with all required assets and creates
# a portable distribution package.
#
# CROSS-PLATFORM PATH SUPPORT:
#   - All paths in core/src/paths/ use cfg(target_os = "windows") for Windows-specific paths
#   - Windows uses %APPDATA%, %LOCALAPPDATA%, and %TEMP% instead of Unix ~/.config, ~/.local, /tmp
#   - Logs go to %LOCALAPPDATA%\marco\log\ on Windows (not current directory)
#   - Asset detection checks exe_dir\marco_assets first (portable app support)
#
# Usage:
#   .\builds\windows\build_polo.ps1
#   .\builds\windows\build_polo.ps1 -CheckOnly
#   .\builds\windows\build_polo.ps1 -Help

param(
    [switch]$Help,
    [switch]$CheckOnly
)

$ErrorActionPreference = "Stop"

function Write-Header {
    param([string]$Message)
    Write-Host ""
    Write-Host "=========================================" -ForegroundColor Blue
    Write-Host $Message -ForegroundColor Blue
    Write-Host "=========================================" -ForegroundColor Blue
    Write-Host ""
}

function Write-Success {
    param([string]$Message)
    Write-Host "OK: $Message" -ForegroundColor Green
}

function Write-Error-Custom {
    param([string]$Message)
    Write-Host "ERROR: $Message" -ForegroundColor Red
}

function Write-Warning-Custom {
    param([string]$Message)
    Write-Host "WARN: $Message" -ForegroundColor Yellow
}

function Write-Info {
    param([string]$Message)
    Write-Host "INFO: $Message" -ForegroundColor Cyan
}

function Show-Help {
    @"
Polo Windows Package Builder

USAGE:
    .\builds\windows\build_polo.ps1 [OPTIONS]

DESCRIPTION:
    Builds a portable Windows package for Polo markdown viewer.

OPTIONS:
    -Help         Show this help message
    -CheckOnly    Check dependencies only (don't build)

OUTPUT:
    Creates: polo_VERSION_win64.zip in the workspace root.
    Contains: polo.exe, servo-runner.exe, and all required assets.
"@
}

if ($Help) {
    Show-Help
    exit 0
}

# Get script and root directories
$ScriptDir = $PSScriptRoot
$RootDir = Split-Path -Parent (Split-Path -Parent $ScriptDir)
Set-Location $RootDir

# Configuration
$PackageName = "polo"
$Architecture = "win64"
$VersionFile = "$RootDir\builds\linux\version.json"

Write-Header "Polo Windows Package Builder"

# Read version from JSON
if (-not (Test-Path $VersionFile)) {
    Write-Error-Custom "Version file not found: $VersionFile"
    exit 1
}

$VersionData = Get-Content $VersionFile | ConvertFrom-Json
$PoloVersion = $VersionData.polo

Write-Host "Package: $PackageName"
Write-Host "Version: $PoloVersion"
Write-Host "Architecture: $Architecture"
Write-Host ""

# Check dependencies
Write-Header "Checking Dependencies"

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error-Custom "Rust/Cargo not found"
    exit 1
}
Write-Success "Rust/Cargo found"

if (-not (Get-Command uv -ErrorAction SilentlyContinue)) {
    Write-Error-Custom "uv not found (required for Servo build)"
    Write-Info "Install: irm https://astral.sh/uv/install.ps1 | iex"
    exit 1
}
Write-Success "uv found"

if (-not (Test-Path "C:\Program Files\LLVM\bin\libclang.dll")) {
    Write-Error-Custom "LLVM/Clang not found"
    Write-Info "Install: winget install LLVM.LLVM"
    exit 1
}
Write-Success "LLVM/Clang found"

if (-not (Get-Command protoc -ErrorAction SilentlyContinue)) {
    Write-Error-Custom "protoc not found"
    Write-Info "Download from: https://github.com/protocolbuffers/protobuf/releases"
    exit 1
}
Write-Success "protoc found"

if ($CheckOnly) {
    Write-Success "Dependency check complete"
    exit 0
}

# Set build environment
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
$env:Path = "C:\Users\$env:USERNAME\.local\bin;C:\protoc\bin;$env:Path"

# Build Polo and servo-runner
Write-Header "Building Polo Viewer"
cargo build --release -p polo
if ($LASTEXITCODE -ne 0) {
    Write-Error-Custom "Failed to build polo"
    exit 1
}
Write-Success "Polo binary built (includes servo-runner from servo-gtk)"

# Create package structure
$BuildDir = "$RootDir\target\polo_package"
if (Test-Path $BuildDir) {
    Remove-Item -Recurse -Force $BuildDir
}

Write-Info "Setting up package directory structure"
New-Item -ItemType Directory -Force -Path "$BuildDir" | Out-Null
New-Item -ItemType Directory -Force -Path "$BuildDir\marco_assets" | Out-Null
New-Item -ItemType Directory -Force -Path "$BuildDir\marco_assets\themes" | Out-Null
New-Item -ItemType Directory -Force -Path "$BuildDir\marco_assets\fonts" | Out-Null
New-Item -ItemType Directory -Force -Path "$BuildDir\marco_assets\language" | Out-Null
New-Item -ItemType Directory -Force -Path "$BuildDir\marco_assets\icons" | Out-Null
New-Item -ItemType Directory -Force -Path "$BuildDir\doc" | Out-Null

# Copy binaries
Copy-Item "target\release\polo.exe" "$BuildDir\"
# servo-runner is built as part of servo-gtk dependency and included in polo.exe process
# Note: Servo uses a subprocess model, so servo-runner.exe is spawned by polo at runtime
if (Test-Path "target\release\servo-runner.exe") {
    Copy-Item "target\release\servo-runner.exe" "$BuildDir\"
    Write-Success "Binaries copied (polo.exe + servo-runner.exe)"
} else {
    Write-Success "Binary copied (polo.exe)"
    Write-Info "servo-runner.exe will be built on-demand by polo"
}

# Copy assets (Polo needs themes for HTML rendering)
# Use marco_assets directory name to match what the path detection expects
# Preserve the html_viever subdirectory structure
New-Item -ItemType Directory -Force -Path "$BuildDir\marco_assets\themes\html_viever" | Out-Null
Copy-Item -Recurse "assets\themes\html_viever\*" "$BuildDir\marco_assets\themes\html_viever\" -Force
Write-Success "Themes copied"

if (Test-Path "assets\fonts") {
    Copy-Item -Recurse "assets\fonts\*" "$BuildDir\marco_assets\fonts\" -Force
    Write-Success "Fonts copied"
}

if (Test-Path "assets\language") {
    Copy-Item -Recurse "assets\language\*" "$BuildDir\marco_assets\language\" -Force
    Write-Success "Language files copied"
}

if (Test-Path "assets\icons") {
    Copy-Item -Recurse "assets\icons\*" "$BuildDir\marco_assets\icons\" -Force
    Write-Success "Icons copied"
}

if (Test-Path "assets\settings_org.ron") {
    Copy-Item "assets\settings_org.ron" "$BuildDir\marco_assets\"
    Write-Success "Settings template copied"
}

# Copy documentation
Copy-Item "README.md" "$BuildDir\doc\"
Copy-Item "LICENSE" "$BuildDir\doc\"
if (Test-Path "changelog\polo.md") {
    Copy-Item "changelog\polo.md" "$BuildDir\doc\changelog.md"
}
if (Test-Path "servo_runner\README.md") {
    Copy-Item "servo_runner\README.md" "$BuildDir\doc\servo-runner.md"
}
Write-Success "Documentation copied"

# Create README for Windows package
$ReadmeContent = @"
# Polo - Markdown Viewer for Windows

Version: $PoloVersion

## Quick Start

1. Double-click polo.exe to launch the viewer
2. Open markdown files via File > Open or drag-and-drop

## Files

- polo.exe - Main viewer application
- marco_servo-runner.exe - Web rendering subprocess (spawned automatically)
- assets/ - Themes, fonts, and configuration files
- doc/ - Documentation and license

## Requirements

- Windows 10/11 (64-bit)
- No installation needed - this is a portable application

## Logs and Data Directories

### Logs
Application logs are written to:
  %LOCALAPPDATA%\marco\log\YYYYMM\YYMMDD.log

Example: C:\Users\YourName\AppData\Local\marco\log\202601\260122.log

Logs are organized by month and rotated when they exceed 10 MB.

### Configuration
User settings and preferences:
  %APPDATA%\marco\

### Cache
Temporary files and cache:
  %LOCALAPPDATA%\marco\cache\

## More Information

See doc/README.md for full documentation.
Visit: https://github.com/Ranrar/Marco
"@

Set-Content -Path "$BuildDir\README.txt" -Value $ReadmeContent
Write-Success "README.txt created"

# Create package archive
Write-Header "Creating Package Archive"
$PackageFile = "${PackageName}_${PoloVersion}_${Architecture}.zip"

if (Test-Path $PackageFile) {
    Remove-Item $PackageFile
}

Compress-Archive -Path "$BuildDir\*" -DestinationPath $PackageFile -CompressionLevel Optimal
Write-Success "Package created: $PackageFile"

# Calculate sizes
$BinarySize = [math]::Round((Get-Item "target\release\polo.exe").Length / 1MB, 2)
$PackageSize = [math]::Round((Get-Item $PackageFile).Length / 1MB, 2)

Write-Header "Build Complete"
Write-Host "Package file: $PackageFile"
Write-Host "Polo binary: $BinarySize MB"
Write-Host "Package size: $PackageSize MB"
Write-Host ""
Write-Host "Extract the ZIP and run polo.exe to launch the viewer."
