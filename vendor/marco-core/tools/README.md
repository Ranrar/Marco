# tools

Developer tooling for `marco-core`. None of these crates are published to
crates.io (`publish = false`).

| Directory | Purpose |
|---|---|
| [`marco-ast/`](marco-ast/README.md) | AST introspection CLI — inspect parse trees, HTML output, and intelligence results |
| [`perf-lab/`](perf-lab/README.md) | Performance and regression benchmarking harness |
| [`spec/`](spec/README.md) | CommonMark and extension spec fixture JSON files |
| [`tests/`](tests/README.md) | Extension spec conformance tests (loaded by `perf-lab`) |

## Building

Each tool is a standalone crate with its own `Cargo.toml`. Build from the
repo root using `--manifest-path`:

```bash
# marco-ast
cargo build --manifest-path tools/marco-ast/Cargo.toml

# perf-lab
cargo build --manifest-path tools/perf-lab/Cargo.toml --release
```

## Running tests

```bash
# Extension spec conformance (tools/tests/)
cargo test --manifest-path tools/perf-lab/Cargo.toml --test extension_spec_it

# marco-ast integration tests
cargo test --manifest-path tools/marco-ast/Cargo.toml
```
