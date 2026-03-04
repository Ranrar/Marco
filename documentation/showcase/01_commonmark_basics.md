# CommonMark Basics

Marco is 100% CommonMark compliant (652/652 spec tests). This file demonstrates all core Markdown features.

---

## Headings

# Heading 1
## Heading 2
### Heading 3
#### Heading 4
##### Heading 5
###### Heading 6

Setext-style headings also work:

Level 1 Heading
===============

Level 2 Heading
---------------

---

## Paragraphs and Line Breaks

This is a paragraph with multiple sentences. Sentences flow together naturally across line boundaries in the source.

This is a second paragraph — separated by a blank line.

This line ends with two spaces  
which creates a hard line break.

A backslash at the end also works:\
This is on a new line.

---

## Emphasis

*This text is italic* and _so is this_.

**This text is bold** and __so is this__.

***This text is bold and italic*** and ___so is this___.

You can also combine: **bold with *nested italic* inside** or *italic with **nested bold** inside*.

---

## Blockquotes

> This is a blockquote.
> It can span multiple lines.

> Blockquotes can be nested:
>
> > This is a nested blockquote.
> >
> > > And even deeper.

> Blockquotes can contain other elements:
>
> - A list item
> - Another item
>
> And a paragraph.

---

## Lists

### Unordered Lists

- Item one
- Item two
  - Nested item
  - Another nested item
    - Doubly nested
- Item three

Asterisks and plus signs also work:

* Item with asterisk
+ Item with plus

### Ordered Lists

1. First item
2. Second item
   1. Nested ordered item
   2. Another nested ordered item
3. Third item

Numbers don't need to be sequential — Marco follows CommonMark and renumbers:

1. First
5. Still second
3. Still third

### Task Lists

- [x] Completed task
- [ ] Incomplete task
- [x] Another completed task

---

## Code

### Inline Code

Use backticks for `inline code`, like `cargo build` or a variable `let x = 42`.

Backtick strings handle internal backticks: `` use `code` here ``

### Fenced Code Blocks

```rust
fn main() {
    println!("Hello, Marco!");
}
```

```python
def greet(name: str) -> str:
    return f"Hello, {name}!"

print(greet("Marco"))
```

```bash
cargo build --release
cargo test --workspace
```

### Indented Code Blocks

    This is an indented code block.
    Four spaces or one tab prefixes each line.

---

## Links

[Marco on GitHub](https://github.com/Ranrar/Marco)

[Link with title](https://github.com/Ranrar/Marco "Marco Markdown Editor")

Autolinks: <https://github.com/Ranrar/Marco> and <user@example.com>

---

## Images

![Marco Logo](https://raw.githubusercontent.com/Ranrar/marco/refs/heads/main/documentation/user_guide/Logo_marco_and_polo.png)

Images with alt text and titles:

![Alt text](https://via.placeholder.com/200x100 "Image title")

---

## Thematic Breaks

Three or more hyphens, asterisks, or underscores on their own line:

---

***

___

---

## HTML Blocks

Raw HTML is passed through:

<details>
<summary>Click to expand</summary>

This content is hidden until expanded.

</details>

Inline HTML also works: <kbd>Ctrl</kbd>+<kbd>S</kbd> to save.

---

## Entities and Special Characters

HTML entities: &copy; &amp; &lt; &gt; &mdash; &ndash; &hellip;

Numeric references: &#169; &#38; &#8212;

Backslash escapes: \* \_ \` \# \[ \] \( \) \{ \} \+ \- \. \! \|

---

## Reference-Style Links

Reference links keep prose clean when URLs are long.

[Marco][marco-link] is a Markdown editor.
[Polo][polo-link] is its companion viewer.
[CommonMark][cm] defines the base spec.

[marco-link]: https://github.com/Ranrar/Marco
[polo-link]: https://github.com/Ranrar/Marco
[cm]: https://commonmark.org

Shortcut references also work: [marco-link]

---

## Inline HTML in Paragraphs

Text with <strong>strong HTML</strong> and <em>emphasized HTML</em>.

A <abbr title="HyperText Markup Language">HTML</abbr> abbreviation.
