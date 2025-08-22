# Creating a Custom Markdown Flavor for Marco

This document explains how to define your **AST**, **syntax rules**, and **Pest grammar** to build a custom Markdown parser in Rust.

You’ll learn how to structure the files and how they work together.

---

## 1️⃣ AST: `ast.ron`

The **AST (Abstract Syntax Tree)** represents the *semantic structure* of a Markdown document. It abstracts away syntax details like `#` or `*`.

### Example

```ron
RootNode {
    type: "root",
    children: [
        Heading {
            type: "heading",
            depth: 1,
            children: [
                TextNode { type: "text", value: "Heading 1" }
            ]
        },
        Paragraph {
            type: "paragraph",
            children: [
                TextNode { type: "text", value: "This is a paragraph." }
            ]
        },
        List {
            type: "list",
            ordered: false,
            children: [
                ListItem {
                    type: "listItem",
                    children: [
                        Paragraph {
                            type: "paragraph",
                            children: [
                                TextNode { type: "text", value: "First item" }
                            ]
                        }
                    ]
                }
            ]
        }
    ]
}
```

**Notes:**

* Each node represents a semantic element (`Heading`, `Paragraph`, `ListItem`).
* Inline elements like `Strong` or `Emphasis` are children of `Paragraph`.
* The AST should not care about syntax details (`#`, `-`, `*`).

---

## 2️⃣ Syntax rules: `syntax.ron`

The **syntax file** defines **allowed node types and rules**, useful for validation or extensions.

### Example

```ron
RootNode {
    allowed_children: ["Heading", "Paragraph", "List", "BlockQuote", "CodeBlock", "Table", "Link", "Image"]
}

Heading {
    allowed_children: ["TextNode", "Strong", "Emphasis"]
    allowed_depths: [1, 2, 3, 4, 5, 6]
}

Paragraph {
    allowed_children: ["TextNode", "Strong", "Emphasis", "InlineCode", "Strikethrough"]
}

List {
    ordered_allowed: [true, false]
    allowed_children: ["ListItem"]
}

ListItem {
    allowed_children: ["Paragraph", "List"]
}
```

**Notes:**

* This is mostly **meta-data**, used to **validate AST structures**.
* Developers can extend this when adding **new custom elements**.

---

## 3️⃣ Grammar file: `markdown.pest`

The **Pest grammar** defines the **syntax** of your Markdown flavor. It maps text to AST nodes.

### Example

```pest
WHITESPACE = _{ " " | "\t" }
NEWLINE    = _{ "\n" | "\r\n" }
TEXT       = { (!NEWLINE ~ ANY)+ }

document   = { SOI ~ block* ~ EOI }
block      = { heading | paragraph | list }

# -------------------
# Headings
# -------------------
heading    = { atx_heading }
atx_heading = { ("######" | "#####" | "####" | "###" | "##" | "#") ~ WHITESPACE ~ TEXT }

# -------------------
# Paragraphs & inline
# -------------------
paragraph  = { (!NEWLINE ~ ANY)+ }
strong     = { "**" ~ (!"**" ~ ANY)+ ~ "**" }
emphasis   = { "*" ~ (!"*" ~ ANY)+ ~ "*" }
inlinecode = { "`" ~ (!"`" ~ ANY)+ ~ "`" }

# -------------------
# Lists
# -------------------
list       = { (unordered_list)+ }
unordered_list = { (list_item ~ NEWLINE)+ }
list_item  = { ("-" | "*" | "+") ~ WHITESPACE ~ TEXT }
```

**Notes:**

* AST nodes like `Heading` and `Paragraph` correspond to grammar rules.
* Inline elements like `Strong` and `Emphasis` are handled inside paragraphs.
* You can extend this grammar to add **custom blocks**, **task lists**, or **tables**.

---

## 4️⃣ Workflow

1. **Define your AST** (`ast.ron`) → Decide semantic nodes.
2. **Define syntax rules** (`syntax.ron`) → Define allowed children, attributes, validations.
3. **Define grammar** (`markdown.pest`) → Map text to AST nodes.
4. **Parse text** → Use Pest to create a parse tree.
5. **Build AST** → Walk the parse tree and construct your AST nodes.
6. **Validate AST** → Check against `syntax.ron` rules.
7. **Render / process** → Generate HTML, PDF, or custom output.

---

## 5️⃣ Adding Your Own Markdown Flavor

To extend the language:

1. Add new AST nodes (e.g., `Admonition`, `VideoEmbed`).
2. Add corresponding syntax rules in `syntax.ron`.
3. Add grammar rules in `markdown.pest` for the text format.
4. Update the parser to build AST nodes from new grammar rules.
5. Optionally, update the renderer to handle new node types.

---

### ✅ Tips

* Keep AST **semantic**, not syntactic.
* Keep grammar **concrete**, i.e., it knows about symbols (`#`, `*`, etc.).
* Use `syntax.ron` to enforce **valid structures** without touching parser code.
* Test with small Markdown samples to validate your flavor.

---

This setup gives a **clean separation**:

| File            | Role                           |
| --------------- | ------------------------------ |
| `ast.ron`       | Semantic model of Markdown     |
| `syntax.ron`    | Rules and validation for AST   |
| `markdown.pest` | Maps text → AST (parser rules) |