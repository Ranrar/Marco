#!/bin/bash
# Debian Package Installer for Marco Markdown Editor and Polo Viewer (Linux)
#
# This script creates a Debian package (.deb) for system-wide installation
# of Marco and Polo. Once installed as a .deb package, users can manage
# installation through apt/dpkg and receive updates through system package
# managers.
#
# This installer:
#   1. Checks dependencies (Rust, GTK4, WebKit6, etc.)
#   2. Builds release binaries
#   3. Creates a properly structured Debian package
#   4. Installs the .deb package system-wide
#
# Usage: bash tests/install/linux.sh
# Requires: sudo access for system-wide installation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PACKAGE_NAME="marco-suite"
MARCO_VERSION=$(grep '^version' marco/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
MAINTAINER="Kim Skov Rasmussen <kim@skovrasmussen.com>"
ARCHITECTURE="amd64"
BUILD_DIR="/tmp/marco-deb-build"
INSTALL_PREFIX="/usr"

# Show help
if [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
    cat << 'EOF'
Marco & Polo Debian Package Installer

USAGE:
    bash tests/install/linux.sh [OPTIONS]

DESCRIPTION:
    Creates a Debian package (.deb) for Marco (markdown editor) and Polo (markdown viewer).
    The generated .deb file can then be installed system-wide using apt or dpkg.

    The package includes:
    - Marco binary (/usr/bin/marco)
    - Polo binary (/usr/bin/polo)
    - Desktop entries for menu integration
    - System icons at multiple resolutions
    - Shared assets (themes, fonts, documentation)
    - Man pages and documentation

PACKAGE CONTENTS (when installed):
    /usr/bin/marco                   Marco editor binary
    /usr/bin/polo                    Polo viewer binary
    /usr/share/marco/                Shared assets
    /usr/share/applications/         Desktop entries
    /usr/share/icons/hicolor/        System icons
    /usr/share/man/man1/             Man pages
    /usr/share/doc/marco-suite/      Documentation

REQUIREMENTS FOR BUILDING:
    - Rust 1.70+ and Cargo (for compilation)
    - GTK4 development files (libgtk-4-dev)
    - WebKitGTK development files (libwebkitgtk-6.0-dev or libwebkit2gtk-4.1-dev)
    - GtkSourceView5 development files (libgtksourceview-5-dev)
    - Build tools (gcc, pkg-config, make)
    - ImageMagick (optional, for icon scaling)

OPTIONS:
    -h, --help      Show this help message
    -c, --check     Check dependencies only (don't build package)

EXAMPLES:
    # Build the .deb package
    bash tests/install/linux.sh

    # Check if build dependencies are available
    bash tests/install/linux.sh --check

OUTPUT:
    The script creates: marco-suite_VERSION_amd64.deb

INSTALLATION:
    Once the .deb file is created, install it with:
    
    sudo apt install ./marco-suite_VERSION_amd64.deb
    
    Or:
    
    sudo dpkg -i marco-suite_VERSION_amd64.deb

SEE ALSO:
    tests/install/README.md - Complete installation guide
    tests/install/install.sh - User-local installer
    tests/install/uninstall.sh - User-local uninstaller

EOF
    exit 0
fi

# Helper functions
print_header() {
    echo ""
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}=========================================${NC}"
    echo ""
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

# Reject unknown options (after helper functions are defined)
if [ ! -z "$1" ] && [ "$1" != "-c" ] && [ "$1" != "--check" ]; then
    print_error "Unknown option: $1"
    echo "Use 'bash tests/install/linux.sh --help' for usage information"
    exit 1
fi

# Dependency checking
check_dependencies() {
    print_header "Checking Dependencies"

    local missing_deps=()
    local missing_dev_deps=()

    # Check for Rust toolchain
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found"
        missing_deps+=("rustc" "cargo")
        echo "  Install from: https://rustup.rs/"
    else
        print_success "Rust/Cargo found ($(cargo --version))"
    fi

    # Check for pkg-config
    if ! command -v pkg-config &> /dev/null; then
        print_error "pkg-config not found"
        missing_dev_deps+=("pkg-config")
    else
        print_success "pkg-config found"
    fi

    # Check for GCC
    if ! command -v gcc &> /dev/null; then
        print_error "GCC not found"
        missing_dev_deps+=("build-essential")
    else
        print_success "GCC found ($(gcc --version | head -1))"
    fi

    # Check for ImageMagick (optional)
    if ! command -v convert &> /dev/null; then
        print_warning "ImageMagick 'convert' not found (optional, for icon scaling)"
        echo "   Install with: sudo apt install imagemagick"
    else
        print_success "ImageMagick found"
    fi

    # Check for GTK4 development files
    if ! pkg-config --exists gtk4; then
        print_error "GTK4 development files not found"
        missing_dev_deps+=("libgtk-4-dev")
    else
        print_success "GTK4 found ($(pkg-config --modversion gtk4))"
    fi

    # Check for GtkSourceView5
    if ! pkg-config --exists gtksourceview-5; then
        print_error "GtkSourceView5 development files not found"
        missing_dev_deps+=("libgtksourceview-5-dev")
    else
        print_success "GtkSourceView5 found ($(pkg-config --modversion gtksourceview-5))"
    fi

    # Check for WebKitGTK (check both 6.0 and 4.1)
    if pkg-config --exists webkitgtk-6.0; then
        print_success "WebKitGTK 6.0 found ($(pkg-config --modversion webkitgtk-6.0))"
    elif pkg-config --exists webkit2gtk-4.1; then
        print_success "WebKit2GTK 4.1 found ($(pkg-config --modversion webkit2gtk-4.1))"
    else
        print_error "WebKitGTK development files not found"
        missing_dev_deps+=("libwebkitgtk-6.0-dev")
    fi

    # Check for Fontconfig
    if ! pkg-config --exists fontconfig; then
        print_error "Fontconfig development files not found"
        missing_dev_deps+=("libfontconfig-dev")
    else
        print_success "Fontconfig found"
    fi

    # Report missing dependencies
    if [ ${#missing_deps[@]} -gt 0 ] || [ ${#missing_dev_deps[@]} -gt 0 ]; then
        echo ""
        print_error "Missing dependencies detected!"
        
        if [ ${#missing_dev_deps[@]} -gt 0 ]; then
            echo ""
            print_info "Install required packages:"
            echo "  sudo apt update"
            echo "  sudo apt install ${missing_dev_deps[@]}"
        fi
        
        return 1
    fi

    print_success "All required dependencies found!"
    return 0
}

# Only check dependencies
if [ "$1" = "-c" ] || [ "$1" = "--check" ]; then
    check_dependencies
    exit $?
fi

# Main installation flow
print_header "Marco & Polo Debian Package Installation"

# Check dependencies
check_dependencies || {
    print_error "Please install missing dependencies and try again"
    exit 1
}

# Clean up old build directory
if [ -d "$BUILD_DIR" ]; then
    print_info "Cleaning up previous build..."
    rm -rf "$BUILD_DIR"
fi

# Create directory structure for Debian package
print_header "Building Debian Package"

print_info "Creating package directory structure..."
mkdir -p "$BUILD_DIR/DEBIAN"
mkdir -p "$BUILD_DIR/usr/bin"
mkdir -p "$BUILD_DIR/usr/share/applications"
mkdir -p "$BUILD_DIR/usr/share/icons/hicolor"
mkdir -p "$BUILD_DIR/usr/share/marco/doc"
mkdir -p "$BUILD_DIR/usr/share/man/man1"
mkdir -p "$BUILD_DIR/usr/share/doc/marco-suite"

# Build binaries
print_info "Building Marco and Polo binaries (this may take a few minutes)..."
cargo build --release --workspace
print_success "Build complete"

# Copy binaries
print_info "Copying binaries..."
cp target/release/marco "$BUILD_DIR/usr/bin/marco"
cp target/release/polo "$BUILD_DIR/usr/bin/polo"
chmod 755 "$BUILD_DIR/usr/bin/marco"
chmod 755 "$BUILD_DIR/usr/bin/polo"
print_success "Binaries copied"

# Copy desktop entries
print_info "Copying desktop entries..."
cp tests/install/marco.desktop "$BUILD_DIR/usr/share/applications/marco.desktop"
cp tests/install/polo.desktop "$BUILD_DIR/usr/share/applications/polo.desktop"
print_success "Desktop entries copied"

# Copy and process icons
print_info "Installing system icons..."
mkdir -p "$BUILD_DIR/usr/share/icons/hicolor/64x64/apps"
mkdir -p "$BUILD_DIR/usr/share/icons/hicolor/128x128/apps"
mkdir -p "$BUILD_DIR/usr/share/icons/hicolor/256x256/apps"

# 64x64 icon (use favicon.png)
cp assets/icons/favicon.png "$BUILD_DIR/usr/share/icons/hicolor/64x64/apps/marco.png"
cp assets/icons/favicon.png "$BUILD_DIR/usr/share/icons/hicolor/64x64/apps/polo.png"

# 128x128 and 256x256 icons (scale from high-res icon if ImageMagick available)
if command -v convert &> /dev/null; then
    print_info "Scaling icons with ImageMagick..."
    # 128x128
    convert assets/icons/icon_662x662.png -resize 128x128 "$BUILD_DIR/usr/share/icons/hicolor/128x128/apps/marco.png" 2>/dev/null || {
        print_warning "Failed to create 128x128 icon, using favicon as fallback"
        cp assets/icons/favicon.png "$BUILD_DIR/usr/share/icons/hicolor/128x128/apps/marco.png"
    }
    cp "$BUILD_DIR/usr/share/icons/hicolor/128x128/apps/marco.png" "$BUILD_DIR/usr/share/icons/hicolor/128x128/apps/polo.png"
    
    # 256x256
    convert assets/icons/icon_662x662.png -resize 256x256 "$BUILD_DIR/usr/share/icons/hicolor/256x256/apps/marco.png" 2>/dev/null || {
        print_warning "Failed to create 256x256 icon, using favicon as fallback"
        cp assets/icons/favicon.png "$BUILD_DIR/usr/share/icons/hicolor/256x256/apps/marco.png"
    }
    cp "$BUILD_DIR/usr/share/icons/hicolor/256x256/apps/marco.png" "$BUILD_DIR/usr/share/icons/hicolor/256x256/apps/polo.png"
else
    print_warning "ImageMagick not found, using favicon for all icon sizes"
    # Fallback: use favicon for both 128x128 and 256x256
    cp assets/icons/favicon.png "$BUILD_DIR/usr/share/icons/hicolor/128x128/apps/marco.png"
    cp assets/icons/favicon.png "$BUILD_DIR/usr/share/icons/hicolor/128x128/apps/polo.png"
    cp assets/icons/favicon.png "$BUILD_DIR/usr/share/icons/hicolor/256x256/apps/marco.png"
    cp assets/icons/favicon.png "$BUILD_DIR/usr/share/icons/hicolor/256x256/apps/polo.png"
fi
print_success "Icons installed"

# Copy shared assets
print_info "Copying shared assets..."
cp -r assets/fonts "$BUILD_DIR/usr/share/marco/"
cp -r assets/icons "$BUILD_DIR/usr/share/marco/"
cp -r assets/themes "$BUILD_DIR/usr/share/marco/"
cp -r assets/language "$BUILD_DIR/usr/share/marco/"
if [ -f "assets/settings_org.ron" ]; then
    cp assets/settings_org.ron "$BUILD_DIR/usr/share/marco/settings.ron"
elif [ -f "assets/settings.ron" ]; then
    cp assets/settings.ron "$BUILD_DIR/usr/share/marco/settings.ron"
fi
print_success "Assets copied"

# Create man pages
print_info "Creating man pages..."
MANPAGE_DATE=$(date "+%B %Y")
cat > "$BUILD_DIR/usr/share/man/man1/marco.1" << MANEOF
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

cat > "$BUILD_DIR/usr/share/man/man1/polo.1" << MANEOF
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

chmod 644 "$BUILD_DIR/usr/share/man/man1/marco.1"
chmod 644 "$BUILD_DIR/usr/share/man/man1/polo.1"
print_success "Man pages created"

# Create copyright file
print_info "Creating package metadata..."
cat > "$BUILD_DIR/usr/share/doc/marco-suite/copyright" << 'EOF'
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

# Copy main documentation
if [ -d "documentation" ]; then
    cp -r documentation/* "$BUILD_DIR/usr/share/marco/doc/" 2>/dev/null || true
fi
cp README.md "$BUILD_DIR/usr/share/marco/doc/README.md" 2>/dev/null || true
cp LICENSE "$BUILD_DIR/usr/share/marco/doc/LICENSE" 2>/dev/null || true

print_success "Metadata created"

# Create control file
print_info "Generating control file..."
INSTALLED_SIZE=$(du -sk "$BUILD_DIR" | cut -f1)

cat > "$BUILD_DIR/DEBIAN/control" << EOF
Package: marco-suite
Version: ${MARCO_VERSION}
Architecture: ${ARCHITECTURE}
Maintainer: ${MAINTAINER}
Installed-Size: ${INSTALLED_SIZE}
Depends: libgtk-4-1 (>= 4.0), libglib2.0-0t64 (>= 2.68) | libglib2.0-0 (>= 2.68), libgtksourceview-5-0 (>= 5.0), libwebkitgtk-6.0-4 (>= 2.40) | libwebkit2gtk-4.1-0 (>= 2.30), libjavascriptcoregtk-6.0-1 (>= 2.40) | libjavascriptcoregtk-4.1-0 (>= 2.30), libfontconfig1 (>= 2.12), libcairo2 (>= 1.16), libpango-1.0-0 (>= 1.44)
Suggests: imagemagick
Description: Marco & Polo - A Markdown Composer and Viewer
 Marco is a fast, native Markdown editor built in Rust with live preview,
 syntax extensions, and a custom parser for technical documentation.
 .
 Polo is a lightweight Markdown viewer with identical rendering engine.
 .
 Features:
  - Full CommonMark support (100% compliance)
  - Live render preview
  - Custom Markdown grammar with extensions
  - Document navigation and table of contents
  - Syntax highlighting for code blocks
 .
 Includes:
  - marco: Full-featured Markdown editor with SourceView5 text editing
  - polo: Lightweight Markdown viewer
  - Themes, fonts, and documentation
 .
 Runtime dependencies:
  - GTK4 (libgtk-4-1) - UI framework
  - GLib (libglib2.0-0t64 or libglib2.0-0) - Core utilities
  - GtkSourceView5 (libgtksourceview-5-0) - Text editing for Marco
  - WebKitGTK 6.0 or WebKit2GTK 4.1 - Preview rendering
  - JavaScriptCore 6.0 or 4.1 - JavaScript engine
  - Fontconfig (libfontconfig1) - Font configuration
  - Cairo (libcairo2) - 2D graphics
  - Pango (libpango-1.0-0) - Text layout
Homepage: https://github.com/Ranrar/marco
EOF

print_success "Control file generated"

# Create postinst script (runs after package installation)
cat > "$BUILD_DIR/DEBIAN/postinst" << 'EOF'
#!/bin/bash
set -e

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database /usr/share/applications/
fi

# Update icon cache
if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -f -t /usr/share/icons/hicolor/
fi

echo "Marco and Polo installed successfully!"
echo "Launch with: marco or polo"
EOF

chmod 755 "$BUILD_DIR/DEBIAN/postinst"

# Create prerm script (runs before package removal)
cat > "$BUILD_DIR/DEBIAN/prerm" << 'EOF'
#!/bin/bash
set -e
echo "Removing Marco and Polo..."
EOF

chmod 755 "$BUILD_DIR/DEBIAN/prerm"

print_success "Installation scripts created"

# Build the .deb package
print_header "Creating .deb Package"

PACKAGE_FILE="${PACKAGE_NAME}_${MARCO_VERSION}_${ARCHITECTURE}.deb"

print_info "Building package: $PACKAGE_FILE"
dpkg-deb --build "$BUILD_DIR" "$PACKAGE_FILE"
print_success "Package created: $PACKAGE_FILE"

# Final summary
print_header "Build Complete"

echo "Debian package created successfully!"
echo ""
echo "Package file: $PACKAGE_FILE"
echo "Package size: $(du -h "$PACKAGE_FILE" | cut -f1)"
echo ""
echo "Package contents:"
echo "  Binaries:"
echo "    /usr/bin/marco"
echo "    /usr/bin/polo"
echo ""
echo "  Desktop integration:"
echo "    /usr/share/applications/marco.desktop"
echo "    /usr/share/applications/polo.desktop"
echo ""
echo "  System icons:"
echo "    /usr/share/icons/hicolor/*/apps/marco.png"
echo "    /usr/share/icons/hicolor/*/apps/polo.png"
echo ""
echo "  Shared assets:"
echo "    /usr/share/marco/fonts/"
echo "    /usr/share/marco/themes/"
echo "    /usr/share/marco/language/"
echo "    /usr/share/marco/icons/"
echo "    /usr/share/marco/doc/"
echo ""
echo "  Documentation:"
echo "    /usr/share/man/man1/marco.1"
echo "    /usr/share/man/man1/polo.1"
echo "    /usr/share/doc/marco-suite/copyright"
echo ""
echo "========================================="
echo ""
print_success "To install the package, run:"
echo "  sudo apt install ./$PACKAGE_FILE"
echo ""
print_success "Or:"
echo "  sudo dpkg -i $PACKAGE_FILE"
echo ""
print_success "After installation, launch with:"
echo "  marco           # Markdown editor"
echo "  polo            # Markdown viewer"
echo ""
print_success "To uninstall:"
echo "  sudo apt remove marco-suite"
echo ""
echo "========================================="