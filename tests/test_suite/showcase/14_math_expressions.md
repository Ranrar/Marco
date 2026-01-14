# Math expressions showcase

Planned feature (match GitHub Markdown):
- Inline math: `$...$`
- Block math: `$$...$$` on its own line
- Optional: fenced ` ```math ` blocks

## Canonical examples

Inline: $x^2 + y^2 = z^2$

$$
\int_0^\infty e^{-x^2} \, dx = \frac{\sqrt{\pi}}{2}
$$

```math
E = mc^2
```

## Edge cases

### Dollar escaping

Inside math: $\$100$ (literal dollar inside math)
Outside math: This costs \$5.

### Literal dollars near math (GitHub guidance)

Use <span>$</span> to show a dollar sign without starting math.

### Backticks to avoid delimiter ambiguity

GitHub documents an alternative inline form when backticks help:

$`\text{literal $ inside}`$

### Not math

`$x$` inside code spans must stay literal.

A price like $5 should not necessarily start math (ambiguous; expected behavior should be decided).
