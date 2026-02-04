#!/usr/bin/env pwsh
# Local testing script for Windows CI workflow
# Replicates the steps from .github/workflows/ci-windows.yml

param(
    [switch]$SkipTests,
    [switch]$Clean
)

$ErrorActionPreference = "Stop"

Write-Host "=== Local Windows CI Test ===" -ForegroundColor Cyan

# Check if MSYS2 is installed
if (-not (Test-Path "C:\msys64")) {
    Write-Host "ERROR: MSYS2 is not installed at C:\msys64" -ForegroundColor Red
    Write-Host "Please install MSYS2 from: https://www.msys2.org/" -ForegroundColor Yellow
    Write-Host "Then run: pacman -S mingw-w64-ucrt-x86_64-gtk4 mingw-w64-ucrt-x86_64-gtksourceview5 mingw-w64-ucrt-x86_64-librsvg mingw-w64-ucrt-x86_64-cairo mingw-w64-ucrt-x86_64-gdk-pixbuf2 mingw-w64-ucrt-x86_64-pkg-config mingw-w64-ucrt-x86_64-gcc mingw-w64-ucrt-x86_64-binutils" -ForegroundColor Yellow
    exit 1
}

# 1. Check Rust toolchain
Write-Host "`n[1/6] Checking Rust toolchain..." -ForegroundColor Green
rustc --version
cargo --version

# 2. Check/Install GNU target
Write-Host "`n[2/6] Checking GNU target..." -ForegroundColor Green
$targets = rustup target list --installed
if ($targets -notcontains "x86_64-pc-windows-gnu") {
    Write-Host "Installing x86_64-pc-windows-gnu target..."
    rustup target add x86_64-pc-windows-gnu
} else {
    Write-Host "GNU target already installed"
}

# 3. Setup environment variables
Write-Host "`n[3/6] Setting up build environment..." -ForegroundColor Green
$env:PKG_CONFIG_PATH = "C:\msys64\ucrt64\lib\pkgconfig"
$env:PATH = "C:\msys64\ucrt64\bin;C:\msys64\usr\bin;$env:PATH"

# Verify pkg-config is accessible
Write-Host "Testing pkg-config..."
try {
    $pkgConfigVersion = & "C:\msys64\ucrt64\bin\pkg-config.exe" --version 2>&1
    Write-Host "pkg-config version: $pkgConfigVersion"
} catch {
    Write-Host "ERROR: pkg-config not found in MSYS2" -ForegroundColor Red
    exit 1
}

# 4. Clean if requested
if ($Clean) {
    Write-Host "`n[4/6] Cleaning build artifacts..." -ForegroundColor Green
    if (Test-Path "target\windows") {
        Remove-Item -Recurse -Force "target\windows"
        Write-Host "Cleaned target\windows directory"
    }
} else {
    Write-Host "`n[4/6] Skipping clean (use -Clean to clean)" -ForegroundColor Green
}

# 5. Build workspace using MSYS2 shell
Write-Host "`n[5/6] Building workspace (GNU target)..." -ForegroundColor Green
Write-Host "This will run in MSYS2 UCRT64 environment..." -ForegroundColor Yellow

$msys2Script = @"
# Add Windows Cargo to PATH (from rustup)
export PATH="`$HOME/.cargo/bin:/c/Users/`$USERNAME/.cargo/bin:`$PATH"

# Allow pkg-config to work with GNU target
export PKG_CONFIG_ALLOW_CROSS=1

# Verify environment
echo "=== Environment Check ==="
echo "pkg-config version:"
pkg-config --version
echo "PKG_CONFIG_PATH: `$PKG_CONFIG_PATH"
echo "PKG_CONFIG_ALLOW_CROSS: `$PKG_CONFIG_ALLOW_CROSS"
echo "Cargo version:"
cargo --version
echo "Rustc version:"
rustc --version

echo ""
echo "=== Building Workspace ==="
cargo build --workspace --locked --target x86_64-pc-windows-gnu --target-dir target/windows
"@

# Write script to temp file (UTF-8 without BOM for bash compatibility)
$tempScript = Join-Path $env:TEMP "msys2-build-script.sh"
$utf8NoBom = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllText($tempScript, $msys2Script, $utf8NoBom)

# Run the build in MSYS2 UCRT64 environment
& "C:\msys64\usr\bin\bash.exe" -l -c "export MSYSTEM=UCRT64; export PKG_CONFIG_PATH=/ucrt64/lib/pkgconfig; source /etc/profile; cd '$(Get-Location | ForEach-Object { $_.Path -replace '\\', '/' -replace '^C:', '/c' })'; bash '$($tempScript -replace '\\', '/' -replace '^C:', '/c')'"

if ($LASTEXITCODE -ne 0) {
    Write-Host "`nBuild failed with exit code: $LASTEXITCODE" -ForegroundColor Red
    Remove-Item -Force $tempScript
    exit $LASTEXITCODE
}

Remove-Item -Force $tempScript

# 6. Run tests
if (-not $SkipTests) {
    Write-Host "`n[6/6] Running tests (core library)..." -ForegroundColor Green
    
    $testScript = @"
# Add Windows Cargo to PATH
export PATH="`$HOME/.cargo/bin:/c/Users/`$USERNAME/.cargo/bin:`$PATH"

echo "=== Running Core Library Tests ==="
cargo test -p core --lib --locked
"@
    
    $tempTestScript = Join-Path $env:TEMP "msys2-test-script.sh"
    $utf8NoBom = New-Object System.Text.UTF8Encoding $false
    [System.IO.File]::WriteAllText($tempTestScript, $testScript, $utf8NoBom)
    
    & "C:\msys64\usr\bin\bash.exe" -l -c "export MSYSTEM=UCRT64; export PKG_CONFIG_PATH=/ucrt64/lib/pkgconfig; source /etc/profile; cd '$(Get-Location | ForEach-Object { $_.Path -replace '\\', '/' -replace '^C:', '/c' })'; bash '$($tempTestScript -replace '\\', '/' -replace '^C:', '/c')'"
    
    $testExitCode = $LASTEXITCODE
    Remove-Item -Force $tempTestScript
    
    if ($testExitCode -ne 0) {
        Write-Host "`nTests failed with exit code: $testExitCode" -ForegroundColor Red
        exit $testExitCode
    }
} else {
    Write-Host "`n[6/6] Skipping tests (use without -SkipTests to run)" -ForegroundColor Yellow
}

Write-Host "`n=== CI Test Complete ===" -ForegroundColor Green
Write-Host "All steps passed successfully!" -ForegroundColor Green
