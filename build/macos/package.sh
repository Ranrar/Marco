#!/usr/bin/env bash
# Package Marco and Polo as macOS .app bundles and (optionally) install them
# to /Applications.
#
# Prerequisites:
#   - Homebrew GTK stack (gtk4, gtksourceview5, librsvg, adwaita-icon-theme)
#   - Release binaries already built: target/release/marco, target/release/polo
#
# Usage:
#   bash build/macos/package.sh            # bundle into build/installer/macos
#   bash build/macos/package.sh --install  # also copy to /Applications
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ASSETS="$ROOT/marco-shared/src/assets"
DIST="$ROOT/build/installer/macos"
VERSION="$(grep -m1 '^version' "$ROOT/marco/Cargo.toml" | sed 's/.*"\(.*\)"/\1/')"
INSTALL=0
[ "${1:-}" = "--install" ] && INSTALL=1

command -v sips >/dev/null || { echo "sips not found"; exit 1; }
command -v iconutil >/dev/null || { echo "iconutil not found"; exit 1; }

make_icns() {
    local src="$1" out="$2" name="$3"
    local work iconset
    work="$(mktemp -d)"
    iconset="$work/${name}.iconset"
    mkdir -p "$iconset"
    for s in 16 32 128 256 512; do
        sips -z "$s" "$s" "$src" --out "$iconset/icon_${s}x${s}.png" >/dev/null
        local d=$((s * 2))
        sips -z "$d" "$d" "$src" --out "$iconset/icon_${s}x${s}@2x.png" >/dev/null
    done
    iconutil -c icns "$iconset" -o "$out"
    rm -rf "$work"
}

write_plist() {
    # $1 = plist path, $2 = name, $3 = bundle id
    cat > "$1" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>$2</string>
    <key>CFBundleDisplayName</key><string>$2</string>
    <key>CFBundleIdentifier</key><string>$3</string>
    <key>CFBundleVersion</key><string>$VERSION</string>
    <key>CFBundleShortVersionString</key><string>$VERSION</string>
    <key>CFBundleExecutable</key><string>$2</string>
    <key>CFBundleIconFile</key><string>$2.icns</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>LSMinimumSystemVersion</key><string>12.0</string>
    <key>NSHighResolutionCapable</key><true/>
    <key>LSApplicationCategoryType</key><string>public.app-category.productivity</string>
</dict>
</plist>
EOF
}

bundle_app() {
    local name="$1" bin="$2" icon_png="$3" bundle_id="$4"
    local app="$DIST/${name}.app"
    echo "Packaging ${name}.app (v$VERSION)"
    rm -rf "$app"
    mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"

    make_icns "$icon_png" "$app/Contents/Resources/${name}.icns" "$name"
    cp -R "$ASSETS" "$app/Contents/Resources/assets"
    cp "$bin" "$app/Contents/MacOS/${name}"
    write_plist "$app/Contents/Info.plist" "$name" "$bundle_id"

    # Ad-hoc signing keeps Gatekeeper quiet for locally-built bundles.
    codesign --force --deep --sign - "$app" >/dev/null 2>&1 || true
}

mkdir -p "$DIST"
bundle_app "Marco" "$ROOT/target/release/marco" \
    "$ASSETS/icons/icon_662x662_marco.png" "io.github.marco.marco"
bundle_app "Polo" "$ROOT/target/release/polo" \
    "$ASSETS/icons/icon_662x662_polo.png" "io.github.marco.polo"

if [ "$INSTALL" -eq 1 ]; then
    echo "Installing to /Applications"
    cp -R "$DIST/Marco.app" "$DIST/Polo.app" /Applications/
    echo "Done: /Applications/Marco.app and /Applications/Polo.app"
else
    echo "Bundles ready in $DIST (use --install to copy to /Applications)"
fi
