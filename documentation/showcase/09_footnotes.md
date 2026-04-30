# Footnotes

Marco supports two footnote syntaxes: GitHub-style reference footnotes and inline footnotes.

---

## GitHub-Style Footnotes

Define a reference with `[^label]` in the text, and the definition anywhere in the document with `[^label]: content`.

Here is a sentence with a footnote.[^1]

Another sentence with a named footnote.[^note]

A third sentence with a second named footnote.[^ref]

[^1]: This is the first footnote.
[^note]: Named footnotes keep the source readable.
[^ref]: Footnote definitions can appear anywhere — even at the end of the document.

---

## Inline Footnotes (Marco Extension)

The `^[content]` syntax defines the footnote at the reference point — no separate definition needed.

This is a paragraph with an inline footnote.^[Inline footnotes include the content right where you write the reference. They are collected and rendered at the end of the document, just like reference footnotes.]

Complex inline footnote with formatting.^[*Italic*, **bold**, and `inline code` all work inside inline footnotes.]

---

## Footnotes with Rich Content

Reference footnotes can span multiple lines. Continuation lines are indented with 4 spaces.

This sentence references a long footnote.[^long]

[^long]: First line of the definition.
    Second line (indented with 4 spaces) continues the same footnote.
    Third line also part of the same definition.

---

## Multiple Footnotes in a Section

The CommonMark spec[^cm] and GFM spec[^gfm] define the baseline. Marco's parser[^parser] extends both.

[^cm]: CommonMark Spec v0.31.2 — https://spec.commonmark.org/0.31.2/
[^gfm]: GitHub Flavored Markdown Spec — https://github.github.com/gfm/
[^parser]: Marco uses a hand-crafted nom-based parser. Source in the `marco-core` crate.

---

## Forward References

Footnote references can appear before their definitions.

This uses a forward-defined footnote.[^forward]

Content continues here, and the definition appears later.

[^forward]: This definition appears after the reference in the source — it still works correctly.

---

## Unicode Labels

Footnotes support Unicode labels.

A reference with a Japanese label.[^参考]

[^参考]: Unicode labels work for any script.

---

## Footnotes in Lists

- First item with a footnote[^list1]
- Second item with another footnote[^list2]

[^list1]: Footnote for the first list item.
[^list2]: Footnote for the second list item.

---

## Mixed Reference and Inline Footnotes

Both styles render into the same footnote section at the end of the document.

A reference footnote[^mixed1] and an inline footnote^[This inline footnote and the reference footnote below both appear in the same rendered footnotes section.] in the same paragraph.

[^mixed1]: Reference footnote in mixed example.

---

## Practical Examples

### Academic Writing

The theory was first proposed in 1905.[^einstein]

Later work confirmed the prediction experimentally.[^confirmation]

[^einstein]: Einstein, A. (1905). *Zur Elektrodynamik bewegter Körper*. Annalen der Physik, 17(10), 891-921.
[^confirmation]: Eddington, A. S. (1920). *Space, Time and Gravitation*. Cambridge University Press.

### Technical Documentation

The default cache size is 128 entries.[^cache-note]

Increasing it beyond 512 has diminishing returns.[^bench]

[^cache-note]: Configurable via `cache_size` in `settings.ron`.
[^bench]: Based on internal benchmarks with typical Markdown documents (1-50 KB).
