# Links, Heading IDs, and Autolinks

---

## Inline Links

The basic link form: `[text](url)` and `[text](url "title")`.

[Marco on GitHub](https://github.com/Ranrar/Marco)

[Marco with title](https://github.com/Ranrar/Marco "Marco Markdown Editor on GitHub")

Links work with formatted text: [**Bold link**](https://github.com/Ranrar/Marco) and [*italic link*](https://commonmark.org).

---

## Reference-Style Links

Reference links separate the URL from the prose, keeping long links out of the way.

Marco is built with [Rust][rust] and [GTK4][gtk4]. The parser uses [nom][nom] combinators.

The [CommonMark spec][commonmark] defines the baseline. [GFM][gfm] extends it with tables and task lists.

[rust]: https://www.rust-lang.org
[gtk4]: https://gtk.org
[nom]: https://github.com/rust-bakery/nom
[commonmark]: https://commonmark.org
[gfm]: https://github.github.com/gfm/

### Shortcut and Collapsed References

[commonmark] ← shortcut reference (label used as text)

[Marco][marco-link] ← full reference

[marco-link][] ← collapsed reference (link text = label)

[marco-link]: https://github.com/Ranrar/Marco

### Forward References

Forward-defined reference links work too — the definition can appear after the use:

See [the spec][spec-link] for details.

[spec-link]: https://spec.commonmark.org

---

## CommonMark Autolinks

Angle-bracket autolinks convert to clickable links:

<https://github.com/Ranrar/Marco>

<mailto:user@example.com> or simply <user@example.com>

---

## GFM Autolink Literals

Bare URLs and emails (no angle brackets needed):

https://github.com/Ranrar/Marco

http://commonmark.org/help/

www.rust-lang.org

contact@example.com

Autolinks work mid-sentence: Visit https://github.com/Ranrar/Marco for the source.

Multiple in a row: https://rust-lang.org and https://gtk.org are both great projects.

Trailing punctuation is excluded from the link:

- https://example.com. ← the period is not part of the link
- https://example.com, ← the comma is not part of the link
- (https://example.com) ← parentheses balance correctly

---

## Extended Heading IDs {#heading-ids}

Append `{#custom-id}` to a heading to give it a stable anchor.

### Installation {#installation}

### Configuration {#configuration}

### Advanced Usage {#advanced-usage}

You can then link to specific headings by ID:

- [Jump to Installation](#installation)
- [Jump to Configuration](#configuration)
- [Jump to Advanced Usage](#advanced-usage)

Without explicit IDs, Marco generates slugs from heading text automatically.

---

## Internal Links

Link to headings within the same document using auto-generated slugs or explicit IDs:

[Back to the top of this page](#links-heading-ids-and-autolinks)

[Jump to Reference-Style Links section](#reference-style-links)

---

## Internal File Links

Link to other Markdown files in the same project. Marco resolves them relative to the current file.

[View the linked document](files/linked_doc.md)

[Jump to a section inside the linked document](files/linked_doc.md#linked-section)

---

## Images as Links

Wrap an image in a link to make it clickable:

[![Marco Logo](https://raw.githubusercontent.com/Ranrar/marco/refs/heads/main/documentation/user_guide/Logo_marco_and_polo.png)](https://github.com/Ranrar/Marco)

---

## Link Titles

Titles appear as tooltips on hover:

[CommonMark](https://commonmark.org "The CommonMark Markdown specification")

[Rust Language](https://www.rust-lang.org "A language empowering everyone to build reliable and efficient software")

---

## Link Edge Cases

Empty link text: [](https://example.com)

Link containing code: [`cargo build`](https://doc.rust-lang.org/cargo/)

Link with special characters: [100% CommonMark](https://spec.commonmark.org)

Nested brackets in link text: [Marco [editor]](https://github.com/Ranrar/Marco)

---

## External Links (hover to see URL in footer)

Hover over each link below — the URL appears in the footer status bar.

### HTTPS / HTTP

[GitHub](https://github.com)

[Rust Language](https://www.rust-lang.org)

[GTK Project](https://gtk.org)

[CommonMark Spec](https://spec.commonmark.org)

[crates.io](https://crates.io)

[docs.rs](https://docs.rs)

### Mailto

[Contact example](mailto:hello@example.com)

### Long URL (ellipsis test)

[Very long path](https://www.rust-lang.org/learn/get-started#installing-rust-on-linux-or-macos-with-rustup)

---

## Local File Links (hover to see path in footer)

These link to files on disk relative to this document.

[Showcase: Tables and Task Lists](02_tables_and_task_lists.md)

[Showcase: Footnotes](09_footnotes.md)

[Showcase: Table of Contents](16_table_of_contents.md)

[Linked document (files/)](files/linked_doc.md)

[Linked document — specific section](files/linked_doc.md#linked-section)
