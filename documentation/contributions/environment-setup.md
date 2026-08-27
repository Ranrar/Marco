# Environment setup

Quick, reliable steps to get a local dev build running on Linux and Windows.

## 1. Clone and enter the repo

```bash
git clone https://github.com/<your-user>/marco.git
cd marco
```

## 2. Rust toolchain

```bash
rustup show
```

This repo pins Rust in `rust-toolchain.toml`. If needed, install/update:

```bash
rustup update
```

## 3. Open the right VS Code workspace

See [Dev workspaces (VS Code)](workflow.md#dev-workspaces-vs-code) — use `marco-linux.code-workspace` on Linux and `marco-windows.code-workspace` on Windows.

## Linux dev setup

Install dependencies (Debian/Ubuntu):

```bash
sudo apt-get update
sudo apt-get install -y \
  python3 build-essential pkg-config \
  libgtk-4-dev libgtksourceview-5-dev libwebkitgtk-6.0-dev libfontconfig-dev \
  dpkg-dev fakeroot gzip
```

Build and run:

```bash
cargo run -p marco
```

Common checks:

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
```

Optional package build:

```bash
bash build/linux/debian/build_deb.sh --no-bump
```

## Windows dev setup (MSYS2 UCRT64)

Important: the Windows build flow in this repo uses MSYS2 UCRT64 + pkg-config.

Install [MSYS2](https://www.msys2.org/), then open the "MSYS2 UCRT64" shell and install required packages:

```bash
pacman -Syu
pacman -S --needed \
  mingw-w64-ucrt-x86_64-gtk4 \
  mingw-w64-ucrt-x86_64-gtksourceview5 \
  mingw-w64-ucrt-x86_64-librsvg \
  mingw-w64-ucrt-x86_64-cairo \
  mingw-w64-ucrt-x86_64-gdk-pixbuf2 \
  mingw-w64-ucrt-x86_64-pkg-config \
  mingw-w64-ucrt-x86_64-gcc \
  mingw-w64-ucrt-x86_64-binutils
```

Install the Rust GNU target (PowerShell or bash):

```bash
rustup target add x86_64-pc-windows-gnu
```

If running from PowerShell, set env vars for the current session:

```powershell
$env:PKG_CONFIG_PATH = "C:\msys64\ucrt64\lib\pkgconfig"
$env:PATH = "C:\msys64\ucrt64\bin;C:\msys64\usr\bin;$env:PATH"
```

Verify tool availability:

```bash
pkg-config --version
rustup target list --installed
```

Build and run:

```bash
cargo run -p marco
```

If you still hit linker/pkg-config issues, run the repo helper script:

```powershell
.\build\windows\test_ci_locally.ps1
```

Portable package build:

```powershell
.\build\windows\build_portable.ps1
```

## First troubleshooting checklist

If build fails on Windows with "pkg-config not found":

1. Confirm you installed `mingw-w64-ucrt-x86_64-pkg-config` in MSYS2 UCRT64.
2. Confirm `C:\msys64\ucrt64\bin` is on `PATH` in the shell where you run `cargo`.
3. Confirm `PKG_CONFIG_PATH` points to `C:\msys64\ucrt64\lib\pkgconfig`.
4. Reopen the terminal after changing `PATH`.

If build fails on Linux with missing GTK/WebKit headers:

1. Reinstall the distro packages listed above.
2. Run: `pkg-config --modversion gtk4`

## Daily commands

```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace
```

Run app binaries:

```bash
cargo run -p marco
cargo run -p polo
```

See also: [Quickstart & dev commands](quickstart-commands.md) for release-mode build/run commands.
