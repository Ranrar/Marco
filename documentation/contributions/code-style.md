# Code style and expectations

- Keep UI code in `marco/src/components/` and `marco/src/ui/`.
- Keep shared, GTK-free application logic in `marco-shared/src/` (buffer management, settings, paths, loaders).
- Pure parser / renderer / intelligence logic lives in the external [`marco-core`](https://github.com/Ranrar/marco-core) crate — contribute parser changes there.
- Follow Rust idioms and project patterns (use `Result<T, E>`, avoid panics in library code, document public APIs).
- Add unit tests under the appropriate module; integration tests for the apps go under `marco/tests/` or `polo/tests/` if/when needed (this repo no longer hosts the parser test suite).
