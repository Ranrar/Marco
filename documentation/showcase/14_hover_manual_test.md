# Hover Manual Test Document

Use this file to verify editor hover popovers. Move the mouse over the marked targets.

---

## 1) Heading hover

Hover this heading line itself.

## 2) Inline formatting

Hover each of these:

- *emphasis*
- **strong**
- ***strong emphasis***
- ~~strikethrough~~
- ==mark/highlight==
- x^superscript^
- H~subscript~

## 3) Links and images

Hover these:

- Link: [Marco Project](https://example.com/marco "Project title")
- Image: ![A colorful test image](https://example.com/image.png)

## 4) Code span and code block

Inline code span: `let x = 42;`

```rust
fn main() {
    println!("hover this code block");
}
```

## 5) Inline HTML

Hover this inline HTML: <span class="hover-test">inline html content for hover preview 🎨</span>

## 6) Hard break and soft break

This line ends with two spaces for a hard break.  
This next line should be treated as following a hard break.

This line is a normal line
with a soft line break between lines.

## 7) Thematic break

Hover the rule below:

---

## 8) Blockquote

> This is a blockquote.
> Hover anywhere inside this quote block.

## 9) Mixed nested case

**[Nested link in strong](https://example.com/nested "Nested title")**

(Expectation: hovering inside link text should prefer link hover.)

---

If hover still does not appear:
1. Save file.
2. Move mouse over the exact target tokens (link text, inline code, heading text, etc.).
3. Pause briefly (~350ms) without moving.
