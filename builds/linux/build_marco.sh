#!/bin/bash
# Build Debian package (.deb) for Marco Markdown Editor (Linux)
#
# This script builds only the Marco editor package.
#
# Usage:
#   bash builds/linux/build_marco.sh
#   bash builds/linux/build_marco.sh --check
#   bash builds/linux/build_marco.sh --help

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
PACKAGE_NAME="marco"
MAINTAINER="Kim Skov Rasmussen <kim@skovrasmussen.com>"
INSTALL_PREFIX="/usr"
ARCHITECTURE="amd64"

BUILD_DIR="$(mktemp -d /tmp/marco-deb-build.XXXXXX)"
trap 'rm -rf "$BUILD_DIR"' EXIT

VERSION_FILE="$ROOT_DIR/builds/linux/version.json"

show_help() {
    cat << 'EOF'
Marco Debian Package Builder

USAGE:
    bash builds/linux/build_marco.sh [OPTIONS]

DESCRIPTION:
    Builds a Debian package (.deb) for Marco markdown editor only.

OPTIONS:
    -h, --help      Show this help message
    -c, --check     Check dependencies only (don't build)

OUTPUT:
    Creates: marco_VERSION_amd64.deb in the workspace root.
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
MARCO_VERSION="$(python3 -c 'import json;print(json.load(open("'$VERSION_FILE'"))["marco"])')"

print_header "Marco Debian Package Builder"
echo "Package: $PACKAGE_NAME"
echo "Version: $MARCO_VERSION"
echo "Architecture: $ARCHITECTURE"
echo ""

# Check dependencies (minimal set for Marco)
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

# Build Marco
print_header "Building Marco Editor"
cargo build --release -p marco
print_success "Marco binary built"

# Prepare package structure
print_info "Setting up package directory structure"
mkdir -p "$BUILD_DIR/DEBIAN"
mkdir -p "$BUILD_DIR/usr/bin"
mkdir -p "$BUILD_DIR/usr/share/applications"
mkdir -p "$BUILD_DIR/usr/share/icons/hicolor"/{16x16,24x24,32x32,48x48,64x64,96x96,128x128,160x160,192x192,256x256,512x512}/apps
mkdir -p "$BUILD_DIR/usr/share/marco"/{themes,fonts,language,icons,doc}

# Copy binary
cp -f target/release/marco "$BUILD_DIR/usr/bin/"
chmod 755 "$BUILD_DIR/usr/bin/marco"

# Copy desktop file
cp -f builds/linux/marco.desktop "$BUILD_DIR/usr/share/applications/"

# Copy assets
cp -rf assets/themes/editor "$BUILD_DIR/usr/share/marco/themes/"
cp -rf assets/themes/html_viever "$BUILD_DIR/usr/share/marco/themes/"
cp -rf assets/fonts/* "$BUILD_DIR/usr/share/marco/fonts/" || true
cp -rf assets/language/* "$BUILD_DIR/usr/share/marco/language/" || true
cp -f assets/settings_org.ron "$BUILD_DIR/usr/share/marco/settings_org.ron"

# Copy icons
for size in 16 24 32 48 64 96 128 160 192 256 512; do
    if [ -f "assets/icons/marco_${size}.png" ]; then
        cp "assets/icons/marco_${size}.png" "$BUILD_DIR/usr/share/icons/hicolor/${size}x${size}/apps/marco.png"
    fi
done

# Copy documentation
cp -f README.md "$BUILD_DIR/usr/share/marco/doc/"
cp -f LICENSE "$BUILD_DIR/usr/share/marco/doc/"
[ -f "changelog/marco.md" ] && cp -f "changelog/marco.md" "$BUILD_DIR/usr/share/marco/doc/changelog.md"

# Calculate installed size
INSTALLED_SIZE=$(du -sk "$BUILD_DIR" | cut -f1)

# Create control file
cat > "$BUILD_DIR/DEBIAN/control" << EOF
Package: marco
Version: $MARCO_VERSION
Section: editors
Priority: optional
Architecture: $ARCHITECTURE
Maintainer: $MAINTAINER
Installed-Size: $INSTALLED_SIZE
Depends: libgtk-4-1, libsourceview-5-0, libwebkit2gtk-4.1-0
Suggests: polo
Description: Fast and lightweight GTK4 markdown editor
 Marco is a modern markdown editor built with GTK4 and Rust.
 Features include live preview, syntax highlighting, theme support,
 and custom markdown extensions.
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
    gtk-update-icon-cache -f -t /usr/share/icons/hicolor/ || true
fi

echo "Marco editor installed successfully!"
echo "Launch with: marco"
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
PACKAGE_FILE="${PACKAGE_NAME}_${MARCO_VERSION}_${ARCHITECTURE}.deb"

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
echo "To uninstall: sudo dpkg -r marco"
