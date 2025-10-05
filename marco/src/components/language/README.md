# Language component

Purpose

This component is responsible for language translations used in the UI (labels, menus, dialog text, and messages). It is separate from the language/syntax schema work used for Markdown flavors. Keep UI localization under this component.

Where to put translations and code

- Translation files (static assets) should live under `src/assets/language/<locale>/` (for example `src/assets/language/es/`).
- Place runtime/component code in `src/components/language/` (loader, fallback rules, formatting helpers).

File format and conventions

- Use simple key/value TOML files named `ui.toml` per-locale. Example:

```toml
[menu]
file = "File"
edit = "Edit"
[dialog]
open = "Open"
cancel = "Cancel"
```

- Keys should be namespaced (menu, dialog, status, footer, settings) to avoid collisions and aid translators.

Loading and fallback

- The `LocalizationProvider` should load the requested locale and fall back to `en` when keys are missing.
- Support runtime locale change by notifying UI components to refresh strings.

Minimal API contract

- Provide a small trait for the component, e.g.: 

```rust
pub trait LocalizationProvider {
  fn load_locale(&self, locale: &str) -> Result<(), Error>;
  fn t(&self, key: &str) -> String; // simple lookup with fallback
  fn current_locale(&self) -> String;
}
```

Testing

- Add a couple of locales in `src/assets/language/` and unit tests that verify fallback behavior and runtime switching.

Supported locales (examples)

- en (English)
- de (German)
- fr (French)
- es (Spanish)
- it (Italian)
- ja (Japanese)
- ko (Korean)
- and more — add other locales under `src/assets/language/<locale>/` as needed.

Notes

- Keep translation files small and easy to edit. Consider using simple CSV or spreadsheet exports for translators, but commit canonical TOML files into the repo.
-- Document any pluralization rules or formatting expectations in a short `documentation/language.md` file if the UI needs complex localization (dates, plural forms, number formats).

See also: `src/assets/language/language matrix.md`

## Implementation status

- Current implemented locale(s):
  - `en` (English) — implemented (see matrix)
- Many other locales are listed in `src/assets/language/language matrix.md` and can be implemented by contributors.

## How to add a new locale

1. Add a new directory `src/assets/language/<locale>/` and create `ui.toml` with translated keys. Use the existing `ui.toml` structure.
2. Add tests under `tests/language/<locale>.rs` (or a shared test harness) to verify key coverage and fallback behavior.
3. Update `src/assets/language/language matrix.md` with `✔` and your name/link for the implemented locale.
4. Open a pull request and reference the issue that requested or tracked the locale addition.

Small tip: Start with the most-used keys (menu, dialog, settings, footer) and expand coverage in subsequent PRs.

