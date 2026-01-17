# GFM alerts/admonitions showcase

This file exercises GitHub-style alerts (admonitions) derived from blockquotes.

Reference syntax: `> [!NOTE]` (and TIP / IMPORTANT / WARNING / CAUTION).

## Canonical examples

> [!NOTE]
> A note.

> [!TIP]
> A tip.

> [!IMPORTANT]
> Important information.

> [!WARNING]
> A warning.

> [!CAUTION]
> A caution.

## Marker + content in a single paragraph

GitHub often treats these as a single paragraph with a soft break.

> [!NOTE]
> First line
> second line continues the same paragraph

## Custom-header (quote-style) extension

This repo also supports a neutral, blockquote-colored variant that keeps the
admonition title layout but uses a custom icon + title.

> [:joy: Happy Header]
> This should render as an admonition with a custom emoji icon and title,  
> but with quote-like (neutral) colors.

## Edge cases

### Case-insensitive marker

> [!note]
> Lowercase kind.

> [!Warning]
> Mixed case kind.

### Unknown marker (should remain a normal blockquote)

> [!UNKNOWN]
> This should not become an admonition.

### Marker-only paragraph

> [!NOTE]
>
> Body starts after a blank line.

### Marker with trailing whitespace

> [!NOTE]   
> Trailing spaces after marker.

### Nested blockquote (alerts cannot be nested on GitHub)

> [!NOTE]
> Outer
> > [!TIP]
> > Inner (should not be transformed if nesting is disallowed)

### Admonition nested inside a list (top-level constraint)

- Item
  > [!NOTE]
  > Nested inside list (should not be transformed if only top-level is allowed)
