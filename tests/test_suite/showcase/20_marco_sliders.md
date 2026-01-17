# Marco Sliders (marco_sliders)

This showcase demonstrates Marco's slideshow extension.

- `@slidestart[:tN]` begins a slide deck.
- `---` splits slides (horizontal).
- `--` marks a vertical split (metadata only for now).
- `@slideend` ends the deck.

@slidestart:t5

## Slide 1

Welcome to **Marco Sliders**.

- Use the arrows to navigate
- Use the dots to jump to a slide
- If a timer is provided, autoplay loops

---

## Slide 2

Inline formatting works: `code`, *emphasis*, **strong**, and links: <https://github.com>.

A table inside a slide:

| a | b |
|---|---|
| 1 | 2 |

--

## Slide 3 (vertical marker)

The `--` separator is recorded as metadata on the slide.

A fenced code block should NOT be treated as slide syntax:

```text
---
--
@slidestart:t1
@slideend
```

@slideend
