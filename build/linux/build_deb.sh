#!/bin/bash
# Build Debian package (.deb) for Marco Markdown Editor + Polo Viewer (Linux)
#
# This script ONLY builds the package. It does not install/uninstall.
#
# Usage:
#   bash build/linux/build_deb.sh
#   bash build/linux/build_deb.sh --check
#   bash build/linux/build_deb.sh --version-only
#   bash build/linux/build_deb.sh --help

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

# Repo policy: always produce amd64-named packages/artifacts.
# (This is a naming/packaging constraint for our CI/release flow.)
ARCHITECTURE="amd64"

BUILD_DIR="$(mktemp -d /tmp/marco-deb-build.XXXXXX)"
trap 'rm -rf "$BUILD_DIR"' EXIT

VERSION_FILE="$ROOT_DIR/build/version.json"

CORE_VERSION=""
MARCO_VERSION=""
POLO_VERSION=""

show_help() {
    cat << 'EOF'
Marco & Polo Debian Package Builder

USAGE:
    bash build/linux/build_deb.sh [OPTIONS]

DESCRIPTION:
    Builds a Debian package (.deb) for Marco (editor) and Polo (viewer).
    Does NOT install it.

    Versions are tracked in: build/version.json
    By default, running this script uses the current versions from version.json.
    Use --bump or --set to change versions before building.

OPTIONS:
    -h, --help      Show this help message
    -c, --check     Check dependencies only (don't build)
    --version-only  Bump/set versions and sync Cargo.toml, then exit (no build)
    --no-bump       Build using current versions (do not change version.json)
    --bump MODE     Bump version before building: patch|minor|major (default: patch)
    --set VERSION   Set Core/Marco/Polo versions to VERSION (X.Y.Z) before building
    --alpha-artifact  Also create a CI-friendly alpha-named copy:
                    marco-suite_alpha_VERSION_amd64.deb

OUTPUT:
    Creates: build/installer/linux/marco-suite_VERSION_amd64.deb
    If --alpha-artifact is set, also creates: build/installer/linux/marco-suite_alpha_VERSION_amd64.deb
EOF
}

BUMP_MODE="patch"
DO_BUMP="false"
SET_VERSION=""
CHECK_ONLY="false"
CREATE_ALPHA_ARTIFACT="false"
VERSION_ONLY="false"

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
        --version-only)
            VERSION_ONLY="true"
            shift
            ;;
        --no-bump)
            DO_BUMP="false"
            shift
            ;;
        --bump)
            if [ -z "${2:-}" ]; then
                print_error "--bump requires a value: patch|minor|major"
                exit 1
            fi
            BUMP_MODE="$2"
            shift 2
            ;;
        --set)
            if [ -z "${2:-}" ]; then
                print_error "--set requires a version: X.Y.Z"
                exit 1
            fi
            SET_VERSION="$2"
            DO_BUMP="false"
            shift 2
            ;;
        --alpha-artifact)
            CREATE_ALPHA_ARTIFACT="true"
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            echo "Use 'bash build/linux/build_deb.sh --help' for usage information"
            exit 1
            ;;
    esac
done

validate_semver() {
    local v="$1"
    if [[ ! "$v" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        return 1
    fi
    return 0
}

ensure_version_file() {
    if [ -f "$VERSION_FILE" ]; then
        return 0
    fi

    print_warning "Version file not found; creating: $VERSION_FILE"

    local core_v marco_v polo_v
    core_v="$(grep '^version' core/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')"
    marco_v="$(grep '^version' marco/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')"
    polo_v="$(grep '^version' polo/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')"

    python3 - <<PY
import json
from pathlib import Path
Path("$VERSION_FILE").write_text(json.dumps({
  "linux": {
    "core": "$core_v",
    "marco": "$marco_v",
    "polo": "$polo_v"
  },
  "windows": {
    "core": "$core_v",
    "marco": "$marco_v",
    "polo": "$polo_v"
  }
}, indent=2) + "\n")
PY
}

read_versions() {
    CORE_VERSION="$(python3 -c 'import json;print(json.load(open("'$VERSION_FILE'"))["linux"]["core"])')"
    MARCO_VERSION="$(python3 -c 'import json;print(json.load(open("'$VERSION_FILE'"))["linux"]["marco"])')"
    POLO_VERSION="$(python3 -c 'import json;print(json.load(open("'$VERSION_FILE'"))["linux"]["polo"])')"
}

write_versions() {
    local core_v="$1"
    local marco_v="$2"
    local polo_v="$3"
    python3 - <<PY
import json
from pathlib import Path
Path("$VERSION_FILE").write_text(json.dumps({
  "linux": {
    "core": "$core_v",
    "marco": "$marco_v",
    "polo": "$polo_v"
  },
  "windows": {
    "core": "$core_v",
    "marco": "$marco_v",
    "polo": "$polo_v"
  }
}, indent=2) + "\n")
PY
}

bump_semver() {
    local v="$1"
    local mode="$2"
    python3 - "$v" "$mode" <<'PY'
import sys
v = sys.argv[1]
mode = sys.argv[2]
maj, mi, pa = [int(x) for x in v.split('.')]
if mode == 'patch':
    pa += 1
elif mode == 'minor':
    mi += 1
    pa = 0
elif mode == 'major':
    maj += 1
    mi = 0
    pa = 0
else:
    raise SystemExit(2)
print(f"{maj}.{mi}.{pa}")
PY
}

set_cargo_version() {
    local toml_path="$1"
    local new_version="$2"
    python3 - "$toml_path" "$new_version" <<'PY'
from pathlib import Path
import re
import sys

toml_path = sys.argv[1]
new_version = sys.argv[2]

path = Path(toml_path)
text = path.read_text(encoding="utf-8")
lines = text.splitlines(True)

in_pkg = False
done = False

for i, line in enumerate(lines):
    if re.match(r"^\[package\]\s*$", line.strip()):
        in_pkg = True
        continue

    # Stop once we leave the [package] section.
    if in_pkg and line.lstrip().startswith('[') and line.strip() != "[package]":
        in_pkg = False

    if in_pkg and (not done) and re.match(r"^version\s*=\s*\"[^\"]+\"", line):
        lines[i] = re.sub(
            r"^version\s*=\s*\"[^\"]+\"",
            f'version = "{new_version}"',
            line,
        )
        done = True
        break

if not done:
    raise SystemExit(f"Could not update version in {path}")

path.write_text(''.join(lines), encoding="utf-8")
PY
}

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

    if ! command -v python3 &>/dev/null; then
        print_error "python3 not found (required for version management)"
        missing_deps+=("python3")
    else
        print_success "python3 found ($(python3 --version 2>&1))"
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

if [ "$CHECK_ONLY" = "true" ]; then
    check_dependencies
    exit $?
fi

print_header "Marco & Polo Debian Package Build"

check_dependencies || {
    print_error "Please install missing dependencies and try again"
    exit 1
}

print_header "Versioning"

ensure_version_file
read_versions

if ! validate_semver "$CORE_VERSION" || ! validate_semver "$MARCO_VERSION" || ! validate_semver "$POLO_VERSION"; then
    print_error "Invalid version found in $VERSION_FILE (expected X.Y.Z)"
    exit 1
fi

if [ -n "$SET_VERSION" ]; then
    if ! validate_semver "$SET_VERSION"; then
        print_error "Invalid version for --set: $SET_VERSION (expected X.Y.Z)"
        exit 1
    fi

    print_info "Setting Core/Marco/Polo version to: $SET_VERSION"
    CORE_VERSION="$SET_VERSION"
    MARCO_VERSION="$SET_VERSION"
    POLO_VERSION="$SET_VERSION"
    write_versions "$CORE_VERSION" "$MARCO_VERSION" "$POLO_VERSION"
elif [ "$DO_BUMP" = "true" ]; then
    if [ "$BUMP_MODE" != "patch" ] && [ "$BUMP_MODE" != "minor" ] && [ "$BUMP_MODE" != "major" ]; then
        print_error "Invalid bump mode: $BUMP_MODE (expected patch|minor|major)"
        exit 1
    fi

    print_info "Bumping versions ($BUMP_MODE)..."
    CORE_VERSION="$(bump_semver "$CORE_VERSION" "$BUMP_MODE")"
    MARCO_VERSION="$(bump_semver "$MARCO_VERSION" "$BUMP_MODE")"
    POLO_VERSION="$(bump_semver "$POLO_VERSION" "$BUMP_MODE")"
    write_versions "$CORE_VERSION" "$MARCO_VERSION" "$POLO_VERSION"
else
    print_info "Using existing versions from $VERSION_FILE"
fi

print_info "Core version:  $CORE_VERSION"
print_info "Marco version: $MARCO_VERSION"
print_info "Polo version:  $POLO_VERSION"

print_info "Syncing Cargo.toml versions..."
set_cargo_version "core/Cargo.toml" "$CORE_VERSION"
set_cargo_version "marco/Cargo.toml" "$MARCO_VERSION"
set_cargo_version "polo/Cargo.toml" "$POLO_VERSION"
print_success "Versions updated"

if [ "$VERSION_ONLY" = "true" ]; then
    print_header "Version Sync Complete"
    echo "Updated versions only (no build):"
    echo "  build/version.json: core=$CORE_VERSION marco=$MARCO_VERSION polo=$POLO_VERSION"
    echo "  core/Cargo.toml:       $CORE_VERSION"
    echo "  marco/Cargo.toml:      $MARCO_VERSION"
    echo "  polo/Cargo.toml:       $POLO_VERSION"
    exit 0
fi

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
install -m 0644 build/linux/marco.desktop "$BUILD_DIR${INSTALL_PREFIX}/share/applications/marco.desktop"
install -m 0644 build/linux/polo.desktop "$BUILD_DIR${INSTALL_PREFIX}/share/applications/polo.desktop"
print_success "Desktop entries copied"

print_info "Installing system icons..."
ICON_SIZES="16 24 32 48 64 96 128 160 192 256 512"
for sz in $ICON_SIZES; do
    install -d -m 0755 "$BUILD_DIR${INSTALL_PREFIX}/share/icons/hicolor/${sz}x${sz}/apps"
done

# Repo icon sources (per-app)
MARCO_ICON_64="assets/icons/icon_64x64_marco.png"
POLO_ICON_64="assets/icons/icon_64x64_polo.png"
MARCO_ICON_662="assets/icons/icon_662x662_marco.png"
POLO_ICON_662="assets/icons/icon_662x662_polo.png"

HAS_CONVERT="false"
if command -v convert &>/dev/null; then
    HAS_CONVERT="true"
    print_info "Scaling icons with ImageMagick..."
else
    print_warning "ImageMagick not found, using 64x64 icons as fallbacks for all sizes"
fi

install_icon_set() {
    local app="$1"
    local src64="$2"
    local src662="$3"

    for sz in $ICON_SIZES; do
        local out="$BUILD_DIR${INSTALL_PREFIX}/share/icons/hicolor/${sz}x${sz}/apps/${app}.png"

        if [ "$sz" = "64" ]; then
            install -m 0644 "$src64" "$out"
            continue
        fi

        if [ "$HAS_CONVERT" = "true" ]; then
            # Produce a *square* icon matching the target directory size.
            # Some sources may have a non-square canvas; lintian requires exact NxN.
            convert "$src662" \
                -resize "${sz}x${sz}" \
                -background none \
                -gravity center \
                -extent "${sz}x${sz}" \
                "$out" 2>/dev/null || {
                print_warning "Failed to create ${app} ${sz}x${sz} icon, using 64x64 as fallback"
                install -m 0644 "$src64" "$out"
            }
        else
            install -m 0644 "$src64" "$out"
        fi
    done
}

install_icon_set "marco" "$MARCO_ICON_64" "$MARCO_ICON_662"
install_icon_set "polo" "$POLO_ICON_64" "$POLO_ICON_662"
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
.TH POLO 1 "${MANPAGE_DATE}" "polo ${POLO_VERSION}" "User Commands"
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
Copyright: 2025-2026 Kim Skov Rasmussen
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
 Marco is a fast, Markdown editor with a live preview to help you write
 clean documentation, notes, and README files.
 .
 Polo is the companion viewer for quickly opening Markdown documents with the
 same rendering as Marco, using minimal resources.
 .
 Highlights include CommonMark-compliant Markdown plus useful extensions like
 tables, task lists, footnotes, and callouts.
 .
 This package includes:
    - marco: Markdown editor with live preview
    - polo: Markdown viewer
    - Built-in themes, fonts, and documentation
Homepage: https://github.com/Ranrar/marco
EOF

print_success "Control file generated"

cat > "$BUILD_DIR/DEBIAN/postinst" << 'EOF'
#!/bin/bash
set -e

install_user_icons() {
    # This is intentionally opt-in-ish: only runs when we can reliably identify
    # the interactive user that invoked sudo/dpkg.
    #
    # NOTE: Debian policy generally discourages touching per-user files from a package.
    # This project supports it as a convenience for local installs when requested.

    local user="${SUDO_USER:-}"
    if [ -z "$user" ] || [ "$user" = "root" ]; then
        return 0
    fi

    # Resolve home directory
    local home
    home="$(getent passwd "$user" | cut -d: -f6 2>/dev/null || true)"
    if [ -z "$home" ]; then
        home="/home/$user"
    fi
    if [ ! -d "$home" ]; then
        return 0
    fi

    local base="$home/.local/share/icons/hicolor"
    local marker_dir="$home/.local/share/marco-suite"
    local marker_file="$marker_dir/user-icons-installed"

    # Keep in sync with the system icon sizes installed by the package.
    local icon_sizes="16 24 32 48 64 96 128 160 192 256 512"

    # Create directories with correct ownership
    install -d -m 0755 -o "$user" -g "$user" "$marker_dir"

    for sz in $icon_sizes; do
        install -d -m 0755 -o "$user" -g "$user" "$base/${sz}x${sz}/apps"

        # Copy icons from the system-installed hicolor theme. These exist because they are dpkg-managed.
        if [ -f "/usr/share/icons/hicolor/${sz}x${sz}/apps/marco.png" ]; then
            install -m 0644 -o "$user" -g "$user" "/usr/share/icons/hicolor/${sz}x${sz}/apps/marco.png" "$base/${sz}x${sz}/apps/marco.png" || true
        fi
        if [ -f "/usr/share/icons/hicolor/${sz}x${sz}/apps/polo.png" ]; then
            install -m 0644 -o "$user" -g "$user" "/usr/share/icons/hicolor/${sz}x${sz}/apps/polo.png" "$base/${sz}x${sz}/apps/polo.png" || true
        fi
    done

    echo "installed-by=marco-suite" > "$marker_file" || true
    chown "$user:$user" "$marker_file" 2>/dev/null || true
    chmod 0644 "$marker_file" 2>/dev/null || true

    if command -v gtk-update-icon-cache &>/dev/null; then
        # Run as the user so the cache files are user-owned.
        su -s /bin/sh -c "gtk-update-icon-cache -f -t '$base' || true" "$user" 2>/dev/null || true
    fi
}

if command -v update-desktop-database &>/dev/null; then
    update-desktop-database /usr/share/applications/ || true
fi

if command -v gtk-update-icon-cache &>/dev/null; then
    gtk-update-icon-cache -f -t /usr/share/icons/hicolor/ || true
fi

# Optional: also install icons to the invoking user's ~/.local/share/icons/hicolor
install_user_icons

echo "Marco and Polo installed successfully!"
echo "Launch with: marco or polo"
EOF
chmod 755 "$BUILD_DIR/DEBIAN/postinst"

cat > "$BUILD_DIR/DEBIAN/postrm" << 'EOF'
#!/bin/bash
set -e

# NOTE:
# Debian packages typically should not modify per-user files in home directories (e.g. ~/.local/share/icons).
# This project optionally installs per-user icons for convenience; when it does, it also removes them.

remove_user_icons() {
    local user="${SUDO_USER:-}"
    if [ -z "$user" ] || [ "$user" = "root" ]; then
        return 0
    fi

    local home
    home="$(getent passwd "$user" | cut -d: -f6 2>/dev/null || true)"
    if [ -z "$home" ]; then
        home="/home/$user"
    fi
    if [ ! -d "$home" ]; then
        return 0
    fi

    local base="$home/.local/share/icons/hicolor"
    local marker_file="$home/.local/share/marco-suite/user-icons-installed"

    # Only remove if we previously installed them.
    if [ ! -f "$marker_file" ]; then
        return 0
    fi

    local icon_sizes="16 24 32 48 64 96 128 160 192 256 512"
    for sz in $icon_sizes; do
        rm -f "$base/${sz}x${sz}/apps/marco.png" "$base/${sz}x${sz}/apps/polo.png" 2>/dev/null || true
    done
    rm -f "$marker_file" 2>/dev/null || true

    if command -v gtk-update-icon-cache &>/dev/null; then
        su -s /bin/sh -c "gtk-update-icon-cache -f -t '$base' || true" "$user" 2>/dev/null || true
    fi
}

case "$1" in
    remove|purge|upgrade|failed-upgrade|abort-install|abort-upgrade|disappear)
        if command -v update-desktop-database &>/dev/null; then
            update-desktop-database /usr/share/applications/ || true
        fi

        if command -v gtk-update-icon-cache &>/dev/null; then
            gtk-update-icon-cache -f -t /usr/share/icons/hicolor/ || true
        fi

        # Optional user-local cleanup (if installed by postinst)
        remove_user_icons

        # Only on purge: remove empty directories if dpkg has already removed payload files.
        if [ "$1" = "purge" ]; then
            rmdir --ignore-fail-on-non-empty /usr/share/marco/icons 2>/dev/null || true
            rmdir --ignore-fail-on-non-empty /usr/share/marco/fonts 2>/dev/null || true
            rmdir --ignore-fail-on-non-empty /usr/share/marco/language 2>/dev/null || true
            rmdir --ignore-fail-on-non-empty /usr/share/marco/themes 2>/dev/null || true
            rmdir --ignore-fail-on-non-empty /usr/share/marco/doc 2>/dev/null || true
            rmdir --ignore-fail-on-non-empty /usr/share/marco 2>/dev/null || true
        fi
        ;;
esac

exit 0
EOF
chmod 755 "$BUILD_DIR/DEBIAN/postrm"
print_success "Maintainer scripts created"

print_header "Creating .deb Package"

# Ensure installer output directory exists
INSTALLER_DIR="$ROOT_DIR/build/installer/linux"
mkdir -p "$INSTALLER_DIR"

PACKAGE_FILE="$INSTALLER_DIR/${PACKAGE_NAME}_${MARCO_VERSION}_${ARCHITECTURE}.deb"
print_info "Building package: $PACKAGE_FILE"

# Build under fakeroot so files in the package are owned by root:root
if command -v fakeroot &>/dev/null; then
    fakeroot dpkg-deb --build "$BUILD_DIR" "$PACKAGE_FILE"
else
    print_warning "fakeroot not available; package files may be owned by your user (lintian will complain)"
    dpkg-deb --build "$BUILD_DIR" "$PACKAGE_FILE"
fi
print_success "Package created: $PACKAGE_FILE"

if [ "$CREATE_ALPHA_ARTIFACT" = "true" ]; then
    ALPHA_PACKAGE_FILE="$INSTALLER_DIR/${PACKAGE_NAME}_alpha_${MARCO_VERSION}_${ARCHITECTURE}.deb"
    cp -f "$PACKAGE_FILE" "$ALPHA_PACKAGE_FILE"
    print_success "Alpha artifact created: $ALPHA_PACKAGE_FILE"
fi

print_header "Build Complete"
echo "Debian package created successfully!"
echo ""
echo "Package file: $PACKAGE_FILE"
echo "Package (compressed) size: $(du -h "$PACKAGE_FILE" | cut -f1)"
INSTALLED_MIB="$(awk -v kib="$INSTALLED_SIZE" 'BEGIN{printf "%.1f", kib/1024}')"
echo "Installed size (uncompressed): ${INSTALLED_SIZE} KiB (~${INSTALLED_MIB} MiB)"
echo ""
print_success "To install the package:"
echo "  sudo dpkg -i $PACKAGE_FILE"
echo "  # If dependencies are missing, run: sudo apt -f install"
echo ""
print_success "To uninstall the package:"
echo "  sudo dpkg -r ${PACKAGE_NAME}"
