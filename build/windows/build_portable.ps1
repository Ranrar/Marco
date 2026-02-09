# Windows Portable Build Script
# Creates a simple zip file with Marco, Polo, and assets for portable deployment

param(
    [switch]$Release = $true,
    [switch]$SkipBuild,
    [string]$Msys2Root = "C:\\msys64",
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

function Assert-Msys2Ucrt64 {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Root
    )

    if (-not (Test-Path $Root)) {
        Write-Host "ERROR: MSYS2 is not installed at: $Root" -ForegroundColor Red
        Write-Host "Please install MSYS2 from: https://www.msys2.org/" -ForegroundColor Yellow
        Write-Host "Then (in an MSYS2 UCRT64 shell) run:" -ForegroundColor Yellow
        Write-Host "  pacman -Syu" -ForegroundColor Yellow
        Write-Host "  pacman -S --needed mingw-w64-ucrt-x86_64-gtk4 mingw-w64-ucrt-x86_64-gtksourceview5 mingw-w64-ucrt-x86_64-librsvg mingw-w64-ucrt-x86_64-cairo mingw-w64-ucrt-x86_64-gdk-pixbuf2 mingw-w64-ucrt-x86_64-pkg-config mingw-w64-ucrt-x86_64-gcc mingw-w64-ucrt-x86_64-binutils" -ForegroundColor Yellow
        throw "MSYS2 not found"
    }

    $pkgConfigExe = Join-Path $Root "ucrt64\\bin\\pkg-config.exe"
    $pkgConfExe = Join-Path $Root "ucrt64\\bin\\pkgconf.exe"
    $usrPkgConfigExe = Join-Path $Root "usr\\bin\\pkg-config.exe"

    if (-not (Test-Path $pkgConfigExe) -and -not (Test-Path $pkgConfExe) -and -not (Test-Path $usrPkgConfigExe)) {
        Write-Host "ERROR: MSYS2 pkg-config (or pkgconf) not found under: $Root" -ForegroundColor Red
        Write-Host "Tried:" -ForegroundColor Yellow
        Write-Host "  $pkgConfigExe" -ForegroundColor Yellow
        Write-Host "  $pkgConfExe" -ForegroundColor Yellow
        Write-Host "  $usrPkgConfigExe" -ForegroundColor Yellow
        Write-Host "Install the required MSYS2 packages (see above) and re-run." -ForegroundColor Yellow
        throw "pkg-config missing"
    }
}

function Copy-Msys2GtkRuntime {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Root,

        [Parameter(Mandatory = $true)]
        [string]$StagingRoot,

        [Parameter(Mandatory = $true)]
        [string[]]$EntryBinaries
    )

    $ucrtBin = Join-Path $Root "ucrt64\\bin"
    $usrBin = Join-Path $Root "usr\\bin"

    $objdumpExe = Join-Path $ucrtBin "objdump.exe"
    if (-not (Test-Path $objdumpExe)) {
        throw "MSYS2 binutils not found (expected $objdumpExe). Install mingw-w64-ucrt-x86_64-binutils."
    }

    $ignoreDlls = @(
        # Windows system DLLs
        "KERNEL32.DLL", "USER32.DLL", "GDI32.DLL", "ADVAPI32.DLL", "SHELL32.DLL", "OLE32.DLL",
        "OLEAUT32.DLL", "COMDLG32.DLL", "COMCTL32.DLL", "WS2_32.DLL", "IPHLPAPI.DLL",
        "CRYPT32.DLL", "SETUPAPI.DLL", "SHLWAPI.DLL", "VERSION.DLL", "WINMM.DLL",
        "IMM32.DLL", "UXTHEME.DLL", "DWMAPI.DLL", "UCRTBASE.DLL", "MSVCRT.DLL",
        "VCRUNTIME140.DLL", "VCRUNTIME140_1.DLL", "MSVCP140.DLL",
        "D3D11.DLL", "DXGI.DLL", "DWRITE.DLL", "D2D1.DLL", "WINHTTP.DLL", "BCRYPT.DLL",
        "NTDLL.DLL"
    ) | ForEach-Object { $_.ToUpperInvariant() }

    function Get-DllImports {
        param([Parameter(Mandatory = $true)][string]$File)

        $out = & $objdumpExe -p $File 2>$null
        if ($LASTEXITCODE -ne 0) {
            throw "objdump failed for: $File"
        }

        $dlls = @()
        foreach ($line in $out) {
            if ($line -match 'DLL Name:\s*(.+)$') {
                $name = $Matches[1].Trim()
                if ($name) {
                    $dlls += $name
                }
            }
        }
        return $dlls | Sort-Object -Unique
    }

    function Resolve-Dll {
        param([Parameter(Mandatory = $true)][string]$DllName)

        $candidate1 = Join-Path $ucrtBin $DllName
        if (Test-Path $candidate1) { return $candidate1 }

        $candidate2 = Join-Path $usrBin $DllName
        if (Test-Path $candidate2) { return $candidate2 }

        return $null
    }

    $queue = [System.Collections.Generic.Queue[string]]::new()
    $visited = [System.Collections.Generic.HashSet[string]]::new()
    $copySet = [System.Collections.Generic.Dictionary[string, string]]::new()

    foreach ($bin in $EntryBinaries) {
        if ($bin -and (Test-Path $bin)) {
            $queue.Enqueue($bin)
        }
    }

    while ($queue.Count -gt 0) {
        $current = $queue.Dequeue()
        if (-not $visited.Add($current)) {
            continue
        }

        $imports = Get-DllImports -File $current
        foreach ($dll in $imports) {
            $upper = $dll.ToUpperInvariant()
            if ($ignoreDlls -contains $upper) {
                continue
            }

            $resolved = Resolve-Dll -DllName $dll
            if (-not $resolved) {
                continue
            }

            if (-not $copySet.ContainsKey($upper)) {
                $copySet[$upper] = $resolved
                $queue.Enqueue($resolved)
            }
        }
    }

    if ($copySet.Count -eq 0) {
        Write-Warning "No MSYS2 runtime DLLs were discovered to copy. The binaries may still rely on MSYS2 being on PATH."
        return
    }

    Write-Host "  Copying MSYS2 runtime DLLs..." -ForegroundColor Gray
    foreach ($src in ($copySet.Values | Sort-Object -Unique)) {
        $dest = Join-Path $StagingRoot ([System.IO.Path]::GetFileName($src))
        Copy-Item -Path $src -Destination $dest -Force
    }
    Write-Host "    + Runtime DLLs: $($copySet.Count)" -ForegroundColor Green

    # GLib schemas (required for many GTK/GSettings lookups)
    $schemaSrcDir = Join-Path $Root "ucrt64\\share\\glib-2.0\\schemas"
    $schemaFile = Join-Path $schemaSrcDir "gschemas.compiled"
    if (Test-Path $schemaFile) {
        $schemaDestDir = Join-Path $StagingRoot "share\\glib-2.0\\schemas"
        New-Item -ItemType Directory -Path $schemaDestDir -Force | Out-Null
        Copy-Item -Path $schemaFile -Destination (Join-Path $schemaDestDir "gschemas.compiled") -Force
        Write-Host "    + GLib schemas (gschemas.compiled)" -ForegroundColor Green
    } else {
        Write-Warning "GLib schemas not found at: $schemaFile"
    }

    # GDK Pixbuf loaders (needed for images/SVGs depending on usage)
    $pixbufLoaderSrcDir = Join-Path $Root "ucrt64\\lib\\gdk-pixbuf-2.0\\2.10.0\\loaders"
    $pixbufCacheSrc = Join-Path $Root "ucrt64\\lib\\gdk-pixbuf-2.0\\2.10.0\\loaders.cache"
    if (Test-Path $pixbufLoaderSrcDir) {
        $pixbufLoaderDestDir = Join-Path $StagingRoot "lib\\gdk-pixbuf-2.0\\2.10.0\\loaders"
        New-Item -ItemType Directory -Path $pixbufLoaderDestDir -Force | Out-Null
        Copy-Item -Path (Join-Path $pixbufLoaderSrcDir "*.dll") -Destination $pixbufLoaderDestDir -Force -ErrorAction SilentlyContinue
        if (Test-Path $pixbufCacheSrc) {
            $pixbufCacheDestDir = Join-Path $StagingRoot "lib\\gdk-pixbuf-2.0\\2.10.0"
            New-Item -ItemType Directory -Path $pixbufCacheDestDir -Force | Out-Null
            Copy-Item -Path $pixbufCacheSrc -Destination (Join-Path $pixbufCacheDestDir "loaders.cache") -Force
        }
        Write-Host "    + GDK Pixbuf loaders" -ForegroundColor Green
    }
}

function Copy-WebView2Loader {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ProjectRoot,

        [Parameter(Mandatory = $true)]
        [string]$BuildType,

        [Parameter(Mandatory = $true)]
        [string]$StagingRoot
    )

    # wry/WebView2 needs WebView2Loader.dll next to the executable.
    # The webview2-com-sys crate provides redistributable loader DLLs in its build output.
    $buildOutRoot = Join-Path $ProjectRoot "target\\windows\\x86_64-pc-windows-gnu\\$BuildType\\build"
    if (-not (Test-Path $buildOutRoot)) {
        Write-Warning "WebView2 build output directory not found: $buildOutRoot"
        return
    }

    $candidates = Get-ChildItem -Path $buildOutRoot -Recurse -Filter "WebView2Loader.dll" -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -match '\\out\\x64\\WebView2Loader\.dll$' } |
        Sort-Object LastWriteTime -Descending

    if (-not $candidates -or $candidates.Count -eq 0) {
        Write-Warning "WebView2Loader.dll (x64) not found under: $buildOutRoot"
        return
    }

    $src = $candidates[0].FullName
    $dest = Join-Path $StagingRoot "WebView2Loader.dll"
    Copy-Item -Path $src -Destination $dest -Force
    Write-Host "    + WebView2Loader.dll" -ForegroundColor Green
}

function Ensure-GnuTargetInstalled {
    $installedTargets = & rustup target list --installed
    if ($installedTargets -notcontains "x86_64-pc-windows-gnu") {
        Write-Host "Installing Rust target x86_64-pc-windows-gnu..." -ForegroundColor Yellow
        & rustup target add x86_64-pc-windows-gnu
    }
}

function Set-Msys2Ucrt64Env {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Root
    )

    $env:PKG_CONFIG_ALLOW_CROSS = "1"
    $env:PKG_CONFIG_PATH = (Join-Path $Root "ucrt64\\lib\\pkgconfig")
    $env:PATH = "$(Join-Path $Root 'ucrt64\\bin');$(Join-Path $Root 'usr\\bin');$env:PATH"

    # Prefer an explicit PKG_CONFIG to avoid relying on shim names.
    # In MSYS2 UCRT64, the implementation is often `pkgconf.exe`.
    $candidatePkgConfig = @(
        (Join-Path $Root "ucrt64\\bin\\pkg-config.exe"),
        (Join-Path $Root "ucrt64\\bin\\pkgconf.exe"),
        (Join-Path $Root "usr\\bin\\pkg-config.exe")
    ) | Where-Object { Test-Path $_ } | Select-Object -First 1

    if ($candidatePkgConfig) {
        $env:PKG_CONFIG = $candidatePkgConfig
    }
}

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

    # This portable build uses the GNU target to interop with MSYS2-provided GTK/GLib.
    # Ensure MSYS2 is available and set up the UCRT64 environment so pkg-config works.
    Assert-Msys2Ucrt64 -Root $Msys2Root
    Ensure-GnuTargetInstalled
    Set-Msys2Ucrt64Env -Root $Msys2Root

    Write-Host "  Using MSYS2 root: $Msys2Root" -ForegroundColor Gray
    Write-Host "  PKG_CONFIG_PATH: $env:PKG_CONFIG_PATH" -ForegroundColor Gray
    
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

# Bundle WebView2Loader.dll (required by wry/WebView2 on Windows)
Write-Host "  Bundling WebView2 loader..." -ForegroundColor Gray
Copy-WebView2Loader -ProjectRoot $projectRoot -BuildType $buildType -StagingRoot $stagingRoot

# Bundle MSYS2 GTK runtime so the portable zip runs on machines without MSYS2.
# Without this, users will see missing DLL errors like libgio-2.0-0.dll.
Write-Host "  Bundling GTK/GLib runtime (MSYS2 UCRT64)..." -ForegroundColor Gray
try {
    Assert-Msys2Ucrt64 -Root $Msys2Root
    Copy-Msys2GtkRuntime -Root $Msys2Root -StagingRoot $stagingRoot -EntryBinaries @(
        (Join-Path $stagingRoot "marco.exe"),
        (Join-Path $stagingRoot "polo.exe")
    )
} catch {
    Write-Warning "Could not bundle MSYS2 runtime: $($_.Exception.Message)"
    Write-Warning "The portable package may require MSYS2 UCRT64 bin on PATH (e.g. C:\\msys64\\ucrt64\\bin)."
}

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
