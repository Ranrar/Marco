# Localization (UI language)

Marco's UI localization lives in the Marco crate under:

- Code: `marco/src/components/language/`
- Translation assets: `marco-shared/src/assets/language/*.toml`

This is separate from Markdown syntax/language work (parsing, highlighting, etc.).

For the canonical, up-to-date implementation, see:

- `marco/src/components/language/mod.rs` (loader, locale scanning, per-key fallback)
- `marco-shared/src/assets/language/en.toml` (reference schema / template)

## Locale files (TOML)

- Locale code must be **ISO 639-1**: exactly 2 lowercase ASCII letters (e.g. `en`, `de`).
- For macrolanguages with major script/dialect splits (e.g. Chinese), the code may
  optionally carry an **ISO 3166-1 alpha-2 region subtag**, written as
  `{lang}-{REGION}` (BCP 47 style): lowercase 2-letter language, hyphen, uppercase
  2-letter region — e.g. `zh-CN` (Simplified) / `zh-TW` (Traditional). Plain
  underscore (`zh_CN`) or lowercase-region (`zh-cn`) forms are rejected.
- Files are named: `marco-shared/src/assets/language/{code}.toml`
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

Region-qualified example (partial):

```toml
[language]
code = "zh-CN"
native_name = "简体中文"
```

## Loading and fallback

- The localization manager loads the requested locale at runtime.
- Missing keys fall back **per key** to built-in English defaults (so incomplete translations don't crash the UI).
- `SimpleLocalizationManager::load_locale_with_fallback` cascades a whole-locale load: try the requested code as-is, then (if it's region-qualified) its bare language, then English — always making exactly one attempt at English as the final step (not zero, if the requested code was already `en`; not two, if the bare-language retry was already `en`). Use this instead of the plain `load_locale` when resolving a user- or system-selected locale, so an unavailable regional variant degrades gracefully instead of jumping straight to English.
- System-locale auto-detection (`detect_system_locale_bcp47`, used for "System Default" in Settings → Language) preserves the region subtag when the OS/environment reports one — `zh_CN` → `zh-CN`, and Windows' `zh-Hans-CN` / `zh-Hant-TW` script-tagged forms are also resolved to `zh-CN` / `zh-TW`. Locale strings that carry a region-less script subtag only (e.g. bare `zh-Hans`) fall back to the bare language.

## How to add a new locale

1. Copy `marco-shared/src/assets/language/en.toml` to `marco-shared/src/assets/language/{code}.toml`, where `{code}` is a bare ISO 639-1 code, or `{lang}-{REGION}` if the language needs a regional variant (see above).
2. Translate values, keeping keys unchanged.
3. Ensure `[language.native_name]` is set.
4. Add yourself to `marco-shared/src/assets/language/language_matrix.md`: move the language's row from "Not yet translated" to "Done" and credit yourself as Author/Contributor. This is a required part of the contribution, not optional — a locale file without a matrix entry is an incomplete PR.

Reference: ISO 639-1 codes
https://en.wikipedia.org/wiki/List_of_ISO_639_language_codes

## Adding a new translatable string

Most UI text is already wired up, but a few surfaces are still hardcoded regardless of
locale (see [CONTRIBUTING.md](../../CONTRIBUTING.md) for the current list). If you're
translating a string and it has no key in `en.toml`, wire it up first — a translation
key touches four places, all of which must agree on the exact key path:

1. **`marco/src/components/language/mod.rs`** — add the field to the relevant
   `*Translations` struct, and add the matching `Self::get_string(value, &["section",
   "key"], &fallback....)` call in `load_translations_from_value`.
2. **`marco/src/components/language/default_translations.rs`** — add the English
   fallback value used before any TOML loads and whenever a locale file is missing
   the key.
3. **`marco-shared/src/assets/language/en.toml`** — add the key under the matching
   `[section]`, with the same English text as the default above.
4. **The call site** — replace the hardcoded literal with a read from the
   `Translations` struct (e.g. `&translations.messages.my_new_key`). Check whether the
   surrounding widget already has a `Translations` (or a narrower sub-struct) in
   scope; if not, either thread it through from the nearest caller that has one, or —
   for a one-off dialog/action not on a hot path — call
   `crate::ui::dialogs::current_translations()` directly, which resolves the active
   locale on demand without threading state through the call chain.

If the widget's text needs to update immediately when the user switches language at
runtime (not just at next launch), also wire it into the relevant
`update_*_translations` function (e.g. `update_menu_translations`,
`update_footer_translations`) called from the language-switch handler in
`marco/src/main.rs`. A one-off dialog opened fresh each time (e.g. via
`current_translations()`) doesn't need this — it always resolves the current locale
when it's opened.

A test worth copying: `smoke_test_newly_added_keys_load_from_toml` in
`marco/src/components/language/mod.rs` loads a full set of new keys from a
distinct-from-default TOML fixture and asserts each one round-trips — this catches a
typo'd key path that would otherwise silently fall back to the (identical-looking)
English default and go unnoticed.