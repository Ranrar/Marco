# Disable autolink literals inside code spans showcase

Planned acceptance example:

- `` `http://www.example.com` `` should render as inline code and must not become a link.

## Canonical examples

`http://www.example.com`
`https://example.com/path?query=value`
`www.example.com`
`user@example.com`

## Edge cases

### Backticks inside code spans

`` `code with `backtick` inside` ``

### Code spans next to text

Use `https://example.com` in examples.

### Outside code spans (may linkify if autolink literals are enabled)

https://example.com
www.example.com
user@example.com
