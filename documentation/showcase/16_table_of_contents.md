# Table of Contents

Marco can automatically generate and update a **Table of Contents** for any document.
Use **Modules → Insert / Update TOC** to insert or refresh the TOC at the cursor position.

---

## How It Works

The TOC is a Markdown list wrapped in two HTML comment markers:

```markdown
<!-- TOC -->
- [Section One](#section-one)
- [Section Two](#section-two)
  - [Sub-section](#sub-section)

<!-- /TOC -->
```

- `<!-- TOC -->` marks the start of the generated block.
- `<!-- /TOC -->` marks the end.
- The list items are standard Markdown links — they work in any renderer.
- The markers themselves are invisible HTML comments; they do not appear in the preview.

---

## Inserting a TOC

1. Open a document that has headings.
2. Place the cursor where you want the TOC (usually near the top, after the title).
3. Choose **Modules → Insert / Update TOC** from the menu.

Marco parses the document, collects all headings, and inserts the TOC block at the cursor.

---

## Updating an Existing TOC

If the document already contains a `<!-- TOC -->` / `<!-- /TOC -->` block, the same
**Insert / Update TOC** action replaces the existing block with a freshly generated one.
You can invoke it anytime after adding, renaming, or reordering headings.

---

## Heading Slugs

Marco derives anchor IDs from heading text using the GitHub slug algorithm:

| Heading text | Generated slug |
|---|---|
| `Getting Started` | `getting-started` |
| `FAQ & Troubleshooting` | `faq-troubleshooting` |
| `Step 1: Install` | `step-1-install` |
| `C++ Overview` | `c-overview` |

Rules:
- Text is lowercased.
- Letters and digits are kept as-is.
- Spaces, hyphens, and underscores become `-`.
- All other characters are dropped.
- Consecutive hyphens are collapsed to one; leading and trailing hyphens are removed.

---

## Explicit Heading IDs

If a heading uses a custom ID with the `{#id}` syntax, that ID takes priority over the
auto-generated slug in the TOC link:

```markdown
## Deployment Guide {#deploy}
```

The TOC entry will link to `#deploy`, not `#deployment-guide`.

---

## Duplicate Headings

When two or more headings produce the same slug, Marco appends a numeric suffix to keep
anchors unique — identical to GitHub's behaviour:

| Heading | Slug |
|---|---|
| `## Introduction` | `introduction` |
| `## Introduction` | `introduction-1` |
| `## Introduction` | `introduction-2` |

---

## Nesting and Indentation

The TOC list is indented relative to the **minimum** heading level present in the document.
If the shallowest heading in a document is `##`, that level gets zero indentation; `###`
gets one level of indentation (two spaces), and so on.

```markdown
<!-- TOC -->
- [Overview](#overview)
  - [Background](#background)
  - [Goals](#goals)
- [Installation](#installation)
  - [Linux](#linux)
  - [Windows](#windows)
    - [Prerequisites](#prerequisites)
- [Usage](#usage)

<!-- /TOC -->
```

---

## Live Example

The TOC below was generated from the headings in this very file.

<!-- TOC -->
- [Table of Contents](#table-of-contents)
- [How It Works](#how-it-works)
- [Inserting a TOC](#inserting-a-toc)
- [Updating an Existing TOC](#updating-an-existing-toc)
- [Heading Slugs](#heading-slugs)
- [Explicit Heading IDs](#explicit-heading-ids)
- [Duplicate Headings](#duplicate-headings)
- [Nesting and Indentation](#nesting-and-indentation)
- [Live Example](#live-example)
- [All Six Heading Levels](#all-six-heading-levels)
  - [H2 — Chapter](#h2-chapter)
    - [H3 — Section](#h3-section)
      - [H4 — Subsection](#h4-subsection)
        - [H5 — Detail](#h5-detail)
          - [H6 — Fine print](#h6-fine-print)

<!-- /TOC -->

---

## All Six Heading Levels

This section exercises every heading level so you can verify the TOC panel renders
and navigates all six depths correctly.

## H2 — Chapter

H2 is the main chapter level. Use it for top-level sections after the document title.

### H3 — Section

H3 groups related topics within a chapter.

#### H4 — Subsection

H4 breaks a section into focused sub-topics.

##### H5 — Detail

H5 is rarely needed but available for deeply nested reference material.

###### H6 — Fine print

H6 is the deepest heading level. The TOC panel respects the configurable **depth** setting
(1-6) in *Settings → Layout*, so you can hide H5/H6 entries when they clutter the panel.
