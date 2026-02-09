# Localization (UI language)

Marco's UI localization lives in the Marco crate under:

- Code: `marco/src/components/language/`
- Translation assets: `assets/language/*.toml`

This is separate from Markdown syntax/language work (parsing, highlighting, etc.).

For the canonical, up-to-date implementation, see:

- `marco/src/components/language/mod.rs` (loader, locale scanning, per-key fallback)
- `assets/language/en.toml` (reference schema / template)

## Locale files (TOML)

- Locale code must be **ISO 639-1**: exactly 2 lowercase ASCII letters (e.g. `en`, `de`).
- Files are named: `assets/language/{code}.toml`
- Each locale file should include:
  - `[language] code = ".."`
  - `[language] native_name = ".."` (used in the language selector)

Example (partial):

```toml
[language]
code = "en"
native_name = "English"

[menu]
file = "File"
edit = "Edit"

[settings.tabs]
editor = "Editor"
language = "Language"
```

## Loading and fallback

- The localization manager loads the requested locale at runtime.
- Missing keys fall back **per key** to built-in English defaults (so incomplete translations don't crash the UI).

## How to add a new locale

1. Copy `assets/language/en.toml` to `assets/language/{code}.toml`.
2. Translate values, keeping keys unchanged.
3. Ensure `[language.native_name]` is set.
4. Update `assets/language/language_matrix.md`.

Reference: ISO 639-1 codes
https://en.wikipedia.org/wiki/List_of_ISO_639_language_codes