# Definition Lists

Definition lists let you pair terms with one or more definitions — useful for glossaries, API references, and FAQ sections.

## Basic Syntax

Term
: Definition

---

## Single-Definition Terms

HTML
: HyperText Markup Language — the standard markup language for web pages.

CSS
: Cascading Style Sheets — a language for describing the presentation of HTML documents.

JavaScript
: A lightweight, interpreted programming language with first-class functions.

---

## Multiple Definitions per Term

Rust
: A systems programming language focused on safety, speed, and concurrency.
: Developed by Mozilla Research, first stable release in 2015.
: Used to build Marco's parser, renderer, and UI.

GTK
: GIMP Toolkit — a free and open-source widget toolkit for creating GUIs.
: Available on Linux, Windows, and macOS.

---

## Terms with Inline Formatting

**CommonMark**
: A strongly-defined, highly compatible specification of Markdown, maintained at https://commonmark.org.

*nom*
: A parser combinator library for Rust. Used in the `marco-core` crate to build Marco's hand-crafted Markdown parser.

`RON`
: Rusty Object Notation — a human-readable configuration format. Marco stores settings as `.ron` files.

---

## Complex Definitions

Terms can have multi-line definitions with continuation lines (indented):

Parser
: A component that reads input text and produces a structured representation (AST).
  In Marco, the parser takes Markdown source text and produces a `Document` node
  tree that the renderer then converts to HTML.

AST
: Abstract Syntax Tree — a tree-like data structure that represents the syntactic
  structure of source text without including every token from the original input.
  Marco's AST is defined in the `marco-core` crate (`parser/ast.rs`).

---

## Definitions with Lists

Markdown Flavors
: **CommonMark** — the base specification
  - 652 spec tests, all passing in Marco
  - Defines block and inline parsing rules
: **GFM (GitHub Flavored Markdown)** — a CommonMark superset
  - Adds tables, task lists, strikethrough, autolink literals
: **Marco Extensions** — project-specific additions
  - Tab blocks, slider decks, inline footnotes, mark, super/subscript

---

## Definitions with Code

`cargo test`
: Runs the test suite for the current crate. With `--workspace` it tests all crates.

  ```bash
  cargo test --workspace              # all tests
  cargo test --workspace -- --nocapture  # show println! output
  cargo test -p core                  # core crate only
  ```

`cargo clippy`
: Runs the Clippy linter to catch common mistakes and style issues.

  ```bash
  cargo clippy --workspace --all-targets
  ```

---

## Glossary Example

API
: Application Programming Interface — a set of definitions and protocols for building and integrating applications.

CLI
: Command-Line Interface — a text-based interface for interacting with software.

GUI
: Graphical User Interface — a visual interface using windows, icons, and menus.

IDE
: Integrated Development Environment — software providing comprehensive facilities to programmers.

LSP
: Language Server Protocol — a protocol for providing language features (completion, diagnostics, hover) between tools.

WASM
: WebAssembly — a binary instruction format for a stack-based virtual machine, designed to be a compilation target.
