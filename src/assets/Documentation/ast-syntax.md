# Active-Line Markdown Syntax Trace Setup

## Overview

This guide explains how to set up a **Markdown AST + syntax mapping system** for an **active-line footer** in a Markdown editor. This allows the footer to show the **type of node**, its **Markdown syntax**, and the **content** of the currently active line.

---

## Folder Structure

Organize your project like this:

```
markdown_schema/
├─ commonmark/
│  ├─ ast.ron
│  └─ syntax.ron
├─ gfm/
│  ├─ ast.ron
│  └─ syntax.ron
├─ pandoc/
│  ├─ ast.ron
│  └─ syntax.ron
```

* **AST files** define the document structure for each Markdown variant.
* **Syntax mapping files** define how AST nodes map to literal Markdown syntax.

---

## Step 1: Create AST Template (`ast.ron`)

Example for **CommonMark**:

```ron
RootNode {
    type: "root",
    children: [
        Heading { type: "heading", depth: 1, children: [ TextNode { type: "text", value: "Heading 1" } ] },
        Paragraph { type: "paragraph", children: [ TextNode { type: "text", value: "Sample paragraph" } ] },
        List { type: "list", ordered: false, children: [
            ListItem { type: "listItem", children: [ TextNode { type: "text", value: "First item" } ] },
            ListItem { type: "listItem", children: [ TextNode { type: "text", value: "Second item" } ] }
        ] },
        CodeBlock { type: "codeBlock", language: Some("rust"), value: "fn main() {}" },
        InlineCode { type: "inlineCode", value: "let x = 5;" }
    ]
}
```

* Each node represents a **Markdown element** (heading, paragraph, list, code block, etc.)
* You can extend this AST for **GFM** (tables, task lists) or **Pandoc** (footnotes, math).

---

## Step 2: Create Syntax Mapping (`syntax.ron`)

Example mapping for **CommonMark**:

````ron
Heading1 { node_type: "heading", depth: 1, markdown_syntax: "#" }
Heading2 { node_type: "heading", depth: 2, markdown_syntax: "##" }
Paragraph { node_type: "paragraph", markdown_syntax: "" }
Strong { node_type: "strong", markdown_syntax: "**" }
Emphasis { node_type: "emphasis", markdown_syntax: "*" }
BlockQuote { node_type: "blockquote", markdown_syntax: ">" }
ListUnordered { node_type: "list", ordered: false, markdown_syntax: "-" }
ListOrdered { node_type: "list", ordered: true, markdown_syntax: "1." }
ListItem { node_type: "listItem", markdown_syntax: "-" }
CodeBlock { node_type: "codeBlock", markdown_syntax: "```" }
InlineCode { node_type: "inlineCode", markdown_syntax: "`" }
HorizontalRule { node_type: "thematicBreak", markdown_syntax: "---" }
Link { node_type: "link", markdown_syntax: "[text](url)" }
Image { node_type: "image", markdown_syntax: "![alt](url)" }
````

* Links the **AST node type** to the actual Markdown syntax.
* Used by the footer to show what syntax the active line represents.

---

## Step 3: Integrate Into Editor

1. **Parse Markdown**

   * Use a Rust parser like [`pulldown-cmark`](https://docs.rs/pulldown-cmark/) or [`comrak`](https://docs.rs/comrak/) to generate an AST.

2. **Track Active Line**

   * When the cursor moves, identify the **AST node** corresponding to that line.

3. **Display in Footer**

   * Show:

     ```
     Node: Heading (Level 1)
     Syntax: #
     Content: Heading 1
     ```
   * Update dynamically as the user moves between lines.

---

## Step 4: Supporting Multiple Markdown Variants

* You can have variant-specific AST and syntax mapping files:

  ```
  markdown_defs/gfm/ast.ron
  markdown_defs/gfm/syntax.ron
  ```
* Supports GFM features like **tables, task lists, strikethrough**.
* Pandoc variant can include **footnotes, citations, math blocks**.

**Benefits:**

* Separation of concerns (structure vs syntax)
* Extensible for new Markdown flavors
* Easier maintenance and testing

---

## Step 5: Tips & Best Practices

1. **Do not hardcode syntax detection** in the editor. Use AST + syntax mapping.
2. **Keep AST and syntax mapping in separate files** for clarity.
3. **Sync mapping entries** with AST nodes to avoid mismatches.
4. **Reuse syntax mapping** for nodes shared between variants.

---

## Example Footer Output

If the active line contains:

```
## Subheading
```

Footer displays:

```
Node: Heading (Level 2)
Syntax: ##
Content: Subheading
```