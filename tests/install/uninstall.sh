#!/bin/bash
# User-local uninstall script for Marco Markdown Editor and Polo Viewer (Linux)
# 
# This script removes Marco and Polo from the user's local directories.
# It provides options to keep or remove user configuration and data.
#
# Usage: bash tests/install/uninstall.sh

set -e

# Show help if requested
if [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
    cat << 'EOF'
Marco & Polo Uninstaller

USAGE:
    bash tests/install/uninstall.sh [OPTIONS]

DESCRIPTION:
    Interactive uninstaller for Marco and Polo. Safely removes binaries,
    desktop entries, icons, and assets with options to preserve user data.

WHAT CAN BE REMOVED:
    - Binaries (marco, polo)
    - Desktop entries (application menu)
    - System icons
    - Shared assets (themes, fonts, language files)
    - User configuration (settings)
    - User data/cache/logs

DEFAULT BEHAVIOR:
    - Application files are removed
    - User configuration and data are KEPT by default
    - Interactive prompts confirm each removal

OPTIONS:
    -h, --help     Show this help message
    -y, --yes      Skip confirmation (use default answers)
    -f, --force    Remove everything including user data

EXAMPLES:
    # Interactive uninstall (recommended)
    bash tests/install/uninstall.sh

    # Remove everything including user data
    bash tests/install/uninstall.sh --force

PATHS:
    ~/.local/bin/marco              Marco binary
    ~/.local/bin/polo               Polo binary
    ~/.local/share/marco/           Shared assets
    ~/.config/marco/                Marco configuration
    ~/.config/polo/                 Polo configuration
    ~/.local/share/marco/data/      Marco data/cache/logs
    ~/.local/share/polo/data/       Polo data/cache/logs

SEE ALSO:
    tests/install/README.md - Installation guide
    tests/install/install.sh - Installer script

EOF
    exit 0
fi

echo "========================================="
echo "Marco & Polo Uninstaller"
echo "========================================="
echo ""
echo "This will remove Marco and Polo from your system."
echo ""

# Function to ask yes/no questions
ask_yes_no() {
    local prompt="$1"
    local default="$2"
    local answer
    
    if [ "$default" = "y" ]; then
        prompt="$prompt [Y/n]: "
    else
        prompt="$prompt [y/N]: "
    fi
    
    read -p "$prompt" answer
    answer=${answer:-$default}
    
    case "$answer" in
        [Yy]|[Yy][Ee][Ss]) return 0 ;;
        *) return 1 ;;
    esac
}

# Track what we're removing
REMOVE_BINARIES=false
REMOVE_DESKTOP=false
REMOVE_ICONS=false
REMOVE_ASSETS=false
REMOVE_CONFIG=false
REMOVE_DATA=false

# Ask what to remove
echo "What would you like to remove?"
echo ""

if ask_yes_no "Remove binaries (marco, polo)?" "y"; then
    REMOVE_BINARIES=true
fi

if ask_yes_no "Remove desktop entries (application menu entries)?" "y"; then
    REMOVE_DESKTOP=true
fi

if ask_yes_no "Remove system icons?" "y"; then
    REMOVE_ICONS=true
fi

if ask_yes_no "Remove shared assets (themes, fonts, language files)?" "y"; then
    REMOVE_ASSETS=true
fi

echo ""
echo "User data and configuration:"
echo ""

if ask_yes_no "Remove user configuration (~/.config/marco/ and ~/.config/polo/)?" "n"; then
    REMOVE_CONFIG=true
fi

if ask_yes_no "Remove user data/cache/logs (~/.local/share/marco/data/ and ~/.local/share/polo/data/)?" "n"; then
    REMOVE_DATA=true
fi

# Confirm before proceeding
echo ""
echo "========================================="
echo "Removal Summary:"
echo "========================================="
[ "$REMOVE_BINARIES" = true ] && echo "✓ Binaries (~/.local/bin/)"
[ "$REMOVE_DESKTOP" = true ] && echo "✓ Desktop entries (~/.local/share/applications/)"
[ "$REMOVE_ICONS" = true ] && echo "✓ System icons (~/.local/share/icons/)"
[ "$REMOVE_ASSETS" = true ] && echo "✓ Shared assets (~/.local/share/marco/)"
[ "$REMOVE_CONFIG" = true ] && echo "✓ User configuration (~/.config/marco/ and ~/.config/polo/)"
[ "$REMOVE_DATA" = true ] && echo "✓ User data/cache (~/.local/share/marco/data/ and ~/.local/share/polo/data/)"
echo ""

if ! ask_yes_no "Proceed with uninstallation?" "y"; then
    echo "Uninstallation cancelled."
    exit 0
fi

echo ""
echo "Removing components..."
echo ""

# Remove binaries
if [ "$REMOVE_BINARIES" = true ]; then
    echo "Removing binaries..."
    if [ -f "$HOME/.local/bin/marco" ]; then
        rm "$HOME/.local/bin/marco"
        echo "  ✓ Removed marco"
    else
        echo "  ℹ marco not found"
    fi
    
    if [ -f "$HOME/.local/bin/polo" ]; then
        rm "$HOME/.local/bin/polo"
        echo "  ✓ Removed polo"
    else
        echo "  ℹ polo not found"
    fi
fi

# Remove desktop entries
if [ "$REMOVE_DESKTOP" = true ]; then
    echo "Removing desktop entries..."
    if [ -f "$HOME/.local/share/applications/marco.desktop" ]; then
        rm "$HOME/.local/share/applications/marco.desktop"
        echo "  ✓ Removed marco.desktop"
    else
        echo "  ℹ marco.desktop not found"
    fi
    
    if [ -f "$HOME/.local/share/applications/polo.desktop" ]; then
        rm "$HOME/.local/share/applications/polo.desktop"
        echo "  ✓ Removed polo.desktop"
    else
        echo "  ℹ polo.desktop not found"
    fi
    
    # Update desktop database
    if command -v update-desktop-database &> /dev/null; then
        update-desktop-database "$HOME/.local/share/applications/" 2>/dev/null || true
        echo "  ✓ Updated desktop database"
    fi
fi

# Remove system icons
if [ "$REMOVE_ICONS" = true ]; then
    echo "Removing system icons..."
    ICON_SIZES=("256x256" "128x128" "64x64" "48x48")
    REMOVED_COUNT=0
    
    for size in "${ICON_SIZES[@]}"; do
        if [ -f "$HOME/.local/share/icons/hicolor/$size/apps/marco.png" ]; then
            rm "$HOME/.local/share/icons/hicolor/$size/apps/marco.png"
            ((REMOVED_COUNT++))
        fi
        if [ -f "$HOME/.local/share/icons/hicolor/$size/apps/polo.png" ]; then
            rm "$HOME/.local/share/icons/hicolor/$size/apps/polo.png"
            ((REMOVED_COUNT++))
        fi
    done
    
    if [ $REMOVED_COUNT -gt 0 ]; then
        echo "  ✓ Removed $REMOVED_COUNT icon files"
    else
        echo "  ℹ No icon files found"
    fi
    
    # Update icon cache
    if command -v gtk-update-icon-cache &> /dev/null; then
        gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor/" 2>/dev/null || true
        echo "  ✓ Updated icon cache"
    fi
fi

# Remove shared assets
if [ "$REMOVE_ASSETS" = true ]; then
    echo "Removing shared assets..."
    
    # Only remove the asset root if data directory doesn't exist or is being removed
    if [ "$REMOVE_DATA" = true ] || [ ! -d "$HOME/.local/share/marco/data" ]; then
        if [ -d "$HOME/.local/share/marco" ]; then
            rm -rf "$HOME/.local/share/marco"
            echo "  ✓ Removed entire ~/.local/share/marco/"
        else
            echo "  ℹ Asset directory not found"
        fi
    else
        # Selectively remove asset subdirectories but keep data/
        ASSET_DIRS=("fonts" "icons" "themes" "language" "documentation")
        REMOVED_COUNT=0
        
        for dir in "${ASSET_DIRS[@]}"; do
            if [ -d "$HOME/.local/share/marco/$dir" ]; then
                rm -rf "$HOME/.local/share/marco/$dir"
                ((REMOVED_COUNT++))
            fi
        done
        
        # Remove settings.ron if it exists
        if [ -f "$HOME/.local/share/marco/settings.ron" ]; then
            rm "$HOME/.local/share/marco/settings.ron"
            ((REMOVED_COUNT++))
        fi
        
        if [ $REMOVED_COUNT -gt 0 ]; then
            echo "  ✓ Removed $REMOVED_COUNT asset directories/files"
            echo "  ℹ Kept data directory: ~/.local/share/marco/data/"
        else
            echo "  ℹ No asset directories found"
        fi
    fi
fi

# Remove user configuration
if [ "$REMOVE_CONFIG" = true ]; then
    echo "Removing user configuration..."
    REMOVED_COUNT=0
    
    if [ -d "$HOME/.config/marco" ]; then
        rm -rf "$HOME/.config/marco"
        echo "  ✓ Removed ~/.config/marco/"
        ((REMOVED_COUNT++))
    else
        echo "  ℹ Marco config not found"
    fi
    
    if [ -d "$HOME/.config/polo" ]; then
        rm -rf "$HOME/.config/polo"
        echo "  ✓ Removed ~/.config/polo/"
        ((REMOVED_COUNT++))
    else
        echo "  ℹ Polo config not found"
    fi
    
    [ $REMOVED_COUNT -eq 0 ] && echo "  ℹ No configuration directories found"
fi

# Remove user data/cache
if [ "$REMOVE_DATA" = true ]; then
    echo "Removing user data and cache..."
    REMOVED_COUNT=0
    
    if [ -d "$HOME/.local/share/marco/data" ]; then
        rm -rf "$HOME/.local/share/marco/data"
        echo "  ✓ Removed ~/.local/share/marco/data/"
        ((REMOVED_COUNT++))
    else
        echo "  ℹ Marco data directory not found"
    fi
    
    if [ -d "$HOME/.local/share/polo/data" ]; then
        rm -rf "$HOME/.local/share/polo/data"
        echo "  ✓ Removed ~/.local/share/polo/data/"
        ((REMOVED_COUNT++))
    else
        echo "  ℹ Polo data directory not found"
    fi
    
    # Clean up empty parent directory if it exists and is empty
    if [ -d "$HOME/.local/share/marco" ] && [ -z "$(ls -A $HOME/.local/share/marco)" ]; then
        rmdir "$HOME/.local/share/marco"
        echo "  ✓ Removed empty ~/.local/share/marco/"
    fi
    
    if [ -d "$HOME/.local/share/polo" ] && [ -z "$(ls -A $HOME/.local/share/polo)" ]; then
        rmdir "$HOME/.local/share/polo"
        echo "  ✓ Removed empty ~/.local/share/polo/"
    fi
    
    [ $REMOVED_COUNT -eq 0 ] && echo "  ℹ No data directories found"
fi

echo ""
echo "========================================="
echo "Uninstallation Complete!"
echo "========================================="
echo ""

# Show what remains (if anything)
REMAINS=false

if [ "$REMOVE_BINARIES" = false ] && { [ -f "$HOME/.local/bin/marco" ] || [ -f "$HOME/.local/bin/polo" ]; }; then
    echo "Remaining: Binaries at ~/.local/bin/"
    REMAINS=true
fi

if [ "$REMOVE_CONFIG" = false ] && { [ -d "$HOME/.config/marco" ] || [ -d "$HOME/.config/polo" ]; }; then
    echo "Remaining: Configuration at ~/.config/marco/ and ~/.config/polo/"
    REMAINS=true
fi

if [ "$REMOVE_DATA" = false ] && { [ -d "$HOME/.local/share/marco/data" ] || [ -d "$HOME/.local/share/polo/data" ]; }; then
    echo "Remaining: User data at ~/.local/share/marco/data/ and ~/.local/share/polo/data/"
    REMAINS=true
fi

if [ "$REMAINS" = false ]; then
    echo "All Marco and Polo files have been removed from your system."
fi

echo ""
echo "Thank you for using Marco and Polo!"
echo "========================================="
