#!/bin/bash
# Install the Marco & Polo .deb package
# Usage: bash install_deb.sh [deb-file]
set -e

PACKAGE_NAME="marco-suite"

if [ "$EUID" -ne 0 ]; then
  echo "Please run as root: sudo bash install_deb.sh [deb-file]"
  exit 1
fi

DEB_FILE="$1"
if [ -z "$DEB_FILE" ]; then
  DEB_FILE=$(ls marco-suite_*.deb 2>/dev/null | head -1)
fi
if [ ! -f "$DEB_FILE" ]; then
  echo "Error: .deb package not found. Build it with: bash tests/install/build_deb.sh"
  exit 1
fi

echo "Installing $DEB_FILE ..."
apt install -y "./$DEB_FILE" || dpkg -i "$DEB_FILE"
echo "\nInstallation complete. Launch with: marco or polo"
