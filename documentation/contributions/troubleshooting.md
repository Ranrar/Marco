# Troubleshooting

- **GTK CSS errors**: Ensure you run from the repository root so relative theme paths resolve. Check `marco-shared/src/assets/themes/` exists.
- **Missing icons**: Confirm `marco-shared/src/assets/icons/` is present and that `crate::paths::find_asset_root()` (or `MarcoPaths::new()`) finds the repo asset path.
- **Preview not updating**: Verify the buffer change signal is firing and that the core parser is working correctly. Check the WebKit6 console for base URI issues with local images.
- **Core parsing issues**: The Markdown engine lives in the external [`marco-core`](https://github.com/Ranrar/marco-core) crate. If markdown isn't rendering correctly, reproduce against `marco-core` directly and file the issue there.
- **Local images not displaying**: Ensure WebKit6 security settings are enabled and DocumentBuffer is providing correct base URIs for file:// protocol access.
- **Import errors**: Use `marco_core::` for parser / render / intelligence APIs, `marco_shared::` for buffer / paths / settings, and `crate::` for local modules within marco or polo binaries.
- **Rust Analyzer shows lots of "can't find crate ..."**: Make sure you opened the correct VS Code workspace for your OS.
  - On Linux use `marco-linux.code-workspace`.
  - On Windows use `marco-windows.code-workspace`.
  - If you set a Windows GNU target on Linux, you may hit `glib-sys` / `pkg-config` cross-compilation failures which cascade into many unrelated diagnostics.

If you hit a problem you can't resolve, open an issue with a short description, steps to reproduce, and the output of running the app in a terminal.
