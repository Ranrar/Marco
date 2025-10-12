#!/bin/bash
# User-local install script for Marco Markdown Editor and Polo Viewer (Linux)
# bash tests/install/local.sh
set -e

echo "Building Marco and Polo..."

# 1. Build the release binaries (marco and polo)
cargo build --release -p marco
cargo build --release -p polo

# 2. Create user bin directory if needed
mkdir -p "$HOME/.local/bin"

# 3. Copy binaries
echo "Installing binaries..."
cp target/release/marco "$HOME/.local/bin/marco"
cp target/release/polo "$HOME/.local/bin/polo"

# 4. Install .desktop files
echo "Installing desktop entries..."
mkdir -p "$HOME/.local/share/applications"
cp tests/install/marco.desktop "$HOME/.local/share/applications/marco.desktop"
cp tests/install/polo.desktop "$HOME/.local/share/applications/polo.desktop"

# 5. Install icon at multiple resolutions for best display quality
echo "Installing icons..."
# Install 256x256 icon (scaled from high-quality source)
mkdir -p "$HOME/.local/share/icons/hicolor/256x256/apps"
convert assets/icons/icon.png -resize 256x256 "$HOME/.local/share/icons/hicolor/256x256/apps/marco.png"
cp "$HOME/.local/share/icons/hicolor/256x256/apps/marco.png" "$HOME/.local/share/icons/hicolor/256x256/apps/polo.png"

# Install 128x128 icon
mkdir -p "$HOME/.local/share/icons/hicolor/128x128/apps" 
convert assets/icons/icon.png -resize 128x128 "$HOME/.local/share/icons/hicolor/128x128/apps/marco.png"
cp "$HOME/.local/share/icons/hicolor/128x128/apps/marco.png" "$HOME/.local/share/icons/hicolor/128x128/apps/polo.png"

# Install 64x64 icon
mkdir -p "$HOME/.local/share/icons/hicolor/64x64/apps"
cp assets/icons/favicon.png "$HOME/.local/share/icons/hicolor/64x64/apps/marco.png"
cp assets/icons/favicon.png "$HOME/.local/share/icons/hicolor/64x64/apps/polo.png"

# Install 48x48 icon  
mkdir -p "$HOME/.local/share/icons/hicolor/48x48/apps"
convert assets/icons/favicon.png -resize 48x48 "$HOME/.local/share/icons/hicolor/48x48/apps/marco.png"
cp "$HOME/.local/share/icons/hicolor/48x48/apps/marco.png" "$HOME/.local/share/icons/hicolor/48x48/apps/polo.png"

# Install 48x48 icon  
mkdir -p "$HOME/.local/share/icons/hicolor/48x48/apps"
convert assets/icons/favicon.png -resize 48x48 "$HOME/.local/share/icons/hicolor/48x48/apps/marco.png"
cp "$HOME/.local/share/icons/hicolor/48x48/apps/marco.png" "$HOME/.local/share/icons/hicolor/48x48/apps/polo.png"

# 6. Install settings
echo "Installing settings..."
mkdir -p "$HOME/.local/share/marco/"
# Remove old settings
rm -f "$HOME/.local/share/marco/settings.ron"
cp assets/settings_org.ron "$HOME/.local/share/marco/settings.ron"

# 7. Install themes
echo "Installing themes..."
mkdir -p "$HOME/.local/share/marco/themes"
# Remove old themes
rm -rf "$HOME/.local/share/marco/themes/"*
cp -r assets/themes/ "$HOME/.local/share/marco/"

# 8. Install fonts
echo "Installing fonts..."
mkdir -p "$HOME/.local/share/marco/fonts"
# Remove old fonts
rm -rf "$HOME/.local/share/marco/fonts/"*
cp -r assets/fonts/ "$HOME/.local/share/marco/"

# 9. Install icons to asset directory for application use
echo "Installing application icons..."
mkdir -p "$HOME/.local/share/marco/icons"
cp assets/icons/favicon.png "$HOME/.local/share/marco/icons/"
cp assets/icons/icon.png "$HOME/.local/share/marco/icons/"

# 10. Install language files
echo "Installing language files..."
mkdir -p "$HOME/.local/share/marco/language"
# Remove old language files
rm -rf "$HOME/.local/share/marco/language/"*
cp -r assets/language/ "$HOME/.local/share/marco/"

# 10. Install language files
echo "Installing language files..."
mkdir -p "$HOME/.local/share/marco/language"
# Remove old language files
rm -rf "$HOME/.local/share/marco/language/"*
cp -r assets/language/ "$HOME/.local/share/marco/"

# 11. Install user guide and logo
echo "Installing documentation..."
mkdir -p "$HOME/.local/share/marco/doc"
# Remove old user guide and logo if they exist
rm -f "$HOME/.local/share/marco/doc/user_guide.md"
rm -f "$HOME/.local/share/marco/doc/logo.png"
cp -f "documentation/user guide/user_guide.md" "$HOME/.local/share/marco/doc/"
cp -f "documentation/user guide/logo.png" "$HOME/.local/share/marco/doc/"

# 12. Update desktop database
echo "Updating desktop database..."
update-desktop-database "$HOME/.local/share/applications/"

echo ""
echo "========================================="
echo "Installation complete!"
echo "========================================="
echo "Marco (editor) installed to: $HOME/.local/bin/marco"
echo "Polo (viewer) installed to: $HOME/.local/bin/polo"
echo "Assets installed to: $HOME/.local/share/marco/"
echo ""
echo "You may need to log out and back in for icons to be fully recognized."
echo "You can now launch:"
echo "  - Marco from your application menu or with 'marco' in a terminal"
echo "  - Polo from your application menu or with 'polo' in a terminal"
echo "========================================="
