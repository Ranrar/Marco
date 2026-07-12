# Quickstart & dev commands

Build all crates:

```bash
cargo build --release --workspace
```

Build specific crates:

```bash
cargo build --release -p marco-shared # Shared library only
cargo build --release -p marco        # Full editor
cargo build --release -p polo         # Viewer only
```

Run the full editor (development):

```bash
cargo run --release -p marco
```

Run the viewer only:

```bash
cargo run --release -p polo
```

Run tests for all crates:

```bash
cargo test --workspace --lib --tests -- --nocapture
```

The parser/renderer test suite lives in the [`marco-core`](https://github.com/Ranrar/marco-core) repository — clone it separately and run `cargo test` there to validate parser behavior.
