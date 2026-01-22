#!/bin/bash
# Build Debian package (.deb) for Polo Markdown Viewer (Linux)
#
# This script builds only the Polo viewer package.
#
# Usage:
#   bash builds/linux/build_polo.sh
#   bash builds/linux/build_polo.sh --check
#   bash builds/linux/build_polo.sh --help

set -euo pipefail
umask 022

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_header() {
    echo ""
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}=========================================${NC}"
    echo ""
}

print_success() { echo -e "${GREEN}OK: $1${NC}"; }
print_error() { echo -e "${RED}ERROR: $1${NC}"; }
print_warning() { echo -e "${YELLOW}WARN: $1${NC}"; }
print_info() { echo -e "${BLUE}INFO: $1${NC}"; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$ROOT_DIR"

# Configuration
PACKAGE_NAME="polo"
MAINTAINER="Kim Skov Rasmussen <kim@skovrasmussen.com>"
INSTALL_PREFIX="/usr"
ARCHITECTURE="amd64"

BUILD_DIR="$(mktemp -d /tmp/polo-deb-build.XXXXXX)"
trap 'rm -rf "$BUILD_DIR"' EXIT

VERSION_FILE="$ROOT_DIR/builds/linux/version.json"

show_help() {
    cat << 'EOF'
Polo Debian Package Builder

USAGE:
    bash builds/linux/build_polo.sh [OPTIONS]

DESCRIPTION:
    Builds a Debian package (.deb) for Polo markdown viewer only.

OPTIONS:
    -h, --help      Show this help message
    -c, --check     Check dependencies only (don't build)

OUTPUT:
    Creates: polo_VERSION_amd64.deb in the workspace root.
EOF
}

CHECK_ONLY="false"

while [ $# -gt 0 ]; do
    case "$1" in
        -h|--help)
            show_help
            exit 0
            ;;
        -c|--check)
            CHECK_ONLY="true"
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Read version
POLO_VERSION="$(python3 -c 'import json;print(json.load(open("'$VERSION_FILE'"))["polo"])')"

print_header "Polo Debian Package Builder"
echo "Package: $PACKAGE_NAME"
echo "Version: $POLO_VERSION"
echo "Architecture: $ARCHITECTURE"
echo ""

# Check dependencies
print_header "Checking Dependencies"

if ! command -v cargo &>/dev/null; then
    print_error "Rust/Cargo not found"
    exit 1
fi
print_success "Rust/Cargo found"

if ! command -v dpkg-deb &>/dev/null; then
    print_error "dpkg-deb not found (install dpkg)"
    exit 1
fi
print_success "dpkg-deb found"

if [ "$CHECK_ONLY" = "true" ]; then
    print_success "Dependency check complete"
    exit 0
fi

# Build Polo and servo-runner
print_header "Building Polo Viewer"
cargo build --release -p polo
print_success "Polo binary built"

cargo build --release --bin servo-runner
print_success "servo-runner binary built"

# Prepare package structure
print_info "Setting up package directory structure"
mkdir -p "$BUILD_DIR/DEBIAN"
mkdir -p "$BUILD_DIR/usr/bin"
mkdir -p "$BUILD_DIR/usr/share/applications"
mkdir -p "$BUILD_DIR/usr/share/doc/polo"
mkdir -p "$BUILD_DIR/usr/share/lintian/overrides"
mkdir -p "$BUILD_DIR/usr/share/icons/hicolor"/{16x16,24x24,32x32,48x48,64x64,96x96,128x128,160x160,192x192,256x256,512x512}/apps
mkdir -p "$BUILD_DIR/usr/share/marco"/{themes,fonts,language,icons,doc}

# Copy binaries
cp -f target/release/polo "$BUILD_DIR/usr/bin/"
cp -f target/release/servo-runner "$BUILD_DIR/usr/bin/"
chmod 755 "$BUILD_DIR/usr/bin/polo"
chmod 755 "$BUILD_DIR/usr/bin/servo-runner"

# Strip binaries to reduce size and satisfy lintian
if command -v strip &>/dev/null; then
    strip --strip-unneeded "$BUILD_DIR/usr/bin/polo" 2>/dev/null || true
    strip --strip-unneeded "$BUILD_DIR/usr/bin/servo-runner" 2>/dev/null || true
fi

# Create copyright file
cat > "$BUILD_DIR/usr/share/doc/polo/copyright" << 'EOF'
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: Polo
Upstream-Contact: Kim Skov Rasmussen <kim@skovrasmussen.com>
Source: https://github.com/Ranrar/Marco

Files: *
Copyright: 2024-2026 Kim Skov Rasmussen
License: MIT

License: MIT
 Permission is hereby granted, free of charge, to any person obtaining a copy
 of this software and associated documentation files (the "Software"), to deal
 in the Software without restriction, including without limitation the rights
 to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 copies of the Software, and to permit persons to whom the Software is
 furnished to do so, subject to the following conditions:
 .
 The above copyright notice and this permission notice shall be included in all
 copies or substantial portions of the Software.
 .
 THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 SOFTWARE.
EOF

# Create changelog (compress it as required by Debian policy)
cat > "$BUILD_DIR/usr/share/doc/polo/changelog" << EOF
polo ($POLO_VERSION) unstable; urgency=low

  * Release version $POLO_VERSION
  * See changelog/polo.md for detailed changes

 -- $MAINTAINER  $(date -R)
EOF
gzip -9n "$BUILD_DIR/usr/share/doc/polo/changelog"

# Create lintian overrides for acceptable issues
cat > "$BUILD_DIR/usr/share/lintian/overrides/polo" << 'EOF'
# servo-runner is a large binary that includes the Servo web engine
# Embedded libraries are expected and required for functionality
polo: embedded-library *

# Man pages for GUI applications are often omitted
polo: no-manual-page
EOF

# Copy desktop file
cp -f builds/linux/polo.desktop "$BUILD_DIR/usr/share/applications/"

# Copy assets (Polo needs themes for rendering)
cp -rf assets/themes/html_viever "$BUILD_DIR/usr/share/marco/themes/"
cp -rf assets/fonts/* "$BUILD_DIR/usr/share/marco/fonts/" || true
cp -rf assets/language/* "$BUILD_DIR/usr/share/marco/language/" || true
cp -f assets/settings_org.ron "$BUILD_DIR/usr/share/marco/settings_org.ron"

# Copy and resize icons (use exact file names from assets/icons/)
if command -v convert &>/dev/null; then
    # ImageMagick available - resize icons to exact sizes
    if [ -f "assets/icons/icon_64x64_polo.png" ]; then
        for size in 16 24 32 48 64 96; do
            mkdir -p "$BUILD_DIR/usr/share/icons/hicolor/${size}x${size}/apps"
            convert "assets/icons/icon_64x64_polo.png" -resize ${size}x${size}! "$BUILD_DIR/usr/share/icons/hicolor/${size}x${size}/apps/polo.png" 2>/dev/null || true
        done
    fi
    
    if [ -f "assets/icons/icon_662x662_polo.png" ]; then
        for size in 128 160 192 256 512; do
            mkdir -p "$BUILD_DIR/usr/share/icons/hicolor/${size}x${size}/apps"
            convert "assets/icons/icon_662x662_polo.png" -resize ${size}x${size}! "$BUILD_DIR/usr/share/icons/hicolor/${size}x${size}/apps/polo.png" 2>/dev/null || true
        done
    fi
else
    # No ImageMagick - only install icons in matching sizes
    if [ -f "assets/icons/icon_64x64_polo.png" ]; then
        mkdir -p "$BUILD_DIR/usr/share/icons/hicolor/64x64/apps"
        cp "assets/icons/icon_64x64_polo.png" "$BUILD_DIR/usr/share/icons/hicolor/64x64/apps/polo.png"
    fi
    
    # Skip 662x656 icon as it doesn't match standard sizes
    print_info "Install imagemagick for automatic icon resizing to standard sizes"
fi

# Copy documentation
cp -f README.md "$BUILD_DIR/usr/share/marco/doc/"
cp -f LICENSE "$BUILD_DIR/usr/share/marco/doc/"
[ -f "changelog/polo.md" ] && cp -f "changelog/polo.md" "$BUILD_DIR/usr/share/marco/doc/changelog.md"
[ -f "servo_runner/README.md" ] && cp -f "servo_runner/README.md" "$BUILD_DIR/usr/share/marco/doc/servo-runner.md"

# Calculate installed size
INSTALLED_SIZE=$(du -sk "$BUILD_DIR" | cut -f1)

# Create control file
cat > "$BUILD_DIR/DEBIAN/control" << EOF
Package: polo
Version: $POLO_VERSION
Section: text
Priority: optional
Architecture: $ARCHITECTURE
Maintainer: $MAINTAINER
Installed-Size: $INSTALLED_SIZE
Depends: libc6, libgtk-4-1, libglib2.0-0, libcairo2, libpango-1.0-0, libgdk-pixbuf-2.0-0, libharfbuzz0b, libfontconfig1
Suggests: marco
Description: Fast and lightweight GTK4 markdown viewer with Servo rendering
 Polo is a modern markdown viewer built with GTK4, Rust, and the Servo
 web engine. It provides read-only viewing of markdown files with full
 support for Marco's custom markdown extensions.
 .
 Includes servo-runner subprocess for web rendering.
Homepage: https://github.com/Ranrar/Marco
EOF

# Create postinst
cat > "$BUILD_DIR/DEBIAN/postinst" << 'EOF'
#!/bin/bash
set -e

if command -v update-desktop-database &>/dev/null; then
    update-desktop-database /usr/share/applications/ || true
fi

if command -v gtk-update-icon-cache &>/dev/null; then
    gtk-update-icon-cache -f /usr/share/icons/hicolor/ || true
fi

if command -v gtk-update-icon-cache &>/dev/null; then
    gtk-update-icon-cache -f -t /usr/share/icons/hicolor/ || true
fi

echo "Polo viewer installed successfully!"
echo "Launch with: polo"
EOF
chmod 755 "$BUILD_DIR/DEBIAN/postinst"

# Create postrm
cat > "$BUILD_DIR/DEBIAN/postrm" << 'EOF'
#!/bin/bash
set -e

case "$1" in
    remove|purge|upgrade|failed-upgrade|abort-install|abort-upgrade|disappear)
        if command -v update-desktop-database &>/dev/null; then
            update-desktop-database /usr/share/applications/ || true
        fi

        if command -v gtk-update-icon-cache &>/dev/null; then
            gtk-update-icon-cache -f -t /usr/share/icons/hicolor/ || true
        fi

        if [ "$1" = "purge" ]; then
            rmdir --ignore-fail-on-non-empty /usr/share/marco/* 2>/dev/null || true
            rmdir --ignore-fail-on-non-empty /usr/share/marco 2>/dev/null || true
        fi
        ;;
esac

exit 0
EOF
chmod 755 "$BUILD_DIR/DEBIAN/postrm"

# Build package
print_header "Creating .deb Package"
PACKAGE_FILE="${PACKAGE_NAME}_${POLO_VERSION}_${ARCHITECTURE}.deb"

if command -v fakeroot &>/dev/null; then
    fakeroot dpkg-deb --build "$BUILD_DIR" "$PACKAGE_FILE"
else
    dpkg-deb --build "$BUILD_DIR" "$PACKAGE_FILE"
fi

print_success "Package created: $PACKAGE_FILE"

print_header "Build Complete"
echo "Package file: $PACKAGE_FILE"
echo "Size: $(du -h "$PACKAGE_FILE" | cut -f1)"
echo ""
echo "To install: sudo dpkg -i $PACKAGE_FILE"
echo "To uninstall: sudo dpkg -r polo"
