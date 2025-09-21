#!/bin/bash
# User-local install script for Marco Markdown Editor (Linux)
# bash dev/install_marco_user.sh
set -e

# 1. test the binary
# cargo test

# # 2. Build the binary
# cargo build

# 3. Build the release binary
cargo build --release

# 4. Create user bin directory if needed
mkdir -p "$HOME/.local/bin"

# 5. Copy binary
cp target/release/marco "$HOME/.local/bin/marco"

# 6. Install .desktop file with MARCO_DATA_DIR fallback and file opening support
mkdir -p "$HOME/.local/share/applications"
cp tests/install/marco.desktop "$HOME/.local/share/applications/marco.desktop"

# 7. Install icon at multiple resolutions for best display quality
# Install 256x256 icon (scaled from high-quality source)
mkdir -p "$HOME/.local/share/icons/hicolor/256x256/apps"
convert src/assets/icons/icon.png -resize 256x256 "$HOME/.local/share/icons/hicolor/256x256/apps/marco.png"

# Install 128x128 icon
mkdir -p "$HOME/.local/share/icons/hicolor/128x128/apps" 
convert src/assets/icons/icon.png -resize 128x128 "$HOME/.local/share/icons/hicolor/128x128/apps/marco.png"

# Install 64x64 icon
mkdir -p "$HOME/.local/share/icons/hicolor/64x64/apps"
cp src/assets/icons/favicon.png "$HOME/.local/share/icons/hicolor/64x64/apps/marco.png"

# Install 48x48 icon  
mkdir -p "$HOME/.local/share/icons/hicolor/48x48/apps"
convert src/assets/icons/favicon.png -resize 48x48 "$HOME/.local/share/icons/hicolor/48x48/apps/marco.png"

# 8. Install settings
mkdir -p "$HOME/.local/share/marco/"
# Remove old settings
rm -rf "$HOME/.local/share/marco/*"
cp -r src/assets/settings.ron "$HOME/.local/share/marco/settings.ron"

# 8. Install themes
mkdir -p "$HOME/.local/share/marco/themes"
# Remove old themes
rm -rf "$HOME/.local/share/marco/themes/*"
cp -r src/assets/themes/ "$HOME/.local/share/marco/"

# 9. Install fonts
 mkdir -p "$HOME/.local/share/marco/fonts"
 # Remove old fonts
 rm -rf "$HOME/.local/share/marco/fonts*"
 cp -r src/assets/fonts/ "$HOME/.local/share/marco/"

# 9b. Install icons to asset directory for application use
mkdir -p "$HOME/.local/share/marco/icons"
cp src/assets/icons/favicon.png "$HOME/.local/share/marco/icons/"
cp src/assets/icons/icon.png "$HOME/.local/share/marco/icons/"

# 10. Install language files to bin directory
mkdir -p "$HOME/.local/share/marco/language"
# Remove old language files
rm -rf "$HOME/.local/share/marco/language/*"
cp -r src/components/language/ "$HOME/.local/share/marco/"

# 11. Install user guide and logo
mkdir -p "$HOME/.local/share/marco/doc"
# Remove old user guide and logo if they exist
rm -f "$HOME/.local/share/marco/doc/user_guide.md"
rm -f "$HOME/.local/share/marco/doc/logo.png"
cp -f "documentation/user guide/user_guide.md" "$HOME/.local/share/marco/doc/"
cp -f "documentation/user guide/logo.png" "$HOME/.local/share/marco/doc/"

# 12. Update desktop database
update-desktop-database "$HOME/.local/share/applications/"

echo "Marco installed for user $USER."
echo "Themes copied to $HOME/.local/share/marco/themes."
echo "Language files copied to $HOME/.local/share/marco/language."
echo "You may need to log out and back in for icons and schemas to be fully recognized."
echo "You can now launch Marco from your application menu or with 'marco' in a terminal."
