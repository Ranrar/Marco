# Collaboration component

Purpose

Add real-time collaborative editing support (Yjs / CRDT) that integrates with Marco's `DocumentBuffer` and UI.

Quick integration notes

- Implement component code in `src/components/collab/`.
- Provide a `CollabBackend` trait that supports: `connect`, `disconnect`, `apply_remote_ops`, and `get_local_patch`.
- The component should emit events the UI can subscribe to for remote cursor and presence updates.

References

- Language component: `../language/README.md` — language schemas and AST validation should be compatible with collab updates; ensure schema-aware merge behavior if required.

Testing

- Include unit tests for concurrent patch application and a small in-process integration test that runs two backends exchanging patches.

Design notes

- Treat remote edits as regular edits and ensure undo/redo handles them gracefully.
- Consider keeping the collab transport layer behind a small trait so multiple transports (WebSocket, local IPC, or direct in-memory for tests) are possible.
