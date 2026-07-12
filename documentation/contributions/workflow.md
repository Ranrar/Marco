# Suggested workflow

1. Open an issue describing the change or bug you want to address.
2. Fork the repository and create a feature branch.
3. Add tests where appropriate and keep changes small and focused.
4. If your change involves the Markdown grammar / parser / renderer, please file it against the [`marco-core`](https://github.com/Ranrar/marco-core) repository instead — that engine lives there.
5. Run `cargo build` and `cargo test` locally.
6. Open a pull request describing the change and link the related issue.

## Dev workspaces (VS Code)

This repo includes two VS Code workspace files. Use the one that matches your **native OS**:

- **Linux**: `marco-linux.code-workspace`
  - Uses Rust Analyzer + `clippy` on save.
- **Windows (MSVC)**: `marco-windows.code-workspace`
  - Configures Rust Analyzer to use the `x86_64-pc-windows-msvc` target.

> Note: We intentionally avoid a "Windows GNU cross-compile from Linux" workspace because GTK/Glib dependencies require a full cross sysroot + `pkg-config` setup. If you point Rust Analyzer at `x86_64-pc-windows-gnu` on Linux you will likely see `glib-sys` / `pkg-config` cross-compilation errors and cascading "can't find crate ..." diagnostics.
