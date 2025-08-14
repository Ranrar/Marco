# Markdown Syntax Validation Using AST + Syntax Mapping

## Overview

This guide explains how to use the **AST** and **syntax mapping** to validate Markdown syntax in a document. This allows the editor to **detect errors**, **highlight them**, and optionally **suggest corrections**, while remaining **variant-aware**.

---

## Folder Structure

The validation system uses the same structure as auto-close:

```
markdown_defs/
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

* `ast.ron` defines **Markdown nodes** and their hierarchy.
* `syntax.ron` defines **how nodes map to literal Markdown syntax**.
* Validation uses both to check that the document follows the expected structure.

---

## Step 1: Parse Document into AST

* Use a Markdown parser such as [`pulldown-cmark`](https://docs.rs/pulldown-cmark/) or [`comrak`](https://docs.rs/comrak/).
* Generate an AST for the document.
* Example AST node for a heading:

```ron
Heading { type: "heading", depth: 2, children: [ TextNode { type: "text", value: "Subheading" } ] }
```

---

## Step 2: Load Syntax Mapping

* Syntax mapping tells you the expected Markdown syntax for each node type.
* Example mapping:

```ron
Heading2 { node_type: "heading", depth: 2, markdown_syntax: "##" }
Strong { node_type: "strong", markdown_syntax: "**" }
Emphasis { node_type: "emphasis", markdown_syntax: "*" }
```

* This allows validation logic to know what the **correct syntax** should be.

---

## Step 3: Validate Each Node

1. Traverse the AST line by line or node by node.
2. For each node:

   * Find the **expected syntax** in the mapping.
   * Compare it to the **actual text** in the document.
   * If it doesn’t match, mark it as an error.

### Example Checks

#### Heading

```text
AST Node: Heading, depth=2
Expected Syntax: ##
Actual Text: ### Wrong Heading
Result: Error – expected "##"
```

#### Bold Text

```text
AST Node: Strong
Expected Syntax: **
Actual Text: *bold*
Result: Error – expected "**"
```

#### Inline Code

```text
AST Node: InlineCode
Expected Syntax: `
Actual Text: `fn main()`
Result: Valid
```

---

## Step 4: Highlight and Correct Errors

* Use the editor to **highlight invalid lines**.
* Show a **tooltip or footer message** with node type, expected syntax, and error.
* Optionally, offer **auto-correction** using the syntax mapping:

```text
Auto-Fix Example:
*bold* → **bold**
```

* Combine with **auto-close logic** to prevent many errors in real-time.

---

## Step 5: Benefits

* **Accurate Node-Level Validation:** Checks structure for headings, lists, emphasis, links, code, and more.
* **Variant-Specific:** Works for CommonMark, GFM, Pandoc, or custom Markdown variants.
* **Active-Line Feedback:** Can be integrated with the footer to show live validation info.
* **Supports Auto-Fix:** Ensures Markdown is consistent and correctly formatted.

---

## Step 6: Integration with Editor

1. Parse the document continuously or on-demand.
2. Traverse the AST and validate nodes using syntax mapping.
3. Highlight errors and optionally suggest fixes.
4. Update footer with **active node type**, **expected syntax**, and **content**.

Example footer output for an error line:

```
Node: Heading (Level 2)
Expected Syntax: ##
Actual Text: ###
Content: Wrong Heading
Error: Syntax mismatch
```

---

✅ **Conclusion**

Using AST + syntax mapping for validation allows your editor to:

* Detect Markdown syntax errors precisely at the node level.
* Support multiple Markdown flavors.
* Work seamlessly with auto-close and active-line footer features.