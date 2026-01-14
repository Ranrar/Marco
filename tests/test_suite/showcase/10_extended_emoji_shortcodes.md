# Extended emoji shortcodes showcase

Implemented transform: recognized `:shortcode:` → Unicode emoji.

Notes:
- Only *recognized* shortcodes are converted.
- Unknown shortcodes stay literal (e.g. `:unknown:`).
- Code spans are parsed before emoji shortcodes, so ```:joy:``` stays code.

## Canonical examples

:joy:
:smile:
:rocket:
:+1:

Also supported (non-exhaustive):

:heart: :tada: :fire: :eyes: :bug: :memo: :coffee: :x: :warning: :info:

## Edge cases

### Unknown shortcode (should stay literal)

:not_a_real_emoji:

### Mid-text splitting (should convert in the middle)

Hello :joy: world
Hi :rocket:!

### Inside code spans (must stay literal)

`:joy:`

```
:joy: inside a fenced code block should also stay literal
```

### Adjacent punctuation

Great job: :tada:!
Parentheses (:smile:)

### Inside emphasis

*Emphasis :smile: emphasis*

### Inside links (design choice)

[:rocket: launch](https://example.com)
