# Installation Scripts Quick Reference

## Files in this directory

- **`install.sh`** - User-local installer (recommended)
- **`uninstall.sh`** - Interactive uninstaller
- **`marco.desktop`** - Desktop entry for Marco editor
- **`polo.desktop`** - Desktop entry for Polo viewer
- **`README.md`** - Complete installation guide

## Quick Commands

```bash
# Install (builds automatically if needed)
bash tests/install/install.sh

# Install without rebuilding (if binaries exist)
bash tests/install/install.sh
# → Answer 'N' when asked to rebuild

# Uninstall (interactive)
bash tests/install/uninstall.sh

# Verify installation
marco --version
polo --version
```

## Installation Paths

```
~/.local/bin/              # Binaries (marco, polo)
~/.local/share/marco/      # Shared assets (themes, fonts, etc.)
~/.config/marco/           # Marco configuration
~/.config/polo/            # Polo configuration
~/.local/share/applications/  # Desktop entries
~/.local/share/icons/      # System icons
```

## Common Tasks

### Clean reinstall
```bash
bash tests/install/uninstall.sh  # Remove everything
bash tests/install/install.sh    # Fresh install
```

### Update binaries only
```bash
cargo build --release -p marco -p polo
cp target/release/marco ~/.local/bin/
cp target/release/polo ~/.local/bin/
```

### Update assets only
```bash
rm -rf ~/.local/share/marco/themes
cp -r assets/themes ~/.local/share/marco/
# Repeat for fonts, icons, etc.
```

### Reset configuration
```bash
rm -rf ~/.config/marco/settings.ron
rm -rf ~/.config/polo/settings.ron
# Will be recreated with defaults on next launch
```

## See Also

- **README.md** - Full documentation with architecture details and troubleshooting
- **install.sh** - The installer script source
- **uninstall.sh** - The uninstaller script source
- **../../README.md** - Project documentation
- **../../CONTRIBUTING.md** - Development guidelines
