#!/bin/bash
# User-local install script for Marco Markdown Editor and Polo Viewer (Linux)
# 
# This script installs Marco and Polo to the user's local directories according
# to the XDG Base Directory specification and the new paths system architecture.
#
# Installation locations:
#   - Binaries:         ~/.local/bin/
#   - Desktop entries:  ~/.local/share/applications/
#   - System icons:     ~/.local/share/icons/hicolor/
#   - Assets:           ~/.local/share/marco/
#   - Config:           ~/.config/marco/ (created on first run)
#   - Data/Cache:       ~/.local/share/marco/data/ (created on first run)
#
# Usage: bash tests/install/install.sh

set -e

# Show help if requested
if [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
    cat << 'EOF'
Marco & Polo User-Local Installer

USAGE:
    bash tests/install/install.sh

DESCRIPTION:
    Installs Marco (markdown editor) and Polo (markdown viewer) to the
    user's local directories following XDG Base Directory specification.

INSTALLATION LOCATIONS:
    ~/.local/bin/marco              Marco editor binary
    ~/.local/bin/polo               Polo viewer binary
    ~/.local/share/marco/           Shared assets (themes, fonts, etc.)
    ~/.local/share/applications/    Desktop entries
    ~/.local/share/icons/hicolor/   System icons
    ~/.config/marco/                Marco config (created on first run)
    ~/.config/polo/                 Polo config (created on first run)

REQUIREMENTS:
    - GTK4 and WebKit6 libraries
    - ImageMagick (optional, for icon scaling)
    - update-desktop-database (optional)
    - gtk-update-icon-cache (optional)

OPTIONS:
    -h, --help    Show this help message

EXAMPLES:
    # Install Marco and Polo
    bash tests/install/install.sh

    # Uninstall later
    bash tests/install/uninstall.sh

SEE ALSO:
    tests/install/README.md - Complete installation guide
    tests/install/uninstall.sh - Uninstaller script

EOF
    exit 0
fi

echo "========================================="
echo "Marco & Polo User-Local Installation"
echo "========================================="
echo ""

# Check for required tools
if ! command -v convert &> /dev/null; then
    echo "Warning: ImageMagick 'convert' command not found."
    echo "Icon scaling will be skipped. Install with: sudo apt install imagemagick"
    CONVERT_AVAILABLE=false
else
    CONVERT_AVAILABLE=true
fi

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo (Rust toolchain) not found."
    echo "Install Rust from: https://rustup.rs/"
    exit 1
fi

# Check if binaries already exist
if [ -f "target/release/marco" ] && [ -f "target/release/polo" ]; then
    echo "Found existing release binaries."
    read -p "Rebuild binaries? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Building Marco and Polo..."
        cargo build --release -p marco
        cargo build --release -p polo
    else
        echo "Using existing binaries."
    fi
else
    echo "Building Marco and Polo (this may take a few minutes)..."
    cargo build --release -p marco
    cargo build --release -p polo
fi

# 2. Create user bin directory if needed
mkdir -p "$HOME/.local/bin"

# 3. Copy binaries
echo "Installing binaries..."
cp target/release/marco "$HOME/.local/bin/marco"
cp target/release/polo "$HOME/.local/bin/polo"
chmod +x "$HOME/.local/bin/marco"
chmod +x "$HOME/.local/bin/polo"

# 4. Install .desktop files
echo "Installing desktop entries..."
mkdir -p "$HOME/.local/share/applications"
cp tests/install/marco.desktop "$HOME/.local/share/applications/marco.desktop"
cp tests/install/polo.desktop "$HOME/.local/share/applications/polo.desktop"

# 5. Install system icons at multiple resolutions for desktop integration
echo "Installing system icons..."
# These icons are for the desktop environment to display in menus/launchers

if [ "$CONVERT_AVAILABLE" = true ]; then
    # Install 256x256 icon (scaled from high-quality source)
    mkdir -p "$HOME/.local/share/icons/hicolor/256x256/apps"
    convert assets/icons/icon_662x662.png -resize 256x256 "$HOME/.local/share/icons/hicolor/256x256/apps/marco.png"
    cp "$HOME/.local/share/icons/hicolor/256x256/apps/marco.png" "$HOME/.local/share/icons/hicolor/256x256/apps/polo.png"

    # Install 128x128 icon
    mkdir -p "$HOME/.local/share/icons/hicolor/128x128/apps" 
    convert assets/icons/icon_662x662.png -resize 128x128 "$HOME/.local/share/icons/hicolor/128x128/apps/marco.png"
    cp "$HOME/.local/share/icons/hicolor/128x128/apps/marco.png" "$HOME/.local/share/icons/hicolor/128x128/apps/polo.png"

    # Install 48x48 icon  
    mkdir -p "$HOME/.local/share/icons/hicolor/48x48/apps"
    convert assets/icons/favicon.png -resize 48x48 "$HOME/.local/share/icons/hicolor/48x48/apps/marco.png"
    cp "$HOME/.local/share/icons/hicolor/48x48/apps/marco.png" "$HOME/.local/share/icons/hicolor/48x48/apps/polo.png"
fi

# Install 64x64 icon (no conversion needed)
mkdir -p "$HOME/.local/share/icons/hicolor/64x64/apps"
cp assets/icons/favicon.png "$HOME/.local/share/icons/hicolor/64x64/apps/marco.png"
cp assets/icons/favicon.png "$HOME/.local/share/icons/hicolor/64x64/apps/polo.png"

# 6. Install shared assets to ~/.local/share/marco/ (asset_root)
# This structure matches the new paths system architecture
echo "Installing shared assets..."
ASSET_ROOT="$HOME/.local/share/marco"

# Create clean asset root
echo "  Creating asset directory structure..."
mkdir -p "$ASSET_ROOT"

# Install fonts (IcoMoon icon font, UI fonts)
echo "  Installing fonts..."
rm -rf "$ASSET_ROOT/fonts"
cp -r assets/fonts "$ASSET_ROOT/"

# Install icons (application icons used by Marco/Polo at runtime)
echo "  Installing application icons..."
rm -rf "$ASSET_ROOT/icons"
cp -r assets/icons "$ASSET_ROOT/"

# Install themes (editor themes + HTML preview themes)
echo "  Installing themes..."
rm -rf "$ASSET_ROOT/themes"
cp -r assets/themes "$ASSET_ROOT/"

# Install language files (translations)
echo "  Installing language files..."
rm -rf "$ASSET_ROOT/language"
cp -r assets/language "$ASSET_ROOT/"

# Install documentation (user guide, screenshots)
echo "  Installing documentation..."
rm -rf "$ASSET_ROOT/documentation"
mkdir -p "$ASSET_ROOT/documentation"
cp -r documentation/* "$ASSET_ROOT/documentation/"

# Install default settings template
echo "  Installing default settings..."
if [ -f "assets/settings_org.ron" ]; then
    cp assets/settings_org.ron "$ASSET_ROOT/settings.ron"
elif [ -f "assets/settings.ron" ]; then
    cp assets/settings.ron "$ASSET_ROOT/settings.ron"
fi

# 7. Update desktop database and icon cache
echo "Updating desktop database..."
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database "$HOME/.local/share/applications/" 2>/dev/null || true
fi

echo "Updating icon cache..."
if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor/" 2>/dev/null || true
fi

# 8. Verify installation
echo ""
echo "========================================="
echo "Installation Complete!"
echo "========================================="
echo ""
echo "✓ Binaries installed:"
echo "    Marco (editor): $HOME/.local/bin/marco"
echo "    Polo (viewer):  $HOME/.local/bin/polo"
echo ""
echo "✓ Shared assets installed:"
echo "    Asset root:     $ASSET_ROOT"
echo "    - fonts/        (IcoMoon icon font, UI fonts)"
echo "    - icons/        (application icons)"
echo "    - themes/       (editor + preview themes)"
echo "    - language/     (translations)"
echo "    - documentation/ (user guide)"
echo ""
echo "✓ Desktop integration:"
echo "    Desktop entries: $HOME/.local/share/applications/"
echo "    System icons:    $HOME/.local/share/icons/hicolor/"
echo ""
echo "⚠ Notes:"
echo "  - You may need to log out and back in for icons to appear"
echo "  - Config will be created at: ~/.config/marco/ (on first run)"
echo "  - User data/cache at: ~/.local/share/marco/data/ (on first run)"
echo ""
echo "Launch with:"
echo "  marco         # Markdown editor"
echo "  marco file.md # Open specific file"
echo "  polo          # Markdown viewer"
echo "  polo file.md  # View specific file"
echo ""
echo "To uninstall:"
echo "  bash tests/install/uninstall.sh"
echo ""
echo "========================================="
