# Footnotes showcase

Planned feature: GitHub-style footnotes (`[^1]` + `[^1]: ...`).

## Canonical examples

Here is a footnote reference[^1].

[^1]: Footnote definition.

## Multiple references + multiple definitions

Two refs: one[^a] and two[^b].

[^a]: First definition.
[^b]: Second definition with **formatting** and `code`.

## Edge cases

### Forward definition

Ref before definition[^forward].

[^forward]: Defined later.

### Multi-line definitions

A multi-line footnote[^multi].

[^multi]: First line
    second line (indented)
    third line

### Unicode labels

Unicode label[^参考].

[^参考]: Unicode definition.

### Missing definition

Missing def[^missing].
