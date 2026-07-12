# Contributing to Marco

Thank you for your interest in contributing to Marco. This document is a short index — the actual guides live under [documentation/contributions/](documentation/contributions/).

Marco is developed cross-platform, but the day-to-day workflow is **native per OS** (Linux builds on Linux, Windows builds on Windows).

We welcome contributions of all sizes: bug fixes, new editor features, additional themes, documentation improvements, translations, and core parser enhancements. If your change involves the Markdown grammar / parser / renderer, please file it against the [`marco-core`](https://github.com/Ranrar/marco-core) repository instead — that engine lives there.

**Just here to add or fix a translation?** → [Localization (UI language)](documentation/contributions/language.md)

## Guide

- [Suggested workflow](documentation/contributions/workflow.md) — the PR process, plus VS Code workspace setup
- [Environment setup](documentation/contributions/environment-setup.md) — Linux & Windows dev environment, daily commands, first troubleshooting checklist
- [Code style and expectations](documentation/contributions/code-style.md)
- [Architecture](documentation/contributions/architecture.md) — workspace structure, Polo notes, marco-core layout, how the app works, embedding/API, configuration file locations
- [Localization (UI language)](documentation/contributions/language.md) — locale-code rules, adding a locale, wiring up a new translatable string
  - [language_matrix.md](marco-shared/src/assets/language/language_matrix.md) — coverage & contributors
- [Themes](documentation/contributions/themes.md) — adding HTML preview themes and editor style schemes
- [Quickstart & dev commands](documentation/contributions/quickstart-commands.md) — release-mode build/run/test commands
- [Troubleshooting](documentation/contributions/troubleshooting.md)

If you hit a problem you can't resolve, open an issue with a short description, steps to reproduce, and the output of running the app in a terminal.

## High-value contributions

These are areas where an implemented contribution will have big impact. If you plan to work on any of these, open an issue first so we can coordinate and reserve the scope.

### Easy high-value contributions

No new architecture needed — these follow an existing, documented pattern and are a great way to make a first real contribution.

- **Localization**
  Translate the UI into a new language, add a missing regional variant, or wire up one of the still-hardcoded strings. See [Localization (UI language)](documentation/contributions/language.md).

- **Themes**
  Add an HTML preview theme or an editor syntax-highlighting scheme. Both are auto-discovered from their asset folder — no code changes needed. See [Themes](documentation/contributions/themes.md).

### Collaborative editing (Yjs / CRDT)
- **Goal**: Add a shared-document component that syncs buffer state across peers using a CRDT backend (Yjs, automerge, or similar).
- **Integration points**:
  - Create a new `marco/src/components/collab/` module that implements a `CollabBackend` trait (connect, disconnect, apply_remote_ops, get_local_patch).
  - Wire the component into the editor buffer event loop: when the local buffer changes, the component should produce and broadcast a patch; when remote patches arrive, they should be applied to the `DocumentBuffer` using documented public update methods.
  - Respect existing undo/redo and cursor/selection synchronization: treat remote changes as first-class edits and emit events the UI can use to update cursors.
- **Testing notes**: add unit tests for concurrent patches, and an integration test using two in-process backends that exchange patches.
- **Reference**: [Collaboration component](documentation/contributions/collab-component.md) — integration notes and references

### AI-assisted tools
- **Goal**: Provide a component API and example component that offers in-editor assistance (summaries, rewrite suggestions, universal spell checking, autocorrect).
- **Integration points**:
  - Define a `marco/src/components/ai/` interface that accepts a text range and returns suggested edits or annotations. Keep the component optional and behind a feature flag or runtime toggle.
  - Provide a small example implementation that uses an HTTP-based LLM adapter (local or remote) and demonstrates non-blocking requests using async tasks; always run requests off the UI thread and apply edits on the main loop.
  - Offer a CLI or developer test harness under `tests/ai/` to run the component against sample documents.
- **Security & privacy notes**: document privacy expectations clearly. Components that call external APIs must expose where data is sent and provide opt-in configuration.
- **Reference**: [AI component](documentation/contributions/ai-component.md) — guidance and interface notes

If you add a new component folder, add its doc under `documentation/contributions/` (e.g. `documentation/contributions/{component}-component.md`) rather than a `README.md` inside the component folder — this keeps all contributor-facing docs in one place. Explain the contract, tests, and how to run the component's dev harness.