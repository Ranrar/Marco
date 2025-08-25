This folder contains developer tooling. You can implement new tools in Rust here.

Sample tool: `tool_runner`

To build and run the sample tool from the repo root:

```bash
cd tools/tool_runner
cargo run
```

From VS Code: open `marco2.tools.code-workspace` and run the provided task `cargo run:tool_runner`.

Notes:
- The repo already contains a Python-based tool under `tools/ast_syntax_checker`. The new Rust tools can coexist; pick the language per tool.
- If your Rust tools need to access workspace-level files (Cargo.toml at repo root), consider adding a `Cargo.toml` in `tools/` to manage a workspace or use `cargo --manifest-path` where needed.
