# GFM task lists showcase

This file collects the task list examples referenced in the roadmap/tracker and adds a few edge cases.

## Canonical syntax (GFM)

- [ ] Unchecked task
- [x] Checked task (lowercase x)
- [X] Checked task (uppercase X)

## Ordered task lists

1. [ ] First
2. [x] Second
3. [ ] Third

## Checklist-style paragraphs (no list marker)

These are commonly written as hard-break-separated lines.

[ ] Line 1
[x] Line 2
[X] Line 3

## Edge cases

### Spacing variations

- [ ]  Two spaces after the marker
- [ ]	Tab after the marker
- [ ]
  Continuation line in the same list item

### Nested lists

- [ ] Parent task
  - [ ] Nested task
  - Not a task

### Not-a-task lookalikes

- [] Missing space inside brackets (should not be a task)
- [x ] Extra space (should not be a task)
- [xx] Too many characters (should not be a task)
- [ ]Not a task (no space after marker)

### Inline occurrences

Inline task markers are supported mid-paragraph:

This [x] is a checked inline task marker.

This [ ] is an unchecked inline task marker.

Punctuation adjacency should work: done? [x], nice.

This should stay a link (not a checkbox): [x](https://example.com)

`[x]` inside code spans must stay literal.
