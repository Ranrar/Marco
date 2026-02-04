# Windows Portable Build Script
# Creates a simple zip file with Marco, Polo, and assets for portable deployment

param(
    [switch]$Release = $true,
    [switch]$SkipBuild,
    [Alias('h')]
    [switch]$Help
)

$ErrorActionPreference = 'Stop'

# This script is intended to run on Windows.
# Running it via `pwsh` on Linux will attempt a Linux→Windows cross-compilation,
# which requires a full Windows GTK/GLib sysroot + cross pkg-config setup.
$runningOnWindows = $false
if (Get-Variable -Name IsWindows -ErrorAction SilentlyContinue) {
    # PowerShell 6+ defines $IsWindows/$IsLinux/$IsMacOS automatic variables.
    $runningOnWindows = [bool]$IsWindows
} else {
    # Windows PowerShell 5.1 does not define $IsWindows and only runs on Windows.
    # Use a conservative fallback to avoid false failures.
    $runningOnWindows = ($env:OS -eq 'Windows_NT') -or ($PSVersionTable.PSEdition -eq 'Desktop')
}

if (-not $runningOnWindows) {
    Write-Error "This script must be run on Windows. You appear to be running PowerShell on a non-Windows OS, which is not supported for building the Windows GTK binaries. Use a Windows machine/VM or the GitHub Actions windows-latest job (alpha-release workflow)."
    exit 1
}

if ($Help) {
    Write-Host @"
Marco Windows Portable Build Script

Creates a portable zip package containing Marco and Polo with all assets.

USAGE:
    .\build\windows\build_portable.ps1 [OPTIONS]

OPTIONS:
    -Release   Use release build (default: true)
    -SkipBuild Skip building binaries (use existing ones)
    -Help, -h  Show this help message

EXAMPLES:
    # Build binaries and create portable package (default)
    .\build\windows\build_portable.ps1

    # Create package using existing binaries
    .\build\windows\build_portable.ps1 -SkipBuild

OUTPUT:
    build\installer\marco-suite_alpha_<version>_windows_amd64.zip

STRUCTURE:
    MarcoPortable/
    |-- marco.exe
    |-- polo.exe
    |-- assets/              # All application assets
    |   |-- icons/
    |   |-- language/
    |   |-- themes/
    |   +-- settings_org.ron
    |-- config/              # User config (empty, created on first run)
    |-- data/                # User data (empty, created on first run)
    |-- LICENSE
    +-- README.txt

NOTE: The portable version automatically detects it's in portable mode
      and stores config/data in its own directory (not %LOCALAPPDATA%).
"@
    exit 0
}

Write-Host "=====================================" -ForegroundColor Cyan
Write-Host "Marco Portable Build for Windows" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host ""

# Ensure we're in project root
if (-not (Test-Path "Cargo.toml")) {
    Write-Error "ERROR: Must run from project root directory"
    exit 1
}

$projectRoot = Get-Location

# Get version from version.json
$versionFile = Join-Path $projectRoot 'build\version.json'
if (Test-Path $versionFile) {
    $json = Get-Content $versionFile -Raw | ConvertFrom-Json
    $version = $json.windows.marco
    Write-Host "Version: $version" -ForegroundColor Cyan
} else {
    $version = "0.0.0"
    Write-Warning "Could not find build/version.json; using version: $version"
}

# Setup paths
$buildType = if ($Release) { "release" } else { "debug" }
$targetDir = Join-Path $projectRoot "target\windows\x86_64-pc-windows-gnu\$buildType"
$marcoExe = Join-Path $targetDir "marco.exe"
$poloExe = Join-Path $targetDir "polo.exe"

Write-Host "Build configuration:" -ForegroundColor Cyan
Write-Host "  Build type: $buildType" -ForegroundColor Gray
Write-Host "  Target dir: $targetDir" -ForegroundColor Gray
Write-Host ""

# Build binaries (or skip if requested)
Write-Host "[1/4] Building binaries..." -ForegroundColor Cyan

if ($SkipBuild) {
    Write-Host "  Skipping build (using existing binaries)" -ForegroundColor Yellow
    
    if (-not (Test-Path $marcoExe)) {
        Write-Error "Marco binary not found at: $marcoExe"
        Write-Error "Run without -SkipBuild to build binaries"
        exit 1
    }
    
    if (-not (Test-Path $poloExe)) {
        Write-Warning "Polo binary not found at: $poloExe"
        Write-Warning "Package will only include marco.exe"
    }
} else {
    Write-Host "  Building Marco and Polo (release, workspace)..." -ForegroundColor Gray
    
    $buildArgs = @('build', '--workspace', '--target', 'x86_64-pc-windows-gnu', '--target-dir', 'target/windows')
    if ($Release) {
        $buildArgs += '--release'
    }
    
    & cargo @buildArgs
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Build failed"
        exit 1
    }
    
    if (-not (Test-Path $marcoExe)) {
        Write-Error "Build succeeded but marco.exe not found at: $marcoExe"
        exit 1
    }
    
    Write-Host "  OK Build complete" -ForegroundColor Green
}

# Verify binaries
if (-not (Test-Path $marcoExe)) {
    Write-Error "Marco binary not found: $marcoExe"
    exit 1
}

if (-not (Test-Path $poloExe)) {
    Write-Warning "Polo binary not found - will be excluded from package"
    $poloExe = $null
}

# Create staging directory
$stagingName = "marco-suite_alpha_${version}_windows_amd64"
$stagingRoot = Join-Path $projectRoot "build\windows\temp\$stagingName"

if (Test-Path $stagingRoot) {
    Write-Host "Cleaning existing staging directory..." -ForegroundColor Yellow
    Remove-Item $stagingRoot -Recurse -Force
}

Write-Host ""
Write-Host "[2/4] Creating portable package structure..." -ForegroundColor Cyan
New-Item -ItemType Directory -Path $stagingRoot -Force | Out-Null

# Copy binaries
Write-Host "  Copying binaries..." -ForegroundColor Gray
Copy-Item -Path $marcoExe -Destination $stagingRoot -Force
Write-Host "    + marco.exe" -ForegroundColor Green

if ($poloExe -and (Test-Path $poloExe)) {
    Copy-Item -Path $poloExe -Destination $stagingRoot -Force
    Write-Host "    + polo.exe" -ForegroundColor Green
}

# Copy assets directory (this is what the app looks for in portable mode)
Write-Host "  Copying assets..." -ForegroundColor Gray
$assetsSource = Join-Path $projectRoot "assets"
if (-not (Test-Path $assetsSource)) {
    Write-Error "Assets directory not found at: $assetsSource"
    exit 1
}

$assetsDest = Join-Path $stagingRoot "assets"
Copy-Item -Path $assetsSource -Destination $stagingRoot -Recurse -Force

# Remove settings_org.ron from assets (users should have clean config)
$settingsOrg = Join-Path $assetsDest "settings_org.ron"
if (Test-Path $settingsOrg) {
    Remove-Item $settingsOrg -Force
}

Write-Host "    + assets/ (icons, themes, languages)" -ForegroundColor Green

# Create empty config and data directories (portable mode uses these)
Write-Host "  Creating user directories..." -ForegroundColor Gray
New-Item -ItemType Directory -Path (Join-Path $stagingRoot "config") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $stagingRoot "data") -Force | Out-Null
Write-Host "    + config/ (will store user settings)" -ForegroundColor Green
Write-Host "    + data/ (will store user data)" -ForegroundColor Green

# Copy LICENSE and README
Write-Host "  Copying documentation..." -ForegroundColor Gray
$licensePath = Join-Path $projectRoot "LICENSE"
if (Test-Path $licensePath) {
    Copy-Item -Path $licensePath -Destination $stagingRoot -Force
    Write-Host "    + LICENSE" -ForegroundColor Green
}

# Create a portable-specific README
$portableReadme = @"
Marco Portable for Windows
===========================

Version: $version

This is a portable version of Marco that runs without installation.
All settings and data are stored in the 'config' and 'data' folders
next to the executable, making it perfect for USB drives.

Quick Start:
1. Double-click marco.exe to start the Marco editor
2. Double-click polo.exe to start the Polo viewer (lightweight)

Features:
- No installation required
- Runs from any location (including USB drives)
- Settings stored in .\config\
- User data stored in .\data\
- Includes all themes, icons, and language files

System Requirements:
- Windows 10 or later (x64)
- WebView2 runtime (will prompt to install if missing)
  Download: https://go.microsoft.com/fwlink/p/?LinkId=2124703

For more information:
- GitHub: https://github.com/marco-editor/marco
- Report issues: https://github.com/marco-editor/marco/issues

License:
See LICENSE file for terms of use.
"@
$readmePath = Join-Path $stagingRoot "README.txt"
$portableReadme | Out-File -FilePath $readmePath -Encoding UTF8
Write-Host "    + README.txt" -ForegroundColor Green

# Create manifest
$manifestPath = Join-Path $stagingRoot "MANIFEST.txt"
$versionFile = Join-Path $projectRoot 'build\version.json'
if (Test-Path $versionFile) {
    $json = Get-Content $versionFile -Raw | ConvertFrom-Json
    $manifest = @(
        "Marco Portable for Windows",
        "Version: $version",
        "Build: $buildType",
        "Portable: Yes",
        "",
        "Component Versions:",
        "  core:  $($json.windows.core)",
        "  marco: $($json.windows.marco)",
        "  polo:  $($json.windows.polo)",
        "",
        "Built: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')",
        "",
        "Package Contents:",
        "  - marco.exe (Markdown editor)",
        "  - polo.exe (Markdown viewer)",
        "  - assets/ (icons, themes, languages)",
        "  - config/ (user settings, created on first run)",
        "  - data/ (user data, created on first run)",
        "",
        "Portable Mode:",
        "This build automatically detects it is running in portable mode.",
        "All user data is stored in the package directory",
        "",
        "Debug Options:",
        "To enable debug features, edit config/settings.ron and set:",
        "  debug: Some(true),       // Enables debug menu in settings",
        "  log_to_file: Some(true), // Enables logging to log/ folder",
        "",
        "For more information:",
        "https://github.com/marco-editor/marco"
    )
    $manifest | Out-File -FilePath $manifestPath -Encoding UTF8
    Write-Host "    + MANIFEST.txt" -ForegroundColor Green
}

# Create zip file
Write-Host ""
Write-Host "[3/4] Creating zip archive..." -ForegroundColor Cyan

$installerDir = Join-Path $projectRoot "build\installer"
if (-not (Test-Path $installerDir)) {
    New-Item -ItemType Directory -Path $installerDir -Force | Out-Null
}

$zipName = "${stagingName}.zip"
$zipPath = Join-Path $installerDir $zipName

if (Test-Path $zipPath) {
    Remove-Item $zipPath -Force
}

# Compress using PowerShell (available on all Windows 10+)
$tempParent = Join-Path $projectRoot "build\windows\temp"
Compress-Archive -Path $stagingRoot -DestinationPath $zipPath -CompressionLevel Optimal

if (Test-Path $zipPath) {
    $size = (Get-Item $zipPath).Length / 1MB
    Write-Host "  + Created: $zipName" -ForegroundColor Green
    Write-Host "    Size: $([math]::Round($size, 2)) MB" -ForegroundColor Gray
} else {
    Write-Error "Failed to create zip file"
    exit 1
}

# Cleanup staging directory
Write-Host ""
Write-Host "[4/4] Cleaning up..." -ForegroundColor Cyan
Remove-Item $tempParent -Recurse -Force -ErrorAction SilentlyContinue
Write-Host "  + Removed staging directory" -ForegroundColor Green

# Summary
Write-Host ""
Write-Host "=====================================" -ForegroundColor Green
Write-Host "Portable Package Created!" -ForegroundColor Green
Write-Host "=====================================" -ForegroundColor Green
Write-Host ""
Write-Host "Package: $zipPath" -ForegroundColor Cyan
Write-Host "Size: $([math]::Round($size, 2)) MB" -ForegroundColor Cyan
Write-Host ""
Write-Host "To use:" -ForegroundColor Yellow
Write-Host "  1. Extract the zip to any location" -ForegroundColor Gray
Write-Host "  2. Run marco.exe or polo.exe" -ForegroundColor Gray
Write-Host "  3. Settings will be saved in the extracted folder" -ForegroundColor Gray
Write-Host ""
Write-Host "Perfect for:" -ForegroundColor Yellow
Write-Host "  • USB drives" -ForegroundColor Gray
Write-Host "  • Portable installations" -ForegroundColor Gray
Write-Host "  • Testing without installation" -ForegroundColor Gray
Write-Host "  • Shared network folders" -ForegroundColor Gray
Write-Host ""

exit 0
