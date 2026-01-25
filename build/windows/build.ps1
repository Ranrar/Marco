# Windows Build Script for Marco
# PowerShell script to build Marco on Windows using MSYS2/MinGW

param(
    [switch]$Release,
    [switch]$Check,
    [switch]$Clean,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

function Show-Help {
    Write-Host @"
Marco Windows Build Script

USAGE:
    .\build\windows\build.ps1 [OPTIONS]

OPTIONS:
    -Release    Build in release mode (optimized)
    -Check      Only check compilation (no binary output)
    -Clean      Clean build artifacts before building
    -Help       Show this help message

EXAMPLES:
    .\build\windows\build.ps1 -Release
    .\build\windows\build.ps1 -Check
    .\build\windows\build.ps1 -Clean -Release

REQUIREMENTS:
    - MSYS2 with MinGW-w64 toolchain
    - GTK4 development libraries
    - Rust toolchain (1.90.0+)
    - WebView2 runtime (Windows 10/11)

NOTES:
    This builds Marco with wry (WebView2) on Windows.
    For Linux builds, use build/linux/build_deb.sh
"@
}

if ($Help) {
    Show-Help
    exit 0
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Marco Windows Build" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check if we're in the project root
if (-not (Test-Path "Cargo.toml")) {
    Write-Host "ERROR: Must run from project root directory" -ForegroundColor Red
    exit 1
}

# Clean if requested
if ($Clean) {
    Write-Host "Cleaning build artifacts..." -ForegroundColor Yellow
    cargo clean
    Write-Host "Clean complete" -ForegroundColor Green
}

# Build command
$BuildArgs = @("build", "-p", "marco")

if ($Release) {
    $BuildArgs += "--release"
    Write-Host "Building in RELEASE mode..." -ForegroundColor Green
} else {
    Write-Host "Building in DEBUG mode..." -ForegroundColor Yellow
}

# Add Windows target feature flag
$BuildArgs += "--target-dir"
$BuildArgs += "target/windows"

if ($Check) {
    Write-Host "Running cargo check (Windows target)..." -ForegroundColor Yellow
    cargo check -p marco --target-dir target/windows
} else {
    Write-Host "Building Marco (Windows with wry/WebView2)..." -ForegroundColor Yellow
    & cargo @BuildArgs
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "========================================" -ForegroundColor Green
        Write-Host "Build successful!" -ForegroundColor Green
        Write-Host "========================================" -ForegroundColor Green
        
        $BinaryPath = if ($Release) { "target/windows/release/marco.exe" } else { "target/windows/debug/marco.exe" }
        if (Test-Path $BinaryPath) {
            Write-Host ""
            Write-Host "Binary location: $BinaryPath" -ForegroundColor Cyan
            $Size = (Get-Item $BinaryPath).Length / 1MB
            Write-Host "Binary size: $([math]::Round($Size, 2)) MB" -ForegroundColor Cyan
        }
    } else {
        Write-Host ""
        Write-Host "Build failed with exit code: $LASTEXITCODE" -ForegroundColor Red
        exit $LASTEXITCODE
    }
}
