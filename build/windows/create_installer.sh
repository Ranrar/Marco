#!/bin/bash
# Create Windows installer package (ZIP) for Marco
# This script packages the built Marco binary and dependencies

set -euo pipefail

# Colors
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

print_success() { echo -e "${GREEN}✓ $1${NC}"; }
print_error() { echo -e "${RED}✗ ERROR: $1${NC}"; }
print_info() { echo -e "${BLUE}i $1${NC}"; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$ROOT_DIR"

# Read version
VERSION_FILE="$ROOT_DIR/build/version.json"
if [ ! -f "$VERSION_FILE" ]; then
    print_error "Version file not found: $VERSION_FILE"
    exit 1
fi

VERSION=$(python3 -c "import json;print(json.load(open('$VERSION_FILE'))['windows']['marco'])")
INSTALLER_DIR="$ROOT_DIR/build/installer/windows"
PACKAGE_NAME="marco-suite_${VERSION}_windows_x64"
ZIP_FILE="$INSTALLER_DIR/${PACKAGE_NAME}.zip"

print_header "Marco Windows Installer Package"
print_info "Version: $VERSION"
print_info "Output: $ZIP_FILE"

# Ensure installer directory exists
mkdir -p "$INSTALLER_DIR"

# Check if binary exists
BINARY_PATH="$ROOT_DIR/target/windows/release/marco.exe"
if [ ! -f "$BINARY_PATH" ]; then
    print_error "Binary not found: $BINARY_PATH"
    print_info "Run: build/windows/build.ps1 -Release first"
    exit 1
fi

# Create temporary packaging directory
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

PACKAGE_DIR="$TEMP_DIR/$PACKAGE_NAME"
mkdir -p "$PACKAGE_DIR"

print_info "Copying binary..."
cp "$BINARY_PATH" "$PACKAGE_DIR/"
print_success "Binary copied"

print_info "Copying assets..."
mkdir -p "$PACKAGE_DIR/assets"
cp -r "$ROOT_DIR/assets/"* "$PACKAGE_DIR/assets/" || true
print_success "Assets copied"

print_info "Creating README..."
cat > "$PACKAGE_DIR/README.txt" << 'EOF'
Marco Markdown Editor - Windows Edition
========================================

INSTALLATION:
1. Extract this ZIP to a location of your choice (e.g., C:\Program Files\Marco)
2. Ensure WebView2 runtime is installed (included in Windows 11, or download from Microsoft)
3. Run marco.exe

SYSTEM REQUIREMENTS:
- Windows 10 version 1803 or later (Windows 11 recommended)
- Microsoft Edge WebView2 Runtime
- 4GB RAM minimum

WEBVIEW2 RUNTIME:
If marco.exe doesn't start, you may need to install the WebView2 runtime:
https://developer.microsoft.com/en-us/microsoft-edge/webview2/

DOCUMENTATION:
See: https://github.com/Ranrar/Marco

LICENSE:
MIT License - See LICENSE file in source repository

SUPPORT:
Report issues: https://github.com/Ranrar/Marco/issues
EOF

print_success "README created"

print_info "Creating ZIP package..."
cd "$TEMP_DIR"
zip -r "$ZIP_FILE" "$PACKAGE_NAME" > /dev/null
print_success "ZIP package created"

print_header "Package Complete"
echo "Windows installer package: $ZIP_FILE"
echo "Package size: $(du -h "$ZIP_FILE" | cut -f1)"
echo ""
print_info "To install:"
echo "  1. Extract the ZIP file"
echo "  2. Run marco.exe"
