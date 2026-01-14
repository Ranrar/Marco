# Extended emoji shortcodes showcase

Planned optional transform: `:joy:` → 😀/😂 (or equivalent).

## Canonical examples

:joy:
:smile:
:rocket:
:+1:

## Edge cases

### Unknown shortcode (should stay literal)

:not_a_real_emoji:

### Inside code spans (must stay literal)

`:joy:`

### Adjacent punctuation

Great job: :tada:!
Parentheses (:smile:)

### Inside emphasis

*Emphasis :smile: emphasis*

### Inside links (design choice)

[:rocket: launch](https://example.com)
