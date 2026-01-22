# servo-runner

This directory contains the servo-runner binary, which is a subprocess spawned by polo's Servo-based WebView.

## Architecture

Polo uses servo-gtk for web rendering, which implements a two-process architecture:
- **polo (GTK process)**: The main application with GTK4 UI and servo-gtk WebView widget
- **servo-runner (rendering process)**: A subprocess that runs the Servo web engine

The two processes communicate via IPC using Protocol Buffers over stdin/stdout pipes.

### Why Two Processes?

The two-process architecture provides:
- **Isolation**: Rendering engine crashes don't take down the main UI
- **Security**: Web content runs in a separate process
- **Resource management**: Easier to monitor and control rendering process resources

## Current Status

- **Servo version**: ~v0.0.2 (git revision `b9f5a7920f18bd5294ebb95bbb422199f6371a54`, November 2024)
- **Rendering mode**: Software rendering (CPU-based)
- **Binary size**: 
  - Debug build: ~502MB
  - **Release build: ~120MB** (recommended)
- **Process management**: Subprocess is properly terminated on polo window close via `force_exit()`

**Note**: Servo v0.0.3 update is pending upstream due to breaking API changes in Servo's development.

## Subprocess Lifecycle

### Spawning
- servo-runner is spawned by servo-gtk when WebView widget is created
- Communication established via stdin/stdout pipes
- Process ID logged for debugging

### Cleanup
- When polo window closes, `WebView::cleanup()` is called via `connect_close_request` signal
- ServoRunner sends shutdown message to subprocess
- `force_exit()` called to send SIGKILL, ensuring subprocess termination
- No orphaned processes remain after polo exits

## Source

This code depends on the [servo-gtk](https://github.com/nacho/servo-gtk) library, which is maintained as an external repository at `/servo-gtk/`.

## Files

- `src/main.rs`: Main servo-runner entry point (originally `servo_runner/runner.rs`)
- `src/resource_reader.rs`: GResource reader for Servo assets
- `build.rs`: Build script for gresource compilation and protobuf code generation
- `proto/ipc.proto`: Protocol Buffers schema for IPC messages
- `resources/`: Servo resources compiled into binary via gresource

## Building

servo-runner is built as part of the marco workspace:

```bash
# Debug build (unoptimized, larger binary)
cargo build --bin servo-runner

# Release build (optimized, recommended for performance)
cargo build --release --bin servo-runner
```

**Performance Note**: Always use release builds for performance testing and production. Debug builds are significantly slower due to lack of compiler optimizations.

## Usage

servo-runner is not meant to be run directly. It is spawned automatically by polo when creating a Servo WebView.

The servo-gtk library launches it via:
```rust
cargo run --bin servo-runner
```

During development, polo will automatically spawn the debug build if available in `target/debug/`. For production, ensure release builds are available in `target/release/`.

### Process Cleanup

When polo closes, it sends a shutdown signal to servo-runner via the IPC channel. The subprocess should terminate automatically when:
- The polo window is closed normally
- Polo receives a termination signal (SIGTERM, SIGINT)
- The WebView widget is destroyed

If servo-runner processes become orphaned (still running after polo exits), you can manually clean them up:
```bash
pkill servo-runner
# or for force kill:
pkill -9 servo-runner
```

**Note**: The window destruction handler in polo explicitly unparents the WebView to ensure the dispose method is called, which triggers the shutdown signal to servo-runner.

## Dependencies

The Servo git revision in `Cargo.toml` must match the revision used by servo-gtk to avoid dependency conflicts. Both packages depend on:
- `libservo`: Core Servo web engine
- `embedder_traits`: Servo embedder API

### Windows

Building servo-runner and polo on Windows requires several additional dependencies beyond the standard Rust toolchain:

#### Required Tools

1. **uv** (Python package manager)
   - Required by Servo's build system for WebIDL code generation
   - Install: `irm https://astral.sh/uv/install.ps1 | iex`
   - Adds to: `C:\Users\<username>\.local\bin`
   - Set in PATH or session: `$env:Path = "C:\Users\<username>\.local\bin;$env:Path"`

2. **LLVM/Clang** (libclang for bindgen)
   - Required for generating Rust FFI bindings
   - Install: `winget install LLVM.LLVM`
   - Default location: `C:\Program Files\LLVM\bin`
   - Set environment variable: `$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"`
   - Permanent: `[Environment]::SetEnvironmentVariable("LIBCLANG_PATH", "C:\Program Files\LLVM\bin", "User")`

3. **Protocol Buffers Compiler (protoc)**
   - Required for IPC message code generation
   - Download: https://github.com/protocolbuffers/protobuf/releases (get `protoc-*-win64.zip`)
   - Extract to: `C:\protoc` (or any location)
   - Add to PATH: `$env:Path = "C:\protoc\bin;$env:Path"`

4. **Visual Studio 2022** (for Servo compilation)
   - Required components via Visual Studio Installer:
     - Windows 10/11 SDK (version ≥ 10.0.19041.0)
     - MSVC v143 - VS 2022 C++ x64/x86 build tools (Latest)
     - C++ ATL for latest v143 build tools (x86 & x64)
   - Can be installed during rustup setup or separately

#### Build Environment Setup

For a complete build session, set all required environment variables:

```powershell
# Add tools to PATH
$env:Path = "C:\Users\kim\.local\bin;C:\protoc\bin;$env:Path"

# Set libclang path for bindgen
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"

# Build polo (includes servo-runner)
cargo build -p polo --release
```

#### Permanent Configuration

To avoid setting variables each session, add them permanently:

```powershell
# Add to user PATH
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
[Environment]::SetEnvironmentVariable("Path", "$currentPath;C:\protoc\bin;C:\Users\kim\.local\bin", "User")

# Set LIBCLANG_PATH
[Environment]::SetEnvironmentVariable("LIBCLANG_PATH", "C:\Program Files\LLVM\bin", "User")
```

Then restart your shell for changes to take effect.

## Updating

To update servo-runner when servo-gtk is updated:

1. Check the Servo git revision used by the new servo-gtk version (in its `Cargo.toml`)
2. Clone the updated servo-gtk repository
3. Copy `servo_runner/` directory contents to `servo_runner/src/` (preserving structure)
4. Update `libservo` and `embedder_traits` git revisions in `servo_runner/Cargo.toml` to match servo-gtk
5. Update version comments in `Cargo.toml`
6. Test build with `cargo build --release --bin servo-runner`
7. Test runtime with `cargo run --release -p polo`

**Important**: Servo is pre-1.0 (v0.0.x) with frequent breaking API changes. Always verify compatibility before updating.
