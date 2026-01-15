# Link handling showcase (internal/external/edge cases)

This file is meant for **manual preview testing** of Marco/Polo link handling:
- internal relative files resolve against *this file's folder*
- missing local targets should appear **red + strikethrough** and show a tooltip
- external links should prompt a confirmation dialog before opening

---

## Internal anchors (same document)

Jump to a heading that exists:
- [Go to "Anchor target A"](#anchor-target-a)

Jump to an anchor that does *not* exist (should be blocked if anchor validation is enabled):
- [Go to missing anchor](#this-anchor-does-not-exist)

Fragment-only edge cases:
- [Just a hash](#)
- [Hash with spaces](#anchor%20with%20spaces)

### Anchor target A {#anchor-target-a}

This is the target.

### Anchor with spaces {#anchor with spaces}

This heading uses an extended id with spaces.

---

## Internal markdown files (relative to this folder)

These should open inside the app:
- [Linked doc](files/linked_doc.md)
- [Linked doc → anchor](files/linked_doc.md#linked-section)
- [Nested doc](files/subdir/nested.md)
- [Nested doc → anchor](files/subdir/nested.md#nested-anchor)

Windows-style separators (expected to be treated as a *literal* URL by the renderer; useful to see how it gets classified):
- [Backslashes](files\\linked_doc.md)

Spaces + percent encoding:
- [Space in filename (raw)](files/space name.md)
- [Space in filename (encoded)](files/space%20name.md)

Unicode filename:
- [Unicode filename](files/unicode-✓.md)

Path traversal / escape attempts:
- [Workspace README (escape up)](../../../README.md)

Unsupported file types (should be blocked by the decision engine):
- [PDF (unsupported)](files/unsupported.pdf)

Missing file targets (should show missing styling in preview):
- [Missing markdown](files/DOES_NOT_EXIST.md)
- [Missing image](files/img/DOES_NOT_EXIST.png)

---

## Images (internal file URLs in markdown)

These are regular images (not click targets by default, but good for base-uri testing):

![PNG](files/img/test.png)
![JPG](files/img/test.jpg)
![SVG](files/img/test.svg)

---

## External links (should require confirmation)

- https: [Example](https://example.com)
- http: [Example](http://example.com)
- www shorthand: [Example](www.example.com)
- mailto: [Email](mailto:test@example.com)
- tel: [Phone](tel:+1-555-0100)

With query + fragment:
- [Example query+hash](https://example.com/path?x=1&y=two#frag)

Autolink forms:
- <https://example.com>
- <mailto:test@example.com>

---

## Potentially dangerous / should be blocked

These should not be opened:
- [javascript URL](javascript:alert('nope'))
- [data URL](data:text/html,<h1>nope</h1>)

---

## HTML anchors (WebKit should still route through the same decision engine)

<a href="https://example.com">External via HTML &lt;a&gt;</a>

<a href="files/linked_doc.md">Internal via HTML &lt;a&gt;</a>

<a href="files/DOES_NOT_EXIST.md">Missing internal via HTML &lt;a&gt;</a>

---

## Reference-style links

[ref-internal]: files/linked_doc.md
[ref-missing]: files/DOES_NOT_EXIST.md
[ref-external]: https://example.com

- [Reference internal][ref-internal]
- [Reference missing][ref-missing]
- [Reference external][ref-external]
