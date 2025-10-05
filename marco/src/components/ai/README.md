# AI component

Purpose

Provide in-editor assistance (summaries, rewrite suggestions, linting, and transformation suggestions) using optional external models or local adapters.

Where to put code

- Implement component code in `src/components/ai/`.
- Keep network calls off the UI thread; use Rust async tasks and channels to apply edits on the main loop.

Minimum API contract

- A `AiAssistant` trait should expose:
  - `fn analyze_range(&self, text: &str, start: usize, end: usize) -> Result<AiResponse>`
  - `fn suggest_edits(&self, text: &str, start: usize, end: usize) -> Result<Vec<Edit>>`

Testing

- Add unit tests that exercise the trait using a local mock adapter.
- Provide a small developer harness under `documentation/example/ai_example/` to run examples against sample documents.

Security & privacy

- Document where data is sent and provide explicit opt-in configuration for remote LLM use.
