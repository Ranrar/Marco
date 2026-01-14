# Extended heading IDs showcase

Planned syntax example (Markdown Guide extended syntax):

- `### Title {#custom-id}`

This showcase is meant for **visual verification** in the preview:

- Headings that successfully parse an ID will render `id="..."` in HTML.
- When an `id` exists, Marco also injects a small **anchor icon** (visible on hover/focus).
- You can click the anchor icon or any of the links below to jump to the section.

## Quick links (click to jump) {#quick-links}

- [Canonical: custom-id](#custom-id)
- [Canonical: another-id](#another-id)
- [Characters: dash/underscore/dot](#id-with-dash_underscore.and.dot)
- [Numbers: starts-with-number](#123-starts-with-number)
- [Unicode: café-résumé](#café-résumé)
- [Duplicate: dup](#dup)
- [Invalid: Space before id { #bad } (should not jump)](#bad)
- [Invalid: Title {#id} trailing text (should not jump)](#id)

## Canonical examples

### Title {#custom-id}

Try:

- [Jump to custom-id](#custom-id)

## Another title {#another-id}

Try:

- [Jump to another-id](#another-id)

## ID character coverage

### Mixed characters {#id-with-dash_underscore.and.dot}

Try:

- [Jump to mixed characters](#id-with-dash_underscore.and.dot)

### Starts with numbers {#123-starts-with-number}

Try:

- [Jump to 123-starts-with-number](#123-starts-with-number)

## Edge cases

### Whitespace

These should **not** be recognized as heading IDs and should remain as literal text.

### Space before id { #bad }

Try (should not jump anywhere useful):

- [Jump to #bad](#bad)

### Unicode and punctuation

### Café résumé {#café-résumé}

Try:

- [Jump to café-résumé](#café-résumé)

### Trailing spaces after the ID

Depending on implementation details, this may or may not be treated as a valid ID.
If it is valid, the anchor icon should appear; if not, the `{#...}` stays in the title.

### Trailing spaces example {#trailing-spaces}    

Try:

- [Jump to trailing-spaces](#trailing-spaces)

### Duplicate IDs (how conflicts are handled is a design choice)

## Duplicate {#dup}

## Duplicate again {#dup}

Try:

- [Jump to dup](#dup) (will typically land on the first matching `id`)

### Not a heading ID

#### In code span: `### Title {#custom-id}`

#### Missing closing brace {#broken

#### Not at end of heading

### Title {#id} trailing text

Try (should not jump anywhere useful):

- [Jump to #id](#id)

### Multiple braces at end (should not parse)

If your parser requires the ID marker to be the final token, this should remain literal.

### Title {#one}{#two}
