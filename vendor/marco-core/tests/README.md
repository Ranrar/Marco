# tests

Integration tests for `marco-core`. Each `.rs` file is a separate `cargo test`
binary that exercises only the public API re-exported from `lib.rs`.

See [`Documentation/testing.md`](../Documentation/testing.md) for:

- The full test inventory (93 integration tests across 23 files)
- CommonMark conformance numbers and how to interpret them
- Regression-guard thresholds and strict/verbose modes
- Conventions for adding new tests

## Running

```bash
# All tests (unit + integration + doc tests)
cargo test --locked

# A single integration binary
cargo test --test commonmark_spec_it --locked

# Strict CommonMark conformance mode
MARCO_SPEC_STRICT=1 cargo test --test commonmark_spec_it --locked

# Print failing spec examples
MARCO_SPEC_VERBOSE=1 cargo test --test commonmark_spec_it --locked
```

## Fixtures

Spec fixture JSON files live in [`tools/spec/`](../tools/spec/README.md) and
are loaded by `commonmark_spec_it.rs` via `include_str!`.
