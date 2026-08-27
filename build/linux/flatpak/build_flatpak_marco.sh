#!/bin/bash
# Build a Flatpak bundle for Marco, the editor.
#
# Marco and Polo are TWO Flatpaks. This script builds only Marco; Polo has its
# own script, build_flatpak_polo.sh, in this directory. Neither builds the
# other, and they can be run in either order -- but if you want to exercise
# Polo's "open in Marco" handover, Marco has to be installed for the session bus
# to have anything to activate.
#
# This script ONLY builds (and optionally installs for testing). It does not
# publish anything.
#
# Usage:
#   bash build/linux/flatpak/build_flatpak_marco.sh
#   bash build/linux/flatpak/build_flatpak_marco.sh --check
#   bash build/linux/flatpak/build_flatpak_marco.sh --no-install
#   bash build/linux/flatpak/build_flatpak_marco.sh --no-bundle
#   bash build/linux/flatpak/build_flatpak_marco.sh --help

APP_NAME="Marco"
APP_ROLE="editor"
APP_ID="io.github.ranrar.Marco"
APP_SUBDIR="marco"
APP_COMMAND="markdowncomposer"
VERSION_KEY="marco"
BUNDLE_PREFIX="marco"

# Paths relative to build/linux/flatpak/marco/. The .service file is not
# optional: Marco's desktop file sets DBusActivatable=true, and `build-export`
# treats a missing service file as a hard error when it sees that key.
REQUIRED_FILES=(
    "${APP_ID}.desktop"
    "${APP_ID}.service"
    "${APP_ID}.metainfo.xml"
    "icons/${APP_ID}.png"
)

# shellcheck source=_common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_common.sh"

flatpak_main "$@"
