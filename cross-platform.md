# Cross-Platform Implementation Status

## Overview

Marco is configured to support multiple platforms using a hybrid approach:
- **Linux**: webkit6 (GTK4-native WebKit)  
- **Windows**: wry (WebView2/Chromium wrapper)

## Current Status

### Core Library ✅ **READY**
- **Status**: Fully cross-platform
- **Dependencies**: Pure Rust, no UI dependencies
- **Works on**: Linux, Windows
- **Build tested**: Linux ✅, Windows ⏳

### Marco (Editor) ✅ **BUILD-READY**
- **Dependencies**: ✅ Platform-specific configured (webkit6 for Linux, wry for Windows)
- **Linux**: ✅ Fully working with webkit6, tested and stable
- **Windows**: ⏳ Configured, needs build testing and wry implementation
- **Platform abstraction**: ✅ Created but not integrated (Linux uses webkit6 directly, works fine)

### Polo (Viewer) ✅ **BUILD-READY**
- **Dependencies**: ✅ Platform-specific configured (webkit6 for Linux, wry for Windows)
- **Linux**: ✅ Fully working with webkit6, tested and stable
- **Windows**: ⏳ Configured, needs build testing and wry implementation

## Build System Status

### ✅ Completed
- [x] Platform-specific dependencies in Cargo.toml (conditional compilation)
- [x] Core library is platform-agnostic
- [x] Linux builds and runs successfully
- [x] Platform abstraction layer created (`platform_webview.rs`)
- [x] Build scripts for all platforms (Linux .deb, Windows .zip)
- [x] Version management system (platform-specific version.json)
- [x] CI/CD workflow for Linux (GitHub Actions)
- [x] **Cross-platform file paths** - Asset discovery, install locations, config directories
- [x] **Cross-platform logging** - Platform-appropriate log directory locations

### ⏳ Pending (Requires Windows Environment)
- [ ] Test marco compilation on Windows (needs MSYS2 + GTK4 + WebView2)
- [ ] Test polo compilation on Windows
- [ ] Implement wry WebView integration for Windows
- [ ] CI/CD workflow for Windows builds

## Implementation Details

### Platform Abstraction Strategy

The codebase uses **conditional compilation** for platform-specific code:

```toml
[target.'cfg(target_os = "linux")'.dependencies]
webkit6 = { workspace = true }

[target.'cfg(windows)'.dependencies]
wry = { workspace = true }
tao = { workspace = true }
```

**Current approach**:
- Linux code uses `webkit6::WebView` directly (mature, stable)
- Windows will use `PlatformWebView` wrapper for wry
- When building on Linux → only webkit6 is compiled
- When building on Windows → only wry is compiled

**Why not integrate PlatformWebView everywhere now?**
- Would require refactoring 20+ files in marco and polo
- Linux implementation is stable and working
- Windows implementation is stub (untested)
### Phase 4: UI Polish 📋
- [ ] Platform-specific theming
- [ ] Native file dialogs per platform
- [ ] Platform-specific shortcuts
- [ ] Performance optimization

## Technical Details

### Platform Abstraction Trait

```rust
pub trait WebViewProvider {
    fn load_html(&self, html: &str, base_uri: Option<&str>);
    fn evaluate_script(&self, script: &str) -> Result<()>;
    fn set_background_color(&self, color: &str);
    fn scroll_to_position(&self, position: f64);
}
```

### Conditional Compilation

```toml
[target.'cfg(target_os = "linux")'.dependencies]
webkit6 = { workspace = true }

[target.'cfg(windows)'.dependencies]
wry = { workspace = true }
tao = { workspace = true }
```

### Platform-Specific Paths

Marco uses conditional compilation to support platform-appropriate file paths:

#### Asset Root Discovery
**Linux:**
1. Development: `target/{debug|release}/marco_assets/`
2. User install: `~/.local/share/marco/`
3. System local: `/usr/local/share/marco/`
4. System global: `/usr/share/marco/`

**Windows:**
1. Development: `target\{debug|release}\marco_assets\`
2. User install: `%LOCALAPPDATA%\Marco\`
3. System local: `%PROGRAMFILES%\Marco\`
4. System global: `%PROGRAMDATA%\Marco\`

#### Configuration & Data Directories
**Linux:**
- Config: `~/.config/marco/` (XDG_CONFIG_HOME)
- Data: `~/.local/share/marco/` (XDG_DATA_HOME)
- Cache: `~/.cache/marco/` (XDG_CACHE_HOME)
- Logs: `~/.cache/marco/logs/` or `cwd/log` (dev mode)

**Windows:**
- Config: `%APPDATA%\marco\`
- Data: `%LOCALAPPDATA%\Marco\`
- Cache: `%LOCALAPPDATA%\Marco\cache\`
- Logs: `%LOCALAPPDATA%\Marco\logs\` or `cwd\log` (dev mode)

**Implementation:** See `core/src/paths/{core,install}.rs` and `core/src/logic/logger.rs`

### Build Targets

| Platform | Build Command | Output |
|----------|--------------|--------|
| Linux | `bash build/linux/build_deb.sh` | `build/installer/linux/*.deb` |
| Windows | `.\build\windows\build.ps1 -Release` | `target\windows\release\marco.exe` |

## Known Issues

### Windows
- GTK4 + wry window handle extraction not implemented
- Custom protocol handler needs setup
- WebView2 runtime dependency management

## Next Steps

1. **Implement Windows wry integration** in `platform_webview.rs`
2. **Test on Windows** with MSYS2 + GTK4
3. **Update CI/CD** for Windows builds
4. **Create installers** for each platform

## Contributing

When adding platform-specific code:
1. Use `#[cfg(target_os = "...")]` attributes
2. Update the `WebViewProvider` trait if needed
3. Test on at least 2 platforms before merging
4. Update this document with progress

## Resources

- **wry documentation**: https://github.com/tauri-apps/wry
- **tao documentation**: https://github.com/tauri-apps/tao
- **webkit6-rs**: https://gtk-rs.org/gtk4-rs/stable/latest/docs/webkit6/
- **GTK4 Windows**: https://www.gtk.org/docs/installations/windows
