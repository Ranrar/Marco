# Marco - Markdown Composer

A simple markdown editor built with Rust and GTK4.

## Features

- Split-pane view with markdown source on the left and preview on the right
- Syntax highlighting for markdown
- File operations (New, Open, Save)
- Real-time preview updates as you type

## Prerequisites

Before building this application, you need to install Rust and GTK4 development libraries.

### Install Rust

First, install Rust using rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Install GTK4 Development Libraries

### Ubuntu/Debian:
```bash
sudo apt update
sudo apt install libgtk-4-dev libgtksourceview-5-dev build-essential
```

### Fedora:
```bash
sudo dnf install gtk4-devel gtksourceview5-devel
```

### Arch Linux:
```bash
sudo pacman -S gtk4 gtksourceview5
```

## Building and Running

1. Clone or navigate to this directory
2. Build and run the application:

```bash
cargo run
```

## Usage

- **New**: Clear the editor to start a new document
- **Open**: Open an existing markdown file
- **Save**: Save the current document
- Type markdown in the left pane and see the live preview in the right pane

## Dependencies

- `gtk4` - GTK4 bindings for Rust
- `sourceview5` - GTK SourceView for syntax highlighting
- `pulldown-cmark` - Markdown parser
- `glib` - GLib bindings

## Project Structure

- `src/main.rs` - Application entry point and window setup
- `src/editor.rs` - Main editor widget with split-pane layout
- `Cargo.toml` - Project dependencies and metadata
- `build.rs` - Build script for GTK resources
