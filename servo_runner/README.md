# servo-runner

This directory contains the servo-runner binary, which is a subprocess spawned by polo's Servo-based WebView.

## Architecture

Polo uses servo-gtk for web rendering, which implements a two-process architecture:
- **polo (GTK process)**: The main application with GTK4 UI and servo-gtk WebView widget
- **servo-runner (rendering process)**: A subprocess that runs the Servo web engine

The two processes communicate via IPC using Protocol Buffers over stdin/stdout pipes.

## Source

This code is vendored from the [servo-gtk](https://github.com/nacho/servo-gtk) repository.

## Files

- `src/main.rs`: Main servo-runner entry point (originally `servo_runner/runner.rs`)
- `src/resource_reader.rs`: GResource reader for Servo assets
- `build.rs`: Build script for gresource compilation and protobuf code generation
- `proto/ipc.proto`: Protocol Buffers schema for IPC messages
- `resources/`: Servo resources compiled into binary via gresource

## Building

servo-runner is built as part of the marco workspace:

```bash
cargo build --bin servo-runner
```

## Usage

servo-runner is not meant to be run directly. It is spawned automatically by polo when creating a Servo WebView.

The servo-gtk library launches it via:
```rust
cargo run --bin servo-runner
```

## Updating

To update servo-runner when servo-gtk is updated:

1. Clone the updated servo-gtk repository
2. Copy `servo_runner/` directory contents (preserving structure)
3. Update Servo git revision in `Cargo.toml` if needed
4. Rebuild the workspace
