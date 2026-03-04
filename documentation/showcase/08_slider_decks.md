# Slider Decks

Marco's slider extension lets you author interactive slideshows directly in Markdown. The preview renders them with navigation controls and an optional autoplay timer.

## Syntax Overview

| Element | Syntax | Description |
|---------|--------|-------------|
| Start deck | `@slidestart` or `@slidestart:tN` | Opens a slide deck; `:tN` sets autoplay (seconds) |
| Horizontal break | `---` | New slide |
| Vertical marker | `--` | Metadata marker for vertical context |
| End deck | `@slideend` | Closes the deck |

---

## Basic Slide Deck

@slidestart

## Welcome to Marco Slides

Use the arrow buttons or dots to navigate.

Every slide supports **full Markdown** — emphasis, lists, code, tables, math.

---

## Lists on Slides

Key features of Marco's parser:

- Hand-crafted **nom**-based grammar
- 100% CommonMark compliance
- Multiple extension layers (GFM, Marco)
- Native Rust — no Node.js or browser required

---

## Code on a Slide

```rust
fn main() {
    let markdown = "# Hello, *Marco*!";
    let doc = core::parser::parse(markdown).unwrap();
    let html = core::render::render(&doc, &Default::default()).unwrap();
    println!("{}", html);
}
```

---

## Tables on a Slide

| Extension | Syntax | Output |
|-----------|--------|--------|
| Mark | `==text==` | ==highlighted== |
| Superscript | `^text^` | x^2^ |
| Subscript | `~text~` | H~2~O |

---

## Math on a Slide

Euler's identity: $e^{i\pi} + 1 = 0$

$$
\int_{-\infty}^{\infty} e^{-x^2}\, dx = \sqrt{\pi}
$$

@slideend

---

## Slide Deck with Autoplay

The `:tN` option after `@slidestart` enables autoplay at N-second intervals. The deck loops.

@slidestart:t4

## Slide 1 — Auto-advances in 4 seconds

This deck loops automatically.

---

## Slide 2 — Project Goals

- Fast native rendering
- Offline, privacy-first
- Extensible Markdown grammar

---

## Slide 3 — Tech Stack

Built with:
- Rust + GTK4
- nom (parser combinators)
- KaTeX (math)
- Mermaid (diagrams)

@slideend
---

## Code Blocks Inside Slides are Safe

Fenced code blocks that happen to contain `---`, `--`, or `@slidestart`/`@slideend` are **not** treated as slide syntax:

@slidestart

## Slide with a Safe Code Block

The following code block contains slider syntax but must not trigger it:

```text
@slidestart:t3
---
--
@slideend
```

The renderer correctly ignores these inside fences.

@slideend

---

## Practical Presentation Example

@slidestart

## API Documentation

**Service:** User Management API  
**Version:** 2.4.0  
**Base URL:** `https://api.example.com/v2`

---

## Authentication

All requests require a Bearer token:

```http
GET /users HTTP/1.1
Host: api.example.com
Authorization: Bearer <your-token>
```

---

## GET /users

Returns a paginated list of users.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | int | 1 | Page number |
| `limit` | int | 20 | Items per page |

---

## Response Format

```json
{
  "users": [
    { "id": 1, "name": "Alice", "email": "alice@example.com" },
    { "id": 2, "name": "Bob",   "email": "bob@example.com" }
  ],
  "total": 142,
  "page": 1,
  "limit": 20
}
```

---

## Error Codes

| Code | Meaning |
|------|---------|
| 200 | Success |
| 401 | Unauthorized |
| 404 | Not found |
| 429 | Rate limited |

> [!WARNING]
> Rate limiting applies: 100 requests per minute per token.

@slideend
