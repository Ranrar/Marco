#!/bin/bash
# Build a Flatpak bundle for Polo, the viewer.
#
# Marco and Polo are TWO Flatpaks. This script builds only Polo; Marco has its
# own script, build_flatpak_marco.sh, in this directory.
#
# Polo installs and runs perfectly well on its own. With Marco absent, its
# "open in Marco" action becomes an offer to install Marco instead -- that is
# the intended behaviour, not a failure. See polo/src/marco_link.rs.
#
# This script ONLY builds (and optionally installs for testing). It does not
# publish anything.
#
# Usage:
#   bash build/linux/flatpak/build_flatpak_polo.sh
#   bash build/linux/flatpak/build_flatpak_polo.sh --check
#   bash build/linux/flatpak/build_flatpak_polo.sh --no-install
#   bash build/linux/flatpak/build_flatpak_polo.sh --no-bundle
#   bash build/linux/flatpak/build_flatpak_polo.sh --help

APP_NAME="Polo"
APP_ROLE="viewer"
APP_ID="io.github.ranrar.Marco.Polo"
APP_SUBDIR="polo"
APP_COMMAND="markdownviewer"
VERSION_KEY="polo"
BUNDLE_PREFIX="polo"

# Paths relative to build/linux/flatpak/polo/. There is deliberately no
# .service file here: Polo calls Marco, never the other way round, and
# DBusActivatable without a service file would fail `build-export`.
REQUIRED_FILES=(
    "${APP_ID}.desktop"
    "${APP_ID}.metainfo.xml"
    "icons/${APP_ID}.png"
)

# shellcheck source=_common.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_common.sh"

flatpak_main "$@"
