#!/bin/bash
# Uninstall Marco & Polo Debian package
# Usage: bash uninstall_deb.sh
set -e

PACKAGE_NAME="marco-suite"

if [ "$EUID" -ne 0 ]; then
  echo "Please run as root: sudo bash uninstall_deb.sh"
  exit 1
fi

echo "This will remove the $PACKAGE_NAME package and all installed files."
read -p "Are you sure you want to continue? [y/N] " confirm
if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
  echo "Aborted."
  exit 0
fi

apt remove -y "$PACKAGE_NAME" || dpkg -r "$PACKAGE_NAME"
echo "\nUninstall complete."
