#!/bin/bash
# Build Debian package (.deb) for Marco Markdown Editor + Polo Viewer (Linux)
#
# This script ONLY builds the package. It does not install/uninstall.
#
# Usage:
#   bash tests/install/build_deb.sh
#   bash tests/install/build_deb.sh --check
#   bash tests/install/build_deb.sh --help

set -euo pipefail

# Ensure we create files with standard Debian-ish permissions (dirs 755, files 644)
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
PACKAGE_NAME="marco-suite"
MAINTAINER="Kim Skov Rasmussen <kim@skovrasmussen.com>"
INSTALL_PREFIX="/usr"

if command -v dpkg &>/dev/null; then
    ARCHITECTURE="$(dpkg --print-architecture)"
else
    ARCHITECTURE="amd64"
fi

BUILD_DIR="$(mktemp -d /tmp/marco-deb-build.XXXXXX)"
trap 'rm -rf "$BUILD_DIR"' EXIT

MARCO_VERSION="$(grep '^version' marco/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')"

show_help() {
    cat << 'EOF'
Marco & Polo Debian Package Builder

USAGE:
    bash tests/install/build_deb.sh [OPTIONS]

DESCRIPTION:
    Builds a Debian package (.deb) for Marco (editor) and Polo (viewer).
    Does NOT install it. Use tests/install/install_deb.sh to install.

OPTIONS:
    -h, --help      Show this help message
    -c, --check     Check dependencies only (don't build)

OUTPUT:
    Creates: marco-suite_VERSION_ARCH.deb in the workspace root.
EOF
}

if [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; then
    show_help
    exit 0
fi

# Reject unknown options
if [ -n "${1:-}" ] && [ "${1:-}" != "-c" ] && [ "${1:-}" != "--check" ]; then
    print_error "Unknown option: ${1:-}"
    echo "Use 'bash tests/install/build_deb.sh --help' for usage information"
    exit 1
fi

check_dependencies() {
    print_header "Checking Dependencies"

    local missing_deps=()
    local missing_dev_deps=()

    if ! command -v cargo &>/dev/null; then
        print_error "Rust/Cargo not found"
        missing_deps+=("rustc" "cargo")
        echo "  Install from: https://rustup.rs/"
    else
        print_success "Rust/Cargo found ($(cargo --version))"
    fi

    if ! command -v pkg-config &>/dev/null; then
        print_error "pkg-config not found"
        missing_dev_deps+=("pkg-config")
    else
        print_success "pkg-config found"
    fi

    if ! command -v gcc &>/dev/null; then
        print_error "GCC not found"
        missing_dev_deps+=("build-essential")
    else
        print_success "GCC found ($(gcc --version | head -1))"
    fi

    if ! command -v dpkg-deb &>/dev/null; then
        print_error "dpkg-deb not found"
        missing_dev_deps+=("dpkg")
    else
        print_success "dpkg-deb found"
    fi

    if ! command -v fakeroot &>/dev/null; then
        print_warning "fakeroot not found (recommended; otherwise package files may be owned by your user)"
        missing_dev_deps+=("fakeroot")
    else
        print_success "fakeroot found"
    fi

    if ! command -v gzip &>/dev/null; then
        print_error "gzip not found"
        missing_dev_deps+=("gzip")
    else
        print_success "gzip found"
    fi

    if ! command -v strip &>/dev/null; then
        print_warning "strip not found (recommended to avoid unstripped-binary lintian errors)"
        missing_dev_deps+=("binutils")
    else
        print_success "strip found"
    fi

    if ! command -v convert &>/dev/null; then
        print_warning "ImageMagick 'convert' not found (optional, for icon scaling)"
        echo "   Install with: sudo apt install imagemagick"
    else
        print_success "ImageMagick found"
    fi

    if command -v pkg-config &>/dev/null; then
        if ! pkg-config --exists gtk4; then
            print_error "GTK4 development files not found"
            missing_dev_deps+=("libgtk-4-dev")
        else
            print_success "GTK4 found ($(pkg-config --modversion gtk4))"
        fi

        if ! pkg-config --exists gtksourceview-5; then
            print_error "GtkSourceView5 development files not found"
            missing_dev_deps+=("libgtksourceview-5-dev")
        else
            print_success "GtkSourceView5 found ($(pkg-config --modversion gtksourceview-5))"
        fi

        if pkg-config --exists webkitgtk-6.0; then
            print_success "WebKitGTK 6.0 found ($(pkg-config --modversion webkitgtk-6.0))"
        elif pkg-config --exists webkit2gtk-4.1; then
            print_success "WebKit2GTK 4.1 found ($(pkg-config --modversion webkit2gtk-4.1))"
        else
            print_error "WebKitGTK development files not found"
            missing_dev_deps+=("libwebkitgtk-6.0-dev")
        fi

        if ! pkg-config --exists fontconfig; then
            print_error "Fontconfig development files not found"
            missing_dev_deps+=("libfontconfig-dev")
        else
            print_success "Fontconfig found"
        fi
    fi

    if [ ${#missing_deps[@]} -gt 0 ] || [ ${#missing_dev_deps[@]} -gt 0 ]; then
        echo ""
        print_error "Missing dependencies detected!"

        if [ ${#missing_dev_deps[@]} -gt 0 ]; then
            echo ""
            print_info "Install required packages:"
            echo "  sudo apt update"
            echo "  sudo apt install ${missing_dev_deps[*]}"
        fi

        return 1
    fi

    print_success "All required dependencies found!"
    return 0
}

if [ "${1:-}" = "-c" ] || [ "${1:-}" = "--check" ]; then
    check_dependencies
    exit $?
fi

print_header "Marco & Polo Debian Package Build"

check_dependencies || {
    print_error "Please install missing dependencies and try again"
    exit 1
}

print_header "Building Debian Package"

print_info "Creating package directory structure..."
install -d -m 0755 "$BUILD_DIR/DEBIAN"
install -d -m 0755 "$BUILD_DIR${INSTALL_PREFIX}/bin"
install -d -m 0755 "$BUILD_DIR${INSTALL_PREFIX}/share/applications"
install -d -m 0755 "$BUILD_DIR${INSTALL_PREFIX}/share/icons/hicolor"
install -d -m 0755 "$BUILD_DIR${INSTALL_PREFIX}/share/marco/doc"
install -d -m 0755 "$BUILD_DIR${INSTALL_PREFIX}/share/man/man1"
install -d -m 0755 "$BUILD_DIR${INSTALL_PREFIX}/share/doc/${PACKAGE_NAME}"

print_info "Building Marco and Polo binaries (release, workspace)..."
cargo build --release --workspace
print_success "Build complete"

print_info "Copying binaries..."
install -m 0755 target/release/marco "$BUILD_DIR${INSTALL_PREFIX}/bin/marco"
install -m 0755 target/release/polo "$BUILD_DIR${INSTALL_PREFIX}/bin/polo"

# Strip binaries inside the package payload (avoid changing your local build artifacts)
if command -v strip &>/dev/null; then
    strip --strip-unneeded "$BUILD_DIR${INSTALL_PREFIX}/bin/marco" 2>/dev/null || true
    strip --strip-unneeded "$BUILD_DIR${INSTALL_PREFIX}/bin/polo" 2>/dev/null || true
fi
print_success "Binaries copied"

print_info "Copying desktop entries..."
install -m 0644 tests/install/marco.desktop "$BUILD_DIR${INSTALL_PREFIX}/share/applications/marco.desktop"
install -m 0644 tests/install/polo.desktop "$BUILD_DIR${INSTALL_PREFIX}/share/applications/polo.desktop"
print_success "Desktop entries copied"

print_info "Installing system icons..."
install -d -m 0755 "$BUILD_DIR${INSTALL_PREFIX}/share/icons/hicolor/64x64/apps"
install -d -m 0755 "$BUILD_DIR${INSTALL_PREFIX}/share/icons/hicolor/128x128/apps"
install -d -m 0755 "$BUILD_DIR${INSTALL_PREFIX}/share/icons/hicolor/256x256/apps"

install -m 0644 assets/icons/favicon.png "$BUILD_DIR${INSTALL_PREFIX}/share/icons/hicolor/64x64/apps/marco.png"
install -m 0644 assets/icons/favicon.png "$BUILD_DIR${INSTALL_PREFIX}/share/icons/hicolor/64x64/apps/polo.png"

if command -v convert &>/dev/null; then
    print_info "Scaling icons with ImageMagick..."
    convert assets/icons/icon_662x662.png -resize 128x128 \
        "$BUILD_DIR${INSTALL_PREFIX}/share/icons/hicolor/128x128/apps/marco.png" 2>/dev/null || {
        print_warning "Failed to create 128x128 icon, using favicon as fallback"
        install -m 0644 assets/icons/favicon.png "$BUILD_DIR${INSTALL_PREFIX}/share/icons/hicolor/128x128/apps/marco.png"
    }
    install -m 0644 "$BUILD_DIR${INSTALL_PREFIX}/share/icons/hicolor/128x128/apps/marco.png" \
        "$BUILD_DIR${INSTALL_PREFIX}/share/icons/hicolor/128x128/apps/polo.png"

    convert assets/icons/icon_662x662.png -resize 256x256 \
        "$BUILD_DIR${INSTALL_PREFIX}/share/icons/hicolor/256x256/apps/marco.png" 2>/dev/null || {
        print_warning "Failed to create 256x256 icon, using favicon as fallback"
        install -m 0644 assets/icons/favicon.png "$BUILD_DIR${INSTALL_PREFIX}/share/icons/hicolor/256x256/apps/marco.png"
    }
    install -m 0644 "$BUILD_DIR${INSTALL_PREFIX}/share/icons/hicolor/256x256/apps/marco.png" \
        "$BUILD_DIR${INSTALL_PREFIX}/share/icons/hicolor/256x256/apps/polo.png"
else
    print_warning "ImageMagick not found, using favicon for all icon sizes"
    install -m 0644 assets/icons/favicon.png "$BUILD_DIR${INSTALL_PREFIX}/share/icons/hicolor/128x128/apps/marco.png"
    install -m 0644 assets/icons/favicon.png "$BUILD_DIR${INSTALL_PREFIX}/share/icons/hicolor/128x128/apps/polo.png"
    install -m 0644 assets/icons/favicon.png "$BUILD_DIR${INSTALL_PREFIX}/share/icons/hicolor/256x256/apps/marco.png"
    install -m 0644 assets/icons/favicon.png "$BUILD_DIR${INSTALL_PREFIX}/share/icons/hicolor/256x256/apps/polo.png"
fi
print_success "Icons installed"

print_info "Copying shared assets..."
cp -r assets/fonts "$BUILD_DIR${INSTALL_PREFIX}/share/marco/"
cp -r assets/icons "$BUILD_DIR${INSTALL_PREFIX}/share/marco/"
cp -r assets/themes "$BUILD_DIR${INSTALL_PREFIX}/share/marco/"
cp -r assets/language "$BUILD_DIR${INSTALL_PREFIX}/share/marco/"

# Normalize permissions on copied trees (cp -r preserves working tree perms)
find "$BUILD_DIR${INSTALL_PREFIX}/share/marco" -type d -exec chmod 0755 {} +
find "$BUILD_DIR${INSTALL_PREFIX}/share/marco" -type f -exec chmod 0644 {} +
if [ -f "assets/settings_org.ron" ]; then
    install -m 0644 assets/settings_org.ron "$BUILD_DIR${INSTALL_PREFIX}/share/marco/settings.ron"
elif [ -f "assets/settings.ron" ]; then
    install -m 0644 assets/settings.ron "$BUILD_DIR${INSTALL_PREFIX}/share/marco/settings.ron"
fi
print_success "Assets copied"

print_info "Creating man pages..."
MANPAGE_DATE="$(date "+%B %Y")"

cat > "$BUILD_DIR${INSTALL_PREFIX}/share/man/man1/marco.1" << MANEOF
.TH MARCO 1 "${MANPAGE_DATE}" "marco ${MARCO_VERSION}" "User Commands"
.SH NAME
marco \- A GTK4-based Markdown editor with live preview and custom syntax extensions
.SH SYNOPSIS
.B marco
[\fIOPTIONS\fR] [\fIFILE\fR]
.SH DESCRIPTION
Marco is a fast, native Markdown editor built in Rust with live preview, syntax extensions, and a custom parser for technical documentation.
.SH OPTIONS
.TP
.B FILE
Open the specified Markdown file
.SH EXAMPLES
.TP
Start Marco editor
.B marco
.TP
Open a specific Markdown file
.B marco ~/Documents/readme.md
.SH SEE ALSO
.B polo(1)
.SH AUTHOR
Kim Skov Rasmussen
.SH WEBSITE
https://github.com/Ranrar/marco
MANEOF

cat > "$BUILD_DIR${INSTALL_PREFIX}/share/man/man1/polo.1" << MANEOF
.TH POLO 1 "${MANPAGE_DATE}" "polo ${MARCO_VERSION}" "User Commands"
.SH NAME
polo \- A lightweight GTK4-based Markdown viewer with WebKit6 rendering
.SH SYNOPSIS
.B polo
[\fIOPTIONS\fR] [\fIFILE\fR]
.SH DESCRIPTION
Polo is a lightweight Markdown viewer that displays rendered Markdown documents using the same engine as Marco.
.SH OPTIONS
.TP
.B FILE
Open the specified Markdown file for viewing
.SH EXAMPLES
.TP
Start Polo viewer
.B polo
.TP
Open and view a Markdown file
.B polo ~/Documents/readme.md
.SH SEE ALSO
.B marco(1)
.SH AUTHOR
Kim Skov Rasmussen
.SH WEBSITE
https://github.com/Ranrar/marco
MANEOF

chmod 644 "$BUILD_DIR${INSTALL_PREFIX}/share/man/man1/marco.1" "$BUILD_DIR${INSTALL_PREFIX}/share/man/man1/polo.1"

# Compress man pages (lintian: uncompressed-manual-page)
gzip -9n "$BUILD_DIR${INSTALL_PREFIX}/share/man/man1/marco.1"
gzip -9n "$BUILD_DIR${INSTALL_PREFIX}/share/man/man1/polo.1"
print_success "Man pages created"

print_info "Creating package metadata..."
cat > "$BUILD_DIR${INSTALL_PREFIX}/share/doc/${PACKAGE_NAME}/copyright" << 'EOF'
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: marco
Upstream-Contact: Kim Skov Rasmussen <kim@skovrasmussen.com>
Source: https://github.com/Ranrar/marco

Files: *
Copyright: 2024-2025 Kim Skov Rasmussen
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

# Add a minimal Debian changelog (lintian: no-changelog)
cat > "$BUILD_DIR${INSTALL_PREFIX}/share/doc/${PACKAGE_NAME}/changelog" << EOF
${PACKAGE_NAME} (${MARCO_VERSION}) unstable; urgency=medium

    * Automated build.

 -- ${MAINTAINER}  $(date -R)
EOF
chmod 0644 "$BUILD_DIR${INSTALL_PREFIX}/share/doc/${PACKAGE_NAME}/changelog"
gzip -9n "$BUILD_DIR${INSTALL_PREFIX}/share/doc/${PACKAGE_NAME}/changelog"

if [ -d "documentation" ]; then
    cp -r documentation/* "$BUILD_DIR${INSTALL_PREFIX}/share/marco/doc/" 2>/dev/null || true
fi
cp README.md "$BUILD_DIR${INSTALL_PREFIX}/share/marco/doc/README.md" 2>/dev/null || true
cp LICENSE "$BUILD_DIR${INSTALL_PREFIX}/share/marco/doc/LICENSE" 2>/dev/null || true
print_success "Metadata created"

print_info "Generating control file..."
INSTALLED_SIZE="$(du -sk "$BUILD_DIR" | cut -f1)"

cat > "$BUILD_DIR/DEBIAN/control" << EOF
Package: ${PACKAGE_NAME}
Version: ${MARCO_VERSION}
Section: editors
Priority: optional
Architecture: ${ARCHITECTURE}
Maintainer: ${MAINTAINER}
Installed-Size: ${INSTALLED_SIZE}
Depends: libc6, libgtk-4-1 (>= 4.0), libglib2.0-0t64 (>= 2.68) | libglib2.0-0 (>= 2.68), libgtksourceview-5-0 (>= 5.0), libwebkitgtk-6.0-4 (>= 2.40) | libwebkit2gtk-4.1-0 (>= 2.30), libjavascriptcoregtk-6.0-1 (>= 2.40) | libjavascriptcoregtk-4.1-0 (>= 2.30), libfontconfig1 (>= 2.12), libcairo2 (>= 1.16), libpango-1.0-0 (>= 1.44)
Suggests: imagemagick
Description: Marco & Polo - A Markdown Composer and Viewer
 Marco is a fast, native Markdown editor built in Rust with live preview,
 syntax extensions, and a custom parser for technical documentation.
 .
 Polo is a lightweight Markdown viewer with identical rendering engine.
 .
 Includes:
  - marco: Full-featured Markdown editor with SourceView5 text editing
  - polo: Lightweight Markdown viewer
  - Themes, fonts, and documentation
Homepage: https://github.com/Ranrar/marco
EOF

print_success "Control file generated"

cat > "$BUILD_DIR/DEBIAN/postinst" << 'EOF'
#!/bin/bash
set -e

if command -v update-desktop-database &>/dev/null; then
    update-desktop-database /usr/share/applications/ || true
fi

if command -v gtk-update-icon-cache &>/dev/null; then
    gtk-update-icon-cache -f -t /usr/share/icons/hicolor/ || true
fi

echo "Marco and Polo installed successfully!"
echo "Launch with: marco or polo"
EOF
chmod 755 "$BUILD_DIR/DEBIAN/postinst"
print_success "Maintainer scripts created"

print_header "Creating .deb Package"

PACKAGE_FILE="${PACKAGE_NAME}_${MARCO_VERSION}_${ARCHITECTURE}.deb"
print_info "Building package: $PACKAGE_FILE"

# Build under fakeroot so files in the package are owned by root:root
if command -v fakeroot &>/dev/null; then
    fakeroot dpkg-deb --build "$BUILD_DIR" "$PACKAGE_FILE"
else
    print_warning "fakeroot not available; package files may be owned by your user (lintian will complain)"
    dpkg-deb --build "$BUILD_DIR" "$PACKAGE_FILE"
fi
print_success "Package created: $PACKAGE_FILE"

print_header "Build Complete"
echo "Debian package created successfully!"
echo ""
echo "Package file: $PACKAGE_FILE"
echo "Package (compressed) size: $(du -h "$PACKAGE_FILE" | cut -f1)"
INSTALLED_MIB="$(awk -v kib="$INSTALLED_SIZE" 'BEGIN{printf "%.1f", kib/1024}')"
echo "Installed size (uncompressed): ${INSTALLED_SIZE} KiB (~${INSTALLED_MIB} MiB)"
echo ""
print_success "To install the package:"
echo "  sudo bash tests/install/install_deb.sh $PACKAGE_FILE"
echo ""
print_success "To uninstall the package:"
echo "  sudo bash tests/install/uninstall_deb.sh"
