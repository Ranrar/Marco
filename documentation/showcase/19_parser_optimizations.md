# Parser Optimisation Smoke Test

This document exercises every code path touched by the five optimisations
shipped in marco-core 1.1. Nothing is removed — all standard Markdown features
must continue to render correctly.

---

## 1 — Multi-line Paragraphs (Newline Scan)

The span-to-position conversion now uses a single-pass loop instead of two
separate `O(n)` scans. Each block below spans several source lines and
contains multiple AST nodes, so the optimised path is hit on every node.

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor
incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis
nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.

Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore
eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt
in culpa qui officia deserunt mollit anim id est laborum.

Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac
turpis egestas. Vestibulum tortor quam, feugiat vitae, ultricies eget, tempor
sit amet, ante. Donec eu libero sit amet quam egestas semper.

Aenean ultricies mi vitae est. Mauris placerat eleifend leo. Quisque sit amet
est et sapien ullamcorper pharetra. Vestibulum erat wisi, condimentum sed,
commodo vitae, ornare sit amet, wisi.

---

## 2 — Inline Fast-Path (Prose Characters)

The inline dispatcher now skips ~25 parser attempts for bytes that cannot start
a special sequence. The paragraphs below are dense prose with commas, periods,
digits, and mixed case — the exact bytes the fast-path covers.

The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor
jugs. How valiantly the zealous monks quaff the expensive burgundy. Sixty zippers
were quickly picked from the woven jute bag.

Paragraph 2, sentence 1 ends with a number: 42. Sentence 2 has digits inline:
the year 2024 was notable. Sentence 3: version 1.1.0 ships today, March 5 2026.

Plain text mixed with edge-case bytes: C++ templates use angle brackets but only
within code spans. URLs like https://example.com are autolinks. Email foo@bar.com
is also an autolink.

Normal prose should not trigger special parsers: hello world, foo bar baz, one
two three four five six seven eight nine ten eleven twelve thirteen fourteen
fifteen sixteen seventeen eighteen nineteen twenty.

---

## 3 — Hard Line Breaks (Space Fast-Path Edge Case)

Two trailing spaces create a hard break. The fast-path must NOT skip spaces
before `\n` when there are two or more — otherwise hard breaks would be lost.

Line one ends with two trailing spaces  
Line two continues here.

Another pair:  
First line.  
Second line.  
Third line.

Backslash hard break also works:\
This is on a new line after a backslash.

---

## 4 — Reference Links (Label Normalisation)

`normalize_label` now builds the output string directly without an intermediate
`Vec<&str>`. The cases below stress whitespace collapsing in labels.

Single-word labels: [Rust][rust], [GTK4][gtk4], [nom][nom].

Multi-word labels with extra spaces in the definition:

[CommonMark spec][commonmark] · [GitHub Flavored Markdown][gfm] · [WebKit][webkit]

Collapsed reference: [commonmark][] · [gfm][]

Shortcut reference: [rust]

Case-insensitive label matching: [RUST][RUST] and [Rust][rust] resolve the same.

[RUST]: https://www.rust-lang.org

[rust]: https://www.rust-lang.org
[gtk4]: https://gtk.org
[nom]: https://github.com/rust-bakery/nom
[commonmark]: https://commonmark.org
[gfm]: https://github.github.com/gfm/
[webkit]: https://webkit.org

---

## 5 — All Six Heading Levels (Renderer Buffer)

The heading renderer now uses `char::from_digit()` instead of `to_string()`,
avoiding two heap allocations per heading. All six levels must render correctly.

# Heading Level 1
## Heading Level 2
### Heading Level 3
#### Heading Level 4
##### Heading Level 5
###### Heading Level 6

Setext headings also exercise the same rendering path:

Setext Level 1
==============

Setext Level 2
--------------

---

## 6 — Fenced Code Blocks (No `format!`)

The class attribute for fenced code blocks is now built with `push_str` chains.
Every language tag below must appear as `class="language-<lang>"` in the output.

```rust
fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
```

```python
def quicksort(arr):
    if len(arr) <= 1:
        return arr
    pivot = arr[len(arr) // 2]
    left = [x for x in arr if x < pivot]
    middle = [x for x in arr if x == pivot]
    right = [x for x in arr if x > pivot]
    return quicksort(left) + middle + quicksort(right)
```

```bash
#!/usr/bin/env bash
set -euo pipefail
echo "Building workspace…"
cargo build --workspace
```

```toml
[workspace]
resolver = "3"
members = ["marco", "polo", "marco-shared"]
```

A code block with no language tag (no class attribute should be emitted):

```
plain text block
no language tag
```

Inline code is unaffected: `let x = 42;` and `cargo run -p marco`.

---

## 7 — Footnotes (Renderer Push-Str Chain)

Footnote list items (`<li id="fn{n}">`) are now built with `push_str` chains.
Multiple footnotes exercise incrementing IDs.

First reference.[^alpha] Second reference.[^beta] Third reference.[^gamma]
Fourth reference.[^delta] Fifth reference.[^epsilon]

Inline footnote — no separate definition needed.^[This footnote is defined
inline. It contains *italic*, **bold**, and `code` to exercise inline parsing
inside footnote content.]

Rich-content footnote with continuation lines.[^long]

[^alpha]: First footnote — `fn1` in the rendered list.
[^beta]: Second footnote — `fn2` in the rendered list.
[^gamma]: Third footnote. Contains a [link](https://commonmark.org).
[^delta]: Fourth footnote. Contains `inline code`.
[^epsilon]: Fifth footnote — the last numbered item.
[^long]: Opening line of the long footnote.
    Continuation line 1, indented with four spaces.
    Continuation line 2, still part of the same definition.

---

## 8 — Inline Capacity Hint (`Vec::with_capacity(8)`)

`parse_inlines_from_span` pre-allocates 8 slots. Paragraphs with more than 8
inline nodes stress the growth path; paragraphs with fewer confirm the hint
does not break anything.

### Fewer than 8 inline nodes

**Bold** and *italic* and `code`.

### Exactly 8 inline nodes

**one** *two* `three` ~~four~~ ==five== **six** *seven* `eight`

### More than 8 inline nodes (growth path)

**one** *two* `three` ~~four~~ ==five== **six** *seven* `eight` **nine** *ten*
`eleven` ~~twelve~~ ==thirteen== **fourteen** *fifteen* `sixteen` **seventeen**
*eighteen* `nineteen` ~~twenty~~

### Deeply nested inline spans

**Bold with *italic and `code` inside* and more bold** — then plain prose —
then *italic with ==highlight== inside* — then ~~strike with **bold** inside~~.

---

## 9 — Mixed Block Types (All Paths Together)

This section combines every block type so the parser processes them in
sequence, hitting each optimised path in a single parse run.

> **Blockquote** with *inline formatting*, a [reference link][rust], and
> `inline code`. The prose is long enough to trigger the fast-path.
>
> Second blockquote paragraph. Numbers: 1, 2, 3. Punctuation: commas, periods.

| Feature | Before | After | Gain |
|---|---|---|---|
| Small workload | 6 472 ns | 5 003 ns | −23% |
| Medium workload | 146 263 ns | 91 524 ns | −37% |
| Large workload | 2 727 867 ns | 2 103 670 ns | −23% |
| Pathological | 11 516 501 ns | 9 127 307 ns | −21% |
| GFM spec | 433 231 ns | 361 778 ns | −16% |
| Marco spec | 934 192 ns | 849 928 ns | −9% |

- Plain list item with prose: the quick brown fox.
- Item with **bold** and *italic* and `code`.
- Item with a [link](https://github.com/Ranrar/Marco).
- Item with a footnote reference.[^table-fn]

1. Ordered item one — plain prose.
2. Ordered item two — **bold**, *italic*.
3. Ordered item three — `code`, ~~strike~~.

[^table-fn]: Footnote from inside a list item, exercising the renderer path.

---

## 10 — Autolinks and GFM Autolink Literals

Plain autolink: <https://www.rust-lang.org>

Email autolink: <user@example.com>

GFM autolink literals (no angle brackets):

Visit https://www.rust-lang.org for Rust documentation.

Send mail to user@example.com for support.

---

*End of parser optimisation smoke test. All blocks above must render without
errors, broken output, or missing content.*
