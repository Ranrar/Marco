#!/usr/bin/env python3
"""
Generate a large stress-test document for Marco / Polo.
Produces ~10 000 sections cycling through every Marco markdown feature.

Usage:
    python3 generate_stresstest.py > stresstest.md
    python3 generate_stresstest.py --output documentation/stresstest.md
"""

import argparse
import sys

# ---------------------------------------------------------------------------
# Feature-section generators
# Each generator takes the section number (1-based) and returns a string.
# ---------------------------------------------------------------------------

def sec_headings(n):
    return f"""\
# Section {n} — Headings

## H2 heading in section {n}

### H3 heading in section {n}

#### H4 heading in section {n}

##### H5 heading in section {n}

###### H6 heading in section {n}

Setext H1 in section {n}
=========================

Setext H2 in section {n}
-------------------------

This paragraph follows the heading block for section {n}.

"""

def sec_emphasis(n):
    return f"""\
# Section {n} — Emphasis and Inline Formatting

Regular paragraph in section {n} with *italic*, **bold**, ***bold-italic***, and ~~strikethrough~~ text.

You can combine: **bold with *nested italic* inside** and *italic with **nested bold** inside*.

Inline `code span` and a hard line break  
right here in section {n}.

A backslash line break:\
continues on the next line in section {n}.

"""

def sec_blockquote(n):
    return f"""\
# Section {n} — Blockquotes

> This is a simple blockquote in section {n}.
> It continues across multiple source lines.

> Nested blockquotes:
>
> > Level two in section {n}.
> >
> > > Level three in section {n}.

> A blockquote with **bold**, *italic*, and `code` in section {n}.

"""

def sec_lists(n):
    return f"""\
# Section {n} — Lists

Unordered list:

- Item alpha in section {n}
- Item beta in section {n}
  - Nested item one
  - Nested item two
    - Deeply nested in section {n}
- Item gamma in section {n}

Ordered list:

1. First item in section {n}
2. Second item in section {n}
   1. Sub-item A
   2. Sub-item B
3. Third item in section {n}

Mixed nesting:

- Outer A in section {n}
  1. Inner ordered one
  2. Inner ordered two
- Outer B in section {n}

"""

def sec_tasklist(n):
    return f"""\
# Section {n} — Task Lists

- [x] Completed task in section {n}
- [ ] Pending task in section {n}
- [x] Another completed task
- [ ] Another pending task
- [x] All core features implemented
- [ ] Performance profiling for section {n}

"""

def sec_code(n):
    langs = [
        ("rust",       f"fn task_{n}(x: u64) -> u64 {{\n    x.wrapping_mul({n})\n}}"),
        ("python",     f"def task_{n}(x: int) -> int:\n    return x * {n}"),
        ("javascript", f"function task{n}(x) {{\n  return x * {n};\n}}"),
        ("bash",       f"#!/usr/bin/env bash\necho \"Section {n}: $(date)\""),
        ("toml",       f"[section_{n}]\nvalue = {n}\nenabled = true"),
    ]
    lang, code = langs[n % len(langs)]
    return f"""\
# Section {n} — Code Blocks

Fenced code block with `{lang}`:

```{lang}
{code}
```

Indented code block (4-space indent):

    // indented code in section {n}
    let x = {n};

Inline code: `let result_{n} = compute({n});`

"""

def sec_table(n):
    return f"""\
# Section {n} — Tables

| Column A | Column B | Column C |
|----------|----------|----------|
| Row 1 A in section {n} | Row 1 B | Row 1 C |
| Row 2 A | Row 2 B in section {n} | Row 2 C |
| Row 3 A | Row 3 B | Row 3 C in section {n} |

Alignment variants:

| Left-aligned | Centered | Right-aligned |
|:-------------|:--------:|--------------:|
| `{n}` left | `{n}` center | `{n}` right |
| alpha | beta | gamma |
| delta | epsilon | zeta |

"""

def sec_admonition(n):
    types = ["NOTE", "TIP", "IMPORTANT", "WARNING", "CAUTION"]
    kind = types[n % len(types)]
    return f"""\
# Section {n} — Admonitions

> [!{kind}]
> This is a **{kind}** admonition in section {n}.
> It contains *italic text*, `inline code`, and a list:
>
> - Item one for section {n}
> - Item two for section {n}
> - Item three

"""

def sec_math(n):
    return f"""\
# Section {n} — Math (KaTeX)

Inline math: The formula for section {n} is $f(x) = x^{{{n}}} + \\sqrt{{{n}}}$.

Euler's identity: $e^{{i\\pi}} + 1 = 0$ (section {n}).

Display math:

$$
\\int_{{0}}^{{{n}}} x^2 \\, dx = \\frac{{{n}^3}}{{3}}
$$

$$
\\sum_{{k=1}}^{{{n}}} k = \\frac{{{n}({n}+1)}}{{2}}
$$

"""

def sec_mermaid(n):
    diagrams = [
        # flowchart
        f"""\
```mermaid
graph TD
    A{n}[Start {n}] --> B{n}{{Check {n}?}}
    B{n} -- Yes --> C{n}[Pass]
    B{n} -- No --> D{n}[Fail]
    D{n} --> E{n}[Retry {n}]
    E{n} --> B{n}
```""",
        # sequence
        f"""\
```mermaid
sequenceDiagram
    participant U{n} as User {n}
    participant S{n} as Server {n}
    U{n}->>S{n}: Request section {n}
    S{n}-->>U{n}: Response {n}
    U{n}->>S{n}: Acknowledge {n}
```""",
    ]
    diagram = diagrams[n % len(diagrams)]
    return f"""\
# Section {n} — Mermaid Diagrams

{diagram}

"""

def sec_tab_block(n):
    return f"""\
# Section {n} — Tab Blocks

:::tab
@tab Overview {n}
This is the overview panel for section {n}. It contains **bold text** and *italic text*.

- Feature A in section {n}
- Feature B in section {n}

@tab Code {n}
```rust
fn section_{n}() -> &'static str {{
    "hello from section {n}"
}}
```

@tab Notes {n}
> [!TIP]
> Tab blocks in section {n} use pure CSS switching — no JavaScript needed.
:::

"""

def sec_slider(n):
    return f"""\
# Section {n} — Slider Deck

@slidestart

## Slide 1 of section {n}

Welcome to slide deck **{n}**. Use arrow buttons to navigate.

---

## Slide 2 of section {n}

Key points:

- Point A for deck {n}
- Point B for deck {n}
- Point C for deck {n}

---

## Slide 3 of section {n}

```rust
fn slide_{n}() {{
    println!("Slide {n}");
}}
```

@slideend

"""

def sec_footnotes(n):
    return f"""\
# Section {n} — Footnotes

This sentence has a reference footnote.[^fn{n}a]

Another sentence uses a named footnote.[^fn{n}b]

An inline footnote right here.^[This is an inline footnote for section {n} — no separate definition needed.]

[^fn{n}a]: This is footnote A for section {n}.
[^fn{n}b]: This is footnote B for section {n}. It provides extra context.

"""

def sec_definition_list(n):
    return f"""\
# Section {n} — Definition Lists

Term {n} Alpha
: Definition of term {n} alpha — the first and primary meaning.
: A second definition providing additional context for section {n}.

Term {n} Beta
: Single definition for term {n} beta.

**Formatted term {n}**
: Definition with *italic* and `code` inside for section {n}.

"""

def sec_links(n):
    return f"""\
# Section {n} — Links and Images

Inline link: [CommonMark spec](https://spec.commonmark.org) in section {n}.

Reference-style link: [Marco repo][marco-{n}]

[marco-{n}]: https://github.com/Ranrar/Marco

Autolink: <https://example.com/section/{n}>

Autolink email: <user{n}@example.com>

Image (alt text only, no external load):

![Placeholder image for section {n}](https://placehold.co/400x200?text=Section+{n})

"""

def sec_horizontal_rule(n):
    hr_styles = ["---", "***", "___"]
    hr = hr_styles[n % len(hr_styles)]
    return f"""\
# Section {n} — Horizontal Rules and Separators

Paragraph before the rule in section {n}.

{hr}

Paragraph after the first rule in section {n}.

---

Paragraph after the second rule.

***

Final paragraph in the HR section {n}.

"""

def sec_nested_structures(n):
    return f"""\
# Section {n} — Nested Structures

Blockquote containing a list:

> Blockquote in section {n}:
>
> - Item one
> - Item two
>   - Nested item
> - Item three

List containing a blockquote:

- Outer item in section {n}

  > Blockquote inside list item for section {n}.

- Second outer item

  ```rust
  fn nested_{n}() {{}}
  ```

"""

def sec_html_entities(n):
    return f"""\
# Section {n} — Special Characters and Escapes

Escaped Markdown punctuation: \\# \\* \\_ \\[ \\] \\( \\) \\` \\~ in section {n}.

HTML entities: &amp; &lt; &gt; &quot; &copy; &reg; &trade; &mdash; &ndash; &hellip;

Unicode directly: — – … → ← ↑ ↓ ↔ ✓ ✗ ★ ☆ in section {n}.

"""

def sec_mixed_inline(n):
    return f"""\
# Section {n} — Mixed Inline Elements

A paragraph combining **bold** and *italic* with `code`, a [link](https://example.com), ~~strikethrough~~, and math $x={n}$ all in one line of section {n}.

> A blockquote also combining **bold**, *italic*, `code`, and $\\pi \\approx 3.14159$ in section {n}.

| Inline | Sample for section {n} |
|--------|------------------------|
| Bold | **bold text** |
| Italic | *italic text* |
| Code | `code span` |
| Math | $e={n}$ |
| Strike | ~~crossed out~~ |

"""

def sec_long_paragraph(n):
    # Generate a longer prose paragraph to stress paragraph rendering
    sentences = " ".join([
        f"This is sentence {s} of the long paragraph in section {n}." for s in range(1, 16)
    ])
    return f"""\
# Section {n} — Long Paragraph

{sentences}

Another paragraph follows with **bold phrases** and *italic phrases* interspersed at regular intervals throughout the text to ensure inline parsing is exercised under load in section {n}. The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs. How vividly the quartz sphinx jumped.

"""

# ---------------------------------------------------------------------------
# Ordered list of generators (cycles through these for 10 000 sections)
# ---------------------------------------------------------------------------
GENERATORS = [
    sec_headings,
    sec_emphasis,
    sec_blockquote,
    sec_lists,
    sec_tasklist,
    sec_code,
    sec_table,
    sec_admonition,
    sec_math,
    sec_mermaid,
    sec_tab_block,
    sec_slider,
    sec_footnotes,
    sec_definition_list,
    sec_links,
    sec_horizontal_rule,
    sec_nested_structures,
    sec_html_entities,
    sec_mixed_inline,
    sec_long_paragraph,
]

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="Generate Marco stress-test document")
    parser.add_argument("--sections", type=int, default=10_000,
                        help="Number of sections to generate (default: 10000)")
    parser.add_argument("--output", "-o", default=None,
                        help="Output file path (default: stdout)")
    args = parser.parse_args()

    out = open(args.output, "w", encoding="utf-8") if args.output else sys.stdout

    # Document header
    out.write(f"""\
---
title: Marco Stress Test — {args.sections} Sections
description: Comprehensive stress-test document covering every Marco/Polo markdown feature.
---

# Marco Stress Test Document

This document contains **{args.sections} sections**, cycling through every Markdown
feature supported by Marco and Polo. It is designed to exercise the parser, renderer,
and preview engine under realistic load.

Feature types covered (cycling in order):

| # | Feature |
|---|---------|
| 1 | Headings (h1–h6, setext) |
| 2 | Emphasis (italic, bold, bold-italic, strikethrough) |
| 3 | Blockquotes (nested) |
| 4 | Lists (ordered, unordered, nested) |
| 5 | Task lists |
| 6 | Code blocks (fenced + indented, 5 languages) |
| 7 | Tables (with alignment) |
| 8 | Admonitions (NOTE/TIP/IMPORTANT/WARNING/CAUTION) |
| 9 | Math — KaTeX (inline + display) |
| 10 | Mermaid diagrams (flowchart + sequence) |
| 11 | Tab blocks |
| 12 | Slider decks |
| 13 | Footnotes (reference + inline) |
| 14 | Definition lists |
| 15 | Links and images |
| 16 | Horizontal rules |
| 17 | Nested structures |
| 18 | HTML entities + escapes |
| 19 | Mixed inline elements |
| 20 | Long paragraphs |

---

""")

    n_types = len(GENERATORS)
    for i in range(1, args.sections + 1):
        gen = GENERATORS[(i - 1) % n_types]
        out.write(gen(i))

    if args.output:
        out.close()
        print(f"Generated {args.sections} sections → {args.output}", file=sys.stderr)

if __name__ == "__main__":
    main()
