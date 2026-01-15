# Footnotes showcase

Supported:

- GitHub-style footnotes (`[^1]` + `[^1]: ...`).
- Marco inline footnotes (`^[...]`).

## Canonical examples

Here is a footnote reference[^1].

[^1]: Footnote definition.

## Inline footnotes (Marco extension)

Inline footnote right here.^[This content is defined at the reference point.]

Complex inline footnote.^[*italic*, **bold**, and `code`]

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
