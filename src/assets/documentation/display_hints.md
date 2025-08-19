display_hints.ron — schema-driven UI hints for markdown syntax

Purpose
-------
Place an optional `display_hints.ron` in a schema directory alongside `syntax.ron` to provide a map of `node_type` -> ordered list of preferred capture names. The parser will load this file (if present) and `MarkdownSyntaxMap::build_display_hints()` will return these hints. The footer and other UI consumers can use the hints to pick which capture to display for a syntax token.

Format
------
The file is a RON map literal mapping string keys to arrays of strings. Example:

(
    "frontmatter": ["value"],
    "heading": ["text"],
    "image-size": ["w"],
    "video": ["id1", "id2"],
    "link-target": ["h", "t"],
    "definition": ["desc"],
)

Guidelines for building `display_hints.ron`
------------------------------------------
1. Identify `node_type` values used by your `syntax.ron` entries. These are the `node_type` fields in your rules. Use those exact strings as keys.
2. For each `node_type`, choose one or more preferred capture names. These should match named capture groups used in the `re:` patterns in `syntax.ron` (for example `(?P<id>...)`, `(?P<w>\d+)`).
3. Order captures by preference. The UI will pick the first capture that exists on a token.
4. If your regexes use unnamed groups, consider adding a named capture to make UI display deterministic.

Placement
---------
Put `display_hints.ron` in the same directory as `syntax.ron` (e.g. `src/assets/markdown_schema/Marco/display_hints.ron`). The loader will look for this file when a schema is loaded.

Examples
--------
- For an HTML image size rule that captures width as `(?P<w>\d+)`, use:
  "image-size": ["w"]
- For a Youtube embed where the ID appears in two groups `id1` and `id2`, prefer `id1` then `id2`:
  "video": ["id1", "id2"]

Notes
-----
- This is intentionally data-driven so different schemas can provide tailored hints.
- Do not include comments or trailing commas that are invalid in RON (simple map as above is safe).
- The parser logs a debug message when `MARCO_DEBUG_PARSER` is set and parsing fails; use that to diagnose syntax issues in `display_hints.ron`.

If you later want an automated helper, you can generate a suggested `display_hints.ron` by enumerating the named capture groups in each compiled regex rule; but per your request, no helper was added here — just the docs and the example file.
